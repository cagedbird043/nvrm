//! H.264 SPS/PPS (ITU-T H.264 7.3.2.1/7.3.2.2) and HDR SEI.
#[derive(Clone, Debug, Default)]
pub struct H264Params {
    pub width: i32,
    pub height: i32,
    pub profile_idc: i32,
    pub level_idc: i32,
    pub constraint_flags: i32,
    pub log2_max_frame_num_minus4: i32,
    pub log2_max_poc_lsb_minus4: i32,
    pub poc_type: i32,
    pub num_reorder: i32,
    pub num_ref_idx_l0_minus1: i32,
    pub num_ref_frames: i32,
    pub entropy_cabac: bool,
    pub pic_init_qp: i32,
    pub transform_8x8: bool,
    pub vui: bool,
    pub full_range: bool,
    pub colour: [i32; 3],
    pub chroma444: bool,
    pub lossless: bool,
}

pub(crate) struct Bw<const N: usize> {
    pub(crate) buf: [u8; N],
    pub(crate) bits: usize,
}
impl<const N: usize> Bw<N> {
    pub(crate) fn new() -> Self {
        Self {
            buf: [0; N],
            bits: 0,
        }
    }
    pub(crate) fn put(&mut self, v: u32, n: i32) {
        for i in (0..n).rev() {
            if (v >> i) & 1 != 0 {
                self.buf[self.bits >> 3] |= 0x80 >> (self.bits & 7);
            }
            self.bits += 1;
        }
    }
    pub(crate) fn flag(&mut self, v: bool) {
        self.put(v as u32, 1);
    }
    pub(crate) fn ue(&mut self, v: u32) {
        let x = v.wrapping_add(1);
        let n = 31 - x.leading_zeros();
        self.put(0, n as i32);
        self.put(x, n as i32 + 1);
    }
    pub(crate) fn se(&mut self, v: i32) {
        self.ue(if v > 0 {
            2 * v as u32 - 1
        } else {
            2 * (-v) as u32
        });
    }
    pub(crate) fn align(&mut self) {
        while self.bits & 7 != 0 {
            self.put(0, 1);
        }
    }
    pub(crate) fn trailing(&mut self) {
        self.put(1, 1);
        self.align();
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.buf[..self.bits / 8]
    }
}
pub(crate) fn escape(rbsp: &[u8], out: &mut Vec<u8>) {
    let mut zeros = 0;
    for &v in rbsp {
        if zeros >= 2 && v <= 3 {
            out.push(3);
            zeros = 0;
        }
        out.push(v);
        zeros = if v == 0 { zeros + 1 } else { 0 };
    }
}
fn emit(b: &Bw<128>, header: u8, out: &mut Vec<u8>) {
    out.extend_from_slice(&[0, 0, 0, 1, header]);
    escape(b.bytes(), out);
}
pub fn write_sps(p: &H264Params, out: &mut Vec<u8>) {
    let mut b = Bw::<128>::new();
    let wmb = (p.width + 15) / 16;
    let hmb = (p.height + 15) / 16;
    b.put(p.profile_idc as u32, 8);
    b.put(p.constraint_flags as u32, 8);
    b.put(p.level_idc as u32, 8);
    b.ue(0);
    if p.profile_idc >= 100 {
        b.ue(if p.chroma444 { 3 } else { 1 });
        if p.chroma444 {
            b.put(0, 1);
        }
        b.ue(0);
        b.ue(0);
        b.flag(p.lossless);
        b.put(0, 1);
    }
    b.ue(p.log2_max_frame_num_minus4 as u32);
    b.ue(p.poc_type as u32);
    if p.poc_type == 0 {
        b.ue(p.log2_max_poc_lsb_minus4 as u32);
    }
    b.ue(p.num_ref_frames as u32);
    b.put(0, 1);
    b.ue(wmb as u32 - 1);
    b.ue(hmb as u32 - 1);
    b.put(1, 1);
    b.put(1, 1);
    let cu = if p.chroma444 { 1 } else { 2 };
    let cr = (wmb * 16 - p.width) / cu;
    let cb = (hmb * 16 - p.height) / cu;
    b.flag(cr != 0 || cb != 0);
    if cr != 0 || cb != 0 {
        b.ue(0);
        b.ue(cr as u32);
        b.ue(0);
        b.ue(cb as u32);
    }
    b.flag(p.vui);
    if p.vui {
        b.put(0, 1);
        b.put(0, 1);
        b.put(1, 1);
        b.put(5, 3);
        b.flag(p.full_range);
        b.put(1, 1);
        for c in p.colour {
            b.put(if c != 0 { c } else { 1 } as u32, 8);
        }
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(0, 1);
        b.put(1, 1);
        b.put(1, 1);
        b.ue(2);
        b.ue(1);
        b.ue(16);
        b.ue(16);
        b.ue(p.num_reorder as u32);
        b.ue(p.num_ref_frames as u32);
    }
    b.trailing();
    emit(&b, 0x67, out);
}
pub fn write_pps(p: &H264Params, out: &mut Vec<u8>) {
    let mut b = Bw::<128>::new();
    b.ue(0);
    b.ue(0);
    b.flag(p.entropy_cabac);
    b.put(0, 1);
    b.ue(0);
    b.ue(p.num_ref_idx_l0_minus1 as u32);
    b.ue(0);
    b.put(0, 1);
    b.put(0, 2);
    b.se(p.pic_init_qp - 26);
    b.se(0);
    b.se(0);
    b.put(1, 1);
    b.put(0, 1);
    b.put(0, 1);
    if p.transform_8x8 {
        b.put(1, 1);
        b.put(0, 1);
        b.se(0);
    }
    b.trailing();
    emit(&b, 0x68, out);
}
/// H.264 D.1.29/D.1.31, H.265 D.2.28/D.2.35; HEVC prefix SEI or H.264 SEI.
pub fn write_hdr_sei(
    hevc: bool,
    mdcv: Option<&[u8; 24]>,
    cll: Option<&[u8; 4]>,
    out: &mut Vec<u8>,
) {
    let mut rbsp = [0; 64];
    let mut n = 0;
    if let Some(v) = mdcv {
        rbsp[..2].copy_from_slice(&[137, 24]);
        rbsp[2..26].copy_from_slice(v);
        n = 26;
    }
    if let Some(v) = cll {
        rbsp[n..n + 2].copy_from_slice(&[144, 4]);
        rbsp[n + 2..n + 6].copy_from_slice(v);
        n += 6;
    }
    if n == 0 {
        return;
    }
    rbsp[n] = 0x80;
    n += 1;
    out.extend_from_slice(&[0, 0, 0, 1]);
    if hevc {
        out.extend_from_slice(&[39 << 1, 1]);
    } else {
        out.push(6);
    }
    escape(&rbsp[..n], out);
}

#[cfg(test)]
mod tests {
    use super::*;
    // Golden bytes generated by linking the original mickys C implementation.
    #[test]
    fn c_golden_bytes() {
        let mut out = vec![0x5a];
        let p = H264Params {
            width: 16,
            height: 16,
            profile_idc: 66,
            level_idc: 41,
            constraint_flags: 192,
            log2_max_frame_num_minus4: 4,
            log2_max_poc_lsb_minus4: 5,
            poc_type: 0,
            num_reorder: 2,
            num_ref_idx_l0_minus1: 1,
            num_ref_frames: 3,
            entropy_cabac: false,
            pic_init_qp: 29,
            transform_8x8: false,
            vui: false,
            full_range: false,
            colour: [0, 0, 0],
            chroma444: false,
            lossless: false,
            ..Default::default()
        };
        write_sps(&p, &mut out);
        write_pps(&p, &mut out);
        assert_eq!(out[0], 0x5a);
        assert_eq!(
            &out[1..],
            &[
                0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0xc0, 0x29, 0x96, 0x62, 0x3c, 0x80, 0x00, 0x00,
                0x00, 0x01, 0x68, 0xca, 0x83, 0x72
            ]
        );
        out.truncate(1);
        let p = H264Params {
            width: 721,
            height: 479,
            profile_idc: 244,
            level_idc: 41,
            constraint_flags: 0,
            log2_max_frame_num_minus4: 4,
            log2_max_poc_lsb_minus4: 5,
            poc_type: 2,
            num_reorder: 2,
            num_ref_idx_l0_minus1: 1,
            num_ref_frames: 3,
            entropy_cabac: true,
            pic_init_qp: 0,
            transform_8x8: true,
            vui: true,
            full_range: true,
            colour: [9, 16, 9],
            chroma444: true,
            lossless: true,
            ..Default::default()
        };
        write_sps(&p, &mut out);
        write_pps(&p, &mut out);
        assert_eq!(out[0], 0x5a);
        assert_eq!(
            &out[1..],
            &[
                0x00, 0x00, 0x00, 0x01, 0x67, 0xf4, 0x00, 0x29, 0x91, 0xc5, 0x64, 0x02, 0xe0, 0xf7,
                0x84, 0x2a, 0x6e, 0x12, 0x20, 0x12, 0x0d, 0xa0, 0x88, 0x45, 0x92, 0x00, 0x00, 0x00,
                0x01, 0x68, 0xea, 0x80, 0x6b, 0xcb
            ]
        );
        out.truncate(1);
    }
}
