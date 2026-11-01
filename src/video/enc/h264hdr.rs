//! H.264 SPS/PPS (ITU-T H.264 7.3.2.1/7.3.2.2) and HDR SEI.
#[derive(Clone, Debug, Default)]
pub struct H264Params {
    pub width: i32,
    pub height: i32,
    pub profile_idc: i32,
    pub level_idc: i32,
    pub constraint_flags: i32,
    pub log2_max_frame_num_minus4: i32,
    pub log2_max_poc_lsb_minus4: i32,
    pub poc_type: i32,
    pub num_reorder: i32,
    pub num_ref_idx_l0_minus1: i32,
    pub num_ref_frames: i32,
    pub entropy_cabac: bool,
    pub pic_init_qp: i32,
    pub transform_8x8: bool,
    pub vui: bool,
    pub full_range: bool,
    pub colour: [i32; 3],
    pub chroma444: bool,
    pub lossless: bool,
}

pub(crate) fn config_params(c: &super::EncConfig) -> H264Params {
    let cbp = !c.chroma444 && !c.lossless && c.h264_profile == 66;
    H264Params {
        width: c.width, height: c.height, colour: c.colour,
        profile_idc: if c.chroma444 || c.lossless { 244 } else if c.h264_profile == 100 { 100 } else if cbp { 66 } else { 77 },
        constraint_flags: if cbp { 0xc0 } else { 0 },
        level_idc: if c.h264_level != 0 { c.h264_level } else if ((c.width + 15) / 16) * ((c.height + 15) / 16) > 36864 { 60 } else { 51 },
        chroma444: c.chroma444, lossless: c.lossless,
        log2_max_frame_num_minus4: 4, poc_type: if c.bframes > 0 { 0 } else { 2 },
        log2_max_poc_lsb_minus4: 4, num_reorder: i32::from(c.bframes > 0),
        num_ref_frames: c.num_refs, num_ref_idx_l0_minus1: c.num_refs - 1,
        entropy_cabac: c.cabac, transform_8x8: c.h264_profile == 100 || c.chroma444,
        pic_init_qp: 26, vui: true, full_range: c.full_range,
    }
}

pub(crate) struct Bw<const N: usize> {
    pub(crate) buf: [u8; N],
    pub(crate) bits: usize,
}
impl<const N: usize> Bw<N> {
    pub(crate) fn new() -> Self {
        Self {
            buf: [0; N],
            bits: 0,
        }
    }
    pub(crate) fn put(&mut self, v: u32, n: i32) {
        for i in (0..n).rev() {
            if (v >> i) & 1 != 0 {
                self.buf[self.bits >> 3] |= 0x80 >> (self.bits & 7);
            }
            self.bits += 1;
        }
    }
    pub(crate) fn flag(&mut self, v: bool) {
        self.put(v as u32, 1);
    }
    pub(crate) fn ue(&mut self, v: u32) {
        let x = v.wrapping_add(1);
        let n = 31 - x.leading_zeros();
        self.put(0, n as i32);
        self.put(x, n as i32 + 1);
    }
    pub(crate) fn se(&mut self, v: i32) {
        self.ue(if v > 0 {
            2 * v as u32 - 1
        } else {
            2 * (-v) as u32
        });
    }
    pub(crate) fn align(&mut self) {
        while self.bits & 7 != 0 {
            self.put(0, 1);
        }
    }
    pub(crate) fn trailing(&mut self) {
        self.put(1, 1);
        self.align();
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.buf[..self.bits / 8]
    }
}
pub(crate) fn escape(rbsp: &[u8], out: &mut Vec<u8>) {
    let mut zeros = 0;
    for &v in rbsp {
        if zeros >= 2 && v <= 3 {
            out.push(3);
            zeros = 0;
        }
        out.push(v);
        zeros = if v == 0 { zeros + 1 } else { 0 };
    }
}
fn emit(b: &Bw<128>, header: u8, out: &mut Vec<u8>) {
    out.extend_from_slice(&[0, 0, 0, 1, header]);
    escape(b.bytes(), out);
}
pub fn write_sps(p: &H264Params, out: &mut Vec<u8>) {
    let mut b = Bw::<128>::new();
    let wmb = (p.width + 15) / 16;
    let hmb = (p.height + 15) / 16;
    b.put(p.profile_idc as u32, 8);
    b.put(p.constraint_flags as u32, 8);
    b.put(p.level_idc as u32, 8);
    b.ue(0);
    if p.profile_idc >= 100 {
        b.ue(if p.chroma444 { 3 } else { 1 });
        if p.chroma444 {
            b.put(0, 1);
        }
        b.ue(0);
        b.ue(0);
        b.flag(p.lossless);
        b.put(0, 1);
    }
    b.ue(p.log2_max_frame_num_minus4 as u32);
    b.ue(p.poc_type as u32);
    if p.poc_type == 0 {
        b.ue(p.log2_max_poc_lsb_minus4 as u32);
    }
    b.ue(p.num_ref_frames as u32);
    b.put(0, 1);
    b.ue(wmb as u32 - 1);
    b.ue(hmb as u32 - 1);
    b.put(1, 1);
    b.put(1, 1);
    let cu = if p.chroma444 { 1 } else { 2 };
    let cr = (wmb * 16 - p.width) / cu;
    let cb = (hmb * 16 - p.height) / cu;
    b.flag(cr != 0 || cb != 0);
    if cr != 0 || cb != 0 {
        b.ue(0);
        b.ue(cr as u32);
        b.ue(0);
        b.ue(cb as u32);
    }
    b.flag(p.vui);
    if p.vui {
        b.put(0, 1);
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
        b.put(1, 1);
        b.put(1, 1);
        b.ue(2);
        b.ue(1);
        b.ue(16);
        b.ue(16);
        b.ue(p.num_reorder as u32);
        b.ue(p.num_ref_frames as u32);
    }
    b.trailing();
    emit(&b, 0x67, out);
}
pub fn write_pps(p: &H264Params, out: &mut Vec<u8>) {
    let mut b = Bw::<128>::new();
    b.ue(0);
    b.ue(0);
    b.flag(p.entropy_cabac);
    b.put(0, 1);
    b.ue(0);
    b.ue(p.num_ref_idx_l0_minus1 as u32);
    b.ue(0);
    b.put(0, 1);
    b.put(0, 2);
    b.se(p.pic_init_qp - 26);
    b.se(0);
    b.se(0);
    b.put(1, 1);
    b.put(0, 1);
    b.put(0, 1);
    if p.transform_8x8 {
        b.put(1, 1);
        b.put(0, 1);
        b.se(0);
    }
    b.trailing();
    emit(&b, 0x68, out);
}
/// H.264 D.1.29/D.1.31, H.265 D.2.28/D.2.35; HEVC prefix SEI or H.264 SEI.
pub fn write_hdr_sei(
    hevc: bool,
    mdcv: Option<&[u8; 24]>,
    cll: Option<&[u8; 4]>,
    out: &mut Vec<u8>,
) {
    let mut rbsp = [0; 64];
    let mut n = 0;
    if let Some(v) = mdcv {
        rbsp[..2].copy_from_slice(&[137, 24]);
        rbsp[2..26].copy_from_slice(v);
        n = 26;
    }
    if let Some(v) = cll {
        rbsp[n..n + 2].copy_from_slice(&[144, 4]);
        rbsp[n + 2..n + 6].copy_from_slice(v);
        n += 6;
    }
    if n == 0 {
        return;
    }
    rbsp[n] = 0x80;
    n += 1;
    out.extend_from_slice(&[0, 0, 0, 1]);
    if hevc {
        out.extend_from_slice(&[39 << 1, 1]);
    } else {
        out.push(6);
    }
    escape(&rbsp[..n], out);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_parameters_preserve_extent_qp_and_coding_tools() {
        use crate::video::{bits, h264::ps};
        for chroma444 in [false, true] {
            let p = H264Params { width: if chroma444 { 721 } else { 16 },
                height: if chroma444 { 479 } else { 16 }, profile_idc: if chroma444 { 244 } else { 66 },
                level_idc: 41, log2_max_frame_num_minus4: 4, log2_max_poc_lsb_minus4: 5,
                poc_type: if chroma444 { 2 } else { 0 }, num_ref_frames: 3,
                num_ref_idx_l0_minus1: 1, entropy_cabac: chroma444,
                pic_init_qp: if chroma444 { 0 } else { 29 }, transform_8x8: chroma444,
                chroma444, lossless: chroma444, ..Default::default() };
            let mut bytes = Vec::new(); write_sps(&p, &mut bytes); write_pps(&p, &mut bytes);
            let units = bits::split_annexb(&bytes);
            let sps = ps::parse_sps_for_encode(&bits::unescape(units[0].nal)).unwrap();
            let pps = ps::parse_pps_for_encode(&bits::unescape(units[1].nal), |id| (id == sps.id).then(|| sps.clone())).unwrap();
            assert_eq!(sps.width() - sps.crop.0 - sps.crop.1, p.width as u32);
            assert_eq!(sps.height() - sps.crop.2 - sps.crop.3, p.height as u32);
            assert_eq!(sps.chroma_format_idc, if chroma444 { 3 } else { 1 });
            assert_eq!(sps.qpprime_y_zero_transform_bypass, p.lossless);
            assert_eq!(pps.pic_init_qp_minus26 + 26, p.pic_init_qp);
            assert_eq!(pps.entropy_coding_mode, p.entropy_cabac);
            assert_eq!(pps.transform_8x8_mode, p.transform_8x8);
        }
    }
}

pub(crate) fn write_typed_sps(s: &crate::video::h264::ps::Sps, id: u32, out: &mut Vec<u8>) {
    let mut b = Bw::<128>::new();
    b.put(s.profile_idc as u32, 8); b.put(s.constraint_flags as u32, 8); b.put(s.level_idc as u32, 8); b.ue(id);
    if s.profile_idc >= 100 {
        b.ue(s.chroma_format_idc);
        if s.chroma_format_idc == 3 { b.flag(s.separate_colour_plane); }
        b.ue(s.bit_depth_luma_minus8); b.ue(s.bit_depth_chroma_minus8);
        b.flag(s.qpprime_y_zero_transform_bypass); b.flag(false);
    }
    b.ue(s.log2_max_frame_num_minus4); b.ue(s.pic_order_cnt_type);
    if s.pic_order_cnt_type == 0 { b.ue(s.log2_max_poc_lsb_minus4); }
    b.ue(s.max_num_ref_frames); b.flag(s.gaps_in_frame_num_allowed);
    b.ue(s.pic_width_in_mbs - 1); b.ue(s.pic_height_in_map_units - 1);
    b.flag(s.frame_mbs_only); b.flag(s.direct_8x8_inference);
    let crop = s.crop != (0, 0, 0, 0); b.flag(crop);
    if crop {
        let unit = if s.chroma_format_idc == 3 { 1 } else { 2 };
        for value in [s.crop.0, s.crop.1, s.crop.2, s.crop.3] { b.ue(value / unit); }
    }
    let v = &s.vui;
    b.flag(true); b.flag(v.aspect_ratio_idc.is_some());
    if let Some(idc) = v.aspect_ratio_idc {
        b.put(u32::from(idc), 8);
        if idc == 255 { for size in v.sar { b.put(u32::from(size), 16); } }
    }
    b.flag(false);
    b.flag(true); b.put(5, 3); b.flag(v.full_range); b.flag(true);
    b.put(v.colour_primaries, 8); b.put(v.transfer_characteristics, 8); b.put(v.matrix_coefficients, 8);
    b.flag(false); b.flag(v.timing.is_some());
    if let Some((tick, scale, fixed)) = v.timing { b.put(tick, 32); b.put(scale, 32); b.flag(fixed); }
    for hrd in [&v.nal_hrd, &v.vcl_hrd] {
        b.flag(hrd.is_some());
        if let Some(h) = hrd {
            b.ue(h.cpb.len() as u32 - 1);
            let rate_scale = (0..16).find(|&scale| h.cpb.iter().all(|p| p.0 % (1 << (6 + scale)) == 0)).unwrap();
            let size_scale = (0..16).find(|&scale| h.cpb.iter().all(|p| p.1 % (1 << (4 + scale)) == 0)).unwrap();
            b.put(rate_scale, 4); b.put(size_scale, 4);
            for &(rate, size, cbr) in &h.cpb {
                b.ue((rate >> (6 + rate_scale)) as u32 - 1); b.ue((size >> (4 + size_scale)) as u32 - 1); b.flag(cbr);
            }
            b.put(h.initial_cpb_removal_delay_length - 1, 5); b.put(h.cpb_removal_delay_length - 1, 5);
            b.put(h.dpb_output_delay_length - 1, 5); b.put(h.time_offset_length, 5);
        }
    }
    if v.nal_hrd.is_some() || v.vcl_hrd.is_some() { b.flag(v.low_delay_hrd); }
    b.flag(v.pic_struct_present);
    let restriction = v.max_num_reorder_frames.is_some() || v.max_dec_frame_buffering.is_some(); b.flag(restriction);
    if restriction {
        b.flag(true); b.ue(2); b.ue(1); b.ue(16); b.ue(16);
        b.ue(v.max_num_reorder_frames.unwrap_or(0)); b.ue(v.max_dec_frame_buffering.unwrap_or(s.max_num_ref_frames));
    }
    b.trailing(); emit(&b, 0x67, out);
}

pub(crate) fn write_typed_pps(p: &crate::video::h264::ps::Pps, out: &mut Vec<u8>) {
    let mut b = Bw::<128>::new();
    b.ue(p.id); b.ue(p.sps_id); b.flag(p.entropy_coding_mode); b.flag(p.bottom_field_pic_order_in_frame_present); b.ue(0);
    b.ue(p.num_ref_idx_l0_default_active_minus1); b.ue(p.num_ref_idx_l1_default_active_minus1);
    b.flag(p.weighted_pred); b.put(p.weighted_bipred_idc, 2);
    b.se(p.pic_init_qp_minus26); b.se(0); b.se(p.chroma_qp_index_offset);
    b.flag(p.deblocking_filter_control_present); b.flag(p.constrained_intra_pred); b.flag(p.redundant_pic_cnt_present);
    if p.transform_8x8_mode || p.second_chroma_qp_index_offset != p.chroma_qp_index_offset {
        b.flag(p.transform_8x8_mode); b.flag(false); b.se(p.second_chroma_qp_index_offset);
    }
    b.trailing(); emit(&b, 0x68, out);
}
