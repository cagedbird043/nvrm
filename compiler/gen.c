/* SPDX-License-Identifier: MIT
 * Build-time kernel generator: compiles one SPIR-V compute entrypoint for one shader model with
 * the bridge's ABI (nv_nak.c) and prints, as JSON, everything a runtime without NAK needs to
 * launch it: the SASS, the resource metadata, and a QMD template filled by Mesa's own
 * nak_fill_qmd plus the bit runs where the per-launch values (program address, cbuf 0 address,
 * grid size) go. The runs are measured from nak_fill_qmd one input bit at a time and then
 * checked against nak_fill_qmd on combined values; the generator fails rather than emit a
 * kernel needing local memory, a CRS stack or constant data, none of which the runtime provides.
 *
 * usage: nv-nak-gen <sm> <entrypoint> <file.spv>
 */
#include "nv_nak.h"
#include "nv_nak_build.h"
#include "nouveau/compiler/nak.h"
#include "nouveau/headers/nv_device_info.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define CBUF_BYTES 1152
#define ADDR_LOW 8   /* the runtime places programs and cbufs 256-byte aligned */
#define ADDR_HIGH 49 /* exclusive: 49-bit GPU virtual addresses */

enum { PROGRAM, CBUF, GRID_X, GRID_Y, GRID_Z, VARIABLES };
static const char *const names[VARIABLES] = { "program", "cbuf", "grid_x", "grid_y", "grid_z" };

struct launch { uint64_t value[VARIABLES]; };

static const struct nv_device_info *dev;
static const struct nak_shader_info *info;
static uint32_t qmd_bytes;

static void fill(const struct launch *l, uint32_t *qmd)
{
    struct nak_qmd_info q = {
        .addr = l->value[PROGRAM],
        .smem_size = info->cs.smem_size,
        .global_size = { l->value[GRID_X], l->value[GRID_Y], l->value[GRID_Z] },
        .num_cbufs = 1,
        .cbufs = { { .index = 0, .size = CBUF_BYTES, .addr = l->value[CBUF] } },
    };
    memset(qmd, 0, NAK_MAX_QMD_SIZE_B);
    nak_fill_qmd(dev, info, &q, qmd, qmd_bytes);
}

static int bit(const uint32_t *q, unsigned b) { return q[b / 32] >> (b % 32) & 1; }

/* run[v][k]: QMD bit receiving input bit k of variable v, or -1 when the bit is not carried */
static int run[VARIABLES][64];
static unsigned width[VARIABLES];

static void pack(const uint32_t *template, const struct launch *l, uint32_t *qmd)
{
    memcpy(qmd, template, NAK_MAX_QMD_SIZE_B);
    for (unsigned v = 0; v < VARIABLES; v++)
        for (unsigned k = 0; k < 64; k++)
            if (l->value[v] >> k & 1) qmd[run[v][k] / 32] |= 1u << (run[v][k] % 32);
}

static uint64_t lcg(uint64_t *s)
{
    *s = *s * 6364136223846793005ull + 1442695040888963407ull;
    return *s >> 11;
}

int main(int argc, char **argv)
{
    if (argc != 4) { fprintf(stderr, "usage: %s <sm> <entrypoint> <file.spv>\n", argv[0]); return 2; }
    uint32_t sm = (uint32_t)strtoul(argv[1], NULL, 10);
    struct nv_device_info device;
    if (nv_nak_device(sm, &device)) { fprintf(stderr, "unsupported shader model %u\n", sm); return 1; }
    dev = &device;
    FILE *f = fopen(argv[3], "rb");
    if (!f) { perror(argv[3]); return 1; }
    static uint32_t words[1 << 18];
    size_t count = fread(words, 4, sizeof(words) / 4, f);
    fclose(f);
    char error[512];
    uint32_t constant_bytes = 0;
    struct nak_shader_bin *bin = nv_nak_build(words, count, argv[2], dev, &constant_bytes, error, sizeof(error));
    if (!bin) { fprintf(stderr, "%s: %s\n", argv[3], error); return 1; }
    info = &bin->info;
    if (info->slm_size || info->crs_size || constant_bytes) {
        fprintf(stderr, "%s: needs local memory %u, CRS %u, constant data %u bytes; the runtime provides none\n",
                argv[3], info->slm_size, info->crs_size, constant_bytes);
        return 1;
    }
    qmd_bytes = nak_qmd_size_B(dev);

    struct nak_qmd_dispatch_size_layout grid = nak_get_qmd_dispatch_size_layout(dev);
    width[PROGRAM] = width[CBUF] = ADDR_HIGH;
    width[GRID_X] = grid.x_end - grid.x_start;
    width[GRID_Y] = grid.y_end - grid.y_start;
    width[GRID_Z] = grid.z_end - grid.z_start;
    for (unsigned v = GRID_X; v <= GRID_Z; v++)
        if (width[v] > 32) width[v] = 32;

    uint32_t template[NAK_MAX_QMD_DWORDS], probe[NAK_MAX_QMD_DWORDS], check[NAK_MAX_QMD_DWORDS];
    struct launch zero = { { 0 } };
    fill(&zero, template);
    int used[NAK_MAX_QMD_SIZE_B * 8] = { 0 };
    for (unsigned v = 0; v < VARIABLES; v++) {
        unsigned low = v <= CBUF ? ADDR_LOW : 0;
        for (unsigned k = 0; k < 64; k++) run[v][k] = -1;
        for (unsigned k = low; k < width[v]; k++) {
            struct launch l = zero;
            l.value[v] = 1ull << k;
            fill(&l, probe);
            int found = -1;
            for (unsigned b = 0; b < qmd_bytes * 8; b++) {
                if (bit(probe, b) == bit(template, b)) continue;
                if (found >= 0 || bit(template, b) || used[b]) {
                    fprintf(stderr, "%s bit %u does not map to one free QMD bit\n", names[v], k);
                    return 1;
                }
                found = (int)b;
            }
            if (found < 0) { fprintf(stderr, "%s bit %u changes no QMD bit\n", names[v], k); return 1; }
            used[found] = 1;
            run[v][k] = found;
        }
    }

    /* combined values: packing by the runs must reproduce nak_fill_qmd exactly */
    uint64_t seed = sm * 0x9e3779b97f4a7c15ull;
    struct launch samples[4];
    for (unsigned i = 0; i < 64; i++) {
        struct launch l;
        for (unsigned v = 0; v < VARIABLES; v++) {
            unsigned low = v <= CBUF ? ADDR_LOW : 0;
            uint64_t mask = (width[v] >= 64 ? ~0ull : (1ull << width[v]) - 1) & ~((1ull << low) - 1);
            l.value[v] = lcg(&seed) & mask;
        }
        if (i < 4) samples[i] = l;
        fill(&l, probe);
        pack(template, &l, check);
        if (memcmp(probe, check, qmd_bytes)) { fprintf(stderr, "combined launch %u differs from nak_fill_qmd\n", i); return 1; }
    }

    printf("{\"sm\": %u, \"compute_class\": %u, \"code\": \"", sm, dev->cls_compute);
    for (uint32_t i = 0; i < bin->code_size; i++) printf("%02x", ((const uint8_t *)bin->code)[i]);
    printf("\", \"registers\": %u, \"shared_bytes\": %u, \"barriers\": %u, \"workgroup\": [%u, %u, %u],\n",
           info->num_gprs, info->cs.smem_size, info->num_control_barriers,
           info->cs.local_size[0], info->cs.local_size[1], info->cs.local_size[2]);
    printf(" \"cbuf_bytes\": %u, \"qmd\": [", CBUF_BYTES);
    for (unsigned i = 0; i < qmd_bytes / 4; i++) printf("%s%u", i ? ", " : "", template[i]);
    printf("],\n \"width\": {");
    for (unsigned v = 0; v < VARIABLES; v++) printf("%s\"%s\": %u", v ? ", " : "", names[v], width[v]);
    printf("},\n \"runs\": {");
    for (unsigned v = 0; v < VARIABLES; v++) {
        printf("%s\"%s\": [", v ? ", " : "", names[v]);
        /* contiguous input bits landing on contiguous QMD bits: [input bit, QMD bit, length] */
        int first = 1;
        for (unsigned k = 0; k < 64;) {
            if (run[v][k] < 0) { k++; continue; }
            unsigned n = 1;
            while (k + n < 64 && run[v][k + n] == run[v][k] + (int)n) n++;
            printf("%s[%u, %d, %u]", first ? "" : ", ", k, run[v][k], n);
            first = 0;
            k += n;
        }
        printf("]");
    }
    printf("},\n \"samples\": [");
    for (unsigned i = 0; i < 4; i++) {
        fill(&samples[i], probe);
        printf("%s\n  {\"values\": [", i ? "," : "");
        for (unsigned v = 0; v < VARIABLES; v++) printf("%s%llu", v ? ", " : "", (unsigned long long)samples[i].value[v]);
        printf("], \"qmd\": [");
        for (unsigned w = 0; w < qmd_bytes / 4; w++) printf("%s%u", w ? ", " : "", probe[w]);
        printf("]}");
    }
    printf("]}\n");
    nak_shader_bin_destroy(bin);
    return 0;
}
