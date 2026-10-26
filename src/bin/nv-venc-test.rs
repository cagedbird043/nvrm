//! Host raw-input encode tool; base options/output match media/test/nvenc_test.c.
//! Extra options exercise the complete EncConfig on the same C/Rust comparison harness.
use nv_bionic_native::video::enc::{EncConfig, EncFrame, Encoder, ENC_AV1, ENC_HEVC};
use std::{
    collections::VecDeque,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};
fn value(args: &[String], i: &mut usize) -> Result<i32, String> {
    *i += 1;
    args.get(*i)
        .ok_or("missing option value")?
        .parse()
        .map_err(|_| "invalid integer".into())
}
fn hex<const N: usize>(s: &str) -> Result<[u8; N], String> {
    if s.len() != 2 * N {
        return Err("wrong metadata hex length".into());
    }
    let mut out = [0; N];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| "invalid metadata hex")?;
    }
    Ok(out)
}
fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let mut c = EncConfig {
        qp_i: 26,
        qp_p: 28,
        num_refs: 1,
        fps: 30,
        h264_profile: 100,
        cabac: true,
        ..Default::default()
    };
    let mut i = 1;
    let mut pci = None;
    while i < a.len() && a[i].starts_with('-') {
        match a[i].as_str() {
            "-hevc" => c.codec = ENC_HEVC,
            "-av1" => {
                c.codec = ENC_AV1;
                c.qp_i = 90;
                c.qp_p = 110;
            }
            "-cbr" => {
                c.rc_mode = 2;
                c.bitrate = value(&a, &mut i)? * 1000;
            }
            "-vbr" => {
                c.rc_mode = 1;
                c.bitrate = value(&a, &mut i)? * 1000;
            }
            "-qp" => {
                c.qp_i = value(&a, &mut i)?;
                c.qp_p = c.qp_i;
            }
            "-gop" => c.idr_period = value(&a, &mut i)?,
            "-fps" => c.fps = value(&a, &mut i)?,
            "-profile" => c.h264_profile = value(&a, &mut i)?,
            "-level" => c.h264_level = value(&a, &mut i)?,
            "-bf" => c.bframes = value(&a, &mut i)?,
            "-refs" => c.num_refs = value(&a, &mut i)?,
            "-depth" => c.depth = value(&a, &mut i)?,
            "-slices" => c.slices = value(&a, &mut i)?,
            "-aq" => c.aq = value(&a, &mut i)?,
            "-engine" => c.engine = value(&a, &mut i)?,
            "-vbv" => c.vbv_bits = value(&a, &mut i)? * 1000,
            "-maxrate" => c.max_bitrate = value(&a, &mut i)? * 1000,
            "-10" => c.bit_depth = 10,
            "-444" => c.chroma444 = true,
            "-quality" => c.quality = true,
            "-two-pass" => c.two_pass = true,
            "-lossless" => c.lossless = true,
            "-no-cu8" => c.no_cu8 = true,
            "-full-range" => c.full_range = true,
            "-v" => c.verbose = true,
            "-colour" => {
                for v in &mut c.colour {
                    *v = value(&a, &mut i)?;
                }
            }
            "-mdcv" => {
                i += 1;
                c.mdcv = hex(a.get(i).ok_or("missing MDCV")?)?;
                c.has_mdcv = true;
            }
            "-cll" => {
                i += 1;
                c.cll = hex(a.get(i).ok_or("missing CLL")?)?;
                c.has_cll = true;
            }
            "-pci" => {
                i += 1;
                let p = a
                    .get(i)
                    .ok_or("missing PCI address")?
                    .replace([':', '.'], " ");
                let p: Vec<_> = p
                    .split_whitespace()
                    .map(|x| u32::from_str_radix(x, 16))
                    .collect::<Result<_, _>>()?;
                pci = Some(
                    p.try_into()
                        .map_err(|_| "PCI address must be domain:bus:device.function")?,
                );
            }
            x => {
                eprintln!("unknown option {x}");
                return Ok(2);
            }
        }
        i += 1;
    }
    if a.len() - i != 4 {
        eprintln!("usage: nvenc_test [-hevc|-av1] [-cbr KBPS|-vbr KBPS] [-qp QP] [-gop N] [-fps F] [-profile 66|77|100] [-level IDC] W H in.nv12 out");
        return Ok(2);
    }
    c.width = a[i].parse()?;
    c.height = a[i + 1].parse()?;
    let (w, h) = (c.width as usize, c.height as usize);
    let samples = if c.chroma444 {
        3 * w * h
    } else {
        w * h + w * h.div_ceil(2)
    };
    let frame_bytes = samples
        * if c.bit_depth == 10 && !c.chroma444 {
            2
        } else {
            1
        };
    let mut input = File::open(&a[i + 2])?;
    let mut output = File::create(&a[i + 3])?;
    let count = input.metadata()?.len() as usize / frame_bytes;
    let mut buf = vec![0; frame_bytes];
    let mut e = Encoder::open_on(&c, pci).map_err(|e| e.to_string())?;
    let mut hdr = Vec::new();
    e.headers(&mut hdr);
    output.write_all(&hdr)?;
    let mut stream = Vec::with_capacity(w * h * 2 + 4096);
    let mut pending = VecDeque::with_capacity(e.cfg.depth as usize);
    let (mut n, mut failed, mut total, mut ms) = (0, 0, hdr.len(), 0.0);
    let mut order = Vec::with_capacity(count);
    let mut pos = 0;
    while pos < count {
        let end = if c.idr_period > 0 {
            (pos + c.idr_period as usize).min(count)
        } else {
            count
        };
        order.push((pos, false, false, true));
        let mut anchor = pos;
        while anchor + 1 < end {
            let next = (anchor + e.cfg.bframes.max(0) as usize + 1).min(end - 1);
            order.push((next, false, false, false));
            for b in anchor + 1..next {
                order.push((b, true, b + 1 == next, false));
            }
            anchor = next;
        }
        pos = end;
    }
    for (disp, b, show, force) in order {
        if pending.len() == e.cfg.depth as usize {
            let (t, s) = pending.pop_front().unwrap();
            finish(
                &mut e,
                t,
                s,
                &mut stream,
                &hdr,
                &mut output,
                &c,
                &mut n,
                &mut failed,
                &mut total,
                &mut ms,
            )?;
        }
        input.seek(SeekFrom::Start((disp * frame_bytes) as u64))?;
        input.read_exact(&mut buf)?;
        if c.chroma444 {
            e.load_yuv444(
                &buf[..w * h],
                w,
                &buf[w * h..2 * w * h],
                w,
                &buf[2 * w * h..],
                w,
            )
            .map_err(|e| e.to_string())?;
        } else if c.bit_depth == 10 {
            e.load_p010(&buf[..2 * w * h], 2 * w, &buf[2 * w * h..], 2 * w)
                .map_err(|e| e.to_string())?;
        } else {
            e.load_nv12(&buf[..w * h], w, &buf[w * h..], w)
                .map_err(|e| e.to_string())?;
        }
        pending.push_back((
            e.submit_ex(force, b, disp as i32)
                .map_err(|e| e.to_string())?,
            show,
        ));
    }
    while let Some((t, s)) = pending.pop_front() {
        finish(
            &mut e,
            t,
            s,
            &mut stream,
            &hdr,
            &mut output,
            &c,
            &mut n,
            &mut failed,
            &mut total,
            &mut ms,
        )?;
    }
    println!(
        "{n} frames ({failed} failed), {total} bytes, mean engine time {:.3} ms",
        if n > failed {
            ms / f64::from(n - failed)
        } else {
            0.0
        }
    );
    Ok(i32::from(failed != 0))
}
#[allow(clippy::too_many_arguments)]
fn finish(
    e: &mut Encoder,
    t: u32,
    show: bool,
    stream: &mut Vec<u8>,
    hdr: &[u8],
    output: &mut File,
    c: &EncConfig,
    n: &mut i32,
    failed: &mut i32,
    total: &mut usize,
    ms: &mut f64,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.clear();
    let fr: EncFrame = match e.wait(t, stream) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("frame {} failed ({err})", *n);
            *failed += 1;
            *n += 1;
            return Ok(());
        }
    };
    if fr.idr && *n > 0 && !hdr.is_empty() {
        output.write_all(hdr)?;
        *total += hdr.len();
    }
    output.write_all(stream)?;
    *total += fr.size;
    *ms += fr.ms;
    if c.verbose || *n < 3 {
        println!(
            "frame {}: {} {} bytes, {:.3} ms, avgQP {}",
            *n,
            if fr.idr { "IDR" } else { "P" },
            fr.size,
            fr.ms,
            fr.avg_qp
        );
    }
    *n += 1;
    if show && c.codec == ENC_AV1 {
        stream.clear();
        e.av1_show_existing(stream);
        output.write_all(stream)?;
        *total += stream.len();
    }
    Ok(())
}
fn main() {
    match run() {
        Ok(c) => std::process::exit(c),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
