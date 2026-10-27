//! Serialized decoder C ABI, matching video/nv_c2_dec.h.
pub use super::c2_dec::NvC2Dec;
use super::{
    c2_dec::{last_error, Native},
    ffi::{self, NvVideoDevice, NvVideoHdrMetadata, NvVideoPicture, NvVideoTarget},
    VideoError,
};
use std::ffi::c_void;
pub(super) const OK: i32 = 0;
pub(super) const FATAL: i32 = -1;
pub(super) const INVALID: i32 = -2;
pub(super) const LIMIT: i32 = -3;
const CAP: u64 = 2_097_152;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvC2DecConfig {
    pub abi_version: u32,
    pub codec: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub bit_depth: u32,
    pub extra_output_slots: u32,
    pub codec_config: *const u8,
    pub codec_config_len: usize,
    pub reserved: [u32; 8],
}
#[repr(C)]
#[derive(Default)]
pub struct NvC2DecBuffer {
    pub target: NvVideoTarget,
    pub cookie: u64,
}
pub type Fetch = unsafe extern "C" fn(*mut c_void, u32, u32, u32, *mut NvC2DecBuffer) -> i32;
pub type Discard = unsafe extern "C" fn(*mut c_void, u64);
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvC2DecCallbacks {
    pub ctx: *mut c_void,
    pub fetch: Option<Fetch>,
    pub discard: Option<Discard>,
}
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct NvC2DecResult {
    pub consumed: u32,
    pub blocked: u32,
    pub cause: i32,
    pub reserved: u32,
}
impl Default for NvC2DecResult {
    fn default() -> Self {
        Self {
            consumed: 2,
            blocked: 0,
            cause: 0,
            reserved: 0,
        }
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct NvC2DecEvent {
    pub r#type: u32,
    pub empty: u32,
    pub corrupt: u32,
    pub eos: u32,
    pub index: u64,
    pub cookie: u64,
    pub release_fence_fd: i32,
    pub size_changed: u32,
    pub final_for_index: u32,
    pub reserved: u32,
    pub picture: NvVideoPicture,
    pub hdr: NvVideoHdrMetadata,
}
impl NvC2DecEvent {
    pub(super) fn empty() -> Self {
        Self {
            r#type: 1,
            empty: 1,
            corrupt: 0,
            eos: 0,
            index: 0,
            cookie: 0,
            release_fence_fd: -1,
            size_changed: 0,
            final_for_index: 1,
            reserved: 0,
            picture: NvVideoPicture::default(),
            hdr: NvVideoHdrMetadata::default(),
        }
    }
}
#[repr(C)]
#[derive(Default)]
pub struct NvC2DecHdrStatic {
    pub present: u32,
    pub red: [f32; 2],
    pub green: [f32; 2],
    pub blue: [f32; 2],
    pub white: [f32; 2],
    pub max_luminance: f32,
    pub min_luminance: f32,
    pub max_cll: f32,
    pub max_fall: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NvC2GrallocSnapshot {
    pub abi_version: u32,
    pub native_version: i32,
    pub num_fds: i32,
    pub num_ints: i32,
    pub payload_words: u32,
    pub magic: u32,
    pub num_planes: u32,
    pub fds: [i32; 5],
    pub strides: [u32; 4],
    pub offsets: [u32; 4],
    pub sizes: [u32; 4],
    pub width: u32,
    pub height: u32,
    pub drm_format: u32,
    pub tiling: u32,
    pub modifier: u64,
    pub reserved_region_size: u64,
    pub total_size: u64,
    pub pixel_stride: u32,
    pub droid_format: i32,
    pub reserved: [u32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct NvC2DecAllocation {
    pub block_width: u32,
    pub block_height: u32,
    pub hal_format: u32,
    pub policy: u32,
}

pub(super) fn check_csd(current: u64, addition: u64) -> i32 {
    match current.checked_add(addition) {
        None => INVALID,
        Some(n) if n > CAP => LIMIT,
        Some(_) => OK,
    }
}
pub(super) fn guard(f: impl FnOnce() -> i32) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(rc) => rc,
        Err(_) => {
            ffi::fail(VideoError::DeviceLost(
                "Codec2 decoder internal panic".into(),
            ));
            FATAL
        }
    }
}
pub(super) fn invalid(why: &str) -> i32 {
    ffi::fail(VideoError::Invalid(why.into()));
    INVALID
}
fn publish(dec: &NvC2Dec) {
    if let Some((_, why)) = &dec.failure {
        ffi::fail(VideoError::DeviceLost(why.clone()));
    }
}
unsafe fn bytes<'a>(data: *const u8, len: usize) -> Result<&'a [u8], i32> {
    if len == 0 {
        Ok(&[])
    } else if data.is_null() || len > isize::MAX as usize {
        Err(invalid("NULL/oversized input"))
    } else {
        Ok(unsafe { std::slice::from_raw_parts(data, len) })
    }
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_create(
    dev: *mut NvVideoDevice,
    cfg: *const NvC2DecConfig,
    cb: *const NvC2DecCallbacks,
    out: *mut *mut NvC2Dec,
) -> i32 {
    if !out.is_null() {
        unsafe { out.write(std::ptr::null_mut()) };
    }
    guard(|| {
        if dev.is_null() || cfg.is_null() || cb.is_null() || out.is_null() {
            return invalid("NULL create argument");
        }
        let (cfg, cb) = unsafe { (*cfg, *cb) };
        if cfg.abi_version != 1
            || cfg.reserved != [0; 8]
            || !(1..=4).contains(&cfg.codec)
            || cfg.max_width == 0
            || cfg.max_height == 0
            || cb.fetch.is_none()
            || cb.discard.is_none()
        {
            return invalid("invalid decoder configuration/callbacks");
        }
        let csd = match unsafe { bytes(cfg.codec_config, cfg.codec_config_len) } {
            Ok(d) => d,
            Err(rc) => return rc,
        };
        let rc = nv_c2_dec_check_csd_size(0, csd.len() as u64);
        if rc != OK {
            return rc;
        }
        let native = match Native::open(dev, &cfg, csd) {
            Ok(n) => n,
            Err(why) => {
                ffi::fail(VideoError::DeviceLost(why));
                return FATAL;
            }
        };
        unsafe {
            out.write(Box::into_raw(Box::new(NvC2Dec::new(
                Box::new(native),
                cfg,
                csd.to_vec(),
                cb,
            ))))
        };
        OK
    })
}
unsafe fn operation(
    dec: *mut NvC2Dec,
    out: *mut NvC2DecResult,
    f: impl FnOnce(&mut NvC2Dec, &mut NvC2DecResult) -> i32,
) -> i32 {
    if !out.is_null() {
        unsafe { out.write(NvC2DecResult::default()) };
    }
    if dec.is_null() || out.is_null() {
        return invalid("NULL decoder/result");
    }
    let d = unsafe { &mut *dec };
    let r = unsafe { &mut *out };
    let rc = guard(|| f(d, r));
    if rc == FATAL {
        if d.failure.is_none() {
            d.fail(FATAL, last_error());
        }
        r.consumed = 2;
        r.cause = d.failure.as_ref().unwrap().0;
        publish(d);
    } else if rc < 0 {
        r.consumed = 2;
    }
    rc
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_queue(
    dec: *mut NvC2Dec,
    data: *const u8,
    len: usize,
    index: u64,
    flags: u32,
    out: *mut NvC2DecResult,
) -> i32 {
    unsafe {
        operation(dec, out, |d, r| {
            if flags & !3 != 0 {
                return invalid("unknown decoder flags");
            }
            let data = match bytes(data, len) {
                Ok(v) => v,
                Err(rc) => return rc,
            };
            d.queue(data, index, flags, r)
        })
    }
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_drain(dec: *mut NvC2Dec, out: *mut NvC2DecResult) -> i32 {
    unsafe { operation(dec, out, |d, r| d.drain(r)) }
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_flush(dec: *mut NvC2Dec, out: *mut NvC2DecResult) -> i32 {
    unsafe { operation(dec, out, |d, r| d.flush(r)) }
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_poll(dec: *mut NvC2Dec, out: *mut NvC2DecEvent) -> i32 {
    guard(|| {
        if dec.is_null() || out.is_null() {
            return invalid("NULL poll argument");
        }
        match unsafe { &mut *dec }.poll() {
            Some(e) => {
                unsafe { out.write(e) };
                1
            }
            None => 0,
        }
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_destroy(dec: *mut NvC2Dec) {
    guard(|| {
        if !dec.is_null() {
            drop(unsafe { Box::from_raw(dec) });
        }
        OK
    });
}
#[no_mangle]
pub extern "C" fn nv_c2_dec_check_csd_size(current: u64, addition: u64) -> i32 {
    guard(|| {
        let rc = check_csd(current, addition);
        if rc != OK {
            ffi::fail(VideoError::Invalid(
                if rc == LIMIT {
                    "CSD exceeds 2 MiB"
                } else {
                    "CSD size arithmetic overflow"
                }
                .into(),
            ));
        }
        rc
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_hdr_static(
    codec: u32,
    input: *const NvVideoHdrMetadata,
    out: *mut NvC2DecHdrStatic,
) -> i32 {
    guard(|| {
        if input.is_null() || out.is_null() || !(1..=4).contains(&codec) {
            return invalid("invalid HDR argument");
        }
        let m = unsafe { &*input };
        let mut h = NvC2DecHdrStatic::default();
        if m.has_mastering_display != 0 {
            h.present = 1;
            let d = &m.mastering_display;
            let (r, g, b, c, max, min) = if codec == 4 {
                (0, 1, 2, 1.0 / 65536.0, 1.0 / 256.0, 1.0 / 16384.0)
            } else {
                (2, 0, 1, 0.00002, 0.0001, 0.0001)
            };
            let chroma = |i: usize| {
                [
                    d.display_primaries_x[i] as f32 * c,
                    d.display_primaries_y[i] as f32 * c,
                ]
            };
            h.red = chroma(r);
            h.green = chroma(g);
            h.blue = chroma(b);
            h.white = [d.white_point_x as f32 * c, d.white_point_y as f32 * c];
            h.max_luminance = d.max_luminance as f32 * max;
            h.min_luminance = d.min_luminance as f32 * min;
        }
        if m.has_content_light_level != 0 {
            h.present = 1;
            h.max_cll = m.max_content_light_level as f32;
            h.max_fall = m.max_pic_average_light_level as f32;
        }
        unsafe { out.write(h) };
        OK
    })
}
const _: () = {
    use std::mem::{align_of, offset_of, size_of};
    const WIDE: bool = cfg!(target_pointer_width = "64");
    assert!(size_of::<NvC2DecConfig>() == if WIDE { 72 } else { 64 });
    assert!(align_of::<NvC2DecConfig>() == if WIDE { 8 } else { 4 });
    assert!(size_of::<NvC2DecBuffer>() == 88 && offset_of!(NvC2DecBuffer, cookie) == 80);
    assert!(size_of::<NvC2DecCallbacks>() == if WIDE { 24 } else { 12 });
    assert!(size_of::<NvC2DecResult>() == 16 && offset_of!(NvC2DecResult, cause) == 8);
    assert!(size_of::<NvC2DecEvent>() == if WIDE { 200 } else { 196 });
    assert!(offset_of!(NvC2DecEvent, picture) == 48);
    assert!(offset_of!(NvC2DecEvent, hdr) == if WIDE { 136 } else { 132 });
    assert!(size_of::<NvC2DecHdrStatic>() == 52 && align_of::<NvC2DecHdrStatic>() == 4);
    assert!(
        size_of::<NvC2GrallocSnapshot>() == 160 && offset_of!(NvC2GrallocSnapshot, modifier) == 112
    );
    assert!(size_of::<NvC2DecAllocation>() == 16);
    assert!(offset_of!(NvC2DecConfig, codec_config) == 24);
    assert!(offset_of!(NvC2DecConfig, codec_config_len) == 24 + size_of::<*const u8>());
    assert!(align_of::<NvC2DecCallbacks>() == size_of::<*const u8>());
    assert!(offset_of!(NvC2DecCallbacks, fetch) == size_of::<*const u8>());
    assert!(offset_of!(NvC2DecCallbacks, discard) == 2 * size_of::<*const u8>());
    assert!(align_of::<NvC2DecBuffer>() == align_of::<u64>());
    assert!(align_of::<NvC2DecEvent>() == align_of::<u64>());
    assert!(align_of::<NvC2DecResult>() == 4);
    assert!(offset_of!(NvC2DecHdrStatic, red) == 4);
    assert!(align_of::<NvC2GrallocSnapshot>() == align_of::<u64>());
    assert!(offset_of!(NvC2GrallocSnapshot, pixel_stride) == 136);
    assert!(align_of::<NvC2DecAllocation>() == 4 && offset_of!(NvC2DecAllocation, policy) == 12);
};
