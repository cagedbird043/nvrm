//! GPU ingest of external 4:2:0 dma-bufs into the encoder's NVENC input surface.
//!
//! NVENC reads 8/16-bit semi-planar input in block-linear tiles two GOBs (16 rows) high; this GPU's
//! GOB addressing is the one the copy engine writes, so a copy-engine rectangle copy converts any
//! linear or NVIDIA block-linear NV12/P010 dma-bufs into the layout (measured: byte-identical to
//! the CPU tiler `layout::in_tile_plane`). No pixel crosses the CPU. The dma-bufs are imported
//! into the encoder's own address space once per key and stay mapped until [`Ingest::forget`].
use super::super::{
    channel::wait_sync_file,
    session::{vram, CopyEngine},
    VResult, VideoError,
};
use crate::{
    gpu::{hi_lo, method, Push, SubmitStatus},
    rm::Memory,
    gpu::GpuMapping,
};
use std::{
    collections::HashMap,
    os::fd::{AsFd, AsRawFd, BorrowedFd},
    rc::Rc,
    time::Duration,
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

/// An external NV12 (`bytes_per_sample` 1) or P010 (2) surface: luma plane, interleaved CbCr plane.
#[derive(Clone, Copy, Debug)]
pub struct Source {
    pub planes: [Plane; 2],
    pub layout: Layout,
    pub bytes_per_sample: u32,
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
}

/// Destination of a copy: both planes of one NVENC input slot (block linear, two GOBs high), or a
/// pitch-linear staging buffer.
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
        })
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
    ) -> VResult<[usize; 2]> {
        let mut order: Vec<i32> = Vec::new();
        let mut index = [0usize; 2];
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

    /// Copies the visible region of `source` into `dst` and returns after the copy engine finished.
    /// 4:2:0 sources (two planes, `bytes_per_sample` 1 or 2) go to a tiled slot; a one-plane BGRA
    /// source (`bytes_per_sample` 4) goes to a pitch-linear staging buffer.
    pub fn copy(
        &mut self,
        rm: &super::super::venc::NvVencDevice,
        key: u64,
        source: &Source,
        dst: &Destination,
    ) -> VResult<()> {
        let bps = source.bytes_per_sample;
        let planes = if matches!(dst, Destination::Linear { .. }) { 1 } else { 2 };
        let index = self.import(rm, key, source, planes)?;
        let imports = &self.imports[&key];
        let mut words = Vec::with_capacity(64);
        method(&mut words, 4, 0, &[self.engine.copy_class]); // SET_OBJECT
        match *dst {
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
        let ci = match self.engine.commands.iter().position(|(_, token)| {
            *token == 0 || self.engine.queue.shared().retired(*token).unwrap_or(false)
        }) {
            Some(i) => i,
            None => {
                self.engine.commands.push((vram(&rm.device, &rm.space, 4096)?, 0));
                self.engine.commands.len() - 1
            }
        };
        let cmd = &mut self.engine.commands[ci].0;
        for (i, word) in words.iter().enumerate() {
            cmd.cpu.write_u32(i * 4, *word)?;
        }
        crate::platform::device_full_barrier();
        let push = Push {
            address: cmd.gpu.address(),
            size: (words.len() * 4) as u32,
            flags: 0,
        };
        let token = match self
            .engine
            .queue
            .submit(&[push], &rm.space, &[], &[], &[], |_| Ok(()))?
        {
            SubmitStatus::Submitted(t) => t,
            SubmitStatus::Full(_) => return Err(VideoError::Again("copy queue full".into())),
        };
        self.engine.commands[ci].1 = token;
        if !self.engine.queue.shared().retired(token)? {
            let fence = self
                .engine
                .queue
                .context_fence(rm.drm.as_raw_fd(), token)?;
            wait_sync_file(fence.as_raw_fd(), Duration::from_secs(5))
                .map_err(VideoError::DeviceLost)?;
        }
        Ok(())
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
}
