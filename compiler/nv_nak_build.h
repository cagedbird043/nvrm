/* SPDX-License-Identifier: MIT
 * Bridge internals shared by the shared library (nv_nak.c) and the build-time kernel
 * generator (gen.c). Not part of the library ABI. */
#ifndef NV_NATIVE_NAK_BUILD_H
#define NV_NATIVE_NAK_BUILD_H
#include <stddef.h>
#include <stdint.h>
struct nv_device_info;
struct nak_shader_bin;
/* The device description NAK compiles and fills QMDs for at shader model `sm`; -1 if unsupported. */
int nv_nak_device(uint32_t sm, struct nv_device_info *dev);
/* Compiles one SPIR-V compute entrypoint with the bridge ABI. Returns the NAK binary (free with
 * nak_shader_bin_destroy) and the shader's constant data size, or NULL with `error` set. */
struct nak_shader_bin *nv_nak_build(const uint32_t *words, size_t count, const char *entry,
                                    const struct nv_device_info *dev, uint32_t *constant_bytes,
                                    char *error, size_t capacity);
#endif
