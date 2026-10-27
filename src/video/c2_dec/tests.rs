use super::*;
use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    rc::Rc,
};
mod libc {
    pub const F_GETFD: i32 = 1;
    pub const EFD_CLOEXEC: i32 = 0x80000;
    unsafe extern "C" {
        pub fn eventfd(init: u32, flags: i32) -> i32;
        pub fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    }
}
// Another parallel test may reuse a closed fd number; compare eventfd identity,
// not just F_GETFD on that number.
fn fence_identity(fd: i32) -> Option<String> {
    std::fs::read_to_string(format!("/proc/self/fdinfo/{fd}"))
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("eventfd-id:"))
                .map(str::to_owned)
        })
}
fn fence_probe(fd: i32) -> (OwnedFd, String) {
    let keepalive = unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) }
        .try_clone_to_owned()
        .unwrap();
    (keepalive, fence_identity(fd).unwrap())
}
#[derive(Default)]
struct Control {
    calls: Vec<(Vec<u8>, u64, u32)>,
    reopens: Vec<(u32, u32, Vec<u8>)>,
    released: Vec<u64>,
    next_id: u64,
    copy_format: u32,
    fail: &'static str,
    acquire_seen: bool,
}
struct Fake {
    c: Rc<RefCell<Control>>,
    ready: VecDeque<NvVideoPicture>,
    reorder: Vec<NvVideoPicture>,
    codec: u32,
    max: (u32, u32),
    required: (u32, u32),
    key: bool,
    limit: usize,
}
impl Backend for Fake {
    fn decode(&mut self, d: &[u8], i: u64, f: u32) -> i32 {
        self.c.borrow_mut().calls.push((d.to_vec(), i, f));
        if self.c.borrow().fail == "decode" {
            return -5;
        }
        if self.c.borrow().fail == "again" {
            return -6;
        }
        assert!(self.c.borrow().fail != "panic", "injected decode panic");
        if self.c.borrow().fail == "replay" && d.first() == Some(&b'P') {
            return -5;
        }
        if !d.is_empty() && d[0] == b'P' {
            return 0;
        }
        if d.first() == Some(&255) {
            return -3;
        }
        if !d.is_empty() && self.ready.len() + self.reorder.len() >= self.limit {
            return -6;
        }
        let n = if d.is_empty() { 0 } else { d[0] };
        if n != 0 {
            let w = if d.len() > 3 {
                u16::from_le_bytes([d[2], d[3]]) as u32
            } else {
                640
            };
            let h = 360;
            if w > self.max.0 {
                if d.get(1) == Some(&1) {
                    self.required = (w, h);
                    return -7;
                } else {
                    return -2;
                }
            }
            if !self.key && d.get(1) != Some(&1) {
                return -3;
            }
            self.key = true;
            for _ in 0..n {
                self.c.borrow_mut().next_id += 1;
                let p = NvVideoPicture {
                    id: self.c.borrow().next_id,
                    timestamp: i,
                    width: w,
                    height: h,
                    crop_width: w,
                    crop_height: h,
                    bit_depth: if d.get(4) == Some(&10) { 10 } else { 8 },
                    reserved: [u32::from(d.get(5).copied().unwrap_or(i as u8)), 0, 0, 0, 0],
                    ..Default::default()
                };
                if self.codec <= 2 {
                    self.reorder.push(p);
                } else {
                    self.ready.push_back(p);
                }
            }
            if self.codec <= 2 && self.reorder.len() > 2 {
                self.reorder.sort_by_key(|p| p.reserved[0]);
                self.ready.push_back(self.reorder.remove(0));
            }
        }
        if f == 1 {
            self.reorder.sort_by_key(|p| p.reserved[0]);
            self.ready.extend(self.reorder.drain(..));
        }
        n as i32
    }
    fn next(&mut self, p: &mut NvVideoPicture) -> i32 {
        if self.c.borrow().fail == "next" {
            return -5;
        }
        if let Some(v) = self.ready.pop_front() {
            *p = v;
            1
        } else {
            0
        }
    }
    fn hdr(&mut self, _: u64, h: &mut NvVideoHdrMetadata) -> i32 {
        h.has_content_light_level = 1;
        h.max_content_light_level = 1000;
        if self.c.borrow().fail == "hdr" {
            -5
        } else {
            0
        }
    }
    fn copy(&mut self, _: u64, t: &NvVideoTarget, f: &mut i32) -> i32 {
        self.c.borrow_mut().copy_format = t.format;
        self.c.borrow_mut().acquire_seen = t.acquire_fence_fd < 0
            || unsafe { libc::fcntl(t.acquire_fence_fd, libc::F_GETFD) } >= 0;
        *f = unsafe { libc::eventfd(1, libc::EFD_CLOEXEC) };
        if self.c.borrow().fail == "copy" {
            -5
        } else {
            0
        }
    }
    fn release(&mut self, i: u64) -> i32 {
        self.c.borrow_mut().released.push(i);
        if self.c.borrow().fail == "release" {
            -5
        } else {
            0
        }
    }
    fn flush(&mut self) -> i32 {
        self.ready.clear();
        self.reorder.clear();
        self.key = false;
        if self.c.borrow().fail == "flush" {
            -5
        } else {
            0
        }
    }
    fn required(&mut self) -> Result<(u32, u32), i32> {
        Ok(self.required)
    }
    fn reopen(&mut self, c: &NvC2DecConfig, s: &[u8]) -> Result<(), String> {
        self.max = (c.max_width, c.max_height);
        self.c
            .borrow_mut()
            .reopens
            .push((c.max_width, c.max_height, s.to_vec()));
        if self.c.borrow().fail == "reopen" {
            Err("injected".into())
        } else {
            Ok(())
        }
    }
}
struct Pool {
    capacity: usize,
    busy: HashSet<u64>,
    next: u64,
    discarded: Vec<u64>,
    format: u32,
    acquire: i32,
    seen: Vec<(u32, u32, u32)>,
}
impl Pool {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            busy: HashSet::new(),
            next: 0,
            discarded: Vec::new(),
            format: 0,
            acquire: -1,
            seen: Vec::new(),
        }
    }
}
unsafe extern "C" fn fetch(
    ctx: *mut std::ffi::c_void,
    w: u32,
    h: u32,
    f: u32,
    b: *mut NvC2DecBuffer,
) -> i32 {
    let p = unsafe { &mut *ctx.cast::<Pool>() };
    if p.busy.len() == p.capacity {
        return 1;
    }
    p.next += 1;
    p.busy.insert(p.next);
    p.seen.push((w, h, f));
    unsafe {
        *b = NvC2DecBuffer {
            target: NvVideoTarget {
                format: p.format,
                acquire_fence_fd: p.acquire,
                ..Default::default()
            },
            cookie: p.next,
        }
    };
    p.acquire = -1;
    0
}
unsafe extern "C" fn discard(ctx: *mut std::ffi::c_void, c: u64) {
    let p = unsafe { &mut *ctx.cast::<Pool>() };
    assert!(p.busy.remove(&c));
    p.discarded.push(c);
}
fn config(codec: u32) -> NvC2DecConfig {
    NvC2DecConfig {
        abi_version: 1,
        codec,
        max_width: 1920,
        max_height: 1088,
        bit_depth: 8,
        extra_output_slots: 0,
        codec_config: std::ptr::null(),
        codec_config_len: 0,
        reserved: [0; 8],
    }
}
fn make(codec: u32, p: &mut Pool) -> (NvC2Dec, Rc<RefCell<Control>>) {
    let c = Rc::new(RefCell::new(Control::default()));
    let b = Fake {
        c: c.clone(),
        ready: VecDeque::new(),
        reorder: Vec::new(),
        codec,
        max: (1920, 1088),
        required: (0, 0),
        key: false,
        limit: 8,
    };
    (
        NvC2Dec::new(
            Box::new(b),
            config(codec),
            vec![1, 2, 3],
            NvC2DecCallbacks {
                ctx: (p as *mut Pool).cast(),
                fetch: Some(fetch),
                discard: Some(discard),
            },
        ),
        c,
    )
}
fn queue(d: &mut NvC2Dec, n: u8, index: u64, eos: bool) -> NvC2DecResult {
    let mut r = NvC2DecResult::default();
    let data = [n, 1];
    assert_eq!(
        d.queue(
            if n == 0 { &[] } else { &data },
            index,
            u32::from(eos),
            &mut r
        ),
        OK
    );
    r
}
fn events(d: &mut NvC2Dec, p: &mut Pool) -> Vec<NvC2DecEvent> {
    let mut v = Vec::new();
    while let Some(mut e) = d.poll() {
        close_fd(e.release_fence_fd);
        e.release_fence_fd = -1;
        if e.cookie != 0 {
            assert!(p.busy.remove(&e.cookie));
        }
        v.push(e);
    }
    v
}
#[test]
fn vp9_hidden_frames() {
    let mut p = Pool::new(8);
    let (mut d, _) = make(3, &mut p);
    queue(&mut d, 1, 0, false);
    assert_eq!(events(&mut d, &mut p)[0].empty, 0);
    let mut r = NvC2DecResult::default();
    assert_eq!(d.queue(&[0, 1], 1, 0, &mut r), OK);
    let e = events(&mut d, &mut p);
    assert_eq!((e[0].index, e[0].empty), (1, 1));
    queue(&mut d, 1, 2, false);
    queue(&mut d, 1, 3, false);
    queue(&mut d, 0, 4, true);
    let e = events(&mut d, &mut p);
    assert_eq!(e.iter().map(|e| e.index).collect::<Vec<_>>(), [2, 3, 4]);
    assert_eq!(e[2].eos, 1);
}
#[test]
fn h264_reorder_and_eos() {
    let mut p = Pool::new(8);
    let (mut d, _) = make(1, &mut p);
    for (i, poc) in [0, 3, 1, 2].into_iter().enumerate() {
        let mut result = NvC2DecResult::default();
        assert_eq!(
            d.queue(&[1, 1, 128, 2, 8, poc], i as u64, 0, &mut result),
            OK
        );
    }
    assert_eq!(
        events(&mut d, &mut p)
            .iter()
            .map(|e| e.index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    queue(&mut d, 1, 4, true);
    let e = events(&mut d, &mut p);
    assert_eq!(e.iter().map(|e| e.index).collect::<Vec<_>>(), [3, 1, 4]);
    assert_eq!(e.iter().map(|e| e.eos).collect::<Vec<_>>(), [0, 0, 1]);
}
#[test]
fn flush_seek() {
    let mut p = Pool::new(8);
    let (mut d, _) = make(2, &mut p);
    queue(&mut d, 1, 0, false);
    queue(&mut d, 1, 1, false);
    let mut r = NvC2DecResult::default();
    assert_eq!(d.flush(&mut r), OK);
    assert!(events(&mut d, &mut p).iter().all(|e| e.empty == 1));
    assert_eq!(d.queue(&[1, 0], 2, 0, &mut r), OK);
    assert_eq!(events(&mut d, &mut p)[0].corrupt, 1);
    queue(&mut d, 1, 3, false);
    queue(&mut d, 0, 4, true);
    assert_eq!(events(&mut d, &mut p).len(), 2);
}
#[test]
fn resolution_change_and_10bit() {
    let mut p = Pool::new(8);
    let (mut d, c) = make(4, &mut p);
    let mut r = NvC2DecResult::default();
    assert_eq!(d.queue(&[1, 1, 128, 2, 8], 0, 0, &mut r), OK);
    assert_eq!(d.queue(&[1, 1, 128, 2, 8], 1, 0, &mut r), OK);
    assert_eq!(d.queue(&[1, 1, 128, 7, 10], 2, 0, &mut r), OK);
    assert_eq!(
        events(&mut d, &mut p)
            .iter()
            .map(|e| e.size_changed)
            .collect::<Vec<_>>(),
        [1, 0, 1]
    );
    assert_eq!(c.borrow().copy_format, 2);
    assert_eq!(d.queue(&[1, 0, 0, 16, 8], 3, 0, &mut r), FATAL);
    assert_eq!(d.queue(&[1, 1], 4, 0, &mut r), FATAL);
}
#[test]
fn upswitch_recreates_session_and_replays_csd() {
    let mut p = Pool::new(8);
    let (mut d, c) = make(2, &mut p);
    let mut r = NvC2DecResult::default();
    assert_eq!(d.queue(b"P0", 0, 2, &mut r), OK);
    events(&mut d, &mut p);
    queue(&mut d, 1, 1, false);
    queue(&mut d, 1, 2, false);
    assert_eq!(d.queue(&[1, 1, 0, 15], 3, 0, &mut r), OK);
    let e = events(&mut d, &mut p);
    assert_eq!(e.iter().map(|e| e.index).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(c.borrow().reopens, [(3840, 1088, vec![1, 2, 3])]);
    assert!(c.borrow().calls.iter().any(|q| q.0 == b"P0" && q.1 == 0));
    queue(&mut d, 0, 4, true);
    assert_eq!(events(&mut d, &mut p).len(), 2);
}
#[test]
fn unit_without_picture_retires() {
    let mut p = Pool::new(8);
    let (mut d, _) = make(2, &mut p);
    queue(&mut d, 1, 0, false);
    let mut r = NvC2DecResult::default();
    assert_eq!(d.queue(&[0, 1], 1, 0, &mut r), OK);
    let e = events(&mut d, &mut p);
    assert_eq!((e[0].index, e[0].empty), (1, 1));
}
#[test]
fn hdr_metadata_on_output() {
    let mut p = Pool::new(8);
    let (mut d, _) = make(3, &mut p);
    queue(&mut d, 1, 0, false);
    assert_eq!(events(&mut d, &mut p)[0].hdr.max_content_light_level, 1000);
}
#[test]
fn hdr_units() {
    let mut h = NvC2DecHdrStatic::default();
    let mut m = NvVideoHdrMetadata::default();
    m.has_mastering_display = 1;
    m.mastering_display = NvVideoMasteringDisplay {
        display_primaries_x: [13250, 7500, 34000],
        display_primaries_y: [34500, 3000, 16000],
        white_point_x: 15635,
        white_point_y: 16450,
        max_luminance: 10000000,
        min_luminance: 1,
    };
    assert_eq!(unsafe { nv_c2_dec_hdr_static(2, &m, &mut h) }, OK);
    assert!((h.red[0] - 0.68).abs() < 0.0001);
    assert_eq!(h.max_luminance, 1000.);
    m.mastering_display = NvVideoMasteringDisplay {
        display_primaries_x: [44564, 17367, 9830],
        display_primaries_y: [20972, 45220, 3932],
        white_point_x: 20493,
        white_point_y: 21561,
        max_luminance: 256000,
        min_luminance: 2,
    };
    assert_eq!(unsafe { nv_c2_dec_hdr_static(4, &m, &mut h) }, OK);
    assert!(
        (h.red[0] - 0.68).abs() < 0.0001
            && (h.green[1] - 0.69).abs() < 0.0001
            && (h.blue[0] - 0.15).abs() < 0.0001
    );
    assert!((h.white[0] - 0.3127).abs() < 0.0001 && (h.min_luminance - 0.000122).abs() < 0.0001);
    assert_eq!(h.max_luminance, 1000.);
    m.has_mastering_display = 0;
    m.has_content_light_level = 1;
    m.max_content_light_level = 400;
    unsafe { nv_c2_dec_hdr_static(4, &m, &mut h) };
    assert_eq!(h.present, 1);
    assert_eq!(h.red, [0., 0.]);
    assert_eq!(h.max_cll, 400.);
    m.has_content_light_level = 0;
    unsafe { nv_c2_dec_hdr_static(4, &m, &mut h) };
    assert_eq!(h.present, 0);
}
#[test]
fn backpressure() {
    let mut p = Pool::new(1);
    let (mut d, _) = make(3, &mut p);
    queue(&mut d, 1, 0, false);
    let r = queue(&mut d, 1, 1, false);
    assert_eq!((r.consumed, r.blocked), (1, 1));
    let e = d.poll().unwrap();
    close_fd(e.release_fence_fd);
    p.busy.remove(&e.cookie);
    let mut r = NvC2DecResult::default();
    assert_eq!(d.drain(&mut r), OK);
    assert_eq!(events(&mut d, &mut p)[0].index, 1);
}
#[test]
fn device_lost_and_poll_after_error() {
    let mut p = Pool::new(8);
    let (mut d, c) = make(3, &mut p);
    queue(&mut d, 1, 0, false);
    c.borrow_mut().fail = "decode";
    let mut r = NvC2DecResult::default();
    assert_eq!(d.queue(&[1, 1], 1, 0, &mut r), FATAL);
    assert_eq!(d.drain(&mut r), FATAL);
    assert_eq!(events(&mut d, &mut p)[0].index, 0);
    assert!(d.poll().is_none());
}
#[test]
fn fence_passthrough_and_destroy_unpolled() {
    let mut p = Pool::new(8);
    let (mut d, _) = make(3, &mut p);
    queue(&mut d, 1, 0, false);
    let e = d.poll().unwrap();
    assert!(e.release_fence_fd >= 0);
    let owned = unsafe { OwnedFd::from_raw_fd(e.release_fence_fd) };
    assert!(unsafe { libc::fcntl(owned.as_raw_fd(), libc::F_GETFD) } >= 0);
    p.busy.remove(&e.cookie);
    queue(&mut d, 1, 1, false);
    let fd = d.events[0].release_fence_fd;
    let (_keepalive, identity) = fence_probe(fd);
    drop(d);
    assert!(p.busy.is_empty());
    assert_eq!(p.discarded.len(), 1);
    assert_ne!(fence_identity(fd).as_ref(), Some(&identity));
}
#[test]
fn front_target_format() {
    let mut p = Pool::new(8);
    p.format = 3;
    let (mut d, c) = make(4, &mut p);
    queue(&mut d, 1, 0, false);
    assert_eq!(c.borrow().copy_format, 3);
    events(&mut d, &mut p);
    let mut r = NvC2DecResult::default();
    assert_eq!(d.queue(&[1, 1, 128, 2, 10], 1, 0, &mut r), FATAL);
    assert_eq!(p.discarded.len(), 1);
}
#[test]
fn multi_output_final_and_eos_pressure() {
    let mut p = Pool::new(1);
    let (mut d, _) = make(3, &mut p);
    let r = queue(&mut d, 2, 7, true);
    assert_eq!((r.consumed, r.blocked), (1, 1));
    let first = events(&mut d, &mut p);
    assert_eq!(
        (first[0].index, first[0].final_for_index, first[0].eos),
        (7, 0, 0)
    );
    let mut r = NvC2DecResult::default();
    assert_eq!(d.drain(&mut r), OK);
    let last = events(&mut d, &mut p);
    assert_eq!(
        (last[0].index, last[0].final_for_index, last[0].eos),
        (7, 1, 1)
    );
    assert!(d.poll().is_none());
}
#[test]
fn csd_limit_atomic_and_runs() {
    let mut p = Pool::new(8);
    let (mut d, c) = make(1, &mut p);
    let mut r = NvC2DecResult::default();
    let mut csd = vec![0; 2_097_152];
    csd[0] = b'P';
    assert_eq!(d.queue(&csd, 0, 2, &mut r), OK);
    let calls = c.borrow().calls.len();
    assert_eq!(d.queue(b"P", 1, 2, &mut r), FATAL);
    assert_eq!(d.failure.as_ref().unwrap().0, LIMIT);
    assert_eq!(d.csd_bytes, 2_097_152);
    assert_eq!(c.borrow().calls.len(), calls);
    assert_eq!(check_csd(2_097_152, 0), OK);
    assert_eq!(check_csd(2_097_152, 1), LIMIT);
    assert_eq!(check_csd(u64::MAX, 1), INVALID);
    let mut p = Pool::new(8);
    let (mut d, _) = make(2, &mut p);
    d.queue(b"P0", 0, 2, &mut r);
    queue(&mut d, 1, 1, false);
    d.queue(b"P1", 2, 2, &mut r);
    assert_eq!(d.csd, [b"P1".to_vec()]);
}
#[test]
fn ffi_null_empty_and_invalid_result() {
    let mut p = Pool::new(8);
    let (mut d, _) = make(3, &mut p);
    let mut r = NvC2DecResult::default();
    assert_eq!(
        unsafe { nv_c2_dec_queue(&mut d, std::ptr::null(), 0, 0, 2, &mut r) },
        OK
    );
    assert_eq!(events(&mut d, &mut p)[0].empty, 1);
    assert_eq!(
        unsafe { nv_c2_dec_queue(&mut d, std::ptr::null(), 1, 1, 0, &mut r) },
        INVALID
    );
    assert_eq!(r.consumed, 2);
    assert_eq!(
        unsafe { nv_c2_dec_queue(&mut d, std::ptr::null(), 0, 1, 0, &mut r) },
        OK
    );
    assert!(d.poll().is_none());
}
#[test]
fn fence_cleanup_before_and_after_copy() {
    for failure in ["format", "hdr", "copy", "release", ""] {
        let mut p = Pool::new(8);
        p.acquire = unsafe { libc::eventfd(1, libc::EFD_CLOEXEC) };
        let acquire = p.acquire;
        let (_keepalive, identity) = fence_probe(acquire);
        let (mut d, c) = make(3, &mut p);
        c.borrow_mut().fail = failure;
        if failure == "format" {
            p.format = 2;
        }
        let mut r = NvC2DecResult::default();
        let rc = d.queue(&[1, 1], 0, 0, &mut r);
        assert_eq!(rc, if failure.is_empty() { OK } else { FATAL });
        assert_ne!(fence_identity(acquire).as_ref(), Some(&identity));
        if failure.is_empty() {
            assert!(c.borrow().acquire_seen);
            events(&mut d, &mut p);
        } else {
            assert_eq!(p.discarded.len(), 1);
            assert_eq!(c.borrow().released, [1]);
        }
    }
}
#[test]
fn flush_with_held_eos_and_failure_events() {
    let mut p = Pool::new(8);
    let (mut d, c) = make(3, &mut p);
    queue(&mut d, 1, 0, false);
    let event = d.events.pop_front().unwrap();
    d.eos = Some((1, true, Some(NvC2DecEvent { index: 1, ..event })));
    d.pending.push_back(Pending {
        index: 1,
        remaining: 0,
    });
    c.borrow_mut().fail = "flush";
    let mut r = NvC2DecResult::default();
    assert_eq!(d.flush(&mut r), FATAL);
    let e = events(&mut d, &mut p);
    assert_eq!(e.last().unwrap().eos, 1);
    assert_eq!(e.len(), 1);
}
#[test]
fn dpb_retry_and_resize_pressure() {
    let mut p = Pool::new(1);
    let (mut d, _) = make(3, &mut p);
    for i in 0..12 {
        let r = queue(&mut d, 1, i, false);
        if r.consumed == 0 {
            assert_eq!(r.blocked, 1);
            break;
        }
    }
    assert!(d.retry.is_some());
    events(&mut d, &mut p);
    let mut r = NvC2DecResult::default();
    assert_eq!(d.flush(&mut r), OK);
    events(&mut d, &mut p);
    assert!(d.retry.is_none());
    let (mut d, c) = make(3, &mut p);
    queue(&mut d, 1, 0, false);
    queue(&mut d, 1, 1, false);
    assert_eq!(d.queue(&[1, 1, 0, 10], 2, 0, &mut r), OK);
    assert_eq!((r.consumed, r.blocked), (0, 1));
    events(&mut d, &mut p);
    r = NvC2DecResult::default();
    assert_eq!(d.queue(&[1, 1, 0, 10], 2, 0, &mut r), OK);
    assert_eq!(c.borrow().reopens[0].0, 2560);
    events(&mut d, &mut p);
    r = NvC2DecResult::default();
    d.drain(&mut r);
    events(&mut d, &mut p);
}
#[test]
fn caps_gate_and_owned_startup_config() {
    let mut cfg = config(4);
    let mut caps = super::super::NvVideoCaps::default();
    assert!(validate_caps(&cfg, &caps).is_err());
    caps.supported = 1;
    caps.min_width = 16;
    caps.min_height = 16;
    caps.max_width = 4096;
    caps.max_height = 4096;
    caps.bit_depth_mask = (1 << 8) | (1 << 10);
    assert!(validate_caps(&cfg, &caps).is_ok());
    cfg.max_width = 8192;
    assert!(validate_caps(&cfg, &caps).is_err());
    cfg.max_width = 1920;
    cfg.bit_depth = 12;
    assert!(validate_caps(&cfg, &caps).is_err());
    let mut p = Pool::new(1);
    let (d, _) = make(4, &mut p);
    assert_eq!(d.startup, [1, 2, 3]);
    assert_eq!(nv_c2_dec_check_csd_size(0, 2_097_153), LIMIT);
}

#[test]
fn multi_output_without_eos_and_fatal_result_poll() {
    for codec in [3, 4] {
        let mut p = Pool::new(8);
        let (mut d, c) = make(codec, &mut p);
        queue(&mut d, 3, 42, false);
        let output = events(&mut d, &mut p);
        assert_eq!(
            output
                .iter()
                .map(|e| (e.index, e.final_for_index, e.eos))
                .collect::<Vec<_>>(),
            [(42, 0, 0), (42, 0, 0), (42, 1, 0)]
        );
        queue(&mut d, 1, 43, false);
        c.borrow_mut().fail = "decode";
        let mut result = NvC2DecResult::default();
        assert_eq!(
            unsafe { nv_c2_dec_queue(&mut d, [1, 1].as_ptr(), 2, 44, 0, &mut result) },
            FATAL
        );
        assert_eq!((result.consumed, result.cause), (2, FATAL));
        let first_error = last_error();
        assert_eq!(events(&mut d, &mut p)[0].index, 43);
        assert_eq!(unsafe { nv_c2_dec_drain(&mut d, &mut result) }, FATAL);
        assert_eq!(last_error(), first_error);
    }
}

#[test]
fn dpb_pressure_without_pool_refusal_and_panic_latch() {
    let mut p = Pool::new(8);
    let (mut d, c) = make(3, &mut p);
    c.borrow_mut().fail = "again";
    let mut result = NvC2DecResult::default();
    assert_eq!(
        unsafe { nv_c2_dec_queue(&mut d, [1, 1].as_ptr(), 2, 0, 0, &mut result) },
        OK
    );
    assert_eq!((result.consumed, result.blocked), (0, 2));
    c.borrow_mut().fail = "";
    assert_eq!(
        unsafe { nv_c2_dec_queue(&mut d, [1, 1].as_ptr(), 2, 0, 0, &mut result) },
        OK
    );
    assert_eq!((result.consumed, result.blocked), (1, 0));
    events(&mut d, &mut p);
    c.borrow_mut().fail = "panic";
    assert_eq!(
        unsafe { nv_c2_dec_queue(&mut d, [1, 1].as_ptr(), 2, 1, 0, &mut result) },
        FATAL
    );
    assert_eq!((result.consumed, result.cause), (2, FATAL));
    assert!(last_error().contains("panic"));
    assert_eq!(unsafe { nv_c2_dec_flush(&mut d, &mut result) }, FATAL);
}
#[test]
fn corrupt_eos_is_last_and_resize_replay_failure_is_terminal() {
    let mut p = Pool::new(8);
    let (mut d, _) = make(2, &mut p);
    queue(&mut d, 1, 0, false);
    queue(&mut d, 1, 1, false);
    let mut result = NvC2DecResult::default();
    assert_eq!(d.queue(&[255], 2, 1, &mut result), OK);
    let e = events(&mut d, &mut p);
    assert_eq!(
        e.iter().map(|e| (e.index, e.eos)).collect::<Vec<_>>(),
        [(0, 0), (1, 0), (2, 1)]
    );
    assert_eq!((e[2].empty, e[2].corrupt), (1, 1));
    let (mut d, c) = make(2, &mut p);
    assert_eq!(d.queue(b"P0", 0, 2, &mut result), OK);
    events(&mut d, &mut p);
    c.borrow_mut().fail = "replay";
    assert_eq!(d.queue(&[1, 1, 0, 15], 1, 0, &mut result), FATAL);
    assert!(d.failure.as_ref().unwrap().1.contains("replay"));
}
#[test]
fn destroy_held_picture_and_withheld_eos_cookie() {
    let mut p = Pool::new(1);
    let (mut d, c) = make(3, &mut p);
    queue(&mut d, 2, 0, true);
    let first = d.poll().unwrap();
    close_fd(first.release_fence_fd);
    assert!(d.held.is_some());
    drop(d);
    assert_eq!(c.borrow().released, [1, 2]);
    p.busy.remove(&first.cookie);
    let (mut d, _) = make(3, &mut p);
    queue(&mut d, 1, 1, false);
    let e = d.poll().unwrap();
    let fd = e.release_fence_fd;
    let (_keepalive, identity) = fence_probe(fd);
    d.eos = Some((1, true, Some(e)));
    drop(d);
    assert!(p.busy.is_empty());
    assert_eq!(p.discarded.len(), 1);
    assert_ne!(fence_identity(fd).as_ref(), Some(&identity));
}
