//! AV1 default CDFs in NVDEC layout (mesa-nova av1_probs.rs, MIT).
//! Each CDF stores 32768 minus the spec values, without the terminator or counter.
use super::av1tables::*;
pub const PROBS_BYTES: usize = 0x2fc0;
pub fn q_ctx(q: i32) -> usize {
    if q <= 20 {
        0
    } else if q <= 60 {
        1
    } else if q <= 120 {
        2
    } else {
        3
    }
}
struct Put<'a> {
    t: &'a mut [u8],
    at: usize,
    stride: usize,
}
impl Put<'_> {
    fn vals(&mut self, v: &[u16]) {
        for (i, &v) in v.iter().enumerate() {
            self.t[self.at + 2 * i..self.at + 2 * i + 2]
                .copy_from_slice(&(32768 - v).to_le_bytes());
        }
        self.at += 2 * self.stride;
    }
    fn spec(&mut self, cdfs: &[u16], count: usize, len: usize) {
        for k in 0..count {
            self.vals(&cdfs[k * len..k * len + len - 2]);
        }
    }
    fn uniform(&mut self, nsym: usize, count: usize) {
        let mut v = [0; 16];
        for i in 1..nsym {
            v[i - 1] = ((32768 * i + nsym / 2) / nsym) as u16;
        }
        for _ in 0..count {
            self.vals(&v[..nsym - 1]);
        }
    }
}
pub fn default_probs(q: usize, table: &mut [u8]) {
    table[..PROBS_BYTES].fill(0);
    let mut p = Put {
        t: table,
        at: 0,
        stride: 0,
    };

    p.at = 0x0000;
    p.stride = 3;
    p.spec(&Default_Partition_W8_Cdf, 4, 5);
    p.at = 0x0018;
    p.stride = 1;
    p.spec(&Default_Use_Wiener_Cdf, 1, 3);
    p.at = 0x001a;
    p.stride = 1;
    p.spec(&Default_Use_Sgrproj_Cdf, 1, 3);
    p.at = 0x001c;
    p.stride = 2;
    p.spec(&Default_Restoration_Type_Cdf, 1, 4);
    p.at = 0x0020;
    p.stride = 16;
    p.spec(&Default_Partition_W16_Cdf, 4, 11);
    p.spec(&Default_Partition_W32_Cdf, 4, 11);
    p.spec(&Default_Partition_W64_Cdf, 4, 11);
    p.at = 0x0132;
    p.stride = 16;
    p.spec(&Default_Partition_W128_Cdf, 4, 9); // in the space the 64x64 CDFs leave
    p.at = 0x01a0;
    p.stride = 12;
    p.spec(&Default_Intra_Frame_Y_Mode_Cdf, 25, 14);
    p.at = 0x03f8;
    p.stride = 1;
    p.spec(&Default_Segment_Id_Predicted_Cdf, 3, 3);
    p.at = 0x03fe;
    p.stride = 7;
    p.spec(&Default_Segment_Id_Cdf, 3, 9);
    p.at = 0x0428;
    p.stride = 1;
    p.spec(&Default_Skip_Cdf, 3, 3);
    p.at = 0x042e;
    p.stride = 3;
    p.spec(&Default_Delta_Q_Cdf, 1, 5);
    p.at = 0x0434;
    p.stride = 3;
    for _i in 0..4 {
        p.spec(&Default_Delta_Lf_Cdf, 1, 5);
    } // DeltaLFMulti: FRAME_LF_COUNT copies
    p.at = 0x044c;
    p.stride = 3;
    p.spec(&Default_Delta_Lf_Cdf, 1, 5);
    p.at = 0x0452;
    p.stride = 1;
    p.spec(&Default_Skip_Mode_Cdf, 3, 3);
    p.at = 0x0458;
    p.stride = 1;
    p.spec(&Default_Txfm_Split_Cdf, 21, 3);
    p.at = 0x0484;
    p.stride = 2;
    p.spec(&Default_Tx_8x8_Cdf, 3, 3);
    p.at = 0x0490;
    p.stride = 2;
    p.spec(&Default_Tx_16x16_Cdf, 3, 4);
    p.spec(&Default_Tx_32x32_Cdf, 3, 4);
    p.spec(&Default_Tx_64x64_Cdf, 3, 4);
    p.at = 0x04b4;
    p.stride = 12;
    p.spec(&Default_Y_Mode_Cdf, 4, 14);
    p.at = 0x0514;
    p.stride = 13;
    p.spec(&Default_Uv_Mode_Cfl_Not_Allowed_Cdf, 13, 14);
    p.at = 0x0666;
    p.stride = 13;
    p.spec(&Default_Uv_Mode_Cfl_Allowed_Cdf, 13, 15);
    p.at = 0x07b8;
    p.stride = 1;
    p.spec(&Default_Is_Inter_Cdf, 4, 3);
    p.at = 0x07c0;
    p.stride = 1;
    p.spec(&Default_Comp_Mode_Cdf, 5, 3);
    p.at = 0x07ca;
    p.stride = 1;
    p.spec(&Default_Single_Ref_Cdf, 18, 3);
    p.at = 0x07ee;
    p.stride = 1;
    p.spec(&Default_Comp_Ref_Type_Cdf, 5, 3);
    p.at = 0x07f8;
    p.stride = 1;
    p.spec(&Default_Uni_Comp_Ref_Cdf, 9, 3);
    p.at = 0x080a;
    p.stride = 1;
    p.spec(&Default_Comp_Ref_Cdf, 9, 3);
    p.at = 0x081c;
    p.stride = 1;
    p.spec(&Default_Comp_Bwd_Ref_Cdf, 6, 3);
    p.at = 0x0828;
    p.stride = 1;
    p.spec(&Default_New_Mv_Cdf, 6, 3);
    p.at = 0x0834;
    p.stride = 1;
    p.spec(&Default_Zero_Mv_Cdf, 2, 3);
    p.at = 0x0838;
    p.stride = 1;
    p.spec(&Default_Ref_Mv_Cdf, 6, 3);
    p.at = 0x084a;
    p.stride = 1;
    p.spec(&Default_Drl_Mode_Cdf, 3, 3);
    p.at = 0x0850;
    p.stride = 2;
    p.spec(&Default_Interp_Filter_Cdf, 16, 4);
    // motion vectors: per component, from the spec's context 0 (normal motion vectors)
    p.at = 0x0890;
    p.stride = 3;
    p.spec(&Default_Mv_Joint_Cdf, 1, 5);
    p.at = 0x0896;
    p.stride = 1;
    for _c in 0..2 {
        p.spec(&Default_Mv_Sign_Cdf, 1, 3);
    }
    p.at = 0x089a;
    p.stride = 10;
    for _c in 0..2 {
        p.spec(&Default_Mv_Class_Cdf[(0) * 12..], 1, 12);
    }
    p.at = 0x08c2;
    p.stride = 3;
    for _c in 0..2 {
        p.spec(&Default_Mv_Class0_Fr_Cdf[(0) * 10 + (0) * 5..], 2, 5);
    }
    p.at = 0x08da;
    p.stride = 3;
    for _c in 0..2 {
        p.spec(&Default_Mv_Fr_Cdf[(0) * 5..], 1, 5);
    }
    p.at = 0x08e6;
    p.stride = 1;
    for _c in 0..2 {
        p.spec(&Default_Mv_Class0_Hp_Cdf, 1, 3);
    }
    p.at = 0x08ea;
    p.stride = 1;
    for _c in 0..2 {
        p.spec(&Default_Mv_Hp_Cdf, 1, 3);
    }
    p.at = 0x08ee;
    p.stride = 1;
    for _c in 0..2 {
        p.spec(&Default_Mv_Class0_Bit_Cdf, 1, 3);
    }
    p.at = 0x08f2;
    p.stride = 1;
    for _c in 0..2 {
        p.spec(&Default_Mv_Bit_Cdf, 10, 3);
    }
    p.at = 0x091a;
    p.stride = 1;
    p.spec(&Default_Use_Obmc_Cdf, 22, 3);
    p.at = 0x0946;
    p.stride = 2;
    p.spec(&Default_Motion_Mode_Cdf, 22, 4);
    p.at = 0x099e;
    p.stride = 7;
    p.spec(&Default_Compound_Mode_Cdf, 8, 9);
    p.at = 0x0a70;
    p.stride = 1;
    p.spec(&Default_Compound_Type_Cdf, 22, 3);
    p.at = 0x0a9c;
    p.stride = 1;
    p.uniform(2, 1);
    p.spec(&Default_Inter_Intra_Cdf, 3, 3); // block size group 0: unused
    p.at = 0x0aa4;
    p.stride = 3;
    p.uniform(4, 1);
    p.spec(&Default_Inter_Intra_Mode_Cdf, 3, 5);
    p.at = 0x0abc;
    p.stride = 1;
    p.spec(&Default_Wedge_Inter_Intra_Cdf, 22, 3);
    p.at = 0x0ae8;
    p.stride = 15;
    p.spec(&Default_Wedge_Index_Cdf, 22, 17);
    p.at = 0x0d7c;
    p.stride = 1;
    p.spec(&Default_Palette_Y_Mode_Cdf, 21, 3);
    p.at = 0x0da6;
    p.stride = 1;
    p.spec(&Default_Palette_Uv_Mode_Cdf, 2, 3);
    p.at = 0x0daa;
    p.stride = 6;
    p.spec(&Default_Palette_Y_Size_Cdf, 7, 8);
    p.at = 0x0dfe;
    p.stride = 6;
    p.spec(&Default_Palette_Uv_Size_Cdf, 7, 8);
    p.at = 0x0e52;
    p.stride = 7;
    p.spec(&Default_Cfl_Sign_Cdf, 1, 9);
    p.at = 0x0e60;
    p.stride = 15;
    p.spec(&Default_Cfl_Alpha_Cdf, 6, 17);
    p.at = 0x0f14;
    p.stride = 1;
    p.spec(&Default_Intrabc_Cdf, 1, 3);
    p.at = 0x0f16;
    p.stride = 6;
    p.spec(&Default_Angle_Delta_Cdf, 8, 8);
    p.at = 0x0f76;
    p.stride = 4;
    p.spec(&Default_Filter_Intra_Mode_Cdf, 1, 6);
    p.at = 0x0f7e;
    p.stride = 1;
    p.spec(&Default_Filter_Intra_Cdf, 22, 3);
    p.at = 0x0faa;
    p.stride = 1;
    p.spec(&Default_Comp_Group_Idx_Cdf, 6, 3);
    p.at = 0x0fb8;
    p.stride = 1;
    p.spec(&Default_Compound_Idx_Cdf, 6, 3);
    // palette colour index CDFs: those of n and 10 - n colours share 8 values, those of 5 colours take 4
    // Palette colour index CDFs: n and 10-n share eight values; five colours take four.
    p.at = 0xfe0;
    p.stride = 8;
    p.spec(&Default_Palette_Size_2_Y_Color_Cdf, 5, 3);
    p.at = 0xfe2;
    p.stride = 8;
    p.spec(&Default_Palette_Size_8_Y_Color_Cdf, 5, 9);
    p.at = 0x1030;
    p.stride = 8;
    p.spec(&Default_Palette_Size_3_Y_Color_Cdf, 5, 4);
    p.at = 0x1034;
    p.stride = 8;
    p.spec(&Default_Palette_Size_7_Y_Color_Cdf, 5, 8);
    p.at = 0x1080;
    p.stride = 8;
    p.spec(&Default_Palette_Size_4_Y_Color_Cdf, 5, 5);
    p.at = 0x1086;
    p.stride = 8;
    p.spec(&Default_Palette_Size_6_Y_Color_Cdf, 5, 7);
    p.at = 0x10d0;
    p.stride = 4;
    p.spec(&Default_Palette_Size_5_Y_Color_Cdf, 5, 6);
    p.at = 0x1100;
    p.stride = 8;
    p.spec(&Default_Palette_Size_2_Uv_Color_Cdf, 5, 3);
    p.at = 0x1102;
    p.stride = 8;
    p.spec(&Default_Palette_Size_8_Uv_Color_Cdf, 5, 9);
    p.at = 0x1150;
    p.stride = 8;
    p.spec(&Default_Palette_Size_3_Uv_Color_Cdf, 5, 4);
    p.at = 0x1154;
    p.stride = 8;
    p.spec(&Default_Palette_Size_7_Uv_Color_Cdf, 5, 8);
    p.at = 0x11a0;
    p.stride = 8;
    p.spec(&Default_Palette_Size_4_Uv_Color_Cdf, 5, 5);
    p.at = 0x11a6;
    p.stride = 8;
    p.spec(&Default_Palette_Size_6_Uv_Color_Cdf, 5, 7);
    p.at = 0x11f0;
    p.stride = 4;
    p.spec(&Default_Palette_Size_5_Uv_Color_Cdf, 5, 6);
    // transform types: libaom's 4 transform sizes per set; the sizes a set is not used at hold uniform CDFs
    p.at = 0x1220;
    p.stride = 8;
    p.spec(&Default_Intra_Tx_Type_Set1_Cdf, 26, 8);
    p.uniform(7, 2 * 13);
    p.at = 0x1560;
    p.stride = 4;
    p.spec(&Default_Intra_Tx_Type_Set2_Cdf, 39, 6);
    p.uniform(5, 13);
    p.at = 0x1700;
    p.stride = 16;
    p.spec(&Default_Inter_Tx_Type_Set1_Cdf, 2, 17);
    p.uniform(16, 2);
    p.at = 0x1780;
    p.stride = 16;
    p.uniform(12, 2);
    p.spec(&Default_Inter_Tx_Type_Set2_Cdf, 1, 13);
    p.uniform(12, 1);
    p.at = 0x1796;
    p.stride = 16;
    p.spec(&Default_Inter_Tx_Type_Set3_Cdf, 4, 3); // after set 2's
                                                   // coefficients, of quantizer context q
    p.at = 0x1800;
    p.stride = 1;
    p.spec(&Default_Txb_Skip_Cdf[(q) * 195..], 5 * 13, 3);
    p.at = 0x1882;
    p.stride = 1;
    p.spec(&Default_Eob_Extra_Cdf[(q) * 270..], 5 * 2 * 9, 3);
    p.at = 0x1940;
    p.stride = 4;
    p.spec(&Default_Eob_Pt_16_Cdf[(q) * 24..], 2 * 2, 6);
    p.at = 0x1960;
    p.stride = 8;
    p.spec(&Default_Eob_Pt_32_Cdf[(q) * 28..], 2 * 2, 7);
    p.at = 0x19a0;
    p.stride = 8;
    p.spec(&Default_Eob_Pt_64_Cdf[(q) * 32..], 2 * 2, 8);
    p.at = 0x19e0;
    p.stride = 8;
    p.spec(&Default_Eob_Pt_128_Cdf[(q) * 36..], 2 * 2, 9);
    p.at = 0x1a20;
    p.stride = 8;
    p.spec(&Default_Eob_Pt_256_Cdf[(q) * 40..], 2 * 2, 10);
    p.at = 0x1a60;
    p.stride = 16;
    for pl in 0..2 {
        p.spec(&Default_Eob_Pt_512_Cdf[(q) * 22 + (pl) * 11..], 1, 11);
        p.uniform(10, 1);
    }
    p.at = 0x1ae0;
    p.stride = 16;
    for pl in 0..2 {
        p.spec(&Default_Eob_Pt_1024_Cdf[(q) * 24 + (pl) * 12..], 1, 12);
        p.uniform(11, 1);
    }
    p.at = 0x1b60;
    p.stride = 2;
    p.spec(&Default_Coeff_Base_Eob_Cdf[(q) * 160..], 5 * 2 * 4, 4);
    p.at = 0x1c00;
    p.stride = 4;
    p.spec(&Default_Coeff_Base_Cdf[(q) * 2100..], 5 * 2 * 42, 5);
    p.at = 0x2920;
    p.stride = 1;
    p.spec(&Default_Dc_Sign_Cdf[(q) * 18..], 2 * 3, 3);
    p.at = 0x2930;
    p.stride = 4;
    p.spec(&Default_Coeff_Br_Cdf[(q) * 1050..], 5 * 2 * 21, 5);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c_default_table_golden() {
        // FNV-1a of all 0x2fc0 bytes from the C oracle, for each quantizer context.
        let hashes = [
            0x31e9baa11dbd6a54u64,
            0xe05d33580ba6742d,
            0x3a1892d570b67974,
            0xfb284c8b7ea484cc,
        ];
        let mut table = [0xa5; PROBS_BYTES + 1];
        for (q, expected) in hashes.into_iter().enumerate() {
            default_probs(q, &mut table);
            assert_eq!(table[PROBS_BYTES], 0xa5);
            let hash = table[..PROBS_BYTES]
                .iter()
                .fold(0xcbf29ce484222325u64, |h, &b| {
                    (h ^ b as u64).wrapping_mul(0x100000001b3)
                });
            assert_eq!(hash, expected);
        }
        for (q, ctx) in [
            (0, 0),
            (20, 0),
            (21, 1),
            (60, 1),
            (61, 2),
            (120, 2),
            (121, 3),
            (255, 3),
        ] {
            assert_eq!(q_ctx(q), ctx);
        }
    }
}
