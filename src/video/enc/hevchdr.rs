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

/// The selected ME/MD/Q mode and the SPS must share this minimum coding unit.
pub(crate) fn min_cb_log2(c: &super::EncConfig) -> u32 {
    if c.quality && !c.no_cu8 { 3 } else { 4 }
}

pub(crate) fn config_params(c: &super::EncConfig) -> HevcParams {
    HevcParams {
        width: c.width, height: c.height, log2_max_poc_lsb_minus4: 4,
        max_dec_pic_buffering: c.num_refs + 1 + c.dpb_extra, sps_rps_negative: c.num_refs,
        num_reorder: i32::from(c.bframes > 0), num_ref_idx_l0_default: c.num_refs,
        pic_init_qp: 26, vui: true, full_range: c.full_range, colour: c.colour,
        min_cb8: min_cb_log2(c) == 3, bit_depth: c.bit_depth, chroma444: c.chroma444,
        lossless: c.lossless, no_sdh: true, no_ts: c.bit_depth > 8,
        ..Default::default()
    }
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
    #[test]
    fn generated_parameters_preserve_cropped_extent_and_ten_bit_444_tools() {
        use crate::video::{bits, hevc::ps};
        let p = HevcParams { width: 1919, height: 1079, log2_max_poc_lsb_minus4: 4,
            max_dec_pic_buffering: 3, num_reorder: 1, sps_rps_negative: 3,
            num_ref_idx_l0_default: 1, pic_init_qp: 0, vui: true, full_range: true,
            colour: [9, 16, 9], bit_depth: 10, chroma444: true, lossless: true,
            min_cb8: true, no_sdh: true, no_ts: true, no_cqd: true,
            no_sao: true, no_amp: true, sis: true, ..Default::default() };
        let mut bytes = Vec::new(); write_vps(&p, &mut bytes); write_sps(&p, &mut bytes); write_pps(&p, &mut bytes);
        let units = bits::split_annexb(&bytes);
        let sps = ps::parse_sps_for_encode(&bits::unescape(units[1].nal)).unwrap();
        let pps = ps::parse_pps_for_encode(&bits::unescape(units[2].nal)).unwrap();
        assert_eq!(sps.width - sps.conf_win[0] - sps.conf_win[1], p.width as u32);
        assert_eq!(sps.height - sps.conf_win[2] - sps.conf_win[3], p.height as u32);
        assert_eq!((sps.bit_depth_luma, sps.chroma_format_idc), (10, 3));
        assert_eq!(sps.log2_min_cb, 3); assert!(!sps.sao && !sps.amp);
        assert_eq!(pps.init_qp_minus26 + 26, p.pic_init_qp);
        assert!(pps.transquant_bypass && !pps.cu_qp_delta && !pps.sign_data_hiding && !pps.transform_skip);
    }
}

fn typed_ptl(b: &mut Bw<192>, s: &crate::video::hevc::ps::Sps) {
    b.put(0, 2); b.flag(s.encoder.tier); b.put(s.profile_idc, 5);
    b.put(match s.profile_idc { 1 => 0x6000_0000, 2 => 0x2000_0000, 3 => 0x7000_0000, _ => 0x0800_0000 }, 32);
    b.put(9, 4);
    if s.profile_idc == 4 {
        b.flag(true); b.flag(true); b.flag(s.bit_depth_luma == 8);
        b.put(0, 5); b.flag(true); b.put(0, 32); b.put(0, 2);
    } else { b.put(0, 32); b.put(0, 11); }
    b.flag(false);
    b.put(u32::from(s.encoder.level_idc.unwrap_or_else(|| level_for(s.width as i32, s.height as i32) as u8)), 8);
}

fn typed_timing(b: &mut Bw<192>, t: &crate::video::hevc::ps::EncoderTiming) {
    b.put(t.num_units_in_tick, 32); b.put(t.time_scale, 32); b.flag(t.poc_ticks.is_some());
    if let Some(ticks) = t.poc_ticks { b.ue((ticks - 1) as u32); }
}
fn typed_hrd(b: &mut Bw<192>, h: &crate::video::hevc::ps::EncoderHrd) {
    b.flag(!h.nal.is_empty()); b.flag(!h.vcl.is_empty()); b.flag(false);
    b.put(0, 4); b.put(0, 4);
    b.put(h.initial_cpb_removal_delay_length - 1, 5); b.put(h.au_cpb_removal_delay_length - 1, 5);
    b.put(h.dpb_output_delay_length - 1, 5);
    b.flag(h.fixed_duration.is_some());
    if let Some(duration) = h.fixed_duration { b.ue((duration - 1) as u32); }
    else { b.flag(false); b.flag(h.low_delay); }
    if !h.low_delay { b.ue(h.nal.len().max(h.vcl.len()) as u32 - 1); }
    for cpb in [&h.nal, &h.vcl] {
        for &(rate, size, cbr) in cpb { b.ue((rate >> 6) as u32 - 1); b.ue((size >> 4) as u32 - 1); b.flag(cbr); }
    }
}

pub(crate) fn write_typed_vps(s: &crate::video::hevc::ps::Sps, id: u32, out: &mut Vec<u8>) {
    let mut b = Bw::<192>::new();
    b.put(id, 4); b.put(3, 2); b.put(0, 6); b.put(0, 3); b.flag(true); b.put(0xffff, 16);
    typed_ptl(&mut b, s); b.flag(false);
    b.ue(s.max_dec_pic_buffering - 1); b.ue(s.max_num_reorder_pics); b.ue(0);
    b.put(0, 6); b.ue(0);
    let timing = s.encoder.vps_timing.as_ref().or(s.encoder.timing.as_ref()); b.flag(timing.is_some());
    if let Some(t) = timing {
        typed_timing(&mut b, t); b.ue(u32::from(s.encoder.vps_hrd.is_some()));
        if let Some(h) = &s.encoder.vps_hrd { b.ue(0); typed_hrd(&mut b, h); }
    }
    b.flag(false); b.trailing(); emit(&b, 32, out);
}
pub(crate) fn write_typed_sps(s: &crate::video::hevc::ps::Sps, id: u32, vps_id: u32, out: &mut Vec<u8>) {
    let mut b = Bw::<192>::new();
    b.put(vps_id, 4); b.put(0, 3); b.flag(true); typed_ptl(&mut b, s);
    b.ue(id); b.ue(s.chroma_format_idc); if s.chroma_format_idc == 3 { b.flag(s.separate_colour_plane); }
    b.ue(s.width); b.ue(s.height); let crop = s.conf_win != [0; 4]; b.flag(crop);
    if crop { for value in s.conf_win { b.ue(value); } }
    b.ue(s.bit_depth_luma - 8); b.ue(s.bit_depth_chroma - 8); b.ue(s.log2_max_poc_lsb - 4);
    b.flag(false); b.ue(s.max_dec_pic_buffering - 1); b.ue(s.max_num_reorder_pics); b.ue(0);
    b.ue(s.log2_min_cb - 3); b.ue(s.log2_ctb - s.log2_min_cb); b.ue(s.log2_min_tb - 2);
    b.ue(s.log2_max_tb - s.log2_min_tb); b.ue(s.max_th_depth_inter); b.ue(s.max_th_depth_intra);
    b.flag(false); b.flag(s.amp); b.flag(s.sao); b.flag(false);
    b.ue(s.st_rps.len() as u32);
    for (i, rps) in s.st_rps.iter().enumerate() {
        if i != 0 { b.flag(false); }
        b.ue(rps.s0.len() as u32); b.ue(rps.s1.len() as u32);
        let mut prev = 0;
        for &(delta, used) in &rps.s0 { b.ue((prev - delta - 1) as u32); b.flag(used); prev = delta; }
        prev = 0;
        for &(delta, used) in &rps.s1 { b.ue((delta - prev - 1) as u32); b.flag(used); prev = delta; }
    }
    b.flag(false); b.flag(s.temporal_mvp); b.flag(s.strong_intra_smoothing);
    b.flag(true); b.flag(s.encoder.aspect_ratio_idc.is_some());
    if let Some(idc) = s.encoder.aspect_ratio_idc {
        b.put(u32::from(idc), 8);
        if idc == 255 { for size in s.encoder.sar { b.put(u32::from(size), 16); } }
    }
    b.flag(false);
    b.flag(true); b.put(5, 3); b.flag(s.encoder.full_range); b.flag(s.encoder.colour.is_some());
    if let Some(colour) = s.encoder.colour { for c in colour { b.put(u32::from(c), 8); } }
    b.flag(false); b.put(0, 3); b.flag(false);
    b.flag(s.encoder.timing.is_some());
    if let Some(t) = &s.encoder.timing {
        typed_timing(&mut b, t); b.flag(s.encoder.hrd.is_some());
        if let Some(h) = &s.encoder.hrd { typed_hrd(&mut b, h); }
    }
    b.flag(false); b.flag(false); b.trailing(); emit(&b, 33, out);
}

pub(crate) fn write_typed_pps(p: &crate::video::hevc::ps::Pps, out: &mut Vec<u8>) {
    let mut b = Bw::<192>::new();
    b.ue(p.id); b.ue(p.sps_id); b.flag(p.dependent_slice_segments_enabled); b.flag(p.output_flag_present);
    b.put(p.num_extra_slice_header_bits, 3); b.flag(p.sign_data_hiding); b.flag(p.cabac_init_present);
    b.ue(p.num_ref_idx_l0_default - 1); b.ue(p.num_ref_idx_l1_default - 1); b.se(p.init_qp_minus26);
    b.flag(p.constrained_intra_pred); b.flag(p.transform_skip); b.flag(p.cu_qp_delta);
    if p.cu_qp_delta { b.ue(p.diff_cu_qp_delta_depth); }
    b.se(p.cb_qp_offset); b.se(p.cr_qp_offset); b.flag(p.slice_chroma_qp_offsets_present);
    b.flag(p.weighted_pred); b.flag(p.weighted_bipred); b.flag(p.transquant_bypass); b.flag(false); b.flag(p.entropy_coding_sync);
    b.flag(p.loop_filter_across_slices); b.flag(p.deblocking_control_present);
    if p.deblocking_control_present {
        b.flag(p.deblocking_override_enabled); b.flag(p.deblocking_disabled);
        if !p.deblocking_disabled { b.se(p.beta_offset_div2); b.se(p.tc_offset_div2); }
    }
    b.flag(false); b.flag(p.lists_modification_present); b.ue(p.log2_parallel_merge_level - 2);
    b.flag(p.slice_header_extension_present); b.flag(false); b.trailing(); emit(&b, 34, out);
}
