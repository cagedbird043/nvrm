//! Codec2 driver state machine. No C2 objects cross this layer.
use super::ffi::*;
use super::ffi_dec::*;
use std::{
    collections::VecDeque,
    ffi::CStr,
    os::fd::{FromRawFd, OwnedFd},
};

pub(super) trait Backend {
    fn decode(&mut self, data: &[u8], index: u64, flags: u32) -> i32;
    fn next(&mut self, out: &mut NvVideoPicture) -> i32;
    fn hdr(&mut self, id: u64, out: &mut NvVideoHdrMetadata) -> i32;
    fn copy(&mut self, id: u64, target: &NvVideoTarget, fence: &mut i32) -> i32;
    fn release(&mut self, id: u64) -> i32;
    fn flush(&mut self) -> i32;
    fn required(&mut self) -> Result<(u32, u32), i32>;
    fn reopen(&mut self, cfg: &NvC2DecConfig, csd: &[u8]) -> Result<(), String>;
}

pub(super) fn last_error() -> String {
    // SAFETY: the low-level ABI owns a thread-local NUL-terminated string.
    unsafe { CStr::from_ptr(nv_video_last_error()) }
        .to_string_lossy()
        .into_owned()
}
pub(super) struct Native {
    device: *mut NvVideoDevice,
    session: *mut NvVideoSession,
}
impl Native {
    pub fn open(
        device: *mut NvVideoDevice,
        cfg: &NvC2DecConfig,
        csd: &[u8],
    ) -> Result<Self, String> {
        let mut n = Self {
            device,
            session: std::ptr::null_mut(),
        };
        n.reopen(cfg, csd)?;
        Ok(n)
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        unsafe { nv_video_session_destroy(self.session) }
    }
}
impl Backend for Native {
    fn decode(&mut self, d: &[u8], i: u64, f: u32) -> i32 {
        unsafe { nv_video_decode(self.session, d.as_ptr(), d.len(), i, f) }
    }
    fn next(&mut self, p: &mut NvVideoPicture) -> i32 {
        unsafe { nv_video_next_picture(self.session, p) }
    }
    fn hdr(&mut self, i: u64, h: &mut NvVideoHdrMetadata) -> i32 {
        unsafe { nv_video_picture_hdr(self.session, i, h) }
    }
    fn copy(&mut self, i: u64, t: &NvVideoTarget, f: &mut i32) -> i32 {
        unsafe { nv_video_copy_out(self.session, i, t, f) }
    }
    fn release(&mut self, i: u64) -> i32 {
        unsafe { nv_video_picture_release(self.session, i) }
    }
    fn flush(&mut self) -> i32 {
        unsafe { nv_video_flush(self.session) }
    }
    fn required(&mut self) -> Result<(u32, u32), i32> {
        let (mut w, mut h) = (0, 0);
        let rc = unsafe { nv_video_required_size(self.session, &mut w, &mut h) };
        if rc < 0 {
            Err(rc)
        } else {
            Ok((w, h))
        }
    }
    fn reopen(&mut self, c: &NvC2DecConfig, csd: &[u8]) -> Result<(), String> {
        unsafe { nv_video_session_destroy(self.session) };
        self.session = std::ptr::null_mut();
        let mut caps = super::NvVideoCaps::default();
        let rc = unsafe { nv_video_query_caps(self.device, c.codec, &mut caps) };
        if rc < 0 {
            return Err(last_error());
        }
        validate_caps(c, &caps)?;
        let info = NvVideoSessionInfo {
            codec: c.codec,
            max_width: c.max_width,
            max_height: c.max_height,
            bit_depth: c.bit_depth,
            extra_output_slots: c.extra_output_slots,
            codec_config: csd.as_ptr(),
            codec_config_len: csd.len(),
            reserved: [0; 8],
        };
        let rc = unsafe { nv_video_session_create(self.device, &info, &mut self.session) };
        if rc < 0 || self.session.is_null() {
            Err(last_error())
        } else {
            Ok(())
        }
    }
}
fn validate_caps(c: &NvC2DecConfig, caps: &super::NvVideoCaps) -> Result<(), String> {
    if caps.supported == 0 {
        return Err("codec not supported by GPU".into());
    }
    if c.max_width < caps.min_width
        || c.max_height < caps.min_height
        || c.max_width > caps.max_width
        || c.max_height > caps.max_height
    {
        return Err("requested dimensions outside GPU caps".into());
    }
    if c.bit_depth >= 32 || caps.bit_depth_mask & (1 << c.bit_depth) == 0 {
        return Err("bit depth not supported".into());
    }
    Ok(())
}

// Closing is centralized; acquire fds and unpolled release fds are driver-owned.
pub(super) fn close_fd(fd: i32) {
    if fd >= 0 {
        drop(unsafe { OwnedFd::from_raw_fd(fd) });
    }
}
#[derive(Clone, Copy)]
struct Pending {
    index: u64,
    remaining: u32,
}

pub struct NvC2Dec {
    backend: Box<dyn Backend>,
    cfg: NvC2DecConfig,
    startup: Vec<u8>,
    cb: NvC2DecCallbacks,
    pending: VecDeque<Pending>,
    events: VecDeque<NvC2DecEvent>,
    held: Option<NvVideoPicture>,
    eos: Option<(u64, bool, Option<NvC2DecEvent>)>,
    resize: Option<(u32, u32)>,
    csd: Vec<Vec<u8>>,
    csd_bytes: u64,
    last_was_csd: bool,
    display: (u32, u32),
    pub(super) failure: Option<(i32, String)>,
    retry: Option<(u64, u32, usize)>,
    ended: bool,
}
// SAFETY: all state, backend RM objects and callback context are used serially,
// as required by the ABI. No concurrent reference is exposed.
unsafe impl Send for NvC2Dec {}
impl NvC2Dec {
    pub(super) fn new(
        backend: Box<dyn Backend>,
        mut cfg: NvC2DecConfig,
        startup: Vec<u8>,
        cb: NvC2DecCallbacks,
    ) -> Self {
        cfg.codec_config = std::ptr::null();
        cfg.codec_config_len = 0;
        Self {
            backend,
            cfg,
            startup,
            cb,
            pending: VecDeque::new(),
            events: VecDeque::new(),
            held: None,
            eos: None,
            resize: None,
            csd: Vec::new(),
            csd_bytes: 0,
            last_was_csd: false,
            display: (0, 0),
            failure: None,
            retry: None,
            ended: false,
        }
    }
    fn reorders(&self) -> bool {
        self.cfg.codec == 1 || self.cfg.codec == 2
    }
    pub(super) fn fail(&mut self, cause: i32, why: String) -> i32 {
        if self.failure.is_none() {
            self.failure = Some((cause, why));
        }
        FATAL
    }
    fn low_fail(&mut self, what: &str, rc: i32) -> i32 {
        self.fail(FATAL, format!("{what}: {rc}: {}", last_error()))
    }
    fn empty(&mut self, index: u64, corrupt: bool) {
        self.events.push_back(NvC2DecEvent {
            index,
            corrupt: u32::from(corrupt),
            ..NvC2DecEvent::empty()
        });
    }
    fn finish_all(&mut self) {
        while let Some(p) = self.pending.pop_front() {
            if self.eos.as_ref().is_some_and(|e| e.0 == p.index) {
                continue;
            }
            self.empty(p.index, false);
        }
    }
    fn finish_eos(&mut self) {
        if let Some((index, _, event)) = self.eos.take() {
            let mut e = event.unwrap_or_else(NvC2DecEvent::empty);
            e.index = index;
            e.eos = 1;
            self.events.push_back(e);
            self.ended = true;
        }
    }
    fn dispose(&self, e: NvC2DecEvent) {
        close_fd(e.release_fence_fd);
        if e.cookie != 0 {
            unsafe { (self.cb.discard.unwrap())(self.cb.ctx, e.cookie) }
        }
    }
    fn emit(&mut self, p: NvVideoPicture) -> Result<bool, i32> {
        let fmt = if p.bit_depth > 8 { 2 } else { 1 };
        let mut b = NvC2DecBuffer::default();
        b.target.acquire_fence_fd = -1;
        let rc = unsafe {
            (self.cb.fetch.unwrap())(self.cb.ctx, p.crop_width, p.crop_height, fmt, &mut b)
        };
        if rc == 1 {
            self.held = Some(p);
            return Ok(false);
        }
        if rc != 0 {
            self.backend.release(p.id);
            return Err(self.fail(FATAL, format!("output pool fetch failed: {rc}")));
        }
        // Guard the transferred acquire fd even on failures before copy_out.
        let acquire = if b.target.acquire_fence_fd >= 0 {
            Some(unsafe { OwnedFd::from_raw_fd(b.target.acquire_fence_fd) })
        } else {
            None
        };
        if b.target.format == 0 {
            b.target.format = fmt;
        }
        let mut hdr = NvVideoHdrMetadata::default();
        let mut fence = -1;
        let err =
            if b.cookie == 0 || (b.target.format != fmt && !(fmt == 1 && b.target.format == 3)) {
                Some(("output pool target/cookie", -1))
            } else {
                let rc = self.backend.hdr(p.id, &mut hdr);
                if rc < 0 {
                    Some(("picture HDR", rc))
                } else {
                    let rc = self.backend.copy(p.id, &b.target, &mut fence);
                    if rc < 0 {
                        Some(("copy out", rc))
                    } else {
                        None
                    }
                }
            };
        let error_text = err.as_ref().map(|_| last_error());
        drop(acquire);
        let released = self.backend.release(p.id);
        if let Some((what, rc)) = err {
            close_fd(fence);
            unsafe { (self.cb.discard.unwrap())(self.cb.ctx, b.cookie) };
            return Err(self.fail(FATAL, format!("{what}: {rc}: {}", error_text.unwrap())));
        }
        if released < 0 {
            close_fd(fence);
            unsafe { (self.cb.discard.unwrap())(self.cb.ctx, b.cookie) };
            return Err(self.low_fail("picture release", released));
        }
        if !self.reorders() {
            while self.pending.front().is_some_and(|q| q.index != p.timestamp) {
                let q = self.pending.pop_front().unwrap();
                self.empty(q.index, false);
            }
        }
        let Some(pos) = self.pending.iter().position(|q| q.index == p.timestamp) else {
            close_fd(fence);
            unsafe { (self.cb.discard.unwrap())(self.cb.ctx, b.cookie) };
            return Err(self.fail(FATAL, "picture has no pending work".into()));
        };
        let q = &mut self.pending[pos];
        if q.remaining == 0 {
            close_fd(fence);
            unsafe { (self.cb.discard.unwrap())(self.cb.ctx, b.cookie) };
            return Err(self.fail(FATAL, "picture count exceeded accepted count".into()));
        }
        q.remaining -= 1;
        let final_for_index = u32::from(q.remaining == 0);
        if final_for_index != 0 {
            self.pending.remove(pos);
        }
        let e = NvC2DecEvent {
            r#type: 1,
            empty: 0,
            corrupt: 0,
            eos: 0,
            index: p.timestamp,
            cookie: b.cookie,
            release_fence_fd: fence,
            size_changed: u32::from(self.display != (p.crop_width, p.crop_height)),
            final_for_index,
            reserved: 0,
            picture: p,
            hdr,
        };
        self.display = (p.crop_width, p.crop_height);
        if final_for_index != 0 && self.eos.as_ref().is_some_and(|q| q.0 == p.timestamp) {
            self.eos.as_mut().unwrap().2 = Some(e);
        } else {
            self.events.push_back(e);
        }
        Ok(true)
    }
    pub(super) fn drain(&mut self, r: &mut NvC2DecResult) -> i32 {
        if self.failure.is_some() {
            return FATAL;
        }
        loop {
            let p = if let Some(p) = self.held.take() {
                p
            } else {
                let mut p = NvVideoPicture::default();
                let rc = self.backend.next(&mut p);
                if rc < 0 {
                    return self.low_fail("next picture", rc);
                }
                if rc == 0 {
                    break;
                }
                p
            };
            match self.emit(p) {
                Ok(true) => (),
                Ok(false) => {
                    r.blocked = 1;
                    return OK;
                }
                Err(rc) => return rc,
            }
        }
        if self.eos.as_ref().is_some_and(|q| q.1) {
            self.finish_all();
            self.finish_eos();
        }
        OK
    }
    fn finish_resize(&mut self, r: &mut NvC2DecResult) -> i32 {
        let rc = self.drain(r);
        if rc < 0 || r.blocked != 0 {
            return rc;
        }
        self.finish_all();
        let (w, h) = self.resize.unwrap();
        self.cfg.max_width = self.cfg.max_width.max(w);
        self.cfg.max_height = self.cfg.max_height.max(h);
        if let Err(e) = self.backend.reopen(&self.cfg, &self.startup) {
            return self.fail(FATAL, format!("resize recreate: {e}"));
        }
        self.resize = None;
        for i in 0..self.csd.len() {
            let rc = self.backend.decode(&self.csd[i], 0, 0);
            if rc < 0 {
                return self.low_fail("resize CSD replay", rc);
            }
        }
        OK
    }
    pub(super) fn queue(&mut self, d: &[u8], index: u64, flags: u32, r: &mut NvC2DecResult) -> i32 {
        if self.failure.is_some() {
            return FATAL;
        }
        if self.ended {
            return INVALID;
        }
        if self.retry.is_some_and(|q| q != (index, flags, d.len())) {
            return INVALID;
        }
        r.consumed = 0;
        if self.resize.is_some() {
            let rc = self.finish_resize(r);
            if rc < 0 {
                return rc;
            }
            if r.blocked != 0 {
                return OK;
            }
        }
        let eos = flags & 1 != 0;
        if flags & 2 != 0 && !eos {
            if self.reorders() && !d.is_empty() {
                let current = if self.last_was_csd { self.csd_bytes } else { 0 };
                if check_csd(current, d.len() as u64) != OK {
                    return self.fail(LIMIT, "CSD replay run exceeds 2 MiB".into());
                }
                let rc = self.backend.decode(d, index, 0);
                if rc == -3 {
                    self.empty(index, true);
                    r.consumed = 1;
                    return OK;
                }
                if rc < 0 {
                    return self.low_fail("decode CSD", rc);
                }
                if !self.last_was_csd {
                    self.csd.clear();
                    self.csd_bytes = 0;
                }
                self.csd.push(d.to_vec());
                self.csd_bytes += d.len() as u64;
                self.last_was_csd = true;
            }
            self.empty(index, false);
            r.consumed = 1;
            self.retry = None;
            return OK;
        }
        self.last_was_csd = false;
        if eos && self.eos.is_none() {
            self.eos = Some((index, false, None));
        }
        loop {
            let rc = self.backend.decode(d, index, u32::from(eos));
            if rc == -6 {
                let before = self.events.len();
                let dr = self.drain(r);
                if dr < 0 {
                    return dr;
                }
                if r.blocked != 0 || self.events.len() == before {
                    if r.blocked == 0 {
                        r.blocked = 2;
                    }
                    self.retry = Some((index, flags, d.len()));
                    return OK;
                }
                continue;
            }
            if rc == -3 {
                if eos {
                    let rc = self.backend.decode(&[], index, 1);
                    if rc < 0 {
                        return self.low_fail("corrupt EOS drain", rc);
                    }
                    let mut event = NvC2DecEvent::empty();
                    event.index = index;
                    event.corrupt = 1;
                    self.eos = Some((index, true, Some(event)));
                } else {
                    self.empty(index, true);
                }
                break;
            }
            if rc == -7 {
                let size = match self.backend.required() {
                    Ok(v) => v,
                    Err(e) => return self.low_fail("required size", e),
                };
                let rc = self.backend.decode(&[], index, 1);
                if rc < 0 {
                    return self.low_fail("resize drain submit", rc);
                }
                self.resize = Some(size);
                let rc = self.finish_resize(r);
                if rc < 0 {
                    return rc;
                }
                if r.blocked != 0 {
                    self.retry = Some((index, flags, d.len()));
                    return OK;
                }
                continue;
            }
            if rc < 0 {
                return self.low_fail("decode", rc);
            }
            if !d.is_empty() {
                if rc > 0 || eos {
                    self.pending.push_back(Pending {
                        index,
                        remaining: rc as u32,
                    });
                } else {
                    self.empty(index, false);
                }
            }
            if eos {
                self.eos.as_mut().unwrap().1 = true;
            }
            break;
        }
        self.retry = None;
        r.consumed = 1;
        self.drain(r)
    }
    pub(super) fn flush(&mut self, r: &mut NvC2DecResult) -> i32 {
        if self.failure.is_some() {
            return FATAL;
        }
        if let Some(p) = self.held.take() {
            self.backend.release(p.id);
        }
        self.resize = None;
        self.retry = None;
        self.ended = false;
        let rc = self.backend.flush();
        self.finish_all();
        self.finish_eos();
        self.ended = false;
        r.blocked = 0;
        if rc < 0 {
            self.low_fail("flush", rc)
        } else {
            OK
        }
    }
    pub(super) fn poll(&mut self) -> Option<NvC2DecEvent> {
        self.events.pop_front()
    }
}
impl Drop for NvC2Dec {
    fn drop(&mut self) {
        if let Some(p) = self.held.take() {
            self.backend.release(p.id);
        }
        while let Some(e) = self.events.pop_front() {
            self.dispose(e);
        }
        if let Some((_, _, Some(e))) = self.eos.take() {
            self.dispose(e);
        }
    }
}

#[cfg(test)]
mod tests;
