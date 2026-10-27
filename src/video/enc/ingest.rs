//! GPU ingest of external YUV dma-bufs into the encoder's NVENC input surface.
//!
//! NVENC reads 8/16-bit semi-planar input in block-linear tiles two GOBs (16 rows) high; this GPU's
//! GOB addressing is the one the copy engine writes, so a copy-engine rectangle copy converts any
//! linear or NVIDIA block-linear NV12/P010 dma-bufs into the layout (measured: byte-identical to
//! the CPU tiler `layout::in_tile_plane`). Planar 4:4:4 chroma (444P, Q416) is interleaved into
//! the CbCr plane by two remapped copies, one per component (measured byte-exact in
//! `tests::chroma_444_interleave`). No pixel crosses the CPU. The dma-bufs are imported into the
//! encoder's own address space once per key and stay mapped until [`Ingest::forget`].
use super::super::{
    compute::ComputeEngine,
    session::{vram, CopyEngine, VramBuffer},
    VResult, VideoError,
};
use crate::{
    gpu::{hi_lo, method},
    rm::Memory,
    gpu::GpuMapping,
};
use std::{
    collections::HashMap,
    os::fd::{AsFd, AsRawFd, BorrowedFd},
    rc::Rc,
};

/// Source surface layout. `BlockLinear` is NVIDIA's 2D block-linear with 64-byte wide blocks of
/// `8 << log2_height` rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Linear,
    BlockLinear { log2_height: u32 },
}

/// One plane of an external dma-buf. `fd` is borrowed for the duration of the call; planes with
/// the same fd share one import.
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub fd: i32,
    pub offset: u64,
    pub pitch: u32,
}

/// An external surface: NV12 (`bytes_per_sample` 1) or P010 (2) with a luma and an interleaved
/// CbCr plane; with `chroma444`, 444P (1) or Q416 (2) with luma, Cb and Cr planes; or one BGRA
/// plane (4). Unused planes are ignored.
#[derive(Clone, Copy, Debug)]
pub struct Source {
    pub planes: [Plane; 3],
    pub layout: Layout,
    pub bytes_per_sample: u32,
    pub chroma444: bool,
}

/// DRM_FORMAT_MOD_NVIDIA_BLOCK_LINEAR_2D(c, s, g, k, h) (drm_fourcc.h):
/// 0x10 | h[3:0] | k[19:12] | g[21:20] | s[22] | c[25:23].
/// Only uncompressed generation-2 (Turing and later) layouts are readable by the copy engine.
pub fn layout_from_modifier(modifier: u64) -> Option<Layout> {
    if modifier == 0 {
        return Some(Layout::Linear);
    }
    let v = modifier & ((1 << 56) - 1);
    if modifier >> 56 != 0x03 || v & 0x10 == 0 || v >> 26 != 0 {
        return None;
    }
    let (h, g, c) = ((v & 0xf) as u32, (v >> 20) & 3, (v >> 23) & 7);
    (c == 0 && g == 2 && h <= 5).then_some(Layout::BlockLinear { log2_height: h })
}

struct Import {
    memory: Rc<Memory>,
    gpu: GpuMapping,
}

pub(crate) struct Ingest {
    engine: CopyEngine,
    imports: HashMap<u64, Vec<Import>>,
    /// BGRA conversion: the compute engine (`None` inside: this GPU has no validated kernels)
    /// and the VRAM scratch for the linear BGRA copy and the linear NV12 result.
    compute: Option<Option<ComputeEngine>>,
    scratch: Option<(VramBuffer, usize)>,
}

/// Destination of a copy: both planes of one NVENC input slot (block linear, two GOBs high; the
/// chroma plane is `pitch` wide for 4:2:0 and `2 * pitch` wide for 4:4:4), or a pitch-linear
/// staging buffer.
pub(crate) enum Destination {
    Tiled { luma: u64, chroma: u64, pitch: u32, width: u32, height: u32 },
    Linear { address: u64, pitch: u32, width: u32, height: u32 },
}

enum Target {
    Tiled(u64, u32),
    Pitch(u64, u32),
}

impl Ingest {
    pub fn new(rm: &super::super::venc::NvVencDevice) -> VResult<Self> {
        Ok(Self {
            engine: CopyEngine::new(&rm.device, &rm.space)?,
            imports: HashMap::new(),
            compute: None,
            scratch: None,
        })
    }

    /// Converts a one-plane BGRA `source` (bytes B, G, R, A/X) into the tiled NV12 slot `dst` on
    /// the GPU: the copy engine brings the visible region into a linear VRAM scratch, the
    /// `bgra_nv12` kernel converts it there (bit-exact with the CPU conversion), and the copy
    /// engine tiles luma and CbCr into the slot. No pixel crosses the CPU. Returns `Ok(false)`,
    /// having done nothing, when this GPU's compute kernels are not validated.
    pub fn bgra_to_nv12(
        &mut self,
        rm: &super::super::venc::NvVencDevice,
        key: u64,
        source: &Source,
        dst: &Destination,
    ) -> VResult<bool> {
        let &Destination::Tiled { luma, chroma, pitch, width, height } = dst else {
            return Err(VideoError::Invalid("BGRA conversion needs a tiled NVENC slot".into()));
        };
        if self.compute.is_none() {
            self.compute = Some(match ComputeEngine::supported(&rm.device)? {
                Some(_) => Some(ComputeEngine::new(&rm.device, &rm.space)?),
                None => None,
            });
        }
        if self.compute.as_ref().is_some_and(|c| c.is_none()) {
            return Ok(false);
        }
        let (w, h) = (width as usize, height as usize);
        let even = (w + 1) & !1;
        let src_pitch = (w * 4).next_multiple_of(64);
        let luma_pitch = w.next_multiple_of(64);
        let chroma_pitch = even.next_multiple_of(64);
        let luma_at = (src_pitch * h).next_multiple_of(256);
        let chroma_at = (luma_at + luma_pitch * h).next_multiple_of(256);
        let bytes = chroma_at + chroma_pitch * h.div_ceil(2);
        if self.scratch.as_ref().is_none_or(|s| s.1 < bytes) {
            self.scratch = None;
            let size = bytes.next_multiple_of(1 << 16);
            self.scratch = Some((vram(&rm.device, &rm.space, size)?, size));
        }
        let base = self.scratch.as_ref().unwrap().0.gpu.address();
        let linear = Destination::Linear { address: base, pitch: src_pitch as u32, width, height };
        self.copy(rm, key, source, &linear)?;

        let compute = self.compute.as_mut().unwrap().as_mut().unwrap();
        let kernel = compute.kernel("bgra_nv12")?;
        let push: Vec<u8> = [width, height, src_pitch as u32, luma_pitch as u32, chroma_pitch as u32]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let bindings = [
            (0, base, (src_pitch * h) as u32),
            (1, base + luma_at as u64, (luma_pitch * h) as u32),
            (2, base + chroma_at as u64, (chroma_pitch * h.div_ceil(2)) as u32),
        ];
        let groups = [
            (w.div_ceil(4) as u32).div_ceil(kernel.workgroup[0]),
            (h.div_ceil(2) as u32).div_ceil(kernel.workgroup[1]),
            1,
        ];
        let token = compute.launch(&rm.space, kernel, &bindings, &push, groups)?;
        compute.wait(rm.drm.as_raw_fd(), token)?;

        let mut words = Vec::with_capacity(48);
        method(&mut words, 4, 0, &[self.engine.copy_class]); // SET_OBJECT
        for (at, line_pitch, line, rows, slot) in [
            (luma_at, luma_pitch, w, h, luma),
            (chroma_at, chroma_pitch, even, h.div_ceil(2), chroma),
        ] {
            let plane = Plane { fd: -1, offset: at as u64, pitch: line_pitch as u32 };
            Self::rectangle(
                &mut words,
                Layout::Linear,
                &plane,
                base + at as u64,
                Target::Tiled(slot, pitch),
                line as u32,
                rows as u32,
                bytes as u64,
            )?;
        }
        self.run(rm, &words)?;
        Ok(true)
    }

    pub fn forget(&mut self, key: u64) {
        self.imports.remove(&key);
    }

    fn import(
        &mut self,
        rm: &super::super::venc::NvVencDevice,
        key: u64,
        source: &Source,
        planes: usize,
    ) -> VResult<[usize; 3]> {
        let mut order: Vec<i32> = Vec::new();
        let mut index = [0usize; 3];
        for (p, plane) in source.planes[..planes].iter().enumerate() {
            if plane.fd < 0 {
                return Err(VideoError::Invalid(format!("plane {p} dma-buf fd {}", plane.fd)));
            }
            index[p] = match order.iter().position(|&fd| fd == plane.fd) {
                Some(i) => i,
                None => {
                    order.push(plane.fd);
                    order.len() - 1
                }
            };
        }
        if self.imports.get(&key).is_some_and(|v| v.len() == order.len()) {
            return Ok(index);
        }
        let mut imports = Vec::with_capacity(order.len());
        for (i, fd) in order.iter().enumerate() {
            // SAFETY: the caller lends a live fd for the duration of the call.
            let borrowed = unsafe { BorrowedFd::borrow_raw(*fd) };
            let memory = crate::external::import(&rm.device, rm.drm.as_fd(), borrowed)
                .map_err(|e| VideoError::Invalid(format!("dma-buf import object {i}: {e}")))?;
            let gpu = rm.space.map(&memory)?;
            imports.push(Import { memory, gpu });
        }
        self.imports.insert(key, imports);
        Ok(index)
    }

    /// Rejects a plane whose pitch or offset cannot hold `rows` rows of `line` bytes in `size`.
    pub(crate) fn check_plane(layout: Layout, plane: &Plane, line: u32, rows: u32, size: u64) -> VResult<()> {
        let need = match layout {
            Layout::Linear => u64::from(plane.pitch) * u64::from(rows - 1) + u64::from(line),
            Layout::BlockLinear { log2_height } => {
                u64::from(plane.pitch) * u64::from(rows.next_multiple_of(8 << log2_height))
            }
        };
        let valid = plane.pitch >= line
            && match layout {
                Layout::Linear => plane.pitch % 4 == 0,
                Layout::BlockLinear { .. } => plane.pitch % 64 == 0,
            };
        if !valid || plane.offset.checked_add(need).is_none_or(|end| end > size) {
            return Err(VideoError::Invalid(format!(
                "plane pitch {} offset {} cannot hold {rows} rows of {line} bytes in a {size}-byte dma-buf",
                plane.pitch, plane.offset
            )));
        }
        Ok(())
    }

    /// Appends the copy of one `rows` x `line`-byte rectangle of `plane` to `words`.
    fn rectangle(
        words: &mut Vec<u32>,
        layout: Layout,
        plane: &Plane,
        source: u64,
        target: Target,
        line: u32,
        rows: u32,
        size: u64,
    ) -> VResult<()> {
        Self::check_plane(layout, plane, line, rows, size)?;
        let mut launch = 2 | 1 << 2 | 1 << 9; // non-pipelined, flush, multi-line
        match layout {
            Layout::Linear => launch |= 1 << 7, // source pitch
            Layout::BlockLinear { log2_height } => {
                method(words, 4, 0x728, &[log2_height << 4 | 1 << 12]); // SET_SRC_BLOCK_SIZE
                let src_rows = rows.next_multiple_of(8 << log2_height);
                method(words, 4, 0x72c, &[plane.pitch, src_rows, 1, 0]);
                method(words, 4, 0x744, &[0, 0]); // SRC_ORIGIN_X/Y
            }
        }
        let (destination, dst_pitch) = match target {
            Target::Tiled(address, pitch) => {
                method(words, 4, 0x70c, &[1 << 4 | 1 << 12]); // SET_DST_BLOCK_SIZE: two GOBs high
                method(words, 4, 0x710, &[pitch, rows.next_multiple_of(16), 1, 0]);
                method(words, 4, 0x74c, &[0, 0]); // DST_ORIGIN_X/Y
                (address, pitch)
            }
            Target::Pitch(address, pitch) => {
                launch |= 1 << 8; // destination pitch
                (address, pitch)
            }
        };
        let [ih, il] = hi_lo(source);
        let [oh, ol] = hi_lo(destination);
        method(words, 4, 0x400, &[ih, il, oh, ol]);
        method(words, 4, 0x410, &[plane.pitch, dst_pitch, line, rows]);
        method(words, 4, 0x300, &[launch]);
        Ok(())
    }

    /// Appends the copy of one full-resolution chroma plane (`width` x `rows` samples of `bps`
    /// bytes) into one component of the interleaved CbCr plane of a tiled NVENC input slot:
    /// `component` 0 writes the even (Cb) samples, 1 the odd (Cr) ones. The other component is
    /// NO_WRITE, so the copy for the second plane leaves the first one in place
    /// (SET_REMAP_COMPONENTS, cl90b5.h; with remapping on, X dimensions count elements, pitches
    /// stay in bytes; a source element is one sample, a destination element one CbCr pair).
    #[allow(clippy::too_many_arguments)]
    fn chroma_component(
        words: &mut Vec<u32>,
        layout: Layout,
        plane: &Plane,
        source: u64,
        size: u64,
        destination: u64,
        dst_pitch: u32,
        width: u32,
        rows: u32,
        bps: u32,
        component: u32,
    ) -> VResult<()> {
        Self::check_plane(layout, plane, width * bps, rows, size)?;
        if dst_pitch < 2 * width * bps || dst_pitch % 64 != 0 {
            return Err(VideoError::Invalid(format!("interleaved chroma pitch {dst_pitch}")));
        }
        let mut launch = 2 | 1 << 2 | 1 << 9 | 1 << 10; // non-pipelined, flush, multi-line, remap
        match layout {
            Layout::Linear => launch |= 1 << 7, // source pitch
            Layout::BlockLinear { log2_height } => {
                method(words, 4, 0x728, &[log2_height << 4 | 1 << 12]); // SET_SRC_BLOCK_SIZE
                let src_rows = rows.next_multiple_of(8 << log2_height);
                method(words, 4, 0x72c, &[plane.pitch / bps, src_rows, 1, 0]);
                method(words, 4, 0x744, &[0, 0]); // SRC_ORIGIN_X/Y
            }
        }
        // DST_X/DST_Y: SRC_X (0) into this component, NO_WRITE (6) into the other; DST_Z/W
        // NO_WRITE; COMPONENT_SIZE bps; NUM_SRC_COMPONENTS ONE; NUM_DST_COMPONENTS TWO.
        let (x, y) = if component == 0 { (0, 6) } else { (6, 0) };
        method(words, 4, 0x708, &[x | y << 4 | 6 << 8 | 6 << 12 | (bps - 1) << 16 | 1 << 24]);
        method(words, 4, 0x70c, &[1 << 4 | 1 << 12]); // SET_DST_BLOCK_SIZE: two GOBs high
        method(words, 4, 0x710, &[dst_pitch / (2 * bps), rows.next_multiple_of(16), 1, 0]);
        method(words, 4, 0x74c, &[0, 0]); // DST_ORIGIN_X/Y
        let [ih, il] = hi_lo(source);
        let [oh, ol] = hi_lo(destination);
        method(words, 4, 0x400, &[ih, il, oh, ol]);
        method(words, 4, 0x410, &[plane.pitch, dst_pitch, width, rows]);
        method(words, 4, 0x300, &[launch]);
        Ok(())
    }

    /// Submits `words` on the copy engine and returns after it finished.
    fn run(&mut self, rm: &super::super::venc::NvVencDevice, words: &[u32]) -> VResult<()> {
        self.engine.run(&rm.device, &rm.space, rm.drm.as_raw_fd(), words)
    }

    /// Copies the visible region of `source` into `dst` and returns after the copy engine finished.
    /// YUV sources (`bytes_per_sample` 1 or 2) go to a tiled slot: 4:2:0 as two rectangle
    /// copies, 4:4:4 as the luma copy plus one remapped copy per chroma component. A one-plane
    /// BGRA source (`bytes_per_sample` 4) goes to a pitch-linear staging buffer.
    pub fn copy(
        &mut self,
        rm: &super::super::venc::NvVencDevice,
        key: u64,
        source: &Source,
        dst: &Destination,
    ) -> VResult<()> {
        let bps = source.bytes_per_sample;
        let planes = match dst {
            Destination::Linear { .. } => 1,
            Destination::Tiled { .. } if source.chroma444 => 3,
            Destination::Tiled { .. } => 2,
        };
        let index = self.import(rm, key, source, planes)?;
        let imports = &self.imports[&key];
        let mut words = Vec::with_capacity(96);
        method(&mut words, 4, 0, &[self.engine.copy_class]); // SET_OBJECT
        match *dst {
            Destination::Tiled { luma, chroma, pitch, width, height } if source.chroma444 => {
                let plane = &source.planes[0];
                let import = &imports[index[0]];
                Self::rectangle(
                    &mut words,
                    source.layout,
                    plane,
                    import.gpu.address() + plane.offset,
                    Target::Tiled(luma, pitch),
                    width * bps,
                    height,
                    import.memory.size as u64,
                )?;
                for p in 1..3usize {
                    let plane = &source.planes[p];
                    let import = &imports[index[p]];
                    Self::chroma_component(
                        &mut words,
                        source.layout,
                        plane,
                        import.gpu.address() + plane.offset,
                        import.memory.size as u64,
                        chroma,
                        2 * pitch,
                        width,
                        height,
                        bps,
                        p as u32 - 1,
                    )?;
                }
            }
            Destination::Tiled { luma, chroma, pitch, width, height } => {
                for p in 0..2usize {
                    let (rows, line) = if p == 0 {
                        (height, width * bps)
                    } else {
                        (height.div_ceil(2), width.div_ceil(2) * 2 * bps)
                    };
                    let plane = &source.planes[p];
                    let import = &imports[index[p]];
                    Self::rectangle(
                        &mut words,
                        source.layout,
                        plane,
                        import.gpu.address() + plane.offset,
                        Target::Tiled(if p == 0 { luma } else { chroma }, pitch),
                        line,
                        rows,
                        import.memory.size as u64,
                    )?;
                }
            }
            Destination::Linear { address, pitch, width, height } => {
                let plane = &source.planes[0];
                let import = &imports[index[0]];
                Self::rectangle(
                    &mut words,
                    source.layout,
                    plane,
                    import.gpu.address() + plane.offset,
                    Target::Pitch(address, pitch),
                    width * bps,
                    height,
                    import.memory.size as u64,
                )?;
            }
        }
        self.run(rm, &words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers() {
        let bl = |c: u64, s: u64, g: u64, k: u64, h: u64| {
            0x03 << 56 | 0x10 | h | k << 12 | g << 20 | s << 22 | c << 23
        };
        assert_eq!(layout_from_modifier(0), Some(Layout::Linear));
        assert_eq!(
            layout_from_modifier(0x0300_0000_0060_6010),
            Some(Layout::BlockLinear { log2_height: 0 })
        );
        assert_eq!(
            layout_from_modifier(bl(0, 1, 2, 6, 4)),
            Some(Layout::BlockLinear { log2_height: 4 })
        );
        assert_eq!(layout_from_modifier(bl(1, 1, 2, 6, 0)), None); // compressed
        assert_eq!(layout_from_modifier(bl(0, 0, 0, 0xfe, 0)), None); // pre-Turing GOBs
        assert_eq!(layout_from_modifier(bl(0, 1, 2, 6, 6)), None);
        assert_eq!(layout_from_modifier(0x0100_0000_0000_0001), None); // other vendor
        assert_eq!(layout_from_modifier(0x00ff_ffff_ffff_ffff), None); // invalid
    }

    /// Byte offset of (`x` bytes, `y` rows) in NVIDIA block linear: GOBs of 64 bytes x 8 rows,
    /// blocks one GOB wide and `8 << log2_height` rows high, `pitch` bytes per row (the GOB
    /// swizzle of `layout::in_offset`).
    fn bl_offset(x: usize, y: usize, pitch: usize, log2_height: u32) -> usize {
        let bh = 8usize << log2_height;
        let inside = x % 16 + (y % 4) * 16 + ((x / 16) % 2) * 64 + ((y / 4) % 2) * 128
            + ((x / 32) % 2) * 256;
        (y / bh) * pitch * bh + (x / 64) * 64 * bh + ((y % bh) / 8) * 512 + inside
    }

    fn put(buf: &mut crate::video::session::VramBuffer, bytes: &[u8]) {
        for (i, c) in bytes.chunks(4).enumerate() {
            let mut w = [0u8; 4];
            w[..c.len()].copy_from_slice(c);
            buf.cpu.write_u32(i * 4, u32::from_le_bytes(w)).unwrap();
        }
    }

    fn get(buf: &crate::video::session::VramBuffer, len: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(len + 4);
        for i in 0..len.div_ceil(4) {
            out.extend_from_slice(&buf.cpu.read_u32(i * 4).unwrap().to_le_bytes());
        }
        out.truncate(len);
        out
    }

    fn first_mismatch(got: &[u8], want: &[u8]) -> Option<(usize, usize, u8, u8)> {
        let n = got.iter().zip(want).filter(|(a, b)| a != b).count();
        let i = got.iter().zip(want).position(|(a, b)| a != b)?;
        Some((n, i, got[i], want[i]))
    }

    /// Stage-1a experiment: two remapped copy-engine passes (Cb into component X, Cr into
    /// component Y, the other component NO_WRITE) assemble the interleaved 4:4:4 CbCr plane of a
    /// tiled NVENC input slot from two planar chroma planes. Checks after each pass that written
    /// bytes equal the source and every other byte keeps its non-zero prefill, for 1- and 2-byte
    /// samples, linear and block-linear sources, sizes crossing GOB and row boundaries, padded
    /// pitches, and both passes in one submission.
    #[test]
    #[ignore = "local NVIDIA GPU: copy-engine NO_WRITE remap into a tiled NVENC input plane"]
    fn chroma_444_interleave() {
        use crate::video::{enc::layout::in_offset, session::vram, venc::NvVencDevice};
        let rm = NvVencDevice::open_on(None).unwrap();
        let mut ingest = Ingest::new(&rm).unwrap();
        let up = |v: usize, a: usize| v.div_ceil(a) * a;
        let mut cases = 0;
        for bps in [1usize, 2] {
            for (w, h) in [(64usize, 16usize), (70, 37), (200, 130), (1920, 1080)] {
                for layout in [
                    Layout::Linear,
                    Layout::BlockLinear { log2_height: 1 },
                    Layout::BlockLinear { log2_height: 4 },
                ] {
                    for extra in [0usize, 64] {
                        let (pitch, rows) = match layout {
                            Layout::Linear => (up(w * bps, 64) + 64 + extra, h),
                            Layout::BlockLinear { log2_height } => {
                                (up(w * bps, 64) + extra, up(h, 8 << log2_height))
                            }
                        };
                        let plane_bytes = up(pitch * rows, 4096);
                        let mut src = vram(&rm.device, &rm.space, 2 * plane_bytes).unwrap();
                        let sample = |p: usize, x: usize, y: usize, k: usize| -> u8 {
                            if p == 0 {
                                (x * 7 + y * 13 + k * 101 + 1) as u8
                            } else {
                                (x * 11 + y * 3 + k * 57 + 129) as u8
                            }
                        };
                        let mut image = vec![0u8; 2 * plane_bytes];
                        for p in 0..2 {
                            for y in 0..h {
                                for x in 0..w {
                                    for k in 0..bps {
                                        let o = match layout {
                                            Layout::Linear => y * pitch + x * bps + k,
                                            Layout::BlockLinear { log2_height } => {
                                                bl_offset(x * bps + k, y, pitch, log2_height)
                                            }
                                        };
                                        image[p * plane_bytes + o] = sample(p, x, y, k);
                                    }
                                }
                            }
                        }
                        put(&mut src, &image);
                        let dpitch = up(2 * w * bps, 64) + extra;
                        let dsize = dpitch * up(h, 16);
                        let mut dst = vram(&rm.device, &rm.space, dsize).unwrap();
                        let prefill: Vec<u8> = (0..dsize).map(|i| (i * 31 + 0x5a) as u8 | 1).collect();
                        let (src_va, dst_va) = (src.gpu.address(), dst.gpu.address());
                        let plane = |p: usize| Plane {
                            fd: -1,
                            offset: (p * plane_bytes) as u64,
                            pitch: pitch as u32,
                        };
                        let pass = |words: &mut Vec<u32>, p: usize| {
                            Ingest::chroma_component(
                                words,
                                layout,
                                &plane(p),
                                src_va + (p * plane_bytes) as u64,
                                (2 * plane_bytes) as u64,
                                dst_va,
                                dpitch as u32,
                                w as u32,
                                h as u32,
                                bps as u32,
                                p as u32,
                            )
                            .unwrap();
                        };
                        let mut want = prefill.clone();
                        let expect = |want: &mut Vec<u8>, p: usize| {
                            for y in 0..h {
                                for x in 0..w {
                                    for k in 0..bps {
                                        want[in_offset((2 * x + p) * bps + k, y, dpitch)] = sample(p, x, y, k);
                                    }
                                }
                            }
                        };
                        let tag = format!("bps {bps} {w}x{h} {layout:?} pitch {pitch} dpitch {dpitch}");
                        // Separate submissions, checked after each pass.
                        put(&mut dst, &prefill);
                        for p in 0..2 {
                            let mut words = Vec::new();
                            method(&mut words, 4, 0, &[ingest.engine.copy_class]);
                            pass(&mut words, p);
                            ingest.run(&rm, &words).unwrap();
                            expect(&mut want, p);
                            let got = get(&dst, dsize);
                            assert_eq!(first_mismatch(&got, &want), None, "{tag}: after pass {p}");
                        }
                        // Both passes in one push buffer.
                        put(&mut dst, &prefill);
                        let mut words = Vec::new();
                        method(&mut words, 4, 0, &[ingest.engine.copy_class]);
                        pass(&mut words, 0);
                        pass(&mut words, 1);
                        ingest.run(&rm, &words).unwrap();
                        let got = get(&dst, dsize);
                        assert_eq!(first_mismatch(&got, &want), None, "{tag}: one submission");
                        // The same plane from the CPU tiler the 4:4:4 CPU loaders use
                        // (`Encoder::load_x444`: interleaved CbCr, 2 * pitch).
                        let mut interleaved = vec![0u8; 2 * w * bps * h];
                        for y in 0..h {
                            for x in 0..w {
                                for (p, k) in (0..2).flat_map(|p| (0..bps).map(move |k| (p, k))) {
                                    interleaved[y * 2 * w * bps + (2 * x + p) * bps + k] = sample(p, x, y, k);
                                }
                            }
                        }
                        let mut cpu = prefill.clone();
                        crate::video::enc::layout::in_tile_plane(&mut cpu, &interleaved, 2 * w * bps, 2 * w * bps, h, dpitch);
                        assert_eq!(first_mismatch(&got, &cpu), None, "{tag}: CPU tiler");
                        cases += 1;
                    }
                }
            }
        }
        eprintln!("{cases} cases byte-exact");
    }

    /// Stage-3 acceptance: the GPU BGRA conversion (`bgra_to_nv12`) writes the NVENC input slot
    /// byte for byte as the CPU conversion of the encoder's fallback (`layout::in_tile_bgra`),
    /// padding included. Patterns: per-channel extremes, solid colours, a chroma checkerboard and
    /// a pseudo-random image whose X/A byte is garbage; sizes odd and even, crossing tile and
    /// GOB boundaries; linear sources with padded pitches and block-linear ones.
    #[test]
    #[ignore = "local NVIDIA GPU: compute BGRA -> NV12 into a tiled NVENC input slot"]
    fn bgra_to_nv12_matches_cpu() {
        use crate::video::{enc::layout::in_tile_bgra, session::vram, venc::NvVencDevice};
        let rm = NvVencDevice::open_on(None).unwrap();
        let mut ingest = Ingest::new(&rm).unwrap();
        let up = |v: usize, a: usize| v.div_ceil(a) * a;
        let patterns: [(&str, fn(usize, usize, usize) -> [u8; 4]); 5] = [
            ("extremes", |x, y, _| {
                let v = |b: usize| if (x >> b ^ y) & 1 == 0 { 0 } else { 255 };
                [v(0), v(1), v(2), 0x5a]
            }),
            ("white", |_, _, _| [255, 255, 255, 255]),
            ("red", |_, _, _| [0, 0, 255, 0]),
            ("chroma checker", |x, y, _| if (x ^ y) & 1 == 0 { [255, 0, 0, 0] } else { [0, 0, 255, 0] }),
            ("noise", |x, y, s| {
                let h = ((x * 0x9e37_79b1 ^ y * 0x85eb_ca77 ^ s * 0xc2b2_ae3d) as u32).wrapping_mul(0x27d4_eb2f);
                h.to_le_bytes()
            }),
        ];
        let mut cases = 0;
        for (w, h) in [(1usize, 1usize), (3, 5), (64, 16), (70, 37), (642, 362), (1920, 1080)] {
            for layout in [Layout::Linear, Layout::BlockLinear { log2_height: 1 }, Layout::BlockLinear { log2_height: 4 }] {
                for extra in [0usize, 128] {
                    let (src_pitch, src_rows) = match layout {
                        Layout::Linear => (up(w * 4, 64) + extra, h),
                        Layout::BlockLinear { log2_height } => (up(w * 4, 64) + extra, up(h, 8 << log2_height)),
                    };
                    let size = up(src_pitch * src_rows, 1 << 16);
                    let memory = rm.device.allocate_vram(size).unwrap();
                    let mut map = memory.map().unwrap();
                    let fd = crate::external::export(&memory, rm.drm.as_fd()).unwrap();
                    let pitch = up(w, 64);
                    let luma_sz = pitch * up(h, 32);
                    let slot_sz = luma_sz + pitch * up(h.div_ceil(2), 16);
                    for (name, pattern) in &patterns {
                        let mut image = vec![0u8; size];
                        let mut linear = vec![0u8; w * 4 * h];
                        for y in 0..h {
                            for x in 0..w {
                                let px = pattern(x, y, cases);
                                linear[(y * w + x) * 4..][..4].copy_from_slice(&px);
                                for (k, b) in px.iter().enumerate() {
                                    let o = match layout {
                                        Layout::Linear => y * src_pitch + x * 4 + k,
                                        Layout::BlockLinear { log2_height } => bl_offset(x * 4 + k, y, src_pitch, log2_height),
                                    };
                                    image[o] = *b;
                                }
                            }
                        }
                        for (i, c) in image.chunks(4).enumerate() {
                            map.write_u32(i * 4, u32::from_le_bytes(c.try_into().unwrap())).unwrap();
                        }
                        crate::platform::device_full_barrier();
                        let plane = Plane { fd: fd.as_raw_fd(), offset: 0, pitch: src_pitch as u32 };
                        let source = Source { planes: [plane; 3], layout, bytes_per_sample: 4, chroma444: false };
                        let slot = vram(&rm.device, &rm.space, slot_sz).unwrap();
                        let dst = Destination::Tiled {
                            luma: slot.gpu.address(),
                            chroma: slot.gpu.address() + luma_sz as u64,
                            pitch: pitch as u32,
                            width: w as u32,
                            height: h as u32,
                        };
                        assert!(ingest.bgra_to_nv12(&rm, cases as u64 + 1, &source, &dst).unwrap(), "compute not available");
                        ingest.forget(cases as u64 + 1);
                        let got = get(&slot, slot_sz);
                        let mut want = vec![0u8; slot_sz];
                        let (l, c) = want.split_at_mut(luma_sz);
                        let mut scratch=vec![0; ((w+1)&!1).div_ceil(64)*64*48];
                        in_tile_bgra(l, c, &linear, w * 4, w, h, pitch, &mut scratch);
                        let tag = format!("{name} {w}x{h} {layout:?} pitch {src_pitch}");
                        assert_eq!(first_mismatch(&got, &want), None, "{tag}");
                        cases += 1;
                    }
                }
            }
        }
        eprintln!("{cases} BGRA cases byte-exact with the CPU conversion");
    }
}
