//! Video processing for VA VideoProc: scaling, cropping, colour and format conversion of one
//! picture into another on the GPU, plus the CPU read-back of a dma-buf picture.
//!
//! A conversion runs in three steps, none of them on the CPU: the copy engine brings the input
//! planes into a pitch-linear VRAM scratch (a CPU-provided input is staged in system memory and
//! read from there), the `vpp` kernel (shaders/vpp.comp) writes the output planes into the
//! scratch, and the copy engine lays them into the output dma-buf (linear or NVIDIA block linear).
//! A same-format, same-size, uncropped copy without colour change is one copy-engine pass per
//! plane. Only GPUs whose compute class is hardware-validated (compute.rs) run the kernel;
//! [`Processor::supported`] tells the frontend whether to offer VideoProc at all.
use super::{
    compute::ComputeEngine,
    enc::ingest::{Ingest, Layout, Plane},
    open_render_node, open_rm,
    session::{vram, CopyEngine, VramBuffer},
    VResult, VideoError,
};
use crate::{
    gpu::{hi_lo, method, AddressSpace, GpuMapping},
    rm::{Device, Mapping, Memory as RmMemory},
};
use std::{
    collections::HashMap,
    os::fd::{AsFd, AsRawFd, BorrowedFd},
    rc::Rc,
};

/// Byte order of a packed 8-bit RGB picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgbOrder {
    /// B, G, R, A/X in memory.
    Bgra,
    /// R, G, B, A/X in memory.
    Rgba,
}

/// Sample layout of a picture. YUV samples of more than 8 bits sit in the high bits of 16-bit
/// containers; `bits` is the depth the codes are read and written at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pixels {
    /// Luma and interleaved CbCr at half resolution in both directions (NV12, P010, P012, P016).
    Yuv420 { bits: u32 },
    /// Luma, Cb and Cr planes at full resolution (444P, Q416).
    Yuv444 { bits: u32 },
    /// One plane of 4-byte pixels.
    Rgb(RgbOrder),
}

impl Pixels {
    pub fn planes(self) -> usize {
        match self {
            Pixels::Yuv420 { .. } => 2,
            Pixels::Yuv444 { .. } => 3,
            Pixels::Rgb(_) => 1,
        }
    }
    fn bits(self) -> u32 {
        match self {
            Pixels::Yuv420 { bits } | Pixels::Yuv444 { bits } => bits,
            Pixels::Rgb(_) => 8,
        }
    }
    /// Bytes per sample (per pixel for RGB).
    fn bps(self) -> u32 {
        match self {
            Pixels::Rgb(_) => 4,
            p if p.bits() > 8 => 2,
            _ => 1,
        }
    }
    /// (bytes per row, rows) of plane `p` of a `width` x `height` picture.
    pub fn plane_extent(self, p: usize, width: u32, height: u32) -> (u32, u32) {
        match (self, p) {
            (Pixels::Yuv420 { .. }, 1) => (width.div_ceil(2) * 2 * self.bps(), height.div_ceil(2)),
            _ => (width * self.bps(), height),
        }
    }
    fn valid(self) -> bool {
        match self {
            Pixels::Yuv420 { bits } | Pixels::Yuv444 { bits } => bits == 8 || (9..=16).contains(&bits),
            Pixels::Rgb(_) => true,
        }
    }
}

/// Where a picture's planes are: a dma-buf (its descriptors borrowed for the call; `key` names
/// the buffer in the import cache until [`Processor::forget`]), or CPU memory.
pub enum Memory<'a> {
    Dma { key: u64, planes: [Plane; 3], layout: Layout },
    Host { bytes: &'a [u8], pitch: usize, offsets: [usize; 3] },
}

pub struct Picture<'a> {
    pub pixels: Pixels,
    pub width: u32,
    pub height: u32,
    pub memory: Memory<'a>,
}

/// YUV <-> R'G'B' matrix (Kr, Kb of ITU-T H.273); `Rgb` is the identity of RGB pictures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matrix {
    Rgb,
    Bt601,
    Bt709,
    Bt2020,
    Fcc,
    Smpte240,
}

impl Matrix {
    fn kr_kb(self) -> (f64, f64) {
        match self {
            Matrix::Rgb | Matrix::Bt709 => (0.2126, 0.0722),
            Matrix::Bt601 => (0.299, 0.114),
            Matrix::Bt2020 => (0.2627, 0.0593),
            Matrix::Fcc => (0.30, 0.11),
            Matrix::Smpte240 => (0.212, 0.087),
        }
    }
}

/// Colour of one side of an operation. `chroma_site` is the centre of the first 4:2:0 chroma
/// sample in luma pixels (x: 0.5 left, 1.0 centre; y: 0.5 top, 1.0 centre, 1.5 bottom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colour {
    pub matrix: Matrix,
    pub full_range: bool,
    pub chroma_site: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// One VideoProc pipeline: the `src_rect` of the input scaled into the `dst_rect` of the output,
/// every output pixel outside `dst_rect` set to `background` (0xAARRGGBB, full-range R'G'B').
#[derive(Clone, Copy, Debug)]
pub struct Operation {
    pub src_rect: Rect,
    pub dst_rect: Rect,
    pub src_colour: Colour,
    pub dst_colour: Colour,
    pub background: u32,
}

type Affine = [[f64; 4]; 3];

/// (offset, scale) of luma / R'G'B' and of chroma codes at `bits` for `full_range`.
fn levels(bits: u32, full_range: bool, rgb: bool) -> ((f64, f64), (f64, f64)) {
    let unit = f64::from(1u32 << (bits - 8));
    let max = f64::from((1u32 << bits) - 1);
    let luma = if full_range { (0.0, max) } else { (16.0 * unit, 219.0 * unit) };
    let chroma = if rgb {
        luma
    } else if full_range {
        (f64::from(1u32 << (bits - 1)), max)
    } else {
        (128.0 * unit, 224.0 * unit)
    };
    (luma, chroma)
}

fn compose(a: &Affine, b: &Affine) -> Affine {
    let mut out = [[0.0; 4]; 3];
    for r in 0..3 {
        for c in 0..4 {
            out[r][c] = (0..3).map(|k| a[r][k] * b[k][c]).sum::<f64>() + if c == 3 { a[r][3] } else { 0.0 };
        }
    }
    out
}

/// Codes of `pixels` in `colour` -> R'G'B' in 0..1.
fn decode(pixels: Pixels, colour: &Colour) -> Affine {
    let rgb = matches!(pixels, Pixels::Rgb(_));
    let ((yo, ys), (co, cs)) = levels(pixels.bits(), colour.full_range, rgb);
    if rgb {
        return [[1.0 / ys, 0.0, 0.0, -yo / ys], [0.0, 1.0 / ys, 0.0, -yo / ys], [0.0, 0.0, 1.0 / ys, -yo / ys]];
    }
    let (kr, kb) = colour.matrix.kr_kb();
    let kg = 1.0 - kr - kb;
    // Y', Pb, Pr from codes.
    let unit: Affine = [[1.0 / ys, 0.0, 0.0, -yo / ys], [0.0, 1.0 / cs, 0.0, -co / cs], [0.0, 0.0, 1.0 / cs, -co / cs]];
    let (rv, bu) = (2.0 * (1.0 - kr), 2.0 * (1.0 - kb));
    let to_rgb: Affine = [
        [1.0, 0.0, rv, 0.0],
        [1.0, -bu * kb / kg, -rv * kr / kg, 0.0],
        [1.0, bu, 0.0, 0.0],
    ];
    compose(&to_rgb, &unit)
}

/// R'G'B' in 0..1 -> codes of `pixels` in `colour`.
fn encode(pixels: Pixels, colour: &Colour) -> Affine {
    let rgb = matches!(pixels, Pixels::Rgb(_));
    let ((yo, ys), (co, cs)) = levels(pixels.bits(), colour.full_range, rgb);
    if rgb {
        return [[ys, 0.0, 0.0, yo], [0.0, ys, 0.0, yo], [0.0, 0.0, ys, yo]];
    }
    let (kr, kb) = colour.matrix.kr_kb();
    let kg = 1.0 - kr - kb;
    let (rv, bu) = (2.0 * (1.0 - kr), 2.0 * (1.0 - kb));
    let to_ypbpr: Affine = [
        [kr, kg, kb, 0.0],
        [-kr / bu, -kg / bu, (1.0 - kb) / bu, 0.0],
        [(1.0 - kr) / rv, -kg / rv, -kb / rv, 0.0],
    ];
    let codes: Affine = [[ys, 0.0, 0.0, yo], [0.0, cs, 0.0, co], [0.0, 0.0, cs, co]];
    compose(&codes, &to_ypbpr)
}

/// The affine map from input codes to output codes of `op` (see shaders/vpp.comp).
pub fn conversion(src: Pixels, dst: Pixels, op: &Operation) -> [[f64; 4]; 3] {
    compose(&encode(dst, &op.dst_colour), &decode(src, &op.src_colour))
}

/// Output codes (c0, c1, c2) of the background colour.
pub fn background_codes(dst: Pixels, op: &Operation) -> [u32; 3] {
    let rgb = [(op.background >> 16) & 255, (op.background >> 8) & 255, op.background & 255];
    let m = encode(dst, &op.dst_colour);
    let max = f64::from((1u32 << dst.bits()) - 1);
    let mut out = [0u32; 3];
    for (r, o) in out.iter_mut().enumerate() {
        let v = (0..3).map(|k| m[r][k] * f64::from(rgb[k]) / 255.0).sum::<f64>() + m[r][3];
        *o = (v + 0.5).floor().clamp(0.0, max) as u32;
    }
    out
}

struct Import {
    memory: Rc<RmMemory>,
    gpu: GpuMapping,
}

/// System memory the CPU writes inputs into and reads results from, mapped for the GPU.
struct Host {
    // Field order is drop order: CPU view, GPU view, memory.
    cpu: Mapping,
    gpu: GpuMapping,
    _memory: Rc<RmMemory>,
    size: usize,
}

/// A plane as the copy engine and the kernel address it.
#[derive(Clone, Copy)]
struct Surface {
    address: u64,
    pitch: u32,
    layout: Layout,
}

const PARAMS_BYTES: usize = 256;

pub struct Processor {
    compute: ComputeEngine,
    engine: CopyEngine,
    imports: HashMap<u64, Vec<(i32, Import)>>,
    scratch: Option<(VramBuffer, usize)>,
    host: Option<Host>,
    drm: std::fs::File,
    space: AddressSpace,
    device: Rc<Device>,
}

impl Processor {
    /// Whether the GPU at `pci` runs the VideoProc kernel (its compute class is validated).
    pub fn supported(pci: [u32; 4]) -> VResult<bool> {
        let (device, _) = open_rm(Some(pci))?;
        Ok(ComputeEngine::supported(&device)?.is_some())
    }

    pub fn open(pci: [u32; 4]) -> VResult<Self> {
        let (device, pci) = open_rm(Some(pci))?;
        let space = AddressSpace::new(&device)?;
        let compute = ComputeEngine::new(&device, &space)?;
        let engine = CopyEngine::new(&device, &space)?;
        let drm = open_render_node(pci)?;
        Ok(Self { compute, engine, imports: HashMap::new(), scratch: None, host: None, drm, space, device })
    }

    /// Drops the imports of the dma-bufs named `key`.
    pub fn forget(&mut self, key: u64) {
        self.imports.remove(&key);
    }

    /// GPU address and size of the dma-buf `fd` of `key`, imported on first use.
    fn import(&mut self, key: u64, fd: i32) -> VResult<(u64, u64)> {
        if fd < 0 {
            return Err(VideoError::Invalid(format!("dma-buf fd {fd}")));
        }
        let list = self.imports.entry(key).or_default();
        if let Some((_, i)) = list.iter().find(|(f, _)| *f == fd) {
            return Ok((i.gpu.address(), i.memory.size as u64));
        }
        // SAFETY: the caller lends a live fd for the duration of the call.
        let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
        let memory = crate::external::import(&self.device, self.drm.as_fd(), borrowed)
            .map_err(|e| VideoError::Invalid(format!("dma-buf import: {e}")))?;
        let gpu = self.space.map(&memory)?;
        let out = (gpu.address(), memory.size as u64);
        list.push((fd, Import { memory, gpu }));
        Ok(out)
    }

    /// The planes of a dma-buf picture, checked against the buffer sizes.
    fn dma_planes(&mut self, pic: &Picture) -> VResult<Vec<Surface>> {
        let Memory::Dma { key, planes, layout } = &pic.memory else {
            return Err(VideoError::Invalid("picture is not a dma-buf".into()));
        };
        let mut out = Vec::with_capacity(3);
        for (p, plane) in planes[..pic.pixels.planes()].iter().enumerate() {
            let (address, size) = self.import(*key, plane.fd)?;
            let (line, rows) = pic.pixels.plane_extent(p, pic.width, pic.height);
            Ingest::check_plane(*layout, plane, line, rows, size)?;
            out.push(Surface { address: address + plane.offset, pitch: plane.pitch, layout: *layout });
        }
        Ok(out)
    }

    fn scratch(&mut self, bytes: usize) -> VResult<u64> {
        if self.scratch.as_ref().is_none_or(|s| s.1 < bytes) {
            self.scratch = None;
            let size = bytes.next_multiple_of(1 << 20);
            self.scratch = Some((vram(&self.device, &self.space, size)?, size));
        }
        Ok(self.scratch.as_ref().unwrap().0.gpu.address())
    }

    fn host(&mut self, bytes: usize) -> VResult<&mut Host> {
        if self.host.as_ref().is_none_or(|h| h.size < bytes) {
            self.host = None;
            let size = bytes.next_multiple_of(1 << 20);
            let memory = self.device.allocate_aligned(size, 4096, false, true)?;
            let gpu = self.space.map(&memory)?;
            let cpu = memory.map()?;
            self.host = Some(Host { cpu, gpu, _memory: memory, size });
        }
        Ok(self.host.as_mut().unwrap())
    }

    /// Appends the copy of a `line`-byte x `rows` rectangle from `src` to `dst` (cl90b5.h).
    fn rectangle(words: &mut Vec<u32>, src: Surface, dst: Surface, line: u32, rows: u32) {
        let mut launch = 2 | 1 << 2 | 1 << 9; // non-pipelined, flush, multi-line
        match src.layout {
            Layout::Linear => launch |= 1 << 7,
            Layout::BlockLinear { log2_height } => {
                method(words, 4, 0x728, &[log2_height << 4 | 1 << 12]); // SET_SRC_BLOCK_SIZE
                method(words, 4, 0x72c, &[src.pitch, rows.next_multiple_of(8 << log2_height), 1, 0]);
                method(words, 4, 0x744, &[0, 0]); // SRC_ORIGIN_X/Y
            }
        }
        match dst.layout {
            Layout::Linear => launch |= 1 << 8,
            Layout::BlockLinear { log2_height } => {
                method(words, 4, 0x70c, &[log2_height << 4 | 1 << 12]); // SET_DST_BLOCK_SIZE
                method(words, 4, 0x710, &[dst.pitch, rows.next_multiple_of(8 << log2_height), 1, 0]);
                method(words, 4, 0x74c, &[0, 0]); // DST_ORIGIN_X/Y
            }
        }
        let [ih, il] = hi_lo(src.address);
        let [oh, ol] = hi_lo(dst.address);
        method(words, 4, 0x400, &[ih, il, oh, ol]);
        method(words, 4, 0x410, &[src.pitch, dst.pitch, line, rows]);
        method(words, 4, 0x300, &[launch]);
    }

    /// Copies whole planes `pairs` (source, destination, plane index) of a `pixels` picture.
    fn copy_planes(&mut self, pixels: Pixels, width: u32, height: u32, pairs: &[(Surface, Surface, usize)]) -> VResult<()> {
        let mut words = Vec::with_capacity(64);
        method(&mut words, 4, 0, &[self.engine.copy_class]); // SET_OBJECT
        for &(src, dst, p) in pairs {
            let (line, rows) = pixels.plane_extent(p, width, height);
            Self::rectangle(&mut words, src, dst, line, rows);
        }
        let drm = self.drm.as_raw_fd();
        self.engine.run(&self.device, &self.space, drm, &words)
    }

    /// Pitch-linear plane geometry of a `pixels` picture: (pitches, offsets, total bytes), each
    /// plane 256-byte aligned with a 64-byte multiple pitch covering whole 4-column groups.
    fn linear(pixels: Pixels, width: u32, height: u32) -> ([u32; 3], [usize; 3], usize) {
        let (mut pitches, mut offsets, mut at) = ([0u32; 3], [0usize; 3], 0usize);
        for p in 0..pixels.planes() {
            let (line, rows) = pixels.plane_extent(p, width.next_multiple_of(4), height);
            pitches[p] = line.next_multiple_of(64);
            offsets[p] = at;
            at = (at + pitches[p] as usize * rows as usize).next_multiple_of(256);
        }
        (pitches, offsets, at)
    }

    /// Runs `op` from `src` (a dma-buf or CPU memory) into the dma-buf `dst` and returns after
    /// the GPU finished.
    pub fn process(&mut self, src: &Picture, dst: &Picture, op: &Operation) -> VResult<()> {
        for pic in [src, dst] {
            if !pic.pixels.valid() || pic.width == 0 || pic.height == 0 || pic.width > 16384 || pic.height > 16384 {
                return Err(VideoError::Invalid(format!("picture {:?} {}x{}", pic.pixels, pic.width, pic.height)));
            }
        }
        let s = op.src_rect;
        if s.x < 0 || s.y < 0 || s.width == 0 || s.height == 0
            || s.x as u64 + u64::from(s.width) > u64::from(src.width)
            || s.y as u64 + u64::from(s.height) > u64::from(src.height)
        {
            return Err(VideoError::Invalid(format!("source region {s:?} outside {}x{}", src.width, src.height)));
        }
        if op.dst_rect.width == 0 || op.dst_rect.height == 0 {
            return Err(VideoError::Invalid("empty output region".into()));
        }
        let dst_planes = self.dma_planes(dst)?;
        let whole = |r: Rect, p: &Picture| r == Rect { x: 0, y: 0, width: p.width, height: p.height };
        let same_colour = src.pixels == dst.pixels
            && op.src_colour.full_range == op.dst_colour.full_range
            && (matches!(src.pixels, Pixels::Rgb(_)) || op.src_colour.matrix == op.dst_colour.matrix)
            && (!matches!(src.pixels, Pixels::Yuv420 { .. }) || op.src_colour.chroma_site == op.dst_colour.chroma_site);
        if same_colour && (src.width, src.height) == (dst.width, dst.height) && whole(op.src_rect, src) && whole(op.dst_rect, dst) {
            let src_planes = self.stage_input(src)?;
            let pairs: Vec<_> = (0..src.pixels.planes()).map(|p| (src_planes[p], dst_planes[p], p)).collect();
            return self.copy_planes(src.pixels, src.width, src.height, &pairs);
        }

        // Scratch: [input planes when the input is a dma-buf][output planes][parameters].
        let (in_pitch, in_off, in_bytes) = Self::linear(src.pixels, src.width, src.height);
        let (out_pitch, out_off, out_bytes) = Self::linear(dst.pixels, dst.width, dst.height);
        let dma_input = matches!(src.memory, Memory::Dma { .. });
        let out_at = if dma_input { in_bytes } else { 0 };
        let params_at = out_at + out_bytes;
        let base = self.scratch(params_at + PARAMS_BYTES)?;
        let inputs: Vec<(u64, u32)> = if dma_input {
            let planes = self.dma_planes(src)?;
            let pairs: Vec<_> = (0..src.pixels.planes())
                .map(|p| (planes[p], Surface { address: base + in_off[p] as u64, pitch: in_pitch[p], layout: Layout::Linear }, p))
                .collect();
            self.copy_planes(src.pixels, src.width, src.height, &pairs)?;
            (0..src.pixels.planes()).map(|p| (base + in_off[p] as u64, in_pitch[p])).collect()
        } else {
            self.stage_input(src)?.iter().map(|s| (s.address, s.pitch)).collect()
        };

        let params = self.params(src, dst, op, &inputs, out_pitch);
        let scratch = &mut self.scratch.as_mut().unwrap().0;
        for (i, w) in params.iter().enumerate() {
            scratch.cpu.write_u32(params_at + i * 4, *w)?;
        }
        crate::platform::device_full_barrier();
        let mut bindings = Vec::with_capacity(7);
        for (p, &(address, pitch)) in inputs.iter().enumerate() {
            let (_, rows) = src.pixels.plane_extent(p, src.width, src.height);
            bindings.push((p as u32, address, pitch * rows));
        }
        for p in 0..dst.pixels.planes() {
            let (_, rows) = dst.pixels.plane_extent(p, dst.width, dst.height);
            bindings.push((3 + p as u32, base + (out_at + out_off[p]) as u64, out_pitch[p] * rows));
        }
        bindings.push((6, base + params_at as u64, PARAMS_BYTES as u32));
        let kernel = self.compute.kernel("vpp")?;
        let groups = [
            dst.width.div_ceil(4).div_ceil(kernel.workgroup[0]),
            dst.height.div_ceil(2).div_ceil(kernel.workgroup[1]),
            1,
        ];
        let token = self.compute.launch(&self.space, kernel, &bindings, &[], groups)?;
        self.compute.wait(self.drm.as_raw_fd(), token)?;

        let pairs: Vec<_> = (0..dst.pixels.planes())
            .map(|p| {
                let s = Surface { address: base + (out_at + out_off[p]) as u64, pitch: out_pitch[p], layout: Layout::Linear };
                (s, dst_planes[p], p)
            })
            .collect();
        self.copy_planes(dst.pixels, dst.width, dst.height, &pairs)
    }

    /// The planes of an input as the GPU reads them: a dma-buf's own, or CPU memory copied into
    /// the system-memory staging buffer.
    fn stage_input(&mut self, pic: &Picture) -> VResult<Vec<Surface>> {
        let Memory::Host { bytes, pitch, offsets } = &pic.memory else {
            return self.dma_planes(pic);
        };
        let (pitches, at, total) = Self::linear(pic.pixels, pic.width, pic.height);
        let host = self.host(total)?;
        let mut out = Vec::with_capacity(3);
        for p in 0..pic.pixels.planes() {
            let (line, rows) = pic.pixels.plane_extent(p, pic.width, pic.height);
            for row in 0..rows as usize {
                let from = offsets[p] + row * pitch;
                let data = bytes
                    .get(from..from + line as usize)
                    .ok_or_else(|| VideoError::Invalid("CPU picture smaller than its planes".into()))?;
                host.cpu.write_bytes(at[p] + row * pitches[p] as usize, data)?;
            }
            out.push(Surface { address: host.gpu.address() + at[p] as u64, pitch: pitches[p], layout: Layout::Linear });
        }
        crate::platform::device_full_barrier();
        Ok(out)
    }

    /// The kernel's parameter block (shaders/vpp.comp `Params`).
    fn params(&self, src: &Picture, dst: &Picture, op: &Operation, inputs: &[(u64, u32)], out_pitch: [u32; 3]) -> Vec<u32> {
        let kind = |p: Pixels| match p {
            Pixels::Yuv420 { .. } => 0u32,
            Pixels::Yuv444 { .. } => 1,
            Pixels::Rgb(_) => 2,
        };
        let shift = |p: Pixels| if p.bps() == 2 { 16 - p.bits() } else { 0 };
        let order = |p: Pixels| u32::from(p == Pixels::Rgb(RgbOrder::Rgba));
        let f = |v: f64| (v as f32).to_bits();
        let pitch = |i: usize| inputs.get(i).map_or(0, |v| v.1);
        let (s, d) = (op.src_rect, op.dst_rect);
        let (sx1, sy1) = (s.x + s.width as i32 - 1, s.y + s.height as i32 - 1);
        let chroma = if matches!(src.pixels, Pixels::Yuv420 { .. }) {
            [s.x / 2, s.y / 2, sx1 / 2, sy1 / 2]
        } else {
            [s.x, s.y, sx1, sy1]
        };
        let m = conversion(src.pixels, dst.pixels, op);
        let bg = background_codes(dst.pixels, op);
        let mut w = vec![
            kind(src.pixels), src.pixels.bps(), shift(src.pixels), order(src.pixels),
            kind(dst.pixels), dst.pixels.bps(), shift(dst.pixels), order(dst.pixels),
            pitch(0), pitch(1), pitch(2),
            out_pitch[0], out_pitch[1], out_pitch[2],
            dst.width, dst.height,
            s.x as u32, s.y as u32, sx1 as u32, sy1 as u32,
            chroma[0] as u32, chroma[1] as u32, chroma[2] as u32, chroma[3] as u32,
            f(f64::from(s.width) / f64::from(d.width)), f(f64::from(s.height) / f64::from(d.height)),
            f(f64::from(s.x)), f(f64::from(s.y)),
            f(f64::from(d.x)), f(f64::from(d.y)),
            f(f64::from(d.x) + f64::from(d.width)), f(f64::from(d.y) + f64::from(d.height)),
            op.src_colour.chroma_site[0].to_bits(), op.src_colour.chroma_site[1].to_bits(),
            op.dst_colour.chroma_site[0].to_bits(), op.dst_colour.chroma_site[1].to_bits(),
            f(f64::from((1u32 << dst.pixels.bits()) - 1)),
            bg[0], bg[1], bg[2], op.background >> 24,
        ];
        w.extend(m.iter().flatten().map(|v| f(*v)));
        w
    }

    /// Copies the `rect` (even origin and size for 4:2:0) of the dma-buf picture `src` into
    /// `dest`: plane `p` at `offsets[p]` with `pitch` bytes per row, 4:2:0 chroma at half height.
    pub fn download(&mut self, src: &Picture, rect: Rect, dest: &mut [u8], pitch: usize, offsets: [usize; 3]) -> VResult<()> {
        if rect.x < 0 || rect.y < 0 || rect.x as u64 + u64::from(rect.width) > u64::from(src.width)
            || rect.y as u64 + u64::from(rect.height) > u64::from(src.height)
            || matches!(src.pixels, Pixels::Yuv420 { .. }) && (rect.x % 2 != 0 || rect.y % 2 != 0 || rect.width % 2 != 0 || rect.height % 2 != 0)
        {
            return Err(VideoError::Invalid(format!("read-back region {rect:?} outside {}x{}", src.width, src.height)));
        }
        let planes = self.dma_planes(src)?;
        let (pitches, at, total) = Self::linear(src.pixels, src.width, src.height);
        let host = self.host(total)?.gpu.address();
        let pairs: Vec<_> = (0..src.pixels.planes())
            .map(|p| (planes[p], Surface { address: host + at[p] as u64, pitch: pitches[p], layout: Layout::Linear }, p))
            .collect();
        self.copy_planes(src.pixels, src.width, src.height, &pairs)?;
        let bytes = self.host.as_ref().unwrap().cpu.system_bytes()?;
        let bps = src.pixels.bps() as usize;
        for p in 0..src.pixels.planes() {
            let (x, y, w, h) = match (src.pixels, p) {
                (Pixels::Yuv420 { .. }, 1) => (rect.x as usize * bps, rect.y as usize / 2, rect.width as usize * bps, rect.height as usize / 2),
                _ => (rect.x as usize * bps, rect.y as usize, rect.width as usize * bps, rect.height as usize),
            };
            for row in 0..h {
                let from = at[p] + (y + row) * pitches[p] as usize + x;
                let to = offsets[p] + row * pitch;
                dest.get_mut(to..to + w)
                    .ok_or_else(|| VideoError::Invalid("read-back destination too small".into()))?
                    .copy_from_slice(&bytes[from..from + w]);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
