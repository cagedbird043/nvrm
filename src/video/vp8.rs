//! VP8 on the engine's `SET_APPLICATION_ID` 5 setup (`nvdec_vp8_pic_s`, public `nvdec_drv.h`).
//!
//! The engine parses the whole frame itself (frame header, probabilities, partitions); the setup
//! carries only the sizes, the frame type and the first partition's length. Values, method list
//! and side buffers follow stock libnvcuvid 615.78.08 as captured in RM traces: setup at the
//! descriptor start, engine status at +0x100, slots golden / alt-ref / last / current, a fixed
//! 0x4b00-byte buffer (method 0x540) directly followed by the history buffer (method 0x418,
//! 512 bytes per macroblock column).
use super::{
    legacy::{Format, Picture, Slots},
    Allocator, BuildContext, VResult, VideoError, CODEC_VP8,
};
use crate::gpu::method;

pub const APP_ID: u32 = 5;
const STATUS: usize = 0x100;
/// Size of the buffer behind method 0x540 (measured: constant for 176x144 to 1280x720).
const SCRATCH: u64 = 0x4b00;
/// Initial contents of the first 0x4cc bytes of the scratch buffer, uploaded by stock before the
/// first picture: the VP8 default probabilities of RFC 6386 in the engine's layout. Coefficient
/// probabilities [4][8][3][11] each padded to 12 bytes (§13.5), y-mode {112, 86, 140, 37} and
/// uv-mode {162, 101, 204} (§16.2) padded to 8, motion-vector probabilities [2][19] (§17.2), zeros.
const INIT_TABLE: &[u8] = include_bytes!("vp8_init.bin");
/// History bytes per macroblock column (`HistBufferSize` = 2 units of 256 per column).
const HISTORY_PER_COLUMN: u64 = 512;
/// Uncompressed data chunk of a key frame: frame tag, start code, 14-bit sizes with scaling bits.
const KEY_CHUNK: usize = 10;
const INTER_CHUNK: usize = 3;

/// One VP8 frame as the VA picture parameter buffer describes it.
#[derive(Clone, Debug)]
pub struct Params {
    pub width: u32,
    pub height: u32,
    pub key_frame: bool,
    pub version: u8,
    /// Bytes of the first partition including the frame header bits already consumed by the
    /// client (the frame tag's `first_part_size`).
    pub first_partition_size: u32,
    /// `update_mb_segmentation_data` of the frame header (0 unless segmentation is enabled).
    pub update_segment_data: bool,
}

impl Params {
    /// Bytes of the uncompressed data chunk (frame tag, and on key frames start code and sizes)
    /// in front of the first partition. The engine reads the partitions without it but counts it
    /// in `VLDBufferSize`, as stock does.
    pub fn chunk_len(&self) -> usize {
        if self.key_frame {
            KEY_CHUNK
        } else {
            INTER_CHUNK
        }
    }
}

#[derive(Default)]
pub struct Vp8 {
    scratch: u64,
    history: u64,
}

fn put32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
fn put16(d: &mut [u8], o: usize, v: u16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

impl Format for Vp8 {
    type Params = Params;
    const CODEC: u32 = CODEC_VP8;
    const STATUS: Option<usize> = Some(STATUS);
    const FIELD_STACKED: bool = false;

    fn allocate(&mut self, alloc: &mut dyn Allocator, max_width: u32, _: u32) -> VResult<()> {
        let history = u64::from(max_width.div_ceil(16)) * HISTORY_PER_COLUMN;
        self.scratch = alloc.vram((SCRATCH + history).next_multiple_of(256) as usize)?;
        self.history = self.scratch + SCRATCH;
        alloc.write(self.scratch, 0, &[0; SCRATCH as usize])?;
        alloc.write(self.scratch, 0, INIT_TABLE)?;
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
        let (mbw, mbh) = (p.width.div_ceil(16), p.height.div_ceil(16));
        let pitch = ctx.layout.pitch;
        // Offsets of the (unused) bottom field images as stock writes them: rows aligned to 16.
        let field = mbh * 8;
        let luma_bot = pitch * field.next_multiple_of(16) / 256;
        let chroma_bot = pitch * (field / 2).next_multiple_of(16) / 256;
        put32(d, 0x34, super::mpeg2::gptimer(mbw * mbh));
        put16(d, 0x38, (mbw * 16) as u16);
        put16(d, 0x3a, (mbh * 16) as u16);
        d[0x3c] = u8::from(p.key_frame);
        d[0x3d] = p.version;
        d[0x3e] = 1; // tileFormat 1, gob_height 0
        d[0x3f] = 1; // errorConcealOn
        put32(d, 0x40, p.first_partition_size);
        put32(d, 0x44, (mbw * 2).max(1));
        put32(d, 0x48, (p.chunk_len() + ctx.bitstream_len) as u32);
        put32(d, 0x4c, pitch / 16);
        put32(d, 0x50, pitch / 16);
        put32(d, 0x58, luma_bot);
        put32(d, 0x64, chroma_bot);
        d[0x8c] = u8::from(p.update_segment_data);

        let cur = slots.current;
        // Stock slot order: golden, alt-ref, last, current. A missing reference decodes against the
        // picture itself (key frames name none).
        let [last, golden, altref] = [0, 1, 2].map(|i| slots.refs[i].unwrap_or(cur));
        let da = ctx.descriptor;
        let mut w = Vec::new();
        method(&mut w, 4, 0x200, &[APP_ID]);
        // Key frames conceal from the current slot (error frame index 3 in bits 7..).
        let conceal = if p.key_frame { 3 << 7 } else { 0 };
        method(&mut w, 4, 0x400, &[0x52050 | APP_ID | conceal]);
        ctx.reloc(&mut w, 0x404, da);
        ctx.reloc(&mut w, 0x408, ctx.bitstream);
        for (i, s) in [golden, altref, last, cur].into_iter().enumerate() {
            ctx.reloc(&mut w, 0x430 + 4 * i as u32, ctx.surfaces[s].luma);
            ctx.reloc(&mut w, 0x474 + 4 * i as u32, ctx.surfaces[s].chroma);
        }
        ctx.reloc(&mut w, 0x418, self.history);
        ctx.reloc(&mut w, 0x424, da + STATUS as u64);
        method(&mut w, 4, 0x40c, &[ctx.picture_index]);
        ctx.reloc(&mut w, 0x540, self.scratch);
        method(&mut w, 4, 0x300, &[0]);
        Ok(w)
    }
}

impl Params {
    pub fn check(&self) -> VResult<()> {
        if self.width == 0 || self.height == 0 || self.width > 0x3fff || self.height > 0x3fff {
            return Err(VideoError::Unsupported(format!(
                "VP8 frame size {}x{}",
                self.width, self.height
            )));
        }
        Ok(())
    }
}
