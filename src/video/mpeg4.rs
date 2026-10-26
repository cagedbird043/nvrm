//! MPEG-4 part 2 (Simple, Advanced Simple, Main without sprites) on the engine's
//! `SET_APPLICATION_ID` 4 setup (`nvdec_mpeg4_pic_s`, public `nvdec_drv.h`).
//!
//! The engine parses the video object plane header and the macroblock layer itself: the bitstream
//! is one VOP from its `0x000001b6` start code on, and the setup carries only the video object
//! layer syntax and the values libnvcuvid's parser takes from the VOP header (picture type,
//! rounding, f_codes, B-VOP time distances). Layout and values follow stock libnvcuvid 615.78.08
//! as captured in RM traces of I/P/B, quarter-sample, interlaced, MPEG-quantiser, resync-marker and
//! odd-sized streams: setup at the descriptor start, one slice offset at +0x200, engine status at
//! +0x300. Output is two stacked field images per surface, like MPEG-2. Slots: current, forward,
//! backward reference (the current surface for an absent one).
use super::{
    legacy::{field_offsets, Format, Picture, Slots},
    mpeg2::gptimer,
    Allocator, BuildContext, VResult, VideoError, CODEC_MPEG4,
};
use crate::gpu::method;

pub const APP_ID: u32 = 4;
const SLICE_OFFSETS: usize = 0x200;
const STATUS: usize = 0x300;
/// Bytes of picture scratch that precede the history buffer.
const SCRATCH: u64 = 0x400;
/// History bytes per macroblock column.
const HISTORY_PER_COLUMN: u64 = 768;
/// Bytes of per-macroblock side data (motion of the backward reference for direct mode), rounded
/// up to 64 KiB.
const SIDE_PER_MB: u64 = 192;
const SIDE_ALIGN: u64 = 0x10000;
/// `sequence_end_code` (ISO 14496-2 6.2.2), twice, as the stream-end marker pair of the setup.
const EOS: [u8; 16] = [0, 0, 1, 0xb1, 0, 0, 0, 0, 0, 0, 1, 0xb1, 0, 0, 0, 0];

/// `vop_coding_type` of the setup.
pub const PTYPE_I: u8 = 0;
pub const PTYPE_P: u8 = 1;
pub const PTYPE_B: u8 = 2;

/// Video object layer syntax and the per-VOP fields the engine does not derive.
#[derive(Clone, Debug)]
pub struct Params {
    /// `video_object_layer_width` / `_height`.
    pub width: u32,
    pub height: u32,
    /// Bits of `vop_time_increment` (1..=16).
    pub time_increment_bits: u8,
    pub resync_marker_disable: bool,
    pub interlaced: bool,
    pub quant_type: bool,
    pub quarter_sample: bool,
    pub short_video_header: bool,
    pub ptype: u8,
    pub rounding: bool,
    pub alternate_scan: bool,
    pub top_field_first: bool,
    /// `vop_fcode_forward` / `vop_fcode_backward`.
    pub fcode: [u8; 2],
    /// B-VOP time distances: between the references (`TRD`) and from the forward one (`TRB`).
    pub trd: u16,
    pub trb: u16,
    /// Quantiser matrices in raster order (all 16 when the VOL does not load any).
    pub intra_matrix: [u8; 64],
    pub inter_matrix: [u8; 64],
}

#[derive(Default)]
pub struct Mpeg4 {
    side: u64,
    scratch: u64,
}

fn put32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
fn put16(d: &mut [u8], o: usize, v: u16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

impl Format for Mpeg4 {
    type Params = Params;
    const CODEC: u32 = CODEC_MPEG4;
    const STATUS: Option<usize> = Some(STATUS);
    const FIELD_STACKED: bool = true;
    /// Bit 3 shows on correct B-VOPs (every stock capture of a B-VOP stream has some, with no
    /// macroblock in error and output identical to a decode without it).
    const BENIGN_STATUS: u32 = 8;

    fn allocate(&mut self, alloc: &mut dyn Allocator, max_width: u32, max_height: u32) -> VResult<()> {
        let (mbw, mbh) = (u64::from(max_width.div_ceil(16)), u64::from(max_height.div_ceil(16)));
        self.side = alloc.vram((mbw * mbh * SIDE_PER_MB).next_multiple_of(SIDE_ALIGN).max(SIDE_ALIGN) as usize)?;
        self.scratch = alloc.vram((SCRATCH + mbw * HISTORY_PER_COLUMN).next_multiple_of(SIDE_ALIGN) as usize)?;
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
        if p.ptype > PTYPE_B || !(1..=16).contains(&p.time_increment_bits) {
            return Err(VideoError::Unsupported(format!(
                "MPEG-4 picture type {}, {} time increment bits",
                p.ptype, p.time_increment_bits
            )));
        }
        let (mbw, mbh) = (p.width.div_ceil(16), p.height.div_ceil(16));
        let pitch = ctx.layout.pitch;
        let (luma_bot, chroma_bot) = field_offsets(pitch, mbh * 16);
        let b = p.ptype == PTYPE_B;
        d[0x34..0x44].copy_from_slice(&EOS);
        d[0x44] = 1; // explicitEOSPresentFlag
        put32(d, 0x48, ctx.bitstream_len as u32);
        put32(d, 0x4c, 1); // slice_count: the VOP is one unit
        put32(d, 0x50, SCRATCH as u32);
        put32(d, 0x54, gptimer(mbw * mbh));
        put16(d, 0x58, p.width as u16);
        put16(d, 0x5a, p.height as u16);
        d[0x5c] = p.time_increment_bits;
        d[0x5d] = u8::from(p.resync_marker_disable);
        d[0x5e] = 1; // tileFormat 1, gob_height 0
        put32(d, 0x60, p.width);
        put32(d, 0x64, p.height);
        put32(d, 0x68, pitch);
        put32(d, 0x6c, pitch);
        put32(d, 0x74, luma_bot);
        put32(d, 0x80, chroma_bot);
        put32(d, 0x88, ((mbw as u64 * HISTORY_PER_COLUMN) / 256) as u32);
        // Only B-VOPs carry time distances; the others get 1.
        let (trd, trb) = if b { (u32::from(p.trd), u32::from(p.trb)) } else { (1, 1) };
        for i in 0..2 {
            put32(d, 0x8c + 4 * i, trd);
            put32(d, 0x94 + 4 * i, trb);
        }
        put16(d, 0xa0, u16::from(p.fcode[0]));
        put16(d, 0xa2, u16::from(p.fcode[1]));
        d[0xa4] = u8::from(p.interlaced);
        d[0xa5] = u8::from(p.quant_type);
        d[0xa6] = u8::from(p.quarter_sample);
        d[0xa7] = u8::from(p.short_video_header);
        d[0xa8] = 1; // curr_output_memory_layout
        d[0xa9] = p.ptype;
        d[0xaa] = u8::from(p.rounding);
        d[0xab] = u8::from(p.alternate_scan);
        d[0xac] = u8::from(p.top_field_first);
        d[0xb0..0xf0].copy_from_slice(&p.intra_matrix);
        d[0xf0..0x130].copy_from_slice(&p.inter_matrix);
        d[0x130] = 1; // ref_memory_layout[0..2]
        d[0x131] = 1;

        let da = ctx.descriptor;
        let forward = slots.refs[0].unwrap_or(slots.current);
        let backward = slots.refs[1].unwrap_or(slots.current);
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
        ctx.reloc(&mut w, 0x414, self.side);
        ctx.reloc(&mut w, 0x418, self.scratch + SCRATCH);
        ctx.reloc(&mut w, 0x424, da + STATUS as u64);
        method(&mut w, 4, 0x40c, &[ctx.picture_index]);
        ctx.reloc(&mut w, 0x4b8, self.scratch);
        method(&mut w, 4, 0x300, &[0]);
        Ok(w)
    }
}
