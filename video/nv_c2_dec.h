/* SPDX-License-Identifier: MIT */
#ifndef NV_C2_DEC_H
#define NV_C2_DEC_H
#include <stddef.h>
#include <stdint.h>
#include "nv_video.h"
#ifdef __cplusplus
extern "C" {
#endif
#define NV_C2_DEC_ABI_VERSION 1u
#define NV_C2_DEC_MAX_CSD_BYTES 2097152u
/* Serialized per handle, including synchronous callbacks and destroy. No callback
 * reentry or exceptions across ABI. All exports contain panics. Error text is
 * thread-local nv_video_last_error(); copy before the next call. Reserved = 0.
 * Config/input bytes borrowed for call only. Device borrowed through destroy.
 * Fetch success transfers unique nonzero cookie and acquire fence; dma-buf fds
 * stay borrowed. Poll transfers cookie and release fence. Destroy discards every
 * unpolled cookie and closes every owned fence. Always poll even after failure. */
struct NvC2Dec;
enum NvC2DecStatus { NV_C2_DEC_OK=0, NV_C2_DEC_FATAL=-1, NV_C2_DEC_INVALID=-2, NV_C2_DEC_LIMIT=-3 };
enum NvC2DecConsumed { NV_C2_DEC_RETRY=0, NV_C2_DEC_CONSUMED=1, NV_C2_DEC_ABORTED=2 };
enum NvC2DecBlocked { NV_C2_DEC_READY=0, NV_C2_DEC_POOL_BLOCKED=1, NV_C2_DEC_DPB_BLOCKED=2 };
enum NvC2DecFlags { NV_C2_DEC_EOS=1u, NV_C2_DEC_CSD=2u };
enum NvC2DecEventType { NV_C2_DEC_WORK=1u };
enum NvC2DecFetch { NV_C2_DEC_FETCH_OK=0, NV_C2_DEC_FETCH_AGAIN=1, NV_C2_DEC_FETCH_FATAL=-1 };
struct NvC2DecConfig {
    uint32_t abi_version, codec, max_width, max_height, bit_depth, extra_output_slots;
    const uint8_t *codec_config;
    size_t codec_config_len;
    uint32_t reserved[8];
};
struct NvC2DecBuffer { struct NvVideoTarget target; uint64_t cookie; };
typedef int32_t (*NvC2DecFetchFn)(void *, uint32_t, uint32_t, uint32_t, struct NvC2DecBuffer *);
typedef void (*NvC2DecDiscardFn)(void *, uint64_t);
struct NvC2DecCallbacks { void *ctx; NvC2DecFetchFn fetch; NvC2DecDiscardFn discard; };
struct NvC2DecResult { uint32_t consumed, blocked; int32_t cause; uint32_t reserved; };
struct NvC2DecEvent {
    uint32_t type, empty, corrupt, eos;
    uint64_t index, cookie;
    int32_t release_fence_fd;
    uint32_t size_changed, final_for_index, reserved;
    struct NvVideoPicture picture;
    struct NvVideoHdrMetadata hdr;
};
struct NvC2DecHdrStatic {
    uint32_t present;
    float red[2], green[2], blue[2], white[2];
    float max_luminance, min_luminance, max_cll, max_fall;
};
int32_t nv_c2_dec_create(struct NvVideoDevice *, const struct NvC2DecConfig *, const struct NvC2DecCallbacks *, struct NvC2Dec **);
int32_t nv_c2_dec_queue(struct NvC2Dec *, const uint8_t *, size_t, uint64_t, uint32_t, struct NvC2DecResult *);
/* Pump only; not EOS. Results from drain/flush use ABORTED. */
int32_t nv_c2_dec_drain(struct NvC2Dec *, struct NvC2DecResult *);
int32_t nv_c2_dec_flush(struct NvC2Dec *, struct NvC2DecResult *);
/* 1 event, 0 empty, INVALID bad args; valid after fatal, never decodes/fetches. */
int32_t nv_c2_dec_poll(struct NvC2Dec *, struct NvC2DecEvent *);
void nv_c2_dec_destroy(struct NvC2Dec *);
int32_t nv_c2_dec_hdr_static(uint32_t, const struct NvVideoHdrMetadata *, struct NvC2DecHdrStatic *);
int32_t nv_c2_dec_check_csd_size(uint64_t, uint64_t);
struct NvC2GrallocSnapshot {
    uint32_t abi_version;
    int32_t native_version, num_fds, num_ints;
    uint32_t payload_words, magic, num_planes;
    int32_t fds[5];
    uint32_t strides[4], offsets[4], sizes[4];
    uint32_t width, height, drm_format, tiling;
    uint64_t modifier, reserved_region_size, total_size;
    uint32_t pixel_stride;
    int32_t droid_format;
    uint32_t reserved[4];
};
enum NvC2LayoutPolicy { NV_C2_LAYOUT_NATIVE=0, NV_C2_LAYOUT_CPU_YV12=1 };
struct NvC2DecAllocation { uint32_t block_width, block_height, hal_format, policy; };
int32_t nv_c2_dec_allocation(uint32_t, uint32_t, uint32_t, uint32_t, struct NvC2DecAllocation *);
int32_t nv_c2_dec_target(const struct NvC2GrallocSnapshot *, const struct NvC2DecAllocation *, struct NvVideoTarget *);
int32_t nv_c2_dec_max_input_size(uint32_t, uint32_t, uint32_t *);
#ifdef __cplusplus
}
#define NV_C2_DEC_ASSERT(c) static_assert(c, #c)
#define NV_C2_DEC_ALIGN(t) alignof(t)
#else
#define NV_C2_DEC_ASSERT(c) _Static_assert(c, #c)
#define NV_C2_DEC_ALIGN(t) _Alignof(t)
#endif
#if UINTPTR_MAX == UINT64_MAX
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecConfig)==72 && NV_C2_DEC_ALIGN(struct NvC2DecConfig)==8);
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecCallbacks)==24 && NV_C2_DEC_ALIGN(struct NvC2DecCallbacks)==8);
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecEvent)==200 && offsetof(struct NvC2DecEvent,hdr)==136);
#else
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecConfig)==64 && NV_C2_DEC_ALIGN(struct NvC2DecConfig)==4);
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecCallbacks)==12 && NV_C2_DEC_ALIGN(struct NvC2DecCallbacks)==4);
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecEvent)==196 && offsetof(struct NvC2DecEvent,hdr)==132);
#endif
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecBuffer)==88 && offsetof(struct NvC2DecBuffer,cookie)==80);
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecResult)==16 && offsetof(struct NvC2DecResult,cause)==8);
NV_C2_DEC_ASSERT(offsetof(struct NvC2DecEvent,picture)==48);
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecHdrStatic)==52 && NV_C2_DEC_ALIGN(struct NvC2DecHdrStatic)==4);
NV_C2_DEC_ASSERT(sizeof(struct NvC2GrallocSnapshot)==160 && offsetof(struct NvC2GrallocSnapshot,modifier)==112);
NV_C2_DEC_ASSERT(sizeof(struct NvC2DecAllocation)==16);
NV_C2_DEC_ASSERT(offsetof(struct NvC2DecConfig,codec_config)==24);
NV_C2_DEC_ASSERT(offsetof(struct NvC2DecConfig,codec_config_len)==24+sizeof(void *));
NV_C2_DEC_ASSERT(offsetof(struct NvC2DecCallbacks,fetch)==sizeof(void *));
NV_C2_DEC_ASSERT(offsetof(struct NvC2DecCallbacks,discard)==2*sizeof(void *));
NV_C2_DEC_ASSERT(NV_C2_DEC_ALIGN(struct NvC2DecBuffer)==NV_C2_DEC_ALIGN(uint64_t));
NV_C2_DEC_ASSERT(NV_C2_DEC_ALIGN(struct NvC2DecEvent)==NV_C2_DEC_ALIGN(uint64_t));
NV_C2_DEC_ASSERT(NV_C2_DEC_ALIGN(struct NvC2DecResult)==4);
NV_C2_DEC_ASSERT(offsetof(struct NvC2DecHdrStatic,red)==4);
NV_C2_DEC_ASSERT(NV_C2_DEC_ALIGN(struct NvC2GrallocSnapshot)==NV_C2_DEC_ALIGN(uint64_t));
NV_C2_DEC_ASSERT(offsetof(struct NvC2GrallocSnapshot,pixel_stride)==136);
NV_C2_DEC_ASSERT(NV_C2_DEC_ALIGN(struct NvC2DecAllocation)==4 && offsetof(struct NvC2DecAllocation,policy)==12);
#undef NV_C2_DEC_ASSERT
#undef NV_C2_DEC_ALIGN
#endif
