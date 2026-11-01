//! NVENC encoder core in Rust: a port of mickys/nvenc (MIT, commit 90d1d6a),
//! driven through this crate's RM services directly.
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
//!
//! `EncConfig::min_qp` / `max_qp` optionally constrain native RC quantizers
//! (H.264/HEVC 0..51, AV1 0..255), through the driver's per-picture minQP/maxQP
//! arrays. Both default to `None`, retaining the original C core's setup bytes.
//!
//! Client-packed mode uses [`packed::ParameterSets`] to own/validate Annex-B parameter
//! NALs or sized AV1 sequence OBUs. Select the picture's PPS with `syntax_for_picture`,
//! install it with [`Encoder::set_packed_syntax`] after draining, then enable
//! [`Encoder::set_frame_only`]. Supply [`packed::PackedPicture`] before each submit.
//! H.264/HEVC counters are unwrapped codec POC; AV1 expands the client's actual hints
//! through skips/wrap, independently of request counts. Client parameter syntax obeys
//! supplied IDRs; metadata-only mode retains native periodic IDRs.
//! `wait_view` returns the entire picture's Annex-B VCL or AV1 frame OBU with all tiles;
//! no generated parameter sets, temporal delimiter or HDR metadata is added in this mode.
//! The caller orders AUD/TD, its original parameter sets, prefix metadata, all frame
//! data, and suffix metadata. Codec2 and native callers which do not enable the mode
//! retain generated headers/metadata and the existing C ABI.
//! Independent sequence/picture masks use [`packed::PackedDefaults::for_config`] as
//! the base for VA semantic overrides, then `ParameterSets::complete_unrequested`
//! returns only the driver-owned headers. Keep the bank staged until picture success.
//! Client sets retain ownership regardless of their enclosing class: a PPS inside
//! SEQUENCE is not replaced merely because the PICTURE mask bit was not requested.
//! `unescaped_parameter_len` locates parameter NAL boundaries with the codec parsers,
//! including raw timing bytes containing start-code patterns. Syntax replacement
//! moves ownership and can restore `None`; failed submissions discard the queued
//! picture and restore pre-submit counters without discarding reference frames.
//! HEVC 4:2:0 and 4:4:4 at 8/10 bits share the setup contract. VPS/SPS clocks and
//! fixed-rate single-CPB HRD constrain hardware bitrate, CPB capacity and frame clock;
//! unsupported HRD modes are rejected before submission.
//! Persistent NAL/VCL CPB occupancy follows effective capacities transactionally;
//! unchanged parameter budgets preserve accumulated RC history. HEVC TMVP is not
//! part of the implemented reference model and is rejected before submission.
//! Client HEVC minimum CU must match the selected controls: 8 only with `quality`
//! and without `no_cu8`, otherwise 16. A mismatched SPS is rejected before submit;
//! installing packed syntax does not silently select another quality/template mode.
//! HEVC coded width/height must fill 32x32 CTBs; the original conformance window
//! must crop them to the session's visible extent. Partial-CTB syntax is rejected.
//! HEVC hardware uses a QP baseline of 26; frame output translates slice deltas to
//! the client PPS while preserving absolute QP and the CABAC-coded suffix.
//!
//! [`Encoder::load_dmabuf_rgb`] ingests B,G,R,A; R,G,B,A; or A,R,G,B byte order with
//! GPU conversion. [`ingest::check_plane`] checks complete linear/tiled allocation
//! extents before import; no CPU channel swap is needed for alpha-first RGB.
pub mod av1hdr;
pub mod av1probs;
mod av1tables;
mod encoder;
mod error;
pub use error::EncError;
pub mod h264hdr;
pub mod hevchdr;
mod hevcslice;
pub mod ingest;
pub mod layout;
pub mod packed;
pub mod nvenc_drv;
pub mod tmpl_av1;
pub mod tmpl_h264;
pub mod tmpl_hevc;
pub use encoder::{
    EncConfig, EncFrame, EncFrameView, Encoder, ENC_AV1, ENC_H264, ENC_HEVC, ENC_MAX_DEPTH, ENC_MAX_REFS,
    ENC_MAX_SLICES,
};
