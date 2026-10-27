//! NVIDIA GPU block-linear and NVENC input surfaces.
//!
//! GPU BL uses 64-byte x 8-row GOBs. NVENC's input layout on Ada
//! (input_cfg block_height 2 / input_bl_mode 0, T2.05d) instead uses
//! 64-byte x 16-row tiles, also written by NVDEC (M8).

pub fn bl_offset(x: usize, y: usize, pitch: usize, block_height_log2: u32) -> usize {
    let gob = ((x % 64) / 32) * 256
        + ((y % 8) / 2) * 64
        + ((x % 32) / 16) * 32
        + (y % 2) * 16
        + x % 16;
    let block_rows = 8usize << block_height_log2;
    let block = (y / block_rows) * (pitch / 64) + x / 64;
    block * (512usize << block_height_log2)
        + ((y / 8) % (1usize << block_height_log2)) * 512
        + gob
}

pub fn bl_plane_size(pitch: usize, rows: usize, block_height_log2: u32) -> usize {
    let block_rows = 8usize << block_height_log2;
    pitch * rows.div_ceil(block_rows) * block_rows
}

pub fn bl_tile_plane(
    dst: &mut [u8], src: &[u8], src_stride: usize, width: usize,
    rows: usize, pitch: usize, bh: u32,
) {
    for y in 0..rows {
        for x in (0..width).step_by(16) {
            let n = (width - x).min(16);
            let d = bl_offset(x, y, pitch, bh);
            let s = y * src_stride + x;
            dst[d..d + n].copy_from_slice(&src[s..s + n]);
        }
    }
}

pub fn in_offset(x: usize, y: usize, pitch: usize) -> usize {
    let inside = x % 16 + (y % 4) * 16 + ((x / 16) % 2) * 64
        + ((y / 4) % 2) * 128 + ((x / 32) % 2) * 256 + ((y / 8) % 2) * 512;
    (y / 16) * pitch * 16 + (x / 64) * 1024 + inside
}

pub fn in_plane_size(pitch: usize, rows: usize) -> usize {
    pitch * rows.div_ceil(16) * 16
}

fn tile_rows(dst: &mut [u8], src: &[u8], stride: usize, width: usize, rows: usize, pitch: usize) {
    // Each 16-byte run is contiguous; neither row nor tile padding is written.
    for y in 0..rows {
        for x in (0..width).step_by(16) {
            let n = (width - x).min(16);
            let d = in_offset(x, y, pitch);
            let s = y * stride + x;
            dst[d..d + n].copy_from_slice(&src[s..s + n]);
        }
    }
}

pub fn in_tile_plane(
    dst: &mut [u8], src: &[u8], src_stride: usize, width: usize, rows: usize, pitch: usize,
) {
    if rows == 0 || width == 0 { return; }
    assert!(pitch.is_multiple_of(64) && width <= pitch && src_stride >= width);
    assert!(src.len() >= (rows - 1) * src_stride + width);
    assert!(dst.len() >= in_plane_size(pitch, rows));
    #[cfg(target_arch = "x86_64")]
    if (dst.as_ptr() as usize).is_multiple_of(16) {
        use std::arch::x86_64::{__m128i, _mm_loadu_si128, _mm_sfence, _mm_stream_si128};
        for y0 in (0..rows).step_by(16) {
            let count = (rows - y0).min(16);
            let full = if count == 16 { width / 64 } else { 0 };
            for tile in 0..full {
                let mut d = y0 * pitch + tile * 1024;
                for y8 in 0..2 {
                    for x32 in 0..2 {
                        for y4 in 0..2 {
                            for x16 in 0..2 {
                                for y in 0..4 {
                                    let s = (y0 + y8 * 8 + y4 * 4 + y) * src_stride
                                        + tile * 64 + x32 * 32 + x16 * 16;
                                    // SAFETY: full tiles have 16 complete source rows; the
                                    // checked destination is aligned and each store advances 16.
                                    unsafe {
                                        let v = _mm_loadu_si128(src.as_ptr().add(s).cast::<__m128i>());
                                        _mm_stream_si128(dst.as_mut_ptr().add(d).cast::<__m128i>(), v);
                                    }
                                    d += 16;
                                }
                            }
                        }
                    }
                }
            }
            for y in y0..y0 + count {
                for x in (full * 64..width).step_by(16) {
                    let n = (width - x).min(16);
                    let d = in_offset(x, y, pitch);
                    let s = y * src_stride + x;
                    dst[d..d + n].copy_from_slice(&src[s..s + n]);
                }
            }
        }
        // SAFETY: publish all non-temporal stores before GPU submission.
        unsafe { _mm_sfence() };
        return;
    }
    tile_rows(dst, src, src_stride, width, rows, pitch);
}

fn untile_rows(dst: &mut [u8], stride: usize, src: &[u8], width: usize, rows: usize, pitch: usize) {
    for y in 0..rows {
        for x in (0..width).step_by(16) {
            let n = (width - x).min(16);
            let d = y * stride + x;
            let s = in_offset(x, y, pitch);
            dst[d..d + n].copy_from_slice(&src[s..s + n]);
        }
    }
}

pub fn in_untile_plane(
    dst: &mut [u8], dst_stride: usize, src: &[u8], width: usize, rows: usize, pitch: usize,
) {
    if rows == 0 || width == 0 { return; }
    let split = rows.div_ceil(16).div_ceil(2) * 16;
    if split >= rows {
        untile_rows(dst, dst_stride, src, width, rows, pitch);
        return;
    }
    let (a, b) = dst.split_at_mut(split * dst_stride);
    std::thread::scope(|scope| {
        scope.spawn(|| untile_rows(a, dst_stride, src, width, split, pitch));
        untile_rows(b, dst_stride, &src[split * pitch..], width, rows - split, pitch);
    });
}

// BT.709 video-range fixed-point conversion. Chroma uses the 2x2 sum;
// odd widths and heights repeat the last pixel/row, just as blocklinear.c.
fn rgb_rows<const BGR:bool>(s0: &[u8], s1: &[u8], y0: &mut [u8], y1: &mut [u8], uv: &mut [u8], width: usize) {
    let (ri,bi)=if BGR {(2,0)}else{(0,2)};
    for x in 0..width {
        let luma = |s: &[u8]| {
            16 + ((47 * s[4 * x + ri] as i32 + 157 * s[4 * x + 1] as i32
                + 16 * s[4 * x + bi] as i32 + 128) >> 8)
        };
        y0[x] = luma(s0) as u8;
        y1[x] = luma(s1) as u8;
    }
    for x in (0..width).step_by(2) {
        let x1 = (x + 1).min(width - 1);
        let sum = |c: usize| s0[4 * x + c] as i32 + s0[4 * x1 + c] as i32
            + s1[4 * x + c] as i32 + s1[4 * x1 + c] as i32;
        let (r, g, b) = (sum(ri), sum(1), sum(bi));
        uv[x] = (128 + ((-26 * r - 87 * g + 112 * b + 512) >> 10)) as u8;
        uv[x + 1] = (128 + ((112 * r - 102 * g - 10 * b + 512) >> 10)) as u8;
    }
}

fn l10r_rows(s0: &[u8], s1: &[u8], y0: &mut [u8], y1: &mut [u8], uv: &mut [u8], width: usize) {
    let pixel = |s: &[u8], x: usize| {
        let a = u32::from_le_bytes(s[4 * x..4 * x + 4].try_into().unwrap());
        [((a >> 20) & 1023) as i32, ((a >> 10) & 1023) as i32, (a & 1023) as i32]
    };
    for x in 0..width {
        let luma = |s| {
            let [r, g, b] = pixel(s, x);
            ((64 + ((11931 * r + 40137 * g + 4052 * b + 32768) >> 16)) << 6) as u16
        };
        y0[2 * x..2 * x + 2].copy_from_slice(&luma(s0).to_le_bytes());
        y1[2 * x..2 * x + 2].copy_from_slice(&luma(s1).to_le_bytes());
    }
    for x in (0..width).step_by(2) {
        let x1 = (x + 1).min(width - 1);
        let a = pixel(s0, x);
        let b = pixel(s0, x1);
        let c = pixel(s1, x);
        let d = pixel(s1, x1);
        let [r, g, bl] = std::array::from_fn(|i| a[i] + b[i] + c[i] + d[i]);
        // VideoToolbox point-samples instead: T6.14 measures a 43 dB difference.
        let u = ((512 + ((-6576 * r - 22124 * g + 28700 * bl + (1 << 17)) >> 18)) << 6) as u16;
        let v = ((512 + ((28700 * r - 26069 * g - 2631 * bl + (1 << 17)) >> 18)) << 6) as u16;
        uv[2 * x..2 * x + 2].copy_from_slice(&u.to_le_bytes());
        uv[2 * x + 2..2 * x + 4].copy_from_slice(&v.to_le_bytes());
    }
}

type ConvertRows = fn(&[u8], &[u8], &mut [u8], &mut [u8], &mut [u8], usize);

fn convert_bands(
    luma: &mut [u8], chroma: &mut [u8], src: &[u8], stride: usize,
    width: usize, rows: usize, pitch: usize, bytes: usize, convert: ConvertRows, buf: &mut [u8],
) {
    let even_width = (width + 1) & !1;
    let bw = (even_width * bytes).div_ceil(64) * 64;
    // Reuse the C core's cache-sized 32 luma + 16 chroma row scratch buffer.
    let (yb, uvb) = buf.split_at_mut(bw * 32);
    for r0 in (0..rows).step_by(32) {
        let count = (rows - r0).min(32);
        for r in (0..count).step_by(2) {
            let s0 = &src[(r0 + r) * stride..];
            let s1 = &src[(r0 + (r + 1).min(count - 1)) * stride..];
            let (before, after) = yb.split_at_mut((r + 1) * bw);
            convert(s0, s1, &mut before[r * bw..], &mut after[..bw], &mut uvb[(r / 2) * bw..], width);
        }
        in_tile_plane(&mut luma[r0 * pitch..], yb, bw, width * bytes, count, pitch);
        in_tile_plane(&mut chroma[(r0 / 2) * pitch..], uvb, bw, even_width * bytes, count.div_ceil(2), pitch);
    }
}

fn convert_surface(
    luma: &mut [u8], chroma: &mut [u8], src: &[u8], stride: usize,
    width: usize, rows: usize, pitch: usize, bytes: usize, convert: ConvertRows, scratch: &mut [u8],
) {
    if rows == 0 || width == 0 { return; }
    convert_bands(luma, chroma, src, stride, width, rows, pitch, bytes, convert, scratch);
}

pub fn in_tile_bgra(
    dst_luma: &mut [u8], dst_chroma: &mut [u8], src: &[u8], src_stride: usize,
    width: usize, rows: usize, pitch: usize, scratch: &mut [u8],
) {
    convert_surface(dst_luma, dst_chroma, src, src_stride, width, rows, pitch, 1, rgb_rows::<true>, scratch);
}

pub(crate) fn in_tile_rgbx(
    luma:&mut [u8],chroma:&mut [u8],src:&[u8],stride:usize,
    width:usize,rows:usize,pitch:usize,scratch:&mut [u8],
) {
    convert_surface(luma,chroma,src,stride,width,rows,pitch,1,rgb_rows::<false>,scratch);
}

pub fn in_tile_l10r(
    dst_luma: &mut [u8], dst_chroma: &mut [u8], src: &[u8], src_stride: usize,
    width: usize, rows: usize, pitch: usize, scratch: &mut [u8],
) {
    convert_surface(dst_luma, dst_chroma, src, src_stride, width, rows, pitch, 2, l10r_rows, scratch);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_offsets() {
        assert_eq!(bl_offset(32, 0, 128, 2), 256);
        assert_eq!(bl_offset(64, 0, 128, 2), 2048);
        assert_eq!(bl_offset(0, 32, 128, 2), 4096);
        assert_eq!(in_offset(16, 0, 128), 64);
        assert_eq!(in_offset(0, 4, 128), 128);
        assert_eq!(in_offset(0, 16, 128), 2048);
    }

    #[test]
    fn tile_roundtrip_preserves_padding() {
        let (width, rows, stride, pitch) = (73, 67, 89, 128);
        let src: Vec<u8> = (0..rows * stride).map(|i| (i * 37) as u8).collect();
        let mut tiled = vec![0xa5; in_plane_size(pitch, rows)];
        let mut dst = vec![0xcd; rows * stride];
        in_tile_plane(&mut tiled, &src, stride, width, rows, pitch);
        in_untile_plane(&mut dst, stride, &tiled, width, rows, pitch);
        for y in 0..rows {
            assert_eq!(&dst[y * stride..y * stride + width], &src[y * stride..y * stride + width]);
            assert!(dst[y * stride + width..(y + 1) * stride].iter().all(|&v| v == 0xcd));
        }
    }
    #[test]
    fn streaming_tiles_match_scalar_at_all_boundaries() {
        for width in [1usize,15,16,17,63,64,65,127,128,129] {
            for rows in [1,3,4,7,8,15,16,17,31,32,33] {
                let pitch=width.div_ceil(64)*64;
                let stride=width+13;
                let src:Vec<u8>=(0..stride*rows).map(|i|(i*37) as u8).collect();
                let n=in_plane_size(pitch,rows);
                let mut expected=vec![0xa5;n];
                tile_rows(&mut expected,&src,stride,width,rows,pitch);
                for offset in 0..16 {
                    let mut dst=vec![0xa5;n+16];
                    in_tile_plane(&mut dst[offset..offset+n],&src,stride,width,rows,pitch);
                    assert_eq!(&dst[offset..offset+n],expected,"w={width} h={rows} off={offset}");
                    assert!(dst[..offset].iter().all(|&x|x==0xa5));
                    assert!(dst[offset+n..].iter().all(|&x|x==0xa5));
                }
            }
        }
    }
}
