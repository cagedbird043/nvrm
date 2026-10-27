#ifndef NV_VENC_H
#define NV_VENC_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

/* Existing low-level ABI: one serialized device owner; negative error codes
 * and diagnostic nv_video_last_error(). Not session status codes below. */
typedef struct NvVencDevice NvVencDevice;
typedef struct NvVencMemory {
    uint64_t id;
    uint64_t size;
    void *cpu;
    uint64_t gpu;
} NvVencMemory;
int32_t nv_venc_open(NvVencDevice **out);
void nv_venc_close(NvVencDevice *dev);
/* kind: 0 NVENC, 1 NVDEC, 2 host channel; newest class, zero if absent. */
uint32_t nv_venc_class(NvVencDevice *dev, uint32_t kind);
int32_t nv_venc_mem_alloc(NvVencDevice *dev, uint64_t size, uint64_t align,
                         uint32_t system, uint32_t cpu, NvVencMemory *out);
void nv_venc_mem_free(NvVencDevice *dev, uint64_t id);
int32_t nv_venc_channel_create(NvVencDevice *dev, uint32_t cls, uint64_t *channel);
void nv_venc_channel_destroy(NvVencDevice *dev, uint64_t channel);
int32_t nv_venc_channel_submit(NvVencDevice *dev, uint64_t channel,
                              const uint32_t *words, size_t count, uint64_t *value);
int32_t nv_venc_channel_completed(NvVencDevice *dev, uint64_t channel, uint64_t *value);
int32_t nv_venc_channel_wait(NvVencDevice *dev, uint64_t channel, uint64_t value);
/* Thread-local diagnostic, borrowed until next error on this thread.
 * Copy immediately; not session-owned. Shared with the decode ABI. */
const char *nv_video_last_error(void);

typedef struct NvVencSess NvVencSess;
enum NvVencSessStatus {
    NV_VENC_SESS_OK = 0,
    NV_VENC_SESS_RECOVERABLE = -1,
    NV_VENC_SESS_TERMINAL = -2
};
enum NvVencCodec {
    NV_VENC_CODEC_H264 = 0,
    NV_VENC_CODEC_HEVC = 1,
    NV_VENC_CODEC_AV1 = 2
};
enum NvVencRc {
    NV_VENC_RC_CQP = 0,
    NV_VENC_RC_VBR = 1,
    NV_VENC_RC_CBR = 2
};
enum NvVencPixelFormat {
    NV_VENC_FMT_NV12 = 0,
    NV_VENC_FMT_NV21 = 1,
    NV_VENC_FMT_I420 = 2,
    NV_VENC_FMT_RGBX = 3,
    NV_VENC_FMT_BGRX = 4
};
enum NvVencAbi { NV_VENC_SESS_ABI_VERSION = 1 };

/* All structs use native ABI alignment, no packing; zero initialize before use.
 * abi_version=1 and struct_size=sizeof(NvVencSessCfg) are required.
 * Unknown enums/flags or nonzero reserved entries are rejected.
 * No pointer in config is retained because config has no pointer fields. */
typedef struct NvVencSessCfg {
    uint32_t abi_version;
    uint32_t struct_size;
    uint32_t pci_present; /* 0 first usable GPU; 1 select exact PCI BDF */
    uint32_t pci_domain;
    uint32_t pci_bus;
    uint32_t pci_device;
    uint32_t pci_function;
    uint32_t codec;       /* NvVencCodec, NOT NV_VIDEO_CODEC_* */
    uint32_t width;       /* positive, even, within advertised caps */
    uint32_t height;      /* positive, even, within advertised caps */
    uint32_t h264_profile;/* H264: 66,77,100; other codecs: zero */
    uint32_t h264_level;  /* H264 level_idc; supported table; otherwise zero */
    uint32_t cabac;       /* H264 0/1, must be 0 for Baseline; others zero */
    uint32_t rc_mode;     /* NvVencRc */
    uint32_t bitrate;     /* RC: 1..200000000 bits/s; CQP: ignored */
    uint32_t max_bitrate; /* RC: bitrate..200000000 bits/s; CQP: ignored */
    uint32_t vbv_bits;    /* RC: 1..200000000 bits; CQP: ignored */
    uint32_t fps;         /* integer pictures/s, 1..65535 */
    uint32_t idr_period;  /* 0..134217728 pictures; 0 disables periodic IDR */
    uint32_t full_range;  /* 0 limited, 1 full */
    uint32_t primaries;   /* H.273 code point 0..255 */
    uint32_t transfer;    /* H.273 code point 0..255 */
    uint32_t matrix;      /* H.273 code point 0..255 */
    uint32_t reserved[4]; /* must be zero */
} NvVencSessCfg;

/* All four fields replace the previous values atomically. No zero-means-keep.
 * Same ranges as cfg; CQP rate fields are ignored, fps/period still apply. */
typedef struct NvVencSessUpdate {
    uint32_t bitrate;
    uint32_t max_bitrate;
    uint32_t vbv_bits;
    uint32_t fps;
    uint32_t idr_period;
} NvVencSessUpdate;

/* data points at the first logical sample of this plane, not allocation base.
 * Used planes are borrowed and readable until encode returns; no FD or pointer
 * is retained. length counts accessible bytes from data; stride is bytes/row.
 * Caller guarantees actual allocation validity and no concurrent producer writes.
 * Rust checks stride >= logical row bytes and checked last-row extent <= length
 * before any slice/load. Unused planes must be {NULL,0,0}. */
typedef struct NvVencPlane {
    const uint8_t *data;
    uint64_t length;
    uint64_t stride;
} NvVencPlane;

/* All output fields are written by encode; zeroed on every failure.
 * data is Rust-owned, read-only, contiguous Annex-B H264/HEVC or AV1 TU bytes.
 * Never free it. Valid only until entry to the next encode/update/codec_config/
 * destroy on this session, INCLUDING failing calls. Copy into a C2-owned block
 * before that call. Calls on other sessions and last_error do not invalidate it.
 * engine_ns is elapsed submission-to-completion time, not CPU input upload time. */
typedef struct NvVencOut {
    const uint8_t *data;
    uint64_t length;
    uint64_t engine_ns;
    uint32_t key;        /* 0/1 */
    uint32_t avg_qp;
    uint32_t bit_count;  /* hardware bits, excluding added software headers */
    uint32_t frame_num;
    uint32_t poc;
    uint32_t reserved;   /* zero */
} NvVencOut;

/* Threading/ownership for ALL session calls:
 * Caller serializes calls and all reads of output views. No reentrancy.
 * Sessions may be moved between threads only with a happens-before handoff;
 * destroying on another thread is allowed after the prior worker has joined.
 * Session is not a shared concurrent object; distinct sessions are independent.
 * All exported calls catch unwinding panics; existing session is then TERMINAL.
 * C pointer validity/lifetime violations and process OOM abort are not catchable.
 * After TERMINAL only destroy is legal; other calls return TERMINAL immediately.
 * No call invokes a caller callback or retains an input pointer.
 */

/* Synchronous create; copies cfg, owns a new independent RM device; never accepts
 * a decode device handle. On every failure *out=NULL. Failure returns TERMINAL.
 * Uses NVENC0, 8-bit 420, refs=1, depth=1, bframes=0, no AQ/two-pass/HDR.
 * CQP defaults: H264/HEVC I=26 P=28, AV1 I=90 P=110. */
int32_t nv_venc_sess_create(const NvVencSessCfg *cfg, NvVencSess **out);

/* Invalidate previous view, validate whole update, commit for next picture.
 * RECOVERABLE invalid update leaves config unchanged, consumes no picture,
 * does not force IDR. Preserve RC history; no implicit reopen or CSD rewrite. */
int32_t nv_venc_sess_update(NvVencSess *sess, const NvVencSessUpdate *update);

/* Synchronous load+submit+wait, exactly one configured-size picture.
 * format: NvVencPixelFormat; force_key: 0/1.
 * NV12/NV21 planes: Y(w*h), interleaved UV/VU(w*h/2), plane2 unused.
 * I420 planes: Y(w*h), U((w/2)*(h/2)), V(same).
 * RGBX/BGRX: plane0 has width*4 bytes/row; planes1/2 unused; X ignored.
 * Extents above describe logical rows; caller stride/padding are respected.
 * On RECOVERABLE: no output, discard input; next successfully submitted picture
 * is forced IDR. On TERMINAL: destroy session, never resubmit into this channel.
 */
int32_t nv_venc_sess_encode(NvVencSess *sess, const NvVencPlane planes[3],
                           uint32_t format, uint32_t force_key, NvVencOut *out);

/* Copy codec configuration: H264 SPS/PPS, HEVC VPS/SPS/PPS (Annex B),
 * AV1 sequence OBU (NOT 4-byte av1C prefix). C++ constructs av1C as today.
 * Always invalidate old output view. len_out must be valid/writable.
 * buf=NULL, capacity=0 is a successful size query; sets required length.
 * Too-small capacity returns RECOVERABLE and sets required length, writes no
 * partial data, consumes no picture and does not force IDR. Other failures set
 * *len_out=0. Successful non-query call writes exactly *len_out bytes.
 */
int32_t nv_venc_sess_codec_config(NvVencSess *sess, uint8_t *buf,
                                 uint64_t capacity, uint64_t *len_out);

/* NULL is allowed. Destroys exactly once; invalidates all views. Never use sess
 * again. Finishes/tears down queued work and releases owned mappings/resources;
 * thread handoff/serialization rules also apply to a poisoned session. */
void nv_venc_sess_destroy(NvVencSess *sess);

/* Pure C2 helpers: no retained pointers/session state, no view invalidation.
 * Return OK/RECOVERABLE and write outputs only on success. Use last_error;
 * caught panic returns TERMINAL. All caller output pointers must be valid. */
/* Binary64 bits of the C2 double rate, copied without numeric conversion.
 * Reject nonfinite/nonpositive fps; round nearest, minimum 1; reject >65535.
 * Negative interval or INT64_MAX -> period zero. Otherwise round
 * interval_us*raw_fps/1e6, clamp to 1..134217728 before integer conversion. */
int32_t nv_venc_c2_timing(uint64_t frame_rate_f64_bits, int64_t sync_interval_us,
                         uint32_t *fps, uint32_t *idr_period);
/* Checked sum, also require representable by C2 uint32_t linear capacity. */
int32_t nv_venc_c2_output_size(uint64_t frame_bytes, uint64_t prepend_bytes,
                             uint32_t *total);
enum NvVencC2LayoutType {
    NV_VENC_C2_YUV = 0, NV_VENC_C2_RGB = 1, NV_VENC_C2_RGBA = 2
};
/* Explicitly translate C2 enums, never memcpy packed C2PlaneInfo.
 * Slot order: Y,U,V for YUV; R,G,B,A for RGB/RGBA. */
typedef struct NvVencC2Plane {
    const uint8_t *data;
    int32_t row_inc;
    int32_t col_inc;
    uint32_t col_sampling;
    uint32_t row_sampling;
    uint32_t allocated_depth;
    uint32_t bit_depth;
    uint32_t right_shift;
    uint32_t root_index;
    uint32_t root_offset;
} NvVencC2Plane;
typedef struct NvVencC2View {
    uint32_t type;       /* NvVencC2LayoutType */
    uint32_t num_planes;
    uint32_t root_planes;
    uint32_t width;      /* mapped view capacity, not requested size */
    uint32_t height;
    uint32_t crop_left;
    uint32_t crop_top;
    uint32_t crop_width;
    uint32_t crop_height;
    NvVencC2Plane planes[4];
} NvVencC2View;
/* No pixel dereference. Validate the successful live C2 mapping's descriptors,
 * origin crop, dimensions, positive strides, 8-bit sample/depth/root relations
 * and checked address extents; recognize exactly the five session formats.
 * Return minimum extents containing requested samples, not row-padding extents.
 * Caller guarantees the successful allocator mapping really backs every sample
 * in crop; this cannot validate fabricated allocator addresses. C++ must keep
 * C2GraphicView alive throughout helper and subsequent sess_encode.
 * Non-origin crop and unsupported layouts return RECOVERABLE; C++ maps invalid
 * input to its existing fatal C2_BAD_VALUE component policy. */
int32_t nv_venc_c2_input(const NvVencC2View *view, uint32_t width, uint32_t height,
                        uint32_t *format, NvVencPlane planes[3]);
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
#define NV_VENC_ASSERT static_assert
#define NV_VENC_ALIGN alignof
#else
#define NV_VENC_ASSERT _Static_assert
#define NV_VENC_ALIGN _Alignof
#endif
NV_VENC_ASSERT(sizeof(NvVencSessCfg) == 108, "session config ABI size");
NV_VENC_ASSERT(NV_VENC_ALIGN(NvVencSessCfg) == 4, "session config ABI alignment");
NV_VENC_ASSERT(offsetof(NvVencSessCfg, reserved) == 92, "session config ABI offset");
NV_VENC_ASSERT(sizeof(NvVencSessUpdate) == 20, "session update ABI size");
NV_VENC_ASSERT(offsetof(NvVencSessUpdate, idr_period) == 16, "session update ABI offset");
#if UINTPTR_MAX == UINT64_MAX
NV_VENC_ASSERT(sizeof(NvVencPlane) == 24 && NV_VENC_ALIGN(NvVencPlane) == 8, "plane ABI layout");
NV_VENC_ASSERT(offsetof(NvVencPlane, length) == 8, "plane ABI offset");
NV_VENC_ASSERT(sizeof(NvVencOut) == 48 && NV_VENC_ALIGN(NvVencOut) == 8, "output ABI layout");
NV_VENC_ASSERT(offsetof(NvVencOut, key) == 24, "output ABI offset");
NV_VENC_ASSERT(sizeof(NvVencC2Plane) == 48 && NV_VENC_ALIGN(NvVencC2Plane) == 8, "C2 plane ABI layout");
NV_VENC_ASSERT(offsetof(NvVencC2Plane, row_inc) == 8, "C2 plane ABI offset");
NV_VENC_ASSERT(sizeof(NvVencC2View) == 232 && NV_VENC_ALIGN(NvVencC2View) == 8, "C2 view ABI layout");
NV_VENC_ASSERT(offsetof(NvVencC2View, planes) == 40, "C2 view ABI offset");
#else
NV_VENC_ASSERT(sizeof(NvVencPlane) == (NV_VENC_ALIGN(uint64_t) == 8 ? 24 : 20), "plane ABI size");
NV_VENC_ASSERT(sizeof(NvVencOut) == (NV_VENC_ALIGN(uint64_t) == 8 ? 48 : 44), "output ABI size");
NV_VENC_ASSERT(sizeof(NvVencC2Plane) == 40, "C2 plane ABI size");
NV_VENC_ASSERT(sizeof(NvVencC2View) == 196, "C2 view ABI size");
NV_VENC_ASSERT(offsetof(NvVencC2View, planes) == 36, "C2 view ABI offset");
#endif
#undef NV_VENC_ASSERT
#undef NV_VENC_ALIGN
#endif
