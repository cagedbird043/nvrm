//! NVENC session ported from mickys/nvenc enc.c/enc.h (MIT, 90d1d6a).
//! Picture setups preserve the C core's bytes; RM ownership and completion are native Rust.
use super::{
    av1hdr, av1probs, h264hdr, hevchdr, layout, nvenc_drv::*, tmpl_av1::*, tmpl_h264::*,
    tmpl_hevc::*,
};
use crate::video::{
    channel::{EngineChannel, NV2080_ENGINE_TYPE_NVENC0},
    venc::{NvVencDevice, NvVencMemory},
    VResult, VideoError,
};
use std::{
    mem::{align_of, size_of},
    time::Instant,
};

pub const ENC_H264: i32 = 0;
pub const ENC_HEVC: i32 = 1;
pub const ENC_AV1: i32 = 2;
pub const ENC_MAX_REFS: usize = 4;
pub const ENC_MAX_SLICES: usize = 16;
pub const ENC_MAX_DEPTH: usize = 3;
const OFF_ME: usize = 0x400;
const OFF_MD: usize = 0x500;
const OFF_Q: usize = 0x600;
const OFF_SLICE: usize = 0x700;
const OFF_AV1_TILE: usize = 0x800;
const SETUP_SIZE: usize = 0x1000;

/// Fields and zero defaults match EncConfig in mickys/enc.h.
#[derive(Clone, Debug, Default)]
pub struct EncConfig {
    pub codec: i32,
    pub width: i32,
    pub height: i32,
    pub qp_i: i32,
    pub qp_p: i32,
    /// Optional native quantizer bounds (H.264/HEVC 0..51, AV1 0..255).
    /// None preserves the mickys setup bytes, including HEVC's signed ten-bit minimum.
    pub min_qp: Option<u8>,
    pub max_qp: Option<u8>,
    pub num_refs: i32,
    pub dpb_extra: i32,
    pub engine: i32,
    pub idr_period: i32,
    pub rc_mode: i32,
    pub bitrate: i32,
    pub max_bitrate: i32,
    pub vbv_bits: i32,
    pub fps: i32,
    pub slices: i32,
    pub spin: bool,
    pub depth: i32,
    pub h264_profile: i32,
    pub h264_level: i32,
    pub cabac: bool,
    pub bit_depth: i32,
    pub chroma444: bool,
    pub lossless: bool,
    pub quality: bool,
    pub no_cu8: bool,
    pub bframes: i32,
    pub aq: i32,
    pub two_pass: bool,
    pub full_range: bool,
    pub colour: [i32; 3],
    pub has_mdcv: bool,
    pub has_cll: bool,
    pub mdcv: [u8; 24],
    pub cll: [u8; 4],
    pub verbose: bool,
}
/// EncFrame metadata. Bitstream bytes are appended to the caller's reusable Vec.
#[derive(Clone, Debug, Default)]
pub struct EncFrame {
    pub size: usize,
    pub idr: bool,
    pub frame_num: u32,
    pub poc: u32,
    pub bits: u32,
    pub ucode_error: u32,
    pub avg_qp: u32,
    pub intra_mbs: u32,
    pub inter_mbs: u32,
    pub intra_cost: u32,
    pub inter_cost: u32,
    pub ms: f64,
    pub hidden: bool,
}

#[derive(Default)]
struct Mem {
    gpu: u64,
    cpu: *mut u8,
    size: usize,
}
impl Mem {
    fn alloc(rm: &mut NvVencDevice, size: usize, align: u64, cpu: bool) -> VResult<Self> {
        let NvVencMemory { gpu, cpu, size, .. } = rm.alloc(size as u64, align, cpu, cpu)?;
        Ok(Self {
            gpu,
            cpu: cpu.cast(),
            size: size as usize,
        })
    }
    // CPU views never escape the encoder; callers fence the slot before CPU access.
    fn bytes(&self) -> &[u8] {
        assert!(!self.cpu.is_null());
        // SAFETY: mapped RM allocation, kept alive by Encoder::rm.
        unsafe { std::slice::from_raw_parts(self.cpu, self.size) }
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        assert!(!self.cpu.is_null());
        // SAFETY: exclusive access to this fenced RM allocation.
        unsafe { std::slice::from_raw_parts_mut(self.cpu, self.size) }
    }
    fn view<T: Copy>(&self, off: usize) -> T {
        assert!(off + size_of::<T>() <= self.size && !self.cpu.is_null());
        // SAFETY: only integer-only nvenc_drv records, read after completion.
        unsafe { self.cpu.add(off).cast::<T>().read_unaligned() }
    }
    fn view_mut<T>(&mut self, off: usize) -> &mut T {
        assert!(off + size_of::<T>() <= self.size && !self.cpu.is_null());
        assert!((self.cpu as usize + off).is_multiple_of(align_of::<T>()));
        // SAFETY: only integer-only nvenc_drv records; zeroed setup memory is valid.
        unsafe { &mut *self.cpu.add(off).cast::<T>() }
    }
    fn copy(&mut self, off: usize, data: &[u8]) {
        self.bytes_mut()[off..off + data.len()].copy_from_slice(data);
    }
}
#[derive(Clone)]
struct Pending {
    idr: bool,
    frame_num: u32,
    poc: u32,
    t0: Instant,
    slot: usize,
    nrefs: usize,
    refs: [usize; 4],
    ref_oh: [i32; 4],
    b: bool,
    hidden: bool,
    timeline: u64,
}
impl Default for Pending {
    fn default() -> Self {
        Self {
            idr: false,
            frame_num: 0,
            poc: 0,
            t0: Instant::now(),
            slot: 0,
            nrefs: 0,
            refs: [0; 4],
            ref_oh: [0; 4],
            b: false,
            hidden: false,
            timeline: 0,
        }
    }
}

pub struct Encoder {
    // Drop the channel before allocations/device. Drop waits for queued work first.
    chan: EngineChannel,
    pub cfg: EncConfig,
    rm: NvVencDevice,
    wmb: usize,
    hmb: usize,
    pub pitch: usize,
    pub luma_sz: usize,
    pub chroma_sz: usize,
    r_pitch: usize,
    r_luma: usize,
    r_slot: usize,
    pub bps: usize,
    input: [Mem; 3],
    setup: [Mem; 3],
    bs: [Mem; 3],
    status: [Mem; 3],
    pend: [Pending; 3],
    recon: Mem,
    hist: Mem,
    coloc: Mem,
    coloc_b: Mem,
    mepred: [Mem; 2],
    rcproc: Mem,
    rcdata: Mem,
    sem: Mem,
    setup1: [Mem; 3],
    bs1: Mem,
    status1: Mem,
    hist1: Mem,
    rec1: Mem,
    coloc1: Mem,
    mepred1: Mem,
    av1_prob_in: Mem,
    av1_prob: Mem,
    av1_line: [Mem; 6],
    seq: u32,
    waited: u32,
    force_idr: bool,
    /// External input GPU VA in this encoder's address space (NVENC input layout).
    pub ext_input: u64,
    frame_num: u32,
    since_idr: u32,
    slot_of: [usize; 4],
    nrefs: usize,
    next_slot: usize,
    b_slot: usize,
    slot_poc: [u32; 6],
    idr_disp: i32,
    nslots: usize,
    av1_hid: Vec<u8>,
    av1_hid_slot: usize,
    av1_hid_disp: i32,
    av1_last_shown: i32,
    aq_stride: usize,
    aq_valid: [bool; 3],
    av1_hq: bool,
    sb_cols: usize,
    sb_rows: usize,
    tile_cols_log2: i32,
    tile_rows_log2: i32,
    tile_w: usize,
    tile_h: usize,
    tile_cols: usize,
    tile_rows: usize,
}
fn up(v: usize, a: usize) -> usize {
    (v + a - 1) & !(a - 1)
}
fn invalid(s: impl Into<String>) -> VideoError {
    VideoError::Invalid(s.into())
}
fn put32(b: &mut [u8], off: usize, v: u32) {
    b[off..off + 4].copy_from_slice(&v.to_le_bytes());
}
fn get32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}
fn put16(b: &mut [u8], off: usize, v: u16) {
    b[off..off + 2].copy_from_slice(&v.to_le_bytes());
}
fn mthd(w: &mut Vec<u32>, m: u32, v: u32) {
    crate::gpu::method(w, 0, m, &[v]);
}
fn addr(w: &mut Vec<u32>, m: u32, a: u64) {
    mthd(w, m, (a >> 8) as u32);
}
impl Encoder {
    pub fn open(cfg: &EncConfig) -> VResult<Self> {
        Self::open_on(cfg, None)
    }
    pub fn open_on(cfg: &EncConfig, pci: Option<[u32; 4]>) -> VResult<Self> {
        // The host C core's rm_shim.c chan_create supports NVENC0 only.
        if cfg.engine != 0 {
            return Err(VideoError::Unsupported("only NVENC0 is supported".into()));
        }
        if cfg.width <= 0 || cfg.height <= 0 {
            return Err(invalid("non-positive encode dimensions"));
        }
        let mut c = cfg.clone();
        c.num_refs = c.num_refs.clamp(1, if c.codec == ENC_AV1 { 4 } else { 3 });
        if c.codec == ENC_H264 && (c.chroma444 || c.lossless) {
            c.cabac = false;
        }
        if c.codec == ENC_H264 && c.h264_profile == 66 {
            c.cabac = false;
            c.bframes = 0;
        }
        if c.lossless {
            c.qp_i = 0;
            c.qp_p = 0;
            c.rc_mode = 0;
        }
        let mut rm = NvVencDevice::open_on(pci)?;
        let class = rm
            .classes
            .iter()
            .copied()
            .filter(|c| c & 255 == 0xb7 && *c >= 0xb000 && *c < 0x10000)
            .max()
            .unwrap_or(0);
        let hevc = c.codec == ENC_HEVC;
        let av1 = c.codec == ENC_AV1;
        if av1 && class != 0xc9b7 {
            return Err(VideoError::Unsupported(format!(
                "this GPU's NVENC (class {class:#x}) has no AV1"
            )));
        }
        if (!hevc && !av1) || c.bit_depth != 10 {
            c.bit_depth = 8;
        }
        if hevc && class == 0xb4b7 {
            c.bframes = 0;
        }
        let bps = if c.bit_depth > 8 { 2 } else { 1 };
        let (w, h) = (c.width as usize, c.height as usize);
        let (wmb, hmb) = (w.div_ceil(16), h.div_ceil(16));
        let rows = if av1 {
            up(h, 64)
        } else if hevc {
            up(hmb * 16, 32)
        } else {
            hmb * 16
        };
        let pitch = up(wmb * 16, 64);
        let luma_sz = up(layout::in_plane_size(pitch * bps, rows), 256);
        let chroma_sz = if c.chroma444 && !av1 {
            2 * luma_sz
        } else {
            layout::in_plane_size(pitch * bps, rows / 2)
        };
        let r_pitch = up(wmb * 16, 256);
        let r_rows = if av1 { up(h, 64) } else { up(hmb * 16, 32) };
        let r_luma = up(r_pitch * bps * r_rows, 256);
        let mut r_slot = up(
            r_luma + r_pitch * bps * if av1 { r_rows / 2 } else { up(hmb * 8, 32) },
            65536,
        );
        if c.chroma444 && !av1 {
            r_slot = up(3 * r_luma, 65536);
        }
        if hevc || av1 {
            r_slot += 65536;
        }
        let mut nslots = c.num_refs as usize + 1;
        let mut b_slot = 0;
        if c.bframes > 0 {
            c.num_refs = c.num_refs.max(2);
            nslots = c.num_refs as usize + 2;
            b_slot = nslots - 1;
        }
        let aq_stride = up(h.div_ceil(32) * 256, 4096);
        let mbs = wmb * hmb;
        let mut hist_sz = up((w.div_ceil(16) + 16) * H264_HIST_BLOCK_SIZE as usize, 65536);
        if hevc {
            hist_sz = up(w.div_ceil(32) * 768, 65536);
        }
        if hevc && class == 0xb4b7 {
            hist_sz = up(
                768 * w.div_ceil(32) * h.div_ceil(32) + 384 * w.div_ceil(32),
                65536,
            );
        }
        let (sb_cols, sb_rows) = (w.div_ceil(64), h.div_ceil(64));
        let (mut cl, mut rl) = (0, 0);
        if av1 {
            while sb_cols.div_ceil(1 << cl) > 64 {
                cl += 1;
            }
            while sb_cols.div_ceil(1 << cl) * sb_rows.div_ceil(1 << rl) > 2304 {
                rl += 1;
            }
            hist_sz = up(
                AV1_HIST_PIXEL_FRAME_SIZE as usize
                    + sb_cols * sb_rows * AV1_HIST_MV_SB_SIZE as usize
                    + (1 << 20),
                65536,
            );
        }
        let (tile_w, tile_h) = (sb_cols.div_ceil(1 << cl), sb_rows.div_ceil(1 << rl));
        c.depth = c.depth.clamp(1, 3);
        if c.fps == 0 {
            c.fps = 60;
        }
        if c.vbv_bits == 0 {
            c.vbv_bits = c.bitrate;
        }
        if c.max_bitrate == 0 {
            c.max_bitrate = c.bitrate;
        }
        if av1 || c.rc_mode == 0 {
            c.two_pass = false;
        }
        let mut input = std::array::from_fn(|_| Mem::default());
        let mut setup = std::array::from_fn(|_| Mem::default());
        let mut bs = std::array::from_fn(|_| Mem::default());
        let mut status = std::array::from_fn(|_| Mem::default());
        let mut setup1 = std::array::from_fn(|_| Mem::default());
        for d in 0..c.depth as usize {
            input[d] = Mem::alloc(&mut rm, luma_sz + chroma_sz + (64 << 10), 4096, true)?;
            input[d].bytes_mut().fill(0);
            setup[d] = Mem::alloc(&mut rm, SETUP_SIZE, 4096, true)?;
            bs[d] = Mem::alloc(&mut rm, up(mbs * 384 + (1 << 20), 65536), 4096, true)?;
            status[d] = Mem::alloc(&mut rm, if av1 { 0x40000 } else { 0x10000 }, 4096, true)?;
            if c.two_pass {
                setup1[d] = Mem::alloc(&mut rm, SETUP_SIZE, 4096, true)?;
            }
        }
        let recon = Mem::alloc(&mut rm, r_slot * nslots, 65536, false)?;
        let hist = Mem::alloc(&mut rm, hist_sz, 4096, false)?;
        let coloc = Mem::alloc(&mut rm, up(mbs * 64, 65536), 4096, false)?;
        let coloc_b = Mem::alloc(&mut rm, up(mbs * 64, 65536), 4096, false)?;
        let me_sz = up(mbs * 64, 65536);
        let mepred = [
            Mem::alloc(&mut rm, me_sz, 4096, false)?,
            Mem::alloc(&mut rm, me_sz, 4096, false)?,
        ];
        let mut rcproc = Mem::alloc(&mut rm, 65536, 4096, true)?;
        let mut rcdata = Mem::alloc(
            &mut rm,
            if av1 {
                0x100 + 3 * up(mbs * 64, 4096) + 4096
            } else {
                65536 + 3 * aq_stride
            },
            4096,
            true,
        )?;
        rcproc.bytes_mut().fill(0);
        rcdata.bytes_mut().fill(0);
        let (bs1, status1, hist1, rec1, coloc1, mepred1) = if c.two_pass {
            (
                Mem::alloc(&mut rm, bs[0].size, 4096, false)?,
                Mem::alloc(&mut rm, ((mbs + 3) * 256 + 0x1ffff) & !0xffff, 4096, false)?,
                Mem::alloc(&mut rm, hist_sz, 4096, false)?,
                Mem::alloc(&mut rm, r_slot, 65536, false)?,
                Mem::alloc(&mut rm, up(mbs * 64, 65536), 4096, false)?,
                Mem::alloc(&mut rm, me_sz, 4096, false)?,
            )
        } else {
            (
                Mem::default(),
                Mem::default(),
                Mem::default(),
                Mem::default(),
                Mem::default(),
                Mem::default(),
            )
        };
        let mut av1_prob_in = Mem::default();
        let mut av1_prob = Mem::default();
        let mut av1_line = std::array::from_fn(|_| Mem::default());
        let mut sem = Mem::alloc(&mut rm, 4096, 4096, true)?;
        sem.bytes_mut().fill(0);
        if av1 {
            av1_prob_in = Mem::alloc(&mut rm, 4 * 0x3000, 4096, true)?;
            av1_prob = Mem::alloc(&mut rm, (nslots + 1) * 65536, 4096, true)?;
            av1_prob_in.bytes_mut().fill(0);
            av1_prob.bytes_mut().fill(0);
            for q in 0..4 {
                av1probs::default_probs(q, &mut av1_prob_in.bytes_mut()[q * 0x3000..]);
            }
            for line in &mut av1_line {
                *line = Mem::alloc(&mut rm, 1 << 20, 4096, true)?;
                line.bytes_mut().fill(0);
            }
            rcproc.copy(0, &AV1_TMPL_RCPROCESS);
        } else if c.rc_mode != 0 {
            let ps = rcproc.view_mut::<nvenc_persistent_state_s>(0);
            ps.Wpbi = [16, 6, 38];
            ps.Whist = [[16; 3], [6; 3], [38; 3]];
            let np = Self::rc_np_cfg(&c);
            if c.idr_period > 0 {
                ps.np = np << 4;
                ps.nb = (c.idr_period - 1 - np) << 4;
            }
            ps.nal_cpb_fullness = (0.5 * c.vbv_bits as f64 * 16.0) as i32;
            ps.vcl_cpb_fullness = ps.nal_cpb_fullness;
        }
        let chan = EngineChannel::new(&rm.device, &rm.space, NV2080_ENGINE_TYPE_NVENC0, class, 0)?;
        let av1_hq = av1 && (cfg.quality || cfg.bframes > 0);
        Ok(Self {
            chan,
            cfg: c,
            rm,
            wmb,
            hmb,
            pitch,
            luma_sz,
            chroma_sz,
            r_pitch,
            r_luma,
            r_slot,
            bps,
            input,
            setup,
            bs,
            status,
            pend: std::array::from_fn(|_| Pending::default()),
            recon,
            hist,
            coloc,
            coloc_b,
            mepred,
            rcproc,
            rcdata,
            sem,
            setup1,
            bs1,
            status1,
            hist1,
            rec1,
            coloc1,
            mepred1,
            av1_prob_in,
            av1_prob,
            av1_line,
            seq: 0,
            waited: 0,
            force_idr: false,
            ext_input: 0,
            frame_num: 0,
            since_idr: 0,
            slot_of: [0; 4],
            nrefs: 0,
            next_slot: 0,
            b_slot,
            slot_poc: [0; 6],
            idr_disp: 0,
            nslots,
            av1_hid: if av1 {
                Vec::with_capacity(bs_capacity(mbs))
            } else {
                Vec::new()
            },
            av1_hid_slot: 0,
            av1_hid_disp: -1,
            av1_last_shown: -1,
            aq_stride,
            aq_valid: [false; 3],
            av1_hq,
            sb_cols,
            sb_rows,
            tile_cols_log2: cl as i32,
            tile_rows_log2: rl as i32,
            tile_w,
            tile_h,
            tile_cols: sb_cols.div_ceil(tile_w),
            tile_rows: sb_rows.div_ceil(tile_h),
        })
    }
    fn rc_np_cfg(c: &EncConfig) -> i32 {
        let bf = c.bframes.max(0);
        if c.idr_period > 0 {
            (c.idr_period + bf - 1) / (bf + 1)
        } else {
            i32::MAX
        }
    }
    fn magic(&self) -> u32 {
        (self.chan.class << 16) | 6
    }
    fn rc_area(&self) -> usize {
        up(self.wmb * self.hmb * 64, 4096)
    }
    pub fn inflight(&self) -> usize {
        self.seq.wrapping_sub(self.waited) as usize
    }
    pub fn close(self) {
        drop(self);
    }
    fn h264_params(&self) -> h264hdr::H264Params {
        let c = &self.cfg;
        let cbp = !c.chroma444 && !c.lossless && c.h264_profile == 66;
        h264hdr::H264Params {
            width: c.width,
            height: c.height,
            colour: c.colour,
            profile_idc: if c.chroma444 || c.lossless {
                244
            } else if c.h264_profile == 100 {
                100
            } else if cbp {
                66
            } else {
                77
            },
            constraint_flags: if cbp { 0xc0 } else { 0 },
            level_idc: if c.h264_level != 0 {
                c.h264_level
            } else if self.wmb * self.hmb > 36864 {
                60
            } else {
                51
            },
            chroma444: c.chroma444,
            lossless: c.lossless,
            log2_max_frame_num_minus4: 4,
            poc_type: if c.bframes > 0 { 0 } else { 2 },
            log2_max_poc_lsb_minus4: 4,
            num_reorder: i32::from(c.bframes > 0),
            num_ref_frames: c.num_refs,
            num_ref_idx_l0_minus1: c.num_refs - 1,
            entropy_cabac: c.cabac,
            transform_8x8: c.h264_profile == 100 || c.chroma444,
            pic_init_qp: 26,
            vui: true,
            full_range: c.full_range,
        }
    }
    fn hevc_params(&self) -> hevchdr::HevcParams {
        let c = &self.cfg;
        hevchdr::HevcParams {
            width: c.width,
            height: c.height,
            log2_max_poc_lsb_minus4: 4,
            max_dec_pic_buffering: c.num_refs + 1 + c.dpb_extra,
            sps_rps_negative: c.num_refs,
            num_reorder: i32::from(c.bframes > 0),
            num_ref_idx_l0_default: c.num_refs,
            pic_init_qp: 26,
            vui: true,
            full_range: c.full_range,
            colour: c.colour,
            min_cb8: c.quality && !c.no_cu8,
            bit_depth: c.bit_depth,
            chroma444: c.chroma444,
            lossless: c.lossless,
            no_sdh: true,
            no_ts: c.bit_depth > 8,
            ..Default::default()
        }
    }
    fn av1_seq(&self) -> av1hdr::Av1SeqParams {
        av1hdr::Av1SeqParams {
            width: self.cfg.width,
            height: self.cfg.height,
            bit_depth: self.cfg.bit_depth,
            full_range: self.cfg.full_range,
            colour: self.cfg.colour,
            enable_cdef: true,
            enable_restoration: true,
            ..Default::default()
        }
    }
    pub fn headers(&self, out: &mut Vec<u8>) {
        match self.cfg.codec {
            ENC_AV1 => {}
            ENC_HEVC => {
                let p = self.hevc_params();
                hevchdr::write_vps(&p, out);
                hevchdr::write_sps(&p, out);
                hevchdr::write_pps(&p, out);
            }
            _ => {
                let p = self.h264_params();
                h264hdr::write_sps(&p, out);
                h264hdr::write_pps(&p, out);
            }
        }
    }
    pub fn av1_config(&self, out: &mut Vec<u8>) {
        av1hdr::write_sequence_header(&self.av1_seq(), out);
    }
    fn input_slot(&self) -> VResult<usize> {
        if self.inflight() >= self.cfg.depth as usize {
            Err(invalid("all encoder input slots are in flight"))
        } else {
            Ok(self.seq as usize % self.cfg.depth as usize)
        }
    }
    fn aq_compute(&mut self, y: &[u8], stride: usize, bytes: usize) {
        let slot = self.seq as usize % self.cfg.depth as usize;
        self.aq_valid[slot] = false;
        if self.cfg.aq == 0 || self.cfg.codec != ENC_HEVC {
            return;
        }
        let (w, h, k) = (
            self.cfg.width as usize,
            self.cfg.height as usize,
            self.cfg.aq as f64,
        );
        let a = &mut self.rcdata.bytes_mut()[65536 + slot * self.aq_stride..];
        for by in 0..h.div_ceil(32) {
            for bx in 0..w.div_ceil(32) {
                let (x0, y0) = (bx * 32, by * 32);
                let (bw, bh) = ((w - x0).min(32), (h - y0).min(32));
                let (mut sum, mut ss) = (0u64, 0u64);
                for r in 0..bh {
                    for col in 0..bw {
                        let off = (y0 + r) * stride + (x0 + col) * bytes;
                        let v = if bytes == 1 { y[off] } else { y[off + 1] } as u64;
                        sum += v;
                        ss += v * v;
                    }
                }
                let n = (bw * bh) as f64;
                let var = (ss as f64 - sum as f64 * sum as f64 / n) / n;
                let o = (var + 1.0).log2() - 8.6;
                a[by * 256 + bx] = (if o > k {
                    k
                } else if o < -k {
                    -k
                } else if o < 0.0 {
                    o - 0.5
                } else {
                    o + 0.5
                } as i8) as u8;
            }
        }
        self.aq_valid[slot] = true;
    }
    pub fn load_nv12(&mut self, y: &[u8], ys: usize, uv: &[u8], uvs: usize) -> VResult<()> {
        let ds = self.input_slot()?;
        self.aq_compute(y, ys, 1);
        let (w, h) = (self.cfg.width as usize, self.cfg.height as usize);
        let (luma, chroma) = self.input[ds].bytes_mut().split_at_mut(self.luma_sz);
        if self.bps == 2 {
            tile_widened(luma, y, ys, w, h, self.pitch * 2);
            tile_widened(chroma, uv, uvs, up(w, 2), h.div_ceil(2), self.pitch * 2);
        } else {
            layout::in_tile_plane(luma, y, ys, w, h, self.pitch);
            layout::in_tile_plane(chroma, uv, uvs, w, h.div_ceil(2), self.pitch);
        }
        Ok(())
    }
    /// P010 planes are bytes in native little-endian sample order; strides are bytes.
    pub fn load_p010(&mut self, y: &[u8], ys: usize, uv: &[u8], uvs: usize) -> VResult<()> {
        let ds = self.input_slot()?;
        self.aq_compute(y, ys, 2);
        let (w, h) = (self.cfg.width as usize, self.cfg.height as usize);
        let (luma, chroma) = self.input[ds].bytes_mut().split_at_mut(self.luma_sz);
        layout::in_tile_plane(luma, y, ys, w * 2, h, self.pitch * 2);
        layout::in_tile_plane(chroma, uv, uvs, up(w, 2) * 2, h.div_ceil(2), self.pitch * 2);
        Ok(())
    }
    pub fn load_yuv444(
        &mut self,
        y: &[u8],
        ys: usize,
        u: &[u8],
        us: usize,
        v: &[u8],
        vs: usize,
    ) -> VResult<()> {
        let ds = self.input_slot()?;
        let (w, h) = (self.cfg.width as usize, self.cfg.height as usize);
        let bps = self.bps;
        let pitch = self.pitch;
        let (luma, chroma) = self.input[ds].bytes_mut().split_at_mut(self.luma_sz);
        if bps == 2 {
            tile_widened(luma, y, ys, w, h, pitch * 2);
        } else {
            layout::in_tile_plane(luma, y, ys, w, h, pitch);
        }
        // Interleave directly into the tiled surface: no per-frame staging allocation.
        let widened_chroma_pitch = pitch * 2 * if 2 * w > pitch { 2 } else { 1 };
        for r in 0..h {
            for col in 0..w {
                for (k, src, s) in [(0, u, us), (1, v, vs)] {
                    let x = (2 * col + k) * bps;
                    let value = src[r * s + col];
                    if bps == 2 {
                        chroma[layout::in_offset(x, r, widened_chroma_pitch)] = 0;
                        chroma[layout::in_offset(x + 1, r, widened_chroma_pitch)] = value;
                    } else {
                        chroma[layout::in_offset(x, r, 2 * pitch)] = value;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn load_nv24(&mut self, y: &[u8], ys: usize, uv: &[u8], uvs: usize) -> VResult<()> {
        let ds = self.input_slot()?;
        let (w, h) = (self.cfg.width as usize, self.cfg.height as usize);
        let (l, c) = self.input[ds].bytes_mut().split_at_mut(self.luma_sz);
        layout::in_tile_plane(l, y, ys, w, h, self.pitch);
        layout::in_tile_plane(c, uv, uvs, 2 * w, h, 2 * self.pitch);
        Ok(())
    }
    pub fn load_x444(&mut self, y: &[u8], ys: usize, uv: &[u8], uvs: usize) -> VResult<()> {
        let ds = self.input_slot()?;
        let (w, h) = (self.cfg.width as usize, self.cfg.height as usize);
        let (l, c) = self.input[ds].bytes_mut().split_at_mut(self.luma_sz);
        layout::in_tile_plane(l, y, ys, 2 * w, h, 2 * self.pitch);
        layout::in_tile_plane(c, uv, uvs, 4 * w, h, 4 * self.pitch);
        Ok(())
    }
    pub fn load_l10r(&mut self, src: &[u8], stride: usize) -> VResult<()> {
        let ds = self.input_slot()?;
        let (l, c) = self.input[ds].bytes_mut().split_at_mut(self.luma_sz);
        layout::in_tile_l10r(
            l,
            c,
            src,
            stride,
            self.cfg.width as usize,
            self.cfg.height as usize,
            self.pitch * 2,
        );
        Ok(())
    }
    pub fn load_bgra(&mut self, src: &[u8], stride: usize) -> VResult<()> {
        let ds = self.input_slot()?;
        let (l, c) = self.input[ds].bytes_mut().split_at_mut(self.luma_sz);
        layout::in_tile_bgra(
            l,
            c,
            src,
            stride,
            self.cfg.width as usize,
            self.cfg.height as usize,
            self.pitch,
        );
        Ok(())
    }
}
fn bs_capacity(mbs: usize) -> usize {
    up(mbs * 384 + (1 << 20), 65536) + 4096
}
fn tile_widened(dst: &mut [u8], src: &[u8], stride: usize, w: usize, h: usize, pitch: usize) {
    for r in 0..h {
        for c in 0..w {
            dst[layout::in_offset(2 * c, r, pitch)] = 0;
            dst[layout::in_offset(2 * c + 1, r, pitch)] = src[r * stride + c];
        }
    }
}
impl Drop for Encoder {
    fn drop(&mut self) {
        if self.chan.submitted > 0 {
            let _ = self.chan.wait(&self.rm.drm, self.chan.submitted);
        }
    }
}

impl Encoder {
    fn setup_h264(&mut self, ds: usize, idr: bool, b: bool, nrefs: usize, poc: u32) {
        let hp = self.h264_params();
        let c = &self.cfg;
        let magic = self.magic();
        let nl0 = if b { nrefs - 1 } else { nrefs };
        let l0 = if b {
            &self.slot_of[1..]
        } else {
            &self.slot_of[..]
        };
        let s = &mut self.setup[ds];
        s.bytes_mut().fill(0);
        let ps = s.view_mut::<nvenc_h264_drv_pic_setup_s>(0);
        ps.magic = magic;
        let mut ic = nvenc_h264_surface_cfg_s {
            frame_width_minus1: (c.width - 1) as u16,
            frame_height_minus1: (c.height - 1) as u16,
            sfc_pitch: self.pitch as u16,
            sfc_pitch_chroma: self.pitch as u16,
            ..Default::default()
        };
        ic.set_block_height(2);
        let mut rc = ic;
        rc.sfc_pitch = self.r_pitch as u16;
        rc.sfc_pitch_chroma = self.r_pitch as u16;
        if c.chroma444 {
            ic.sfc_pitch_chroma = (2 * self.pitch) as u16;
            rc.sfc_pitch_chroma = (2 * self.r_pitch) as u16;
        }
        ps.input_cfg = ic;
        ps.refpic_cfg = rc;
        ps.outputpic_cfg = rc;
        let sp = &mut ps.sps_data;
        sp.set_profile_idc(hp.profile_idc as u32);
        sp.set_level_idc(hp.level_idc as u32);
        sp.set_chroma_format_idc(if c.chroma444 { 3 } else { 1 });
        sp.set_lossless_qpprime_flag(c.lossless as u32);
        sp.set_pic_order_cnt_type(hp.poc_type as u32);
        sp.set_log2_max_frame_num_minus4(4);
        sp.set_log2_max_pic_order_cnt_lsb_minus4(hp.log2_max_poc_lsb_minus4 as u32);
        sp.set_frame_mbs_only(1);
        let pp = &mut ps.pps_data;
        pp.set_num_ref_idx_l0_active_minus1((c.num_refs - 1) as u32);
        pp.set_deblocking_filter_control_present_flag(1);
        pp.set_entropy_coding_mode_flag(c.cabac as u32);
        pp.set_transform_8x8_mode_flag((c.h264_profile == 100 || c.chroma444) as u32);
        let r = &mut ps.rate_control;
        let qb = (c.qp_p + 2).min(51);
        r.QP = [c.qp_p as u8, qb as u8, c.qp_i as u8];
        r.maxQP = [51; 3];
        if let Some(q) = c.min_qp {
            r.minQP = [q.min(51); 3];
        }
        if let Some(q) = c.max_qp {
            r.maxQP = [q.min(51).max(c.min_qp.unwrap_or(0).min(51)); 3];
        }
        if c.min_qp.is_some() || c.max_qp.is_some() {
            for q in &mut r.QP {
                *q = (*q).clamp(r.minQP[0], r.maxQP[0]);
            }
        }
        r.rhopbi = [256; 3];
        r.framerate = c.fps;
        r.gop_length = u32::MAX;
        if c.rc_mode != 0 {
            let pixels = c.width as f64 * c.height as f64;
            let per_frame = c.bitrate as f64 / c.fps as f64;
            r.hrd_type = 2;
            r.maxQPD = 6;
            r.baseQPD = 3;
            r.framerate = c.fps << 8;
            r.buffersize = c.vbv_bits as u32;
            r.nal_cpb_size = c.vbv_bits;
            r.vcl_cpb_size = c.vbv_bits;
            r.nal_bitrate = c.max_bitrate;
            r.vcl_bitrate = c.max_bitrate;
            r.gop_length = if c.idr_period > 0 {
                c.idr_period as u32
            } else {
                u32::MAX
            };
            r.Np = if c.idr_period > 0 {
                (c.idr_period - 1) << 4
            } else {
                i32::MAX
            };
            r.Ravg = (per_frame / pixels * 256.0) as i32;
            r.R = r.Ravg;
            if c.rc_mode == 1 || c.rc_mode == 17 {
                r.R = (c.max_bitrate as f64 / c.fps as f64 / pixels * 256.0) as i32;
            }
            r.iSizeRatioX = 1;
            r.iSizeRatioY = 1;
            if c.rc_mode & 3 == 2 {
                r.hrd_type = 34;
                r.rhopbi = [286, 317, 256];
                r.Np = Self::rc_np_cfg(c);
                r.Bmin = (r.buffersize / 81920) as i32;
                r.Ravg = 0;
            }
        }
        let pc = &mut ps.pic_control;
        pc.l0 = [-2; 8];
        pc.l1 = [-2; 8];
        for i in 0..nl0 {
            pc.l0[i] = (2 * l0[i]) as i8;
            pc.temp_dist_l0[i] = ((poc as i32 - self.slot_poc[l0[i]] as i32) / 2) as i8;
        }
        if b {
            pc.l1[0] = (2 * self.slot_of[0]) as i8;
            pc.temp_dist_l1[0] = ((poc as i32 - self.slot_poc[self.slot_of[0]] as i32) / 2) as i8;
        }
        pc.dist_scale_factor = [[if b { -1 } else { 256 }; 8]; 8];
        if b {
            for i in 0..nl0 {
                let tb = pc.temp_dist_l0[i] as i32;
                let td = tb - pc.temp_dist_l1[0] as i32;
                let tx = (16384 + td.abs() / 2) / td;
                pc.dist_scale_factor[0][i] = ((tb * tx + 32) >> 6).clamp(-1024, 1023) as i16;
            }
            pc.diff_pic_order_cnt_zero = [0xff00ff00, 0xff00fffe];
        } else if !idr {
            pc.diff_pic_order_cnt_zero = [0xfcfcfcfc; 2];
        }
        pc.frame_num = self.frame_num as u16;
        pc.pic_order_cnt_lsb = (poc & 255) as u16;
        pc.slice_control_offset = OFF_SLICE as u32;
        pc.me_control_offset = OFF_ME as u32;
        pc.md_control_offset = OFF_MD as u32;
        pc.q_control_offset = OFF_Q as u32;
        pc.hist_buf_size = self.hist.size as u32;
        pc.bitstream_buf_size = self.bs[ds].size as u32;
        pc.set_pic_type(if idr {
            3
        } else if b {
            1
        } else {
            0
        });
        pc.set_ref_pic_flag((!b) as u32);
        pc.set_cabac_zero_word_enable(1);
        pc.set_e4byteStartCode(1);
        pc.set_slice_mode(0);
        let nsl = (c.slices.clamp(1, 16) as usize).min(self.hmb);
        pc.num_forced_slices_minus1 = (nsl - 1) as u8;
        for k in 0..nsl {
            let sc = s.view_mut::<nvenc_h264_slice_control_s>(
                OFF_SLICE + k * size_of::<nvenc_h264_slice_control_s>(),
            );
            sc.set_num_mb((((k + 1) * self.hmb / nsl - k * self.hmb / nsl) * self.wmb) as u32);
            sc.set_qp_avr(if idr {
                c.qp_i
            } else if b {
                qb
            } else {
                c.qp_p
            } as u32);
            sc.set_qp_slice_max(51);
            if !idr {
                sc.set_num_ref_idx_active_override_flag(1);
                sc.set_num_ref_idx_l0_active_minus1((nl0 - 1) as u32);
                sc.set_num_ref_idx_l1_active_minus1(0);
            }
        }
        let high = c.h264_profile == 100 || c.chroma444;
        s.copy(
            OFF_ME,
            if idr {
                &H264_FAST_ME_IDR
            } else {
                &H264_FAST_ME_P
            },
        );
        s.copy(
            OFF_MD,
            match (!idr, high) {
                (false, false) => &H264_FAST_MD_0_0,
                (false, true) => &H264_FAST_MD_0_1,
                (true, false) => &H264_FAST_MD_1_0,
                (true, true) => &H264_FAST_MD_1_1,
            },
        );
        if c.quality || b {
            s.copy(
                OFF_ME,
                if idr {
                    &H264_TMPL_ME_IDR
                } else if b {
                    &H264_TMPL_ME_B
                } else {
                    &H264_TMPL_ME_P
                },
            );
            s.copy(
                OFF_MD,
                if idr {
                    &H264_TMPL_MD_IDR
                } else if b {
                    &H264_TMPL_MD_B
                } else {
                    &H264_TMPL_MD_P
                },
            );
            s.copy(OFF_Q, &H264_TMPL_Q);
            if high {
                s.view_mut::<nvenc_h264_md_control_s>(OFF_MD)
                    .set_intra_luma8x8_mode_enable(0x1ff);
            }
        } else if self.chan.class == 0xb4b7 {
            s.copy(
                OFF_ME,
                if idr {
                    &H264_NV66_ME_IDR
                } else {
                    &H264_NV66_ME_P
                },
            );
            s.copy(
                OFF_MD,
                if idr {
                    &H264_NV66_MD_IDR
                } else {
                    &H264_NV66_MD_P
                },
            );
            s.copy(OFF_Q, &H264_NV66_Q);
            if high {
                s.view_mut::<nvenc_h264_md_control_s>(OFF_MD)
                    .set_intra_luma8x8_mode_enable(0x1ff);
            }
        } else {
            s.copy(OFF_Q, &H264_TMPL_Q);
        }
    }

    fn setup_hevc(&mut self, ds: usize, idr: bool, b: bool, nrefs: usize, poc: u32) {
        let hp = self.hevc_params();
        let magic = self.magic();
        let c = &self.cfg;
        let (cw, ch) = (hevchdr::coded_width(&hp), hevchdr::coded_height(&hp));
        let s = &mut self.setup[ds];
        s.bytes_mut().fill(0);
        let ps = s.view_mut::<nvenc_h265_drv_pic_setup_s>(0);
        ps.magic = magic;
        let mut ic = nvenc_h265_surface_cfg_s {
            frame_width_minus1: (c.width - 1) as u16,
            frame_height_minus1: (c.height - 1) as u16,
            sfc_pitch: self.pitch as u16,
            sfc_pitch_chroma: self.pitch as u16,
            ..Default::default()
        };
        ic.set_block_height(2);
        let mut rc = ic;
        rc.sfc_pitch = self.r_pitch as u16;
        rc.sfc_pitch_chroma = self.r_pitch as u16;
        rc.frame_width_minus1 = (cw - 1) as u16;
        rc.frame_height_minus1 = (ch - 1) as u16;
        rc.luma_bot_offset = (ch / 2 * self.r_pitch * self.bps / 256) as u32;
        rc.chroma_bot_offset = (up(ch / 2, 32) / 2 * self.r_pitch * self.bps / 256) as u32;
        if c.chroma444 {
            ic.sfc_pitch_chroma = (2 * self.pitch) as u16;
            rc.sfc_pitch_chroma = (2 * self.r_pitch) as u16;
        }
        ps.input_cfg = ic;
        ps.refpic_cfg = rc;
        ps.outputpic_cfg = rc;
        let sp = &mut ps.sps_data;
        sp.set_chroma_format_idc(if c.chroma444 { 3 } else { 1 });
        sp.set_log2_max_pic_order_cnt_lsb_minus4(4);
        sp.set_min_cu_size(if c.quality && !c.no_cu8 { 0 } else { 1 });
        sp.set_max_cu_size(2);
        sp.set_min_tu_size(0);
        sp.set_max_tu_size(3);
        sp.set_max_transform_depth_inter(3);
        sp.set_max_transform_depth_intra(3);
        sp.set_amp_enabled_flag(1);
        sp.set_num_short_term_ref_pic_sets(1);
        sp.set_sample_adaptive_offset_enabled_flag(1);
        sp.set_strong_intra_smoothing_enabled_flag(0);
        sp.set_bit_depth_minus_8((c.bit_depth - 8) as u32);
        let pp = &mut ps.pps_data;
        pp.set_num_ref_idx_l0_active_minus1((c.num_refs - 1) as u32);
        pp.set_transform_skip_enabled_flag((!hp.no_ts) as u32);
        pp.set_transquant_bypass_enabled_flag(c.lossless as u32);
        pp.set_pps_loop_filter_across_slices_enabled_flag(1);
        pp.set_cu_qp_delta_enabled_flag(1);
        pp.set_cabac_init_present_flag(1);
        pp.set_sign_data_hiding_enabled_flag(0);
        let r = &mut ps.rate_control;
        if c.aq != 0 && self.aq_valid[ds] {
            r.aqMode = 3;
            ps.pic_control.act_stat_offset = (65536 + ds * self.aq_stride) as u32;
        }
        let qb = (c.qp_p + 2).min(51);
        r.QP = [c.qp_p as u8, qb as u8, c.qp_i as u8];
        r.minQP = [if c.bit_depth > 8 { 244 } else { 0 }; 3];
        r.maxQP = [51; 3];
        if let Some(q) = c.min_qp {
            r.minQP = [q.min(51); 3];
        }
        if let Some(q) = c.max_qp {
            r.maxQP = [q.min(51).max(c.min_qp.unwrap_or(0).min(51)); 3];
        }
        if c.min_qp.is_some() || c.max_qp.is_some() {
            for q in &mut r.QP {
                *q = (*q).clamp(c.min_qp.unwrap_or(0).min(51), r.maxQP[0]);
            }
        }
        r.rhopbi = [256; 3];
        r.framerate = c.fps;
        r.gop_length = if c.idr_period > 0 {
            c.idr_period as u32
        } else {
            u32::MAX
        };
        if c.rc_mode != 0 {
            let pixels = c.width as f64 * c.height as f64;
            let per_frame = c.bitrate as f64 / c.fps as f64;
            r.hrd_type = 2;
            r.maxQPD = 6;
            r.baseQPD = 3;
            r.framerate = c.fps << 8;
            r.buffersize = c.vbv_bits as u32;
            r.nal_cpb_size = c.vbv_bits;
            r.vcl_cpb_size = c.vbv_bits;
            r.nal_bitrate = c.max_bitrate;
            r.vcl_bitrate = c.max_bitrate;
            r.Np = if c.idr_period > 0 {
                (c.idr_period - 1) << 4
            } else {
                i32::MAX
            };
            r.Ravg = (per_frame / pixels * 256.0) as i32;
            r.R = r.Ravg;
            if c.rc_mode == 1 || c.rc_mode == 17 {
                r.R = (c.max_bitrate as f64 / c.fps as f64 / pixels * 256.0) as i32;
            }
            r.i_size_ratio_x = 1;
            r.i_size_ratio_y = 1;
            if c.rc_mode & 3 == 2 {
                r.hrd_type = 34;
                r.rhopbi = [286, 317, 256];
                r.Np = Self::rc_np_cfg(c);
                r.Bmin = (r.buffersize / 81920) as i32;
                r.Ravg = 0;
            }
        }
        let ctbs = (c.width as u32).div_ceil(32) * (c.height as u32).div_ceil(32);
        let lg = 32 - (ctbs - 1).leading_zeros();
        let pc = &mut ps.pic_control;
        pc.l0 = [-2; 8];
        pc.l1 = [-2; 8];
        pc.poc_entry_l0 = [-1; 8];
        pc.poc_entry_l1 = [-1; 8];
        pc.set_pic_output_flag(1);
        pc.slice_control_offset = OFF_SLICE as u32;
        pc.me_control_offset = OFF_ME as u32;
        pc.md_control_offset = OFF_MD as u32;
        pc.q_control_offset = OFF_Q as u32;
        pc.hist_buf_size = self.hist.size as u32;
        pc.bitstream_buf_size = self.bs[ds].size as u32;
        pc.set_pic_type(if idr {
            3
        } else if b {
            1
        } else {
            0
        });
        pc.set_ref_pic_flag((!b) as u32);
        pc.set_slice_mode(0);
        pc.set_codec(5);
        pc.set_ceil_log2_pic_size_in_ctbs(lg);
        pc.set_sao_luma_mode(31);
        pc.set_sao_chroma_mode(31);
        pc.set_nal_unit_type(if idr {
            19
        } else if b {
            0
        } else {
            1
        });
        pc.set_nuh_temporal_id_plus1(1);
        pc.set_e4byteStartCode(1);
        pc.set_qpfifo(1);
        pc.set_enable_source_image_padding(1);
        pc.strips_in_frame = 1;
        pc.set_chroma_skip_threshold_4x4(8);
        pc.pic_order_cnt_lsb = (poc & 255) as u16;
        if !idr {
            let nl0 = if b { nrefs - 1 } else { nrefs };
            let nl1 = usize::from(b);
            let ne = nl0 + nl1;
            let mut ent = [0; 5];
            let mut td = [0i32; 5];
            for k in 0..nl0 {
                ent[k] = self.slot_of[if b { k + 1 } else { k }];
            }
            if b {
                ent[nl0] = self.slot_of[0];
            }
            for k in 0..ne {
                td[k] = poc as i32 - (self.slot_poc[ent[k]] / 2) as i32;
            }
            let rps = &mut pc.short_term_rps;
            rps.num_negative_pics = nl0 as i16;
            rps.num_positive_pics = nl1 as i16;
            pc.temp_dist_l0 = [-1; 8];
            pc.temp_dist_l1 = [-1; 8];
            pc.dist_scale_factor = [[-1; 8]; 8];
            let mut prev = 0;
            for i in 0..nl0 {
                pc.l0[i] = (2 * ent[i]) as i8;
                pc.poc_entry_l0[i] = i as i8;
                pc.temp_dist_l0[i] = td[i] as i8;
                rps.delta_poc_s0_minus1[i] = (td[i] - prev - 1) as i16;
                prev = td[i];
                rps.used_by_curr_pic_s0_flag |= 1 << i;
            }
            if b {
                pc.l1[0] = (2 * ent[nl0]) as i8;
                pc.poc_entry_l1[0] = nl0 as i8;
                pc.temp_dist_l1[0] = td[nl0] as i8;
                rps.delta_poc_s1_minus1[0] = (-td[nl0] - 1) as i16;
                rps.used_by_curr_pic_s1_flag = 1;
            }
            for i in 0..ne {
                for j in 0..ne {
                    let tx = (16384 + td[j].abs() / 2) / td[j];
                    pc.dist_scale_factor[j][i] = ((td[i] * tx + 32) >> 6).clamp(-4096, 4095) as i16;
                }
            }
            for i in 0..ne {
                pc.same_poc[i * 9 / 32] |= 1 << (i * 9 % 32);
            }
            for j in 0..8 {
                pc.diff_pic_order_cnt_zero[j / 4] |= if j < nl1 {
                    0
                } else {
                    ((0xffu32 << nl0) & 255) << (8 * (j % 4))
                };
            }
        }
        let sc = s.view_mut::<nvenc_h265_slice_control_s>(OFF_SLICE);
        sc.set_num_ctus(ctbs);
        sc.set_qp_avr(if idr {
            c.qp_i
        } else if b {
            qb
        } else {
            c.qp_p
        } as u32);
        sc.set_slice_type(if idr {
            2
        } else if b {
            1
        } else {
            0
        });
        sc.set_slice_loop_filter_across_slices_enabled_flag(1);
        sc.set_slice_sao_luma_flag(1);
        sc.set_slice_sao_chroma_flag(1);
        if !idr {
            sc.set_num_ref_idx_active_override_flag(1);
            sc.set_num_ref_idx_l0_active_minus1((if b { nrefs - 2 } else { nrefs - 1 }) as u32);
            sc.set_num_ref_idx_l1_active_minus1(0);
        }
        s.copy(OFF_ME, &HEVC_FAST_ME);
        s.copy(OFF_MD, &HEVC_FAST_MD);
        s.copy(OFF_Q, &HEVC_FAST_Q);
        if c.quality {
            s.copy(
                OFF_ME,
                if idr {
                    &HEVC_P7_ME_IDR
                } else if b {
                    &HEVC_P7_ME_B
                } else {
                    &HEVC_P7_ME_P
                },
            );
            s.copy(
                OFF_MD,
                if idr {
                    &HEVC_P7_MD_IDR
                } else if b {
                    &HEVC_P7_MD_B
                } else {
                    &HEVC_P7_MD_P
                },
            );
            s.copy(OFF_Q, &HEVC_P7_Q);
            let big = c.width > 1920 || c.height > 1088;
            let me = s.view_mut::<nvenc_h264_me_control_s>(OFF_ME);
            me.limit_mv
                .set_left_mvx_int(if big { -2048 } else { -1024 });
            me.limit_mv.set_right_mvx_int(if big { 2047 } else { 1023 });
            me.limit_mv.set_top_mvy_int(if big { -512 } else { -256 });
            me.limit_mv.set_bottom_mvy_int(if big { 511 } else { 255 });
        }
        if c.bit_depth > 8 {
            let off = std::mem::offset_of!(nvenc_h265_drv_pic_setup_s, pps_data);
            s.bytes_mut()[off + 7] |= 8;
            s.bytes_mut()[0x314] = 0x22;
            s.bytes_mut()[0x315] = 2;
        }
    }
}

fn av1_lf_level(q: i32, key: bool) -> i32 {
    const AC: [i32; 16] = [4, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22];
    let acq = if q < 16 {
        AC[q as usize]
    } else {
        (q * 1191 + 1250) / 1000
    };
    (((acq * 20723 + 1015158 + (1 << 17)) >> 18) - if key { 4 } else { 0 }).clamp(0, 63)
}
impl Encoder {
    fn setup_av1(
        &mut self,
        ds: usize,
        idr: bool,
        is_b: bool,
        q: i32,
        lf: i32,
        nrefs: usize,
        poc: u32,
    ) {
        let area = self.rc_area();
        let c = &self.cfg;
        let rows = up(c.height as usize, 64);
        let b = self.setup[ds].bytes_mut();
        b[..AV1_TMPL_KEY.len()].copy_from_slice(if idr { &AV1_TMPL_KEY } else { &AV1_TMPL_P });
        b[OFF_AV1_TILE..OFF_AV1_TILE + AV1_TMPL_KEY_TILE.len()].copy_from_slice(if idr {
            &AV1_TMPL_KEY_TILE
        } else {
            &AV1_TMPL_P_TILE
        });
        let dims = (c.width - 1) as u32 | ((c.height - 1) as u32) << 16;
        let (rp, ip, rpb) = (
            self.r_pitch as u32,
            self.pitch as u32,
            (self.r_pitch * self.bps) as u32,
        );
        for k in 0..4 {
            let o = 4 + 32 * k;
            let input = k == 1;
            put32(b, o, dims);
            put32(b, o + 4, if input { ip | ip << 16 } else { rp | rp << 16 });
            if input {
                put32(b, o + 0x14, (self.luma_sz >> 8) as u32);
            } else {
                put32(b, o + 0x10, (rows / 2 * rpb as usize >> 8) as u32);
                put32(b, o + 0x14, (self.r_luma >> 8) as u32);
                put32(
                    b,
                    o + 0x18,
                    ((self.r_luma + rows / 4 * rpb as usize) >> 8) as u32,
                );
            }
        }
        if c.bit_depth > 8 {
            put32(b, 0x84, get32(b, 0x84) | 2 << 7 | 2 << 11);
        }
        b[0x98..0x9c].fill(lf as u8);
        if self.av1_hq {
            b[0x1b8..0x1b8 + AV1_TMPL_BF_B_MDME.len()].copy_from_slice(if is_b {
                &AV1_TMPL_BF_B_MDME
            } else {
                &AV1_TMPL_BF_P_MDME
            });
            b[0xa8..0xc0].copy_from_slice(&AV1_TMPL_HQ_FH[..0x18]);
            b[0xc4..0xc8].copy_from_slice(&AV1_TMPL_HQ_FH[0x1c..0x20]);
        }
        if !idr {
            let nl0 = if is_b { nrefs - 1 } else { nrefs };
            let l0 = if is_b {
                &self.slot_of[1..]
            } else {
                &self.slot_of[..]
            };
            for i in 0..4 {
                b[0x128 + i] = if i < nl0 { i as u8 } else { 255 };
                b[0x136 + i] = if i < nl0 { i as u8 } else { 255 };
                b[0x12f + i] = if i < nl0 {
                    poc.wrapping_sub(self.slot_poc[l0[i]]) as u8
                } else {
                    255
                };
            }
            b[0x144] = (b[0x144] & !0xe0) | ((nl0 as u8) << 5);
            if nl0 >= 2 {
                b[0x8d] |= 4;
            }
            if is_b {
                b[0x12c] = 6;
                b[0x13a] = 6;
                b[0x133] = poc.wrapping_sub(self.slot_poc[self.slot_of[0]]) as u8;
                b[0x145] = (b[0x145] & !7) | 1;
                b[0x8d] |= 2;
                b[0x97] = 1;
                b[0x160] = (b[0x160] & !7) | 1;
            }
        }
        b[0xc0] = q as u8;
        b[0xc9] = c.qp_p as u8;
        b[0xca] = (c.qp_p + 26).min(255) as u8;
        b[0xcb] = c.qp_i as u8;
        put32(b, 0x154, self.hist.size as u32);
        put32(b, 0x158, self.bs[ds].size as u32);
        put32(b, 0x15c, 0);
        put32(b, 0x16c, 4096);
        put32(b, 0x18c, 4096);
        put32(b, 0x178, 0x100 + area as u32);
        put32(b, 0x174, 0x100 + 2 * area as u32);
        put32(b, 0x180, 0x100 | (self.sb_cols as u32) << 16);
        put32(b, 0x184, self.sb_rows as u32);
        put32(
            b,
            0x31c,
            0x80000000 | (self.wmb * self.hmb * 56).max(202500) as u32,
        );
        let nt = self.tile_cols * self.tile_rows;
        put32(
            b,
            0x144,
            (get32(b, 0x144) & !0x07fff800) | ((nt - 1) as u32) << 19,
        );
        put32(
            b,
            0x148,
            (get32(b, 0x148) & !0xff0) | ((nt - 1) as u32) << 4,
        );
        put32(
            b,
            0x160,
            (get32(b, 0x160) & !0x7fffff80)
                | (self.tile_cols as u32) << 7
                | (self.tile_rows as u32) << 15,
        );
        for i in 0..nt {
            let o = OFF_AV1_TILE + 32 * i;
            let j = if i + 1 < nt { i + 1 } else { i };
            if i != 0 {
                b.copy_within(OFF_AV1_TILE..OFF_AV1_TILE + 32, o);
            }
            for (k, idx) in [i, j].into_iter().enumerate() {
                let (x, y) = (idx % self.tile_cols, idx / self.tile_cols);
                let corners = [
                    x * self.tile_w,
                    y * self.tile_h,
                    ((x + 1) * self.tile_w).min(self.sb_cols) - 1,
                    ((y + 1) * self.tile_h).min(self.sb_rows) - 1,
                ];
                for (n, v) in corners.into_iter().enumerate() {
                    put16(b, o + 16 * k + 2 * n, v as u16);
                }
            }
            b[o + 0xa] = q as u8;
            if i != 0 {
                b[o + 0xb] &= !8;
            }
            if is_b {
                b[o + 0xb] = (b[o + 0xb] & !3) | 1;
            }
            b[o + 0xc] = i as u8;
        }
        if c.rc_mode != 0 {
            put32(b, 0x88, get32(b, 0x88) | 0x2000);
            let vbr = c.rc_mode == 1 || c.rc_mode == 17;
            let pixels = c.width as f64 * c.height as f64;
            let per_frame = c.bitrate as f64 / c.fps as f64;
            // This integer-only RC record is written into the mapped GPU setup.
            let r = self.setup[ds].view_mut::<nvenc_av1_rc_s>(0xc8);
            *r = Default::default();
            r.hrd_type = 2;
            r.QP = [c.qp_p as u8, (c.qp_p + 16).min(255) as u8, c.qp_i as u8];
            r.maxQP = [255; 3];
            if let Some(q) = c.min_qp {
                r.minQP = [q; 3];
            }
            if let Some(q) = c.max_qp {
                r.maxQP = [q.max(c.min_qp.unwrap_or(0)); 3];
            }
            if c.min_qp.is_some() || c.max_qp.is_some() {
                for q in &mut r.QP {
                    *q = (*q).clamp(r.minQP[0], r.maxQP[0]);
                }
            }
            r.maxQPD = 32;
            r.baseQPD = 16;
            r.rhopbi = if vbr {
                [316, 414, 256]
            } else {
                [294, 330, 256]
            };
            r.framerate = (c.fps as u32) << 8;
            r.buffersize = c.vbv_bits as u32;
            r.nal_cpb_size = r.buffersize;
            r.vcl_cpb_size = r.buffersize;
            r.nal_bitrate = if vbr { c.max_bitrate } else { c.bitrate } as u32;
            r.vcl_bitrate = r.nal_bitrate;
            r.gop_length = if c.idr_period > 0 {
                c.idr_period as u32
            } else {
                u32::MAX
            };
            r.Np = if c.idr_period > 0 {
                (c.idr_period - 1) as u32
            } else {
                0
            };
            r.Ravg = (per_frame / pixels * 256.0) as u32;
            r.R = (r.nal_bitrate as f64 / c.fps as f64 / pixels * 256.0) as u32;
            r.Bmin = if vbr { 0 } else { r.buffersize / 81920 };
            r.i_size_ratio_x = 1;
            r.i_size_ratio_y = 1;
        }
    }
    fn two_pass_setups(&mut self, ds: usize, hevc: bool) {
        self.setup1[ds].copy(0, self.setup[ds].bytes());
        if hevc {
            self.setup[ds].copy(OFF_ME + 12, &HEVC_MP2_ME[12..28]);
            self.setup[ds].copy(OFF_ME + 44, &HEVC_MP2_ME[44..92]);
            self.setup[ds]
                .view_mut::<nvenc_h265_drv_pic_setup_s>(0)
                .rate_control
                .two_pass_rc = 2;
            let p1 = self.setup1[ds].view_mut::<nvenc_h265_drv_pic_setup_s>(0);
            p1.rate_control.two_pass_rc = 1;
            p1.pps_data.set_transform_skip_enabled_flag(0);
            let me = self.setup1[ds].view_mut::<nvenc_h264_me_control_s>(OFF_ME);
            me.set_Cu16partDecisionMadeByFPP(1);
            me.set_fbm_op_winner_num_p_frame(1);
            self.setup1[ds].copy(OFF_MD, &HEVC_MP1_MD);
            self.setup1[ds].bytes_mut()[OFF_Q..OFF_Q + 16].fill(0);
            let sc = self.setup1[ds].view_mut::<nvenc_h265_slice_control_s>(OFF_SLICE);
            sc.set_qp_slice_max(51);
            sc.set_slice_sao_luma_flag(0);
            sc.set_slice_sao_chroma_flag(0);
        } else {
            self.setup[ds]
                .view_mut::<nvenc_h264_drv_pic_setup_s>(0)
                .rate_control
                .two_pass_rc = 2;
            self.setup1[ds]
                .view_mut::<nvenc_h264_drv_pic_setup_s>(0)
                .rate_control
                .two_pass_rc = 1;
        }
    }
    fn av1_methods(
        &self,
        words: &mut Vec<u32>,
        ds: usize,
        idr: bool,
        b: bool,
        slot: usize,
        nrefs: usize,
    ) {
        let rec = self.recon.gpu + slot as u64 * self.r_slot as u64;
        let input = if self.ext_input != 0 {
            self.ext_input
        } else {
            self.input[ds].gpu
        };
        let prob_in = if idr {
            self.av1_prob_in.gpu
        } else {
            self.av1_prob.gpu + self.slot_of[0] as u64 * 65536
        };
        let prob_out = self.av1_prob.gpu + if b { self.nslots } else { slot } as u64 * 65536;
        mthd(words, NVC9B7_SET_APPLICATION_ID, 7);
        let rcm = match self.cfg.rc_mode {
            1 => 17,
            2 => 18,
            x => x,
        };
        mthd(
            words,
            NVC9B7_SET_CONTROL_PARAMS,
            0x00041905 | (rcm as u32) << 24,
        );
        mthd(words, NVC9B7_SET_PICTURE_INDEX, self.seq);
        addr(words, NVC9B7_SET_IN_DRV_PIC_SETUP, self.setup[ds].gpu);
        addr(words, NVC9B7_SET_IOHISTORY, self.hist.gpu);
        if !idr {
            addr(
                words,
                NVC9B7_SET_IN_MEPRED_DATA,
                self.mepred[self.seq as usize & 1].gpu,
            );
            addr(
                words,
                NVC9B7_SET_OUT_MEPRED_DATA,
                self.mepred[(self.seq as usize + 1) & 1].gpu,
            );
        }
        for (m, a) in [
            (NVC9B7_SET_IO_RC_PROCESS, self.rcproc.gpu),
            (NVC9B7_SET_OUT_REF_PIC_LUMA, rec),
            (NVC9B7_SET_OUT_REF_PIC_CHROMA, rec),
            (NVC9B7_SET_OUT_ENC_STATUS, self.status[ds].gpu),
            (NVC9B7_SET_OUT_BITSTREAM, self.bs[ds].gpu),
            (NVC9B7_SET_IN_CUR_PIC, input),
            (NVC9B7_SET_IN_CUR_PIC_CHROMA_U, input),
            (NVC9B7_SET_IN_CUR_PIC_CHROMA_V, input),
            (NVC9B7_SET_IN_RCDATA, self.rcdata.gpu),
            (NVC9B7_SET_IN_AV1_PROBABILITY_DATA, prob_in),
            (NVC9B7_SET_OUT_AV1_PROBABILITY_DATA, prob_out),
        ] {
            addr(words, m, a);
        }
        for (i, m) in [
            NVC9B7_SET_IOENTROPY_ABOVE_MV_DATA,
            NVC9B7_SET_IOENTROPY_ABOVE_CTX_DATA,
            NVC9B7_SET_IODEBLOCK_TOPROW,
            NVC9B7_SET_IODEBLOCK_LEFTCOLUMN,
            NVC9B7_SET_IOCDEF_LEFTCOLUMN,
            NVC9B7_SET_IOLR_LEFTCOLUMN,
        ]
        .into_iter()
        .enumerate()
        {
            addr(words, m, self.av1_line[i].gpu);
        }
        if !idr {
            for i in 0..nrefs {
                let t = if b {
                    if i == 0 {
                        6
                    } else {
                        i - 1
                    }
                } else {
                    i
                };
                let a = self.recon.gpu + self.slot_of[i] as u64 * self.r_slot as u64;
                addr(words, NVC9B7_SET_IN_REF_PIC0_LUMA + 4 * t as u32, a);
                addr(words, NVC9B7_SET_IN_REF_PIC0_CHROMA + 4 * t as u32, a);
            }
        }
    }
    pub fn submit(&mut self, force_idr: bool) -> VResult<u32> {
        self.submit_ex(force_idr, false, -1)
    }
    pub fn submit_ex(&mut self, force_idr: bool, is_b: bool, disp: i32) -> VResult<u32> {
        let ds = self.input_slot()?;
        let b = is_b && self.cfg.bframes > 0 && self.nrefs >= 2;
        let idr = !b
            && (force_idr
                || self.force_idr
                || self.seq == 0
                || (disp < 0
                    && self.cfg.idr_period > 0
                    && self.since_idr >= self.cfg.idr_period as u32));
        self.force_idr = false;
        if idr {
            self.frame_num = 0;
            self.since_idr = 0;
            self.nrefs = 0;
            if disp >= 0 {
                self.idr_disp = disp;
            }
        }
        let slot = if b { self.b_slot } else { self.next_slot };
        let nrefs = if idr { 0 } else { self.nrefs };
        let poc = if disp >= 0 {
            2u32.wrapping_mul((disp - self.idr_disp) as u32)
        } else {
            2 * self.since_idr
        };
        let hevc = self.cfg.codec == ENC_HEVC;
        let av1 = self.cfg.codec == ENC_AV1;
        if av1 {
            // enc_submit_ex fills the shared H.264 controls before submit_av1 patches
            // the AV1 header/tile area. Preserve those bytes too for exact setup dumps.
            self.setup_h264(ds, idr, b, nrefs, poc);
            let q = if idr {
                self.cfg.qp_i
            } else if b {
                (self.cfg.qp_p + 26).min(255)
            } else {
                self.cfg.qp_p
            };
            self.setup_av1(ds, idr, b, q, av1_lf_level(q, idr), nrefs, poc);
        } else if hevc {
            self.setup_hevc(ds, idr, b, nrefs, poc / 2);
        } else {
            self.setup_h264(ds, idr, b, nrefs, poc);
        }
        let mp = self.cfg.two_pass && !(b && hevc);
        if mp {
            self.two_pass_setups(ds, hevc);
        }
        let mut words = Vec::with_capacity(256);
        if av1 {
            self.av1_methods(&mut words, ds, idr, b, slot, nrefs);
        } else {
            let input = if self.ext_input != 0 {
                self.ext_input
            } else {
                self.input[ds].gpu
            };
            let rec_out = self.recon.gpu + slot as u64 * self.r_slot as u64;
            for pass in if mp { 1..=2 } else { 0..=0 } {
                let p1 = pass == 1;
                let rec = if p1 { self.rec1.gpu } else { rec_out };
                mthd(
                    &mut words,
                    NVC9B7_SET_APPLICATION_ID,
                    if hevc { 3 } else { 1 },
                );
                mthd(
                    &mut words,
                    NVC9B7_SET_CONTROL_PARAMS,
                    if hevc { 4 } else { 3 }
                        | 1 << 8
                        | (self.cfg.rc_mode as u32) << 24
                        | u32::from(p1) << 19
                        | u32::from(pass == 2) << 20,
                );
                mthd(&mut words, NVC9B7_SET_PICTURE_INDEX, self.seq);
                for (m, a) in [
                    (
                        NVC9B7_SET_IN_DRV_PIC_SETUP,
                        if p1 {
                            self.setup1[ds].gpu
                        } else {
                            self.setup[ds].gpu
                        },
                    ),
                    (
                        NVC9B7_SET_IOHISTORY,
                        if p1 { self.hist1.gpu } else { self.hist.gpu },
                    ),
                    (
                        if hevc {
                            NVC9B7_SET_OUT_REF_PIC_LUMA
                        } else {
                            NVC9B7_SET_OUT_COLOC_DATA
                        },
                        if hevc {
                            rec
                        } else if p1 {
                            self.coloc1.gpu
                        } else if b {
                            self.coloc_b.gpu
                        } else {
                            self.coloc.gpu
                        },
                    ),
                    (NVC9B7_SET_OUT_REF_PIC_LUMA, rec),
                    (NVC9B7_SET_OUT_REF_PIC_CHROMA, rec + self.r_luma as u64),
                    (
                        NVC9B7_SET_OUT_ENC_STATUS,
                        if p1 {
                            self.status1.gpu
                        } else {
                            self.status[ds].gpu
                        },
                    ),
                    (
                        NVC9B7_SET_OUT_BITSTREAM,
                        if p1 { self.bs1.gpu } else { self.bs[ds].gpu },
                    ),
                    (NVC9B7_SET_IN_CUR_PIC, input),
                    (NVC9B7_SET_IN_CUR_PIC_CHROMA_U, input + self.luma_sz as u64),
                    (NVC9B7_SET_IN_CUR_PIC_CHROMA_V, input + self.luma_sz as u64),
                    (NVC9B7_SET_IO_RC_PROCESS, self.rcproc.gpu),
                    (NVC9B7_SET_IN_RCDATA, self.rcdata.gpu),
                ] {
                    addr(&mut words, m, a);
                }
                if !idr {
                    if !hevc {
                        addr(&mut words, NVC9B7_SET_IN_COLOC_DATA, self.coloc.gpu);
                    }
                    addr(
                        &mut words,
                        NVC9B7_SET_IN_MEPRED_DATA,
                        if pass == 2 {
                            self.mepred1.gpu
                        } else {
                            self.mepred[self.seq as usize & 1].gpu
                        },
                    );
                    addr(
                        &mut words,
                        NVC9B7_SET_OUT_MEPRED_DATA,
                        if p1 {
                            self.mepred1.gpu
                        } else {
                            self.mepred[(self.seq as usize + 1) & 1].gpu
                        },
                    );
                    for i in 0..nrefs {
                        let es = if hevc && b {
                            self.slot_of[if i < nrefs - 1 { i + 1 } else { 0 }]
                        } else {
                            self.slot_of[i]
                        };
                        let n = if !hevc || self.chan.class == 0xb4b7 {
                            es
                        } else {
                            i
                        };
                        let a = self.recon.gpu + es as u64 * self.r_slot as u64;
                        addr(&mut words, NVC9B7_SET_IN_REF_PIC0_LUMA + 4 * n as u32, a);
                        addr(
                            &mut words,
                            NVC9B7_SET_IN_REF_PIC0_CHROMA + 4 * n as u32,
                            a + self.r_luma as u64,
                        );
                    }
                }
                if p1 {
                    mthd(&mut words, NVC9B7_EXECUTE, 0);
                }
            }
        }
        mthd(&mut words, NVC9B7_EXECUTE, 0);
        self.status[ds].bytes_mut()[..if av1 { 0x880 } else { 256 }].fill(0);
        let payload = self.seq.wrapping_add(1);
        // Engine release followed by host acquire orders the channel timeline after NVENC.
        crate::gpu::method(
            &mut words,
            0,
            NVC9B7_SEMAPHORE_A,
            &[(self.sem.gpu >> 32) as u32, self.sem.gpu as u32, payload],
        );
        mthd(&mut words, NVC9B7_SEMAPHORE_D, 0);
        crate::gpu::method(
            &mut words,
            0,
            0x5c,
            &[
                self.sem.gpu as u32,
                (self.sem.gpu >> 32) as u32,
                payload,
                0,
                3,
            ],
        );
        if let Some(dir) = std::env::var_os("ENC_DUMP_SETUP") {
            let path = std::path::Path::new(&dir).join(format!("enc{payload}.bin"));
            let _ = std::fs::write(path, &self.setup[ds].bytes()[..SETUP_SIZE]);
        }
        let hidden = av1
            && !idr
            && !b
            && disp >= 0
            && self.cfg.bframes > 0
            && disp > self.av1_last_shown + 1;
        if hidden {
            self.av1_hid_disp = disp;
        } else if disp >= 0 {
            self.av1_last_shown = if b && disp == self.av1_hid_disp - 1 {
                self.av1_hid_disp
            } else {
                disp
            };
        }
        let mut refs = [0; 4];
        let mut ref_oh = [0; 4];
        for i in 0..nrefs {
            refs[i] = self.slot_of[i];
            ref_oh[i] = (self.slot_poc[self.slot_of[i]] / 2) as i32;
        }
        let t0 = Instant::now();
        let timeline = self.chan.submit(words)?;
        self.seq = payload;
        self.pend[ds] = Pending {
            idr,
            frame_num: self.frame_num,
            poc: if av1 { poc / 2 } else { poc },
            t0,
            slot,
            nrefs,
            refs,
            ref_oh,
            b,
            hidden,
            timeline,
        };
        if !b {
            self.slot_poc[slot] = poc;
            let keep = if idr {
                0
            } else {
                self.nrefs.min(self.cfg.num_refs as usize - 1)
            };
            self.slot_of.copy_within(0..keep, 1);
            self.slot_of[0] = slot;
            self.nrefs = keep + 1;
            for s in 0..=self.cfg.num_refs as usize {
                if !self.slot_of[..self.nrefs].contains(&s) {
                    self.next_slot = s;
                    break;
                }
            }
            self.frame_num = (self.frame_num + 1) % 256;
            self.since_idr += 1;
        }
        Ok(payload)
    }
}

impl Encoder {
    pub fn wait(&mut self, ticket: u32, out: &mut Vec<u8>) -> VResult<EncFrame> {
        if ticket != self.waited.wrapping_add(1)
            || ticket.wrapping_sub(self.waited) > self.seq.wrapping_sub(self.waited)
        {
            return Err(invalid(
                "encoder tickets must be waited in submission order",
            ));
        }
        let ds = ticket.wrapping_sub(1) as usize % self.cfg.depth as usize;
        let p = self.pend[ds].clone();
        let done = self.chan.wait(&self.rm.drm, p.timeline);
        self.waited = ticket;
        if let Err(e) = done {
            self.force_idr = true;
            return Err(e);
        }
        let st = self.status[ds].view::<nvenc_pic_stat_s>(0);
        let mut frame = EncFrame {
            size: 0,
            idr: p.idr,
            frame_num: p.frame_num,
            poc: p.poc,
            bits: st.total_bit_count,
            ucode_error: st.ucode_error_status(),
            avg_qp: st.avgQP as u32,
            intra_mbs: st.intra_mb_count as u32,
            inter_mbs: st.inter_mb_count as u32,
            intra_cost: st.total_intra_cost,
            inter_cost: st.total_inter_cost,
            ms: p.t0.elapsed().as_secs_f64() * 1000.0,
            hidden: false,
        };
        if st.ucode_error_status() != 0
            || st.total_bit_count == 0
            || st.total_bit_count as usize / 8 >= self.bs[ds].size
        {
            self.force_idr = true;
            return Err(VideoError::DeviceLost(format!(
                "NVENC frame {ticket}: microcode error {}, {} bits",
                st.ucode_error_status(),
                st.total_bit_count
            )));
        }
        let start = out.len();
        if self.cfg.codec == ENC_AV1 {
            self.av1_assemble(ds, out, &mut frame)?;
        } else {
            if p.idr {
                h264hdr::write_hdr_sei(
                    self.cfg.codec == ENC_HEVC,
                    if self.cfg.has_mdcv {
                        Some(&self.cfg.mdcv)
                    } else {
                        None
                    },
                    if self.cfg.has_cll {
                        Some(&self.cfg.cll)
                    } else {
                        None
                    },
                    out,
                );
            }
            let off = st.bitstream_start_pos as usize;
            let size = st.total_bit_count as usize / 8;
            let data = self.bs[ds]
                .bytes()
                .get(off..off + size)
                .ok_or_else(|| invalid("NVENC bitstream exceeds output allocation"))?;
            out.extend_from_slice(data);
        }
        frame.size = out.len() - start;
        Ok(frame)
    }
    pub fn encode(&mut self, force_idr: bool, out: &mut Vec<u8>) -> VResult<EncFrame> {
        let t = self.submit(force_idr)?;
        self.wait(t, out)
    }
    pub fn encode_ex(
        &mut self,
        force_idr: bool,
        b: bool,
        disp: i32,
        out: &mut Vec<u8>,
    ) -> VResult<EncFrame> {
        let t = self.submit_ex(force_idr, b, disp)?;
        self.wait(t, out)
    }
    fn av1_assemble(&mut self, ds: usize, out: &mut Vec<u8>, frame: &mut EncFrame) -> VResult<()> {
        let p = &self.pend[ds];
        let st = self.status[ds].view::<nvenc_av1_pic_stat_s>(0);
        let key = p.idr;
        let is_b = p.b;
        let seq = self.av1_seq();
        let lf = st.loop_filter_level as i32;
        let mut fp = av1hdr::Av1FrameParams {
            frame_type: if key { 0 } else { 1 },
            hidden: p.hidden,
            order_hint: p.poc as i32,
            refresh_frame_flags: if is_b { 0 } else { 1 << p.slot },
            primary_ref_frame: if key {
                7
            } else if is_b {
                6
            } else {
                0
            },
            base_q_idx: st.av1_start_QP as i32,
            lf_level: [lf; 4],
            cdef_damping: st.cdef_damping_minus_3() as i32 + 3,
            cdef_bits: if self.av1_hq { 2 } else { 1 },
            cdef_uv: [0; 8],
            cdef_y: [0, if self.av1_hq { 8 } else { 16 }, 16, 36, 0, 0, 0, 0],
            lr_type: [if self.av1_hq { 3 } else { 0 }, 0, 0],
            tx_mode_select: true,
            reference_select: !key,
            interp_filter: 4,
            tile_size_bytes: 4,
            tile_cols_log2: self.tile_cols_log2,
            tile_rows_log2: self.tile_rows_log2,
            context_update_tile_id: 0,
            delta_q_present: self.cfg.rc_mode != 0,
            ..Default::default()
        };
        for i in 0..7 {
            let nr = p.nrefs;
            let r = if nr == 0 {
                0
            } else if is_b {
                if i >= 4 {
                    0
                } else {
                    1 + i.min(nr - 2)
                }
            } else {
                i.min(nr - 1)
            };
            fp.ref_frame_idx[i] = if nr != 0 { p.refs[r] as i32 } else { 0 };
            fp.ref_order_hint[i] = if nr != 0 {
                p.ref_oh[r]
            } else {
                p.poc as i32 - 1
            };
        }
        let nt = self.tile_cols * self.tile_rows;
        if st.num_tiles as usize != nt || nt > 128 {
            return Err(invalid("NVENC AV1 tile count mismatch"));
        }
        let stats = self.status[ds].bytes();
        let bs = self.bs[ds].bytes();
        let mut tiles: [&[u8]; 128] = [&[]; 128];
        for (i, tile) in tiles[..nt].iter_mut().enumerate() {
            let off = get32(stats, 0x100 + 16 * i) as usize;
            let size = get32(stats, 0x104 + 16 * i) as usize;
            if size == 0 {
                return Err(invalid("empty NVENC AV1 tile"));
            }
            *tile = bs
                .get(off..off + size)
                .ok_or_else(|| invalid("NVENC AV1 tile exceeds output allocation"))?;
        }
        if fp.hidden {
            self.av1_hid.clear();
            av1hdr::write_frame(&seq, &fp, &tiles[..nt], &mut self.av1_hid);
            self.av1_hid_slot = p.slot;
            frame.hidden = true;
            return Ok(());
        }
        av1hdr::write_temporal_delimiter(out);
        if key {
            av1hdr::write_sequence_header(&seq, out);
            av1hdr::write_hdr_metadata(
                if self.cfg.has_mdcv {
                    Some(&self.cfg.mdcv)
                } else {
                    None
                },
                if self.cfg.has_cll {
                    Some(&self.cfg.cll)
                } else {
                    None
                },
                out,
            );
        }
        out.extend_from_slice(&self.av1_hid);
        self.av1_hid.clear();
        av1hdr::write_frame(&seq, &fp, &tiles[..nt], out);
        Ok(())
    }
    pub fn av1_show_existing(&self, out: &mut Vec<u8>) {
        av1hdr::write_temporal_delimiter(out);
        av1hdr::write_show_existing(self.av1_hid_slot as i32, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_engines_match_host_c_shim_rejection() {
        let cfg = EncConfig {
            engine: 1,
            width: 642,
            height: 362,
            ..Default::default()
        };
        assert!(
            matches!(Encoder::open(&cfg), Err(VideoError::Unsupported(message)) if message == "only NVENC0 is supported")
        );
    }

    #[test]
    fn rate_control_counts_gop_anchors() {
        let mut c = EncConfig::default();
        assert_eq!(Encoder::rc_np_cfg(&c), i32::MAX);
        c.idr_period = 60;
        assert_eq!(Encoder::rc_np_cfg(&c), 59);
        c.bframes = 2;
        assert_eq!(Encoder::rc_np_cfg(&c), 20);
    }

    #[test]
    fn av1_filter_levels_match_c() {
        assert_eq!(av1_lf_level(80, true), 7);
        assert_eq!(av1_lf_level(100, false), 13);
        assert_eq!(av1_lf_level(0, true), 0);
    }

    #[test]
    fn signed_driver_fields_round_trip() {
        let mut limits = nvenc_h264_me_limit_mv_s::default();
        limits.set_left_mvx_int(-1024);
        limits.set_right_mvx_int(1023);
        limits.set_top_mvy_int(-256);
        limits.set_bottom_mvy_int(255);
        assert_eq!(limits.left_mvx_int(), -1024);
        assert_eq!(limits.right_mvx_int(), 1023);
        assert_eq!(limits.top_mvy_int(), -256);
        assert_eq!(limits.bottom_mvy_int(), 255);
    }

    #[test]
    fn widened_input_preserves_sample_high_bits() {
        let src = [0, 64, 255, 99, 1, 128, 254];
        let mut dst = vec![0xcc; layout::in_plane_size(128, 2)];
        tile_widened(&mut dst, &src, 4, 3, 2, 128);
        for row in 0..2 {
            for col in 0..3 {
                assert_eq!(dst[layout::in_offset(2 * col, row, 128)], 0);
                assert_eq!(
                    dst[layout::in_offset(2 * col + 1, row, 128)],
                    src[4 * row + col]
                );
            }
        }
    }
}
