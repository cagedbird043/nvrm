//! Client parameter sets shared by the NVENC setup and software frame writer.
//! This module does not assemble client access units: the caller owns that ordering.
use super::{av1hdr, EncConfig, ENC_AV1, ENC_H264, ENC_HEVC};
use crate::video::{av1::{header::SequenceHeader, obu}, bits::{self, BitReader}, h264, hevc,
    bitstream, unsupported, VResult};

#[derive(Clone, Debug)]
pub enum PackedSyntax {
    H264 { sps: h264::ps::Sps, pps: h264::ps::Pps },
    Hevc { sps: hevc::ps::Sps, pps: hevc::ps::Pps },
    Av1 { sequence: SequenceHeader },
}

/// VA sequence/picture semantics used only for the parameter-set class the driver owns.
#[derive(Clone, Debug)]
pub enum PackedDefaults {
    H264 { sps: h264::ps::Sps, pps: h264::ps::Pps },
    Hevc { sps: hevc::ps::Sps, pps: hevc::ps::Pps },
}
impl PackedDefaults {
    pub fn for_config(cfg: &EncConfig, pic_init_qp: i32) -> VResult<Self> {
        if cfg.width <= 0 || cfg.height <= 0 || cfg.width > 65504 || cfg.height > 65504 || cfg.num_refs <= 0 {
            return bitstream("invalid default parameter dimensions/references");
        }
        let mut bytes = Vec::new();
        match cfg.codec {
            ENC_H264 => {
                if !(0..=51).contains(&pic_init_qp) { return bitstream("H264 PPS initial QP"); }
                let mut p = super::h264hdr::config_params(cfg); p.pic_init_qp = pic_init_qp;
                super::h264hdr::write_sps(&p, &mut bytes); super::h264hdr::write_pps(&p, &mut bytes);
            }
            ENC_HEVC => {
                if !(-6..=51).contains(&pic_init_qp) { return bitstream("HEVC PPS initial QP"); }
                let mut p = super::hevchdr::config_params(cfg); p.pic_init_qp = pic_init_qp;
                super::hevchdr::write_vps(&p, &mut bytes); super::hevchdr::write_sps(&p, &mut bytes);
                super::hevchdr::write_pps(&p, &mut bytes);
            }
            _ => return unsupported("codec has no sequence/picture parameter split"),
        }
        let mut bank = ParameterSets::new(cfg.codec); bank.update(&bytes)?;
        match bank.syntax(cfg)? {
            PackedSyntax::H264 { sps, pps } => Ok(Self::H264 { sps, pps }),
            PackedSyntax::Hevc { sps, pps } => Ok(Self::Hevc { sps, pps }),
            _ => unreachable!(),
        }
    }
}

/// Owned parameter-set bank. Updates are transactional and may arrive across render calls.
#[derive(Clone, Debug)]
pub struct ParameterSets {
    codec: i32,
    // Type, ID, RBSP, client-owned. Generated defaults remain replaceable.
    sets: Vec<(u8, u32, Vec<u8>, bool)>,
}
impl ParameterSets {
    pub fn new(codec: i32) -> Self { Self { codec, sets: Vec::new() } }
    /// Exact unescaped parameter-NAL extent, including its header and trailing bits.
    /// `nal` starts at the NAL header and may contain following Annex-B units. No
    /// start-code search is performed inside mandatory parameter syntax.
    pub fn unescaped_parameter_len(&self, nal: &[u8]) -> VResult<usize> {
        if nal.first().is_none_or(|b| b & 0x80 != 0) { return bitstream("invalid raw parameter NAL header"); }
        match self.codec {
            ENC_H264 => match nal[0] & 31 {
                7 => h264::ps::unescaped_sps_len(nal),
                8 => h264::ps::unescaped_pps_len(nal, |id| self.sets.iter().find(|s| s.0 == 7 && s.1 == id)
                    .and_then(|s| h264::ps::parse_sps_for_encode(&s.2).ok())),
                _ => unsupported("raw NAL is not an H264 parameter set"),
            },
            ENC_HEVC => {
                if nal.len() < 2 || nal[0] & 1 != 0 || nal[1] != 1 { return unsupported("raw HEVC parameter layer/temporal header"); }
                match (nal[0] >> 1) & 63 {
                    32 => { let mut bytes = 0; validate_vps(nal, Some(&mut bytes))?; Ok(bytes) },
                    33 => hevc::ps::unescaped_sps_len(nal),
                    34 => hevc::ps::unescaped_pps_len(nal),
                    _ => unsupported("raw NAL is not an HEVC parameter set"),
                }
            }
            _ => unsupported("codec has no raw NAL parameter sets"),
        }
    }
    /// Supply client-missing, unrequested parameter sets without replacing client
    /// sets that arrived in another packed class (for example PPS in SEQUENCE).
    /// Call on a picture-staged bank; commit that bank only after the picture succeeds.
    pub fn complete_unrequested(&mut self, cfg: &EncConfig, requested_sequence: bool,
        requested_picture: bool, pps_id: u32, pic_init_qp: i32, defaults: Option<&PackedDefaults>) -> VResult<Vec<u8>> {
        if cfg.codec != self.codec { return bitstream("generated parameter codec"); }
        if self.codec == ENC_AV1 {
            if requested_sequence { self.latest(1)?; }
            return Ok(Vec::new());
        }
        let pps_type = if self.codec == ENC_H264 { 8 } else { 34 };
        let client_pps = self.sets.iter().any(|s| s.0 == pps_type && s.1 == pps_id && s.3);
        let use_picture = requested_picture || client_pps;
        let find = |typ, id| self.sets.iter().find(|s| s.0 == typ && s.1 == id && s.3).map(|s| s.2.as_slice());
        let latest = |typ| self.sets.iter().rev().find(|s| s.0 == typ && s.3).map(|s| s.2.as_slice());
        let missing = || crate::video::VideoError::Bitstream("missing requested parameter set".into());
        // A complete client bank needs no VA defaults, allocation or new headers.
        if client_pps {
            if let Ok(syntax) = self.syntax_for_picture(cfg, pps_id) {
                let complete = match &syntax {
                    PackedSyntax::H264 { sps, .. } => find(7, sps.id).is_some(),
                    PackedSyntax::Hevc { sps, .. } => find(33, sps.id)
                        .is_some_and(|raw| find(32, u32::from(raw[2] >> 4)).is_some()),
                    _ => false,
                };
                if complete { return Ok(Vec::new()); }
            }
        }
        let defaults = defaults.ok_or_else(|| crate::video::VideoError::Bitstream("missing VA parameter defaults".into()))?;
        let (syntax, generate_sps, vps) = match defaults {
            PackedDefaults::H264 { sps: ds, pps: dp } if self.codec == ENC_H264 => {
                let raw_pps = if use_picture { Some(find(8, pps_id).ok_or_else(missing)?) } else { None };
                let sps_id = if let Some(raw) = raw_pps {
                    let mut r = BitReader::new(raw); r.skip(8)?; r.ue()?; r.ue_max(31, "SPS id")?
                } else { dp.sps_id };
                let raw_sps = find(7, sps_id).or_else(|| if use_picture { None } else { latest(7) });
                if requested_sequence && raw_sps.is_none() { return Err(missing()); }
                let mut sps = if let Some(raw) = raw_sps { h264::ps::parse_sps_for_encode(raw)? } else { ds.clone() };
                if use_picture { sps.id = sps_id; }
                let pps = if let Some(raw) = raw_pps {
                    h264::ps::parse_pps_for_encode(raw, |id| (id == sps.id).then(|| sps.clone()))?
                } else { let mut p = dp.clone(); p.sps_id = sps.id; p };
                if pps.id != pps_id || (!use_picture && pps.pic_init_qp_minus26 != pic_init_qp - 26) {
                    return bitstream("H264 VA picture defaults mismatch");
                }
                (PackedSyntax::H264 { sps, pps }, raw_sps.is_none(), None)
            }
            PackedDefaults::Hevc { sps: ds, pps: dp } if self.codec == ENC_HEVC => {
                let mut pps = if use_picture { hevc::ps::parse_pps_for_encode(find(34, pps_id).ok_or_else(missing)?)? } else { dp.clone() };
                let raw_sps = find(33, pps.sps_id).or_else(|| if use_picture { None } else { latest(33) });
                if requested_sequence && raw_sps.is_none() { return Err(missing()); }
                let mut sps = if let Some(raw) = raw_sps { hevc::ps::parse_sps_for_encode(raw)? } else { ds.clone() };
                if use_picture { sps.id = pps.sps_id; } else { pps.sps_id = sps.id; }
                let vps_id = raw_sps.map(|raw| u32::from(raw[2] >> 4))
                    .or_else(|| latest(32).map(|raw| u32::from(raw[2] >> 4))).unwrap_or(0);
                let raw_vps = find(32, vps_id);
                if requested_sequence && raw_vps.is_none() { return Err(missing()); }
                if let Some(raw) = raw_vps {
                    let parsed = validate_vps(raw, None)?;
                    sps.encoder.vps_timing = parsed.timing;
                    sps.encoder.vps_hrd = parsed.hrd;
                }
                if pps.id != pps_id || (!use_picture && pps.init_qp_minus26 != pic_init_qp - 26) {
                    return bitstream("HEVC VA picture defaults mismatch");
                }
                (PackedSyntax::Hevc { sps, pps }, raw_sps.is_none(), Some((vps_id, raw_vps.is_none())))
            }
            _ => return bitstream("VA parameter defaults codec mismatch"),
        };
        syntax.validate(cfg)?;
        let mut generated = Vec::new();
        match &syntax {
            PackedSyntax::H264 { sps, pps } => {
                if generate_sps { super::h264hdr::write_typed_sps(sps, sps.id, &mut generated); }
                if !use_picture { super::h264hdr::write_typed_pps(pps, &mut generated); }
            }
            PackedSyntax::Hevc { sps, pps } => {
                let (vps_id, generate_vps) = vps.unwrap();
                if generate_vps { super::hevchdr::write_typed_vps(sps, vps_id, &mut generated); }
                if generate_sps {
                    if sps.st_rps.len() > 1 || sps.st_rps.iter().any(|r| r.s0.len() > 4 || r.s1.len() > 4) {
                        return unsupported("generated HEVC RPS outside VA reference model");
                    }
                    for rps in &sps.st_rps {
                        if rps.s0.iter().any(|&(v, _)| !(-32767..=-1).contains(&v))
                            || rps.s1.iter().any(|&(v, _)| !(1..=32767).contains(&v))
                            || rps.s0.windows(2).any(|w| w[0].0 <= w[1].0)
                            || rps.s1.windows(2).any(|w| w[0].0 >= w[1].0) {
                            return bitstream("generated HEVC RPS order/range");
                        }
                    }
                    super::hevchdr::write_typed_sps(sps, sps.id, vps_id, &mut generated);
                }
                if !use_picture { super::hevchdr::write_typed_pps(pps, &mut generated); }
            }
            _ => unreachable!(),
        }
        if !generated.is_empty() { self.update_inner(&generated, false)?; }
        self.syntax_for_picture(cfg, pps_id)?;
        Ok(generated)
    }
    pub fn update(&mut self, data: &[u8]) -> VResult<()> {
        self.update_inner(data, true)
    }
    fn update_inner(&mut self, data: &[u8], client: bool) -> VResult<()> {
        if data.is_empty() { return bitstream("empty packed parameter buffer"); }
        let mut updates = Vec::new();
        if self.codec == ENC_AV1 {
            let mut pos = 0;
            while pos < data.len() {
                let header = data[pos]; pos += 1;
                if header & 0x83 != 2 || (header >> 3) & 15 != 1 {
                    return unsupported("packed AV1 parameters require sized sequence OBUs");
                }
                if header & 4 != 0 {
                    if data.get(pos) != Some(&0) { return unsupported("packed AV1 sequence extension"); }
                    pos += 1;
                }
                let length = obu::leb128(data, &mut pos).map_err(|e| crate::video::VideoError::Bitstream(e.to_string()))?;
                let end = usize::try_from(length).ok().and_then(|n| pos.checked_add(n))
                    .filter(|&end| end <= data.len()).ok_or_else(|| crate::video::VideoError::Bitstream("truncated sequence OBU".into()))?;
                let payload = &data[pos..end];
                SequenceHeader::parse_for_encode(payload).map_err(|e| crate::video::VideoError::Bitstream(e.to_string()))?;
                updates.push((1, 0, payload.to_vec()));
                pos = end;
            }
        } else {
            let units = bits::split_annexb(data);
            if units.is_empty() || data[..units[0].start].iter().any(|&b| b != 0) {
                return bitstream("packed parameters require Annex-B framing");
            }
            for (i, n) in units.iter().enumerate() {
                let end = n.nal.as_ptr() as usize - data.as_ptr() as usize + n.nal.len();
                let next_start = units.get(i + 1).map_or(data.len(), |n| n.start);
                if data[end..next_start].iter().any(|&b| b != 0) { return bitstream("empty packed NAL unit"); }
                if n.nal[0] & 0x80 != 0 { return bitstream("forbidden NAL bit"); }
                let rbsp = checked_unescape(n.nal)?;
                let (typ, id) = match self.codec {
                    ENC_H264 => match n.nal[0] & 31 {
                        7 => (7, h264::ps::parse_sps_for_encode(&rbsp)?.id),
                        8 => { let mut r = BitReader::new(&rbsp); r.skip(8)?; (8, r.ue_max(255, "PPS id")?) },
                        _ => return unsupported("non-parameter H264 NAL in parameter bank"),
                    },
                    ENC_HEVC => {
                        if rbsp.len() < 3 || rbsp[1] & 7 != 1 || ((rbsp[0] & 1) | (rbsp[1] >> 3)) != 0 {
                            return unsupported("packed HEVC multilayer/temporal NAL");
                        }
                        match (rbsp[0] >> 1) & 63 {
                            32 => { validate_vps(&rbsp, None)?; (32, u32::from(rbsp[2] >> 4)) },
                            33 => (33, hevc::ps::parse_sps_for_encode(&rbsp)?.id),
                            34 => (34, hevc::ps::parse_pps_for_encode(&rbsp)?.id),
                            _ => return unsupported("non-parameter HEVC NAL in parameter bank"),
                        }
                    }
                    _ => return unsupported("packed codec"),
                };
                updates.push((typ, id, rbsp));
            }
        }
        for (typ, id, bytes) in updates { self.store(typ, id, bytes, client); }
        Ok(())
    }
    fn store(&mut self, typ: u8, id: u32, bytes: Vec<u8>, client: bool) {
        if let Some(i) = self.sets.iter().position(|s| s.0 == typ && s.1 == id) { self.sets.remove(i); }
        self.sets.push((typ, id, bytes, client));
    }
    fn latest(&self, typ: u8) -> VResult<&[u8]> {
        self.sets.iter().rev().find(|s| s.0 == typ).map(|s| s.2.as_slice())
            .ok_or_else(|| crate::video::VideoError::Bitstream(format!("missing packed parameter {typ}")))
    }
    pub fn syntax(&self, cfg: &EncConfig) -> VResult<PackedSyntax> {
        self.syntax_for_id(cfg, None)
    }
    /// Select the picture's PPS explicitly when the client supplies several parameter sets.
    pub fn syntax_for_picture(&self, cfg: &EncConfig, pps_id: u32) -> VResult<PackedSyntax> {
        self.syntax_for_id(cfg, Some(pps_id))
    }
    fn syntax_for_id(&self, cfg: &EncConfig, pps_id: Option<u32>) -> VResult<PackedSyntax> {
        if cfg.codec != self.codec { return bitstream("packed parameter codec differs from encoder"); }
        let picture_set = |typ| match pps_id {
            None => self.latest(typ),
            Some(id) => self.sets.iter().find(|s| s.0 == typ && s.1 == id).map(|s| s.2.as_slice())
                .ok_or_else(|| crate::video::VideoError::Bitstream(format!("missing packed PPS {id}"))),
        };
        let syntax = match self.codec {
            ENC_H264 => {
                let pps = h264::ps::parse_pps_for_encode(picture_set(8)?, |id| self.sets.iter()
                    .find(|s| s.0 == 7 && s.1 == id).and_then(|s| h264::ps::parse_sps_for_encode(&s.2).ok()))?;
                let sps = h264::ps::parse_sps_for_encode(&self.sets.iter().find(|s| s.0 == 7 && s.1 == pps.sps_id)
                    .ok_or_else(|| crate::video::VideoError::Bitstream("missing H264 SPS".into()))?.2)?;
                PackedSyntax::H264 { sps, pps }
            }
            ENC_HEVC => {
                let pps = hevc::ps::parse_pps_for_encode(picture_set(34)?)?;
                let raw_sps = &self.sets.iter().find(|s| s.0 == 33 && s.1 == pps.sps_id)
                    .ok_or_else(|| crate::video::VideoError::Bitstream("missing HEVC SPS".into()))?.2;
                let mut sps = hevc::ps::parse_sps_for_encode(raw_sps)?;
                let raw_vps = &self.sets.iter().find(|s| s.0 == 32 && s.1 == u32::from(raw_sps[2] >> 4))
                    .ok_or_else(|| crate::video::VideoError::Bitstream("HEVC SPS references missing VPS".into()))?.2;
                let vps = validate_vps(raw_vps, None)?;
                sps.encoder.vps_timing = vps.timing;
                sps.encoder.vps_hrd = vps.hrd;
                PackedSyntax::Hevc { sps, pps }
            }
            ENC_AV1 => PackedSyntax::Av1 { sequence: SequenceHeader::parse_for_encode(self.latest(1)?)
                .map_err(|e| crate::video::VideoError::Bitstream(e.to_string()))? },
            _ => return unsupported("packed codec"),
        };
        syntax.validate(cfg)?;
        Ok(syntax)
    }
}

/// Strict EBSP validation, including dangling and unnecessary prevention bytes.
fn checked_unescape(data: &[u8]) -> VResult<Vec<u8>> {
    let mut zeros = 0;
    for (i, &b) in data.iter().enumerate() {
        if zeros >= 2 {
            if b == 3 {
                if data.get(i + 1).is_none_or(|&v| v > 3) { return bitstream("invalid emulation prevention byte"); }
                zeros = 0;
                continue;
            }
            if b <= 2 { return bitstream("unescaped NAL byte sequence"); }
        }
        zeros = if b == 0 { zeros + 1 } else { 0 };
    }
    Ok(bits::unescape(data))
}

impl PackedSyntax {
    pub(crate) fn validate(&self, c: &EncConfig) -> VResult<()> {
        if c.width <= 0 || c.height <= 0 || c.width > 65504 || c.height > 65504 {
            return bitstream("packed session dimensions outside hardware fields");
        }
        match self {
            Self::H264 { sps: s, pps: p } => {
                if c.codec != ENC_H264 || s.width().checked_sub(s.crop.0).and_then(|v| v.checked_sub(s.crop.1)) != Some(c.width as u32)
                    || s.height().checked_sub(s.crop.2).and_then(|v| v.checked_sub(s.crop.3)) != Some(c.height as u32)
                    || s.chroma_format_idc != (if c.chroma444 { 3 } else { 1 })
                    || s.bit_depth_luma_minus8 != 0 || s.bit_depth_chroma_minus8 != 0 {
                    return unsupported("packed H264 dimensions/chroma/depth differ from session");
                }
                if !s.frame_mbs_only || s.separate_colour_plane || s.pic_order_cnt_type == 1
                    || s.gaps_in_frame_num_allowed || s.seq_scaling_matrix_present
                    || s.crop.0 != 0 || s.crop.2 != 0
                    || s.width() != (c.width as u32).next_multiple_of(16)
                    || s.height() != (c.height as u32).next_multiple_of(16)
                    || !s.direct_8x8_inference || s.qpprime_y_zero_transform_bypass != c.lossless
                    || s.log2_max_frame_num_minus4 > 12 || s.log2_max_poc_lsb_minus4 > 12
                    || s.id > 31 || p.id > 255 || !(-26..=25).contains(&p.pic_init_qp_minus26)
                    || !(-12..=12).contains(&p.chroma_qp_index_offset) || !(-12..=12).contains(&p.second_chroma_qp_index_offset)
                    || s.vui.colour_primaries > 255 || s.vui.transfer_characteristics > 255 || s.vui.matrix_coefficients > 255
                    || p.num_ref_idx_l0_default_active_minus1 > 3 || p.num_ref_idx_l1_default_active_minus1 > 3
                    || p.scaling != h264::ps::ScalingMatrices::flat() || p.bottom_field_pic_order_in_frame_present
                    || p.redundant_pic_cnt_present || p.weighted_pred || p.weighted_bipred_idc != 0
                    || !matches!(s.profile_idc, 66 | 77 | 100 | 244)
                    || s.constraint_flags & 3 != 0
                    || (s.profile_idc == 66 && (p.entropy_coding_mode || p.transform_8x8_mode))
                    || (s.profile_idc == 77 && p.transform_8x8_mode)
                    || (c.idr_period != 1 && s.max_num_ref_frames < c.num_refs as u32) || p.sps_id != s.id
                    || (c.bframes > 0 && s.pic_order_cnt_type != 0) {
                    return unsupported("packed H264 coding tools/reference model");
                }
                for h in [&s.vui.nal_hrd, &s.vui.vcl_hrd].into_iter().flatten() {
                    if h.cpb.len() != 1 || c.rc_mode == 0 { return unsupported("packed H264 HRD requires single-CPB rate control"); }
                    let (rate, size, cbr) = h.cpb[0];
                    // Persistent fullness is signed 28.4; the whole CPB must fit.
                    if rate == 0 || size == 0 || rate > i32::MAX as u64 || size > i32::MAX as u64 / 16
                        || rate % 64 != 0 || size % 16 != 0 || (cbr && c.rc_mode & 3 != 2)
                        || !(1..=32).contains(&h.initial_cpb_removal_delay_length)
                        || !(1..=32).contains(&h.cpb_removal_delay_length) || !(1..=32).contains(&h.dpb_output_delay_length)
                        || h.time_offset_length > 31 {
                        return unsupported("packed H264 HRD range");
                    }
                }
                if let Some((tick, scale, _)) = s.vui.timing {
                    if tick == 0 || scale == 0 { return bitstream("zero H264 timing clock"); }
                }
            }
            Self::Hevc { sps: s, pps: p } => {
                let crop_unit = if s.chroma_format_idc == 3 { 1 } else { 2 };
                if c.codec != ENC_HEVC || s.conf_win[0] != 0 || s.conf_win[2] != 0
                    || s.width.checked_sub(crop_unit * s.conf_win[1]) != Some(c.width as u32)
                    || s.height.checked_sub(crop_unit * s.conf_win[3]) != Some(c.height as u32)
                    || s.width != (c.width as u32).next_multiple_of(32)
                    || s.height != (c.height as u32).next_multiple_of(32)
                    || s.bit_depth_luma != c.bit_depth as u32 || s.bit_depth_chroma != c.bit_depth as u32
                    || s.chroma_format_idc != (if c.chroma444 { 3 } else { 1 }) || s.separate_colour_plane {
                    return unsupported("packed HEVC dimensions/chroma/depth differ from session");
                }
                if s.temporal_mvp { return unsupported("packed HEVC temporal motion-vector prediction"); }
                if s.log2_min_cb != super::hevchdr::min_cb_log2(c) {
                    return unsupported("packed HEVC minimum CU differs from the selected ME/MD/Q mode");
                }
                if s.log2_ctb != 5 || s.log2_min_tb != 2
                    || !(2..=5).contains(&s.log2_max_tb) || s.scaling_list_enabled || s.pcm || s.long_term_ref_pics_present
                    || !(4..=16).contains(&s.log2_max_poc_lsb)
                    || s.id > 15 || p.id > 63 || !matches!(s.profile_idc, 1..=4)
                    || s.rext != hevc::ps::SpsRext::default()
                    || p.rext.cross_component_prediction || p.rext.chroma_qp_offset_list_enabled
                    || p.rext.log2_max_transform_skip_size != 2
                    || p.rext.log2_sao_offset_scale_luma != 0 || p.rext.log2_sao_offset_scale_chroma != 0
                    || !(-12..=12).contains(&p.cb_qp_offset) || !(-12..=12).contains(&p.cr_qp_offset)
                    || !(-6..=6).contains(&p.beta_offset_div2) || !(-6..=6).contains(&p.tc_offset_div2)
                    || (s.bit_depth_luma == 8 && p.init_qp_minus26 < -26)
                    || (c.idr_period != 1 && s.max_dec_pic_buffering < c.num_refs as u32)
                    || p.sign_data_hiding || p.transquant_bypass != c.lossless
                    || !(-32..=25).contains(&p.init_qp_minus26)
                    || p.num_ref_idx_l0_default == 0 || p.num_ref_idx_l0_default > 4
                    || p.num_ref_idx_l1_default == 0 || p.num_ref_idx_l1_default > 4
                    || p.log2_parallel_merge_level < 2
                    || p.log2_parallel_merge_level > 6 || s.max_dec_pic_buffering == 0
                    || s.max_dec_pic_buffering > 16 || s.max_num_reorder_pics > 15
                    || s.max_th_depth_inter > 3 || s.max_th_depth_intra > 3
                    || p.diff_cu_qp_delta_depth > s.log2_ctb.saturating_sub(s.log2_min_cb)
                    || (s.bit_depth_luma > 8 && p.transform_skip)
                    || p.dependent_slice_segments_enabled || p.num_extra_slice_header_bits != 0
                    || p.weighted_pred || p.weighted_bipred || p.tiles.is_some() || p.entropy_coding_sync
                    || p.scaling_list.is_some() || p.slice_header_extension_present || p.output_flag_present
                    || p.lists_modification_present || p.sps_id != s.id {
                    return unsupported("packed HEVC coding tools/reference model");
                }
                hevc_rate_contract(s, c)?;
            }
            Self::Av1 { sequence: s } => {
                if c.codec != ENC_AV1 || s.seq_profile != 0 || s.mono_chrome || !s.subsampling_x || !s.subsampling_y
                    || s.bit_depth as i32 != c.bit_depth || s.max_frame_width_minus_1 + 1 != c.width as u32
                    || s.max_frame_height_minus_1 + 1 != c.height as u32 {
                    return unsupported("packed AV1 dimensions/profile/chroma/depth differ from session");
                }
                if s.still_picture || s.reduced_still_picture_header || s.decoder_model_info_present
                    || s.frame_id_numbers_present || s.use_128x128_superblock || s.enable_filter_intra
                    || s.enable_intra_edge_filter || s.enable_interintra_compound || s.enable_masked_compound
                    || s.enable_warped_motion || s.enable_dual_filter || !s.enable_order_hint
                    || s.enable_jnt_comp || s.enable_ref_frame_mvs || s.seq_force_screen_content_tools != 0
                    || s.enable_superres || s.separate_uv_delta_q || s.film_grain_params_present
                    || !(1..=8).contains(&s.order_hint_bits) || s.operating_points.len() != 1
                    || s.operating_points[0].idc != 0 {
                    return unsupported("packed AV1 coding tools/reference model");
                }
            }
        }
        Ok(())
    }
    pub(crate) fn av1_params(&self) -> Option<av1hdr::Av1SeqParams> {
        let Self::Av1 { sequence: s } = self else { return None };
        Some(av1hdr::Av1SeqParams { width: (s.max_frame_width_minus_1 + 1) as i32,
            height: (s.max_frame_height_minus_1 + 1) as i32, bit_depth: s.bit_depth as i32,
            full_range: s.color_range, seq_level_idx: s.operating_points[0].seq_level_idx as i32,
            enable_cdef: s.enable_cdef, enable_restoration: s.enable_restoration,
            order_hint_bits: s.order_hint_bits as i32, colour: s.color_description.map(|v| v as i32) })
    }
}

/// Counters for one client picture. POC is the signed, unwrapped codec POC, not its LSB.
#[derive(Clone, Copy, Debug)]
pub enum PackedPicture {
    H264 { frame_num: u32, poc: i32 },
    Hevc { poc: i32 },
    Av1 { order_hint: u32 },
}
impl PackedPicture {
    pub(crate) fn validate(self, syntax: Option<&PackedSyntax>, c: &EncConfig) -> VResult<()> {
        match (self, syntax) {
            (Self::H264 { frame_num, poc }, Some(PackedSyntax::H264 { sps, .. }))
                if c.codec == ENC_H264 && frame_num < sps.max_frame_num()
                    && (sps.pic_order_cnt_type != 2 || (poc as u32 & (2 * sps.max_frame_num() - 1)) == 2 * frame_num) => Ok(()),
            (Self::Hevc { .. }, Some(PackedSyntax::Hevc { .. })) if c.codec == ENC_HEVC => Ok(()),
            (Self::Av1 { order_hint }, Some(PackedSyntax::Av1 { sequence }))
                if c.codec == ENC_AV1 && order_hint < 1 << sequence.order_hint_bits => Ok(()),
            _ => bitstream("packed picture does not match installed sequence/counter range"),
        }
    }
}

pub(crate) fn unwrap_hint(last: Option<u32>, hint: u32, bits: u32) -> u32 {
    let Some(last) = last else { return hint };
    let modulus = 1u32 << bits;
    let delta = hint.wrapping_sub(last) & (modulus - 1);
    // At the half-range ambiguity prefer forward progress (notably one-bit hints).
    let distance = if delta > modulus / 2 { delta as i32 - modulus as i32 } else { delta as i32 };
    last.wrapping_add_signed(distance)
}

use super::nvenc_drv::*;
pub(crate) fn apply_h264(ps: &mut nvenc_h264_drv_pic_setup_s, s: &h264::ps::Sps,
    p: &h264::ps::Pps, frame_num: u32, poc: u32) {
    let sp = &mut ps.sps_data;
    sp.set_profile_idc(s.profile_idc as u32);
    sp.set_level_idc(s.level_idc as u32);
    sp.set_log2_max_frame_num_minus4(s.log2_max_frame_num_minus4);
    sp.set_pic_order_cnt_type(s.pic_order_cnt_type);
    sp.set_log2_max_pic_order_cnt_lsb_minus4(s.log2_max_poc_lsb_minus4);
    let pp = &mut ps.pps_data;
    pp.set_pic_param_set_id(p.id);
    pp.set_entropy_coding_mode_flag(p.entropy_coding_mode as u32);
    pp.set_num_ref_idx_l0_active_minus1(p.num_ref_idx_l0_default_active_minus1);
    pp.set_num_ref_idx_l1_active_minus1(p.num_ref_idx_l1_default_active_minus1);
    pp.set_pic_init_qp_minus26(p.pic_init_qp_minus26);
    pp.set_chroma_qp_index_offset(p.chroma_qp_index_offset);
    pp.set_second_chroma_qp_index_offset(p.second_chroma_qp_index_offset);
    pp.set_deblocking_filter_control_present_flag(p.deblocking_filter_control_present as u32);
    pp.set_transform_8x8_mode_flag(p.transform_8x8_mode as u32);
    pp.set_constrained_intra_pred_flag(p.constrained_intra_pred as u32);
    ps.pic_control.frame_num = (frame_num & (s.max_frame_num() - 1)) as u16;
    ps.pic_control.pic_order_cnt_lsb = (poc & ((1 << (s.log2_max_poc_lsb_minus4 + 4)) - 1)) as u16;
    if let Some(h) = &s.vui.nal_hrd {
        let (rate, size, _) = h.cpb[0];
        ps.rate_control.nal_bitrate = rate as i32;
        ps.rate_control.nal_cpb_size = size as i32;
        ps.rate_control.buffersize = size as u32;
    }
    if let Some(h) = &s.vui.vcl_hrd {
        let (rate, size, _) = h.cpb[0];
        ps.rate_control.vcl_bitrate = rate as i32;
        ps.rate_control.vcl_cpb_size = size as i32;
    }
    if s.vui.nal_hrd.is_some() || s.vui.vcl_hrd.is_some() {
        let r = &mut ps.rate_control;
        r.hrd_type = r.hrd_type & 32 | u8::from(s.vui.nal_hrd.is_some()) * 2 | u8::from(s.vui.vcl_hrd.is_some());
        r.buffersize = s.vui.nal_hrd.iter().chain(&s.vui.vcl_hrd).map(|h| h.cpb[0].1 as u32).min().unwrap();
        r.Bmin = (r.buffersize / 81920) as i32;
    }
    if let Some((tick, scale, _)) = s.vui.timing {
        if ps.rate_control.hrd_type != 0 {
            ps.rate_control.framerate = ((u64::from(scale) * 128 / u64::from(tick)).min(i32::MAX as u64)) as i32;
        }
    }
}

pub(crate) fn apply_hevc(ps: &mut nvenc_h265_drv_pic_setup_s, s: &hevc::ps::Sps,
    p: &hevc::ps::Pps, poc: u32, cfg: &EncConfig) {
    let sp = &mut ps.sps_data;
    sp.set_chroma_format_idc(s.chroma_format_idc);
    sp.set_log2_max_pic_order_cnt_lsb_minus4(s.log2_max_poc_lsb - 4);
    sp.set_min_cu_size(s.log2_min_cb - 3);
    sp.set_max_cu_size(s.log2_ctb - 3);
    sp.set_min_tu_size(s.log2_min_tb - 2);
    sp.set_max_tu_size(s.log2_max_tb - 2);
    sp.set_max_transform_depth_inter(s.max_th_depth_inter);
    sp.set_max_transform_depth_intra(s.max_th_depth_intra);
    sp.set_amp_enabled_flag(s.amp as u32);
    sp.set_sample_adaptive_offset_enabled_flag(s.sao as u32);
    sp.set_strong_intra_smoothing_enabled_flag(s.strong_intra_smoothing as u32);
    sp.set_num_short_term_ref_pic_sets(s.st_rps.len() as u32);
    let pp = &mut ps.pps_data;
    pp.set_constrained_intra_pred_flag(p.constrained_intra_pred as u32);
    pp.set_pic_param_set_id(p.id);
    pp.set_num_ref_idx_l0_active_minus1(p.num_ref_idx_l0_default - 1);
    pp.set_num_ref_idx_l1_active_minus1(p.num_ref_idx_l1_default - 1);
    // NVENC's slice writer has a fixed QP baseline. Keep the internal baseline
    // deterministic and translate only the output header, never quantization.
    pp.set_pic_init_qp_minus26(0);
    pp.set_pps_cb_qp_offset(p.cb_qp_offset);
    pp.set_pps_cr_qp_offset(p.cr_qp_offset);
    pp.set_transform_skip_enabled_flag(p.transform_skip as u32);
    pp.set_transquant_bypass_enabled_flag(p.transquant_bypass as u32);
    pp.set_cu_qp_delta_enabled_flag(p.cu_qp_delta as u32);
    pp.set_diff_cu_qp_delta_depth(p.diff_cu_qp_delta_depth);
    pp.set_pps_loop_filter_across_slices_enabled_flag(p.loop_filter_across_slices as u32);
    pp.set_cabac_init_present_flag(p.cabac_init_present as u32);
    pp.set_sign_data_hiding_enabled_flag(p.sign_data_hiding as u32);
    pp.set_deblocking_filter_override_enabled_flag(p.deblocking_override_enabled as u32);
    pp.set_pps_slice_chroma_qp_offsets_present(p.slice_chroma_qp_offsets_present as u32);
    pp.set_log2_parallel_merge_level_minus2(p.log2_parallel_merge_level - 2);
    ps.pic_control.pic_order_cnt_lsb = (poc & ((1 << s.log2_max_poc_lsb) - 1)) as u16;
    // NVENC emits the current explicit RPS. Never select an unrelated client SPS RPS.
    ps.pic_control.set_short_term_ref_pic_set_sps_flag(0);
    ps.pic_control.set_sao_luma_mode(if s.sao { 31 } else { 0 });
    ps.pic_control.set_sao_chroma_mode(if s.sao { 31 } else { 0 });
    let contract = hevc_rate_contract(s, cfg).expect("HEVC syntax validated before setup");
    let r = &mut ps.rate_control;
    if let Some(fps) = contract.fps_q8 { if cfg.rc_mode != 0 { r.framerate = fps; } }
    if let Some((rate, size)) = contract.nal { r.nal_bitrate = rate; r.nal_cpb_size = size; }
    if let Some((rate, size)) = contract.vcl { r.vcl_bitrate = rate; r.vcl_cpb_size = size; }
    if let Some(size) = contract.buffer {
        r.buffersize = size;
        r.Bmin = (size / 81920) as i32;
        r.hrd_type = u8::from(contract.nal.is_some()) * 2 | u8::from(contract.vcl.is_some())
            | if cfg.rc_mode & 3 == 2 { 32 } else { 0 };
        let fps = f64::from(r.framerate) / 256.0;
        let pixels = f64::from(cfg.width) * f64::from(cfg.height);
        let cap = contract.max_rate.unwrap() as f64;
        r.R = (cap / fps / pixels * 256.0) as i32;
        r.Ravg = if cfg.rc_mode & 3 == 2 { 0 }
            else { (f64::from(cfg.bitrate).min(cap) / fps / pixels * 256.0) as i32 };
    }
}
pub(crate) fn apply_hevc_slice(sc: &mut nvenc_h265_slice_control_s, s: &hevc::ps::Sps, p: &hevc::ps::Pps) {
    sc.set_slice_sao_luma_flag(s.sao as u32);
    sc.set_slice_sao_chroma_flag(s.sao as u32);
    sc.set_slice_loop_filter_across_slices_enabled_flag(p.loop_filter_across_slices as u32);
    sc.set_deblocking_filter_override_flag(p.deblocking_override_enabled as u32);
    sc.set_slice_deblocking_filter_disabled_flag(p.deblocking_disabled as u32);
    sc.set_slice_beta_offset_div2(p.beta_offset_div2);
    sc.set_slice_tc_offset_div2(p.tc_offset_div2);
}

pub(crate) fn apply_av1(bytes: &mut [u8], s: &av1hdr::Av1SeqParams) {
    // Ada adds a fourth surface ahead of the sequence word. Use typed field setters,
    // but not the pre-Ada top-level struct's offsets.
    let mut seq = unsafe { std::ptr::read_unaligned(bytes.as_ptr().add(0x84).cast::<nvenc_av1_seq_hdr_data_s>()) };
    seq.set_enable_cdef(s.enable_cdef as u32);
    seq.set_enable_restoration(s.enable_restoration as u32);
    unsafe { std::ptr::write_unaligned(bytes.as_mut_ptr().add(0x84).cast::<nvenc_av1_seq_hdr_data_s>(), seq); }
    if !s.enable_cdef { bytes[0xa8..0xc0].fill(0); }
    if !s.enable_restoration { bytes[0xc4..0xc8].fill(0); }
}

fn validate_vps(rbsp: &[u8], consumed: Option<&mut usize>) -> VResult<hevc::ps::EncoderSequence> {
    let mut result = hevc::ps::EncoderSequence::default();
    let mut r = BitReader::new(&rbsp[2..]);
    r.skip(6)?;
    if r.u(6)? != 0 || r.u(3)? != 0 { return unsupported("HEVC encoder VPS multilayer/sublayers"); }
    r.skip(1 + 16 + 96)?;
    r.skip(1)?;
    r.ue()?; r.ue()?; r.ue()?;
    if r.u(6)? != 0 || r.ue()? != 0 { return unsupported("HEVC encoder VPS layer sets"); }
    if r.flag()? {
        result.timing = Some(hevc::ps::encoder_timing(&mut r)?);
        let count = r.ue_max(1, "vps_num_hrd_parameters")?;
        for _ in 0..count {
            if r.ue()? != 0 { return unsupported("HEVC VPS HRD layer set"); }
            result.hrd = Some(hevc::ps::encoder_hrd(&mut r, 0)?);
        }
    }
    if r.flag()? { return unsupported("HEVC encoder VPS extension"); }
    let bytes = r.finish_rbsp(consumed.is_some())?;
    if let Some(out) = consumed { *out = bytes + 2; }
    Ok(result)
}

#[cfg(test)]
mod tests;

pub(crate) fn unwrap_picture_hint(last: Option<u32>, hint: u32, bits: u32, forward: bool) -> u32 {
    if !forward { return unwrap_hint(last, hint, bits); }
    let Some(last) = last else { return hint };
    let modulus = 1u32 << bits;
    let delta = hint.wrapping_sub(last) & (modulus - 1);
    last.wrapping_add(if delta == 0 { modulus } else { delta })
}

#[derive(Default)]
struct HevcRateContract {
    fps_q8: Option<i32>,
    nal: Option<(i32, i32)>,
    vcl: Option<(i32, i32)>,
    buffer: Option<u32>,
    max_rate: Option<i32>,
}

fn hevc_rate_contract(s: &hevc::ps::Sps, cfg: &EncConfig) -> VResult<HevcRateContract> {
    let mut result = HevcRateContract::default();
    let mut clock: Option<(u32, u64)> = None;
    for (timing, hrd) in [(&s.encoder.timing, &s.encoder.hrd), (&s.encoder.vps_timing, &s.encoder.vps_hrd)] {
        let timing = timing.as_ref().or(s.encoder.timing.as_ref()).or(s.encoder.vps_timing.as_ref());
        if let Some(timing) = timing {
            if timing.num_units_in_tick == 0 || timing.time_scale == 0 || timing.poc_ticks.is_some_and(|v| v == 0 || v > u32::MAX as u64) {
                return bitstream("HEVC timing clock range");
            }
        }
        let duration = if let Some(hrd) = hrd {
            if cfg.rc_mode == 0 || hrd.nal.len() > 1 || hrd.vcl.len() > 1
                || hrd.nal.is_empty() && hrd.vcl.is_empty() || hrd.fixed_duration.is_none() {
                return unsupported("packed HEVC HRD requires fixed-rate, single-CPB hardware RC");
            }
            if hrd.low_delay || hrd.fixed_duration.is_some_and(|d| d == 0 || d > u32::MAX as u64)
                || !(1..=32).contains(&hrd.initial_cpb_removal_delay_length)
                || !(1..=32).contains(&hrd.au_cpb_removal_delay_length) || !(1..=32).contains(&hrd.dpb_output_delay_length) {
                return unsupported("HEVC HRD delay range/mode");
            }
            if timing.is_none() { return bitstream("HEVC HRD without timing clock"); }
            if timing.unwrap().poc_ticks.is_some_and(|ticks| Some(ticks) != hrd.fixed_duration) {
                return unsupported("HEVC HRD duration differs from POC clock");
            }
            for (cpb, limit) in [(&hrd.nal, &mut result.nal), (&hrd.vcl, &mut result.vcl)] {
                if let Some(&(rate, size, cbr)) = cpb.first() {
                    if rate == 0 || size == 0 || rate > i32::MAX as u64 || size > i32::MAX as u64 / 16 || rate % 64 != 0 || size % 16 != 0 {
                        return unsupported("packed HEVC CPB range");
                    }
                    if cbr && cfg.rc_mode & 3 != 2 { return unsupported("HEVC CBR HRD with non-CBR session"); }
                    let (rate, size) = (rate as i32, size as i32);
                    *limit = Some(match *limit { Some((r, s)) => (r.min(rate), s.min(size)), None => (rate, size) });
                    result.buffer = Some(result.buffer.map_or(size as u32, |s| s.min(size as u32)));
                    result.max_rate = Some(result.max_rate.map_or(rate, |r| r.min(rate)));
                }
            }
            hrd.fixed_duration
        } else { timing.and_then(|t| t.poc_ticks) };
        if let (Some(timing), Some(duration)) = (timing, duration) {
            let denominator = u64::from(timing.num_units_in_tick).checked_mul(duration)
                .filter(|&d| d != 0).ok_or_else(|| crate::video::VideoError::Bitstream("HEVC timing range".into()))?;
            if let Some((scale, ticks)) = clock {
                if u128::from(scale) * u128::from(denominator) != u128::from(timing.time_scale) * u128::from(ticks) {
                    return bitstream("conflicting HEVC VPS/SPS frame clocks");
                }
            }
            clock = Some((timing.time_scale, denominator));
        }
    }
    if let Some((scale, ticks)) = clock {
        let q8 = u64::from(scale) * 256 / ticks;
        if q8 == 0 || q8 > i32::MAX as u64 { return unsupported("HEVC hardware frame clock range"); }
        result.fps_q8 = Some(q8 as i32);
    }
    Ok(result)
}
