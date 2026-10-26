//! MPEG-2 video (and MPEG-1 constrained to the same syntax) on the engine's `SET_APPLICATION_ID` 1
//! setup (`nvdec_mpeg2_pic_s`, public `nvdec_drv.h`).
//!
//! Layout and values follow stock libnvcuvid 615.78.08 as captured in RM traces of progressive,
//! interlaced-frame and field-coded streams (byte for byte in `tests`): the setup sits at the
//! descriptor start, the slice start-offset table at +0x200, the engine status at +0x900. The
//! bitstream is the picture's slices with their start codes; the setup carries two end-of-sequence
//! start codes (`explicitEOSPresentFlag`). Slots: current, forward, backward reference.
use super::{
    legacy::{field_offsets, Field, Format, Picture, Slots},
    Allocator, BuildContext, VResult, VideoError, CODEC_MPEG2,
};
use crate::gpu::method;

pub const APP_ID: u32 = 1;
const SLICE_OFFSETS: usize = 0x200;
const STATUS: usize = 0x900;
/// Slice offsets that fit between the table start and the status.
pub const MAX_SLICES: usize = (STATUS - SLICE_OFFSETS) / 4;

/// ISO 13818-2 default matrices (6.3.11), raster order.
pub const DEFAULT_INTRA: [u8; 64] = [
    8, 16, 19, 22, 26, 27, 29, 34, 16, 16, 22, 24, 27, 29, 34, 37, 19, 22, 26, 27, 29, 34, 34, 38,
    22, 22, 26, 27, 29, 34, 37, 40, 22, 26, 27, 29, 32, 35, 40, 48, 26, 27, 29, 32, 35, 40, 48, 58,
    26, 27, 29, 34, 38, 46, 56, 69, 27, 29, 35, 38, 46, 56, 69, 83,
];
pub const DEFAULT_INTER: [u8; 64] = [16; 64];

/// `picture_structure` (Table 6-14).
pub const TOP_FIELD: u8 = 1;
pub const BOTTOM_FIELD: u8 = 2;
pub const FRAME_PICTURE: u8 = 3;

/// Sequence and picture level syntax of one MPEG-2 picture, as the VA picture parameter buffer
/// carries it.
#[derive(Clone, Debug)]
pub struct Params {
    /// `horizontal_size` / `vertical_size` of the sequence.
    pub width: u32,
    pub height: u32,
    pub picture_structure: u8,
    pub picture_coding_type: u8,
    pub intra_dc_precision: u8,
    pub frame_pred_frame_dct: bool,
    pub concealment_motion_vectors: bool,
    pub intra_vlc_format: bool,
    pub q_scale_type: bool,
    pub alternate_scan: bool,
    pub top_field_first: bool,
    /// `f_code[s][t]` flattened: forward horizontal/vertical, backward horizontal/vertical.
    pub f_code: [u8; 4],
    /// Quantiser matrices in raster order.
    pub intra_matrix: [u8; 64],
    pub inter_matrix: [u8; 64],
    /// Start of every slice in the bitstream.
    pub slice_offsets: Vec<u32>,
}

#[derive(Default)]
pub struct Mpeg2;

fn put32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
fn put16(d: &mut [u8], o: usize, v: u16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

/// Stock `gptimer_timeout_value`: three frames' worth of macroblocks at 10000 cycles, at least
/// a 1080p frame.
pub fn gptimer(mbs: u32) -> u32 {
    mbs.max(8100).saturating_mul(10000)
}

impl Format for Mpeg2 {
    type Params = Params;
    const CODEC: u32 = CODEC_MPEG2;
    const STATUS: Option<usize> = Some(STATUS);
    const FIELD_STACKED: bool = true;

    fn allocate(&mut self, _: &mut dyn Allocator, _: u32, _: u32) -> VResult<()> {
        Ok(())
    }

    fn build(
        &mut self,
        pic: &Picture<Params>,
        slots: &Slots,
        ctx: &BuildContext,
        d: &mut [u8],
    ) -> VResult<Vec<u32>> {
        let p = &pic.params;
        if p.slice_offsets.is_empty() || p.slice_offsets.len() > MAX_SLICES {
            return Err(VideoError::Unsupported(format!(
                "MPEG-2 picture with {} slices (1..={MAX_SLICES})",
                p.slice_offsets.len()
            )));
        }
        let (mbw, mbh) = (p.width.div_ceil(16), p.height.div_ceil(16));
        let pitch = ctx.layout.pitch;
        let (luma_bot, chroma_bot) = field_offsets(pitch, mbh * 16);
        // eos[16]: two sequence_end_code start codes.
        d[0x34..0x44].copy_from_slice(&[0, 0, 1, 0xb7, 0, 0, 0, 0, 0, 0, 1, 0xb7, 0, 0, 0, 0]);
        put32(d, 0x44, 1);
        put32(d, 0x48, ctx.bitstream_len as u32);
        put32(d, 0x4c, p.slice_offsets.len() as u32);
        put32(d, 0x50, gptimer(mbw * mbh));
        put16(d, 0x54, p.width as u16);
        put16(d, 0x56, p.height as u16);
        d[0x58] = p.picture_structure;
        d[0x59] = p.picture_coding_type;
        d[0x5a] = p.intra_dc_precision;
        d[0x5b] = u8::from(p.frame_pred_frame_dct);
        d[0x5c] = u8::from(p.concealment_motion_vectors);
        d[0x5d] = u8::from(p.intra_vlc_format);
        d[0x5e] = 1; // tileFormat 1, gob_height 0
        d[0x60..0x64].copy_from_slice(&p.f_code);
        put16(d, 0x64, mbw as u16);
        put16(d, 0x66, mbh as u16);
        put32(d, 0x68, pitch);
        put32(d, 0x6c, pitch);
        put32(d, 0x74, luma_bot);
        put32(d, 0x80, chroma_bot);
        put16(d, 0x8c, 1); // output_memory_layout
        put16(d, 0x8e, u16::from(p.alternate_scan));
        put16(d, 0x90, u16::from(pic.field == Field::Second));
        put32(d, 0x98, u32::from(p.q_scale_type));
        put32(d, 0x9c, u32::from(p.top_field_first));
        d[0xa8..0xe8].copy_from_slice(&p.intra_matrix);
        d[0xe8..0x128].copy_from_slice(&p.inter_matrix);
        put32(d, 0x128, 1);
        put32(d, 0x12c, 1);
        for (i, &o) in p.slice_offsets.iter().enumerate() {
            put32(d, SLICE_OFFSETS + 4 * i, o);
        }

        let da = ctx.descriptor;
        let forward = slots.refs[0].unwrap_or(slots.current);
        let backward = slots.refs[1].unwrap_or(forward);
        let mut w = Vec::new();
        method(&mut w, 4, 0x200, &[APP_ID]);
        method(&mut w, 4, 0x400, &[0x52050 | APP_ID]);
        ctx.reloc(&mut w, 0x404, da);
        ctx.reloc(&mut w, 0x408, ctx.bitstream);
        ctx.reloc(&mut w, 0x410, da + SLICE_OFFSETS as u64);
        for (i, s) in [slots.current, forward, backward].into_iter().enumerate() {
            ctx.reloc(&mut w, 0x430 + 4 * i as u32, ctx.surfaces[s].luma);
            ctx.reloc(&mut w, 0x474 + 4 * i as u32, ctx.surfaces[s].chroma);
        }
        ctx.reloc(&mut w, 0x424, da + STATUS as u64);
        method(&mut w, 4, 0x40c, &[ctx.picture_index]);
        method(&mut w, 4, 0x300, &[0]);
        Ok(w)
    }
}
