use super::*;
use crate::video::{TestDmaBuf, VideoDevice};

const LEFT: [f32; 2] = [0.5, 1.0];
const CENTRE: [f32; 2] = [1.0, 1.0];

fn colour(matrix: Matrix, full_range: bool) -> Colour {
    Colour { matrix, full_range, chroma_site: LEFT }
}

/// The affine maps against values computed by hand from H.273: limited BT.709 white and black,
/// and the BT.601 full-range red.
#[test]
fn conversion_known_values() {
    let op = |s: Colour, d: Colour| Operation {
        src_rect: Rect { x: 0, y: 0, width: 2, height: 2 },
        dst_rect: Rect { x: 0, y: 0, width: 2, height: 2 },
        src_colour: s,
        dst_colour: d,
        background: 0xff00_0000,
    };
    let apply = |m: [[f64; 4]; 3], c: [f64; 3]| -> [f64; 3] {
        let mut o = [0.0; 3];
        for (r, v) in o.iter_mut().enumerate() {
            *v = m[r][0] * c[0] + m[r][1] * c[1] + m[r][2] * c[2] + m[r][3];
        }
        o
    };
    let rgb = Pixels::Rgb(RgbOrder::Bgra);
    let nv12 = Pixels::Yuv420 { bits: 8 };
    let p010 = Pixels::Yuv420 { bits: 10 };
    let to_rgb = conversion(nv12, rgb, &op(colour(Matrix::Bt709, false), colour(Matrix::Rgb, true)));
    for (yuv, want) in [([235.0, 128.0, 128.0], 255.0), ([16.0, 128.0, 128.0], 0.0)] {
        for v in apply(to_rgb, yuv) {
            assert!((v - want).abs() < 1e-9, "{yuv:?} -> {v}");
        }
    }
    let to_yuv = conversion(rgb, nv12, &op(colour(Matrix::Rgb, true), colour(Matrix::Bt601, true)));
    let red = apply(to_yuv, [255.0, 0.0, 0.0]);
    let want = [0.299 * 255.0, 128.0 - 0.5 * 0.299 / 0.886 * 255.0, 255.0];
    for (a, b) in red.iter().zip(want) {
        assert!((a - b).abs() < 0.51, "{red:?} vs {want:?}");
    }
    // Same matrix, 8-bit limited to 10-bit limited: x4.
    let up = conversion(nv12, p010, &op(colour(Matrix::Bt709, false), colour(Matrix::Bt709, false)));
    let o = apply(up, [100.0, 50.0, 200.0]);
    for (a, b) in o.iter().zip([400.0, 200.0, 800.0]) {
        assert!((a - b).abs() < 1e-9, "{o:?}");
    }
    let bg = background_codes(nv12, &op(colour(Matrix::Rgb, true), colour(Matrix::Bt709, false)));
    assert_eq!(bg, [16, 128, 128]);
}

/// A CPU picture and its geometry: planes packed, `pitch` bytes per row.
struct Cpu {
    pixels: Pixels,
    width: u32,
    height: u32,
    pitch: usize,
    offsets: [usize; 3],
    bytes: Vec<u8>,
}

impl Cpu {
    fn new(pixels: Pixels, width: u32, height: u32) -> Self {
        let pitch = (width * pixels.bps()) as usize + 8;
        let mut offsets = [0usize; 3];
        let mut at = 0;
        for (p, o) in offsets.iter_mut().enumerate().take(pixels.planes()) {
            *o = at;
            let (_, rows) = pixels.plane_extent(p, width, height);
            at += pitch * rows as usize;
        }
        Self { pixels, width, height, pitch, offsets, bytes: vec![0; at] }
    }

    /// Pseudo-random samples of the format's depth.
    fn random(pixels: Pixels, width: u32, height: u32, seed: u32) -> Self {
        let mut pic = Self::new(pixels, width, height);
        let mut s = seed.wrapping_mul(2654435761).wrapping_add(1);
        for p in 0..pixels.planes() {
            let (_, rows) = pixels.plane_extent(p, width, height);
            for y in 0..rows {
                for e in 0..pic.elements(p) {
                    for c in 0..pic.components(p) {
                        s ^= s << 13;
                        s ^= s >> 17;
                        s ^= s << 5;
                        // Smooth gradient plus noise keeps scaled pictures meaningful.
                        let max = (1u32 << pixels.bits()) - 1;
                        let v = ((e * 7 + y * 3 + c * 50) % (max + 1)).wrapping_add(s % 32) % (max + 1);
                        pic.set(p, e, c, y, v);
                    }
                }
            }
        }
        pic
    }

    fn elements(&self, p: usize) -> u32 {
        match (self.pixels, p) {
            (Pixels::Yuv420 { .. }, 1) => self.width.div_ceil(2),
            _ => self.width,
        }
    }

    fn components(&self, p: usize) -> u32 {
        match (self.pixels, p) {
            (Pixels::Yuv420 { .. }, 1) => 2,
            (Pixels::Rgb(_), _) => 4,
            _ => 1,
        }
    }

    fn at(&self, p: usize, e: u32, c: u32, y: u32) -> usize {
        let bps = match self.pixels {
            Pixels::Rgb(_) => 1,
            px => px.bps() as usize,
        };
        self.offsets[p] + y as usize * self.pitch + (e * self.components(p) + c) as usize * bps
    }

    /// Code (container value shifted down) of component `c` of element `e`, row `y` of plane `p`.
    fn get(&self, p: usize, e: u32, c: u32, y: u32) -> u32 {
        let i = self.at(p, e, c, y);
        if self.pixels.bps() == 2 {
            u32::from(u16::from_le_bytes([self.bytes[i], self.bytes[i + 1]])) >> (16 - self.pixels.bits())
        } else {
            u32::from(self.bytes[i])
        }
    }

    fn set(&mut self, p: usize, e: u32, c: u32, y: u32, v: u32) {
        let i = self.at(p, e, c, y);
        if self.pixels.bps() == 2 {
            let raw = ((v << (16 - self.pixels.bits())) as u16).to_le_bytes();
            self.bytes[i..i + 2].copy_from_slice(&raw);
        } else {
            self.bytes[i] = v as u8;
        }
    }

    fn picture(&self) -> Picture<'_> {
        Picture {
            pixels: self.pixels,
            width: self.width,
            height: self.height,
            memory: Memory::Host { bytes: &self.bytes, pitch: self.pitch, offsets: self.offsets },
        }
    }

    /// (R, G, B) or (Y, Cb, Cr) codes and alpha of pixel (x, y); 4:2:0 chroma of its sample.
    fn pixel(&self, x: u32, y: u32) -> [u32; 4] {
        match self.pixels {
            Pixels::Rgb(order) => {
                let (b0, b1, b2, a) = (self.get(0, x, 0, y), self.get(0, x, 1, y), self.get(0, x, 2, y), self.get(0, x, 3, y));
                if order == RgbOrder::Bgra { [b2, b1, b0, a] } else { [b0, b1, b2, a] }
            }
            Pixels::Yuv420 { .. } => [self.get(0, x, 0, y), self.get(1, x / 2, 0, y / 2), self.get(1, x / 2, 1, y / 2), 0],
            Pixels::Yuv444 { .. } => [self.get(0, x, 0, y), self.get(1, x, 0, y), self.get(2, x, 0, y), 0],
        }
    }
}

/// A dma-buf picture laid out like the frontend's surface backings: block linear one GOB high
/// (or linear), planes at 64-row multiples.
struct Dma {
    buf: TestDmaBuf,
    planes: [Plane; 3],
    layout: Layout,
    pixels: Pixels,
    width: u32,
    height: u32,
}

impl Dma {
    fn new(dev: &VideoDevice, pixels: Pixels, width: u32, height: u32, layout: Layout) -> Self {
        let pitch = (width * pixels.bps()).next_multiple_of(64);
        let mut planes = [Plane { fd: -1, offset: 0, pitch }; 3];
        let mut at = 0u64;
        for (p, plane) in planes.iter_mut().enumerate().take(pixels.planes()) {
            let (line, rows) = pixels.plane_extent(p, width, height);
            plane.offset = at;
            plane.pitch = line.next_multiple_of(64);
            let block = match layout {
                Layout::Linear => 64,
                Layout::BlockLinear { log2_height } => 64.max(8 << log2_height),
            };
            at += u64::from(plane.pitch) * u64::from(rows.next_multiple_of(block));
        }
        let buf = TestDmaBuf::new(dev, at as usize).unwrap();
        buf.fill(0x5a).unwrap();
        for plane in &mut planes {
            plane.fd = buf.fd.as_raw_fd();
        }
        Self { buf, planes, layout, pixels, width, height }
    }

    fn picture(&self, key: u64) -> Picture<'_> {
        Picture {
            pixels: self.pixels,
            width: self.width,
            height: self.height,
            memory: Memory::Dma { key, planes: self.planes, layout: self.layout },
        }
    }

    fn read(&self, vpp: &mut Processor, key: u64) -> Cpu {
        let mut out = Cpu::new(self.pixels, self.width, self.height);
        let rect = Rect { x: 0, y: 0, width: self.width, height: self.height };
        vpp.download(&self.picture(key), rect, &mut out.bytes, out.pitch, out.offsets).unwrap();
        let _ = &self.buf;
        out
    }
}

fn open() -> (VideoDevice, Processor) {
    let dev = VideoDevice::open().unwrap();
    let vpp = Processor::open(dev.pci).unwrap();
    (dev, vpp)
}

const FORMATS: [Pixels; 8] = [
    Pixels::Yuv420 { bits: 8 },
    Pixels::Yuv420 { bits: 10 },
    Pixels::Yuv420 { bits: 12 },
    Pixels::Yuv420 { bits: 16 },
    Pixels::Yuv444 { bits: 8 },
    Pixels::Yuv444 { bits: 10 },
    Pixels::Rgb(RgbOrder::Bgra),
    Pixels::Rgb(RgbOrder::Rgba),
];

fn whole(w: u32, h: u32) -> Rect {
    Rect { x: 0, y: 0, width: w, height: h }
}

fn same(pixels: Pixels, w: u32, h: u32) -> Operation {
    let c = colour(if matches!(pixels, Pixels::Rgb(_)) { Matrix::Rgb } else { Matrix::Bt709 }, matches!(pixels, Pixels::Rgb(_)));
    Operation { src_rect: whole(w, h), dst_rect: whole(w, h), src_colour: c, dst_colour: c, background: 0xff00_0000 }
}

/// Same-format copies (the copy-engine path) are byte-exact for every format, from CPU memory and
/// from a dma-buf, into linear and block-linear dma-bufs; read-back of a sub-rectangle too.
#[test]
#[ignore = "local NVIDIA GPU: VideoProc copy engine"]
fn copy_every_format() {
    let (dev, mut vpp) = open();
    let mut key = 0;
    for pixels in FORMATS {
        for (w, h) in [(2u32, 2u32), (70, 38), (1920, 1080)] {
            let src = Cpu::random(pixels, w, h, w + h);
            for layout in [Layout::Linear, Layout::BlockLinear { log2_height: 0 }, Layout::BlockLinear { log2_height: 4 }] {
                let a = Dma::new(&dev, pixels, w, h, layout);
                let b = Dma::new(&dev, pixels, w, h, Layout::BlockLinear { log2_height: 0 });
                key += 2;
                vpp.process(&src.picture(), &a.picture(key), &same(pixels, w, h)).unwrap();
                vpp.process(&a.picture(key), &b.picture(key + 1), &same(pixels, w, h)).unwrap();
                for (name, d, k) in [("a", &a, key), ("b", &b, key + 1)] {
                    let got = d.read(&mut vpp, k);
                    for p in 0..pixels.planes() {
                        for y in 0..pixels.plane_extent(p, w, h).1 {
                            for e in 0..src.elements(p) {
                                for c in 0..src.components(p) {
                                    assert_eq!(got.get(p, e, c, y), src.get(p, e, c, y), "{pixels:?} {w}x{h} {layout:?} {name} plane {p} ({e},{y}) comp {c}");
                                }
                            }
                        }
                    }
                }
                if w >= 70 {
                    let rect = Rect { x: 4, y: 6, width: 32, height: 20 };
                    let mut part = Cpu::new(pixels, 32, 20);
                    vpp.download(&b.picture(key + 1), rect, &mut part.bytes, part.pitch, part.offsets).unwrap();
                    for y in 0..20 {
                        for x in 0..32 {
                            assert_eq!(part.pixel(x, y)[..3], src.pixel(x + 4, y + 6)[..3], "{pixels:?} sub-rect ({x},{y})");
                        }
                    }
                }
                vpp.forget(key);
                vpp.forget(key + 1);
            }
        }
    }
}

/// Reference model of one output sample, in f64, from the definitions in shaders/vpp.comp:
/// bilinear taps per plane clamped to the source region, then R'G'B' through H.273 formulas.
fn reference(src: &Cpu, dst: Pixels, op: &Operation, q: [f64; 2]) -> [f64; 3] {
    let (s, d) = (op.src_rect, op.dst_rect);
    let px = f64::from(s.x) + (q[0] - f64::from(d.x)) * f64::from(s.width) / f64::from(d.width);
    let py = f64::from(s.y) + (q[1] - f64::from(d.y)) * f64::from(s.height) / f64::from(d.height);
    let tap = |u: f64, lo: i32, hi: i32| {
        let i = u.floor();
        ((i as i32).clamp(lo, hi) as u32, (i as i32 + 1).clamp(lo, hi) as u32, u - i)
    };
    let bilinear = |get: &dyn Fn(u32, u32) -> f64, ux: f64, uy: f64, lo: [i32; 2], hi: [i32; 2]| {
        let (x0, x1, fx) = tap(ux, lo[0], hi[0]);
        let (y0, y1, fy) = tap(uy, lo[1], hi[1]);
        let a = get(x0, y0) * (1.0 - fx) + get(x1, y0) * fx;
        let b = get(x0, y1) * (1.0 - fx) + get(x1, y1) * fx;
        a * (1.0 - fy) + b * fy
    };
    let (lo, hi) = ([s.x, s.y], [s.x + s.width as i32 - 1, s.y + s.height as i32 - 1]);
    let codes: [f64; 3] = match src.pixels {
        Pixels::Rgb(_) => std::array::from_fn(|c| bilinear(&|x, y| f64::from(src.pixel(x, y)[c]), px - 0.5, py - 0.5, lo, hi)),
        Pixels::Yuv444 { .. } => std::array::from_fn(|c| bilinear(&|x, y| f64::from(src.pixel(x, y)[c]), px - 0.5, py - 0.5, lo, hi)),
        Pixels::Yuv420 { .. } => {
            let site = op.src_colour.chroma_site.map(f64::from);
            let (clo, chi) = ([lo[0] / 2, lo[1] / 2], [hi[0] / 2, hi[1] / 2]);
            [
                bilinear(&|x, y| f64::from(src.get(0, x, 0, y)), px - 0.5, py - 0.5, lo, hi),
                bilinear(&|x, y| f64::from(src.get(1, x, 0, y)), (px - site[0]) / 2.0, (py - site[1]) / 2.0, clo, chi),
                bilinear(&|x, y| f64::from(src.get(1, x, 1, y)), (px - site[0]) / 2.0, (py - site[1]) / 2.0, clo, chi),
            ]
        }
    };
    let unit = |bits: u32| f64::from(1u32 << (bits - 8));
    let max = |bits: u32| f64::from((1u32 << bits) - 1);
    // Codes -> R'G'B' in 0..1.
    let rgb = match src.pixels {
        Pixels::Rgb(_) => {
            let (o, k) = if op.src_colour.full_range { (0.0, 255.0) } else { (16.0, 219.0) };
            codes.map(|v| (v - o) / k)
        }
        px => {
            let b = px.bits();
            let (y, pb, pr) = if op.src_colour.full_range {
                (codes[0] / max(b), (codes[1] - f64::from(1u32 << (b - 1))) / max(b), (codes[2] - f64::from(1u32 << (b - 1))) / max(b))
            } else {
                ((codes[0] - 16.0 * unit(b)) / (219.0 * unit(b)), (codes[1] - 128.0 * unit(b)) / (224.0 * unit(b)), (codes[2] - 128.0 * unit(b)) / (224.0 * unit(b)))
            };
            let (kr, kb) = op.src_colour.matrix.kr_kb();
            let r = y + 2.0 * (1.0 - kr) * pr;
            let bl = y + 2.0 * (1.0 - kb) * pb;
            [r, (y - kr * r - kb * bl) / (1.0 - kr - kb), bl]
        }
    };
    match dst {
        Pixels::Rgb(_) => {
            let (o, k) = if op.dst_colour.full_range { (0.0, 255.0) } else { (16.0, 219.0) };
            rgb.map(|v| o + k * v)
        }
        px => {
            let b = px.bits();
            let (kr, kb) = op.dst_colour.matrix.kr_kb();
            let y = kr * rgb[0] + (1.0 - kr - kb) * rgb[1] + kb * rgb[2];
            let (pb, pr) = ((rgb[2] - y) / (2.0 * (1.0 - kb)), (rgb[0] - y) / (2.0 * (1.0 - kr)));
            if op.dst_colour.full_range {
                [y * max(b), pb * max(b) + f64::from(1u32 << (b - 1)), pr * max(b) + f64::from(1u32 << (b - 1))]
            } else {
                [16.0 * unit(b) + 219.0 * unit(b) * y, 128.0 * unit(b) + 224.0 * unit(b) * pb, 128.0 * unit(b) + 224.0 * unit(b) * pr]
            }
        }
    }
}

/// Every output sample of `got` against the reference: within 1 code inside the output region
/// (f32 arithmetic on the GPU), exactly the background outside. Returns the samples checked.
fn check(src: &Cpu, got: &Cpu, op: &Operation, what: &str) -> usize {
    let dst = got.pixels;
    let max = f64::from((1u32 << dst.bits()) - 1);
    let bg = background_codes(dst, op);
    let inside = |q: [f64; 2]| {
        let d = op.dst_rect;
        q[0] >= f64::from(d.x) && q[0] < f64::from(d.x) + f64::from(d.width) && q[1] >= f64::from(d.y) && q[1] < f64::from(d.y) + f64::from(d.height)
    };
    let mut n = 0;
    let mut compare = |q: [f64; 2], comps: &[(usize, u32)], at: &str| {
        let want = if inside(q) {
            reference(src, dst, op, q).map(|v| (v + 0.5).floor().clamp(0.0, max))
        } else {
            bg.map(f64::from)
        };
        for &(c, v) in comps {
            let tolerance = if inside(q) { 1.0 } else { 0.0 };
            assert!((f64::from(v) - want[c]).abs() <= tolerance, "{what}: {at} comp {c}: got {v}, want {}", want[c]);
            n += 1;
        }
    };
    for y in 0..got.height {
        for x in 0..got.width {
            let q = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            let v = got.pixel(x, y);
            match dst {
                Pixels::Yuv420 { .. } => compare(q, &[(0, v[0])], &format!("luma ({x},{y})")),
                _ => compare(q, &[(0, v[0]), (1, v[1]), (2, v[2])], &format!("({x},{y})")),
            }
            if let Pixels::Rgb(_) = dst {
                let alpha = if inside(q) { 255 } else { op.background >> 24 };
                assert_eq!(v[3], alpha, "{what}: alpha ({x},{y})");
            }
        }
    }
    if let Pixels::Yuv420 { .. } = dst {
        let site = op.dst_colour.chroma_site.map(f64::from);
        for k in 0..got.height / 2 {
            for j in 0..got.width / 2 {
                let q = [2.0 * f64::from(j) + site[0], 2.0 * f64::from(k) + site[1]];
                compare(q, &[(1, got.get(1, j, 0, k)), (2, got.get(1, j, 1, k))], &format!("chroma ({j},{k})"));
            }
        }
    }
    n
}

/// The kernel against the reference model: every matrix and range, explicit and inherited
/// conversions between YUV and RGB, depth changes, both chroma sitings, crops whose edges cut
/// chroma samples, up- and down-scaling, and output regions with a non-zero origin that leave
/// background (opaque and transparent) around them.
#[test]
#[ignore = "local NVIDIA GPU: VideoProc kernel"]
fn kernel_matches_reference() {
    let (dev, mut vpp) = open();
    let nv12 = Pixels::Yuv420 { bits: 8 };
    let p010 = Pixels::Yuv420 { bits: 10 };
    let p016 = Pixels::Yuv420 { bits: 16 };
    let y444 = Pixels::Yuv444 { bits: 8 };
    let q416 = Pixels::Yuv444 { bits: 10 };
    let bgra = Pixels::Rgb(RgbOrder::Bgra);
    let rgba = Pixels::Rgb(RgbOrder::Rgba);
    let rgb = |full| colour(Matrix::Rgb, full);
    let mut cases: Vec<(Pixels, Colour, Pixels, Colour)> = Vec::new();
    for m in [Matrix::Bt601, Matrix::Bt709, Matrix::Bt2020] {
        for full in [false, true] {
            cases.push((nv12, colour(m, full), bgra, rgb(true)));
            cases.push((bgra, rgb(true), nv12, colour(m, full)));
            cases.push((p010, colour(m, full), rgba, rgb(true)));
            cases.push((nv12, colour(m, full), nv12, colour(Matrix::Bt709, !full)));
        }
    }
    cases.push((p010, colour(Matrix::Bt2020, false), nv12, colour(Matrix::Bt2020, false)));
    cases.push((nv12, colour(Matrix::Bt709, false), p010, colour(Matrix::Bt709, false)));
    cases.push((p016, colour(Matrix::Bt709, true), q416, colour(Matrix::Bt601, false)));
    cases.push((y444, colour(Matrix::Bt709, false), nv12, Colour { chroma_site: CENTRE, ..colour(Matrix::Bt709, false) }));
    cases.push((Pixels::Yuv420 { bits: 8 }, Colour { chroma_site: CENTRE, ..colour(Matrix::Bt709, false) }, y444, colour(Matrix::Bt709, false)));
    cases.push((q416, colour(Matrix::Bt601, false), bgra, rgb(false)));
    cases.push((rgba, rgb(false), bgra, rgb(true)));
    cases.push((nv12, colour(Matrix::Bt709, false), nv12, colour(Matrix::Bt709, false)));

    let geometry = [
        // (src size, src rect, dst size, dst rect, background)
        ((64, 48), whole(64, 48), (64, 48), whole(64, 48), 0xff00_0000),
        ((96, 64), Rect { x: 3, y: 5, width: 61, height: 37 }, (64, 36), whole(64, 36), 0xff00_0000),
        ((40, 30), whole(40, 30), (130, 98), whole(130, 98), 0xff00_0000),
        ((64, 48), whole(64, 48), (100, 80), Rect { x: 10, y: 6, width: 52, height: 40 }, 0x8020_c040),
        ((64, 48), Rect { x: 1, y: 1, width: 31, height: 23 }, (96, 72), Rect { x: 33, y: 9, width: 45, height: 51 }, 0x0000_0000),
    ];
    let mut key = 1000;
    let mut samples = 0;
    for (i, &(sp, sc, dp, dc)) in cases.iter().enumerate() {
        for (g, &((sw, sh), src_rect, (dw, dh), dst_rect, background)) in geometry.iter().enumerate() {
            let src = Cpu::random(sp, sw, sh, (i * 31 + g) as u32);
            let op = Operation { src_rect, dst_rect, src_colour: sc, dst_colour: dc, background };
            let out = Dma::new(&dev, dp, dw, dh, Layout::BlockLinear { log2_height: 0 });
            key += 2;
            // Alternate CPU and dma-buf inputs.
            if g % 2 == 0 {
                vpp.process(&src.picture(), &out.picture(key), &op).unwrap();
            } else {
                let staged = Dma::new(&dev, sp, sw, sh, Layout::BlockLinear { log2_height: 0 });
                vpp.process(&src.picture(), &staged.picture(key + 1), &same(sp, sw, sh)).unwrap();
                vpp.process(&staged.picture(key + 1), &out.picture(key), &op).unwrap();
            }
            let got = out.read(&mut vpp, key);
            samples += check(&src, &got, &op, &format!("case {i} {sp:?}->{dp:?} geometry {g}"));
            vpp.forget(key);
            vpp.forget(key + 1);
        }
    }
    eprintln!("{} cases x {} geometries, {samples} samples within 1 code", cases.len(), geometry.len());
}

/// The kernel path with an identity conversion and unit scale (an output region at the origin of
/// a larger output) reproduces the input exactly.
#[test]
#[ignore = "local NVIDIA GPU: VideoProc kernel identity"]
fn kernel_identity_is_exact() {
    let (dev, mut vpp) = open();
    for (k, pixels) in FORMATS.into_iter().enumerate() {
        let (w, h) = (70u32, 38u32);
        let src = Cpu::random(pixels, w, h, 77 + k as u32);
        let out = Dma::new(&dev, pixels, w + 10, h + 4, Layout::BlockLinear { log2_height: 0 });
        let op = Operation { dst_rect: whole(w, h), ..same(pixels, w, h) };
        vpp.process(&src.picture(), &out.picture(k as u64), &op).unwrap();
        let got = out.read(&mut vpp, k as u64);
        for y in 0..h {
            for x in 0..w {
                let n = 3;
                assert_eq!(got.pixel(x, y)[..n], src.pixel(x, y)[..n], "{pixels:?} ({x},{y})");
            }
        }
    }
}
