/* SPDX-License-Identifier: MIT
 * Source-built SPIR-V -> NIR -> NAK bridge. Descriptor and push lowering follow
 * Mesa nvk_shader.c / nvk_nir_lower_descriptors.c at the pinned revision,
 * with a deliberately small, explicitly documented root table ABI.
 */
#include "nv_nak.h"
#include "nv_nak_build.h"
#include "compiler/nir/nir.h"
#include "compiler/nir/nir_builder.h"
#include "compiler/spirv/nir_spirv.h"
#include "nouveau/compiler/nak.h"
#include "nouveau/headers/nv_device_info.h"
#include "util/ralloc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* NAK (Mesa >= 26) reads the driver's constant-buffer layout for fragment
 * sample locations/masks and the printf buffer from these two symbols, which
 * every driver linking NAK defines (NVK: nvk_shader.c). The bridge compiles
 * compute SPIR-V only (nv_nak_compile, MESA_SHADER_COMPUTE) and NAK's release
 * build has no printf (NAK_CAN_PRINTF false under NDEBUG), so neither table is
 * ever read: both are zero. */
const struct nak_constant_offset_info nak_const_offsets_base = { 0 };
const struct nak_constant_offset_info nak_const_offsets_turing_graphics = { 0 };

#define EXPORTED __attribute__((visibility("default")))
struct lowering { char *error; size_t capacity; bool failed; };
static void fail(struct lowering *s, const char *message)
{
    s->failed = true;
    if (s->capacity && !s->error[0]) snprintf(s->error, s->capacity, "%s", message);
}
static void diagnostic(void *data, enum nir_spirv_debug_level level,
                       size_t offset, const char *message)
{
    struct lowering *s = data;
    if (level == NIR_SPIRV_DEBUG_LEVEL_ERROR) {
        s->failed = true;
        if (s->capacity) snprintf(s->error, s->capacity, "SPIR-V word %zu: %s", offset, message);
    }
}
static bool lower_root(nir_builder *b, nir_intrinsic_instr *intr, void *data)
{
    struct lowering *s = data;
    nir_def *value = NULL;
    b->cursor = nir_before_instr(&intr->instr);
    if (intr->intrinsic == nir_intrinsic_load_vulkan_descriptor) {
        nir_intrinsic_instr *idx = nir_src_as_intrinsic(intr->src[0]);
        if (!idx || idx->intrinsic != nir_intrinsic_vulkan_resource_index ||
            nir_intrinsic_desc_set(idx) != 0 || nir_intrinsic_binding(idx) >= 8 ||
            !nir_src_is_const(idx->src[0]) || nir_src_as_uint(idx->src[0]) != 0) {
            fail(s, "Only set 0 SSBO bindings 0..7 with descriptorCount=1 are supported");
            return false;
        }
        if (intr->def.num_components != 1 || intr->def.bit_size != 64) {
            fail(s, "Unexpected descriptor address format"); return false;
        }
        value = nir_ldc_nv(b, 1, 64, nir_imm_int(b, 0),
                           nir_imm_int(b, nir_intrinsic_binding(idx) * 16),
                           .align_mul = 8, .align_offset = 0);
    } else if (intr->intrinsic == nir_intrinsic_load_push_constant) {
        if (nir_intrinsic_base(intr) + nir_intrinsic_range(intr) > 128) {
            fail(s, "Push constants exceed 128 bytes"); return false;
        }
        value = nir_ldc_nv(b, intr->def.num_components, intr->def.bit_size,
                           nir_imm_int(b, 0),
                           nir_iadd_imm(b, intr->src[0].ssa, 1024 + nir_intrinsic_base(intr)),
                           .align_mul = intr->def.bit_size / 8, .align_offset = 0);
    } else if (intr->intrinsic == nir_intrinsic_load_base_global_invocation_id ||
               intr->intrinsic == nir_intrinsic_load_base_workgroup_id) {
        value = nir_imm_zero(b, intr->def.num_components, intr->def.bit_size);
    } else if (intr->intrinsic == nir_intrinsic_load_num_workgroups) {
        fail(s, "NumWorkgroups is not supported by the current root table ABI");
        return false;
    }
    if (!value) return false;
    nir_def_rewrite_uses(&intr->def, value);
    nir_instr_remove(&intr->instr);
    return true;
}

static void shared_layout(const struct glsl_type *type, unsigned *size, unsigned *alignment)
{
    unsigned component = glsl_type_is_boolean(type) ? 4 : glsl_get_bit_size(type) / 8;
    *size = component * glsl_get_vector_elements(type);
    *alignment = component;
}

/* The nv_device_info fields NAK reads (api.rs: sm, max_warps_per_mp; qmd.rs: cls_compute,
 * max_smem_per_wg_kB, sm_smem_sizes_kB) for each supported shader model, with the values
 * Mesa's winsys derives for that SM (nouveau_device.c: max_warps_per_mp_for_sm,
 * init_shared_mem_sizes; compute classes from the NVIDIA class headers). */
int nv_nak_device(uint32_t sm, struct nv_device_info *dev)
{
    static const uint16_t ampere[10] = { 0, 8, 16, 32, 64, 100, 132, 164, 196, 228 };
    memset(dev, 0, sizeof(*dev));
    dev->sm = sm;
    switch (sm) {
    case 75:  dev->cls_compute = 0xc5c0; dev->max_warps_per_mp = 32; break;
    case 80:  dev->cls_compute = 0xc6c0; dev->max_warps_per_mp = 64; dev->sm_smem_size_count = 8; break;
    case 86:  dev->cls_compute = 0xc7c0; dev->max_warps_per_mp = 48; dev->sm_smem_size_count = 6; break;
    case 89:  dev->cls_compute = 0xc9c0; dev->max_warps_per_mp = 48; dev->sm_smem_size_count = 6; break;
    case 90:  dev->cls_compute = 0xcbc0; dev->max_warps_per_mp = 64; dev->sm_smem_size_count = 10; break;
    case 120: dev->cls_compute = 0xcec0; dev->max_warps_per_mp = 48; dev->sm_smem_size_count = 6; break;
    default: return -1;
    }
    if (sm == 75) {
        dev->sm_smem_sizes_kB[0] = 32;
        dev->sm_smem_sizes_kB[1] = 64;
        dev->sm_smem_size_count = 2;
    } else {
        memcpy(dev->sm_smem_sizes_kB, ampere, sizeof(ampere[0]) * dev->sm_smem_size_count);
    }
    dev->max_smem_per_wg_kB = dev->sm_smem_sizes_kB[dev->sm_smem_size_count - 1];
    return 0;
}

/* Compiles one SPIR-V compute entrypoint for `dev`. On success returns the NAK binary (destroy
 * with nak_shader_bin_destroy) and the size of the shader's constant data, which this ABI has
 * no buffer for. On failure returns NULL with a diagnostic in `error`. */
struct nak_shader_bin *nv_nak_build(const uint32_t *words, size_t count, const char *entry,
                                    const struct nv_device_info *dev, uint32_t *constant_bytes,
                                    char *error, size_t capacity)
{
    if (capacity) error[0] = 0;
    struct lowering state = { error, capacity, false };
    struct nak_shader_bin *bin = NULL;
    if (!words || count < 5 || words[0] != 0x07230203 || !entry) {
        fail(&state, "Expected a SPIR-V compute module"); return NULL;
    }
    for (size_t i = 5; i < count;) {
        size_t n = words[i] >> 16;
        if (!n || n > count - i) { fail(&state, "Truncated SPIR-V instruction"); return NULL; }
        i += n;
    }
    glsl_type_singleton_init_or_ref();
    struct nak_compiler *nak = nak_compiler_create(dev);
    struct spirv_to_nir_options options = {
        .environment = NIR_SPIRV_VULKAN,
        .ssbo_addr_format = nir_address_format_64bit_global,
        .ubo_addr_format = nir_address_format_64bit_global,
        .phys_ssbo_addr_format = nir_address_format_64bit_global,
        .push_const_addr_format = nir_address_format_32bit_offset,
        .shared_addr_format = nir_address_format_32bit_offset,
        .min_ssbo_alignment = 4, .min_ubo_alignment = 4,
        .debug = { .func = diagnostic, .private_data = &state },
        .skip_os_break_in_debug_build = true,
    };
    nir_shader *nir = spirv_to_nir(words, count, NULL, MESA_SHADER_COMPUTE,
                                  entry, &options, nak_nir_options(nak));
    if (!nir) { fail(&state, "SPIR-V translation failed"); goto done; }
    nir_foreach_variable_with_modes(var, nir, nir_var_mem_ubo | nir_var_uniform) {
        (void)var;
        fail(&state, "Only storage buffers and push constants are supported; no UBO, sampler or image");
    }
    if (nir->info.workgroup_size_variable) fail(&state, "Specialized workgroup sizes are unsupported");
    if (state.failed) goto done;
    NIR_PASS(_, nir, nir_lower_variable_initializers, nir_var_function_temp);
    NIR_PASS(_, nir, nir_lower_returns);
    NIR_PASS(_, nir, nir_inline_functions);
    NIR_PASS(_, nir, nir_opt_copy_prop);
    NIR_PASS(_, nir, nir_opt_deref);
    nir_remove_non_entrypoints(nir);
    NIR_PASS(_, nir, nir_lower_variable_initializers, nir_var_all);
    NIR_PASS(_, nir, nir_split_var_copies);
    NIR_PASS(_, nir, nir_split_per_member_structs);
    nak_preprocess_nir(nir, nak);
    NIR_PASS(_, nir, nir_lower_compute_system_values, NULL);
    NIR_PASS(_, nir, nir_lower_explicit_io, nir_var_mem_push_const, nir_address_format_32bit_offset);
    NIR_PASS(_, nir, nir_shader_intrinsics_pass, lower_root, nir_metadata_none, &state);
    if (state.failed) goto done;
    NIR_PASS(_, nir, nir_lower_explicit_io, nir_var_mem_ssbo | nir_var_mem_global,
             nir_address_format_64bit_global);
    NIR_PASS(_, nir, nir_lower_vars_to_explicit_types, nir_var_mem_shared, shared_layout);
    NIR_PASS(_, nir, nir_lower_explicit_io, nir_var_mem_shared, nir_address_format_32bit_offset);
    NIR_PASS(_, nir, nir_opt_dce);
    bin = nak_compile_shader(nir, false, nak, 0, NULL, false);
    if (!bin) { fail(&state, "NAK compilation failed"); goto done; }
    *constant_bytes = nir->constant_data_size;
done:
    if (nir) ralloc_free(nir);
    nak_compiler_destroy(nak);
    glsl_type_singleton_decref();
    return bin;
}

EXPORTED int nv_nak_compile(const uint32_t *words, size_t count,
                           const char *entry, uint32_t sm,
                           struct nv_nak_result *out, char *error, size_t capacity)
{
    if ((capacity && !error) || !out) return -1;
    memset(out, 0, sizeof(*out));
    struct nv_device_info dev;
    if (nv_nak_device(sm, &dev)) {
        if (capacity) snprintf(error, capacity, "Unsupported shader model %u", sm);
        return -1;
    }
    uint32_t constant_bytes = 0;
    struct nak_shader_bin *bin = nv_nak_build(words, count, entry, &dev, &constant_bytes, error, capacity);
    if (!bin) return -1;
    if (constant_bytes) {
        if (capacity) snprintf(error, capacity, "Shader constant data (%u bytes) has no buffer in this ABI", constant_bytes);
        nak_shader_bin_destroy(bin);
        return -1;
    }
    out->code = malloc(bin->code_size);
    if (!out->code) {
        if (capacity) snprintf(error, capacity, "Out of host memory");
        nak_shader_bin_destroy(bin);
        return -1;
    }
    memcpy(out->code, bin->code, bin->code_size);
    out->code_size = bin->code_size;
    out->registers = bin->info.num_gprs;
    out->local_bytes = bin->info.slm_size;
    out->shared_bytes = bin->info.cs.smem_size;
    out->barriers = bin->info.num_control_barriers;
    for (unsigned i = 0; i < 3; i++) out->workgroup[i] = bin->info.cs.local_size[i];
    out->cbuf_bytes = 1152;
    nak_shader_bin_destroy(bin);
    return 0;
}
EXPORTED void nv_nak_free(struct nv_nak_result *result)
{
    if (result) { free(result->code); memset(result, 0, sizeof(*result)); }
}
