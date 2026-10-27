//! C ABI for owned caps snapshots; all entry points contain unwinds.
use super::{
    caps::*,
    ffi::{self, NvVideoDevice},
    venc, NvVideoCaps, VideoError, ABI_VERSION,
};
use std::{
    ffi::{c_char, CStr, CString},
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
};
fn report((code, message): Error) -> i32 {
    ffi::fail(VideoError::Invalid(message));
    match code {
        -1 | -2 | -4 | -5 | -8 => code,
        _ => -8,
    }
}
fn guard(f: impl FnOnce() -> Result<(), Error>) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => report(e),
        Err(_) => report((-8, "internal caps panic".into())),
    }
}
fn invalid(what: &str) -> Error {
    (-1, format!("{what} is NULL or invalid"))
}
fn backend_error(code: i32) -> Error {
    // SAFETY: the backend's thread-local diagnostic is always NUL terminated;
    // copy it before another ABI call can replace it.
    let msg = unsafe { CStr::from_ptr(ffi::nv_video_last_error()) }
        .to_string_lossy()
        .into_owned();
    let mapped = match code {
        -1 | -2 | -4 | -5 | -8 => code,
        _ => -8,
    };
    (
        mapped,
        if mapped == code {
            msg
        } else {
            format!("backend error {code}: {msg}")
        },
    )
}
/// Caller supplies valid device/output storage and serializes the borrowed device.
#[no_mangle]
pub unsafe extern "C" fn nv_c2_caps_query(dev: *mut NvVideoDevice, out: *mut *mut NvC2Caps) -> i32 {
    guard(|| {
        if out.is_null() {
            return Err(invalid("out"));
        }
        unsafe { out.write(ptr::null_mut()) };
        if dev.is_null() {
            return Err(invalid("dev"));
        }
        let info = unsafe { &(*dev).0.info };
        if info.abi_version != ABI_VERSION || info.nvdec_class == 0 {
            return Err(invalid("device ABI/NVDEC class"));
        }
        let caps = NvC2Caps::query(
            |codec| {
                let mut c = NvVideoCaps::default();
                let rc = unsafe { ffi::nv_video_query_caps(dev, codec, &mut c) };
                if rc == 0 {
                    Ok(c)
                } else {
                    Err(backend_error(rc))
                }
            },
            || {
                let mut enc = ptr::null_mut();
                let rc = unsafe { venc::nv_venc_open(&mut enc) };
                if rc != 0 {
                    let (code, msg) = backend_error(rc);
                    return Err((code, format!("nv_venc_open failed ({rc}): {msg}")));
                }
                struct Owned(*mut venc::NvVencDevice);
                impl Drop for Owned {
                    fn drop(&mut self) {
                        unsafe { venc::nv_venc_close(self.0) }
                    }
                }
                let enc = Owned(enc);
                Ok(unsafe { venc::nv_venc_class(enc.0, 0) })
            },
        )?;
        unsafe { out.write(Box::into_raw(Box::new(caps))) };
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_caps_view(caps: *const NvC2Caps, out: *mut NvC2CapsView) -> i32 {
    guard(|| {
        if out.is_null() {
            return Err(invalid("out"));
        }
        unsafe { out.write(NvC2CapsView::default()) };
        if caps.is_null() {
            return Err(invalid("caps"));
        }
        unsafe { out.write((*caps).view()) };
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_media_codecs_xml(caps: *const NvC2Caps, out: *mut NvC2Bytes) -> i32 {
    guard(|| {
        if out.is_null() {
            return Err(invalid("out"));
        }
        unsafe { out.write(NvC2Bytes::default()) };
        if caps.is_null() {
            return Err(invalid("caps"));
        }
        unsafe { out.write((*caps).xml()) };
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_caps_destroy(caps: *mut NvC2Caps) {
    let _ = guard(|| {
        if !caps.is_null() {
            drop(unsafe { Box::from_raw(caps) })
        }
        Ok(())
    });
}
unsafe fn text(gpu: *const c_char, out: *mut *mut NvC2Text, render: fn(&str) -> String) -> i32 {
    guard(|| {
        if out.is_null() {
            return Err(invalid("out"));
        }
        unsafe { out.write(ptr::null_mut()) };
        if gpu.is_null() {
            return Err(invalid("gpu_name"));
        }
        let gpu = unsafe { CStr::from_ptr(gpu) }
            .to_str()
            .map_err(|_| invalid("gpu_name UTF-8"))?;
        let text = CString::new(render(gpu)).map_err(|_| invalid("text NUL"))?;
        unsafe { out.write(Box::into_raw(Box::new(NvC2Text(text)))) };
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_no_decoder_xml(gpu: *const c_char, out: *mut *mut NvC2Text) -> i32 {
    unsafe { text(gpu, out, no_decoder_xml) }
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_no_decoder_reason(
    gpu: *const c_char,
    out: *mut *mut NvC2Text,
) -> i32 {
    unsafe { text(gpu, out, no_decoder_reason) }
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_text_view(text: *const NvC2Text, out: *mut NvC2Bytes) -> i32 {
    guard(|| {
        if out.is_null() {
            return Err(invalid("out"));
        }
        unsafe { out.write(NvC2Bytes::default()) };
        if text.is_null() {
            return Err(invalid("text"));
        }
        unsafe { out.write(bytes(&(*text).0)) };
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_text_destroy(text: *mut NvC2Text) {
    let _ = guard(|| {
        if !text.is_null() {
            drop(unsafe { Box::from_raw(text) })
        }
        Ok(())
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_and_panic_contract() {
        unsafe {
            let mut c = ptr::dangling_mut();
            assert_eq!(nv_c2_caps_query(ptr::null_mut(), &mut c), -1);
            assert!(c.is_null());
            let mut v = NvC2CapsView {
                abi_version: 99,
                ..Default::default()
            };
            assert_eq!(nv_c2_caps_view(ptr::null(), &mut v), -1);
            assert_eq!(v.abi_version, 0);
            let mut b = NvC2Bytes {
                data: ptr::dangling(),
                len: 99,
            };
            assert_eq!(nv_c2_media_codecs_xml(ptr::null(), &mut b), -1);
            assert!(b.data.is_null());
            assert_eq!(b.len, 0);
            assert_eq!(guard(|| panic!("injected")), -8);
            assert_eq!(report((-7, "backend -7".into())), -8);
            nv_c2_caps_destroy(ptr::null_mut());
            nv_c2_text_destroy(ptr::null_mut());
        }
    }
    #[test]
    fn text_is_owned_and_stable() {
        unsafe {
            let gpu = CString::new("a---b").unwrap();
            let mut t = ptr::null_mut();
            assert_eq!(nv_c2_no_decoder_xml(gpu.as_ptr(), &mut t), 0);
            drop(gpu);
            let mut a = NvC2Bytes::default();
            assert_eq!(nv_c2_text_view(t, &mut a), 0);
            let mut b = NvC2Bytes::default();
            assert_eq!(nv_c2_text_view(t, &mut b), 0);
            assert_eq!(a.data, b.data);
            assert_eq!(a.len, b.len);
            assert_eq!(*a.data.add(a.len), 0);
            nv_c2_text_destroy(t);
        }
    }
}
