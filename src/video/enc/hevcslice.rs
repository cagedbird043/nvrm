//! Translate NVENC's HEVC slice QP baseline without changing CABAC-coded data.
use super::h264hdr::{escape, Bw};
use crate::video::{bits::BitReader, bitstream, hevc::ps::{Pps, Sps}, unsupported, VResult};

// The supported encoder has at most four explicit references, no weights, tiles,
// dependent segments or extensions. Its complete slice header fits in 128 bytes.
const HEADER_BYTES: usize = 128;

pub(super) struct Header {
    pub(super) qp_delta: i32,
    pub(super) qp_start: usize,
    pub(super) qp_end: usize,
    pub(super) alignment_start: usize,
    pub(super) data_offset: usize,
}

/// Parse the complete header of the supported NVENC slice model, not its CABAC
/// data. Positions include the two-byte NAL header and refer to unescaped bytes.
pub(super) fn header(rbsp: &[u8], s: &Sps, p: &Pps) -> VResult<Header> {
    if rbsp.len() < 3 || rbsp[0] & 0x81 != 0 || rbsp[1] != 1 || rbsp[0] >> 1 > 31 {
        return bitstream("invalid HEVC encoder VCL header");
    }
    if p.dependent_slice_segments_enabled || p.output_flag_present || p.num_extra_slice_header_bits != 0
        || p.lists_modification_present || p.weighted_pred || p.weighted_bipred || p.tiles.is_some()
        || p.entropy_coding_sync || p.slice_header_extension_present || p.rext.chroma_qp_offset_list_enabled
        || s.long_term_ref_pics_present || s.separate_colour_plane {
        return unsupported("HEVC encoder slice-header tools");
    }
    let typ = rbsp[0] >> 1;
    let idr = typ == 19 || typ == 20;
    let mut r = BitReader::new(rbsp);
    r.skip(16)?;
    let first = r.flag()?;
    if (16..=23).contains(&typ) { r.flag()?; }
    if r.ue_max(63, "slice_pic_parameter_set_id")? != p.id { return bitstream("HEVC VCL PPS differs from selected picture"); }
    if !first {
        let ctbs = s.width_ctbs() * s.height_ctbs();
        let bits = if ctbs <= 1 { 0 } else { 32 - (ctbs - 1).leading_zeros() };
        if r.u(bits)? >= ctbs { return bitstream("HEVC slice segment address"); }
    }
    let kind = r.ue_max(2, "slice_type")?;
    let mut temporal_mvp = false;
    if !idr {
        r.u(s.log2_max_poc_lsb)?;
        if r.flag()? {
            if s.st_rps.is_empty() { return bitstream("HEVC VCL selects missing SPS RPS"); }
            let n = s.st_rps.len() as u32;
            let bits = if n <= 1 { 0 } else { 32 - (n - 1).leading_zeros() };
            if r.u(bits)? >= n { return bitstream("HEVC VCL SPS RPS index"); }
        } else {
            // NVENC writes explicit, non-predicted RPS. Unlike the decoder's
            // retained StRps, locating this header requires no allocated vectors.
            if !s.st_rps.is_empty() && r.flag()? { return unsupported("predicted NVENC slice RPS"); }
            let negative = r.ue_max(4, "num_negative_pics")?;
            let positive = r.ue_max(4 - negative, "num_positive_pics")?;
            for _ in 0..negative + positive { r.ue_max(32767, "delta_poc_minus1")?; r.flag()?; }
        }
        if s.temporal_mvp { temporal_mvp = r.flag()?; }
    }
    let (mut sao_y, mut sao_c) = (false, false);
    if s.sao { sao_y = r.flag()?; if s.chroma_format_idc != 0 { sao_c = r.flag()?; } }
    if kind != 2 {
        let (mut l0, mut l1) = (p.num_ref_idx_l0_default, p.num_ref_idx_l1_default);
        if r.flag()? {
            l0 = r.ue_max(3, "num_ref_idx_l0_active_minus1")? + 1;
            if kind == 0 { l1 = r.ue_max(3, "num_ref_idx_l1_active_minus1")? + 1; }
        }
        if kind == 0 { r.flag()?; } // mvd_l1_zero_flag
        if p.cabac_init_present { r.flag()?; }
        if temporal_mvp {
            let from_l0 = kind != 0 || r.flag()?;
            if from_l0 && l0 > 1 || !from_l0 && l1 > 1 { r.ue_max(3, "collocated_ref_idx")?; }
        }
        r.ue_max(4, "five_minus_max_num_merge_cand")?;
    }
    let qp_start = r.pos();
    let qp_delta = r.se()?;
    if !(-87..=87).contains(&qp_delta) { return bitstream("HEVC slice QP delta range"); }
    let qp_end = r.pos();
    if p.slice_chroma_qp_offsets_present { r.se()?; r.se()?; }
    let override_filter = p.deblocking_override_enabled && r.flag()?;
    let mut disabled = p.deblocking_disabled;
    if override_filter { disabled = r.flag()?; if !disabled { r.se()?; r.se()?; } }
    if p.loop_filter_across_slices && (sao_y || sao_c || !disabled) { r.flag()?; }
    let alignment_start = r.pos();
    if !r.flag()? { return bitstream("HEVC byte_alignment one bit"); }
    while r.pos() & 7 != 0 { if r.flag()? { return bitstream("HEVC byte_alignment zero bit"); } }
    Ok(Header { qp_delta, qp_start, qp_end, alignment_start, data_offset: r.pos() / 8 })
}

fn start_code(data: &[u8], at: usize) -> Option<usize> {
    if data.get(at..at + 3) == Some(&[0, 0, 1]) { Some(3) }
    else if data.get(at..at + 4) == Some(&[0, 0, 0, 1]) { Some(4) } else { None }
}

/// Append all VCL units, translating the known hardware PPS QP baseline 26 to
/// the client's PPS. New byte_alignment ends nonzero, just like the original;
/// consequently no EPB context crosses into the unchanged CABAC EBSP suffix.
pub(super) fn rewrite(data: &[u8], s: &Sps, p: &Pps, out: &mut Vec<u8>) -> VResult<()> {
    let mut at = 0;
    while at < data.len() {
        let prefix = start_code(data, at).ok_or_else(|| crate::video::VideoError::Bitstream("NVENC HEVC Annex-B framing".into()))?;
        let begin = at + prefix;
        let mut end = begin;
        while end < data.len() && start_code(data, end).is_none() { end += 1; }
        let nal = &data[begin..end];
        let mut rbsp = [0; HEADER_BYTES];
        let mut offsets = [0; HEADER_BYTES + 1];
        let (mut n, mut i, mut zeros) = (0, 0, 0);
        while i < nal.len() && n < rbsp.len() {
            let byte = nal[i];
            if zeros >= 2 && byte == 3 {
                if nal.get(i + 1).is_none_or(|&b| b > 3) { return bitstream("invalid HEVC encoder EPB"); }
                zeros = 0; i += 1; continue;
            }
            rbsp[n] = byte;
            zeros = if byte == 0 { zeros + 1 } else { 0 };
            i += 1; n += 1; offsets[n] = i;
        }
        let h = header(&rbsp[..n], s, p)?;
        if h.data_offset >= n || h.alignment_start + 32 > HEADER_BYTES * 8 {
            return bitstream("HEVC encoder header/data extent");
        }
        let qp_delta = h.qp_delta - p.init_qp_minus26;
        if !(-(6 * (s.bit_depth_luma as i32 - 8))..=51).contains(&(26 + h.qp_delta)) {
            return bitstream("HEVC encoder absolute QP range");
        }
        let mut b = Bw::<HEADER_BYTES>::new();
        for bit in 0..h.qp_start { b.put(u32::from((rbsp[bit / 8] >> (7 - bit % 8)) & 1), 1); }
        b.se(qp_delta);
        for bit in h.qp_end..h.alignment_start { b.put(u32::from((rbsp[bit / 8] >> (7 - bit % 8)) & 1), 1); }
        b.trailing();
        out.extend_from_slice(&data[at..begin]);
        escape(b.bytes(), out);
        out.extend_from_slice(&nal[offsets[h.data_offset]..]);
        at = end;
    }
    Ok(())
}
