//! VC-1 / WMV9 (Simple, Main, Advanced profile) on the engine's `SET_APPLICATION_ID` 2 setup
//! (`nvdec_vc1_pic_s`, public `nvdec_drv.h`).
//!
//! The engine parses the picture layer itself; the setup carries only the sequence and entry-point
//! level syntax and the buffers. Layout and values follow stock libnvcuvid 615.78.08 as captured
//! in RM traces: setup at the descriptor start, slice start offsets at +0x200, engine status at
//! +0x600. Output is two stacked field images per surface, like MPEG-2. Slots: current, forward,
//! backward reference.
use super::{
    legacy::{field_offsets, Format, Picture, Slots},
    mpeg2::gptimer,
    Allocator, BuildContext, VResult, VideoError, CODEC_VC1,
};
use crate::gpu::method;

pub const APP_ID: u32 = 2;
const SLICE_OFFSETS: usize = 0x200;
const STATUS: usize = 0x600;
pub const MAX_SLICES: usize = (STATUS - SLICE_OFFSETS) / 4;
/// Control word bit of a bidirectionally predicted picture.
const CONTROL_B_PICTURE: u32 = 0x80;
/// `profile` of the setup: 1 = Simple/Main, 2 = Advanced.
const PROFILE_ADVANCED: u8 = 2;
const PROFILE_SIMPLE_MAIN: u8 = 1;
/// Frame start code (`prefixStartCode`) of Simple/Main pictures.
const PREFIX_START_CODE: [u8; 4] = [0, 0, 1, 0x0d];
/// Bytes of the entry-point scratch that precede the history buffer.
const SCRATCH: u64 = 0x200;
/// History bytes per macroblock column.
const HISTORY_PER_COLUMN: u64 = 768;
/// Bytes of per-macroblock side data (direct-mode motion of the references), at least 64 KiB.
const SIDE_PER_MB: u64 = 384;
const SIDE_MIN: u64 = 0x10000;

/// Sequence and entry-point level syntax plus the picture kind the control word depends on.
#[derive(Clone, Debug, Default)]
pub struct Params {
    /// Coded size of the entry point.
    pub width: u32,
    pub height: u32,
    pub advanced: bool,
    pub postproc: bool,
    pub pulldown: bool,
    pub interlace: bool,
    pub tfcntr: bool,
    pub finterp: bool,
    pub psf: bool,
    pub multires: bool,
    pub syncmarker: bool,
    pub rangered: bool,
    pub max_b_frames: u8,
    pub dquant: u8,
    pub panscan: bool,
    pub refdist: bool,
    pub quantizer: u8,
    pub extended_mv: bool,
    pub extended_dmv: bool,
    pub overlap: bool,
    pub vstransform: bool,
    pub loopfilter: bool,
    pub fastuvmc: bool,
    /// B or BI picture.
    pub b_picture: bool,
    /// Start of every slice in the staged bitstream.
    pub slice_offsets: Vec<u32>,
}

#[derive(Default)]
pub struct Vc1 {
    side: u64,
    scratch: u64,
}

fn put32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
fn put16(d: &mut [u8], o: usize, v: u16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

impl Format for Vc1 {
    type Params = Params;
    const CODEC: u32 = CODEC_VC1;
    const STATUS: Option<usize> = Some(STATUS);
    const FIELD_STACKED: bool = true;

    fn allocate(&mut self, alloc: &mut dyn Allocator, max_width: u32, max_height: u32) -> VResult<()> {
        let (mbw, mbh) = (u64::from(max_width.div_ceil(16)), u64::from(max_height.div_ceil(16)));
        self.side = alloc.vram((mbw * mbh * SIDE_PER_MB).max(SIDE_MIN).next_multiple_of(256) as usize)?;
        self.scratch = alloc.vram((SCRATCH + mbw * HISTORY_PER_COLUMN).max(SIDE_MIN).next_multiple_of(256) as usize)?;
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
                "VC-1 picture with {} slices (1..={MAX_SLICES})",
                p.slice_offsets.len()
            )));
        }
        let (mbw, mbh) = (p.width.div_ceil(16), p.height.div_ceil(16));
        let pitch = ctx.layout.pitch;
        let (luma_bot, chroma_bot) = field_offsets(pitch, mbh * 16);
        if !p.advanced {
            // Simple/Main streams have no start codes: the engine takes the frame start code here.
            d[0x44..0x48].copy_from_slice(&PREFIX_START_CODE);
        }
        d[0x4c] = 1; // explicitEOSPresentFlag (eos[] stays zero)
        put32(d, 0x50, ctx.bitstream_len as u32);
        put32(d, 0x54, p.slice_offsets.len() as u32);
        put32(d, 0x58, SCRATCH as u32);
        put32(d, 0x5c, gptimer(mbw * mbh));
        put16(d, 0x60, p.width as u16);
        put16(d, 0x62, p.height as u16);
        d[0x64] = if p.advanced { PROFILE_ADVANCED } else { PROFILE_SIMPLE_MAIN };
        d[0x65] = u8::from(p.postproc);
        d[0x66] = u8::from(p.pulldown);
        d[0x67] = u8::from(p.interlace);
        d[0x68] = u8::from(p.tfcntr);
        d[0x69] = u8::from(p.finterp);
        d[0x6a] = u8::from(p.psf);
        d[0x6b] = 1; // tileFormat 1, gob_height 0
        d[0x6c] = u8::from(p.multires);
        d[0x6d] = u8::from(p.syncmarker);
        d[0x6e] = u8::from(p.rangered);
        d[0x6f] = p.max_b_frames;
        d[0x70] = p.dquant;
        d[0x71] = u8::from(p.panscan);
        d[0x72] = u8::from(p.refdist);
        d[0x73] = p.quantizer;
        d[0x74] = u8::from(p.extended_mv);
        d[0x75] = u8::from(p.extended_dmv);
        d[0x76] = u8::from(p.overlap);
        d[0x77] = u8::from(p.vstransform);
        put32(d, 0x13c, mbw * 3);
        put32(d, 0x140, pitch);
        put32(d, 0x144, pitch);
        put32(d, 0x14c, luma_bot);
        put32(d, 0x158, chroma_bot);
        put16(d, 0x160, p.width as u16);
        put16(d, 0x162, p.height as u16);
        d[0x164] = u8::from(p.loopfilter);
        d[0x165] = u8::from(p.fastuvmc);
        d[0x166] = 1; // output_memory_layout
        d[0x167] = 1; // ref_memory_layout[0]
        d[0x168] = 1; // ref_memory_layout[1]
        for (i, &o) in p.slice_offsets.iter().enumerate() {
            put32(d, SLICE_OFFSETS + 4 * i, o);
        }

        let da = ctx.descriptor;
        let forward = slots.refs[0].unwrap_or(slots.current);
        let backward = slots.refs[1].unwrap_or(forward);
        let surface = |s: usize| (ctx.surfaces[s].luma, ctx.surfaces[s].chroma);
        let control = 0x52050 | APP_ID | if p.b_picture { CONTROL_B_PICTURE } else { 0 };
        let mut w = Vec::new();
        method(&mut w, 4, 0x200, &[APP_ID]);
        method(&mut w, 4, 0x400, &[control]);
        ctx.reloc(&mut w, 0x404, da);
        ctx.reloc(&mut w, 0x408, ctx.bitstream);
        ctx.reloc(&mut w, 0x410, da + SLICE_OFFSETS as u64);
        for (i, s) in [slots.current, forward, backward].into_iter().enumerate() {
            let (luma, chroma) = surface(s);
            ctx.reloc(&mut w, 0x430 + 4 * i as u32, luma);
            ctx.reloc(&mut w, 0x474 + 4 * i as u32, chroma);
        }
        ctx.reloc(&mut w, 0x414, self.side);
        ctx.reloc(&mut w, 0x418, self.scratch + SCRATCH);
        ctx.reloc(&mut w, 0x424, da + STATUS as u64);
        method(&mut w, 4, 0x40c, &[ctx.picture_index]);
        ctx.reloc(&mut w, 0x4b8, self.scratch);
        let (luma, chroma) = surface(slots.current);
        ctx.reloc(&mut w, 0x428, luma);
        ctx.reloc(&mut w, 0x42c, chroma);
        method(&mut w, 4, 0x300, &[0]);
        Ok(w)
    }
}
