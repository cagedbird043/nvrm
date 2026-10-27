//! Checked allocation arithmetic and validation of an already copied handle POD.
use super::{ffi::NvVideoTarget, ffi_dec::*};
const YV12: u32 = 0x32315659;
const YVU420_ANDROID: u32 = u32::from_le_bytes(*b"9997");
fn align(v: u64, a: u64) -> Option<u64> {
    v.checked_add(a - 1).map(|n| n / a * a)
}
fn fail(why: &str) -> Result<NvVideoTarget, String> {
    Err(why.into())
}
fn same_object(a: i32, b: i32) -> bool {
    use std::{
        fs::File,
        mem::ManuallyDrop,
        os::{fd::FromRawFd, unix::fs::MetadataExt},
    };
    if a == b {
        return true;
    }
    // Borrow the descriptors without closing or duplicating them.
    let (a, b) = unsafe {
        (
            ManuallyDrop::new(File::from_raw_fd(a)),
            ManuallyDrop::new(File::from_raw_fd(b)),
        )
    };
    match (a.metadata(), b.metadata()) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}
fn target(s: &NvC2GrallocSnapshot, r: &NvC2DecAllocation) -> Result<NvVideoTarget, String> {
    if s.abi_version != 1
        || s.native_version != 12
        || !(1..=5).contains(&s.num_fds)
        || s.num_ints < 0
        || i64::from(s.num_ints) + i64::from(s.num_fds) != 35
        || s.payload_words != 35
        || s.reserved != [0; 4]
    {
        return fail("gralloc envelope");
    }
    if s.magic != 0xabcddcba
        || !(1..=4).contains(&s.num_planes)
        || s.num_fds as u32 != s.num_planes + u32::from(s.reserved_region_size != 0)
    {
        return fail("gralloc magic/plane fd count");
    }
    if s.fds[..s.num_planes as usize].iter().any(|fd| *fd < 0) {
        return fail("gralloc negative image fd");
    }
    let image = s
        .total_size
        .checked_sub(s.reserved_region_size)
        .filter(|v| *v != 0)
        .ok_or("gralloc image budget")?;
    if s.width == 0
        || s.height == 0
        || s.width % 2 != 0
        || s.height % 2 != 0
        || s.width < r.block_width
        || s.height < r.block_height
        || r.block_width == 0
        || r.block_height == 0
        || r.policy > 1
        || s.droid_format as u32 != r.hal_format
    {
        return fail("gralloc allocation geometry/Android format");
    }
    let mut t = NvVideoTarget {
        width: s.width,
        height: s.height,
        modifier: s.modifier,
        acquire_fence_fd: -1,
        cr_dmabuf_fd: -1,
        ..Default::default()
    };
    if r.policy == 1 {
        if r.hal_format != YV12
            || s.num_planes != 3
            || s.modifier != 0
            || ![YV12, YVU420_ANDROID].contains(&s.drm_format)
        {
            return fail("gralloc YV12 format/modifier");
        }
        let ys = align(s.width as u64, 16).ok_or("YV12 stride overflow")?;
        let cs = align(ys / 2, 16).ok_or("YV12 chroma stride overflow")?;
        let y = ys
            .checked_mul(s.height as u64)
            .ok_or("YV12 luma overflow")?;
        let c = cs
            .checked_mul((s.height / 2) as u64)
            .ok_or("YV12 chroma overflow")?;
        let cb = y.checked_add(c).ok_or("YV12 Cb overflow")?;
        let end = cb.checked_add(c).ok_or("YV12 end overflow")?;
        if ys > u32::MAX as u64
            || cs > u32::MAX as u64
            || end > image
            || s.strides[..3] != [ys as u32, cs as u32, cs as u32]
            || s.offsets[..3].iter().map(|n| *n as u64).ne([0, y, cb])
            || s.sizes[..3]
                .iter()
                .zip([y, c, c])
                .any(|(n, min)| (*n as u64) < min)
        {
            return fail("gralloc YV12 descriptor/range");
        }
        t.format = 3;
        t.dmabuf_fd = [s.fds[0], s.fds[2]];
        t.cr_dmabuf_fd = s.fds[1];
        t.pitch = [ys as u32, cs as u32];
        t.cr_pitch = cs as u32;
        t.offset = [0, cb];
        t.cr_offset = y;
        return Ok(t);
    }
    let (fmt, b) = match (r.hal_format, s.drm_format) {
        (0x23, 0x3231564e) => (1, 1u64),
        (0x36, 0x30313050) => (2, 2),
        _ => return fail("gralloc NV12/P010 format"),
    };
    if s.num_planes != 2 {
        return fail("gralloc native plane count");
    }
    let mut ranges = [(0, 0); 2];
    let block = if s.modifier == 0 {
        None
    } else {
        let v = s.modifier & ((1 << 56) - 1);
        let h = v & 15;
        let g = (v >> 20) & 3;
        if s.modifier >> 56 != 3
            || v & 16 == 0
            || v >> 26 != 0
            || (v >> 23) & 7 != 0
            || h > 5
            || !(g == 2 || (g == 3 && fmt == 1))
        {
            return fail("gralloc unsupported block-linear modifier");
        }
        // The real engine generation is validated again by Session::copy_out.
        Some(8 << h)
    };
    for (p, range) in ranges.iter_mut().enumerate() {
        let rows = if p == 0 {
            s.height as u64
        } else {
            (s.height / 2) as u64
        };
        let row = s.width as u64 * b;
        let pitch = s.strides[p] as u64;
        let offset = s.offsets[p] as u64;
        let extent = if let Some(a) = block {
            if pitch % 64 != 0 || pitch < align(row, 64).ok_or("block row overflow")? {
                return fail("gralloc block-linear pitch");
            }
            pitch.checked_mul(align(rows, a).ok_or("block rows overflow")?)
        } else {
            if pitch == 0 || pitch % 4 != 0 || pitch < row {
                return fail("gralloc LINEAR pitch");
            }
            pitch.checked_mul(rows - 1).and_then(|n| n.checked_add(row))
        }
        .ok_or("gralloc plane extent overflow")?;
        let end = offset
            .checked_add(extent)
            .ok_or("gralloc plane end overflow")?;
        if fmt == 2 && (offset % 2 != 0 || pitch % 2 != 0) {
            return fail("gralloc P010 alignment");
        }
        if (s.sizes[p] as u64) < extent || end > image {
            return fail("gralloc plane range");
        }
        *range = (offset, end);
    }
    if same_object(s.fds[0], s.fds[1]) && ranges[0].0 < ranges[1].1 && ranges[1].0 < ranges[0].1 {
        return fail("gralloc overlapping image planes");
    }
    t.format = fmt;
    t.dmabuf_fd = [s.fds[0], s.fds[1]];
    t.offset = [s.offsets[0] as u64, s.offsets[1] as u64];
    t.pitch = [s.strides[0], s.strides[1]];
    Ok(t)
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_target(
    s: *const NvC2GrallocSnapshot,
    r: *const NvC2DecAllocation,
    out: *mut NvVideoTarget,
) -> i32 {
    guard(|| {
        if s.is_null() || r.is_null() || out.is_null() {
            return invalid("NULL target argument");
        }
        match target(unsafe { &*s }, unsafe { &*r }) {
            Ok(t) => {
                unsafe { out.write(t) };
                OK
            }
            Err(why) => invalid(&why),
        }
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_allocation(
    w: u32,
    h: u32,
    fmt: u32,
    cpu: u32,
    out: *mut NvC2DecAllocation,
) -> i32 {
    guard(|| {
        if w == 0 || h == 0 || ![1, 2].contains(&fmt) || cpu > 1 || out.is_null() {
            return invalid("invalid allocation argument");
        }
        let yv12 = cpu == 1 && fmt == 1;
        let (bw, bh) = (
            align(w as u64, if yv12 { 128 } else { 2 }).unwrap(),
            align(h as u64, 2).unwrap(),
        );
        if bw > u32::MAX as u64 || bh > u32::MAX as u64 {
            return invalid("allocation alignment overflow");
        }
        unsafe {
            out.write(NvC2DecAllocation {
                block_width: bw as u32,
                block_height: bh as u32,
                hal_format: if fmt == 2 {
                    0x36
                } else if yv12 {
                    YV12
                } else {
                    0x23
                },
                policy: u32::from(yv12),
            })
        };
        OK
    })
}
#[no_mangle]
pub unsafe extern "C" fn nv_c2_dec_max_input_size(w: u32, h: u32, out: *mut u32) -> i32 {
    guard(|| {
        if w == 0 || h == 0 || out.is_null() {
            return invalid("invalid max input argument");
        }
        let n = (w as u64)
            .div_ceil(64)
            .checked_mul((h as u64).div_ceil(64))
            .and_then(|v| v.checked_mul(3072));
        match n
            .map(|n| n.max(2_097_152))
            .and_then(|n| u32::try_from(n).ok())
        {
            Some(n) => {
                unsafe { out.write(n) };
                OK
            }
            None => invalid("max input size overflow"),
        }
    })
}
#[cfg(test)]
mod tests;
