//! HEVC parameter sets (ITU-T H.265 7.3).
use super::h264hdr::{escape, Bw};
#[derive(Clone, Debug, Default)]
pub struct HevcParams {
    pub width: i32,
    pub height: i32,
    pub level_idc: i32,
    pub log2_max_poc_lsb_minus4: i32,
    pub max_dec_pic_buffering: i32,
    pub num_reorder: i32,
    pub sps_rps_negative: i32,
    pub num_ref_idx_l0_default: i32,
    pub pic_init_qp: i32,
    pub vui: bool,
    pub full_range: bool,
    pub colour: [i32; 3],
    pub bit_depth: i32,
    pub chroma444: bool,
    pub lossless: bool,
    pub min_cb8: bool,
    pub no_sdh: bool,
    pub no_ts: bool,
    pub no_cqd: bool,
    pub no_sao: bool,
    pub no_amp: bool,
    pub sis: bool,
}
pub fn level_for(width: i32, height: i32) -> i32 {
    let ps = width as i64 * height as i64;
    let sr = ps * 60;
    if ps <= 552960 && sr <= 33177600 {
        93
    } else if ps <= 2228224 && sr <= 133693440 {
        123
    } else if ps <= 8912896 && sr <= 534773760 {
        153
    } else {
        183
    }
}
fn level(p: &HevcParams) -> i32 {
    if p.level_idc != 0 {
        p.level_idc
    } else {
        level_for(p.width, p.height)
    }
}
// Whole 32x32 CTBs: NVENC codes partial CTBs as whole; crop with the conformance window (T6.06).
pub fn coded_width(p: &HevcParams) -> usize {
    (p.width as usize + 31) & !31
}
pub fn coded_height(p: &HevcParams) -> usize {
    (p.height as usize + 31) & !31
}
fn profile_tier_level(b: &mut Bw<192>, level: i32, ten: bool, rext: bool) {
    b.put(0, 2);
    b.put(0, 1);
    b.put(
        if rext {
            4
        } else if ten {
            2
        } else {
            1
        },
        5,
    );
    b.put(
        if rext {
            0x08000000
        } else if ten {
            0x20000000
        } else {
            0x60000000
        },
        32,
    );
    b.put(1, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.put(1, 1);
    if rext {
        b.put(1, 1);
        b.put(1, 1);
        b.flag(!ten);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(1, 1);
        b.put(0, 32);
        b.put(0, 2);
    } else {
        b.put(0, 32);
        b.put(0, 11);
    }
    b.put(0, 1);
    b.put(level as u32, 8);
}
fn emit(b: &Bw<192>, typ: u8, out: &mut Vec<u8>) {
    out.extend_from_slice(&[0, 0, 0, 1, typ << 1, 1]);
    escape(b.bytes(), out);
}
pub fn write_vps(p: &HevcParams, out: &mut Vec<u8>) {
    let mut b = Bw::<192>::new();
    b.put(0, 4);
    b.put(1, 1);
    b.put(1, 1);
    b.put(0, 6);
    b.put(0, 3);
    b.put(1, 1);
    b.put(0xffff, 16);
    profile_tier_level(&mut b, level(p), p.bit_depth > 8, p.chroma444);
    b.put(1, 1);
    b.ue(p.max_dec_pic_buffering as u32 - 1);
    b.ue(p.num_reorder as u32);
    b.ue(0);
    b.put(0, 6);
    b.ue(0);
    b.put(0, 1);
    b.put(0, 1);
    b.trailing();
    emit(&b, 32, out);
}
pub fn write_sps(p: &HevcParams, out: &mut Vec<u8>) {
    let mut b = Bw::<192>::new();
    let cw = coded_width(p) as u32;
    let ch = coded_height(p) as u32;
    b.put(0, 4);
    b.put(0, 3);
    b.put(1, 1);
    profile_tier_level(&mut b, level(p), p.bit_depth > 8, p.chroma444);
    b.ue(0);
    b.ue(if p.chroma444 { 3 } else { 1 });
    if p.chroma444 {
        b.put(0, 1);
    }
    b.ue(cw);
    b.ue(ch);
    let u = if p.chroma444 { 1 } else { 2 };
    let cr = (cw - p.width as u32) / u;
    let cb = (ch - p.height as u32) / u;
    b.flag(cr != 0 || cb != 0);
    if cr != 0 || cb != 0 {
        b.ue(0);
        b.ue(cr);
        b.ue(0);
        b.ue(cb);
    }
    let bd = if p.bit_depth > 8 {
        p.bit_depth as u32 - 8
    } else {
        0
    };
    b.ue(bd);
    b.ue(bd);
    b.ue(p.log2_max_poc_lsb_minus4 as u32);
    b.put(1, 1);
    b.ue(p.max_dec_pic_buffering as u32 - 1);
    b.ue(p.num_reorder as u32);
    b.ue(0);
    b.ue(if p.min_cb8 { 0 } else { 1 });
    b.ue(if p.min_cb8 { 2 } else { 1 });
    b.ue(0);
    b.ue(3);
    b.ue(3);
    b.ue(3);
    b.put(0, 1);
    b.flag(!p.no_amp);
    b.flag(!p.no_sao);
    b.put(0, 1);
    b.ue(1);
    b.ue(p.sps_rps_negative as u32);
    b.ue(0);
    for _ in 0..p.sps_rps_negative {
        b.ue(0);
        b.put(0, 1);
    }
    b.put(0, 1);
    b.put(0, 1);
    b.flag(p.sis);
    b.flag(p.vui);
    if p.vui {
        b.put(1, 1);
        b.put(1, 8);
        b.put(0, 1);
        b.put(1, 1);
        b.put(5, 3);
        b.flag(p.full_range);
        b.put(1, 1);
        for c in p.colour {
            b.put(if c != 0 { c } else { 1 } as u32, 8);
        }
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
    }
    b.put(0, 1);
    b.trailing();
    emit(&b, 33, out);
}
pub fn write_pps(p: &HevcParams, out: &mut Vec<u8>) {
    let mut b = Bw::<192>::new();
    b.ue(0);
    b.ue(0);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 3);
    b.flag(!p.no_sdh);
    b.put(1, 1);
    b.ue(if p.num_ref_idx_l0_default != 0 {
        p.num_ref_idx_l0_default
    } else {
        1
    } as u32
        - 1);
    b.ue(0);
    b.se(p.pic_init_qp - 26);
    b.put(0, 1);
    b.flag(!p.no_ts);
    b.flag(!p.no_cqd);
    if !p.no_cqd {
        b.ue(0);
    }
    b.se(0);
    b.se(0);
    b.put(0, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.flag(p.lossless);
    b.put(0, 1);
    b.put(0, 1);
    b.put(1, 1);
    b.put(1, 1);
    b.put(0, 1);
    b.put(0, 1);
    b.se(0);
    b.se(0);
    b.put(0, 1);
    b.put(0, 1);
    b.ue(0);
    b.put(0, 1);
    b.put(0, 1);
    b.trailing();
    emit(&b, 34, out);
}

#[cfg(test)]
mod tests {
    use super::*;
    // Golden bytes generated by linking the original mickys C implementation.
    #[test]
    fn c_golden_bytes() {
        let mut out = vec![0x5a];
        let p = HevcParams {
            width: 1919,
            height: 1079,
            level_idc: 0,
            log2_max_poc_lsb_minus4: 4,
            max_dec_pic_buffering: 3,
            num_reorder: 1,
            sps_rps_negative: 3,
            num_ref_idx_l0_default: 1,
            pic_init_qp: 0,
            vui: true,
            full_range: true,
            colour: [9, 16, 9],
            bit_depth: 10,
            chroma444: true,
            lossless: true,
            min_cb8: true,
            no_sdh: true,
            no_ts: true,
            no_cqd: true,
            no_sao: true,
            no_amp: true,
            sis: true,
            ..Default::default()
        };
        write_vps(&p, &mut out);
        write_sps(&p, &mut out);
        write_pps(&p, &mut out);
        assert_eq!(out[0], 0x5a);
        assert_eq!(
            &out[1..],
            &[
                0x00, 0x00, 0x00, 0x01, 0x40, 0x01, 0x0c, 0x01, 0xff, 0xff, 0x04, 0x08, 0x00, 0x00,
                0x03, 0x00, 0x9c, 0x08, 0x00, 0x00, 0x03, 0x00, 0x00, 0x7b, 0xb5, 0x02, 0x40, 0x00,
                0x00, 0x00, 0x01, 0x42, 0x01, 0x01, 0x04, 0x08, 0x00, 0x00, 0x03, 0x00, 0x9c, 0x08,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x7b, 0x90, 0x00, 0x78, 0x10, 0x02, 0x20, 0xea, 0x29,
                0xb2, 0xda, 0xdc, 0x84, 0x20, 0x22, 0x6a, 0x38, 0x0b, 0x70, 0x91, 0x00, 0x90, 0x08,
                0x00, 0x00, 0x00, 0x01, 0x44, 0x01, 0xc0, 0xe0, 0xd4, 0x62, 0x66, 0x48
            ]
        );
        out.truncate(1);
    }
}
