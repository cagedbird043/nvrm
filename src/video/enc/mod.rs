//! NVENC encoder core in Rust: a port of mickys/nvenc (MIT, commit 90d1d6a) as vendored in
//! `nvidia-prolapse/media/core/nvenc/mickys/`, driven through this crate's RM services directly.
//!
//! [`Encoder::open_on`] selects a PCI GPU (domain, bus, device, function), using
//! the same device-selection rules as the decoder. Load the next input surface,
//! then use `encode` synchronously or `submit` / `wait` for up to three frames in
//! flight. Wait tickets in submission order before reusing their input slots.
//! Completion sleeps on the engine channel's timeline fence, without polling.
//! `headers`, `av1_config`, `wait`, and `av1_show_existing` append to reusable
//! caller-owned byte vectors. Input-plane strides are in bytes, including P010
//! and X444 (little-endian 16-bit samples with the ten bits in the high bits).
//! Only NVENC0 is exposed, matching the host C core's RM shim.
//! [`Encoder::ext_input`] accepts an input GPU VA in the session address space.
//! `ENC_DUMP_SETUP=DIR` writes each complete 4096-byte setup as `encN.bin`.
//!
//! Host comparison CLI: `nv-venc-test [-hevc|-av1] [-cbr KBPS|-vbr KBPS]
//! [-qp QP] [-gop N] [-fps F] [-profile 66|77|100] [-level IDC] W H INPUT OUTPUT`.
//! Additional matrix switches: `-10` (P010), `-444` (planar 8-bit 4:4:4,
//! widened for `-10`), `-bf N`, `-refs N`, `-depth N`, `-slices N`, `-aq N`,
//! `-quality`, `-two-pass`, `-lossless`, `-no-cu8`, `-vbv KBITS`, `-maxrate KBPS`,
//! `-colour P T M`, `-mdcv HEX48`, `-cll HEX8`, `-full-range`, and `-pci BDF`.
pub mod av1hdr;
pub mod av1probs;
mod av1tables;
mod encoder;
pub mod h264hdr;
pub mod hevchdr;
pub mod layout;
pub mod nvenc_drv;
pub mod tmpl_av1;
pub mod tmpl_h264;
pub mod tmpl_hevc;
pub use encoder::{
    EncConfig, EncFrame, Encoder, ENC_AV1, ENC_H264, ENC_HEVC, ENC_MAX_DEPTH, ENC_MAX_REFS,
    ENC_MAX_SLICES,
};
