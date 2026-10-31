//! Standalone CLI. Only the public shared-library ABI is linked, never the RM rlib.
use std::{
    ffi::{c_char, c_void, CStr, OsString},
    io::{self, Write},
    path::PathBuf,
};
#[repr(C)]
struct DeviceInfo {
    abi_version: u32,
    pci_vendor: u32,
    pci_device: u32,
    chipset: u32,
    nvdec_class: u32,
    nvdec_engines: u32,
    name: [u8; 64],
}
#[repr(C)]
#[derive(Default)]
struct View {
    abi_version: u32,
    nvenc_class: u32,
    decoders: *const c_void,
    decoder_count: u32,
    encoders: *const c_void,
    encoder_count: u32,
}
#[repr(C)]
#[derive(Default)]
struct Bytes {
    data: *const c_char,
    len: usize,
}
#[link(name = "nv_bionic_native")]
unsafe extern "C" {
    fn nv_video_device_open(dev: *mut *mut c_void, info: *mut DeviceInfo) -> i32;
    fn nv_video_device_close(dev: *mut c_void);
    fn nv_video_last_error() -> *const c_char;
    fn nv_c2_caps_query(dev: *mut c_void, out: *mut *mut c_void) -> i32;
    fn nv_c2_caps_view(caps: *const c_void, out: *mut View) -> i32;
    fn nv_c2_media_codecs_xml(caps: *const c_void, out: *mut Bytes) -> i32;
    fn nv_c2_caps_destroy(caps: *mut c_void);
    fn nv_c2_no_decoder_xml(gpu: *const c_char, nvdec_class: u32, out: *mut *mut c_void) -> i32;
    fn nv_c2_no_decoder_reason(gpu: *const c_char, nvdec_class: u32, out: *mut *mut c_void) -> i32;
    fn nv_c2_text_view(text: *const c_void, out: *mut Bytes) -> i32;
    fn nv_c2_text_destroy(text: *mut c_void);
}
#[derive(Default)]
struct Options {
    output: Option<PathBuf>,
    check: Option<PathBuf>,
}
fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Options, ()> {
    let mut args = args.into_iter();
    let mut o = Options::default();
    while let Some(arg) = args.next() {
        if arg == "-o" {
            o.output = Some(args.next().ok_or(())?.into());
        } else if arg == "--check" {
            o.check = Some(args.next().ok_or(())?.into());
        } else {
            return Err(());
        }
    }
    Ok(o)
}
fn emit(
    o: &Options,
    xml: &[u8],
    gpu: &str,
    status: i32,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    if let Some(path) = &o.check {
        match std::fs::read(path) {
            Err(_) => {
                let _ = writeln!(stderr, "nvc2-xmlgen: cannot read {}", path.display());
                return 1;
            }
            Ok(got) if got != xml => {
                let _=writeln!(stderr,"nvc2-xmlgen: {} does not match the live caps of {gpu}; regenerate it. Expected:",path.display());
                let _ = stderr.write_all(xml);
                return 1;
            }
            _ => {}
        }
    } else if let Some(path) = &o.output {
        if std::fs::write(path, xml).is_err() {
            let _ = writeln!(stderr, "nvc2-xmlgen: cannot write {}", path.display());
            return 1;
        }
    } else if stdout.write_all(xml).and_then(|_| stdout.flush()).is_err() {
        let _ = writeln!(stderr, "nvc2-xmlgen: cannot write stdout");
        return 1;
    }
    status
}
#[derive(Default)]
struct Handles {
    dev: *mut c_void,
    caps: *mut c_void,
    text: *mut c_void,
    reason: *mut c_void,
}
impl Drop for Handles {
    fn drop(&mut self) {
        unsafe {
            nv_c2_text_destroy(self.reason);
            nv_c2_text_destroy(self.text);
            nv_c2_caps_destroy(self.caps);
            nv_video_device_close(self.dev);
        }
    }
}
fn error() -> String {
    unsafe { CStr::from_ptr(nv_video_last_error()) }
        .to_string_lossy()
        .into_owned()
}
fn generate(o: &Options) -> Result<i32, String> {
    let mut h = Handles::default();
    let mut info: DeviceInfo = unsafe { std::mem::zeroed() };
    let rc = unsafe { nv_video_device_open(&mut h.dev, &mut info) };
    if rc != 0 {
        return Err(format!("nv_video_device_open failed ({rc}): {}", error()));
    }
    if info.abi_version != 2 {
        return Err(format!("backend ABI {}, built for 2", info.abi_version));
    }
    let gpu = unsafe { CStr::from_ptr(info.name.as_ptr().cast()) }.to_string_lossy();
    let mut xml = Bytes::default();
    let mut view = View::default();
    if info.nvdec_class != 0 {
        if unsafe { nv_c2_caps_query(h.dev, &mut h.caps) } != 0 {
            return Err(error());
        }
        if unsafe { nv_c2_caps_view(h.caps, &mut view) } != 0 {
            return Err(error());
        }
        if view.abi_version != 2 {
            return Err(format!("caps ABI {}, built for 2", view.abi_version));
        }
    }
    let status = if view.decoder_count == 0 {
        if unsafe { nv_c2_no_decoder_reason(info.name.as_ptr().cast(), info.nvdec_class, &mut h.reason) } != 0 {
            return Err(error());
        }
        let mut reason = Bytes::default();
        if unsafe { nv_c2_text_view(h.reason, &mut reason) } != 0 {
            return Err(error());
        }
        eprintln!(
            "nvc2-xmlgen: {}",
            unsafe { CStr::from_ptr(reason.data) }.to_string_lossy()
        );
        if unsafe { nv_c2_no_decoder_xml(info.name.as_ptr().cast(), info.nvdec_class, &mut h.text) } != 0 {
            return Err(error());
        }
        if unsafe { nv_c2_text_view(h.text, &mut xml) } != 0 {
            return Err(error());
        }
        3
    } else {
        if unsafe { nv_c2_media_codecs_xml(h.caps, &mut xml) } != 0 {
            return Err(error());
        }
        0
    };
    let bytes = unsafe { std::slice::from_raw_parts(xml.data.cast(), xml.len) };
    Ok(emit(
        o,
        bytes,
        &gpu,
        status,
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    ))
}
fn main() {
    let status = match parse(std::env::args_os().skip(1)) {
        Err(()) => {
            eprintln!("usage: nvc2-xmlgen [-o FILE] | --check FILE");
            2
        }
        Ok(o) => match generate(&o) {
            Ok(status) => status,
            Err(e) => {
                eprintln!("nvc2-xmlgen: {e}");
                1
            }
        },
    };
    std::process::exit(status);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn option_contract() {
        for args in [vec!["-o"], vec!["--check"], vec!["--bad"], vec!["file"]] {
            assert!(parse(args.into_iter().map(Into::into)).is_err());
        }
        let o = parse(
            [
                "-o", "one", "--check", "first", "-o", "two", "--check", "last",
            ]
            .map(Into::into),
        )
        .unwrap();
        assert_eq!(o.output, Some("two".into()));
        assert_eq!(o.check, Some("last".into()));
    }
    #[test]
    fn byte_io_and_status_contract() {
        let path = std::env::temp_dir().join(format!("nvc2-cli-test-{}", std::process::id()));
        let mut out = Vec::new();
        let mut err = Vec::new();
        let xml = b"<xml/>\n";
        for status in [0, 3] {
            assert_eq!(
                emit(&Options::default(), xml, "stub", status, &mut out, &mut err),
                status
            );
            assert!(out.ends_with(xml));
            assert_eq!(
                emit(
                    &Options {
                        output: Some(path.clone()),
                        check: None
                    },
                    xml,
                    "stub",
                    status,
                    &mut out,
                    &mut err
                ),
                status
            );
            let len = out.len();
            assert_eq!(
                emit(
                    &Options {
                        output: Some("/nonexistent/output".into()),
                        check: Some(path.clone())
                    },
                    xml,
                    "stub",
                    status,
                    &mut out,
                    &mut err
                ),
                status
            );
            assert_eq!(len, out.len());
        }
        assert_eq!(std::fs::read(&path).unwrap(), xml);
        assert_eq!(
            emit(
                &Options {
                    output: None,
                    check: Some(path.clone())
                },
                b"stale",
                "stub",
                0,
                &mut out,
                &mut err
            ),
            1
        );
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            emit(
                &Options {
                    output: None,
                    check: Some(path)
                },
                xml,
                "stub",
                3,
                &mut out,
                &mut err
            ),
            1
        );
        assert_eq!(
            emit(
                &Options {
                    output: Some("/nonexistent/output".into()),
                    check: None
                },
                xml,
                "stub",
                0,
                &mut out,
                &mut err
            ),
            1
        );
    }
}
