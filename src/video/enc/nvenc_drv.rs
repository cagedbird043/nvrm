// NVIDIA open-gpu-doc nvenc_drv.h (MIT), NV_NVENC_8_2=1.
// Generated with bindgen 0.73.2 --allowlist-type (records used by mickys/enc.c)
// --with-derive-default --no-doc-comments -- -DNV_NVENC_8_2=1 -Iinclude -Iinclude/sdk.
// Trimmed raw pointer accessors and bulk bitfield constructors; defaults initialise
// integer-only records safely. Layout assertions are generated against clang's C ABI.
#![allow(non_camel_case_types, non_snake_case, dead_code, clippy::all)]

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct __BindgenBitfieldUnit<Storage> {
    storage: Storage,
}
impl<S: AsRef<[u8]> + AsMut<[u8]>> __BindgenBitfieldUnit<S> {
    fn get(&self, offset: usize, width: usize) -> u64 {
        let mut v = 0;
        for bit in 0..width {
            v |= u64::from((self.storage.as_ref()[(offset + bit) / 8] >> ((offset + bit) % 8)) & 1)
                << bit;
        }
        v
    }
    fn set(&mut self, offset: usize, width: usize, value: u64) {
        for bit in 0..width {
            let i = offset + bit;
            let b = &mut self.storage.as_mut()[i / 8];
            *b = (*b & !(1 << (i % 8))) | (((value >> bit) as u8 & 1) << (i % 8));
        }
    }
}

pub const NV_NVENC_DRV_MAGIC_VALUE: u32 = 3384213510;
pub const HIST_BLOCK_SIZE: u32 = 192;
pub const H264_HIST_BLOCK_SIZE: u32 = 224;
pub const H265_HIST_BLOCK_SIZE: u32 = 224;
pub const VP9_HIST_BLOCK_SIZE: u32 = 224;
pub const AV1_HIST_BLOCK_SIZE: u32 = 224;
pub const AV1_HIST_PIXEL_FRAME_SIZE: ::std::os::raw::c_int = 65536;
pub const AV1_HIST_MV_SB_SIZE: ::std::os::raw::c_int = 1024;
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_surface_cfg_s {
    pub frame_width_minus1: ::std::os::raw::c_ushort,
    pub frame_height_minus1: ::std::os::raw::c_ushort,
    pub sfc_pitch: ::std::os::raw::c_ushort,
    pub sfc_pitch_chroma: ::std::os::raw::c_ushort,
    pub sfc_trans_mode: ::std::os::raw::c_ushort,
    pub reserved2: ::std::os::raw::c_ushort,
    pub luma_top_frm_offset: ::std::os::raw::c_uint,
    pub luma_bot_offset: ::std::os::raw::c_uint,
    pub chroma_top_frm_offset: ::std::os::raw::c_uint,
    pub chroma_bot_offset: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_h264_surface_cfg_s {
    #[inline]
    pub fn block_height(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 7) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_block_height(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 7, value as u64);
    }
    #[inline]
    pub fn tiled_16x16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tiled_16x16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 1, value as u64);
    }
    #[inline]
    pub fn memory_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_memory_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 2, value as u64);
    }
    #[inline]
    pub fn nv21_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(10, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_nv21_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(10, 1, value as u64);
    }
    #[inline]
    pub fn input_bl_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(11, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_input_bl_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(11, 2, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 19) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 19, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_sps_data_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_h264_sps_data_s {
    #[inline]
    pub fn profile_idc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_profile_idc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 8, value as u64);
    }
    #[inline]
    pub fn level_idc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_level_idc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 8, value as u64);
    }
    #[inline]
    pub fn chroma_format_idc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_format_idc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 2, value as u64);
    }
    #[inline]
    pub fn pic_order_cnt_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_order_cnt_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 2, value as u64);
    }
    #[inline]
    pub fn log2_max_frame_num_minus4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_log2_max_frame_num_minus4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 4, value as u64);
    }
    #[inline]
    pub fn log2_max_pic_order_cnt_lsb_minus4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_log2_max_pic_order_cnt_lsb_minus4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 4, value as u64);
    }
    #[inline]
    pub fn frame_mbs_only(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_frame_mbs_only(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 1, value as u64);
    }
    #[inline]
    pub fn stereo_mvc_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stereo_mvc_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(29, 1, value as u64);
    }
    #[inline]
    pub fn separate_colour_plane_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_separate_colour_plane_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 1, value as u64);
    }
    #[inline]
    pub fn lossless_qpprime_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(31, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_lossless_qpprime_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(31, 1, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_pps_data_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 8usize]>,
}
impl nvenc_h264_pps_data_s {
    #[inline]
    pub fn pic_param_set_id(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_param_set_id(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 8, value as u64);
    }
    #[inline]
    pub fn entropy_coding_mode_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_entropy_coding_mode_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 1, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l0_active_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(9, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l0_active_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(9, 5, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l1_active_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(14, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l1_active_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(14, 5, value as u64);
    }
    #[inline]
    pub fn weighted_bipred_idc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_weighted_bipred_idc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 2, value as u64);
    }
    #[inline]
    pub fn pic_init_qp_minus26(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(21, 6) << (64 - 6)) as i64 >> (64 - 6)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_pic_init_qp_minus26(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(21, 6, value as u64);
    }
    #[inline]
    pub fn chroma_qp_index_offset(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(27, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_chroma_qp_index_offset(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(27, 5, value as u64);
    }
    #[inline]
    pub fn second_chroma_qp_index_offset(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(32, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_second_chroma_qp_index_offset(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(32, 5, value as u64);
    }
    #[inline]
    pub fn constrained_intra_pred_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(37, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_constrained_intra_pred_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(37, 1, value as u64);
    }
    #[inline]
    pub fn deblocking_filter_control_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(38, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_deblocking_filter_control_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(38, 1, value as u64);
    }
    #[inline]
    pub fn transform_8x8_mode_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(39, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_transform_8x8_mode_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(39, 1, value as u64);
    }
    #[inline]
    pub fn pic_order_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(40, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_order_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(40, 1, value as u64);
    }
    #[inline]
    pub fn weighted_pred_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(41, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_weighted_pred_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(41, 1, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(42, 22) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(42, 22, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_me_hint_cfg_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 16usize]>,
}
impl nvenc_h264_me_hint_cfg_s {
    #[inline]
    pub fn self_temporal_stamp_l0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_temporal_stamp_l0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 3, value as u64);
    }
    #[inline]
    pub fn self_temporal_stamp_l1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_temporal_stamp_l1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 3, value as u64);
    }
    #[inline]
    pub fn self_temporal_explicit(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(6, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_temporal_explicit(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(6, 1, value as u64);
    }
    #[inline]
    pub fn self_temporal_search(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_temporal_search(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 1, value as u64);
    }
    #[inline]
    pub fn self_temporal_refine(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_temporal_refine(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 1, value as u64);
    }
    #[inline]
    pub fn self_temporal_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(9, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_temporal_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(9, 1, value as u64);
    }
    #[inline]
    pub fn coloc_stamp_l0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(10, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_coloc_stamp_l0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(10, 3, value as u64);
    }
    #[inline]
    pub fn coloc_stamp_l1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_coloc_stamp_l1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 3, value as u64);
    }
    #[inline]
    pub fn coloc_explicit(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_coloc_explicit(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 1, value as u64);
    }
    #[inline]
    pub fn coloc_search(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(17, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_coloc_search(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(17, 1, value as u64);
    }
    #[inline]
    pub fn coloc_refine(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_coloc_refine(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 1, value as u64);
    }
    #[inline]
    pub fn coloc_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_coloc_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 1, value as u64);
    }
    #[inline]
    pub fn self_spatial_stamp_l0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_spatial_stamp_l0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 3, value as u64);
    }
    #[inline]
    pub fn self_spatial_stamp_l1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(23, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_spatial_stamp_l1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(23, 3, value as u64);
    }
    #[inline]
    pub fn self_spatial_explicit(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_spatial_explicit(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 1, value as u64);
    }
    #[inline]
    pub fn self_spatial_search(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_spatial_search(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 1, value as u64);
    }
    #[inline]
    pub fn self_spatial_refine(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_spatial_refine(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 1, value as u64);
    }
    #[inline]
    pub fn self_spatial_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_self_spatial_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(29, 1, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 2, value as u64);
    }
    #[inline]
    pub fn external_stamp_l0_refidx0_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l0_refidx0_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l0_refidx1_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(35, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l0_refidx1_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(35, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l0_refidx2_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(38, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l0_refidx2_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(38, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l0_refidx3_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(41, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l0_refidx3_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(41, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l0_refidx4_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(44, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l0_refidx4_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(44, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l0_refidx5_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(47, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l0_refidx5_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(47, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l0_refidx6_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(50, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l0_refidx6_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(50, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l0_refidx7_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(53, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l0_refidx7_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(53, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l1_refidx0_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(56, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l1_refidx0_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(56, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l1_refidx1_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(59, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l1_refidx1_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(59, 3, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(62, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(62, 2, value as u64);
    }
    #[inline]
    pub fn external_stamp_l1_refidx2_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(64, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l1_refidx2_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(64, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l1_refidx3_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(67, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l1_refidx3_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(67, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l1_refidx4_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(70, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l1_refidx4_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(70, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l1_refidx5_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(73, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l1_refidx5_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(73, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l1_refidx6_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(76, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l1_refidx6_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(76, 3, value as u64);
    }
    #[inline]
    pub fn external_stamp_l1_refidx7_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(79, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_stamp_l1_refidx7_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(79, 3, value as u64);
    }
    #[inline]
    pub fn external_explicit(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(82, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_explicit(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(82, 1, value as u64);
    }
    #[inline]
    pub fn external_search(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(83, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_search(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(83, 1, value as u64);
    }
    #[inline]
    pub fn external_refine(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(84, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_refine(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(84, 1, value as u64);
    }
    #[inline]
    pub fn external_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(85, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(85, 1, value as u64);
    }
    #[inline]
    pub fn const_mv_stamp_l0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(86, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_const_mv_stamp_l0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(86, 3, value as u64);
    }
    #[inline]
    pub fn const_mv_stamp_l1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(89, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_const_mv_stamp_l1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(89, 3, value as u64);
    }
    #[inline]
    pub fn const_mv_explicit(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(92, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_const_mv_explicit(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(92, 1, value as u64);
    }
    #[inline]
    pub fn const_mv_search(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(93, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_const_mv_search(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(93, 1, value as u64);
    }
    #[inline]
    pub fn const_mv_refine(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(94, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_const_mv_refine(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(94, 1, value as u64);
    }
    #[inline]
    pub fn const_mv_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(95, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_const_mv_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(95, 1, value as u64);
    }
    #[inline]
    pub fn stamp_refidx1_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(96, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stamp_refidx1_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(96, 3, value as u64);
    }
    #[inline]
    pub fn stamp_refidx2_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(99, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stamp_refidx2_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(99, 3, value as u64);
    }
    #[inline]
    pub fn stamp_refidx3_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(102, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stamp_refidx3_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(102, 3, value as u64);
    }
    #[inline]
    pub fn stamp_refidx4_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(105, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stamp_refidx4_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(105, 3, value as u64);
    }
    #[inline]
    pub fn stamp_refidx5_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(108, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stamp_refidx5_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(108, 3, value as u64);
    }
    #[inline]
    pub fn stamp_refidx6_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(111, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stamp_refidx6_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(111, 3, value as u64);
    }
    #[inline]
    pub fn stamp_refidx7_stamp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(114, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stamp_refidx7_stamp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(114, 3, value as u64);
    }
    #[inline]
    pub fn reserved2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(117, 11) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(117, 11, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_me_const_vec_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_h264_me_const_vec_s {
    #[inline]
    pub fn mvx_frac(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mvx_frac(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn mvx_int(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(2, 12) << (64 - 12)) as i64 >> (64 - 12)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_mvx_int(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(2, 12, value as u64);
    }
    #[inline]
    pub fn reserved_x(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(14, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved_x(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(14, 2, value as u64);
    }
    #[inline]
    pub fn mvy_frac(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mvy_frac(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 2, value as u64);
    }
    #[inline]
    pub fn mvy_int(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(18, 10) << (64 - 10)) as i64 >> (64 - 10)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_mvy_int(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(18, 10, value as u64);
    }
    #[inline]
    pub fn reserved_y(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved_y(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 4, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_me_stamp_s {
    pub bitmask: [::std::os::raw::c_uint; 2usize],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_h264_me_stamp_s {
    #[inline]
    pub fn hor_adjust(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_hor_adjust(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 1, value as u64);
    }
    #[inline]
    pub fn ver_adjust(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(1, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ver_adjust(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(1, 1, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 30) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 30, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_me_limit_mv_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 8usize]>,
}
impl nvenc_h264_me_limit_mv_s {
    #[inline]
    pub fn mv_limit_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mv_limit_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 1, value as u64);
    }
    #[inline]
    pub fn left_mvx_frac(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(1, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_left_mvx_frac(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(1, 2, value as u64);
    }
    #[inline]
    pub fn left_mvx_int(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(3, 12) << (64 - 12)) as i64 >> (64 - 12)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_left_mvx_int(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(3, 12, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(15, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(15, 1, value as u64);
    }
    #[inline]
    pub fn top_mvy_frac(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_top_mvy_frac(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 2, value as u64);
    }
    #[inline]
    pub fn top_mvy_int(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(18, 10) << (64 - 10)) as i64 >> (64 - 10)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_top_mvy_int(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(18, 10, value as u64);
    }
    #[inline]
    pub fn reserved2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 4, value as u64);
    }
    #[inline]
    pub fn right_mvx_frac(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_right_mvx_frac(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 2, value as u64);
    }
    #[inline]
    pub fn right_mvx_int(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(34, 12) << (64 - 12)) as i64 >> (64 - 12)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_right_mvx_int(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(34, 12, value as u64);
    }
    #[inline]
    pub fn reserved3(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(46, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved3(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(46, 2, value as u64);
    }
    #[inline]
    pub fn bottom_mvy_frac(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(48, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bottom_mvy_frac(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(48, 2, value as u64);
    }
    #[inline]
    pub fn bottom_mvy_int(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(50, 10) << (64 - 10)) as i64 >> (64 - 10)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_bottom_mvy_int(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(50, 10, value as u64);
    }
    #[inline]
    pub fn reserved4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(60, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(60, 4, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_me_tebif_s {
    pub teb_K: ::std::os::raw::c_uchar,
    pub teb_N: ::std::os::raw::c_uchar,
    pub teb_P: ::std::os::raw::c_uchar,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 1usize]>,
}
impl nvenc_me_tebif_s {
    #[inline]
    pub fn teb_S(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(0, 4) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_teb_S(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(0, 4, value as u64);
    }
    #[inline]
    pub fn teb_mode(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(4, 4) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_teb_mode(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(4, 4, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvenc_h264_me_control_s {
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub limit_mv: nvenc_h264_me_limit_mv_s,
    pub predsrc: nvenc_h264_me_hint_cfg_s,
    pub l0_hint: nvenc_h264_me_const_vec_s,
    pub l1_hint: nvenc_h264_me_const_vec_s,
    pub l0_pred: nvenc_h264_me_const_vec_s,
    pub l1_pred: nvenc_h264_me_const_vec_s,
    pub shape0: nvenc_h264_me_stamp_s,
    pub shape1: nvenc_h264_me_stamp_s,
    pub shape2: nvenc_h264_me_stamp_s,
    pub shape3: nvenc_h264_me_stamp_s,
    pub shape4: nvenc_h264_me_stamp_s,
    pub shape5: nvenc_h264_me_stamp_s,
    pub shape6: nvenc_h264_me_stamp_s,
    pub shape7: nvenc_h264_me_stamp_s,
    pub teb_para: nvenc_me_tebif_s,
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 8usize]>,
    pub ct_threshold: ::std::os::raw::c_short,
    pub _bitfield_3: __BindgenBitfieldUnit<[u8; 5usize]>,
    pub reserved: [::std::os::raw::c_uchar; 33usize],
}
impl Default for nvenc_h264_me_control_s {
    fn default() -> Self {
        Self {
            _bitfield_1: Default::default(),
            limit_mv: Default::default(),
            predsrc: Default::default(),
            l0_hint: Default::default(),
            l1_hint: Default::default(),
            l0_pred: Default::default(),
            l1_pred: Default::default(),
            shape0: Default::default(),
            shape1: Default::default(),
            shape2: Default::default(),
            shape3: Default::default(),
            shape4: Default::default(),
            shape5: Default::default(),
            shape6: Default::default(),
            shape7: Default::default(),
            teb_para: Default::default(),
            _bitfield_2: Default::default(),
            ct_threshold: Default::default(),
            _bitfield_3: Default::default(),
            reserved: [Default::default(); 33usize],
        }
    }
}
impl nvenc_h264_me_control_s {
    #[inline]
    pub fn me_predictor_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_predictor_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 1, value as u64);
    }
    #[inline]
    pub fn refinement_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(1, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_refinement_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(1, 1, value as u64);
    }
    #[inline]
    pub fn lambda_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_lambda_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 1, value as u64);
    }
    #[inline]
    pub fn const_lambda(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 11) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_const_lambda(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 11, value as u64);
    }
    #[inline]
    pub fn refine_on_search_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(14, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_refine_on_search_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(14, 1, value as u64);
    }
    #[inline]
    pub fn me_only_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(15, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_only_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(15, 1, value as u64);
    }
    #[inline]
    pub fn fps_mvcost(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_fps_mvcost(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 1, value as u64);
    }
    #[inline]
    pub fn sps_mvcost(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(17, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sps_mvcost(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(17, 1, value as u64);
    }
    #[inline]
    pub fn sps_cost_func(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sps_cost_func(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 1, value as u64);
    }
    #[inline]
    pub fn me_icc_mode_mad(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_icc_mode_mad(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 1, value as u64);
    }
    #[inline]
    pub fn sps_filter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sps_filter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 3, value as u64);
    }
    #[inline]
    pub fn mc_filter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(23, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mc_filter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(23, 3, value as u64);
    }
    #[inline]
    pub fn vc1_fastuv_mc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_vc1_fastuv_mc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 1, value as u64);
    }
    #[inline]
    pub fn vc1_mc_rnd(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_vc1_mc_rnd(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 1, value as u64);
    }
    #[inline]
    pub fn mbc_ctrl_arbitor(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mbc_ctrl_arbitor(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 2, value as u64);
    }
    #[inline]
    pub fn mv_only_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mv_only_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 1, value as u64);
    }
    #[inline]
    pub fn average_mvhint_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(31, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_average_mvhint_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(31, 1, value as u64);
    }
    #[inline]
    pub fn mbc_mb_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(0, 9) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mbc_mb_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(0, 9, value as u64);
    }
    #[inline]
    pub fn rmvp_source_half_scaled(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(9, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_rmvp_source_half_scaled(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(9, 1, value as u64);
    }
    #[inline]
    pub fn partDecisionMadeByFPP(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(10, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_partDecisionMadeByFPP(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(10, 1, value as u64);
    }
    #[inline]
    pub fn penalty_factor_FPP(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(11, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_penalty_factor_FPP(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(11, 6, value as u64);
    }
    #[inline]
    pub fn spatial_hint_pattern(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(17, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_spatial_hint_pattern(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(17, 4, value as u64);
    }
    #[inline]
    pub fn temporal_hint_pattern(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(21, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_temporal_hint_pattern(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(21, 6, value as u64);
    }
    #[inline]
    pub fn Cu16partDecisionMadeByFPP(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(27, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_Cu16partDecisionMadeByFPP(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(27, 1, value as u64);
    }
    #[inline]
    pub fn sps_mvp_from_fbm(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(28, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sps_mvp_from_fbm(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(28, 1, value as u64);
    }
    #[inline]
    pub fn fbm_select_best_cu16_parttype_num(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(29, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_fbm_select_best_cu16_parttype_num(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(29, 3, value as u64);
    }
    #[inline]
    pub fn fbm_op_winner_num_p_frame(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(32, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_fbm_op_winner_num_p_frame(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(32, 3, value as u64);
    }
    #[inline]
    pub fn fbm_op_winner_num_b_frame_l0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(35, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_fbm_op_winner_num_b_frame_l0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(35, 3, value as u64);
    }
    #[inline]
    pub fn fbm_op_winner_num_b_frame_l1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(38, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_fbm_op_winner_num_b_frame_l1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(38, 3, value as u64);
    }
    #[inline]
    pub fn fbm_select_best_cu32_parttype_num(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(41, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_fbm_select_best_cu32_parttype_num(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(41, 3, value as u64);
    }
    #[inline]
    pub fn sps_evaluate_merge_cand_num(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(44, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sps_evaluate_merge_cand_num(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(44, 3, value as u64);
    }
    #[inline]
    pub fn fps_quad_thresh_hold(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(47, 10) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_fps_quad_thresh_hold(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(47, 10, value as u64);
    }
    #[inline]
    pub fn external_hint_order(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(57, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_external_hint_order(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(57, 1, value as u64);
    }
    #[inline]
    pub fn coloc_hint_order(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(58, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_coloc_hint_order(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(58, 1, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(59, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(59, 5, value as u64);
    }
    #[inline]
    pub fn hint_type0(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(0, 3) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_hint_type0(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(0, 3, value as u64);
    }
    #[inline]
    pub fn hint_type1(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(3, 3) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_hint_type1(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(3, 3, value as u64);
    }
    #[inline]
    pub fn hint_type2(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(6, 3) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_hint_type2(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(6, 3, value as u64);
    }
    #[inline]
    pub fn hint_type3(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(9, 3) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_hint_type3(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(9, 3, value as u64);
    }
    #[inline]
    pub fn hint_type4(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(12, 3) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_hint_type4(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(12, 3, value as u64);
    }
    #[inline]
    pub fn pyramidal_hint_order(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(15, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_pyramidal_hint_order(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(15, 1, value as u64);
    }
    #[inline]
    pub fn left_hint_delay_N(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(16, 4) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_left_hint_delay_N(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(16, 4, value as u64);
    }
    #[inline]
    pub fn reserved3(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(20, 4) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_reserved3(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(20, 4, value as u64);
    }
    #[inline]
    pub fn cu8partDecisionMadeByFPP(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(24, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu8partDecisionMadeByFPP(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(24, 1, value as u64);
    }
    #[inline]
    pub fn fbm_select_best_cu8_parttype_num(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(25, 3) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_fbm_select_best_cu8_parttype_num(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(25, 3, value as u64);
    }
    #[inline]
    pub fn fbm_op_winner_num_p_frame_cu8(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(28, 3) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_fbm_op_winner_num_p_frame_cu8(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(28, 3, value as u64);
    }
    #[inline]
    pub fn reserved4(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(31, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_reserved4(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(31, 1, value as u64);
    }
    #[inline]
    pub fn cu64partDecisionMadeByFPP(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(32, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64partDecisionMadeByFPP(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(32, 1, value as u64);
    }
    #[inline]
    pub fn fbm_select_best_cu64_parttype_num(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(33, 3) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_fbm_select_best_cu64_parttype_num(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(33, 3, value as u64);
    }
    #[inline]
    pub fn reserved5(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_3.get(36, 4) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_reserved5(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_3.set(36, 4, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_md_control_s {
    pub intra_refresh_cnt: ::std::os::raw::c_ushort,
    pub intra_refresh_offs: ::std::os::raw::c_ushort,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 8usize]>,
    pub bias_inter_16x16: ::std::os::raw::c_short,
    pub bias_inter_16x8: ::std::os::raw::c_short,
    pub bias_inter_8x16: ::std::os::raw::c_short,
    pub bias_inter_8x8: ::std::os::raw::c_short,
    pub bias_pskip: ::std::os::raw::c_short,
    pub bias_bdir: ::std::os::raw::c_short,
    pub bias_intra_over_inter: ::std::os::raw::c_short,
    pub bias_intra_16x16: ::std::os::raw::c_short,
    pub bias_intra_8x8: ::std::os::raw::c_short,
    pub bias_intra_4x4: ::std::os::raw::c_short,
    pub bias_intra_most_prob: ::std::os::raw::c_short,
    pub mv_cost_bias: ::std::os::raw::c_ushort,
    pub intra_nxn_bias_multiplier: ::std::os::raw::c_ushort,
    pub intra_most_prob_bias_multiplier: ::std::os::raw::c_ushort,
    pub pskip_bias_multiplier: ::std::os::raw::c_short,
    pub bdirect_bias_multiplier: ::std::os::raw::c_short,
    pub pskip_esc_threshold: ::std::os::raw::c_ushort,
    pub bdirect_esc_threshold: ::std::os::raw::c_ushort,
    pub early_intra_disable_mpeb_threshold: ::std::os::raw::c_ushort,
    pub tempmv_wt_spread_threshold: ::std::os::raw::c_ushort,
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub bias_favor_intra_16x16_early: ::std::os::raw::c_short,
    pub _bitfield_3: __BindgenBitfieldUnit<[u8; 6usize]>,
    pub bias_tu_4x4: ::std::os::raw::c_short,
    pub bias_tu_8x8: ::std::os::raw::c_short,
    pub _bitfield_4: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub ssd_over_ssim_factor: ::std::os::raw::c_uint,
    pub reserved6: [::std::os::raw::c_uint; 13usize],
}
impl nvenc_h264_md_control_s {
    #[inline]
    pub fn intra_luma4x4_mode_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 9) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_luma4x4_mode_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 9, value as u64);
    }
    #[inline]
    pub fn intra_luma8x8_mode_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(9, 9) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_luma8x8_mode_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(9, 9, value as u64);
    }
    #[inline]
    pub fn intra_luma16x16_mode_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_luma16x16_mode_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 4, value as u64);
    }
    #[inline]
    pub fn intra_chroma_mode_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(22, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_chroma_mode_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(22, 4, value as u64);
    }
    #[inline]
    pub fn inter_penalty_factor_for_ip1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_inter_penalty_factor_for_ip1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 6, value as u64);
    }
    #[inline]
    pub fn l0_part_16x16_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l0_part_16x16_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 1, value as u64);
    }
    #[inline]
    pub fn l0_part_16x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(33, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l0_part_16x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(33, 1, value as u64);
    }
    #[inline]
    pub fn l0_part_8x16_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(34, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l0_part_8x16_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(34, 1, value as u64);
    }
    #[inline]
    pub fn l0_part_8x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(35, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l0_part_8x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(35, 1, value as u64);
    }
    #[inline]
    pub fn l0_part_8x4_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(36, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l0_part_8x4_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(36, 1, value as u64);
    }
    #[inline]
    pub fn l0_part_4x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(37, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l0_part_4x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(37, 1, value as u64);
    }
    #[inline]
    pub fn l0_part_4x4_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(38, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l0_part_4x4_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(38, 1, value as u64);
    }
    #[inline]
    pub fn l0_part_reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(39, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l0_part_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(39, 1, value as u64);
    }
    #[inline]
    pub fn l1_part_16x16_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(40, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l1_part_16x16_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(40, 1, value as u64);
    }
    #[inline]
    pub fn l1_part_16x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(41, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l1_part_16x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(41, 1, value as u64);
    }
    #[inline]
    pub fn l1_part_8x16_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(42, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l1_part_8x16_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(42, 1, value as u64);
    }
    #[inline]
    pub fn l1_part_8x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(43, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l1_part_8x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(43, 1, value as u64);
    }
    #[inline]
    pub fn l1_part_reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(44, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_l1_part_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(44, 4, value as u64);
    }
    #[inline]
    pub fn bi_part_16x16_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(48, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bi_part_16x16_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(48, 1, value as u64);
    }
    #[inline]
    pub fn bi_part_16x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(49, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bi_part_16x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(49, 1, value as u64);
    }
    #[inline]
    pub fn bi_part_8x16_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(50, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bi_part_8x16_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(50, 1, value as u64);
    }
    #[inline]
    pub fn bi_part_8x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(51, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bi_part_8x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(51, 1, value as u64);
    }
    #[inline]
    pub fn bi_part_reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(52, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bi_part_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(52, 4, value as u64);
    }
    #[inline]
    pub fn bdirect_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(56, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bdirect_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(56, 2, value as u64);
    }
    #[inline]
    pub fn bskip_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(58, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bskip_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(58, 1, value as u64);
    }
    #[inline]
    pub fn pskip_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(59, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pskip_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(59, 1, value as u64);
    }
    #[inline]
    pub fn special_reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(60, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_special_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(60, 4, value as u64);
    }
    #[inline]
    pub fn tempmv_wt_distort_threshold(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(0, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tempmv_wt_distort_threshold(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(0, 16, value as u64);
    }
    #[inline]
    pub fn mv_cost_predictor_control(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(16, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mv_cost_predictor_control(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(16, 1, value as u64);
    }
    #[inline]
    pub fn mv_cost_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(17, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mv_cost_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(17, 1, value as u64);
    }
    #[inline]
    pub fn intra_most_prob_force_on(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(18, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_most_prob_force_on(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(18, 1, value as u64);
    }
    #[inline]
    pub fn early_intra_mode_control(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(19, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_early_intra_mode_control(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(19, 2, value as u64);
    }
    #[inline]
    pub fn early_intra_mode_type_16x16dc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(21, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_early_intra_mode_type_16x16dc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(21, 1, value as u64);
    }
    #[inline]
    pub fn early_intra_mode_type_16x16h(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(22, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_early_intra_mode_type_16x16h(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(22, 1, value as u64);
    }
    #[inline]
    pub fn early_intra_mode_type_16x16v(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(23, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_early_intra_mode_type_16x16v(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(23, 1, value as u64);
    }
    #[inline]
    pub fn early_ip_is_final(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(24, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_early_ip_is_final(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(24, 1, value as u64);
    }
    #[inline]
    pub fn chroma_eval_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(25, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_eval_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(25, 1, value as u64);
    }
    #[inline]
    pub fn ip_search_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(26, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ip_search_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(26, 3, value as u64);
    }
    #[inline]
    pub fn multiply_bias_with_lambda(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_multiply_bias_with_lambda(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(29, 1, value as u64);
    }
    #[inline]
    pub fn force_ipcm(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(30, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_force_ipcm(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(30, 1, value as u64);
    }
    #[inline]
    pub fn early_termination_ip1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(31, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_early_termination_ip1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(31, 1, value as u64);
    }
    #[inline]
    pub fn priority_ipred_type_ip1(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(0, 3) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_priority_ipred_type_ip1(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(0, 3, value as u64);
    }
    #[inline]
    pub fn intra_ssd_cnt_4x4(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(3, 4) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_intra_ssd_cnt_4x4(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(3, 4, value as u64);
    }
    #[inline]
    pub fn intra_ssd_cnt_8x8(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(7, 4) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_intra_ssd_cnt_8x8(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(7, 4, value as u64);
    }
    #[inline]
    pub fn intra_ssd_cnt_16x16(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(11, 4) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_intra_ssd_cnt_16x16(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(11, 4, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(15, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_skip_evaluate_enable(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(15, 1, value as u64);
    }
    #[inline]
    pub fn rdo_level(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(16, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_rdo_level(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(16, 3, value as u64);
    }
    #[inline]
    pub fn tu_search_num(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(19, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tu_search_num(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(19, 3, value as u64);
    }
    #[inline]
    pub fn luma_residual_zero_eval(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(22, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_luma_residual_zero_eval(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(22, 2, value as u64);
    }
    #[inline]
    pub fn num_1div8_lambda_ssd(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(24, 7) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_1div8_lambda_ssd(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(24, 7, value as u64);
    }
    #[inline]
    pub fn ofs_mvx_range(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(31, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ofs_mvx_range(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(31, 2, value as u64);
    }
    #[inline]
    pub fn reserved4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(33, 15) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(33, 15, value as u64);
    }
    #[inline]
    pub fn ssim_rdo(&self) -> ::std::os::raw::c_uint {
        self._bitfield_4.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ssim_rdo(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_4.set(0, 2, value as u64);
    }
    #[inline]
    pub fn calc_ssim(&self) -> ::std::os::raw::c_uint {
        self._bitfield_4.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_calc_ssim(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_4.set(2, 1, value as u64);
    }
    #[inline]
    pub fn reserved5(&self) -> ::std::os::raw::c_uint {
        self._bitfield_4.get(3, 29) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved5(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_4.set(3, 29, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_refpic_cmd_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_h264_refpic_cmd_s {
    #[inline]
    pub fn reorder_l0_cmd_count(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reorder_l0_cmd_count(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 4, value as u64);
    }
    #[inline]
    pub fn reorder_l1_cmd_count(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(4, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reorder_l1_cmd_count(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(4, 4, value as u64);
    }
    #[inline]
    pub fn mmco_cmd_count(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mmco_cmd_count(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 4, value as u64);
    }
    #[inline]
    pub fn no_output_of_prior_pic_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(12, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_no_output_of_prior_pic_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(12, 1, value as u64);
    }
    #[inline]
    pub fn long_term_ref_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_long_term_ref_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 1, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(14, 18) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(14, 18, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_ref_pic_reorder_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_h264_ref_pic_reorder_s {
    #[inline]
    pub fn reordering_of_pic_nums_idc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reordering_of_pic_nums_idc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 3, value as u64);
    }
    #[inline]
    pub fn abs_diff_pic_num_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 17) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_abs_diff_pic_num_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 17, value as u64);
    }
    #[inline]
    pub fn long_term_pic_num(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_long_term_pic_num(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 4, value as u64);
    }
    #[inline]
    pub fn abs_diff_view_idx_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_abs_diff_view_idx_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 4, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 4, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_mmco_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_h264_mmco_s {
    #[inline]
    pub fn mmco_cmd_id(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mmco_cmd_id(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 3, value as u64);
    }
    #[inline]
    pub fn difference_of_pic_nums_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 17) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_difference_of_pic_nums_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 17, value as u64);
    }
    #[inline]
    pub fn long_term_pic_num(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_long_term_pic_num(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 3, value as u64);
    }
    #[inline]
    pub fn long_term_frame_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(23, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_long_term_frame_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(23, 3, value as u64);
    }
    #[inline]
    pub fn max_long_term_frame_idx_plus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_max_long_term_frame_idx_plus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 4, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 2, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_slice_control_s {
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub slice_tgt_rate: ::std::os::raw::c_uint,
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 1usize]>,
    pub qp_slice_min: ::std::os::raw::c_uchar,
    pub _bitfield_3: __BindgenBitfieldUnit<[u8; 14usize]>,
    pub ref_pic_cmd_cfg: nvenc_h264_refpic_cmd_s,
    pub ref_pic_list_reorder_cmd: [[nvenc_h264_ref_pic_reorder_s; 8usize]; 2usize],
    pub mmco_cmd: [nvenc_h264_mmco_s; 8usize],
    pub reserved3: ::std::os::raw::c_uint,
}
impl nvenc_h264_slice_control_s {
    #[inline]
    pub fn num_mb(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 19) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_mb(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 19, value as u64);
    }
    #[inline]
    pub fn qp_avr(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_qp_avr(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 8, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 5, value as u64);
    }
    #[inline]
    pub fn slice_alpha_c0_offset_div2(&self) -> ::std::os::raw::c_char {
        self._bitfield_2.get(0, 4) as ::std::os::raw::c_char
    }
    #[inline]
    pub fn set_slice_alpha_c0_offset_div2(&mut self, value: ::std::os::raw::c_char) {
        self._bitfield_2.set(0, 4, value as u64);
    }
    #[inline]
    pub fn slice_beta_offset_div2(&self) -> ::std::os::raw::c_char {
        self._bitfield_2.get(4, 4) as ::std::os::raw::c_char
    }
    #[inline]
    pub fn set_slice_beta_offset_div2(&mut self, value: ::std::os::raw::c_char) {
        self._bitfield_2.set(4, 4, value as u64);
    }
    #[inline]
    pub fn qp_slice_max(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(0, 8) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_qp_slice_max(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(0, 8, value as u64);
    }
    #[inline]
    pub fn force_intra(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(8, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_force_intra(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(8, 1, value as u64);
    }
    #[inline]
    pub fn disable_deblocking_filter_idc(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(9, 2) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_disable_deblocking_filter_idc(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(9, 2, value as u64);
    }
    #[inline]
    pub fn cabac_init_idc(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(11, 2) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_cabac_init_idc(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(11, 2, value as u64);
    }
    #[inline]
    pub fn disable_slice_header(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(13, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_disable_slice_header(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(13, 1, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_3.get(14, 2) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_3.set(14, 2, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_active_override_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(16, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_active_override_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(16, 1, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l0_active_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(17, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l0_active_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(17, 5, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l1_active_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(22, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l1_active_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(22, 5, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(27, 9) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(27, 9, value as u64);
    }
    #[inline]
    pub fn me_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(36, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(36, 4, value as u64);
    }
    #[inline]
    pub fn md_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(40, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_md_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(40, 4, value as u64);
    }
    #[inline]
    pub fn q_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(44, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_q_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(44, 4, value as u64);
    }
    #[inline]
    pub fn limit_slice_top_boundary(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(48, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_limit_slice_top_boundary(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(48, 1, value as u64);
    }
    #[inline]
    pub fn limit_slice_bot_boundary(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(49, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_limit_slice_bot_boundary(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(49, 1, value as u64);
    }
    #[inline]
    pub fn limit_slice_left_boundary(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(50, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_limit_slice_left_boundary(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(50, 1, value as u64);
    }
    #[inline]
    pub fn limit_slice_right_boundary(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(51, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_limit_slice_right_boundary(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(51, 1, value as u64);
    }
    #[inline]
    pub fn ROI_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(52, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(52, 1, value as u64);
    }
    #[inline]
    pub fn ROI_me_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(53, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_me_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(53, 4, value as u64);
    }
    #[inline]
    pub fn ROI_md_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(57, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_md_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(57, 4, value as u64);
    }
    #[inline]
    pub fn ROI_q_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(61, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_q_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(61, 4, value as u64);
    }
    #[inline]
    pub fn ROI_qp_delta(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(65, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_qp_delta(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(65, 8, value as u64);
    }
    #[inline]
    pub fn wp_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(73, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_wp_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(73, 4, value as u64);
    }
    #[inline]
    pub fn reserved2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(77, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(77, 3, value as u64);
    }
    #[inline]
    pub fn ROI_top_mbx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(80, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_top_mbx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(80, 8, value as u64);
    }
    #[inline]
    pub fn ROI_top_mby(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(88, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_top_mby(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(88, 8, value as u64);
    }
    #[inline]
    pub fn ROI_bot_mbx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(96, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_bot_mbx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(96, 8, value as u64);
    }
    #[inline]
    pub fn ROI_bot_mby(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(104, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_bot_mby(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(104, 8, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_rc_s {
    pub hrd_type: ::std::os::raw::c_uchar,
    pub QP: [::std::os::raw::c_uchar; 3usize],
    pub minQP: [::std::os::raw::c_uchar; 3usize],
    pub maxQP: [::std::os::raw::c_uchar; 3usize],
    pub maxQPD: ::std::os::raw::c_uchar,
    pub baseQPD: ::std::os::raw::c_uchar,
    pub rhopbi: [::std::os::raw::c_int; 3usize],
    pub framerate: ::std::os::raw::c_int,
    pub buffersize: ::std::os::raw::c_uint,
    pub nal_cpb_size: ::std::os::raw::c_int,
    pub nal_bitrate: ::std::os::raw::c_int,
    pub vcl_cpb_size: ::std::os::raw::c_int,
    pub vcl_bitrate: ::std::os::raw::c_int,
    pub gop_length: ::std::os::raw::c_uint,
    pub Np: ::std::os::raw::c_int,
    pub Bmin: ::std::os::raw::c_int,
    pub Ravg: ::std::os::raw::c_int,
    pub R: ::std::os::raw::c_int,
    pub ab_alpha: ::std::os::raw::c_uchar,
    pub ab_beta: ::std::os::raw::c_uchar,
    pub prev_act: ::std::os::raw::c_ushort,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 1usize]>,
    pub single_frame_VBV: ::std::os::raw::c_uchar,
    pub two_pass_rc: ::std::os::raw::c_uchar,
    pub rc_class: ::std::os::raw::c_uchar,
    pub first_pass_source_half_scaled: ::std::os::raw::c_uchar,
    pub iSizeRatioX: ::std::os::raw::c_uchar,
    pub iSizeRatioY: ::std::os::raw::c_uchar,
    pub ext_scene_change_flag: ::std::os::raw::c_uchar,
    pub ext_intra_satd: ::std::os::raw::c_uint,
    pub ext_picture_rc_hints: ::std::os::raw::c_uchar,
    pub session_max_qp: ::std::os::raw::c_uchar,
    pub reserve: [::std::os::raw::c_uchar; 2usize],
}
impl nvenc_h264_rc_s {
    #[inline]
    pub fn aqMode(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(0, 3) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_aqMode(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(0, 3, value as u64);
    }
    #[inline]
    pub fn dump_aq_stats(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(3, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_dump_aq_stats(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(3, 1, value as u64);
    }
    #[inline]
    pub fn is_emphasis_level(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(4, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_is_emphasis_level(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(4, 1, value as u64);
    }
    #[inline]
    pub fn reserve0(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(5, 3) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_reserve0(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(5, 3, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_pic_control_s {
    pub l0: [::std::os::raw::c_char; 8usize],
    pub l1: [::std::os::raw::c_char; 8usize],
    pub temp_dist_l0: [::std::os::raw::c_char; 8usize],
    pub temp_dist_l1: [::std::os::raw::c_char; 8usize],
    pub dist_scale_factor: [[::std::os::raw::c_short; 8usize]; 8usize],
    pub diff_pic_order_cnt_zero: [::std::os::raw::c_uint; 2usize],
    pub max_slice_size: ::std::os::raw::c_uint,
    pub max_byte_count_before_resid_zero: ::std::os::raw::c_uint,
    pub delta_pic_order_cnt_bottom: ::std::os::raw::c_int,
    pub frame_num: ::std::os::raw::c_ushort,
    pub pic_order_cnt_lsb: ::std::os::raw::c_ushort,
    pub idr_pic_id: ::std::os::raw::c_ushort,
    pub colour_plane_id: ::std::os::raw::c_ushort,
    pub longTermFlag: ::std::os::raw::c_uint,
    pub num_forced_slices_minus1: ::std::os::raw::c_uchar,
    pub num_me_controls_minus1: ::std::os::raw::c_uchar,
    pub num_md_controls_minus1: ::std::os::raw::c_uchar,
    pub num_q_controls_minus1: ::std::os::raw::c_uchar,
    pub slice_control_offset: ::std::os::raw::c_uint,
    pub me_control_offset: ::std::os::raw::c_uint,
    pub md_control_offset: ::std::os::raw::c_uint,
    pub q_control_offset: ::std::os::raw::c_uint,
    pub hist_buf_size: ::std::os::raw::c_uint,
    pub bitstream_buf_size: ::std::os::raw::c_uint,
    pub bitstream_start_pos: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub cur_interview_ref_pic: ::std::os::raw::c_char,
    pub prev_interview_ref_pic: ::std::os::raw::c_char,
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 1usize]>,
    pub intraRefreshCount: ::std::os::raw::c_uchar,
    pub _bitfield_3: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub slice_stat_offset: ::std::os::raw::c_uint,
    pub mpec_stat_offset: ::std::os::raw::c_uint,
    pub wp_control_offset: ::std::os::raw::c_uint,
    pub num_wp_controls_minus1: ::std::os::raw::c_uchar,
    pub _bitfield_4: __BindgenBitfieldUnit<[u8; 1usize]>,
    pub stripID: ::std::os::raw::c_uchar,
    pub strips_in_frame: ::std::os::raw::c_uchar,
    pub slice_encoding_row_start: ::std::os::raw::c_ushort,
    pub slice_encoding_row_num: ::std::os::raw::c_ushort,
    pub _bitfield_5: __BindgenBitfieldUnit<[u8; 1usize]>,
    pub segment_P2: ::std::os::raw::c_uchar,
    pub segment_thr: ::std::os::raw::c_uchar,
    pub _bitfield_6: __BindgenBitfieldUnit<[u8; 5usize]>,
    pub aq_stat_offset: ::std::os::raw::c_uint,
    pub act_stat_offset: ::std::os::raw::c_uint,
    pub stats_fifo_offset: ::std::os::raw::c_uint,
}
impl nvenc_h264_pic_control_s {
    #[inline]
    pub fn pic_struct(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_struct(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn pic_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 2, value as u64);
    }
    #[inline]
    pub fn ref_pic_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(4, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ref_pic_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(4, 1, value as u64);
    }
    #[inline]
    pub fn slice_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(5, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slice_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(5, 1, value as u64);
    }
    #[inline]
    pub fn ipcm_rewind_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(6, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ipcm_rewind_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(6, 1, value as u64);
    }
    #[inline]
    pub fn cabac_zero_word_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cabac_zero_word_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 1, value as u64);
    }
    #[inline]
    pub fn base_view(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_base_view(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 1, value as u64);
    }
    #[inline]
    pub fn priority_id(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(9, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_priority_id(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(9, 6, value as u64);
    }
    #[inline]
    pub fn view_id(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(15, 10) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_view_id(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(15, 10, value as u64);
    }
    #[inline]
    pub fn temporal_id(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(25, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_temporal_id(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(25, 3, value as u64);
    }
    #[inline]
    pub fn anchor_pic_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_anchor_pic_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 1, value as u64);
    }
    #[inline]
    pub fn inter_view_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_inter_view_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(29, 1, value as u64);
    }
    #[inline]
    pub fn opm_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_opm_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 1, value as u64);
    }
    #[inline]
    pub fn temporal_hint8x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(31, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_temporal_hint8x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(31, 1, value as u64);
    }
    #[inline]
    pub fn codec(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_2.get(0, 3) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_codec(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_2.set(0, 3, value as u64);
    }
    #[inline]
    pub fn e4byteStartCode(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_2.get(3, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_e4byteStartCode(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_2.set(3, 1, value as u64);
    }
    #[inline]
    pub fn qpfifo(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_2.get(4, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_qpfifo(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_2.set(4, 1, value as u64);
    }
    #[inline]
    pub fn me_candidates_output_struct(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(5, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_candidates_output_struct(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(5, 1, value as u64);
    }
    #[inline]
    pub fn me_candidates_output_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(6, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_candidates_output_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(6, 1, value as u64);
    }
    #[inline]
    pub fn medma_output_cost_mode(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_2.get(7, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_medma_output_cost_mode(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_2.set(7, 1, value as u64);
    }
    #[inline]
    pub fn mpec_threshold(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(0, 30) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mpec_threshold(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(0, 30, value as u64);
    }
    #[inline]
    pub fn me_lambda_fifo_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(30, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_lambda_fifo_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(30, 1, value as u64);
    }
    #[inline]
    pub fn me_candidates_rdo_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(31, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_candidates_rdo_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(31, 1, value as u64);
    }
    #[inline]
    pub fn b_as_ref(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_4.get(0, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_b_as_ref(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_4.set(0, 1, value as u64);
    }
    #[inline]
    pub fn ofs_mode(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_4.get(1, 2) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_ofs_mode(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_4.set(1, 2, value as u64);
    }
    #[inline]
    pub fn new_subframe(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_4.get(3, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_new_subframe(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_4.set(3, 1, value as u64);
    }
    #[inline]
    pub fn bit_depth_minus_8(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_4.get(4, 4) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_bit_depth_minus_8(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_4.set(4, 4, value as u64);
    }
    #[inline]
    pub fn segment_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_5.get(0, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_segment_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_5.set(0, 1, value as u64);
    }
    #[inline]
    pub fn background_seg(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_5.get(1, 5) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_background_seg(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_5.set(1, 5, value as u64);
    }
    #[inline]
    pub fn input_sub_sampling(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_5.get(6, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_input_sub_sampling(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_5.set(6, 1, value as u64);
    }
    #[inline]
    pub fn output_sub_sampling(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_5.get(7, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_output_sub_sampling(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_5.set(7, 1, value as u64);
    }
    #[inline]
    pub fn accurate_skip_mv(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_6.get(0, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_accurate_skip_mv(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_6.set(0, 1, value as u64);
    }
    #[inline]
    pub fn intra_sel_4x4_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_6.get(1, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_intra_sel_4x4_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_6.set(1, 1, value as u64);
    }
    #[inline]
    pub fn enable_source_image_padding(&self) -> ::std::os::raw::c_uint {
        self._bitfield_6.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_source_image_padding(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_6.set(2, 1, value as u64);
    }
    #[inline]
    pub fn luma_padding_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_6.get(3, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_luma_padding_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_6.set(3, 2, value as u64);
    }
    #[inline]
    pub fn chroma_padding_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_6.get(5, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_padding_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_6.set(5, 2, value as u64);
    }
    #[inline]
    pub fn stats_fifo_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_6.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stats_fifo_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_6.set(7, 1, value as u64);
    }
    #[inline]
    pub fn chroma_skip_threshold_4x4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_6.get(8, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_skip_threshold_4x4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_6.set(8, 16, value as u64);
    }
    #[inline]
    pub fn intra_sel_4x4_threshold(&self) -> ::std::os::raw::c_uint {
        self._bitfield_6.get(24, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_sel_4x4_threshold(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_6.set(24, 16, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h264_drv_pic_setup_s {
    pub magic: ::std::os::raw::c_uint,
    pub refpic_cfg: nvenc_h264_surface_cfg_s,
    pub input_cfg: nvenc_h264_surface_cfg_s,
    pub outputpic_cfg: nvenc_h264_surface_cfg_s,
    pub sps_data: nvenc_h264_sps_data_s,
    pub pps_data: nvenc_h264_pps_data_s,
    pub rate_control: nvenc_h264_rc_s,
    pub pic_control: nvenc_h264_pic_control_s,
    pub half_scaled_outputpic_cfg: nvenc_h264_surface_cfg_s,
    pub gpTimer_timeout_val: ::std::os::raw::c_uint,
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_surface_cfg_s {
    pub frame_width_minus1: ::std::os::raw::c_ushort,
    pub frame_height_minus1: ::std::os::raw::c_ushort,
    pub sfc_pitch: ::std::os::raw::c_ushort,
    pub sfc_pitch_chroma: ::std::os::raw::c_ushort,
    pub sfc_trans_mode: ::std::os::raw::c_ushort,
    pub reserved2: ::std::os::raw::c_ushort,
    pub luma_top_frm_offset: ::std::os::raw::c_uint,
    pub luma_bot_offset: ::std::os::raw::c_uint,
    pub chroma_top_frm_offset: ::std::os::raw::c_uint,
    pub chroma_bot_offset: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_h265_surface_cfg_s {
    #[inline]
    pub fn block_height(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 7) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_block_height(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 7, value as u64);
    }
    #[inline]
    pub fn tiled_16x16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tiled_16x16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 1, value as u64);
    }
    #[inline]
    pub fn memory_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_memory_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 2, value as u64);
    }
    #[inline]
    pub fn nv21_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(10, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_nv21_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(10, 1, value as u64);
    }
    #[inline]
    pub fn input_bl_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(11, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_input_bl_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(11, 2, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 19) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 19, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_sps_data_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 8usize]>,
}
impl nvenc_h265_sps_data_s {
    #[inline]
    pub fn chroma_format_idc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_format_idc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn separate_colour_plane_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_separate_colour_plane_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 1, value as u64);
    }
    #[inline]
    pub fn log2_max_pic_order_cnt_lsb_minus4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_log2_max_pic_order_cnt_lsb_minus4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 4, value as u64);
    }
    #[inline]
    pub fn min_cu_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_min_cu_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 2, value as u64);
    }
    #[inline]
    pub fn max_cu_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(9, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_max_cu_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(9, 2, value as u64);
    }
    #[inline]
    pub fn min_tu_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(11, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_min_tu_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(11, 2, value as u64);
    }
    #[inline]
    pub fn max_tu_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_max_tu_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 2, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(15, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(15, 1, value as u64);
    }
    #[inline]
    pub fn max_transform_depth_inter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_max_transform_depth_inter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 2, value as u64);
    }
    #[inline]
    pub fn max_transform_depth_intra(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_max_transform_depth_intra(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 2, value as u64);
    }
    #[inline]
    pub fn pcm_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pcm_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 1, value as u64);
    }
    #[inline]
    pub fn pcm_loop_filter_disable_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(21, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pcm_loop_filter_disable_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(21, 1, value as u64);
    }
    #[inline]
    pub fn long_term_ref_pics_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(22, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_long_term_ref_pics_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(22, 1, value as u64);
    }
    #[inline]
    pub fn amp_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(23, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_amp_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(23, 1, value as u64);
    }
    #[inline]
    pub fn strong_intra_smoothing_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_strong_intra_smoothing_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 1, value as u64);
    }
    #[inline]
    pub fn num_short_term_ref_pic_sets(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(25, 7) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_short_term_ref_pic_sets(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(25, 7, value as u64);
    }
    #[inline]
    pub fn num_long_term_ref_pics_sps(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_long_term_ref_pics_sps(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 5, value as u64);
    }
    #[inline]
    pub fn bit_depth_minus_8(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(37, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bit_depth_minus_8(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(37, 4, value as u64);
    }
    #[inline]
    pub fn sample_adaptive_offset_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(41, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sample_adaptive_offset_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(41, 1, value as u64);
    }
    #[inline]
    pub fn sps_temporal_mvp_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(42, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sps_temporal_mvp_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(42, 1, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(43, 21) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(43, 21, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_pps_data_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 8usize]>,
}
impl nvenc_h265_pps_data_s {
    #[inline]
    pub fn pic_param_set_id(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_param_set_id(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 8, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l0_active_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l0_active_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 5, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l1_active_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l1_active_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 5, value as u64);
    }
    #[inline]
    pub fn constrained_intra_pred_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_constrained_intra_pred_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 1, value as u64);
    }
    #[inline]
    pub fn transquant_bypass_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_transquant_bypass_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 1, value as u64);
    }
    #[inline]
    pub fn transform_skip_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_transform_skip_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 1, value as u64);
    }
    #[inline]
    pub fn deblocking_filter_override_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(21, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_deblocking_filter_override_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(21, 1, value as u64);
    }
    #[inline]
    pub fn pps_loop_filter_across_slices_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(22, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pps_loop_filter_across_slices_enabled_flag(
        &mut self,
        value: ::std::os::raw::c_uint,
    ) {
        self._bitfield_1.set(22, 1, value as u64);
    }
    #[inline]
    pub fn cu_qp_delta_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(23, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu_qp_delta_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(23, 1, value as u64);
    }
    #[inline]
    pub fn lists_modification_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_lists_modification_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 1, value as u64);
    }
    #[inline]
    pub fn output_flag_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(25, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_output_flag_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(25, 1, value as u64);
    }
    #[inline]
    pub fn cabac_init_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cabac_init_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 1, value as u64);
    }
    #[inline]
    pub fn pps_slice_chroma_qp_offsets_present(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pps_slice_chroma_qp_offsets_present(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 1, value as u64);
    }
    #[inline]
    pub fn weighted_pred_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_weighted_pred_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 1, value as u64);
    }
    #[inline]
    pub fn weighted_bipred_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_weighted_bipred_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(29, 1, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 2, value as u64);
    }
    #[inline]
    pub fn pic_init_qp_minus26(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(32, 6) << (64 - 6)) as i64 >> (64 - 6)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_pic_init_qp_minus26(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(32, 6, value as u64);
    }
    #[inline]
    pub fn pps_cb_qp_offset(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(38, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_pps_cb_qp_offset(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(38, 5, value as u64);
    }
    #[inline]
    pub fn pps_cr_qp_offset(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(43, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_pps_cr_qp_offset(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(43, 5, value as u64);
    }
    #[inline]
    pub fn dependent_slice_segments_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(48, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_dependent_slice_segments_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(48, 1, value as u64);
    }
    #[inline]
    pub fn sign_data_hiding_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(49, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sign_data_hiding_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(49, 1, value as u64);
    }
    #[inline]
    pub fn diff_cu_qp_delta_depth(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(50, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_diff_cu_qp_delta_depth(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(50, 2, value as u64);
    }
    #[inline]
    pub fn log2_parallel_merge_level_minus2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(52, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_log2_parallel_merge_level_minus2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(52, 3, value as u64);
    }
    #[inline]
    pub fn num_extra_slice_header_bits(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(55, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_extra_slice_header_bits(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(55, 3, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(58, 6) << (64 - 6)) as i64 >> (64 - 6)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(58, 6, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_rc_s {
    pub hrd_type: ::std::os::raw::c_uchar,
    pub QP: [::std::os::raw::c_uchar; 3usize],
    pub minQP: [::std::os::raw::c_uchar; 3usize],
    pub maxQP: [::std::os::raw::c_uchar; 3usize],
    pub maxQPD: ::std::os::raw::c_uchar,
    pub baseQPD: ::std::os::raw::c_uchar,
    pub rhopbi: [::std::os::raw::c_int; 3usize],
    pub framerate: ::std::os::raw::c_int,
    pub buffersize: ::std::os::raw::c_uint,
    pub nal_cpb_size: ::std::os::raw::c_int,
    pub nal_bitrate: ::std::os::raw::c_int,
    pub vcl_cpb_size: ::std::os::raw::c_int,
    pub vcl_bitrate: ::std::os::raw::c_int,
    pub gop_length: ::std::os::raw::c_uint,
    pub Np: ::std::os::raw::c_int,
    pub Bmin: ::std::os::raw::c_int,
    pub Ravg: ::std::os::raw::c_int,
    pub R: ::std::os::raw::c_int,
    pub ab_alpha: ::std::os::raw::c_uchar,
    pub ab_beta: ::std::os::raw::c_uchar,
    pub prev_act: ::std::os::raw::c_ushort,
    pub reserved0: ::std::os::raw::c_uchar,
    pub single_frame_vbv: ::std::os::raw::c_uchar,
    pub two_pass_rc: ::std::os::raw::c_uchar,
    pub rc_class: ::std::os::raw::c_uchar,
    pub first_pass_source_half_scaled: ::std::os::raw::c_uchar,
    pub i_size_ratio_x: ::std::os::raw::c_uchar,
    pub i_size_ratio_y: ::std::os::raw::c_uchar,
    pub aqMode: ::std::os::raw::c_uchar,
    pub dump_aq_stats: ::std::os::raw::c_uchar,
    pub ext_scene_change_flag: ::std::os::raw::c_uchar,
    pub ext_picture_rc_hints: ::std::os::raw::c_uchar,
    pub session_max_qp: ::std::os::raw::c_uchar,
    pub ext_intra_satd: ::std::os::raw::c_uint,
    pub reserved2: [::std::os::raw::c_uchar; 8usize],
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_strps_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub used_by_curr_pic_flag: ::std::os::raw::c_ushort,
    pub use_delta_flag: ::std::os::raw::c_ushort,
    pub num_negative_pics: ::std::os::raw::c_short,
    pub num_positive_pics: ::std::os::raw::c_short,
    pub delta_poc_s0_minus1: [::std::os::raw::c_short; 16usize],
    pub used_by_curr_pic_s0_flag: ::std::os::raw::c_ushort,
    pub delta_poc_s1_minus1: [::std::os::raw::c_short; 16usize],
    pub used_by_curr_pic_s1_flag: ::std::os::raw::c_ushort,
}
impl nvenc_h265_strps_s {
    #[inline]
    pub fn inter_ref_pic_set_prediction_flag(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(0, 1) << (64 - 1)) as i64 >> (64 - 1)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_inter_ref_pic_set_prediction_flag(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(0, 1, value as u64);
    }
    #[inline]
    pub fn delta_idx_minus1(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(1, 6) << (64 - 6)) as i64 >> (64 - 6)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_delta_idx_minus1(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(1, 6, value as u64);
    }
    #[inline]
    pub fn delta_rps_sign(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(7, 1) << (64 - 1)) as i64 >> (64 - 1)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_delta_rps_sign(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(7, 1, value as u64);
    }
    #[inline]
    pub fn abs_delta_rps_minus1(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(8, 16) << (64 - 16)) as i64 >> (64 - 16)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_abs_delta_rps_minus1(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(8, 16, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(24, 8) << (64 - 8)) as i64 >> (64 - 8)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(24, 8, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_ltrps_s {
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub lt_idx_sps: [::std::os::raw::c_uchar; 16usize],
    pub poc_lsb_lt: [::std::os::raw::c_ushort; 16usize],
    pub used_by_curr_pic_lt_flag: ::std::os::raw::c_ushort,
    pub delta_poc_msb_present_flag: ::std::os::raw::c_ushort,
    pub delta_poc_msb_cycle_lt: [::std::os::raw::c_int; 16usize],
}
impl nvenc_h265_ltrps_s {
    #[inline]
    pub fn num_long_term_sps(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(0, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_num_long_term_sps(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(0, 5, value as u64);
    }
    #[inline]
    pub fn num_long_term_pics(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(5, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_num_long_term_pics(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(5, 5, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(10, 22) << (64 - 22)) as i64 >> (64 - 22)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(10, 22, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_ref_pic_list_modification_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub list_entry_l0: [::std::os::raw::c_uchar; 16usize],
    pub list_entry_l1: [::std::os::raw::c_uchar; 16usize],
}
impl nvenc_h265_ref_pic_list_modification_s {
    #[inline]
    pub fn NumPocTotalCurr(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(0, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_NumPocTotalCurr(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(0, 5, value as u64);
    }
    #[inline]
    pub fn ref_pic_list_mod_flag_l0(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(5, 1) << (64 - 1)) as i64 >> (64 - 1)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_ref_pic_list_mod_flag_l0(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(5, 1, value as u64);
    }
    #[inline]
    pub fn ref_pic_list_mod_flag_l1(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(6, 1) << (64 - 1)) as i64 >> (64 - 1)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_ref_pic_list_mod_flag_l1(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(6, 1, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(7, 25) << (64 - 25)) as i64 >> (64 - 25)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(7, 25, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_mv_hevc_cfg_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub slh_inter_layer_pred_layer_idc: [::std::os::raw::c_uchar; 8usize],
}
impl nvenc_mv_hevc_cfg_s {
    #[inline]
    pub fn vps_poc_lsb_not_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_vps_poc_lsb_not_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 1, value as u64);
    }
    #[inline]
    pub fn vps_default_ref_layers_active_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(1, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_vps_default_ref_layers_active_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(1, 1, value as u64);
    }
    #[inline]
    pub fn vps_max_one_active_ref_layer_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_vps_max_one_active_ref_layer_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 1, value as u64);
    }
    #[inline]
    pub fn vps_num_direct_ref_layers(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_vps_num_direct_ref_layers(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 6, value as u64);
    }
    #[inline]
    pub fn slh_discardable_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(9, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slh_discardable_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(9, 1, value as u64);
    }
    #[inline]
    pub fn slh_cross_layer_bla_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(10, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slh_cross_layer_bla_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(10, 1, value as u64);
    }
    #[inline]
    pub fn slh_num_active_ref_layer_pics(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(11, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slh_num_active_ref_layer_pics(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(11, 6, value as u64);
    }
    #[inline]
    pub fn slh_inter_layer_pred_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(17, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slh_inter_layer_pred_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(17, 1, value as u64);
    }
    #[inline]
    pub fn slh_num_inter_layer_ref_pics_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slh_num_inter_layer_ref_pics_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 6, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 8, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_pic_control_s {
    pub l0: [::std::os::raw::c_char; 8usize],
    pub l1: [::std::os::raw::c_char; 8usize],
    pub temp_dist_l0: [::std::os::raw::c_char; 8usize],
    pub temp_dist_l1: [::std::os::raw::c_char; 8usize],
    pub dist_scale_factor: [[::std::os::raw::c_short; 8usize]; 8usize],
    pub diff_pic_order_cnt_zero: [::std::os::raw::c_uint; 2usize],
    pub max_slice_size: ::std::os::raw::c_uint,
    pub max_byte_count_before_resid_zero: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 2usize]>,
    pub pic_order_cnt_lsb: ::std::os::raw::c_ushort,
    pub num_forced_slices_minus1: ::std::os::raw::c_uchar,
    pub num_me_controls_minus1: ::std::os::raw::c_uchar,
    pub num_md_controls_minus1: ::std::os::raw::c_uchar,
    pub num_q_controls_minus1: ::std::os::raw::c_uchar,
    pub slice_control_offset: ::std::os::raw::c_uint,
    pub me_control_offset: ::std::os::raw::c_uint,
    pub md_control_offset: ::std::os::raw::c_uint,
    pub q_control_offset: ::std::os::raw::c_uint,
    pub hist_buf_size: ::std::os::raw::c_uint,
    pub bitstream_buf_size: ::std::os::raw::c_uint,
    pub bitstream_start_pos: ::std::os::raw::c_uint,
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub mpec_threshold: ::std::os::raw::c_uint,
    pub slice_stat_offset: ::std::os::raw::c_uint,
    pub mpec_stat_offset: ::std::os::raw::c_uint,
    pub aq_stat_offset: ::std::os::raw::c_uint,
    pub _bitfield_3: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub short_term_rps: nvenc_h265_strps_s,
    pub long_term_rps: nvenc_h265_ltrps_s,
    pub ref_pic_list_modification: nvenc_h265_ref_pic_list_modification_s,
    pub rc_pass2_offset: ::std::os::raw::c_uint,
    pub wp_control_offset: ::std::os::raw::c_uint,
    pub num_wp_controls_minus1: ::std::os::raw::c_uchar,
    pub stripID: ::std::os::raw::c_uchar,
    pub strips_in_frame: ::std::os::raw::c_uchar,
    pub slice_idx_start: ::std::os::raw::c_uchar,
    pub slice_encoding_row_start: ::std::os::raw::c_ushort,
    pub slice_encoding_row_num: ::std::os::raw::c_ushort,
    pub mv_hevc_cfg: nvenc_mv_hevc_cfg_s,
    pub _bitfield_4: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub poc_entry_l0: [::std::os::raw::c_char; 8usize],
    pub poc_entry_l1: [::std::os::raw::c_char; 8usize],
    pub same_poc: [::std::os::raw::c_uint; 2usize],
    pub act_stat_offset: ::std::os::raw::c_uint,
    pub stats_fifo_offset: ::std::os::raw::c_uint,
}
impl nvenc_h265_pic_control_s {
    #[inline]
    pub fn no_output_of_prior_pics_flag(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_1.get(0, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_no_output_of_prior_pics_flag(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_1.set(0, 1, value as u64);
    }
    #[inline]
    pub fn pic_output_flag(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_1.get(1, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_pic_output_flag(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_1.set(1, 1, value as u64);
    }
    #[inline]
    pub fn mode_decision_only_flag(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_1.get(2, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_mode_decision_only_flag(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_1.set(2, 1, value as u64);
    }
    #[inline]
    pub fn cu_early_terminatiom_enable(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_1.get(3, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_cu_early_terminatiom_enable(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_1.set(3, 1, value as u64);
    }
    #[inline]
    pub fn input_sub_sampling(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_1.get(4, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_input_sub_sampling(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_1.set(4, 1, value as u64);
    }
    #[inline]
    pub fn output_sub_sampling(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_1.get(5, 1) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_output_sub_sampling(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_1.set(5, 1, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_ushort {
        self._bitfield_1.get(6, 10) as ::std::os::raw::c_ushort
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_ushort) {
        self._bitfield_1.set(6, 10, value as u64);
    }
    #[inline]
    pub fn pic_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(0, 2, value as u64);
    }
    #[inline]
    pub fn ref_pic_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ref_pic_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(2, 1, value as u64);
    }
    #[inline]
    pub fn slice_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(3, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slice_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(3, 1, value as u64);
    }
    #[inline]
    pub fn cabac_zero_word_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(4, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cabac_zero_word_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(4, 1, value as u64);
    }
    #[inline]
    pub fn codec(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(5, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_codec(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(5, 3, value as u64);
    }
    #[inline]
    pub fn ceil_log2_pic_size_in_ctbs(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(8, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ceil_log2_pic_size_in_ctbs(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(8, 6, value as u64);
    }
    #[inline]
    pub fn sao_luma_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(14, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sao_luma_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(14, 5, value as u64);
    }
    #[inline]
    pub fn sao_chroma_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(19, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sao_chroma_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(19, 5, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(24, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(24, 8, value as u64);
    }
    #[inline]
    pub fn short_term_ref_pic_set_sps_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(0, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_short_term_ref_pic_set_sps_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(0, 1, value as u64);
    }
    #[inline]
    pub fn short_term_ref_pic_set_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(1, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_short_term_ref_pic_set_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(1, 6, value as u64);
    }
    #[inline]
    pub fn e4byteStartCode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_e4byteStartCode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(7, 1, value as u64);
    }
    #[inline]
    pub fn nal_unit_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(8, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_nal_unit_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(8, 6, value as u64);
    }
    #[inline]
    pub fn nuh_layer_id(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(14, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_nuh_layer_id(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(14, 6, value as u64);
    }
    #[inline]
    pub fn nuh_temporal_id_plus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(20, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_nuh_temporal_id_plus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(20, 3, value as u64);
    }
    #[inline]
    pub fn qpfifo(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(23, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_qpfifo(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(23, 1, value as u64);
    }
    #[inline]
    pub fn enable_source_image_padding(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(24, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_source_image_padding(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(24, 1, value as u64);
    }
    #[inline]
    pub fn luma_padding_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(25, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_luma_padding_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(25, 2, value as u64);
    }
    #[inline]
    pub fn chroma_padding_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(27, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_padding_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(27, 2, value as u64);
    }
    #[inline]
    pub fn stats_fifo_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stats_fifo_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(29, 1, value as u64);
    }
    #[inline]
    pub fn reserved2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_3.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_3.set(30, 2, value as u64);
    }
    #[inline]
    pub fn chroma_skip_threshold_4x4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_4.get(0, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_skip_threshold_4x4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_4.set(0, 16, value as u64);
    }
    #[inline]
    pub fn longTermFlag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_4.get(16, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_longTermFlag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_4.set(16, 16, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvenc_h265_drv_pic_setup_s {
    pub magic: ::std::os::raw::c_uint,
    pub refpic_cfg: nvenc_h265_surface_cfg_s,
    pub input_cfg: nvenc_h265_surface_cfg_s,
    pub outputpic_cfg: nvenc_h265_surface_cfg_s,
    pub sps_data: nvenc_h265_sps_data_s,
    pub pps_data: nvenc_h265_pps_data_s,
    pub rate_control: nvenc_h265_rc_s,
    pub pic_control: nvenc_h265_pic_control_s,
    pub half_scaled_outputpic_cfg: nvenc_h265_surface_cfg_s,
    pub gpTimer_timeout_val: ::std::os::raw::c_uint,
    pub reserved: [::std::os::raw::c_uchar; 240usize],
}
impl Default for nvenc_h265_drv_pic_setup_s {
    fn default() -> Self {
        Self {
            magic: Default::default(),
            refpic_cfg: Default::default(),
            input_cfg: Default::default(),
            outputpic_cfg: Default::default(),
            sps_data: Default::default(),
            pps_data: Default::default(),
            rate_control: Default::default(),
            pic_control: Default::default(),
            half_scaled_outputpic_cfg: Default::default(),
            gpTimer_timeout_val: Default::default(),
            reserved: [Default::default(); 240usize],
        }
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_md_control_s {
    pub intra_refresh_cnt: ::std::os::raw::c_ushort,
    pub intra_refresh_offs: ::std::os::raw::c_ushort,
    pub h265_intra_luma4x4_mode_enable: ::std::os::raw::c_uint,
    pub h265_intra_luma8x8_mode_enable: ::std::os::raw::c_uint,
    pub h265_intra_luma16x16_mode_enable: ::std::os::raw::c_uint,
    pub h265_intra_luma32x32_mode_enable: ::std::os::raw::c_uint,
    pub h265_intra_luma64x64_mode_enable: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 12usize]>,
    pub bias_part_2Nx2N: ::std::os::raw::c_short,
    pub bias_part_2NxN: ::std::os::raw::c_short,
    pub bias_part_Nx2N: ::std::os::raw::c_short,
    pub bias_part_NxN: ::std::os::raw::c_short,
    pub bias_part_2NxnU: ::std::os::raw::c_short,
    pub bias_part_2NxnD: ::std::os::raw::c_short,
    pub bias_part_nLx2N: ::std::os::raw::c_short,
    pub bias_part_nRxN: ::std::os::raw::c_short,
    pub bias_cusize_8x8: ::std::os::raw::c_short,
    pub bias_cusize_16x16: ::std::os::raw::c_short,
    pub bias_cusize_32x32: ::std::os::raw::c_short,
    pub bias_cusize_64x64: ::std::os::raw::c_short,
    pub bias_pskip: ::std::os::raw::c_short,
    pub bias_intra_over_inter: ::std::os::raw::c_short,
    pub bias_intra_most_prob: ::std::os::raw::c_short,
    pub bias_qpp: ::std::os::raw::c_short,
    pub bias_transform_skip: ::std::os::raw::c_short,
    pub mv_cost_bias: ::std::os::raw::c_ushort,
    pub pskip_esc_threshold: ::std::os::raw::c_ushort,
    pub tempmv_wt_spread_threshold: ::std::os::raw::c_ushort,
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub early_termination_mode_ip1: ::std::os::raw::c_uint,
    pub bias_tu_4x4: ::std::os::raw::c_short,
    pub bias_tu_8x8: ::std::os::raw::c_short,
    pub bias_tu_16x16: ::std::os::raw::c_short,
    pub bias_tu_32x32: ::std::os::raw::c_short,
    pub _bitfield_3: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub bias_skip: ::std::os::raw::c_short,
    pub bias_intra_32x32: ::std::os::raw::c_short,
    pub bias_intra_16x16: ::std::os::raw::c_short,
    pub bias_intra_8x8: ::std::os::raw::c_short,
    pub bias_intra_4x4: ::std::os::raw::c_short,
    pub reserved: [::std::os::raw::c_uchar; 22usize],
}
impl nvenc_h265_md_control_s {
    #[inline]
    pub fn h265_intra_luma_mode_enable_leftbits(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 30) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_h265_intra_luma_mode_enable_leftbits(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 30, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 2, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_2Nx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(32, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu16x16_part_2Nx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(32, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_2NxN_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(33, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu16x16_part_2NxN_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(33, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_Nx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(34, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu16x16_part_Nx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(34, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_NxN_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(35, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu16x16_part_NxN_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(35, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_2NxnU_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(36, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu16x16_part_2NxnU_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(36, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_2NxnD_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(37, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu16x16_part_2NxnD_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(37, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_nLx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(38, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu16x16_part_nLx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(38, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_nRx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(39, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu16x16_part_nRx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(39, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_2Nx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(40, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu32x32_part_2Nx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(40, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_2NxN_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(41, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu32x32_part_2NxN_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(41, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_Nx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(42, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu32x32_part_Nx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(42, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_NxN_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(43, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu32x32_part_NxN_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(43, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_2NxnU_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(44, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu32x32_part_2NxnU_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(44, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_2NxnD_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(45, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu32x32_part_2NxnD_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(45, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_nLx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(46, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu32x32_part_nLx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(46, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_nRx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(47, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu32x32_part_nRx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(47, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_2Nx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(48, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64x64_part_2Nx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(48, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_2NxN_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(49, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64x64_part_2NxN_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(49, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_Nx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(50, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64x64_part_Nx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(50, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_NxN_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(51, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64x64_part_NxN_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(51, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_2NxnU_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(52, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64x64_part_2NxnU_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(52, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_2NxnD_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(53, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64x64_part_2NxnD_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(53, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_nLx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(54, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64x64_part_nLx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(54, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_nRx2N_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(55, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_cu64x64_part_nRx2N_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(55, 1, value as u64);
    }
    #[inline]
    pub fn intra_mode_4x4_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(56, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_intra_mode_4x4_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(56, 1, value as u64);
    }
    #[inline]
    pub fn intra_mode_8x8_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(57, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_intra_mode_8x8_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(57, 1, value as u64);
    }
    #[inline]
    pub fn intra_mode_16x16_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(58, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_intra_mode_16x16_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(58, 1, value as u64);
    }
    #[inline]
    pub fn intra_mode_32x32_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(59, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_intra_mode_32x32_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(59, 1, value as u64);
    }
    #[inline]
    pub fn intra_mode_64x64_enable(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(60, 1) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_intra_mode_64x64_enable(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(60, 1, value as u64);
    }
    #[inline]
    pub fn priority_ipred_type_ip1(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(61, 3) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_priority_ipred_type_ip1(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(61, 3, value as u64);
    }
    #[inline]
    pub fn h265_intra_chroma_mode_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(64, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_h265_intra_chroma_mode_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(64, 5, value as u64);
    }
    #[inline]
    pub fn inter_penalty_factor_for_ip1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(69, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_inter_penalty_factor_for_ip1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(69, 6, value as u64);
    }
    #[inline]
    pub fn tusize_4x4_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(75, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_4x4_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(75, 1, value as u64);
    }
    #[inline]
    pub fn tusize_8x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(76, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_8x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(76, 1, value as u64);
    }
    #[inline]
    pub fn tusize_16x16_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(77, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_16x16_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(77, 1, value as u64);
    }
    #[inline]
    pub fn tusize_32x32_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(78, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_32x32_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(78, 1, value as u64);
    }
    #[inline]
    pub fn intra_tusize_decision(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(79, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_tusize_decision(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(79, 1, value as u64);
    }
    #[inline]
    pub fn pskip_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(80, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pskip_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(80, 1, value as u64);
    }
    #[inline]
    pub fn tu_search_basedon_pusize(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(81, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tu_search_basedon_pusize(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(81, 1, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(82, 14) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(82, 14, value as u64);
    }
    #[inline]
    pub fn tempmv_wt_distort_threshold(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(0, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tempmv_wt_distort_threshold(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(0, 16, value as u64);
    }
    #[inline]
    pub fn mv_cost_predictor_control(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(16, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mv_cost_predictor_control(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(16, 1, value as u64);
    }
    #[inline]
    pub fn mv_cost_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(17, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mv_cost_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(17, 1, value as u64);
    }
    #[inline]
    pub fn chroma_eval_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(18, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_eval_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(18, 1, value as u64);
    }
    #[inline]
    pub fn ip_search_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(19, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ip_search_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(19, 4, value as u64);
    }
    #[inline]
    pub fn multiply_bias_with_lambda(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(23, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_multiply_bias_with_lambda(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(23, 1, value as u64);
    }
    #[inline]
    pub fn force_ipcm(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(24, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_force_ipcm(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(24, 1, value as u64);
    }
    #[inline]
    pub fn early_termination_ip1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(25, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_early_termination_ip1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(25, 1, value as u64);
    }
    #[inline]
    pub fn tu_search_num_cu16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(26, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tu_search_num_cu16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(26, 3, value as u64);
    }
    #[inline]
    pub fn tu_search_num_cu32(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(29, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tu_search_num_cu32(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(29, 3, value as u64);
    }
    #[inline]
    pub fn num_1div8_lambda_ssd(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(0, 7) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_num_1div8_lambda_ssd(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(0, 7, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable_cu8(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(7, 1) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_skip_evaluate_enable_cu8(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(7, 1, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable_cu16(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(8, 1) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_skip_evaluate_enable_cu16(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(8, 1, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable_cu32(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(9, 1) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_skip_evaluate_enable_cu32(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(9, 1, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable_cu64(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(10, 1) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_skip_evaluate_enable_cu64(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(10, 1, value as u64);
    }
    #[inline]
    pub fn reserved2(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(11, 1) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_reserved2(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(11, 1, value as u64);
    }
    #[inline]
    pub fn intra_ssd_cnt_4x4(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(12, 4) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_intra_ssd_cnt_4x4(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(12, 4, value as u64);
    }
    #[inline]
    pub fn intra_ssd_cnt_8x8(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(16, 4) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_intra_ssd_cnt_8x8(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(16, 4, value as u64);
    }
    #[inline]
    pub fn intra_ssd_cnt_16x16(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(20, 4) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_intra_ssd_cnt_16x16(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(20, 4, value as u64);
    }
    #[inline]
    pub fn intra_ssd_cnt_32x32(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(24, 4) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_intra_ssd_cnt_32x32(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(24, 4, value as u64);
    }
    #[inline]
    pub fn rdo_level(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(28, 3) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_rdo_level(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(28, 3, value as u64);
    }
    #[inline]
    pub fn reserved3(&self) -> ::std::os::raw::c_short {
        self._bitfield_3.get(31, 1) as ::std::os::raw::c_short
    }
    #[inline]
    pub fn set_reserved3(&mut self, value: ::std::os::raw::c_short) {
        self._bitfield_3.set(31, 1, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvenc_h265_slice_control_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 28usize]>,
    pub reserved5: [::std::os::raw::c_uchar; 36usize],
}
impl Default for nvenc_h265_slice_control_s {
    fn default() -> Self {
        Self {
            _bindgen_align: [Default::default(); 0],
            _bitfield_1: Default::default(),
            reserved5: [Default::default(); 36usize],
        }
    }
}
impl nvenc_h265_slice_control_s {
    #[inline]
    pub fn num_ctus(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 19) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ctus(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 19, value as u64);
    }
    #[inline]
    pub fn qp_avr(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_qp_avr(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 8, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 5, value as u64);
    }
    #[inline]
    pub fn dependent_slice_segment_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_dependent_slice_segment_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 1, value as u64);
    }
    #[inline]
    pub fn slice_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(33, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slice_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(33, 2, value as u64);
    }
    #[inline]
    pub fn five_minus_max_num_merge_cand(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(35, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_five_minus_max_num_merge_cand(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(35, 3, value as u64);
    }
    #[inline]
    pub fn cabac_init_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(38, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cabac_init_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(38, 1, value as u64);
    }
    #[inline]
    pub fn deblocking_filter_override_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(39, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_deblocking_filter_override_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(39, 1, value as u64);
    }
    #[inline]
    pub fn slice_deblocking_filter_disabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(40, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slice_deblocking_filter_disabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(40, 1, value as u64);
    }
    #[inline]
    pub fn slice_loop_filter_across_slices_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(41, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slice_loop_filter_across_slices_enabled_flag(
        &mut self,
        value: ::std::os::raw::c_uint,
    ) {
        self._bitfield_1.set(41, 1, value as u64);
    }
    #[inline]
    pub fn qp_slice_min(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(42, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_qp_slice_min(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(42, 8, value as u64);
    }
    #[inline]
    pub fn qp_slice_max(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(50, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_qp_slice_max(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(50, 8, value as u64);
    }
    #[inline]
    pub fn force_intra(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(58, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_force_intra(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(58, 1, value as u64);
    }
    #[inline]
    pub fn slice_sao_luma_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(59, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slice_sao_luma_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(59, 1, value as u64);
    }
    #[inline]
    pub fn slice_sao_chroma_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(60, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slice_sao_chroma_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(60, 1, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(61, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(61, 3, value as u64);
    }
    #[inline]
    pub fn slice_beta_offset_div2(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(64, 4) << (64 - 4)) as i64 >> (64 - 4)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_slice_beta_offset_div2(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(64, 4, value as u64);
    }
    #[inline]
    pub fn slice_tc_offset_div2(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(68, 4) << (64 - 4)) as i64 >> (64 - 4)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_slice_tc_offset_div2(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(68, 4, value as u64);
    }
    #[inline]
    pub fn slice_cb_qp_offset(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(72, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_slice_cb_qp_offset(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(72, 5, value as u64);
    }
    #[inline]
    pub fn slice_cr_qp_offset(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(77, 5) << (64 - 5)) as i64 >> (64 - 5)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_slice_cr_qp_offset(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(77, 5, value as u64);
    }
    #[inline]
    pub fn slice_temporal_mvp_enabled_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(82, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_slice_temporal_mvp_enabled_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(82, 1, value as u64);
    }
    #[inline]
    pub fn collocated_from_l0_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(83, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_collocated_from_l0_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(83, 1, value as u64);
    }
    #[inline]
    pub fn collocated_ref_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(84, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_collocated_ref_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(84, 5, value as u64);
    }
    #[inline]
    pub fn reserved2(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(89, 7) << (64 - 7)) as i64 >> (64 - 7)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_reserved2(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(89, 7, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_active_override_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(96, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_active_override_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(96, 1, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l0_active_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(97, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l0_active_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(97, 5, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l1_active_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(102, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l1_active_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(102, 5, value as u64);
    }
    #[inline]
    pub fn me_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(107, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_me_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(107, 4, value as u64);
    }
    #[inline]
    pub fn md_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(111, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_md_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(111, 4, value as u64);
    }
    #[inline]
    pub fn q_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(115, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_q_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(115, 4, value as u64);
    }
    #[inline]
    pub fn wp_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(119, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_wp_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(119, 4, value as u64);
    }
    #[inline]
    pub fn reserved3(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(123, 5) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved3(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(123, 5, value as u64);
    }
    #[inline]
    pub fn limit_slice_top_boundary(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(128, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_limit_slice_top_boundary(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(128, 1, value as u64);
    }
    #[inline]
    pub fn limit_slice_bot_boundary(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(129, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_limit_slice_bot_boundary(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(129, 1, value as u64);
    }
    #[inline]
    pub fn limit_slice_left_boundary(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(130, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_limit_slice_left_boundary(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(130, 1, value as u64);
    }
    #[inline]
    pub fn limit_slice_right_boundary(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(131, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_limit_slice_right_boundary(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(131, 1, value as u64);
    }
    #[inline]
    pub fn ROI_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(132, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(132, 1, value as u64);
    }
    #[inline]
    pub fn ROI_me_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(133, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_me_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(133, 4, value as u64);
    }
    #[inline]
    pub fn ROI_md_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(137, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_md_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(137, 4, value as u64);
    }
    #[inline]
    pub fn ROI_q_control_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(141, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_q_control_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(141, 4, value as u64);
    }
    #[inline]
    pub fn ROI_qp_delta(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(145, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_qp_delta(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(145, 8, value as u64);
    }
    #[inline]
    pub fn reserved4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(153, 7) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(153, 7, value as u64);
    }
    #[inline]
    pub fn ROI_top_mbx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(160, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_top_mbx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(160, 16, value as u64);
    }
    #[inline]
    pub fn ROI_top_mby(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(176, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_top_mby(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(176, 16, value as u64);
    }
    #[inline]
    pub fn ROI_bot_mbx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(192, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_bot_mbx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(192, 16, value as u64);
    }
    #[inline]
    pub fn ROI_bot_mby(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(208, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ROI_bot_mby(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(208, 16, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_h265_quant_control_s {
    pub qpp_run_vector_4x4: ::std::os::raw::c_ushort,
    pub qpp_run_vector_8x8: [::std::os::raw::c_ushort; 3usize],
    pub qpp_luma8x8_cost: ::std::os::raw::c_uchar,
    pub qpp_luma16x16_cost: ::std::os::raw::c_uchar,
    pub qpp_chroma_cost: ::std::os::raw::c_uchar,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 1usize]>,
    pub reserved1: ::std::os::raw::c_uint,
}
impl nvenc_h265_quant_control_s {
    #[inline]
    pub fn qpp_mode(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_qpp_mode(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uchar {
        self._bitfield_1.get(2, 4) as ::std::os::raw::c_uchar
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uchar) {
        self._bitfield_1.set(2, 4, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_pic_stat_s {
    pub picture_index: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub total_bit_count: ::std::os::raw::c_uint,
    pub type1_bit_count: ::std::os::raw::c_uint,
    pub pic_type: ::std::os::raw::c_ushort,
    pub num_slices: ::std::os::raw::c_ushort,
    pub ave_activity: ::std::os::raw::c_ushort,
    pub avgQP: ::std::os::raw::c_ushort,
    pub cycle_count: ::std::os::raw::c_uint,
    pub hrdFullness: ::std::os::raw::c_int,
    pub bitstream_start_pos: ::std::os::raw::c_uint,
    pub last_valid_byte_offset: ::std::os::raw::c_uint,
    pub intra_mb_count: ::std::os::raw::c_ushort,
    pub inter_mb_count: ::std::os::raw::c_ushort,
    pub cumulative_intra_cost: ::std::os::raw::c_uint,
    pub cumulative_inter_cost: ::std::os::raw::c_uint,
    pub total_intra_cost: ::std::os::raw::c_uint,
    pub total_inter_cost: ::std::os::raw::c_uint,
    pub complexity: ::std::os::raw::c_uint,
    pub average_mvx: ::std::os::raw::c_short,
    pub average_mvy: ::std::os::raw::c_short,
    pub actual_min_qp_used: ::std::os::raw::c_ushort,
    pub actual_max_qp_used: ::std::os::raw::c_ushort,
    pub hw_perf_me_sps: ::std::os::raw::c_uint,
    pub hw_perf_me_fps: ::std::os::raw::c_uint,
    pub hw_perf_mpec_active_cnt: ::std::os::raw::c_uint,
    pub hw_perf_mpeb_active_cnt: ::std::os::raw::c_uint,
    pub total_dist_ssd: ::std::os::raw::c_ulonglong,
    pub total_dist_ssim: ::std::os::raw::c_uint,
    pub best_satd_l0: ::std::os::raw::c_uint,
    pub best_satd_l1: ::std::os::raw::c_uint,
    pub reserved: [::std::os::raw::c_uint; 5usize],
}
impl nvenc_pic_stat_s {
    #[inline]
    pub fn error_status(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_error_status(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn ucode_error_status(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 30) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ucode_error_status(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 30, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_pic_stat_s {
    pub picture_index: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub total_bit_count: ::std::os::raw::c_uint,
    pub type1_bit_count: ::std::os::raw::c_uint,
    pub num_tiles: ::std::os::raw::c_ushort,
    pub av1_start_QP: ::std::os::raw::c_uchar,
    pub loop_filter_level: ::std::os::raw::c_uchar,
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub cycle_count: ::std::os::raw::c_uint,
    pub hrdFullness: ::std::os::raw::c_uint,
    pub bitstream_start_pos: ::std::os::raw::c_uint,
    pub intra_cu64x64_count: ::std::os::raw::c_uint,
    pub inter_cu64x64_count: ::std::os::raw::c_uint,
    pub intra_cu32x32_count: ::std::os::raw::c_uint,
    pub inter_cu32x32_count: ::std::os::raw::c_uint,
    pub intra_cu16x16_count: ::std::os::raw::c_uint,
    pub inter_cu16x16_count: ::std::os::raw::c_uint,
    pub intra_cu8x8_count: ::std::os::raw::c_uint,
    pub inter_cu8x8_count: ::std::os::raw::c_uint,
    pub cumulative_intra_cost: ::std::os::raw::c_uint,
    pub cumulative_inter_cost: ::std::os::raw::c_uint,
    pub total_intra_cost: ::std::os::raw::c_uint,
    pub total_inter_cost: ::std::os::raw::c_uint,
    pub average_mvx: ::std::os::raw::c_short,
    pub average_mvy: ::std::os::raw::c_short,
    pub actual_min_qp_used: ::std::os::raw::c_short,
    pub actual_max_qp_used: ::std::os::raw::c_short,
    pub hw_perf_me_sps: ::std::os::raw::c_uint,
    pub hw_perf_me_fps: ::std::os::raw::c_uint,
    pub hw_perf_entropy_active_cnt: ::std::os::raw::c_uint,
    pub hw_perf_rdo_active_cnt: ::std::os::raw::c_uint,
    pub ave_activity: ::std::os::raw::c_ushort,
    pub reserved: [::std::os::raw::c_uchar; 18usize],
}
impl nvenc_av1_pic_stat_s {
    #[inline]
    pub fn error_status(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_error_status(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn ucode_error_status(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 30) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ucode_error_status(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 30, value as u64);
    }
    #[inline]
    pub fn pic_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(0, 14) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(0, 14, value as u64);
    }
    #[inline]
    pub fn cdef_damping_minus_3(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(14, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cdef_damping_minus_3(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(14, 2, value as u64);
    }
    #[inline]
    pub fn avgQP(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_2.get(16, 16) << (64 - 16)) as i64 >> (64 - 16)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_avgQP(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_2.set(16, 16, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_loop_filter_cfg_s {
    pub _bindgen_align: [u32; 0],
    pub filter_level: [::std::os::raw::c_uchar; 4usize],
    pub ref_delta: [::std::os::raw::c_char; 8usize],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_av1_loop_filter_cfg_s {
    #[inline]
    pub fn mode_delta_0(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(0, 8) << (64 - 8)) as i64 >> (64 - 8)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_mode_delta_0(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(0, 8, value as u64);
    }
    #[inline]
    pub fn mode_delta_1(&self) -> ::std::os::raw::c_int {
        ((self._bitfield_1.get(8, 8) << (64 - 8)) as i64 >> (64 - 8)) as ::std::os::raw::c_int
    }
    #[inline]
    pub fn set_mode_delta_1(&mut self, value: ::std::os::raw::c_int) {
        self._bitfield_1.set(8, 8, value as u64);
    }
    #[inline]
    pub fn update_ref_delta_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_update_ref_delta_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 8, value as u64);
    }
    #[inline]
    pub fn update_mode_delta0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_update_mode_delta0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 1, value as u64);
    }
    #[inline]
    pub fn update_mode_delta1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(25, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_update_mode_delta1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(25, 1, value as u64);
    }
    #[inline]
    pub fn delta_enabled(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_delta_enabled(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 1, value as u64);
    }
    #[inline]
    pub fn sharpness(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_sharpness(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 3, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 2, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_cdef_cfg_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 24usize]>,
}
impl nvenc_av1_cdef_cfg_s {
    #[inline]
    pub fn cdef_damping_minus_3(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cdef_damping_minus_3(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn y_strength_0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_strength_0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 6, value as u64);
    }
    #[inline]
    pub fn y_strength_1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_strength_1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 6, value as u64);
    }
    #[inline]
    pub fn y_strength_2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(14, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_strength_2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(14, 6, value as u64);
    }
    #[inline]
    pub fn y_strength_3(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_strength_3(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 6, value as u64);
    }
    #[inline]
    pub fn y_strength_4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_strength_4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 6, value as u64);
    }
    #[inline]
    pub fn y_strength_5(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_strength_5(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 6, value as u64);
    }
    #[inline]
    pub fn y_strength_6(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(38, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_strength_6(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(38, 6, value as u64);
    }
    #[inline]
    pub fn y_strength_7(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(44, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_strength_7(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(44, 6, value as u64);
    }
    #[inline]
    pub fn cdef_bits(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(50, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cdef_bits(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(50, 2, value as u64);
    }
    #[inline]
    pub fn uv_strength_0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(52, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_strength_0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(52, 6, value as u64);
    }
    #[inline]
    pub fn uv_strength_1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(58, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_strength_1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(58, 6, value as u64);
    }
    #[inline]
    pub fn uv_strength_2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(64, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_strength_2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(64, 6, value as u64);
    }
    #[inline]
    pub fn uv_strength_3(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(70, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_strength_3(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(70, 6, value as u64);
    }
    #[inline]
    pub fn uv_strength_4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(76, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_strength_4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(76, 6, value as u64);
    }
    #[inline]
    pub fn uv_strength_5(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(82, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_strength_5(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(82, 6, value as u64);
    }
    #[inline]
    pub fn uv_strength_6(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(88, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_strength_6(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(88, 6, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(94, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(94, 2, value as u64);
    }
    #[inline]
    pub fn uv_strength_7(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(96, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_strength_7(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(96, 6, value as u64);
    }
    #[inline]
    pub fn cdef_y_strength_num_minus_1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(102, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cdef_y_strength_num_minus_1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(102, 3, value as u64);
    }
    #[inline]
    pub fn cdef_uv_strength_num_minus_1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(105, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cdef_uv_strength_num_minus_1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(105, 3, value as u64);
    }
    #[inline]
    pub fn cdef_combined_idx_minus_1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(108, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cdef_combined_idx_minus_1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(108, 3, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(111, 17) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(111, 17, value as u64);
    }
    #[inline]
    pub fn cdef_combined_luma_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(128, 24) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cdef_combined_luma_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(128, 24, value as u64);
    }
    #[inline]
    pub fn reserved2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(152, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(152, 8, value as u64);
    }
    #[inline]
    pub fn cdef_combined_chroma_idx(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(160, 24) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cdef_combined_chroma_idx(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(160, 24, value as u64);
    }
    #[inline]
    pub fn reserved3(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(184, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved3(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(184, 8, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_lr_filter_cfg_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_av1_lr_filter_cfg_s {
    #[inline]
    pub fn y_lr_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_lr_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn uv_lr_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_lr_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 2, value as u64);
    }
    #[inline]
    pub fn y_lr_unit_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(4, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_y_lr_unit_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(4, 2, value as u64);
    }
    #[inline]
    pub fn uv_lr_unit_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(6, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_uv_lr_unit_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(6, 2, value as u64);
    }
    #[inline]
    pub fn lr_codebook_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 16) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_lr_codebook_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 16, value as u64);
    }
    #[inline]
    pub fn lr_pe_on_edge_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_lr_pe_on_edge_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 1, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(25, 7) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(25, 7, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_quant_cfg_s {
    pub base_q_idx: ::std::os::raw::c_uchar,
    pub delta_y_dc: ::std::os::raw::c_char,
    pub delta_uv_dc: ::std::os::raw::c_char,
    pub delta_uv_ac: ::std::os::raw::c_char,
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_surface_cfg_s {
    pub frame_width_minus1: ::std::os::raw::c_ushort,
    pub frame_height_minus1: ::std::os::raw::c_ushort,
    pub sfc_pitch: ::std::os::raw::c_ushort,
    pub sfc_pitch_chroma: ::std::os::raw::c_ushort,
    pub sfc_trans_mode: ::std::os::raw::c_ushort,
    pub reserved2: ::std::os::raw::c_ushort,
    pub luma_top_frm_offset: ::std::os::raw::c_uint,
    pub luma_bot_offset: ::std::os::raw::c_uint,
    pub chroma_top_frm_offset: ::std::os::raw::c_uint,
    pub chroma_bot_offset: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_av1_surface_cfg_s {
    #[inline]
    pub fn block_height(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 7) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_block_height(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 7, value as u64);
    }
    #[inline]
    pub fn tiled_16x16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tiled_16x16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 1, value as u64);
    }
    #[inline]
    pub fn memory_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_memory_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 2, value as u64);
    }
    #[inline]
    pub fn nv21_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(10, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_nv21_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(10, 1, value as u64);
    }
    #[inline]
    pub fn input_bl_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(11, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_input_bl_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(11, 2, value as u64);
    }
    #[inline]
    pub fn src_in_lsb(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_src_in_lsb(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 1, value as u64);
    }
    #[inline]
    pub fn reserved(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(14, 18) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(14, 18, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_seq_hdr_data_s {
    pub _bindgen_align: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_av1_seq_hdr_data_s {
    #[inline]
    pub fn min_cu_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_min_cu_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn max_cu_size(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_max_cu_size(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 2, value as u64);
    }
    #[inline]
    pub fn chroma_format_idc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(4, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_format_idc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(4, 2, value as u64);
    }
    #[inline]
    pub fn monochrome_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(6, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_monochrome_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(6, 1, value as u64);
    }
    #[inline]
    pub fn bit_depth_minus_8(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bit_depth_minus_8(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 4, value as u64);
    }
    #[inline]
    pub fn input_bit_depth_minus8(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(11, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_input_bit_depth_minus8(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(11, 4, value as u64);
    }
    #[inline]
    pub fn enable_filter_intra(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(15, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_filter_intra(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(15, 1, value as u64);
    }
    #[inline]
    pub fn enable_intra_edge_filter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_intra_edge_filter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 1, value as u64);
    }
    #[inline]
    pub fn enable_interintra_compound(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(17, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_interintra_compound(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(17, 1, value as u64);
    }
    #[inline]
    pub fn enable_masked_compound(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_masked_compound(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 1, value as u64);
    }
    #[inline]
    pub fn enable_dual_filter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_dual_filter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 1, value as u64);
    }
    #[inline]
    pub fn force_screen_content_tools(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_force_screen_content_tools(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 2, value as u64);
    }
    #[inline]
    pub fn force_integer_mv(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(22, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_force_integer_mv(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(22, 2, value as u64);
    }
    #[inline]
    pub fn enable_jnt_comp(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_jnt_comp(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 1, value as u64);
    }
    #[inline]
    pub fn enable_ref_frame_mvs(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(25, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_ref_frame_mvs(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(25, 1, value as u64);
    }
    #[inline]
    pub fn enable_warped_motion(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_warped_motion(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 1, value as u64);
    }
    #[inline]
    pub fn enable_superres(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_superres(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 1, value as u64);
    }
    #[inline]
    pub fn enable_cdef(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_cdef(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 1, value as u64);
    }
    #[inline]
    pub fn enable_restoration(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_restoration(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(29, 1, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 2, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_frame_hdr_data_s {
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 8usize]>,
    pub ref_frame_sign_bias: [::std::os::raw::c_uchar; 8usize],
    pub lf_cfg: nvenc_av1_loop_filter_cfg_s,
    pub cdef_cfg: nvenc_av1_cdef_cfg_s,
    pub quant_cfg: nvenc_av1_quant_cfg_s,
    pub lrf_cfg: nvenc_av1_lr_filter_cfg_s,
}
impl nvenc_av1_frame_hdr_data_s {
    #[inline]
    pub fn frame_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_frame_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 2, value as u64);
    }
    #[inline]
    pub fn disable_cdf_update(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_disable_cdf_update(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 1, value as u64);
    }
    #[inline]
    pub fn allow_screen_content_tools(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_allow_screen_content_tools(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 1, value as u64);
    }
    #[inline]
    pub fn force_integer_mv(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(4, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_force_integer_mv(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(4, 1, value as u64);
    }
    #[inline]
    pub fn allow_high_precision_mv(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(5, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_allow_high_precision_mv(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(5, 1, value as u64);
    }
    #[inline]
    pub fn switchable_motion_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(6, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_switchable_motion_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(6, 1, value as u64);
    }
    #[inline]
    pub fn is_filter_switchable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_is_filter_switchable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 1, value as u64);
    }
    #[inline]
    pub fn use_ref_frame_mvs(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_use_ref_frame_mvs(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 1, value as u64);
    }
    #[inline]
    pub fn frame_end_update_cdf(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(9, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_frame_end_update_cdf(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(9, 1, value as u64);
    }
    #[inline]
    pub fn interp_filter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(10, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_interp_filter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(10, 3, value as u64);
    }
    #[inline]
    pub fn delta_q_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_delta_q_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 1, value as u64);
    }
    #[inline]
    pub fn delta_q_res(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(14, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_delta_q_res(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(14, 2, value as u64);
    }
    #[inline]
    pub fn allow_intrabc(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_allow_intrabc(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 1, value as u64);
    }
    #[inline]
    pub fn delta_lf_present_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(17, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_delta_lf_present_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(17, 1, value as u64);
    }
    #[inline]
    pub fn delta_lf_res(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_delta_lf_res(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 2, value as u64);
    }
    #[inline]
    pub fn delta_lf_multi(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_delta_lf_multi(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 1, value as u64);
    }
    #[inline]
    pub fn using_qmatrix(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(21, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_using_qmatrix(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(21, 1, value as u64);
    }
    #[inline]
    pub fn frame_tx_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(22, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_frame_tx_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(22, 2, value as u64);
    }
    #[inline]
    pub fn frame_reference_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(24, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_frame_reference_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(24, 2, value as u64);
    }
    #[inline]
    pub fn skip_mode_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_skip_mode_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 1, value as u64);
    }
    #[inline]
    pub fn allow_warped_motion(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_allow_warped_motion(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 1, value as u64);
    }
    #[inline]
    pub fn reduced_tx_set_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reduced_tx_set_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 1, value as u64);
    }
    #[inline]
    pub fn coded_lossless(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_coded_lossless(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(29, 1, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 2, value as u64);
    }
    #[inline]
    pub fn ssim_rdo_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ssim_rdo_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 1, value as u64);
    }
    #[inline]
    pub fn ssim_rdo_alt_frame_scale(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(33, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ssim_rdo_alt_frame_scale(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(33, 8, value as u64);
    }
    #[inline]
    pub fn bicompound_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(41, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_bicompound_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(41, 1, value as u64);
    }
    #[inline]
    pub fn unicompound_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(42, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_unicompound_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(42, 1, value as u64);
    }
    #[inline]
    pub fn mc_hfilter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(43, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mc_hfilter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(43, 4, value as u64);
    }
    #[inline]
    pub fn mc_vfilter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(47, 4) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mc_vfilter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(47, 4, value as u64);
    }
    #[inline]
    pub fn segmentation_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(51, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_segmentation_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(51, 1, value as u64);
    }
    #[inline]
    pub fn segmentation_update_map(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(52, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_segmentation_update_map(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(52, 1, value as u64);
    }
    #[inline]
    pub fn segment_temporal_update(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(53, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_segment_temporal_update(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(53, 1, value as u64);
    }
    #[inline]
    pub fn segIdPreSkip(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(54, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_segIdPreSkip(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(54, 1, value as u64);
    }
    #[inline]
    pub fn LastActiveSegId(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(55, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_LastActiveSegId(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(55, 3, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(58, 6) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(58, 6, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_rc_s {
    pub hrd_type: ::std::os::raw::c_uchar,
    pub QP: [::std::os::raw::c_uchar; 3usize],
    pub minQP: [::std::os::raw::c_uchar; 3usize],
    pub maxQP: [::std::os::raw::c_uchar; 3usize],
    pub maxQPD: ::std::os::raw::c_char,
    pub baseQPD: ::std::os::raw::c_char,
    pub rhopbi: [::std::os::raw::c_uint; 3usize],
    pub framerate: ::std::os::raw::c_uint,
    pub buffersize: ::std::os::raw::c_uint,
    pub nal_cpb_size: ::std::os::raw::c_uint,
    pub nal_bitrate: ::std::os::raw::c_uint,
    pub vcl_cpb_size: ::std::os::raw::c_uint,
    pub vcl_bitrate: ::std::os::raw::c_uint,
    pub gop_length: ::std::os::raw::c_uint,
    pub Np: ::std::os::raw::c_uint,
    pub Bmin: ::std::os::raw::c_uint,
    pub Ravg: ::std::os::raw::c_uint,
    pub R: ::std::os::raw::c_uint,
    pub ab_alpha: ::std::os::raw::c_uchar,
    pub ab_beta: ::std::os::raw::c_uchar,
    pub prev_act: ::std::os::raw::c_ushort,
    pub reserved0: ::std::os::raw::c_uchar,
    pub single_frame_vbv: ::std::os::raw::c_uchar,
    pub two_pass_rc: ::std::os::raw::c_uchar,
    pub rc_class: ::std::os::raw::c_uchar,
    pub first_pass_source_half_scaled: ::std::os::raw::c_uchar,
    pub i_size_ratio_x: ::std::os::raw::c_uchar,
    pub i_size_ratio_y: ::std::os::raw::c_uchar,
    pub aqMode: ::std::os::raw::c_uchar,
    pub dump_aq_stats: ::std::os::raw::c_uchar,
    pub ext_scene_change_flag: ::std::os::raw::c_uchar,
    pub ext_picture_rc_hints: ::std::os::raw::c_uchar,
    pub session_max_qp: ::std::os::raw::c_uchar,
    pub ext_intra_satd: ::std::os::raw::c_uint,
    pub reserved2: [::std::os::raw::c_uchar; 8usize],
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_pic_control_s {
    pub l0: [::std::os::raw::c_char; 4usize],
    pub l1: [::std::os::raw::c_char; 3usize],
    pub temp_dist_l0: [::std::os::raw::c_char; 4usize],
    pub temp_dist_l1: [::std::os::raw::c_char; 3usize],
    pub poc_entry_l0: [::std::os::raw::c_char; 4usize],
    pub poc_entry_l1: [::std::os::raw::c_char; 3usize],
    pub reserved0: [::std::os::raw::c_char; 3usize],
    pub max_byte_count_before_resid_zero: ::std::os::raw::c_uint,
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 8usize]>,
    pub prob_tables_offset: ::std::os::raw::c_uint,
    pub tile_control_offset: ::std::os::raw::c_uint,
    pub hist_buf_size: ::std::os::raw::c_uint,
    pub bitstream_buf_size: ::std::os::raw::c_uint,
    pub bitstream_start_pos: ::std::os::raw::c_uint,
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 4usize]>,
    pub mpec_threshold: ::std::os::raw::c_uint,
    pub tile_stat_offset: ::std::os::raw::c_uint,
    pub mpec_stat_offset: ::std::os::raw::c_uint,
    pub aq_stat_offset: ::std::os::raw::c_uint,
    pub rc_pass2_offset: ::std::os::raw::c_uint,
    pub act_stat_offset: ::std::os::raw::c_uint,
    pub external_hint_number: ::std::os::raw::c_uint,
    pub stripID: ::std::os::raw::c_uchar,
    pub strips_in_frame: ::std::os::raw::c_uchar,
    pub strip_width_in_sbs: ::std::os::raw::c_ushort,
    pub strip_height_in_sbs: ::std::os::raw::c_ushort,
    pub strip_width_offset_in_sbs: ::std::os::raw::c_ushort,
    pub strip_height_offset_in_sbs: ::std::os::raw::c_ushort,
    pub reserved5: ::std::os::raw::c_ushort,
    pub stats_fifo_offset: ::std::os::raw::c_uint,
    pub reserved6: [::std::os::raw::c_uchar; 8usize],
}
impl nvenc_av1_pic_control_s {
    #[inline]
    pub fn mode_decision_only_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mode_decision_only_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 1, value as u64);
    }
    #[inline]
    pub fn cu_early_terminatiom_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(1, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu_early_terminatiom_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(1, 1, value as u64);
    }
    #[inline]
    pub fn pic_output_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_output_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 1, value as u64);
    }
    #[inline]
    pub fn input_sub_sampling(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_input_sub_sampling(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 1, value as u64);
    }
    #[inline]
    pub fn output_sub_sampling(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(4, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_output_sub_sampling(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(4, 1, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l0_active(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(5, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l0_active(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(5, 3, value as u64);
    }
    #[inline]
    pub fn num_ref_idx_l1_active(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_ref_idx_l1_active(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 3, value as u64);
    }
    #[inline]
    pub fn tile_idx_start(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(11, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tile_idx_start(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(11, 8, value as u64);
    }
    #[inline]
    pub fn num_strip_tiles_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_strip_tiles_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 8, value as u64);
    }
    #[inline]
    pub fn force_lf_on_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(27, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_force_lf_on_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(27, 1, value as u64);
    }
    #[inline]
    pub fn force_lf_off_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(28, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_force_lf_off_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(28, 1, value as u64);
    }
    #[inline]
    pub fn vp9_skip_chroma(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(29, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_vp9_skip_chroma(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(29, 1, value as u64);
    }
    #[inline]
    pub fn reserved1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(30, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(30, 2, value as u64);
    }
    #[inline]
    pub fn max_tu_depth_inter(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_max_tu_depth_inter(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 2, value as u64);
    }
    #[inline]
    pub fn max_tu_depth_intra(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(34, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_max_tu_depth_intra(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(34, 2, value as u64);
    }
    #[inline]
    pub fn num_tiles_minus1(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(36, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_tiles_minus1(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(36, 8, value as u64);
    }
    #[inline]
    pub fn enable_source_image_padding(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(44, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_enable_source_image_padding(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(44, 1, value as u64);
    }
    #[inline]
    pub fn luma_padding_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(45, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_luma_padding_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(45, 2, value as u64);
    }
    #[inline]
    pub fn chroma_padding_mode(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(47, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_padding_mode(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(47, 2, value as u64);
    }
    #[inline]
    pub fn reserved2(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(49, 15) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved2(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(49, 15, value as u64);
    }
    #[inline]
    pub fn pic_type(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(0, 2) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pic_type(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(0, 2, value as u64);
    }
    #[inline]
    pub fn ref_pic_flag(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ref_pic_flag(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(2, 1, value as u64);
    }
    #[inline]
    pub fn codec(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(3, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_codec(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(3, 3, value as u64);
    }
    #[inline]
    pub fn qpfifo(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(6, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_qpfifo(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(6, 1, value as u64);
    }
    #[inline]
    pub fn num_tile_cols(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(7, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_tile_cols(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(7, 8, value as u64);
    }
    #[inline]
    pub fn num_tile_rows(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(15, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_tile_rows(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(15, 8, value as u64);
    }
    #[inline]
    pub fn ctx_update_tile_id(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(23, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_ctx_update_tile_id(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(23, 8, value as u64);
    }
    #[inline]
    pub fn stats_fifo_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(31, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_stats_fifo_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(31, 1, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct nvenc_av1_md_control_s {
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 12usize]>,
    pub intra_luma_dir_delta_64x64_enable_0: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_64x64_enable_1: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_64x32_enable_0: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_64x32_enable_1: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_32x32_enable_0: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_32x32_enable_1: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_32x16_enable_0: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_32x16_enable_1: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_16x16_enable_0: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_16x16_enable_1: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_16x8_enable_0: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_16x8_enable_1: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_8x8_enable_0: ::std::os::raw::c_uint,
    pub intra_luma_dir_delta_8x8_enable_1: ::std::os::raw::c_uint,
    pub intra_luma_base_mode_4x4_enable: ::std::os::raw::c_ushort,
    pub intra_luma_base_mode_8x8_enable: ::std::os::raw::c_ushort,
    pub intra_luma_base_mode_16x16_enable: ::std::os::raw::c_ushort,
    pub intra_luma_base_mode_32x32_enable: ::std::os::raw::c_ushort,
    pub intra_luma_base_mode_64x64_enable: ::std::os::raw::c_ushort,
    pub bias_intra_over_inter: ::std::os::raw::c_short,
    pub mv_cost_bias: ::std::os::raw::c_ushort,
    pub bias_tu_4x4: ::std::os::raw::c_short,
    pub bias_tu_8x8: ::std::os::raw::c_short,
    pub bias_tu_16x16: ::std::os::raw::c_short,
    pub bias_tu_32x32: ::std::os::raw::c_short,
    pub bias_tu_64x64: ::std::os::raw::c_short,
    pub bias_tu_8x4: ::std::os::raw::c_short,
    pub bias_tu_4x8: ::std::os::raw::c_short,
    pub bias_tu_16x8: ::std::os::raw::c_short,
    pub bias_tu_8x16: ::std::os::raw::c_short,
    pub bias_tu_32x16: ::std::os::raw::c_short,
    pub bias_tu_16x32: ::std::os::raw::c_short,
    pub bias_skip: ::std::os::raw::c_short,
    pub bias_inter_64x64: ::std::os::raw::c_short,
    pub bias_inter_64x32: ::std::os::raw::c_short,
    pub bias_inter_32x64: ::std::os::raw::c_short,
    pub bias_inter_32x32: ::std::os::raw::c_short,
    pub bias_inter_32x16: ::std::os::raw::c_short,
    pub bias_inter_16x32: ::std::os::raw::c_short,
    pub bias_inter_16x16: ::std::os::raw::c_short,
    pub bias_inter_16x8: ::std::os::raw::c_short,
    pub bias_inter_8x16: ::std::os::raw::c_short,
    pub bias_inter_8x8: ::std::os::raw::c_short,
    pub bias_intra_64x64: ::std::os::raw::c_short,
    pub bias_intra_64x32: ::std::os::raw::c_short,
    pub bias_intra_32x64: ::std::os::raw::c_short,
    pub bias_intra_32x32: ::std::os::raw::c_short,
    pub bias_intra_32x16: ::std::os::raw::c_short,
    pub bias_intra_16x32: ::std::os::raw::c_short,
    pub bias_intra_16x16: ::std::os::raw::c_short,
    pub bias_intra_16x8: ::std::os::raw::c_short,
    pub bias_intra_8x16: ::std::os::raw::c_short,
    pub bias_intra_8x8: ::std::os::raw::c_short,
    pub bias_intra_8x4: ::std::os::raw::c_short,
    pub bias_intra_4x8: ::std::os::raw::c_short,
    pub bias_intra_4x4: ::std::os::raw::c_short,
    pub tx_type_search_mask: [::std::os::raw::c_ushort; 2usize],
    pub _bitfield_2: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl nvenc_av1_md_control_s {
    #[inline]
    pub fn cu8x8_part_2Nx2N_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(0, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu8x8_part_2Nx2N_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(0, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_2Nx2N_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(1, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu16x16_part_2Nx2N_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(1, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_2NxN_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(2, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu16x16_part_2NxN_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(2, 1, value as u64);
    }
    #[inline]
    pub fn cu16x16_part_Nx2N_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(3, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu16x16_part_Nx2N_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(3, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_2Nx2N_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(4, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu32x32_part_2Nx2N_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(4, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_2NxN_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(5, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu32x32_part_2NxN_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(5, 1, value as u64);
    }
    #[inline]
    pub fn cu32x32_part_Nx2N_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(6, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu32x32_part_Nx2N_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(6, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_2Nx2N_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(7, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu64x64_part_2Nx2N_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(7, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_2NxN_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(8, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu64x64_part_2NxN_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(8, 1, value as u64);
    }
    #[inline]
    pub fn cu64x64_part_Nx2N_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(9, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_cu64x64_part_Nx2N_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(9, 1, value as u64);
    }
    #[inline]
    pub fn tusize_4x4_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(10, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_4x4_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(10, 1, value as u64);
    }
    #[inline]
    pub fn tusize_8x8_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(11, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_8x8_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(11, 1, value as u64);
    }
    #[inline]
    pub fn tusize_16x16_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(12, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_16x16_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(12, 1, value as u64);
    }
    #[inline]
    pub fn tusize_32x32_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(13, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_32x32_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(13, 1, value as u64);
    }
    #[inline]
    pub fn tusize_64x64_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(14, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tusize_64x64_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(14, 1, value as u64);
    }
    #[inline]
    pub fn intra_tusize_decision(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(15, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_tusize_decision(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(15, 1, value as u64);
    }
    #[inline]
    pub fn pskip_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(16, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_pskip_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(16, 1, value as u64);
    }
    #[inline]
    pub fn tu_search_basedon_pusize(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(17, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_tu_search_basedon_pusize(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(17, 1, value as u64);
    }
    #[inline]
    pub fn multiply_bias_with_lambda(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(18, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_multiply_bias_with_lambda(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(18, 1, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable_cu8(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(19, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_skip_evaluate_enable_cu8(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(19, 1, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable_cu16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(20, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_skip_evaluate_enable_cu16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(20, 1, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable_cu32(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(21, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_skip_evaluate_enable_cu32(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(21, 1, value as u64);
    }
    #[inline]
    pub fn skip_evaluate_enable_cu64(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(22, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_skip_evaluate_enable_cu64(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(22, 1, value as u64);
    }
    #[inline]
    pub fn inter_rdo_cnt_8x8(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(23, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_inter_rdo_cnt_8x8(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(23, 3, value as u64);
    }
    #[inline]
    pub fn inter_rdo_cnt_16x16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(26, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_inter_rdo_cnt_16x16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(26, 3, value as u64);
    }
    #[inline]
    pub fn inter_rdo_cnt_32x32(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(29, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_inter_rdo_cnt_32x32(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(29, 3, value as u64);
    }
    #[inline]
    pub fn inter_rdo_cnt_64x64(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(32, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_inter_rdo_cnt_64x64(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(32, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_4x4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(35, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_4x4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(35, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_8x8(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(38, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_8x8(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(38, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_8x4(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(41, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_8x4(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(41, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_4x8(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(44, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_4x8(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(44, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_16x16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(47, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_16x16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(47, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_16x8(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(50, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_16x8(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(50, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_8x16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(53, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_8x16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(53, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_32x32(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(56, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_32x32(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(56, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_32x16(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(59, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_32x16(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(59, 3, value as u64);
    }
    #[inline]
    pub fn luma_all_zero_check(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(62, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_luma_all_zero_check(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(62, 1, value as u64);
    }
    #[inline]
    pub fn chroma_all_zero_check(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(63, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_chroma_all_zero_check(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(63, 1, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_16x32(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(64, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_16x32(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(64, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_64x64(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(67, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_64x64(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(67, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_64x32(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(70, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_64x32(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(70, 3, value as u64);
    }
    #[inline]
    pub fn intra_rdo_cnt_32x64(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(73, 3) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_intra_rdo_cnt_32x64(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(73, 3, value as u64);
    }
    #[inline]
    pub fn num_1div8_lambda_ssd(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(76, 7) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_num_1div8_lambda_ssd(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(76, 7, value as u64);
    }
    #[inline]
    pub fn mv_cost_enable(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(83, 1) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_mv_cost_enable(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(83, 1, value as u64);
    }
    #[inline]
    pub fn reserved0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_1.get(84, 12) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_reserved0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_1.set(84, 12, value as u64);
    }
    #[inline]
    pub fn zbin_factor_lt_148(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(0, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_zbin_factor_lt_148(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(0, 8, value as u64);
    }
    #[inline]
    pub fn zbin_factor_gt_148(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(8, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_zbin_factor_gt_148(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(8, 8, value as u64);
    }
    #[inline]
    pub fn round_factor_eq_0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(16, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_round_factor_eq_0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(16, 8, value as u64);
    }
    #[inline]
    pub fn round_factor_ne_0(&self) -> ::std::os::raw::c_uint {
        self._bitfield_2.get(24, 8) as ::std::os::raw::c_uint
    }
    #[inline]
    pub fn set_round_factor_ne_0(&mut self, value: ::std::os::raw::c_uint) {
        self._bitfield_2.set(24, 8, value as u64);
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvenc_av1_drv_pic_setup_s {
    pub magic: ::std::os::raw::c_uint,
    pub refpic_cfg: nvenc_av1_surface_cfg_s,
    pub input_cfg: nvenc_av1_surface_cfg_s,
    pub outputpic_cfg: nvenc_av1_surface_cfg_s,
    pub seqhdr_data: nvenc_av1_seq_hdr_data_s,
    pub framehdr_data: nvenc_av1_frame_hdr_data_s,
    pub rate_control: nvenc_av1_rc_s,
    pub pic_control: nvenc_av1_pic_control_s,
    pub half_scaled_outputpic_cfg: nvenc_av1_surface_cfg_s,
    pub md_control: nvenc_av1_md_control_s,
    pub me_control: nvenc_me_control_s,
    pub gpTimer_timeout_val: ::std::os::raw::c_uint,
    pub reserved0: [::std::os::raw::c_uchar; 4usize],
}
impl Default for nvenc_av1_drv_pic_setup_s {
    fn default() -> Self {
        Self {
            magic: Default::default(),
            refpic_cfg: Default::default(),
            input_cfg: Default::default(),
            outputpic_cfg: Default::default(),
            seqhdr_data: Default::default(),
            framehdr_data: Default::default(),
            rate_control: Default::default(),
            pic_control: Default::default(),
            half_scaled_outputpic_cfg: Default::default(),
            md_control: Default::default(),
            me_control: Default::default(),
            gpTimer_timeout_val: Default::default(),
            reserved0: [Default::default(); 4usize],
        }
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvenc_persistent_state_s {
    pub nal_cpb_fullness: ::std::os::raw::c_int,
    pub vcl_cpb_fullness: ::std::os::raw::c_int,
    pub Wpbi: [::std::os::raw::c_int; 3usize],
    pub Whist: [[::std::os::raw::c_int; 3usize]; 3usize],
    pub np: ::std::os::raw::c_int,
    pub nb: ::std::os::raw::c_int,
    pub t: ::std::os::raw::c_int,
    pub Rmean: ::std::os::raw::c_int,
    pub Bavg: ::std::os::raw::c_int,
    pub average_mvx: ::std::os::raw::c_short,
    pub average_mvy: ::std::os::raw::c_short,
    pub prevQS: ::std::os::raw::c_int,
    pub prevQP: ::std::os::raw::c_short,
    pub fwdQP: ::std::os::raw::c_short,
    pub refQPcol: [::std::os::raw::c_uchar; 48usize],
    pub Rcol: [::std::os::raw::c_uchar; 48usize],
    pub ref1QP: [::std::os::raw::c_uchar; 48usize],
    pub ext_intra_satd_Hist: [::std::os::raw::c_uint; 2usize],
    pub ext_scd_Hist: ::std::os::raw::c_uchar,
    pub scene_change: ::std::os::raw::c_uchar,
    pub qpp_modifier: ::std::os::raw::c_uchar,
    pub Ksatd: ::std::os::raw::c_uchar,
    pub Werr: ::std::os::raw::c_schar,
    pub WerrI: ::std::os::raw::c_schar,
    pub avgBf: ::std::os::raw::c_uchar,
    pub reserved: ::std::os::raw::c_uchar,
    pub printf_offset: ::std::os::raw::c_uint,
    pub reserved2: [::std::os::raw::c_uchar; 4usize],
}
impl Default for nvenc_persistent_state_s {
    fn default() -> Self {
        Self {
            nal_cpb_fullness: Default::default(),
            vcl_cpb_fullness: Default::default(),
            Wpbi: [Default::default(); 3usize],
            Whist: [[Default::default(); 3usize]; 3usize],
            np: Default::default(),
            nb: Default::default(),
            t: Default::default(),
            Rmean: Default::default(),
            Bavg: Default::default(),
            average_mvx: Default::default(),
            average_mvy: Default::default(),
            prevQS: Default::default(),
            prevQP: Default::default(),
            fwdQP: Default::default(),
            refQPcol: [Default::default(); 48usize],
            Rcol: [Default::default(); 48usize],
            ref1QP: [Default::default(); 48usize],
            ext_intra_satd_Hist: [Default::default(); 2usize],
            ext_scd_Hist: Default::default(),
            scene_change: Default::default(),
            qpp_modifier: Default::default(),
            Ksatd: Default::default(),
            Werr: Default::default(),
            WerrI: Default::default(),
            avgBf: Default::default(),
            reserved: Default::default(),
            printf_offset: Default::default(),
            reserved2: [Default::default(); 4usize],
        }
    }
}

// clc9b7.h method IDs used by enc.c (MIT).
pub const NVC9B7_SET_APPLICATION_ID: u32 = 0x00000200;
pub const NVC9B7_SET_APPLICATION_ID_ID_NVENC_H264: u32 = 0x00000001;
pub const NVC9B7_SET_APPLICATION_ID_ID_NVENC_H265: u32 = 0x00000003;
pub const NVC9B7_SET_APPLICATION_ID_ID_NVENC_AV1: u32 = 0x00000007;
pub const NVC9B7_SEMAPHORE_A: u32 = 0x00000240;
pub const NVC9B7_EXECUTE: u32 = 0x00000300;
pub const NVC9B7_SEMAPHORE_D: u32 = 0x00000304;
pub const NVC9B7_SET_IN_REF_PIC0_LUMA: u32 = 0x00000400;
pub const NVC9B7_SET_IN_REF_PIC0_CHROMA: u32 = 0x00000440;
pub const NVC9B7_SET_IN_AV1_PROBABILITY_DATA: u32 = 0x00000480;
pub const NVC9B7_SET_OUT_AV1_PROBABILITY_DATA: u32 = 0x00000484;
pub const NVC9B7_SET_IOENTROPY_ABOVE_CTX_DATA: u32 = 0x00000488;
pub const NVC9B7_SET_IOENTROPY_ABOVE_MV_DATA: u32 = 0x0000048C;
pub const NVC9B7_SET_IODEBLOCK_TOPROW: u32 = 0x00000490;
pub const NVC9B7_SET_IODEBLOCK_LEFTCOLUMN: u32 = 0x00000494;
pub const NVC9B7_SET_IOCDEF_LEFTCOLUMN: u32 = 0x00000498;
pub const NVC9B7_SET_IOLR_LEFTCOLUMN: u32 = 0x0000049C;
pub const NVC9B7_SET_CONTROL_PARAMS: u32 = 0x00000700;
pub const NVC9B7_SET_CONTROL_PARAMS_CODEC_TYPE_H264: u32 = 0x00000003;
pub const NVC9B7_SET_PICTURE_INDEX: u32 = 0x00000704;
pub const NVC9B7_SET_IN_RCDATA: u32 = 0x0000070C;
pub const NVC9B7_SET_IN_DRV_PIC_SETUP: u32 = 0x00000710;
pub const NVC9B7_SET_OUT_ENC_STATUS: u32 = 0x00000718;
pub const NVC9B7_SET_OUT_BITSTREAM: u32 = 0x0000071C;
pub const NVC9B7_SET_IOHISTORY: u32 = 0x00000720;
pub const NVC9B7_SET_IO_RC_PROCESS: u32 = 0x00000724;
pub const NVC9B7_SET_IN_COLOC_DATA: u32 = 0x00000728;
pub const NVC9B7_SET_OUT_COLOC_DATA: u32 = 0x0000072C;
pub const NVC9B7_SET_OUT_REF_PIC_LUMA: u32 = 0x00000730;
pub const NVC9B7_SET_IN_CUR_PIC: u32 = 0x00000734;
pub const NVC9B7_SET_IN_MEPRED_DATA: u32 = 0x00000738;
pub const NVC9B7_SET_OUT_MEPRED_DATA: u32 = 0x0000073C;
pub const NVC9B7_SET_IN_CUR_PIC_CHROMA_U: u32 = 0x00000740;
pub const NVC9B7_SET_IN_CUR_PIC_CHROMA_V: u32 = 0x00000744;
pub const NVC9B7_SET_OUT_REF_PIC_CHROMA: u32 = 0x0000074C;
pub type nvenc_me_control_s = nvenc_h264_me_control_s;
