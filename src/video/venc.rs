//! C ABI for the NVENC encoder core (video/nv_venc.h): the RM services it needs
//! (memory with CPU and GPU mappings, a channel on NVENC0 with its engine
//! object, submission and completion) on top of this crate's RM client. The
//! picture setups and methods are the core's; this side only owns kernel
//! objects and their lifetimes.
use super::{
    channel::{EngineChannel, NV2080_ENGINE_TYPE_NVENC0},
    ffi::{guard, null},
    open_render_node, open_rm, VResult, VideoError,
};
use crate::{
    gpu::{AddressSpace, GpuMapping},
    rm::{Device, Mapping, Memory},
};
use std::{collections::HashMap, rc::Rc};

struct Allocation {
    // Field order is drop order: CPU view, then GPU view, then the memory.
    _cpu: Option<Mapping>,
    _gpu: GpuMapping,
    _memory: Rc<Memory>,
}

pub struct NvVencDevice {
    // Drop order: channels and allocations before the address space and device.
    channels: HashMap<u64, EngineChannel>,
    allocations: HashMap<u64, Allocation>,
    space: AddressSpace,
    drm: std::fs::File,
    device: Rc<Device>,
    classes: Vec<u32>,
    next: u64,
}
// SAFETY: a device's RM objects are only reachable through it; nv_venc.h
// requires callers to serialise each device, so moving it between threads
// never shares its Rc graph concurrently.
unsafe impl Send for NvVencDevice {}

#[repr(C)]
pub struct NvVencMemory {
    pub id: u64,
    pub size: u64,
    pub cpu: *mut std::ffi::c_void,
    pub gpu: u64,
}

/// Newest class of a kind: NVENC classes end in 0xb7, NVDEC classes in 0xb0.
fn newest(classes: &[u32], low: u32) -> u32 {
    classes
        .iter()
        .copied()
        .filter(|c| c & 0xff == low && *c >= 0xb000 && *c < 0x10000)
        .max()
        .unwrap_or(0)
}

impl NvVencDevice {
    fn open() -> VResult<Self> {
        let (device, pci) = open_rm(None)?;
        let classes = device.classes()?;
        if newest(&classes, 0xb7) == 0 {
            return Err(VideoError::Unsupported("this GPU advertises no NVENC class".into()));
        }
        let drm = open_render_node(pci)?;
        let space = AddressSpace::new(&device)?;
        Ok(Self {
            channels: HashMap::new(),
            allocations: HashMap::new(),
            space,
            drm,
            device,
            classes,
            next: 1,
        })
    }

    fn id(&mut self) -> u64 {
        let id = self.next;
        self.next += 1;
        id
    }

    fn alloc(&mut self, size: u64, align: u64, system: bool, cpu: bool) -> VResult<NvVencMemory> {
        let size = usize::try_from(size).map_err(|_| VideoError::Invalid("size".into()))?;
        let align = usize::try_from(align.max(4096))
            .map_err(|_| VideoError::Invalid("alignment".into()))?;
        let memory = self.device.allocate_aligned(size, align, false, system)?;
        let gpu = self.space.map(&memory)?;
        let mapping = if cpu { Some(memory.map()?) } else { None };
        let out = NvVencMemory {
            id: self.id(),
            size: memory.size as u64,
            cpu: mapping
                .as_ref()
                .map_or(std::ptr::null_mut(), |m| m.cpu_address() as *mut _),
            gpu: gpu.address(),
        };
        self.allocations.insert(
            out.id,
            Allocation {
                _cpu: mapping,
                _gpu: gpu,
                _memory: memory,
            },
        );
        Ok(out)
    }

    fn channel(&mut self, id: u64) -> VResult<&mut EngineChannel> {
        self.channels
            .get_mut(&id)
            .ok_or_else(|| VideoError::Invalid(format!("no NVENC channel {id}")))
    }
}

fn dev<'a>(dev: *mut NvVencDevice) -> Result<&'a mut NvVencDevice, VideoError> {
    null(dev, "dev")?;
    // SAFETY: checked non-NULL; the caller serialises use of the device.
    Ok(unsafe { &mut *dev })
}

/// # Safety
/// `out` writable.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_open(out: *mut *mut NvVencDevice) -> i32 {
    guard(|| {
        null(out, "out")?;
        let d = NvVencDevice::open()?;
        // SAFETY: checked non-NULL.
        unsafe { out.write(Box::into_raw(Box::new(d))) };
        Ok(0)
    })
}

/// # Safety
/// `dev` from nv_venc_open (or NULL), not used afterwards.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_close(dev: *mut NvVencDevice) {
    if !dev.is_null() {
        // SAFETY: ownership returns from the caller.
        drop(unsafe { Box::from_raw(dev) });
    }
}

/// Newest class of `kind`: 0 NVENC, 1 NVDEC, 2 host channel. 0 = none or error.
///
/// # Safety
/// Valid device.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_class(d: *mut NvVencDevice, kind: u32) -> u32 {
    let mut class = 0u32;
    guard(|| {
        let d = dev(d)?;
        class = match kind {
            0 => newest(&d.classes, 0xb7),
            1 => newest(&d.classes, 0xb0),
            2 => crate::capabilities::query(&d.device)?.channel,
            _ => 0,
        };
        Ok(0)
    });
    class
}

/// Allocate `size` bytes of system (`system` != 0) or video memory, mapped
/// into the device's GPU address space and, with `cpu` != 0, into the process.
///
/// # Safety
/// Valid device; `out` writable.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_mem_alloc(
    d: *mut NvVencDevice,
    size: u64,
    align: u64,
    system: u32,
    cpu: u32,
    out: *mut NvVencMemory,
) -> i32 {
    guard(|| {
        null(out, "out")?;
        let m = dev(d)?.alloc(size, align, system != 0, cpu != 0)?;
        // SAFETY: checked non-NULL.
        unsafe { out.write(m) };
        Ok(0)
    })
}

/// # Safety
/// Valid device; `id` from nv_venc_mem_alloc, its mappings unused afterwards.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_mem_free(d: *mut NvVencDevice, id: u64) {
    guard(|| {
        dev(d)?.allocations.remove(&id);
        Ok(0)
    });
}

/// A channel on NVENC0 with an engine object of `class`, bound on subchannel 0.
///
/// # Safety
/// Valid device; `out` writable.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_channel_create(
    d: *mut NvVencDevice,
    class: u32,
    out: *mut u64,
) -> i32 {
    guard(|| {
        null(out, "out")?;
        let d = dev(d)?;
        let ch = EngineChannel::new(&d.device, &d.space, NV2080_ENGINE_TYPE_NVENC0, class, 0)?;
        let id = d.id();
        d.channels.insert(id, ch);
        // SAFETY: checked non-NULL.
        unsafe { out.write(id) };
        Ok(0)
    })
}

/// # Safety
/// Valid device.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_channel_destroy(d: *mut NvVencDevice, channel: u64) {
    guard(|| {
        dev(d)?.channels.remove(&channel);
        Ok(0)
    });
}

/// Submit `count` method words; `*value` receives the completion timeline
/// value of this submission.
///
/// # Safety
/// Valid device; `words` readable for `count` words; `value` writable.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_channel_submit(
    d: *mut NvVencDevice,
    channel: u64,
    words: *const u32,
    count: usize,
    value: *mut u64,
) -> i32 {
    guard(|| {
        null(words, "words")?;
        null(value, "value")?;
        // SAFETY: caller guarantees `count` readable words.
        let w = unsafe { std::slice::from_raw_parts(words, count) }.to_vec();
        let v = dev(d)?.channel(channel)?.submit(w)?;
        // SAFETY: checked non-NULL.
        unsafe { value.write(v) };
        Ok(0)
    })
}

/// Last completed timeline value of the channel.
///
/// # Safety
/// Valid device; `value` writable.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_channel_completed(
    d: *mut NvVencDevice,
    channel: u64,
    value: *mut u64,
) -> i32 {
    guard(|| {
        null(value, "value")?;
        let v = dev(d)?.channel(channel)?.completed()?;
        // SAFETY: checked non-NULL.
        unsafe { value.write(v) };
        Ok(0)
    })
}

/// Block (sync_file, no spinning) until `value` completed; a hung engine
/// fails after 2 s.
///
/// # Safety
/// Valid device.
#[no_mangle]
pub unsafe extern "C" fn nv_venc_channel_wait(d: *mut NvVencDevice, channel: u64, value: u64) -> i32 {
    guard(|| {
        let d = dev(d)?;
        let ch = d
            .channels
            .get(&channel)
            .ok_or_else(|| VideoError::Invalid(format!("no NVENC channel {channel}")))?;
        ch.wait(&d.drm, value)?;
        Ok(0)
    })
}
