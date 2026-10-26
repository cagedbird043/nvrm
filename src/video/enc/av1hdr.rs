//! AV1 OBUs for NVENC's supported tools; section numbers refer to the AV1 specification.
use super::h264hdr::Bw;
#[derive(Clone, Debug, Default)]
pub struct Av1SeqParams {
    pub width: i32,
    pub height: i32,
    pub bit_depth: i32,
    pub full_range: bool,
    pub colour: [i32; 3],
    pub seq_level_idx: i32,
    pub enable_cdef: bool,
    pub enable_restoration: bool,
    pub order_hint_bits: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Av1FrameParams {
    pub frame_type: i32,
    pub hidden: bool,
    pub order_hint: i32,
    pub refresh_frame_flags: i32,
    pub ref_frame_idx: [i32; 7],
    pub ref_order_hint: [i32; 7],
    pub primary_ref_frame: i32,
    pub error_resilient: bool,
    pub disable_cdf_update: bool,
    pub disable_frame_end_update_cdf: bool,
    pub base_q_idx: i32,
    pub dq_y_dc: i32,
    pub dq_u_dc: i32,
    pub dq_u_ac: i32,
    pub delta_q_present: bool,
    pub lf_level: [i32; 4],
    pub lf_sharpness: i32,
    pub lf_delta_enabled: bool,
    pub cdef_damping: i32,
    pub cdef_bits: i32,
    pub cdef_y: [i32; 8],
    pub cdef_uv: [i32; 8],
    pub lr_type: [i32; 3],
    pub lr_unit_shift: i32,
    pub tx_mode_select: bool,
    pub reference_select: bool,
    pub reduced_tx_set: bool,
    pub interp_filter: i32,
    pub allow_high_precision_mv: bool,
    pub is_motion_mode_switchable: bool,
    pub tile_cols_log2: i32,
    pub tile_rows_log2: i32,
    pub context_update_tile_id: i32,
    pub tile_size_bytes: i32,
}
fn leb128(mut v: usize, out: &mut Vec<u8>) {
    loop {
        let x = (v & 0x7f) as u8;
        v >>= 7;
        out.push(x | if v != 0 { 0x80 } else { 0 });
        if v == 0 {
            break;
        }
    }
}
fn obu(typ: u8, payload: &[u8], out: &mut Vec<u8>) {
    out.push(typ << 3 | 2);
    leb128(payload.len(), out);
    out.extend_from_slice(payload);
}
pub fn level_for(width: i32, height: i32) -> i32 {
    let ps = width as i64 * height as i64;
    if ps <= 665856 {
        4
    } else if ps <= 1065024 {
        5
    } else if ps <= 2359296 {
        9
    } else if ps <= 8912896 {
        13
    } else {
        17
    }
}
pub fn write_temporal_delimiter(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x12, 0]);
}
fn bits_for(v: i32) -> i32 {
    let mut n = 1;
    while (1i64 << n) <= v as i64 {
        n += 1;
    }
    n
}
pub fn write_sequence_header(s: &Av1SeqParams, out: &mut Vec<u8>) {
    let mut b = Bw::<64>::new();
    b.put(0, 3);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 5);
    b.put(0, 12);
    let lvl = if s.seq_level_idx != 0 {
        s.seq_level_idx
    } else {
        level_for(s.width, s.height)
    };
    b.put(lvl as u32, 5);
    if lvl > 7 {
        b.put(0, 1);
    }
    let wb = bits_for(s.width - 1);
    let hb = bits_for(s.height - 1);
    b.put(wb as u32 - 1, 4);
    b.put(hb as u32 - 1, 4);
    b.put(s.width as u32 - 1, wb);
    b.put(s.height as u32 - 1, hb);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(1, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(
        if s.order_hint_bits != 0 {
            s.order_hint_bits
        } else {
            7
        } as u32
            - 1,
        3,
    );
    b.put(0, 1);
    b.flag(s.enable_cdef);
    b.flag(s.enable_restoration);
    b.flag(s.bit_depth > 8);
    b.put(0, 1);
    b.put(1, 1);
    for c in s.colour {
        b.put(if c != 0 { c } else { 1 } as u32, 8);
    }
    b.flag(s.full_range);
    b.put(0, 2);
    b.put(0, 1);
    b.put(0, 1);
    b.trailing();
    obu(1, b.bytes(), out);
}
fn tile_log2(blk: i32, target: i32) -> i32 {
    let mut k = 0;
    while (blk << k) < target {
        k += 1;
    }
    k
}
fn delta_q(b: &mut Bw<512>, v: i32) {
    b.flag(v != 0);
    if v != 0 {
        b.put(v as u32 & 127, 7);
    }
}
pub fn write_frame(s: &Av1SeqParams, f: &Av1FrameParams, tiles: &[&[u8]], out: &mut Vec<u8>) {
    let mut b = Bw::<512>::new();
    let key = f.frame_type == 0;
    let intra = key;
    let ohb = if s.order_hint_bits != 0 {
        s.order_hint_bits
    } else {
        7
    };
    b.put(0, 1);
    b.put(f.frame_type as u32, 2);
    b.flag(!f.hidden);
    if f.hidden {
        b.put(1, 1);
    }
    let er = key || f.error_resilient;
    if !key {
        b.flag(f.error_resilient);
    }
    b.flag(f.disable_cdf_update);
    b.put(0, 1);
    b.put(f.order_hint as u32 & ((1 << ohb) - 1), ohb);
    if !(intra || er) {
        b.put(f.primary_ref_frame as u32, 3);
    }
    if !key {
        b.put(f.refresh_frame_flags as u32, 8);
    }
    if !intra && er && f.refresh_frame_flags != 0xff {
        for _ in 0..8 {
            b.put(0, ohb);
        }
    }
    if key {
        b.put(0, 1);
    } else {
        b.put(0, 1);
        for v in f.ref_frame_idx {
            b.put(v as u32, 3);
        }
        b.put(0, 1);
        b.flag(f.allow_high_precision_mv);
        b.flag(f.interp_filter == 4);
        if f.interp_filter != 4 {
            b.put(f.interp_filter as u32, 2);
        }
        b.flag(f.is_motion_mode_switchable);
    }
    if !f.disable_cdf_update {
        b.flag(f.disable_frame_end_update_cdf);
    }
    // Uniform spacing tile_info (5.9.15), 64x64 superblocks.
    let mi_cols = 2 * ((s.width + 7) >> 3);
    let mi_rows = 2 * ((s.height + 7) >> 3);
    let sb_cols = (mi_cols + 15) >> 4;
    let sb_rows = (mi_rows + 15) >> 4;
    let min_cols = tile_log2(64, sb_cols);
    let max_cols = tile_log2(1, sb_cols.min(64));
    let max_rows = tile_log2(1, sb_rows.min(64));
    let min_tiles = min_cols.max(tile_log2((4096 * 2304) >> 12, sb_rows * sb_cols));
    b.put(1, 1);
    let mut cl = min_cols;
    while cl < max_cols {
        let inc = cl < f.tile_cols_log2;
        b.flag(inc);
        if !inc {
            break;
        }
        cl += 1;
    }
    let mut rl = (min_tiles - cl).max(0);
    while rl < max_rows {
        let inc = rl < f.tile_rows_log2;
        b.flag(inc);
        if !inc {
            break;
        }
        rl += 1;
    }
    if cl != 0 || rl != 0 {
        b.put(f.context_update_tile_id as u32, cl + rl);
        b.put(f.tile_size_bytes as u32 - 1, 2);
    }
    b.put(f.base_q_idx as u32, 8);
    delta_q(&mut b, f.dq_y_dc);
    delta_q(&mut b, f.dq_u_dc);
    delta_q(&mut b, f.dq_u_ac);
    b.put(0, 1);
    b.put(0, 1);
    if f.base_q_idx > 0 {
        b.flag(f.delta_q_present);
        if f.delta_q_present {
            b.put(0, 2);
            b.put(0, 1);
        }
    }
    let lossless = f.base_q_idx == 0 && f.dq_y_dc == 0 && f.dq_u_dc == 0 && f.dq_u_ac == 0;
    if !lossless {
        b.put(f.lf_level[0] as u32, 6);
        b.put(f.lf_level[1] as u32, 6);
        if f.lf_level[0] != 0 || f.lf_level[1] != 0 {
            b.put(f.lf_level[2] as u32, 6);
            b.put(f.lf_level[3] as u32, 6);
        }
        b.put(f.lf_sharpness as u32, 3);
        b.flag(f.lf_delta_enabled);
        if f.lf_delta_enabled {
            b.put(0, 1);
        }
        if s.enable_cdef {
            b.put(f.cdef_damping as u32 - 3, 2);
            b.put(f.cdef_bits as u32, 2);
            for i in 0..(1 << f.cdef_bits) {
                b.put(f.cdef_y[i] as u32 >> 2, 4);
                b.put(f.cdef_y[i] as u32 & 3, 2);
                b.put(f.cdef_uv[i] as u32 >> 2, 4);
                b.put(f.cdef_uv[i] as u32 & 3, 2);
            }
        }
        if s.enable_restoration {
            for v in f.lr_type {
                b.put(v as u32, 2);
            }
            if f.lr_type.iter().any(|&v| v != 0) {
                b.flag(f.lr_unit_shift > 0);
                if f.lr_unit_shift > 0 {
                    b.flag(f.lr_unit_shift > 1);
                }
                if f.lr_type[1] != 0 || f.lr_type[2] != 0 {
                    b.put(0, 1);
                }
            }
        }
        b.flag(f.tx_mode_select);
    }
    if !intra {
        b.flag(f.reference_select);
    }
    if !intra && f.reference_select {
        // skip_mode_params (5.9.22), including wrapped order hints.
        let m = 1 << (ohb - 1);
        let mask = (1 << ohb) - 1;
        let rel = |a: i32, z: i32| ((a - z) & (m - 1)) - ((a - z) & m);
        let mut fwd = None;
        let mut bwd = None;
        for &v in &f.ref_order_hint {
            let h = v & mask;
            let oh = f.order_hint & mask;
            if rel(h, oh) < 0 {
                if fwd.is_none_or(|v| rel(h, v) > 0) {
                    fwd = Some(h);
                }
            } else if rel(h, oh) > 0 && bwd.is_none_or(|v| rel(h, v) < 0) {
                bwd = Some(h);
            }
        }
        let allowed = if let Some(fwd) = fwd {
            bwd.is_some() || f.ref_order_hint.iter().any(|&h| rel(h & mask, fwd) < 0)
        } else {
            false
        };
        if allowed {
            b.put(0, 1);
        }
    }
    b.flag(f.reduced_tx_set);
    if !intra {
        for _ in 0..7 {
            b.put(0, 1);
        }
    }
    b.align();
    if tiles.len() > 1 {
        b.put(0, 1);
        b.align();
    }
    // Emit directly into the caller's buffer, without a temporary copy of tile data.
    let payload_len = b.bytes().len()
        + tiles.iter().map(|t| t.len()).sum::<usize>()
        + tiles.len().saturating_sub(1) * f.tile_size_bytes as usize;
    out.push(6 << 3 | 2);
    leb128(payload_len, out);
    out.extend_from_slice(b.bytes());
    for (i, tile) in tiles.iter().enumerate() {
        if i + 1 < tiles.len() {
            let sz = tile.len() - 1;
            for k in 0..f.tile_size_bytes {
                out.push((sz >> (8 * k)) as u8);
            }
        }
        out.extend_from_slice(tile);
    }
}
pub fn write_show_existing(slot: i32, out: &mut Vec<u8>) {
    let mut b = Bw::<4>::new();
    b.put(1, 1);
    b.put(slot as u32, 3);
    b.trailing();
    obu(3, b.bytes(), out);
}
/// Convert SEI G/B/R primaries (0.00002), luminance (0.0001 cd/m2) to AV1 R/G/B, 0.16, 24.8, 18.14.
pub fn write_hdr_metadata(mdcv: Option<&[u8; 24]>, cll: Option<&[u8; 4]>, out: &mut Vec<u8>) {
    if let Some(v) = mdcv {
        let be16 = |i| u16::from_be_bytes([v[i], v[i + 1]]) as u64;
        let be32 = |i| u32::from_be_bytes([v[i], v[i + 1], v[i + 2], v[i + 3]]) as u64;
        let mut b = Bw::<32>::new();
        b.put(2, 8);
        for i in [2, 0, 1] {
            b.put(((be16(4 * i) * 131072 + 50000) / 100000) as u32, 16);
            b.put(((be16(4 * i + 2) * 131072 + 50000) / 100000) as u32, 16);
        }
        b.put(((be16(12) * 131072 + 50000) / 100000) as u32, 16);
        b.put(((be16(14) * 131072 + 50000) / 100000) as u32, 16);
        b.put(((be32(16) * 256 + 5000) / 10000) as u32, 32);
        b.put(((be32(20) * 16384 + 5000) / 10000) as u32, 32);
        b.trailing();
        obu(5, b.bytes(), out);
    }
    if let Some(v) = cll {
        let mut b = Bw::<8>::new();
        b.put(1, 8);
        b.put(u16::from_be_bytes([v[0], v[1]]) as u32, 16);
        b.put(u16::from_be_bytes([v[2], v[3]]) as u32, 16);
        b.trailing();
        obu(5, b.bytes(), out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Golden bytes generated by linking the original mickys C implementation.
    #[test]
    fn c_golden_bytes() {
        let mut out = vec![0x5a];
        let s = Av1SeqParams {
            width: 63,
            height: 65,
            bit_depth: 8,
            full_range: false,
            colour: [0, 0, 0],
            seq_level_idx: 0,
            enable_cdef: false,
            enable_restoration: false,
            order_hint_bits: 0,
            ..Default::default()
        };
        write_sequence_header(&s, &mut out);
        assert_eq!(out[0], 0x5a);
        assert_eq!(
            &out[1..],
            &[
                0x0a, 0x0d, 0x00, 0x00, 0x00, 0x22, 0xb7, 0xd0, 0x00, 0x21, 0x81, 0x01, 0x01, 0x01,
                0x04
            ]
        );
        out.truncate(1);
        let s = Av1SeqParams {
            width: 63,
            height: 65,
            bit_depth: 8,
            full_range: false,
            colour: [9, 16, 9],
            seq_level_idx: 0,
            enable_cdef: false,
            enable_restoration: false,
            order_hint_bits: 1,
            ..Default::default()
        };
        let f = Av1FrameParams {
            frame_type: 1,
            hidden: false,
            order_hint: 1,
            refresh_frame_flags: 5,
            ref_frame_idx: [1, 1, 3, 4, 6, 7, 6],
            ref_order_hint: [91, 94, 147, 150, 110, 145, 69],
            primary_ref_frame: 1,
            error_resilient: false,
            disable_cdf_update: false,
            disable_frame_end_update_cdf: false,
            base_q_idx: 1,
            dq_y_dc: -1,
            dq_u_dc: -3,
            dq_u_ac: 5,
            delta_q_present: false,
            lf_level: [20, 46, 52, 61],
            lf_sharpness: 1,
            lf_delta_enabled: false,
            cdef_damping: 4,
            cdef_bits: 1,
            cdef_y: [52, 41, 43, 19, 7, 13, 44, 63],
            cdef_uv: [17, 19, 5, 15, 39, 28, 40, 17],
            lr_type: [3, 1, 1],
            lr_unit_shift: 1,
            tx_mode_select: false,
            reference_select: false,
            reduced_tx_set: false,
            interp_filter: 1,
            allow_high_precision_mv: false,
            is_motion_mode_switchable: false,
            tile_cols_log2: 0,
            tile_rows_log2: 0,
            context_update_tile_id: 0,
            tile_size_bytes: 2,
            ..Default::default()
        };
        write_frame(
            &s,
            &f,
            &[&[0, 1, 2, 3, 0, 0, 1][..]],
            &mut out,
        );
        assert_eq!(out[0], 0x5a);
        assert_eq!(
            &out[1..],
            &[
                0x32, 0x17, 0x31, 0x20, 0xa2, 0x5c, 0xdf, 0x04, 0x80, 0x7f, 0xff, 0x61, 0x42, 0x97,
                0x69, 0xe9, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x00, 0x00, 0x01
            ]
        );
        out.truncate(1);
    }
}
