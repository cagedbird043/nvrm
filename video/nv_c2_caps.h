#ifndef NV_C2_CAPS_H
#define NV_C2_CAPS_H
#include <stdint.h>
#include <stddef.h>
#include "nv_video.h"
#ifdef __cplusplus
extern "C" {
#endif

#define NV_C2_CAPS_ABI_VERSION 2u
#define NV_C2_XMLGEN_EXIT_NO_NVDEC 3
/* Return codes, never errno. NV_VIDEO_* values retain their existing meaning. */
#define NV_C2_OK                 0
#define NV_C2_E_INVALID         (-1)
#define NV_C2_E_UNSUPPORTED     (-2)
#define NV_C2_E_NO_MEMORY       (-4)
#define NV_C2_E_DEVICE_LOST     (-5)
#define NV_C2_E_INTERNAL        (-8) /* caught panic or unusable reported caps */

struct NvC2Caps;
struct NvC2Text;

struct NvC2DecoderCaps {
    uint32_t codec; /* NV_VIDEO_CODEC_*; equals caps.codec */
    const char *name; /* immutable NUL-terminated UTF-8 */
    const char *mime; /* immutable NUL-terminated ASCII */
    struct NvVideoCaps caps;
};
/* NvVideoCaps, included above, has exactly:
 * uint32_t codec, supported;
 * uint32_t max_width, max_height, min_width, min_height;
 * uint32_t bit_depth_mask, chroma_420_only, max_level, reserved[7];
 */
struct NvC2EncoderCaps {
    uint32_t codec; /* NV_VIDEO_CODEC_*, NOT video::enc discriminants */
    const char *name;
    const char *mime;
    uint32_t min_width, min_height, max_width, max_height;
};
struct NvC2CapsView {
    uint32_t abi_version; /* NV_C2_CAPS_ABI_VERSION */
    uint32_t nvenc_class; /* queried class, including unsupported class; 0 = none */
    const struct NvC2DecoderCaps *decoders;
    uint32_t decoder_count;
    const struct NvC2EncoderCaps *encoders;
    uint32_t encoder_count;
};
struct NvC2Bytes {
    const char *data; /* UTF-8, NUL terminator also present at data[len] */
    size_t len;      /* excludes terminator; output/write/compare exactly len bytes */
};

/* dev: valid device from nv_video_device_open with ABI 2 and nonzero NVDEC class.
 * Must remain open until snapshot destruction. out nonnull; *out NULL on failure.
 * Query all decoder caps and NVENC class once; validate and format XML once.
 * No partial snapshot on any failure. Zero advertised decoders -> empty snapshot,
 * without opening NVENC. Otherwise NVENC open failure -> mapped NV_VIDEO error.
 * Unsupported NVENC class -> success with encoder_count=0.
 */
int32_t nv_c2_caps_query(struct NvVideoDevice *dev, struct NvC2Caps **out);
/* out is caller storage; initialized to all zeros on failure. */
int32_t nv_c2_caps_view(const struct NvC2Caps *caps, struct NvC2CapsView *out);
int32_t nv_c2_media_codecs_xml(const struct NvC2Caps *caps, struct NvC2Bytes *out);
void nv_c2_caps_destroy(struct NvC2Caps *caps); /* NULL allowed */

/* gpu_name is borrowed NUL-terminated UTF-8 for this call only, copied internally.
 * Return an owned text handle; *out=NULL on failure. Reason excludes added period;
 * XML has existing exact bytes, including comment sanitization and final newline.
 */
int32_t nv_c2_no_decoder_xml(const char *gpu_name, uint32_t nvdec_class, struct NvC2Text **out);
int32_t nv_c2_no_decoder_reason(const char *gpu_name, uint32_t nvdec_class, struct NvC2Text **out);
int32_t nv_c2_text_view(const struct NvC2Text *text, struct NvC2Bytes *out);
void nv_c2_text_destroy(struct NvC2Text *text); /* NULL allowed */

/* Errors: nv_video_last_error(), same TLS convention as existing video ABI.
 * Copy diagnostic before the next error-producing ABI call on the same thread.
 * Every fallible function catches unwind; no Rust panic crosses C ABI.
 * Destructors catch unwind too and never throw. Allocator abort is not recoverable.
 * Queries/views never transfer ownership of borrowed pointers. Never free/realloc
 * them in C/C++; never retain snapshot pointers after snapshot/device destruction.
 * View arrays, names, MIME, XML bytes remain unchanged through repeated view calls,
 * until their owning handle is destroyed. No process-global scratch buffer.
 * A zero count uses a NULL array. All input/output pointers except documented NULL
 * destructor arguments must be valid; arbitrary dangling pointers are not checked.
 * Serialize all calls/destroy on one handle; independent handles may be used in
 * parallel subject to existing NvVideoDevice serialization requirements.
 */
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
#define NV_C2_ASSERT(c) static_assert(c, #c)
#define NV_C2_ALIGN(t) alignof(t)
#else
#define NV_C2_ASSERT(c) _Static_assert(c, #c)
#define NV_C2_ALIGN(t) _Alignof(t)
#endif
NV_C2_ASSERT(sizeof(struct NvVideoCaps) == 64);
#if UINTPTR_MAX == UINT64_MAX
NV_C2_ASSERT(sizeof(struct NvC2DecoderCaps) == 88);
NV_C2_ASSERT(NV_C2_ALIGN(struct NvC2DecoderCaps) == 8);
NV_C2_ASSERT(offsetof(struct NvC2DecoderCaps, caps) == 24);
NV_C2_ASSERT(sizeof(struct NvC2EncoderCaps) == 40);
NV_C2_ASSERT(NV_C2_ALIGN(struct NvC2EncoderCaps) == 8);
NV_C2_ASSERT(offsetof(struct NvC2EncoderCaps, min_width) == 24);
NV_C2_ASSERT(sizeof(struct NvC2CapsView) == 40);
NV_C2_ASSERT(NV_C2_ALIGN(struct NvC2CapsView) == 8);
NV_C2_ASSERT(offsetof(struct NvC2CapsView, encoders) == 24);
NV_C2_ASSERT(sizeof(struct NvC2Bytes) == 16);
NV_C2_ASSERT(NV_C2_ALIGN(struct NvC2Bytes) == 8);
NV_C2_ASSERT(offsetof(struct NvC2Bytes, len) == 8);
#else
NV_C2_ASSERT(sizeof(struct NvC2DecoderCaps) == 76);
NV_C2_ASSERT(NV_C2_ALIGN(struct NvC2DecoderCaps) == 4);
NV_C2_ASSERT(offsetof(struct NvC2DecoderCaps, caps) == 12);
NV_C2_ASSERT(sizeof(struct NvC2EncoderCaps) == 28);
NV_C2_ASSERT(NV_C2_ALIGN(struct NvC2EncoderCaps) == 4);
NV_C2_ASSERT(offsetof(struct NvC2EncoderCaps, min_width) == 12);
NV_C2_ASSERT(sizeof(struct NvC2CapsView) == 24);
NV_C2_ASSERT(NV_C2_ALIGN(struct NvC2CapsView) == 4);
NV_C2_ASSERT(offsetof(struct NvC2CapsView, encoders) == 16);
NV_C2_ASSERT(sizeof(struct NvC2Bytes) == 8);
NV_C2_ASSERT(NV_C2_ALIGN(struct NvC2Bytes) == 4);
NV_C2_ASSERT(offsetof(struct NvC2Bytes, len) == 4);
#endif
#undef NV_C2_ASSERT
#undef NV_C2_ALIGN
#endif
