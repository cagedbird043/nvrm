use super::*;
use super::super::{h264hdr, hevchdr, tmpl_av1};
use crate::video::av1::header::ParserState;

fn config(codec: i32) -> EncConfig {
    EncConfig { codec, width: 640, height: 360, num_refs: 1, bit_depth: 8,
        fps: 30, qp_i: 24, qp_p: 27, ..Default::default() }
}
fn h264_params() -> h264hdr::H264Params {
    h264hdr::H264Params { width: 640, height: 360, profile_idc: 100, level_idc: 31,
        log2_max_frame_num_minus4: 2, log2_max_poc_lsb_minus4: 1, poc_type: 0,
        num_ref_frames: 1, pic_init_qp: 31, entropy_cabac: true, transform_8x8: true,
        ..Default::default() }
}
#[test]
fn h264_split_lifetime_and_nondefault_setup() {
    let p = h264_params();
    let mut sps = Vec::new(); h264hdr::write_sps(&p, &mut sps);
    let mut pps = Vec::new(); h264hdr::write_pps(&p, &mut pps);
    let mut bank = ParameterSets::new(ENC_H264);
    bank.update(&pps).unwrap();
    assert!(bank.syntax(&config(ENC_H264)).is_err());
    bank.update(&sps).unwrap();
    drop(sps); drop(pps);
    let PackedSyntax::H264 { sps, pps } = bank.syntax(&config(ENC_H264)).unwrap() else { panic!() };
    let mut setup = nvenc_h264_drv_pic_setup_s::default();
    apply_h264(&mut setup, &sps, &pps, 63, 35);
    assert_eq!(setup.sps_data.log2_max_frame_num_minus4(), 2);
    assert_eq!(setup.sps_data.log2_max_pic_order_cnt_lsb_minus4(), 1);
    assert_eq!(setup.pic_control.frame_num, 63);
    assert_eq!(setup.pic_control.pic_order_cnt_lsb, 3);
    assert_eq!(setup.pps_data.pic_init_qp_minus26(), 5);
    assert_eq!(setup.pps_data.entropy_coding_mode_flag(), 1);
    assert_eq!(setup.pps_data.transform_8x8_mode_flag(), 1);
    assert!(PackedPicture::H264 { frame_num: 64, poc: 0 }.validate(
        Some(&PackedSyntax::H264 { sps, pps }), &config(ENC_H264)).is_err());
}
#[test]
fn malformed_updates_preserve_parameter_bank() {
    let p = h264_params();
    let mut bytes = Vec::new(); h264hdr::write_sps(&p, &mut bytes); h264hdr::write_pps(&p, &mut bytes);
    let mut bank = ParameterSets::new(ENC_H264); bank.update(&bytes).unwrap();
    for invalid in [&[][..], &[0, 0, 1, 0x67, 0, 0, 3][..], &[0, 0, 1, 0x67, 0, 0, 3, 4][..],
        &[0, 0, 1, 0xe7, 0x80][..], &[0, 0, 1, 0x67, 0x80][..]] {
        assert!(bank.update(invalid).is_err());
        assert!(bank.syntax(&config(ENC_H264)).is_ok());
    }
    assert!(checked_unescape(&[0x67, 0, 0, 2]).is_err());
    assert_eq!(checked_unescape(&[0x67, 0, 0, 3, 2]).unwrap(), [0x67, 0, 0, 2]);
}
#[test]
fn hevc_sequence_picture_semantics_reach_setup() {
    let p = hevchdr::HevcParams { width: 640, height: 360, bit_depth: 8,
        max_dec_pic_buffering: 2, sps_rps_negative: 1, num_ref_idx_l0_default: 1,
        log2_max_poc_lsb_minus4: 2, pic_init_qp: 29, min_cb8: true,
        no_sdh: true, no_sao: true, ..Default::default() };
    let mut bytes = Vec::new();
    hevchdr::write_vps(&p, &mut bytes); hevchdr::write_sps(&p, &mut bytes); hevchdr::write_pps(&p, &mut bytes);
    let mut bank = ParameterSets::new(ENC_HEVC); bank.update(&bytes).unwrap();
    let mut cfg = config(ENC_HEVC);
    assert!(bank.syntax(&cfg).is_err());
    cfg.quality = true;
    let PackedSyntax::Hevc { sps, pps } = bank.syntax(&cfg).unwrap() else { panic!() };
    let mut setup = nvenc_h265_drv_pic_setup_s::default();
    let mut slice = nvenc_h265_slice_control_s::default();
    apply_hevc(&mut setup, &sps, &pps, 67, &cfg); apply_hevc_slice(&mut slice, &sps, &pps);
    assert_eq!(setup.pic_control.pic_order_cnt_lsb, 3);
    assert_eq!(setup.sps_data.min_cu_size(), 0);
    assert_eq!(setup.sps_data.sample_adaptive_offset_enabled_flag(), 0);
    assert_eq!(slice.slice_sao_luma_flag(), 0);
    assert_eq!(setup.pic_control.short_term_ref_pic_set_sps_flag(), 0);
    let mut unsupported = sps.clone(); unsupported.pcm = true;
    assert!(PackedSyntax::Hevc { sps: unsupported, pps }.validate(&cfg).is_err());
}
fn av1_sequence(bits: i32, cdef: bool, restoration: bool, depth: i32) -> av1hdr::Av1SeqParams {
    av1hdr::Av1SeqParams { width: 640, height: 360, bit_depth: depth, order_hint_bits: bits,
        enable_cdef: cdef, enable_restoration: restoration, full_range: true, colour: [9, 16, 9],
        ..Default::default() }
}
#[test]
fn av1_sequence_and_frame_writer_share_hint_and_filter_syntax() {
    for depth in [8, 10] {
        for bits in [1, 7, 8] {
            for cdef in [false, true] {
                for restoration in [false, true] {
                    let seq = av1_sequence(bits, cdef, restoration, depth);
                    let mut bytes = Vec::new(); av1hdr::write_sequence_header(&seq, &mut bytes);
                    let mut bank = ParameterSets::new(ENC_AV1); bank.update(&bytes).unwrap();
                    let mut cfg = config(ENC_AV1); cfg.bit_depth = depth;
                    let syntax = bank.syntax(&cfg).unwrap();
                    let shared = syntax.av1_params().unwrap();
                    let PackedSyntax::Av1 { sequence } = syntax else { panic!() };
                    assert_eq!(sequence.color_description, [9, 16, 9]);
                    let mut parser = ParserState { seq: Some(sequence), ..Default::default() };
                    for hint in [0, 127, 128, 255, 256] {
                        let f = av1hdr::Av1FrameParams { order_hint: hint, base_q_idx: 100,
                            cdef_damping: 3, lf_level: [10; 4], tile_size_bytes: 4,
                            ..Default::default() };
                        let mut out = Vec::new(); av1hdr::write_frame(&shared, &f, &[&[0x80]], &mut out);
                        let units = obu::split_obus(&out).unwrap();
                        let header = parser.parse_frame_header(units[0].payload, 0, 0).unwrap();
                        assert_eq!(header.order_hint, hint as u32 & ((1 << bits) - 1));
                        assert_eq!(header.base_q_idx, 100);
                        assert_eq!(header.frame_restoration_type, [0; 3]);
                    }
                }
            }
        }
    }
}
#[test]
fn av1_hardware_filter_controls_preserve_quantization_after_hq() {
    for template in [&tmpl_av1::AV1_TMPL_KEY[..], &tmpl_av1::AV1_TMPL_P[..]] {
        for hq in [false, true] {
            for cdef in [false, true] {
                for restoration in [false, true] {
                    let mut setup = [0u8; 4096]; setup[..template.len()].copy_from_slice(template);
                    if hq {
                        setup[0xa8..0xc0].copy_from_slice(&tmpl_av1::AV1_TMPL_HQ_FH[..0x18]);
                        setup[0xc4..0xc8].copy_from_slice(&tmpl_av1::AV1_TMPL_HQ_FH[0x1c..0x20]);
                    }
                    setup[0xc0..0xc4].copy_from_slice(&[100, 3, 7, 11]);
                    apply_av1(&mut setup, &av1_sequence(8, cdef, restoration, 8));
                    let seq = unsafe { std::ptr::read_unaligned(setup.as_ptr().add(0x84).cast::<nvenc_av1_seq_hdr_data_s>()) };
                    assert_eq!(seq.enable_cdef(), cdef as u32);
                    assert_eq!(seq.enable_restoration(), restoration as u32);
                    assert_eq!(&setup[0xc0..0xc4], &[100, 3, 7, 11]);
                    if !cdef { assert!(setup[0xa8..0xc0].iter().all(|&b| b == 0)); }
                    if !restoration { assert!(setup[0xc4..0xc8].iter().all(|&b| b == 0)); }
                }
            }
        }
    }
}
#[test]
fn av1_framing_tools_and_wrap_boundaries() {
    let mut bank = ParameterSets::new(ENC_AV1);
    for invalid in [&[0x08, 0][..], &[0x0a, 0x80][..], &[0x0b, 1, 0x80][..], &[0x0e, 1, 0, 0][..]] {
        assert!(bank.update(invalid).is_err());
    }
    assert_eq!(unwrap_hint(Some(127), 128, 8), 128);
    assert_eq!(unwrap_hint(Some(255), 0, 8), 256);
    assert_eq!(unwrap_hint(Some(128), 127, 8), 127);
    assert_eq!(unwrap_hint(Some(127), 0, 7), 128);
    assert_eq!(unwrap_hint(Some(1), 0, 1), 2);
    let mut bytes = Vec::new(); av1hdr::write_sequence_header(&av1_sequence(8, false, false, 8), &mut bytes);
    bank.update(&bytes).unwrap();
    let PackedSyntax::Av1 { mut sequence } = bank.syntax(&config(ENC_AV1)).unwrap() else { panic!() };
    sequence.enable_superres = true;
    assert!(PackedSyntax::Av1 { sequence }.validate(&config(ENC_AV1)).is_err());
}

#[test]
fn h264_hrd_clock_and_cpb_match_hardware_rate_control() {
    let mut b = h264hdr::Bw::<128>::new();
    b.put(100, 8); b.put(0, 8); b.put(31, 8); b.ue(0);
    b.ue(1); b.ue(0); b.ue(0); b.flag(false); b.flag(false);
    b.ue(2); b.ue(0); b.ue(1); b.ue(1); b.flag(false);
    b.ue(39); b.ue(22); b.flag(true); b.flag(true);
    b.flag(true); b.ue(0); b.ue(0); b.ue(0); b.ue(4);
    b.flag(true); // VUI
    b.flag(false); b.flag(false); b.flag(false); b.flag(false);
    b.flag(true); b.put(1001, 32); b.put(60000, 32); b.flag(true);
    b.flag(true); // NAL HRD
    b.ue(0); b.put(0, 4); b.put(0, 4);
    b.ue(15624); b.ue(124999); b.flag(true);
    b.put(23, 5); b.put(15, 5); b.put(7, 5); b.put(5, 5);
    b.flag(false); b.flag(true); b.flag(false); b.flag(false);
    b.trailing();
    let mut bytes = vec![0, 0, 0, 1, 0x67]; h264hdr::escape(b.bytes(), &mut bytes);
    h264hdr::write_pps(&h264_params(), &mut bytes);
    let mut bank = ParameterSets::new(ENC_H264); bank.update(&bytes).unwrap();
    let mut cfg = config(ENC_H264); cfg.rc_mode = 2; cfg.bitrate = 1_000_000;
    let PackedSyntax::H264 { sps, pps } = bank.syntax(&cfg).unwrap() else { panic!() };
    let hrd = sps.vui.nal_hrd.as_ref().unwrap();
    assert_eq!(hrd.cpb, [(1_000_000, 2_000_000, true)]);
    assert_eq!((hrd.initial_cpb_removal_delay_length, hrd.cpb_removal_delay_length,
        hrd.dpb_output_delay_length, hrd.time_offset_length), (24, 16, 8, 5));
    assert_eq!(sps.vui.timing, Some((1001, 60000, true)));
    assert!(sps.vui.low_delay_hrd);
    let mut setup = nvenc_h264_drv_pic_setup_s::default(); setup.rate_control.hrd_type = 34;
    apply_h264(&mut setup, &sps, &pps, 0, 0);
    assert_eq!(setup.rate_control.nal_bitrate, 1_000_000);
    assert_eq!(setup.rate_control.nal_cpb_size, 2_000_000);
    assert_eq!(setup.rate_control.buffersize, 2_000_000);
    assert_eq!(setup.rate_control.framerate, 60000 * 128 / 1001);
    assert_eq!(setup.pps_data.pic_init_qp_minus26(), 5);
    let mut unsupported = sps.clone(); unsupported.vui.nal_hrd.as_mut().unwrap().cpb.push((1, 1, false));
    assert!(PackedSyntax::H264 { sps: unsupported, pps }.validate(&cfg).is_err());
}

#[test]
fn h264_multiple_picture_parameter_sets_select_by_client_id() {
    let params = h264_params();
    let mut bytes = Vec::new();
    h264hdr::write_sps(&params, &mut bytes); h264hdr::write_pps(&params, &mut bytes);
    let mut b = h264hdr::Bw::<64>::new();
    b.ue(3); b.ue(0); b.flag(true); b.flag(false); b.ue(0);
    b.ue(0); b.ue(0); b.flag(false); b.put(0, 2);
    b.se(-7); b.se(0); b.se(-2); b.flag(true); b.flag(false); b.flag(false);
    b.flag(true); b.flag(false); b.se(3); b.trailing();
    bytes.extend_from_slice(&[0, 0, 1, 0x68]); h264hdr::escape(b.bytes(), &mut bytes);
    let mut bank = ParameterSets::new(ENC_H264); bank.update(&bytes).unwrap();
    for (id, qp, cb, cr) in [(0, 5, 0, 0), (3, -7, -2, 3)] {
        let PackedSyntax::H264 { sps, pps } = bank.syntax_for_picture(&config(ENC_H264), id).unwrap() else { panic!() };
        let mut setup = nvenc_h264_drv_pic_setup_s::default(); apply_h264(&mut setup, &sps, &pps, 0, 0);
        assert_eq!(setup.pps_data.pic_param_set_id(), id);
        assert_eq!(setup.pps_data.pic_init_qp_minus26(), qp);
        assert_eq!(setup.pps_data.chroma_qp_index_offset(), cb);
        assert_eq!(setup.pps_data.second_chroma_qp_index_offset(), cr);
    }
    assert!(bank.syntax_for_picture(&config(ENC_H264), 2).is_err());
    bytes.extend_from_slice(&[0, 0, 1]);
    assert!(bank.update(&bytes).is_err());
    assert!(bank.syntax_for_picture(&config(ENC_H264), 3).is_ok());
}

#[test]
fn independent_h264_parameter_classes_keep_va_semantics() {
    let mut cfg = config(ENC_H264); cfg.h264_profile = 100;
    let mut defaults = PackedDefaults::for_config(&cfg, 31).unwrap();
    let PackedDefaults::H264 { sps, pps } = &mut defaults else { panic!() };
    sps.id = 2; sps.log2_max_frame_num_minus4 = 1; sps.pic_order_cnt_type = 0; sps.log2_max_poc_lsb_minus4 = 2;
    sps.vui.timing = Some((1001, 60000, true));
    sps.vui.aspect_ratio_idc = Some(255); sps.vui.sar = [4, 3];
    pps.id = 3; pps.sps_id = 2; pps.entropy_coding_mode = true; pps.transform_8x8_mode = false;
    pps.chroma_qp_index_offset = -2; pps.second_chroma_qp_index_offset = 3;
    pps.deblocking_filter_control_present = false; pps.constrained_intra_pred = true;
    for sequence in [false, true] {
        let PackedDefaults::H264 { sps, pps } = &defaults else { panic!() };
        let mut client = Vec::new();
        if sequence { h264hdr::write_typed_sps(sps, 2, &mut client); } else { h264hdr::write_typed_pps(pps, &mut client); }
        let mut bank = ParameterSets::new(ENC_H264); bank.update(&client).unwrap();
        let original = bank.sets.clone();
        let generated = bank.complete_unrequested(&cfg, sequence, !sequence, 3, 31, Some(&defaults)).unwrap();
        assert_eq!(bits::split_annexb(&generated).iter().map(|n| n.nal[0] & 31).collect::<Vec<_>>(),
            if sequence { vec![8] } else { vec![7] });
        for original in &original { assert!(bank.sets.iter().any(|s| s == original)); }
        let PackedSyntax::H264 { sps, pps } = bank.syntax_for_picture(&cfg, 3).unwrap() else { panic!() };
        let mut setup = nvenc_h264_drv_pic_setup_s::default(); apply_h264(&mut setup, &sps, &pps, 31, 65);
        assert_eq!(setup.sps_data.log2_max_frame_num_minus4(), 1);
        assert_eq!(setup.pic_control.pic_order_cnt_lsb, 1);
        assert_eq!(setup.pps_data.pic_init_qp_minus26(), 5);
        assert_eq!(setup.pps_data.entropy_coding_mode_flag(), 1);
        assert_eq!(setup.pps_data.transform_8x8_mode_flag(), 0);
        assert_eq!(setup.pps_data.chroma_qp_index_offset(), -2);
        assert_eq!(setup.pps_data.second_chroma_qp_index_offset(), 3);
        assert_eq!(setup.pps_data.deblocking_filter_control_present_flag(), 0);
        assert_eq!(sps.vui.timing, Some((1001, 60000, true)));
        assert_eq!(sps.vui.aspect_ratio_idc, Some(255)); assert_eq!(sps.vui.sar, [4, 3]);
    }
    let mut absent = ParameterSets::new(ENC_H264);
    assert!(absent.complete_unrequested(&cfg, true, false, 3, 31, Some(&defaults)).is_err());
    assert!(absent.complete_unrequested(&cfg, false, true, 3, 31, Some(&defaults)).is_err());
}

#[test]
fn hevc_444_and_independent_parameter_classes_reach_setup() {
    for depth in [8, 10] {
        for chroma444 in [false, true] {
            let mut cfg = config(ENC_HEVC); cfg.bit_depth = depth; cfg.chroma444 = chroma444; cfg.quality = true;
            let mut defaults = PackedDefaults::for_config(&cfg, 31).unwrap();
            let PackedDefaults::Hevc { sps, pps } = &mut defaults else { panic!() };
            assert_eq!(sps.chroma_format_idc, if chroma444 { 3 } else { 1 });
            sps.id = 2; sps.log2_max_poc_lsb = 12;
            sps.sao = false; sps.amp = false;
            sps.encoder.level_idc = Some(123); sps.encoder.full_range = true; sps.encoder.colour = Some([9, 16, 9]);
            sps.encoder.aspect_ratio_idc = Some(255); sps.encoder.sar = [4, 3];
            pps.id = 3; pps.sps_id = 2; pps.cb_qp_offset = 3; pps.cr_qp_offset = -2;
            pps.deblocking_disabled = true; pps.deblocking_control_present = true;
            pps.cu_qp_delta = true; pps.diff_cu_qp_delta_depth = 1;
            for sequence in [false, true] {
                let PackedDefaults::Hevc { sps, pps } = &defaults else { panic!() };
                let mut client = Vec::new();
                if sequence { hevchdr::write_typed_vps(sps, 5, &mut client); hevchdr::write_typed_sps(sps, 2, 5, &mut client); }
                else { hevchdr::write_typed_pps(pps, &mut client); }
                let mut bank = ParameterSets::new(ENC_HEVC); bank.update(&client).unwrap();
                let mut va_defaults = defaults.clone();
                if sequence { let PackedDefaults::Hevc { pps, .. } = &mut va_defaults else { panic!() }; pps.sps_id = 0; }
                let generated = bank.complete_unrequested(&cfg, sequence, !sequence, 3, 31, Some(&va_defaults)).unwrap();
                assert_eq!(bits::split_annexb(&generated).iter().map(|n| (n.nal[0] >> 1) & 63).collect::<Vec<_>>(),
                    if sequence { vec![34] } else { vec![32, 33] });
                let PackedSyntax::Hevc { sps, pps } = bank.syntax_for_picture(&cfg, 3).unwrap() else { panic!() };
                assert_eq!(pps.sps_id, 2);
                let mut setup = nvenc_h265_drv_pic_setup_s::default(); let mut slice = nvenc_h265_slice_control_s::default();
                apply_hevc(&mut setup, &sps, &pps, 4097, &cfg); apply_hevc_slice(&mut slice, &sps, &pps);
                assert_eq!(setup.sps_data.chroma_format_idc(), if chroma444 { 3 } else { 1 });
                assert_eq!(setup.sps_data.log2_max_pic_order_cnt_lsb_minus4(), 8);
                assert_eq!(setup.sps_data.min_cu_size(), 0);
                assert_eq!(setup.sps_data.sample_adaptive_offset_enabled_flag(), 0);
                assert_eq!(setup.pps_data.pic_param_set_id(), 3);
                assert_eq!(setup.pps_data.pps_cb_qp_offset(), 3);
                assert_eq!(setup.pps_data.pps_cr_qp_offset(), -2);
                assert_eq!(slice.slice_deblocking_filter_disabled_flag(), 1);
                assert_eq!(sps.encoder.level_idc, Some(123));
                assert_eq!(sps.encoder.aspect_ratio_idc, Some(255)); assert_eq!(sps.encoder.sar, [4, 3]);
                assert_eq!(sps.encoder.colour, Some([9, 16, 9]));
                assert!(sps.encoder.full_range);
            }
        }
    }
}

#[test]
fn hevc_hrd_roundtrip_constrains_rate_clock_and_both_cpbs() {
    let mut cfg = config(ENC_HEVC); cfg.rc_mode = 2; cfg.fps = 25; cfg.bitrate = 800_000;
    let PackedDefaults::Hevc { mut sps, pps } = PackedDefaults::for_config(&cfg, 29).unwrap() else { panic!() };
    let hrd = hevc::ps::EncoderHrd { nal: vec![(1_000_000, 2_000_000, true)], vcl: vec![(1_200_000, 1_000_000, true)],
        initial_cpb_removal_delay_length: 26, au_cpb_removal_delay_length: 14, dpb_output_delay_length: 9,
        fixed_duration: Some(1), low_delay: false };
    sps.encoder.hrd = Some(hrd.clone());
    sps.encoder.timing = Some(hevc::ps::EncoderTiming { num_units_in_tick: 1001, time_scale: 30000, poc_ticks: Some(1) });
    sps.encoder.vps_timing = Some(hevc::ps::EncoderTiming { num_units_in_tick: 2002, time_scale: 60000, poc_ticks: Some(1) });
    sps.encoder.vps_hrd = Some(hevc::ps::EncoderHrd { nal: vec![(896_000, 1_500_000, true)], vcl: Vec::new(), ..hrd.clone() });
    let mut bytes = Vec::new(); hevchdr::write_typed_vps(&sps, 0, &mut bytes);
    hevchdr::write_typed_sps(&sps, 0, 0, &mut bytes); hevchdr::write_typed_pps(&pps, &mut bytes);
    let mut bank = ParameterSets::new(ENC_HEVC); bank.update(&bytes).unwrap();
    let PackedSyntax::Hevc { sps, pps } = bank.syntax(&cfg).unwrap() else { panic!() };
    assert_eq!(sps.encoder.hrd.as_ref(), Some(&hrd));
    assert_eq!(sps.encoder.vps_hrd.as_ref().unwrap().nal, [(896_000, 1_500_000, true)]);
    let mut setup = nvenc_h265_drv_pic_setup_s::default(); apply_hevc(&mut setup, &sps, &pps, 0, &cfg);
    assert_eq!(setup.rate_control.nal_bitrate, 896_000);
    assert_eq!(setup.rate_control.vcl_bitrate, 1_200_000);
    assert_eq!(setup.rate_control.nal_cpb_size, 1_500_000);
    assert_eq!(setup.rate_control.vcl_cpb_size, 1_000_000);
    assert_eq!(setup.rate_control.buffersize, 1_000_000);
    assert_eq!(setup.rate_control.framerate, 30000 * 256 / 1001);
    assert_eq!(setup.rate_control.hrd_type, 35);
    cfg.rc_mode = 0; assert!(bank.syntax(&cfg).is_err()); cfg.rc_mode = 2;
    let mut invalid = sps.clone(); invalid.encoder.vps_timing.as_mut().unwrap().time_scale = 50000;
    assert!(PackedSyntax::Hevc { sps: invalid, pps: pps.clone() }.validate(&cfg).is_err());
    let mut invalid = sps.clone(); invalid.encoder.hrd.as_mut().unwrap().fixed_duration = None;
    assert!(PackedSyntax::Hevc { sps: invalid, pps }.validate(&cfg).is_err());
    let mut r = BitReader::new(&[0b1010_0000]);
    assert!(hevc::ps::encoder_hrd(&mut r, 0).is_err()); // Explicitly unsupported sub-picture HRD.
}

#[test]
fn av1_inter_hint_skips_and_wrap_share_reference_timeline() {
    let seq = av1_sequence(8, false, false, 8);
    let mut bytes = Vec::new(); av1hdr::write_sequence_header(&seq, &mut bytes);
    let parsed = obu::split_obus(&bytes).unwrap();
    let mut parser = ParserState { seq: Some(SequenceHeader::parse(parsed[0].payload).unwrap()), ..Default::default() };
    let mut last = None;
    for (index, (hint, expected)) in [(0, 0), (2, 2), (127, 127), (128, 128), (255, 255), (0, 256)].into_iter().enumerate() {
        let unwrapped = unwrap_picture_hint(last, hint, 8, true);
        assert_eq!(unwrapped, expected);
        if let Some(previous) = last { assert!((1..=127).contains(&unwrapped.wrapping_sub(previous))); }
        let frame = av1hdr::Av1FrameParams { frame_type: if index == 0 { 0 } else { 1 }, order_hint: unwrapped as i32,
            ref_frame_idx: [0; 7], ref_order_hint: [last.unwrap_or(0) as i32; 7], primary_ref_frame: if index == 0 { 7 } else { 0 },
            refresh_frame_flags: 1, base_q_idx: 100, lf_level: [10; 4], cdef_damping: 3, tile_size_bytes: 4,
            ..Default::default() };
        let mut output = Vec::new(); av1hdr::write_frame(&seq, &frame, &[&[0x80]], &mut output);
        let obu = obu::split_obus(&output).unwrap(); let decoded = parser.parse_frame_header(obu[0].payload, 0, 0).unwrap();
        assert_eq!(decoded.frame_type, if index == 0 { 0 } else { 1 });
        assert_eq!(decoded.order_hint, hint);
        if index > 0 { assert_eq!(decoded.ref_frame_idx[0], 0); }
        parser.update_refs(&decoded); last = Some(unwrapped);
    }
    assert_eq!(unwrap_picture_hint(Some(0), 70, 7, true), 70);
    assert_eq!(unwrap_picture_hint(Some(127), 0, 7, true), 128);
}

#[test]
fn hevc_vbr_hrd_preserves_target_below_client_cap() {
    let mut cfg = config(ENC_HEVC); cfg.rc_mode = 1; cfg.bitrate = 640_000; cfg.max_bitrate = 2_000_000;
    let PackedDefaults::Hevc { mut sps, pps } = PackedDefaults::for_config(&cfg, 28).unwrap() else { panic!() };
    sps.encoder.timing = Some(hevc::ps::EncoderTiming { num_units_in_tick: 1, time_scale: 30, poc_ticks: Some(1) });
    sps.encoder.hrd = Some(hevc::ps::EncoderHrd { nal: vec![(960_000, 1_280_000, false)], vcl: Vec::new(),
        initial_cpb_removal_delay_length: 24, au_cpb_removal_delay_length: 16, dpb_output_delay_length: 8,
        fixed_duration: Some(1), low_delay: false });
    PackedSyntax::Hevc { sps: sps.clone(), pps: pps.clone() }.validate(&cfg).unwrap();
    let mut setup = nvenc_h265_drv_pic_setup_s::default(); apply_hevc(&mut setup, &sps, &pps, 0, &cfg);
    assert_eq!(setup.rate_control.nal_bitrate, 960_000);
    assert_eq!(setup.rate_control.buffersize, 1_280_000);
    assert_eq!(setup.rate_control.hrd_type, 2);
    assert!(setup.rate_control.Ravg > 0 && setup.rate_control.Ravg < setup.rate_control.R);
    assert_eq!(setup.rate_control.framerate, 30 * 256);
}

fn escaped_parameter(nal: &[u8]) -> Vec<u8> {
    let header = if nal[0] & 31 == 7 || nal[0] & 31 == 8 { 1 } else { 2 };
    let mut out = vec![0, 0, 0, 1]; out.extend_from_slice(&nal[..header]);
    h264hdr::escape(&nal[header..], &mut out); out
}

#[test]
fn raw_h264_parameters_use_consumed_syntax_not_payload_startcodes() {
    let mut cfg = config(ENC_H264); cfg.h264_profile = 100;
    let PackedDefaults::H264 { mut sps, mut pps } = PackedDefaults::for_config(&cfg, 31).unwrap() else { panic!() };
    sps.vui.timing = Some((64, 3840, true));
    pps.transform_8x8_mode = false;
    let mut sps_bytes = Vec::new(); h264hdr::write_typed_sps(&sps, 0, &mut sps_bytes);
    let raw_sps = bits::unescape(bits::split_annexb(&sps_bytes)[0].nal);
    assert!(raw_sps.windows(3).any(|w| w == [0, 0, 1]));
    let mut pps_bytes = Vec::new(); h264hdr::write_typed_pps(&pps, &mut pps_bytes);
    let raw_pps = bits::unescape(bits::split_annexb(&pps_bytes)[0].nal);
    let mut stream = raw_sps.clone(); stream.extend_from_slice(&[0, 0, 1]); stream.extend_from_slice(&raw_pps);
    let mut bank = ParameterSets::new(ENC_H264);
    assert_eq!(bank.unescaped_parameter_len(&stream).unwrap(), raw_sps.len());
    bank.update(&escaped_parameter(&stream[..raw_sps.len()])).unwrap();
    let mut remainder = raw_pps.clone(); remainder.extend_from_slice(&[0, 0, 0, 1]); remainder.extend_from_slice(&raw_sps);
    assert_eq!(bank.unescaped_parameter_len(&remainder).unwrap(), raw_pps.len());
    assert_eq!(ParameterSets::new(ENC_H264).unescaped_parameter_len(&remainder).unwrap(), raw_pps.len());
    bank.update(&escaped_parameter(&raw_pps)).unwrap();
    let PackedSyntax::H264 { sps, .. } = bank.syntax(&cfg).unwrap() else { panic!() };
    assert_eq!(sps.vui.timing, Some((64, 3840, true)));
    assert!(bank.unescaped_parameter_len(&raw_sps[..raw_sps.len() - 1]).is_err());
}

#[test]
fn raw_hevc_parameters_preserve_internal_startcodes_and_following_units() {
    let cfg = config(ENC_HEVC);
    let PackedDefaults::Hevc { mut sps, pps } = PackedDefaults::for_config(&cfg, 29).unwrap() else { panic!() };
    let clock = hevc::ps::EncoderTiming { num_units_in_tick: 64, time_scale: 3840, poc_ticks: Some(1) };
    sps.encoder.timing = Some(clock.clone()); sps.encoder.vps_timing = Some(clock);
    let mut bytes = Vec::new(); hevchdr::write_typed_vps(&sps, 0, &mut bytes);
    hevchdr::write_typed_sps(&sps, 0, 0, &mut bytes); hevchdr::write_typed_pps(&pps, &mut bytes);
    let units: Vec<Vec<u8>> = bits::split_annexb(&bytes).iter().map(|n| bits::unescape(n.nal)).collect();
    assert!(units[..2].iter().any(|n| n.windows(3).any(|w| w == [0, 0, 1])));
    let mut bank = ParameterSets::new(ENC_HEVC);
    for i in 0..units.len() {
        let mut rest = units[i].clone();
        for later in &units[i + 1..] { rest.extend_from_slice(&[0, 0, 0, 1]); rest.extend_from_slice(later); }
        assert_eq!(bank.unescaped_parameter_len(&rest).unwrap(), units[i].len());
        bank.update(&escaped_parameter(&units[i])).unwrap();
    }
    let PackedSyntax::Hevc { sps, .. } = bank.syntax(&cfg).unwrap() else { panic!() };
    assert_eq!(sps.encoder.timing.as_ref().unwrap().time_scale, 3840);
    assert_eq!(sps.encoder.vps_timing.as_ref().unwrap().num_units_in_tick, 64);
    assert!(bank.unescaped_parameter_len(&units[1][..units[1].len() - 1]).is_err());
}

#[test]
fn raw_h264_pps_extensions_have_exact_following_nal_boundaries() {
    let mut cfg = config(ENC_H264); cfg.h264_profile = 100;
    let PackedDefaults::H264 { sps, mut pps } = PackedDefaults::for_config(&cfg, 31).unwrap() else { panic!() };
    for transform in [false, true] {
        for chroma in [-12, 0, 12] {
            pps.transform_8x8_mode = transform; pps.second_chroma_qp_index_offset = chroma;
            let mut bytes = Vec::new(); h264hdr::write_typed_pps(&pps, &mut bytes);
            let raw = bits::unescape(bits::split_annexb(&bytes)[0].nal);
            let mut following = raw.clone(); following.extend_from_slice(&[0, 0, 1]);
            let mut next = Vec::new(); h264hdr::write_typed_sps(&sps, 0, &mut next); following.extend_from_slice(&next[4..]);
            let bank = ParameterSets::new(ENC_H264);
            assert_eq!(bank.unescaped_parameter_len(&following).unwrap(), raw.len(), "transform={transform} chroma={chroma}");
        }
    }
}

#[test]
fn sequence_buffers_keep_client_picture_sets_instead_of_va_defaults() {
    for codec in [ENC_H264, ENC_HEVC] {
        let cfg = config(codec);
        let mut client = PackedDefaults::for_config(&cfg, 37).unwrap();
        let mut bytes = Vec::new();
        match &mut client {
            PackedDefaults::H264 { sps, pps } => {
                pps.entropy_coding_mode = false; pps.deblocking_filter_control_present = false;
                h264hdr::write_typed_sps(sps, sps.id, &mut bytes); h264hdr::write_typed_pps(pps, &mut bytes);
            }
            PackedDefaults::Hevc { sps, pps } => {
                pps.cabac_init_present = false; pps.cu_qp_delta = false;
                pps.deblocking_control_present = false; pps.deblocking_override_enabled = false;
                hevchdr::write_typed_vps(sps, 0, &mut bytes);
                hevchdr::write_typed_sps(sps, sps.id, 0, &mut bytes); hevchdr::write_typed_pps(pps, &mut bytes);
            }
        }
        let mut bank = ParameterSets::new(codec); bank.update(&bytes).unwrap();
        let driver = PackedDefaults::for_config(&cfg, 26).unwrap();
        for _ in 0..3 {
            assert!(bank.complete_unrequested(&cfg, true, false, 0, 26, Some(&driver)).unwrap().is_empty());
            match bank.syntax(&cfg).unwrap() {
                PackedSyntax::H264 { pps, .. } => {
                    assert_eq!(pps.pic_init_qp_minus26 + 26, 37);
                    assert!(!pps.entropy_coding_mode && !pps.deblocking_filter_control_present);
                }
                PackedSyntax::Hevc { pps, .. } => {
                    assert_eq!(pps.init_qp_minus26 + 26, 37);
                    assert!(!pps.cabac_init_present && !pps.cu_qp_delta && !pps.deblocking_control_present);
                }
                _ => panic!(),
            }
        }
        assert!(bank.complete_unrequested(&cfg, true, false, 0, 26, None).unwrap().is_empty());
    }
}

#[test]
fn driver_owned_picture_sets_can_change_without_becoming_client_owned() {
    for codec in [ENC_H264, ENC_HEVC] {
        let cfg = config(codec);
        let defaults = PackedDefaults::for_config(&cfg, 26).unwrap();
        let mut bytes = Vec::new();
        match &defaults {
            PackedDefaults::H264 { sps, .. } => h264hdr::write_typed_sps(sps, sps.id, &mut bytes),
            PackedDefaults::Hevc { sps, .. } => { hevchdr::write_typed_vps(sps, 0, &mut bytes); hevchdr::write_typed_sps(sps, sps.id, 0, &mut bytes); }
        }
        let mut bank = ParameterSets::new(codec); bank.update(&bytes).unwrap();
        for qp in [26, 37, 19] {
            let defaults = PackedDefaults::for_config(&cfg, qp).unwrap();
            assert!(!bank.complete_unrequested(&cfg, true, false, 0, qp, Some(&defaults)).unwrap().is_empty());
            let actual = match bank.syntax(&cfg).unwrap() {
                PackedSyntax::H264 { pps, .. } => pps.pic_init_qp_minus26 + 26,
                PackedSyntax::Hevc { pps, .. } => pps.init_qp_minus26 + 26,
                _ => panic!(),
            };
            assert_eq!(actual, qp);
        }
    }
}

#[test]
fn unsupported_hevc_temporal_prediction_is_rejected_before_setup() {
    let cfg = config(ENC_HEVC);
    let PackedDefaults::Hevc { mut sps, pps } = PackedDefaults::for_config(&cfg, 26).unwrap() else { panic!() };
    sps.temporal_mvp = true;
    assert!(matches!(PackedSyntax::Hevc { sps, pps }.validate(&cfg), Err(crate::video::VideoError::Unsupported(_))));
}

#[test]
fn cpb_full_capacity_must_fit_the_persistent_fixed_point_field() {
    for codec in [ENC_H264, ENC_HEVC] {
        let mut cfg = config(codec); cfg.rc_mode = 2; cfg.vbv_bits = 4_000_000;
        for (size, valid) in [(134_217_712, true), (134_217_728, false)] {
            let syntax = match PackedDefaults::for_config(&cfg, 26).unwrap() {
                PackedDefaults::H264 { mut sps, pps } => {
                    sps.vui.nal_hrd = Some(h264::ps::Hrd { cpb: vec![(1_000_000, size, true)],
                        initial_cpb_removal_delay_length: 24, cpb_removal_delay_length: 16,
                        dpb_output_delay_length: 8, time_offset_length: 0 });
                    PackedSyntax::H264 { sps, pps }
                }
                PackedDefaults::Hevc { mut sps, pps } => {
                    sps.encoder.timing = Some(hevc::ps::EncoderTiming { num_units_in_tick: 1, time_scale: 30, poc_ticks: Some(1) });
                    sps.encoder.hrd = Some(hevc::ps::EncoderHrd { nal: vec![(1_000_000, size, true)],
                        vcl: vec![(1_200_000, 2_000_000, true)], initial_cpb_removal_delay_length: 24,
                        au_cpb_removal_delay_length: 16, dpb_output_delay_length: 8,
                        fixed_duration: Some(1), low_delay: false });
                    PackedSyntax::Hevc { sps, pps }
                }
            };
            assert_eq!(syntax.validate(&cfg).is_ok(), valid);
        }
    }
}

fn hardware_hevc_slice(s: &hevc::ps::Sps, p: &hevc::ps::Pps, kind: u32, first: bool, qp: i32, payload: &[u8]) -> Vec<u8> {
    let idr = kind == 2;
    let mut b = h264hdr::Bw::<128>::new();
    b.put(if idr { 19 << 1 } else if kind == 0 { 0 } else { 1 << 1 }, 8); b.put(1, 8);
    b.flag(first); if idr { b.flag(false); } b.ue(p.id);
    if !first { b.put(1, (32 - (s.width_ctbs() * s.height_ctbs() - 1).leading_zeros()) as i32); }
    b.ue(kind);
    if !idr {
        b.put(1, s.log2_max_poc_lsb as i32); b.flag(false);
        if !s.st_rps.is_empty() { b.flag(false); }
        b.ue(1); b.ue(u32::from(kind == 0)); b.ue(0); b.flag(true);
        if kind == 0 { b.ue(0); b.flag(true); }
    }
    if s.sao { b.flag(true); b.flag(true); }
    if !idr {
        b.flag(true); b.ue(0); if kind == 0 { b.ue(0); b.flag(false); }
        if p.cabac_init_present { b.flag(false); } b.ue(0);
    }
    b.se(qp - 26);
    if p.slice_chroma_qp_offsets_present { b.se(0); b.se(0); }
    if p.deblocking_override_enabled {
        b.flag(true); b.flag(p.deblocking_disabled);
        if !p.deblocking_disabled { b.se(p.beta_offset_div2); b.se(p.tc_offset_div2); }
    }
    if p.loop_filter_across_slices && (s.sao || !p.deblocking_disabled) { b.flag(true); }
    b.trailing();
    let mut out = if first { vec![0, 0, 0, 1] } else { vec![0, 0, 1] };
    h264hdr::escape(b.bytes(), &mut out); h264hdr::escape(payload, &mut out); out
}

#[test]
fn hevc_slice_qp_translation_preserves_absolute_qp_and_entropy_boundaries() {
    use super::super::hevcslice;
    let payload = [0x12, 0, 0, 0, 1, 0x80, 0, 0, 2, 0, 0, 3, 4, 0x81];
    let mut escaped_payload = Vec::new(); h264hdr::escape(&payload, &mut escaped_payload);
    for depth in [8, 10] {
        for init in [-6, 0, 26, 37, 51] {
            if depth == 8 && init < 0 { continue; }
            for controls in 0..8 {
                let mut cfg = config(ENC_HEVC); cfg.bit_depth = depth; cfg.bframes = 1; cfg.num_refs = 2;
                let PackedDefaults::Hevc { mut sps, mut pps } = PackedDefaults::for_config(&cfg, init).unwrap() else { panic!() };
                pps.id = 3; pps.cabac_init_present = controls & 1 != 0;
                sps.sao = controls & 2 != 0;
                pps.deblocking_control_present = controls & 4 != 0;
                pps.deblocking_override_enabled = controls & 4 != 0;
                pps.deblocking_disabled = controls & 4 != 0;
                pps.slice_chroma_qp_offsets_present = controls & 4 != 0;
                for kind in [0, 1, 2] {
                    let qp = if depth == 10 && init < 0 { -6 } else { 37 };
                    let mut encoded = hardware_hevc_slice(&sps, &pps, kind, true, qp, &payload);
                    // Two independent segments exercise all-VCL rewriting, including
                    // a different header length/address and a different slice QP.
                    encoded.extend_from_slice(&hardware_hevc_slice(&sps, &pps, kind, false, qp - 1, &payload));
                    let mut output = Vec::new(); hevcslice::rewrite(&encoded, &sps, &pps, &mut output).unwrap();
                    let units = bits::split_annexb(&output); assert_eq!(units.len(), 2);
                    for (i, unit) in units.iter().enumerate() {
                        let rbsp = checked_unescape(unit.nal).unwrap();
                        let hdr = hevcslice::header(&rbsp, &sps, &pps).unwrap();
                        assert_eq!(26 + pps.init_qp_minus26 + hdr.qp_delta, qp - i as i32);
                        assert_eq!(&rbsp[hdr.data_offset..], &payload);
                        assert!(unit.nal.ends_with(&escaped_payload));
                    }
                    assert!(hevcslice::rewrite(&encoded[..5], &sps, &pps, &mut Vec::new()).is_err());
                }
            }
        }
    }
}

#[test]
fn h264_hrd_channels_use_independent_hardware_budgets() {
    let mut cfg = config(ENC_H264); cfg.rc_mode = 2; cfg.vbv_bits = 4_000_000;
    let PackedDefaults::H264 { mut sps, pps } = PackedDefaults::for_config(&cfg, 31).unwrap() else { panic!() };
    let hrd = h264::ps::Hrd { cpb: vec![(1_000_000, 1_000_000, true)],
        initial_cpb_removal_delay_length: 24, cpb_removal_delay_length: 16,
        dpb_output_delay_length: 8, time_offset_length: 0 };
    sps.vui.nal_hrd = Some(hrd.clone());
    sps.vui.vcl_hrd = Some(h264::ps::Hrd { cpb: vec![(1_200_000, 2_000_000, true)], ..hrd });
    sps.vui.timing = Some((1, 60, true));
    let mut bytes = Vec::new(); h264hdr::write_typed_sps(&sps, sps.id, &mut bytes); h264hdr::write_typed_pps(&pps, &mut bytes);
    let mut bank = ParameterSets::new(ENC_H264); bank.update(&bytes).unwrap();
    let PackedSyntax::H264 { sps, pps } = bank.syntax(&cfg).unwrap() else { panic!() };
    let mut setup = nvenc_h264_drv_pic_setup_s::default();
    setup.rate_control.hrd_type = 34;
    setup.rate_control.buffersize = cfg.vbv_bits as u32;
    setup.rate_control.nal_cpb_size = cfg.vbv_bits; setup.rate_control.vcl_cpb_size = cfg.vbv_bits;
    apply_h264(&mut setup, &sps, &pps, 0, 0);
    assert_eq!((setup.rate_control.nal_cpb_size, setup.rate_control.vcl_cpb_size), (1_000_000, 2_000_000));
    assert_eq!((setup.rate_control.nal_bitrate, setup.rate_control.vcl_bitrate), (1_000_000, 1_200_000));
    assert_eq!(setup.rate_control.buffersize, 1_000_000);
    assert_eq!(setup.rate_control.hrd_type, 35);
    assert_eq!(setup.rate_control.framerate, 30 * 256);
    cfg.rc_mode = 1; assert!(bank.syntax(&cfg).is_err());
}

#[test]
fn completing_a_missing_vps_keeps_the_client_sps_and_pps() {
    let cfg = config(ENC_HEVC);
    let PackedDefaults::Hevc { mut sps, mut pps } = PackedDefaults::for_config(&cfg, 37).unwrap() else { panic!() };
    sps.id = 2; pps.id = 3; pps.sps_id = 2;
    pps.cabac_init_present = false; pps.cu_qp_delta = false;
    let mut bytes = Vec::new(); hevchdr::write_typed_sps(&sps, 2, 5, &mut bytes); hevchdr::write_typed_pps(&pps, &mut bytes);
    let mut bank = ParameterSets::new(ENC_HEVC); bank.update(&bytes).unwrap();
    let defaults = PackedDefaults::for_config(&cfg, 26).unwrap();
    let generated = bank.complete_unrequested(&cfg, false, true, 3, 26, Some(&defaults)).unwrap();
    let units = bits::split_annexb(&generated);
    assert_eq!(units.len(), 1); assert_eq!((units[0].nal[0] >> 1) & 63, 32);
    let PackedSyntax::Hevc { sps, pps } = bank.syntax_for_picture(&cfg, 3).unwrap() else { panic!() };
    assert_eq!((sps.id, pps.sps_id, pps.init_qp_minus26 + 26), (2, 2, 37));
    assert!(!pps.cabac_init_present && !pps.cu_qp_delta);
    let later = bank.complete_unrequested(&cfg, false, true, 3, 26, Some(&defaults)).unwrap();
    assert!(bits::split_annexb(&later).iter().all(|unit| (unit.nal[0] >> 1) & 63 == 32));
}

#[test]
fn hevc_client_minimum_cu_must_match_the_actual_control_mode() {
    for quality in [false, true] {
        for no_cu8 in [false, true] {
            let mut cfg = config(ENC_HEVC); cfg.quality = quality; cfg.no_cu8 = no_cu8;
            let PackedDefaults::Hevc { sps, pps } = PackedDefaults::for_config(&cfg, 37).unwrap() else { panic!() };
            let expected = if quality && !no_cu8 { 3 } else { 4 };
            // for_config serializes and reparses the generated parameter sets.
            // Their minimum CU must agree with the selected hardware mode.
            assert_eq!(sps.log2_min_cb, expected);
            for client_minimum in [3, 4] {
                let mut client = sps.clone(); client.log2_min_cb = client_minimum;
                let syntax = PackedSyntax::Hevc { sps: client.clone(), pps: pps.clone() };
                if client_minimum == expected {
                    syntax.validate(&cfg).unwrap();
                    let mut setup = nvenc_h265_drv_pic_setup_s::default();
                    apply_hevc(&mut setup, &client, &pps, 1, &cfg);
                    assert_eq!(setup.sps_data.min_cu_size() + 3, client_minimum);
                } else {
                    assert!(matches!(syntax.validate(&cfg), Err(crate::video::VideoError::Unsupported(_))));
                }
            }
        }
    }
}

#[test]
fn hevc_coded_extent_requires_whole_ctbs_with_the_original_visible_crop() {
    for (width, height, chroma444) in [(32, 32, false), (34, 34, false),
        (640, 360, false), (648, 360, false), (641, 359, true)] {
        for depth in [8, 10] {
            let mut cfg = config(ENC_HEVC);
            cfg.width = width; cfg.height = height; cfg.chroma444 = chroma444; cfg.bit_depth = depth;
            let PackedDefaults::Hevc { sps, pps } = PackedDefaults::for_config(&cfg, 37).unwrap() else { panic!() };
            let (w, h) = (width as u32, height as u32);
            let (coded_w, coded_h) = (w.next_multiple_of(32), h.next_multiple_of(32));
            let crop_unit = if chroma444 { 1 } else { 2 };
            for (cw, ch) in [(coded_w, coded_h), (w.next_multiple_of(16), coded_h),
                (coded_w, h.next_multiple_of(16)), (coded_w + 32, coded_h), (coded_w, coded_h + 32)] {
                let mut client = sps.clone(); client.width = cw; client.height = ch;
                client.conf_win = [0, (cw - w) / crop_unit, 0, (ch - h) / crop_unit];
                // Every candidate crops to the same visible pixels. Only the
                // precise hardware CTB extent is valid, not merely any size
                // between the display extent and its next whole CTB.
                let result = PackedSyntax::Hevc { sps: client, pps: pps.clone() }.validate(&cfg);
                if (cw, ch) == (coded_w, coded_h) { result.unwrap(); }
                else { assert!(matches!(result, Err(crate::video::VideoError::Unsupported(_)))); }
            }
            let mut wrong_crop = sps; wrong_crop.conf_win[3] += 1;
            assert!(PackedSyntax::Hevc { sps: wrong_crop, pps }.validate(&cfg).is_err());
        }
    }
}
