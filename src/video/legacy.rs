//! Parameter-driven decoders of the pre-VP9 NVDEC formats (MPEG-2, MPEG-4 part 2, VC-1, VP8).
//!
//! The engine parses the macroblock layer itself; the client supplies the sequence/picture level
//! fields (a VA picture's parameter buffers) and the engine-readable bitstream. [`Format`] is the
//! per-codec half: picture parameters → descriptor slot and method list. [`LegacyCodec`] is the
//! shared `CodecDecoder`: VA surface identity ↔ session surfaces, reference resolution, and field
//! pairs (the second field of a frame is decoded into the surface of the first).
//!
//! Descriptor layouts, method lists and side-buffer rules follow stock libnvcuvid 615.78.08 as
//! captured in RM traces (`tools/nvdec-trace`); each format module cites what it measured.
use super::{
    Allocator, BuildContext, CodecDecoder, DecodeJob, Job, PictureInfo, VResult, VideoError,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// References a picture may name: MPEG-2/MPEG-4/VC-1 use forward and backward, VP8 last, golden
/// and alt-ref.
pub const MAX_REFS: usize = 3;

/// Which part of a frame a picture codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Frame,
    /// First field of a frame; the second field follows as its own picture.
    First,
    /// Second field: decoded into the surface of the first.
    Second,
}

/// Offsets of the bottom field image inside a frame surface, in 256-byte units: stock stores the
/// two fields as separate images (luma rows aligned to 32, chroma rows to 16).
pub fn field_offsets(pitch: u32, frame_height: u32) -> (u32, u32) {
    let field = frame_height.div_ceil(2);
    (
        pitch * field.next_multiple_of(32) / 256,
        pitch * field.div_ceil(2).next_multiple_of(16) / 256,
    )
}

/// The VA-visible identity of one coded picture and what the format needs to place it.
pub struct Picture<P> {
    /// VA render target (surface id) the picture decodes into.
    pub target: u32,
    /// VA surface ids this picture predicts from, in the order the format's [`Format::build`]
    /// expects (`None`: the stream refers to nothing there).
    pub refs: [Option<u32>; MAX_REFS],
    pub field: Field,
    pub width: u32,
    pub height: u32,
    pub params: P,
}

/// Session surfaces a picture is built from.
pub struct Slots {
    pub current: usize,
    /// Resolved [`Picture::refs`]; an unknown or absent reference is `None`.
    pub refs: [Option<usize>; MAX_REFS],
}

pub trait Format {
    type Params;
    /// `CODEC_*` of this format.
    const CODEC: u32;
    /// Offset of `nvdec_status_s` in the descriptor slot, if the format reports one.
    const STATUS: Option<usize>;
    /// `error_status` bits the engine sets on pictures it decoded correctly.
    const BENIGN_STATUS: u32 = 0;
    /// The engine writes two stacked field images per picture (see `CodecDecoder::field_stacked`).
    const FIELD_STACKED: bool;
    /// Side buffers, once at session creation.
    fn allocate(&mut self, alloc: &mut dyn Allocator, max_width: u32, max_height: u32) -> VResult<()>;
    /// Descriptor slot and method list of one picture. `bitstream` is the engine bitstream the
    /// session staged for it.
    fn build(
        &mut self,
        picture: &Picture<Self::Params>,
        slots: &Slots,
        ctx: &BuildContext,
        desc: &mut [u8],
    ) -> VResult<Vec<u32>>;
}

/// State shared with the VA frontend: the picture it staged for the next `push_unit`, and the
/// session surface every VA target decoded into.
pub struct Shared<P> {
    pub pending: Option<Picture<P>>,
    surfaces: HashMap<u32, usize>,
}

impl<P> Default for Shared<P> {
    fn default() -> Self {
        Self { pending: None, surfaces: HashMap::new() }
    }
}

impl<P> Shared<P> {
    /// The client released `target`: forget its surface.
    pub fn release(&mut self, target: u32) {
        self.surfaces.remove(&target);
    }
}

pub type Handle<P> = Rc<RefCell<Shared<P>>>;

pub struct LegacyCodec<F: Format> {
    format: F,
    shared: Handle<F::Params>,
    job: Option<Picture<F::Params>>,
    bits: Option<Vec<u8>>,
    /// References of the picture being decoded, kept alive until the next picture.
    refs: Vec<usize>,
    /// Surface of a first field whose second field has not arrived yet.
    half: Option<(u32, usize)>,
    /// Surface the next job writes into (second fields).
    output: Option<usize>,
    max: (u32, u32),
}

impl<F: Format> LegacyCodec<F> {
    pub fn new(format: F, shared: Handle<F::Params>, max_width: u32, max_height: u32) -> Self {
        Self {
            format,
            shared,
            job: None,
            bits: None,
            refs: Vec::new(),
            half: None,
            output: None,
            max: (max_width, max_height),
        }
    }
}

impl<F: Format> CodecDecoder for LegacyCodec<F> {
    fn codec(&self) -> u32 {
        F::CODEC
    }
    fn bit_depth(&self) -> u32 {
        8
    }
    fn dpb_size(&self) -> usize {
        MAX_REFS
    }
    fn allocate(&mut self, alloc: &mut dyn Allocator, w: u32, h: u32) -> VResult<()> {
        self.format.allocate(alloc, w, h)
    }
    fn surfaces_needed(&self, _: &[u8]) -> VResult<usize> {
        let second = self.shared.borrow().pending.as_ref().is_some_and(|p| p.field == Field::Second);
        Ok(usize::from(!second))
    }
    fn push_unit(&mut self, data: &[u8]) -> VResult<()> {
        let picture = self
            .shared
            .borrow_mut()
            .pending
            .take()
            .ok_or_else(|| VideoError::Invalid("missing VA picture".into()))?;
        if picture.width == 0
            || picture.height == 0
            || picture.width > self.max.0.next_multiple_of(16)
            || picture.height > self.max.1.next_multiple_of(16)
        {
            return Err(VideoError::Unsupported(format!(
                "coded size {}x{} outside the {}x{} context",
                picture.width, picture.height, self.max.0, self.max.1
            )));
        }
        self.output = None;
        if picture.field == Field::Second {
            match self.half.take() {
                Some((t, s)) if t == picture.target => self.output = Some(s),
                _ => return Err(VideoError::Bitstream("second field without its first field".into())),
            }
        }
        self.job = Some(picture);
        self.bits = Some(data.to_vec());
        Ok(())
    }
    fn next_job(&mut self) -> VResult<Option<Job>> {
        let (Some(p), Some(bitstream)) = (self.job.as_ref(), self.bits.take()) else {
            return Ok(None);
        };
        let info = PictureInfo {
            width: p.width,
            height: p.height,
            crop: [0, 0, p.width, p.height],
            bit_depth: 8,
            keyframe: p.refs.iter().all(Option::is_none),
            ..Default::default()
        };
        Ok(Some(Job::Decode(DecodeJob {
            info,
            show: true,
            display_key: None,
            bitstream,
            readback: None,
        })))
    }
    fn build(&mut self, _job: &DecodeJob, ctx: &BuildContext, d: &mut [u8]) -> VResult<Vec<u32>> {
        let picture = self
            .job
            .take()
            .ok_or_else(|| VideoError::Invalid("build without a staged VA picture".into()))?;
        let current = self.output.unwrap_or(ctx.current);
        let shared = self.shared.borrow();
        let mut slots = Slots { current, refs: [None; MAX_REFS] };
        for (slot, id) in slots.refs.iter_mut().zip(picture.refs) {
            *slot = id.and_then(|id| shared.surfaces.get(&id).copied());
        }
        drop(shared);
        self.refs = slots.refs.iter().flatten().copied().collect();
        let ctx = BuildContext { current, ..*ctx };
        let words = self.format.build(&picture, &slots, &ctx, d)?;
                self.shared.borrow_mut().surfaces.insert(picture.target, current);
        if picture.field == Field::First {
            self.half = Some((picture.target, current));
        }
        Ok(words)
    }
    fn finish(&mut self, _: &DecodeJob, _: usize, _: Option<&[u8]>) -> VResult<()> {
        Ok(())
    }
    fn referenced(&self) -> Vec<usize> {
        let mut r = self.refs.clone();
        r.extend(self.half.map(|h| h.1));
        r
    }
    fn output_surface(&self) -> Option<usize> {
        self.output
    }
    fn status_offset(&self) -> Option<usize> {
        F::STATUS
    }
    fn field_stacked(&self) -> bool {
        F::FIELD_STACKED
    }
    fn benign_status(&self) -> u32 {
        F::BENIGN_STATUS
    }
    fn flush(&mut self) {
        self.job = None;
        self.bits = None;
        self.refs.clear();
        self.half = None;
        self.output = None;
        *self.shared.borrow_mut() = Shared::default();
    }
}
