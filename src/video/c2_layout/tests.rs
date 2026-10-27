use super::*;
fn native() -> (NvC2GrallocSnapshot, NvC2DecAllocation) {
    (
        NvC2GrallocSnapshot {
            abi_version: 1,
            native_version: 12,
            num_fds: 2,
            num_ints: 33,
            payload_words: 35,
            magic: 0xabcddcba,
            num_planes: 2,
            fds: [1000, 1000, -1, -1, -1],
            strides: [640, 640, 0, 0],
            offsets: [0, 640 * 368, 0, 0],
            sizes: [640 * 360, 640 * 180, 0, 0],
            width: 640,
            height: 360,
            drm_format: 0x3231564e,
            total_size: 640 * 548,
            droid_format: 0x23,
            ..Default::default()
        },
        NvC2DecAllocation {
            block_width: 640,
            block_height: 360,
            hal_format: 0x23,
            policy: 0,
        },
    )
}
#[test]
fn padded_vulkan_uv_offset_and_odd_crop() {
    let (s, r) = native();
    let t = target(&s, &r).unwrap();
    assert_eq!(t.offset[1], 640 * 368);
    let mut r = NvC2DecAllocation::default();
    assert_eq!(unsafe { nv_c2_dec_allocation(639, 359, 1, 0, &mut r) }, OK);
    assert_eq!((r.block_width, r.block_height), (640, 360));
    assert_eq!(unsafe { nv_c2_dec_allocation(639, 359, 1, 1, &mut r) }, OK);
    assert_eq!(
        (r.block_width, r.block_height, r.hal_format),
        (640, 360, YV12)
    );
    assert_eq!(
        unsafe { nv_c2_dec_allocation(u32::MAX, 359, 1, 1, &mut r) },
        INVALID
    );
}
#[test]
fn malformed_envelopes_and_planes() {
    let (s, r) = native();
    for mutate in [
        |s: &mut NvC2GrallocSnapshot| s.native_version = 16,
        |s: &mut NvC2GrallocSnapshot| s.num_fds = 0,
        |s: &mut NvC2GrallocSnapshot| s.num_fds = 6,
        |s: &mut NvC2GrallocSnapshot| s.num_ints = -1,
        |s: &mut NvC2GrallocSnapshot| s.payload_words = 34,
        |s: &mut NvC2GrallocSnapshot| s.num_ints = 1,
        |s: &mut NvC2GrallocSnapshot| s.magic = 0,
        |s: &mut NvC2GrallocSnapshot| s.num_planes = 1,
        |s: &mut NvC2GrallocSnapshot| s.fds[1] = -1,
        |s: &mut NvC2GrallocSnapshot| s.width = 639,
        |s: &mut NvC2GrallocSnapshot| s.width = 638,
        |s: &mut NvC2GrallocSnapshot| s.drm_format = 0,
        |s: &mut NvC2GrallocSnapshot| s.droid_format = 0,
        |s: &mut NvC2GrallocSnapshot| s.reserved[0] = 1,
    ] {
        let mut b = s;
        mutate(&mut b);
        assert!(target(&b, &r).is_err());
    }
}
#[test]
fn metadata_fd_never_becomes_chroma() {
    let (mut s, r) = native();
    s.num_planes = 1;
    s.num_fds = 2;
    s.reserved_region_size = 4096;
    s.total_size += 4096;
    s.fds[1] = 2222;
    assert!(target(&s, &r).is_err());
    s.num_planes = 2;
    s.num_fds = 3;
    s.num_ints = 32;
    s.fds[2] = 3333;
    let t = target(&s, &r).unwrap();
    assert_eq!(t.dmabuf_fd, [1000, 2222]);
    s.reserved_region_size = s.total_size + 1;
    assert!(target(&s, &r).is_err());
}
#[test]
fn invalid_strides_ranges_and_overlap() {
    let (s, r) = native();
    for mutate in [
        |s: &mut NvC2GrallocSnapshot| s.strides[0] = 0,
        |s: &mut NvC2GrallocSnapshot| s.strides[0] = 639,
        |s: &mut NvC2GrallocSnapshot| s.strides[1] = 320,
        |s: &mut NvC2GrallocSnapshot| s.offsets[1] = u32::MAX,
        |s: &mut NvC2GrallocSnapshot| s.sizes[1] = 1,
        |s: &mut NvC2GrallocSnapshot| s.offsets[1] = 128,
        |s: &mut NvC2GrallocSnapshot| s.total_size = 1024,
    ] {
        let mut b = s;
        mutate(&mut b);
        assert!(target(&b, &r).is_err());
    }
}
#[test]
fn yv12_plane_order_and_linear_only() {
    let (mut s, mut r) = native();
    r.hal_format = YV12;
    r.policy = 1;
    s.droid_format = YV12 as i32;
    s.drm_format = YV12;
    s.num_planes = 3;
    s.num_fds = 3;
    s.num_ints = 32;
    s.fds = [1000, 1001, 1002, -1, -1];
    s.strides = [640, 320, 320, 0];
    s.offsets = [0, 640 * 360, 640 * 360 + 320 * 180, 0];
    s.sizes = [640 * 360, 320 * 180, 320 * 180, 0];
    s.total_size = 640 * 540;
    let t = target(&s, &r).unwrap();
    assert_eq!(t.format, 3);
    assert_eq!(t.dmabuf_fd, [1000, 1002]);
    assert_eq!(t.cr_dmabuf_fd, 1001);
    assert_eq!(t.offset[1], (640 * 360 + 320 * 180) as u64);
    s.drm_format = YVU420_ANDROID;
    assert!(target(&s, &r).is_ok());
    s.modifier = (3 << 56) | 0x200010;
    assert!(target(&s, &r).is_err());
    s.modifier = 0;
    s.offsets[2] += 1;
    assert!(target(&s, &r).is_err());
}
#[test]
fn p010_and_block_linear() {
    let (mut s, mut r) = native();
    r.hal_format = 0x36;
    s.droid_format = 0x36;
    s.drm_format = 0x30313050;
    s.strides = [1280, 1280, 0, 0];
    s.offsets[1] = 1280 * 368;
    s.sizes = [1280 * 368, 1280 * 184, 0, 0];
    s.total_size = 1280 * 552;
    assert!(target(&s, &r).is_ok());
    s.offsets[1] += 1;
    assert!(target(&s, &r).is_err());
    s.offsets[1] -= 1;
    s.modifier = (3 << 56) | 0x200010;
    assert!(target(&s, &r).is_ok());
    s.modifier = (3 << 56) | 0x300010;
    assert!(target(&s, &r).is_err());
    s.modifier = (3 << 56) | 0x200016;
    assert!(target(&s, &r).is_err());
}
#[test]
fn max_input_byte_identical_and_overflow() {
    for (w, h, expected) in [
        (320, 240, 2_097_152),
        (1920, 1080, 2_097_152),
        (3840, 2160, 6_266_880),
        (8192, 8192, 50_331_648),
    ] {
        let mut n = 0;
        assert_eq!(unsafe { nv_c2_dec_max_input_size(w, h, &mut n) }, OK);
        assert_eq!(n, expected);
    }
    let mut n = 42;
    assert_eq!(
        unsafe { nv_c2_dec_max_input_size(u32::MAX, u32::MAX, &mut n) },
        INVALID
    );
    assert_eq!(n, 42);
    assert_eq!(unsafe { nv_c2_dec_max_input_size(0, 1, &mut n) }, INVALID);
}
