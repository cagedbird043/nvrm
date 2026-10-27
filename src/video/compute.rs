//! Video compute engine: launches the precompiled kernels of [`super::kernels`] on a compute
//! channel of the video stack's own address space.
//!
//! The kernels are Mesa NAK output generated at build time (tools/gen-kernels.py); the runtime
//! carries no compiler. A launch copies the kernel's QMD template (filled by Mesa's
//! `nak_fill_qmd`) and ORs the program address, cbuf 0 address and grid size into the generated
//! bit runs; cbuf 0 follows the bridge ABI (compiler/README.md): storage buffer `binding` at
//! `binding * 16` as { u64 address, u32 bytes, u32 0 }, push constants at 1024, 1152 bytes.
//! Kernels never use local memory, a CRS stack or constant data (the generator refuses them),
//! so the channel needs no local-memory backing.
use super::{
    channel::wait_sync_file,
    session::{vram, VramBuffer},
    VResult, VideoError,
};
use crate::{
    events::RmEvents,
    gpu::{hi_lo, method, AddressSpace, GpuMapping, Push, Queue, SubmitStatus},
    rm::{Device, Mapping, Memory},
    slots::{SlotPool, TimelineView, PAGE_BYTES},
};
use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

/// A precompiled kernel for one shader model (generated tables in [`super::kernels`]).
pub(crate) struct Kernel {
    pub name: &'static str,
    pub sm: u32,
    pub compute_class: u32,
    pub code: &'static [u8],
    pub registers: u32,
    pub shared_bytes: u32,
    pub barriers: u32,
    pub workgroup: [u32; 3],
    pub cbuf_bytes: u32,
    /// QMD with every per-launch value zero.
    pub qmd: &'static [u32],
    /// Per launch value (program, cbuf, grid x, y, z): (value bit, QMD bit, length) runs.
    pub runs: [&'static [(u32, u32, u32)]; 5],
}

impl Kernel {
    /// The QMD of one launch. Fails when a value has a bit no run carries (misaligned address,
    /// grid dimension beyond the QMD field).
    pub(crate) fn pack(&self, values: [u64; 5]) -> VResult<Vec<u32>> {
        let mut qmd = self.qmd.to_vec();
        for (v, &value) in values.iter().enumerate() {
            let mut carried = 0u64;
            for &(from, to, len) in self.runs[v] {
                let mask = if len >= 64 { u64::MAX } else { (1u64 << len) - 1 };
                carried |= mask << from;
                let field = (value >> from) & mask;
                for b in 0..len {
                    if field >> b & 1 != 0 {
                        let bit = (to + b) as usize;
                        qmd[bit / 32] |= 1 << (bit % 32);
                    }
                }
            }
            if value & !carried != 0 {
                return Err(VideoError::Invalid(format!(
                    "kernel {} launch value {v} = {value:#x} does not fit its QMD field",
                    self.name
                )));
            }
        }
        Ok(qmd)
    }
}

/// Compute classes whose kernels were run on hardware (RTX 4060 Laptop, Ada). Other shader
/// models have generated kernels with unit-tested QMD packing but are never launched.
const VALIDATED_CLASSES: &[u32] = &[0xc9c0];

/// Shader model of the generated kernels for a compute class.
fn sm_of_class(class: u32) -> Option<u32> {
    Some(match class {
        0xc5c0 => 75,
        0xc6c0 => 80,
        0xc7c0 => 86,
        0xc9c0 => 89,
        0xcbc0 => 90,
        0xcec0 => 120,
        _ => return None,
    })
}

const CODE_BYTES: usize = 64 << 10;
/// Zeroed tail after the code heap: instruction prefetch may read past the last program.
const CODE_PREFETCH_PAD: usize = 4096;
/// Programs and QMDs are placed 256-byte aligned (NVK: 0x80 programs, 256 QMDs).
const ALIGN: usize = 256;
/// A launch slot: QMD at 0, cbuf 0 at CBUF_OFFSET, command words at WORDS_OFFSET.
const SLOT_BYTES: usize = 4096;
const CBUF_OFFSET: usize = 512;
const WORDS_OFFSET: usize = 2048;

// Compute class methods (NVIDIA clc5c0.h .. clcdc0.h, identical offsets Turing..Blackwell).
const SET_OBJECT: u32 = 0x0000;
const INVALIDATE_SKED_CACHES: u32 = 0x0298;
const SET_SHADER_SHARED_MEMORY_WINDOW_A: u32 = 0x02a0;
const SEND_PCAS_A: u32 = 0x02b4;
const SEND_SIGNALING_PCAS_B: u32 = 0x02bc; // Turing: INVALIDATE | SCHEDULE
const SEND_SIGNALING_PCAS2_B: u32 = 0x02c0; // Ampere+: PCAS_ACTION_INVALIDATE_COPY_SCHEDULE = 3
const SET_SHADER_LOCAL_MEMORY_WINDOW_A: u32 = 0x07b0;
const INVALIDATE_SHADER_CACHES_NO_WFI: u32 = 0x1698; // INSTRUCTION 0:0, CONSTANT 12:12
const SUBCHANNEL: u32 = 1;

/// One storage buffer of a launch: (binding 0..8, GPU address, bytes).
pub(crate) type Binding = (u32, u64, u32);

pub(crate) struct ComputeEngine {
    queue: Queue,
    compute_class: u32,
    sm: u32,
    _pool: Arc<SlotPool>,
    _page: (Rc<Memory>, Mapping, GpuMapping),
    _events: RmEvents,
    device: Rc<Device>,
    code: VramBuffer,
    code_used: usize,
    programs: HashMap<&'static str, u64>,
    /// Programs written since the last instruction-cache invalidate.
    code_dirty: bool,
    initialized: bool,
    /// Launch slots and the completion token of their last launch.
    slots: Vec<(VramBuffer, u64)>,
}

impl ComputeEngine {
    /// The compute class of this device when its kernels are validated on hardware.
    pub(crate) fn supported(device: &Rc<Device>) -> VResult<Option<u32>> {
        let caps = crate::capabilities::query(device)?;
        Ok(VALIDATED_CLASSES.contains(&caps.compute).then_some(caps.compute))
    }

    pub(crate) fn new(device: &Rc<Device>, space: &AddressSpace) -> VResult<Self> {
        let caps = crate::capabilities::query(device)?;
        if !VALIDATED_CLASSES.contains(&caps.compute) {
            return Err(VideoError::Unsupported(format!(
                "video compute kernels are not validated for compute class {:#x}",
                caps.compute
            )));
        }
        let sm = sm_of_class(caps.compute).ok_or_else(|| {
            VideoError::Unsupported(format!("no kernels for compute class {:#x}", caps.compute))
        })?;
        let view = if crate::gpu::modern_host(caps.channel)? {
            TimelineView::Native64
        } else {
            TimelineView::Split32
        };
        let pool = Arc::new(SlotPool::new(view));
        let page_mem = device.allocate_aligned(PAGE_BYTES, PAGE_BYTES, false, true)?;
        let page_cpu = page_mem.map()?;
        let page_gpu = space.map(&page_mem)?;
        pool.install_page(page_cpu.cpu_address(), page_gpu.address())?;
        let retire = pool
            .try_allocate()
            .ok_or_else(|| VideoError::DeviceLost("empty semaphore slot page".into()))?;
        let events = RmEvents::new(device)?;
        let queue = Queue::new_with_events(device, space, 16, &events, &pool, retire)?;
        let code = vram(device, space, CODE_BYTES + CODE_PREFETCH_PAD)?;
        Ok(Self {
            queue,
            compute_class: caps.compute,
            sm,
            _pool: pool,
            _page: (page_mem, page_cpu, page_gpu),
            _events: events,
            device: Rc::clone(device),
            code,
            code_used: 0,
            programs: HashMap::new(),
            code_dirty: false,
            initialized: false,
            slots: Vec::new(),
        })
    }

    /// The kernel `name` for this engine's shader model, checked against the contract this
    /// runtime implements (kernels.rs ABI 1: no local memory, cbuf 0 of 1152 bytes).
    pub(crate) fn kernel(&self, name: &str) -> VResult<&'static Kernel> {
        let all = super::kernels::KERNELS.iter();
        #[cfg(test)]
        let all = all.chain(super::kernels::TEST_KERNELS.iter());
        let mut all = all;
        let k = all
            .find(|k| k.name == name && k.sm == self.sm && k.compute_class == self.compute_class)
            .ok_or_else(|| VideoError::Unsupported(format!("no kernel {name} for sm_{}", self.sm)))?;
        let threads: u32 = k.workgroup.iter().product();
        if super::kernels::ABI != 1
            || k.cbuf_bytes != 1152
            || k.registers > 255
            || k.barriers > 16
            || threads == 0
            || threads > 1024
            || k.shared_bytes > 48 << 10
            || k.code.is_empty()
            || k.code.len() % 16 != 0
        {
            return Err(VideoError::Invalid(format!("kernel {name} breaks the kernel contract")));
        }
        Ok(k)
    }

    /// GPU address of `kernel`'s program, writing it into the code heap on first use.
    fn program(&mut self, kernel: &'static Kernel) -> VResult<u64> {
        if let Some(&address) = self.programs.get(kernel.name) {
            return Ok(address);
        }
        let offset = self.code_used;
        let end = offset + kernel.code.len();
        if end > CODE_BYTES || kernel.code.len() % 4 != 0 {
            return Err(VideoError::NoMemory(format!("kernel {} does not fit the code heap", kernel.name)));
        }
        for (i, word) in kernel.code.chunks_exact(4).enumerate() {
            self.code
                .cpu
                .write_u32(offset + i * 4, u32::from_le_bytes(word.try_into().unwrap()))?;
        }
        self.code_used = end.next_multiple_of(ALIGN);
        self.code_dirty = true;
        let address = self.code.gpu.address() + offset as u64;
        self.programs.insert(kernel.name, address);
        Ok(address)
    }

    /// Forgets every program so the next ones are written from the start of the code heap again
    /// (same addresses, new code). Waits for every launch first.
    pub(crate) fn reset_code(&mut self, drm: i32) -> VResult<()> {
        let pending: Vec<u64> = self.slots.iter().map(|s| s.1).filter(|&t| t != 0).collect();
        for token in pending {
            self.wait(drm, token)?;
        }
        self.programs.clear();
        self.code_used = 0;
        Ok(())
    }

    /// Launches `kernel` over `groups` workgroups with the given storage buffers and push
    /// constants and returns its completion token (see [`ComputeEngine::wait`]).
    pub(crate) fn launch(
        &mut self,
        space: &AddressSpace,
        kernel: &'static Kernel,
        bindings: &[Binding],
        push: &[u8],
        groups: [u32; 3],
    ) -> VResult<u64> {
        if push.len() > 128 || kernel.cbuf_bytes as usize != 1152 {
            return Err(VideoError::Invalid("push constants exceed 128 bytes".into()));
        }
        if groups.contains(&0) {
            return Err(VideoError::Invalid("empty compute grid".into()));
        }
        let program = self.program(kernel)?;
        let si = match self.slots.iter().position(|(_, token)| {
            *token == 0 || self.queue.shared().retired(*token).unwrap_or(false)
        }) {
            Some(i) => i,
            None => {
                self.slots.push((vram(&self.device, space, SLOT_BYTES)?, 0));
                self.slots.len() - 1
            }
        };
        let base = self.slots[si].0.gpu.address();
        let cbuf_address = base + CBUF_OFFSET as u64;
        let mut cbuf = vec![0u8; kernel.cbuf_bytes as usize];
        let mut seen = 0u32;
        for &(binding, address, bytes) in bindings {
            if binding >= 8 || seen & (1 << binding) != 0 {
                return Err(VideoError::Invalid(format!("storage binding {binding} invalid or repeated")));
            }
            seen |= 1 << binding;
            let at = binding as usize * 16;
            cbuf[at..at + 8].copy_from_slice(&address.to_le_bytes());
            cbuf[at + 8..at + 12].copy_from_slice(&bytes.to_le_bytes());
        }
        cbuf[1024..1024 + push.len()].copy_from_slice(push);
        let qmd = kernel.pack([
            program,
            cbuf_address,
            u64::from(groups[0]),
            u64::from(groups[1]),
            u64::from(groups[2]),
        ])?;

        let mut words = Vec::with_capacity(32);
        if !self.initialized {
            method(&mut words, SUBCHANNEL, SET_OBJECT, &[self.compute_class]);
            method(
                &mut words,
                SUBCHANNEL,
                SET_SHADER_SHARED_MEMORY_WINDOW_A,
                &hi_lo(crate::compute::SHARED_WINDOW),
            );
            method(
                &mut words,
                SUBCHANNEL,
                SET_SHADER_LOCAL_MEMORY_WINDOW_A,
                &hi_lo(crate::compute::LOCAL_WINDOW),
            );
            method(&mut words, SUBCHANNEL, INVALIDATE_SKED_CACHES, &[0]);
        }
        // Constant cache every launch: launch slots and their cbufs are reused at the same
        // addresses. Instruction cache after programs were (re)written: the code reached the
        // GPU by the barrier below, before this submission.
        let instruction = u32::from(self.code_dirty);
        method(&mut words, SUBCHANNEL, INVALIDATE_SHADER_CACHES_NO_WFI, &[instruction | 1 << 12]);
        method(&mut words, SUBCHANNEL, SEND_PCAS_A, &[(base >> 8) as u32]);
        if self.compute_class <= 0xc5c0 {
            method(&mut words, SUBCHANNEL, SEND_SIGNALING_PCAS_B, &[3]);
        } else {
            method(&mut words, SUBCHANNEL, SEND_SIGNALING_PCAS2_B, &[3]);
        }

        let slot = &mut self.slots[si].0;
        for (i, w) in qmd.iter().enumerate() {
            slot.cpu.write_u32(i * 4, *w)?;
        }
        for (i, c) in cbuf.chunks(4).enumerate() {
            slot.cpu.write_u32(CBUF_OFFSET + i * 4, u32::from_le_bytes(c.try_into().unwrap()))?;
        }
        for (i, w) in words.iter().enumerate() {
            slot.cpu.write_u32(WORDS_OFFSET + i * 4, *w)?;
        }
        crate::platform::device_full_barrier();
        let push = Push {
            address: base + WORDS_OFFSET as u64,
            size: (words.len() * 4) as u32,
            flags: 0,
        };
        let token = match self.queue.submit(&[push], space, &[], &[], &[], |_| Ok(()))? {
            SubmitStatus::Submitted(t) => t,
            SubmitStatus::Full(_) => return Err(VideoError::Again("compute queue full".into())),
        };
        self.slots[si].1 = token;
        self.initialized = true;
        self.code_dirty = false;
        Ok(token)
    }

    /// Returns once launch `token` finished, or the channel's error (a faulting kernel kills the
    /// channel: its completion never comes, the error notifier says why).
    pub(crate) fn wait(&mut self, drm: i32, token: u64) -> VResult<()> {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            self.queue.check_error().map_err(|e| VideoError::DeviceLost(e.to_string()))?;
            if self.queue.shared().retired(token)? {
                return Ok(());
            }
            let now = std::time::Instant::now();
            if now >= deadline {
                return Err(VideoError::DeviceLost(format!("compute launch {token} timed out")));
            }
            let fence = self.queue.context_fence(drm, token)?;
            // A fence of a dead channel never signals: wake up to look at the notifier.
            let _ = wait_sync_file(
                std::os::fd::AsRawFd::as_raw_fd(&fence),
                (deadline - now).min(Duration::from_millis(50)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::kernels::{ABI, KERNELS, QMD_SAMPLES, TEST_KERNELS};

    fn test_kernel(name: &str, sm: u32) -> &'static Kernel {
        TEST_KERNELS.iter().find(|k| k.name == name && k.sm == sm).unwrap()
    }

    /// The runtime packing reproduces Mesa's nak_fill_qmd for every shader model and QMD
    /// version (2.2 Turing, 3.0 Ampere/Ada, 4.0 Hopper, 5.0 Blackwell).
    #[test]
    fn qmd_packing_matches_nak_fill_qmd() {
        assert_eq!(ABI, 1);
        let mut models = std::collections::BTreeSet::new();
        for &(name, sm, values, expected) in QMD_SAMPLES {
            let qmd = test_kernel(name, sm).pack(values).unwrap();
            assert_eq!(qmd, expected, "{name} sm_{sm} {values:x?}");
            models.insert(sm);
        }
        assert_eq!(models.into_iter().collect::<Vec<_>>(), [75, 80, 86, 89, 90, 120]);
        for k in KERNELS.iter().chain(TEST_KERNELS) {
            assert_eq!(sm_of_class(k.compute_class), Some(k.sm), "{}", k.name);
            assert!(k.code.len() % 16 == 0 && !k.code.is_empty(), "{} sm_{}", k.name, k.sm);
        }
    }

    #[test]
    fn qmd_packing_refuses_values_without_a_field() {
        let k = test_kernel("fill", 89);
        assert!(k.pack([0x1000, 0x2000, 1, 1, 1]).is_ok());
        assert!(k.pack([0x1080, 0x2000, 1, 1, 1]).is_err(), "program not 256-byte aligned");
        assert!(k.pack([0x1000, 0x2000, 1, 1 << 16, 1]).is_err(), "grid height beyond 16 bits");
        assert!(k.pack([1 << 49, 0x2000, 1, 1, 1]).is_err(), "address beyond 49 bits");
    }

    fn read(buf: &VramBuffer, words: usize) -> Vec<u32> {
        (0..words).map(|i| buf.cpu.read_u32(i * 4).unwrap()).collect()
    }

    fn push(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    fn rss_and_fds() -> (u64, usize) {
        let status = std::fs::read_to_string("/proc/self/status").unwrap();
        let rss = status
            .lines()
            .find_map(|l| l.strip_prefix("VmRSS:"))
            .and_then(|v| v.trim().trim_end_matches(" kB").parse().ok())
            .unwrap();
        (rss, std::fs::read_dir("/proc/self/fd").unwrap().count())
    }

    /// Hardware acceptance of the engine (stage 2): a kernel writes a known pattern; two different
    /// programs alternate on one channel, also at the same program address after the code heap
    /// is rewritten, with new push constants on every launch; 10000 launches leave RSS, fds and
    /// the launch slots stable; a kernel faulting on an unmapped address yields an error.
    #[test]
    #[ignore = "local NVIDIA GPU: compute channel launches of the generated kernels"]
    fn engine_on_hardware() {
        use crate::video::venc::NvVencDevice;
        use std::os::fd::AsRawFd;
        let rm = NvVencDevice::open_on(None).unwrap();
        let drm = rm.drm.as_raw_fd();
        let mut engine = ComputeEngine::new(&rm.device, &rm.space).unwrap();
        let fill = engine.kernel("fill").unwrap();
        let mix = engine.kernel("mix").unwrap();
        const N: usize = 4096;
        let out = vram(&rm.device, &rm.space, N * 4).unwrap();
        let bind = |b| (b, out.gpu.address(), (N * 4) as u32);
        let fill_expect = |count: usize, seed: u32, step: u32, prev: &[u32]| -> Vec<u32> {
            (0..N).map(|i| if i < count { seed.wrapping_add((i as u32).wrapping_mul(step)) } else { prev[i] }).collect()
        };
        let mix_expect = |count: usize, seed: u32, prev: &[u32]| -> Vec<u32> {
            (0..N).map(|i| if i < count { (seed ^ i as u32).wrapping_mul(3).wrapping_add(prev[i]) } else { prev[i] }).collect()
        };

        // known pattern, partial last workgroup
        let t = engine.launch(&rm.space, fill, &[bind(0)], &push(&[3000, 7, 5]), [(3000u32).div_ceil(64), 1, 1]).unwrap();
        engine.wait(drm, t).unwrap();
        let mut want = fill_expect(3000, 7, 5, &vec![0; N]);
        assert_eq!(read(&out, N), want, "fill");

        // two programs alternating, new push constants each launch
        for round in 0..20u32 {
            let seed = round.wrapping_mul(0x9e37_79b9);
            let t = if round % 2 == 0 {
                want = fill_expect(N, seed, round + 1, &want);
                engine.launch(&rm.space, fill, &[bind(0)], &push(&[N as u32, seed, round + 1]), [64, 1, 1]).unwrap()
            } else {
                want = mix_expect(N, seed, &want);
                // 64 x 64 elements as an 8 x 8 grid of 8x8 workgroups
                engine.launch(&rm.space, mix, &[bind(1)], &push(&[N as u32, seed, 64]), [8, 8, 1]).unwrap()
            };
            engine.wait(drm, t).unwrap();
            assert_eq!(read(&out, N), want, "alternating round {round}");
        }

        // the same program address holding the other program after the heap is rewritten
        let mut address = None;
        for round in 0..6u32 {
            engine.reset_code(drm).unwrap();
            let (kernel, seed) = if round % 2 == 0 { (fill, round + 100) } else { (mix, round + 200) };
            let t = if round % 2 == 0 {
                want = fill_expect(N, seed, 3, &want);
                engine.launch(&rm.space, kernel, &[bind(0)], &push(&[N as u32, seed, 3]), [64, 1, 1]).unwrap()
            } else {
                want = mix_expect(N, seed, &want);
                engine.launch(&rm.space, kernel, &[bind(1)], &push(&[N as u32, seed, 64]), [8, 8, 1]).unwrap()
            };
            let here = engine.programs[kernel.name];
            assert_eq!(*address.get_or_insert(here), here, "program moved");
            engine.wait(drm, t).unwrap();
            assert_eq!(read(&out, N), want, "rewritten program round {round}");
        }

        // 10000 launches: process and engine resources stay flat
        let before = rss_and_fds();
        let mut last = 0;
        for i in 0..10000u32 {
            last = engine.launch(&rm.space, fill, &[bind(0)], &push(&[64, i, 1]), [1, 1, 1]).unwrap();
            if i % 64 == 63 {
                engine.wait(drm, last).unwrap();
            }
        }
        engine.wait(drm, last).unwrap();
        let after = rss_and_fds();
        let slots = engine.slots.len();
        eprintln!("10000 launches: RSS {} -> {} kB, fds {} -> {}, launch slots {slots}", before.0, after.0, before.1, after.1);
        assert!(after.0 < before.0 + 4096, "RSS grew");
        assert_eq!(after.1, before.1, "fds leaked");
        assert!(slots <= 70, "launch slots grew to {slots}");
        assert_eq!(read(&out, 64), (0..64).map(|i| 9999 + i).collect::<Vec<u32>>());

        // a kernel storing to an unmapped address kills the channel: an error, not a hang
        let unmapped = 0xf0_0000_0000u64; // inside the 1 TiB address space, never mapped
        let t = engine.launch(&rm.space, fill, &[(0, unmapped, 4096)], &push(&[1024, 1, 1]), [16, 1, 1]).unwrap();
        let started = std::time::Instant::now();
        let error = engine.wait(drm, t).unwrap_err();
        eprintln!("faulting launch: {error} after {:?}", started.elapsed());
        assert!(matches!(error, VideoError::DeviceLost(_)));
    }
}
