//! Serialized synchronous Codec2 encoder session ABI (video/nv_venc.h).
use super::{enc::{EncConfig, EncError, Encoder}, VideoError};
use std::{mem::size_of, panic::{catch_unwind, AssertUnwindSafe}, ptr};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NvVencSessCfg {
    pub abi_version: u32, pub struct_size: u32,
    pub pci_present: u32, pub pci_domain: u32, pub pci_bus: u32, pub pci_device: u32, pub pci_function: u32,
    pub codec: u32, pub width: u32, pub height: u32,
    pub h264_profile: u32, pub h264_level: u32, pub cabac: u32,
    pub rc_mode: u32, pub bitrate: u32, pub max_bitrate: u32, pub vbv_bits: u32,
    pub fps: u32, pub idr_period: u32, pub full_range: u32,
    pub primaries: u32, pub transfer: u32, pub matrix: u32, pub reserved: [u32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NvVencSessUpdate {
    pub bitrate: u32, pub max_bitrate: u32, pub vbv_bits: u32, pub fps: u32, pub idr_period: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvVencPlane { pub data: *const u8, pub length: u64, pub stride: u64 }
impl Default for NvVencPlane {
    fn default() -> Self { Self { data: ptr::null(), length: 0, stride: 0 } }
}
#[repr(C)]
#[derive(Default)]
pub struct NvVencOut {
    pub data: *const u8, pub length: u64, pub engine_ns: u64,
    pub key: u32, pub avg_qp: u32, pub bit_count: u32, pub frame_num: u32, pub poc: u32, pub reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NvVencC2Plane {
    pub data: *const u8, pub row_inc: i32, pub col_inc: i32,
    pub col_sampling: u32, pub row_sampling: u32, pub allocated_depth: u32,
    pub bit_depth: u32, pub right_shift: u32, pub root_index: u32, pub root_offset: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NvVencC2View {
    pub type_: u32, pub num_planes: u32, pub root_planes: u32, pub width: u32, pub height: u32,
    pub crop_left: u32, pub crop_top: u32, pub crop_width: u32, pub crop_height: u32,
    pub planes: [NvVencC2Plane; 4],
}
struct NvVencSess { encoder: Encoder, poisoned: bool, config: Vec<u8> }
// SAFETY: NvVencSess constructs its own NVENC device through Encoder::open_on and
// exclusively owns the complete channel, RM Rc/RefCell/Cell graph, CPU/GPU mappings
// and output storage. No reference-counted owner, Rust reference or mutable view
// escapes this wrapper; the only exported byte view is read-only and expires before
// any subsequent operation. All session calls and output reads are externally
// serialized, with a happens-before edge at thread handoff; destruction on another
// thread occurs only after the encode worker has joined. RM/file/mmap resources have
// no thread-affine API requirement in these paths. Moving the whole owner therefore
// cannot race an Rc count, RefCell/Cell access, mapping access or destructor. The
// wrapper is not Sync and must never be shared concurrently.
unsafe impl Send for NvVencSess {}
const _: fn() = || { fn send<T: Send>() {} send::<NvVencSess>(); };
const _: () = {
    assert!(size_of::<NvVencSessCfg>() == 108);
    assert!(size_of::<NvVencSessUpdate>() == 20);
    assert!(std::mem::align_of::<NvVencSessCfg>() == 4);
    assert!(std::mem::align_of::<NvVencSessUpdate>() == 4);
    assert!(std::mem::offset_of!(NvVencSessCfg,reserved) == 92);
    if size_of::<usize>() == 8 {
        assert!(size_of::<NvVencPlane>() == 24 && std::mem::align_of::<NvVencPlane>() == 8);
        assert!(size_of::<NvVencOut>() == 48 && std::mem::align_of::<NvVencOut>() == 8);
        assert!(size_of::<NvVencC2Plane>() == 48 && std::mem::align_of::<NvVencC2Plane>() == 8);
        assert!(size_of::<NvVencC2View>() == 232 && std::mem::align_of::<NvVencC2View>() == 8);
        assert!(std::mem::offset_of!(NvVencOut, key) == 24);
        assert!(std::mem::offset_of!(NvVencC2View, planes) == 40);
    } else {
        let a=std::mem::align_of::<u64>();
        assert!(size_of::<NvVencPlane>() == if a==8 {24}else{20});
        assert!(size_of::<NvVencOut>() == if a==8 {48}else{44});
        assert!(size_of::<NvVencC2Plane>() == 40);
        assert!(size_of::<NvVencC2View>() == 196);
        assert!(std::mem::offset_of!(NvVencOut,key) == if a==8 {24}else{20});
        assert!(std::mem::offset_of!(NvVencC2View,planes) == 36);
    }
};
fn invalid(field: &'static str) -> EncError { EncError::InvalidUpdate { field } }
fn report(e: EncError) -> i32 {
    let status = if e.recoverable() { -1 } else { -2 };
    super::ffi::fail(VideoError::Invalid(e.to_string()));
    status
}
fn pure(f: impl FnOnce() -> Result<(), EncError>) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0, Ok(Err(e)) => report(e), Err(_) => report(EncError::Panic),
    }
}
fn rate_valid(rc: u32, u: &NvVencSessUpdate) -> Result<(), EncError> {
    if !(1..=65535).contains(&u.fps) { return Err(invalid("fps")); }
    if u.idr_period > 134217728 { return Err(invalid("idr_period")); }
    if rc != 0 && (!(1..=200000000).contains(&u.bitrate)
        || !(u.bitrate..=200000000).contains(&u.max_bitrate)
        || !(1..=200000000).contains(&u.vbv_bits)) { return Err(invalid("rate")); }
    Ok(())
}
fn config(c: &NvVencSessCfg) -> Result<EncConfig, EncError> {
    if c.abi_version != 1 || c.struct_size as usize != size_of::<NvVencSessCfg>()
        || c.reserved != [0;4] { return Err(invalid("ABI")); }
    if c.codec > 2 || c.rc_mode > 2 || c.pci_present > 1 || c.cabac > 1 || c.full_range > 1
        || c.primaries > 255 || c.transfer > 255 || c.matrix > 255 { return Err(invalid("enum")); }
    if c.pci_domain > 65535 || c.pci_bus > 255 || c.pci_device > 31 || c.pci_function > 7 {
        return Err(invalid("PCI BDF"));
    }
    let (minw,minh,max)=match c.codec {0=>(50,2,4096),1=>(66,2,8192),2=>(66,66,8192),_=>unreachable!()};
    if c.width < minw || c.height < minh || c.width > max || c.height > max
        || c.width % 2 != 0 || c.height % 2 != 0 { return Err(invalid("dimensions")); }
    if c.codec == 0 {
        if ![66,77,100].contains(&c.h264_profile) || (c.h264_profile == 66 && c.cabac != 0)
            || ![0,10,11,12,13,20,21,22,30,31,32,40,41,42,50,51,52,60,61,62].contains(&c.h264_level) {
            return Err(invalid("H264 profile/level"));
        }
    } else if c.h264_profile != 0 || c.h264_level != 0 || c.cabac != 0 { return Err(invalid("H264 fields")); }
    rate_valid(c.rc_mode, &NvVencSessUpdate { bitrate:c.bitrate,max_bitrate:c.max_bitrate,
        vbv_bits:c.vbv_bits,fps:c.fps,idr_period:c.idr_period })?;
    Ok(EncConfig { codec: match c.codec { 0 => 0, 1 => 1, 2 => 2, _ => unreachable!() },
        width:c.width as i32,height:c.height as i32, num_refs:1,depth:1,bit_depth:8,
        h264_profile:c.h264_profile as i32,h264_level:c.h264_level as i32,cabac:c.cabac!=0,
        rc_mode:c.rc_mode as i32,bitrate:c.bitrate as i32,max_bitrate:c.max_bitrate as i32,
        vbv_bits:c.vbv_bits as i32,fps:c.fps as i32,idr_period:c.idr_period as i32,
        qp_i:if c.codec==2 {90} else {26},qp_p:if c.codec==2 {110} else {28},
        full_range:c.full_range!=0,colour:[c.primaries as i32,c.transfer as i32,c.matrix as i32],
        ..Default::default() })
}
unsafe fn session_call(s: *mut NvVencSess, f: impl FnOnce(&mut NvVencSess) -> Result<(), EncError>) -> i32 {
    if s.is_null() { return report(EncError::InternalInvariant { reason:"null session" }); }
    let s = unsafe { &mut *s };
    if s.poisoned { return -2; }
    match catch_unwind(AssertUnwindSafe(|| f(s))) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => { if !e.recoverable() { s.poisoned=true; } report(e) },
        Err(_) => { s.poisoned=true; report(EncError::Panic) },
    }
}
#[no_mangle]
unsafe extern "C" fn nv_venc_sess_create(c: *const NvVencSessCfg, out: *mut *mut NvVencSess) -> i32 {
    if out.is_null() { return report(EncError::InternalInvariant { reason:"null output" }); }
    unsafe { *out = ptr::null_mut(); }
    let status = pure(|| {
        let c = unsafe { c.as_ref() }.ok_or(invalid("null config"))?;
        let cfg=config(c)?;
        let encoder=Encoder::open_on(&cfg, if c.pci_present!=0 {
            Some([c.pci_domain,c.pci_bus,c.pci_device,c.pci_function]) } else {None})
            .map_err(|source| EncError::DeviceOpen {source})?;
        let mut config=Vec::new();
        config.try_reserve_exact(65536).map_err(|_| EncError::OutputAssemblyCapacity)?;
        if c.codec==2 { encoder.av1_config(&mut config); } else { encoder.headers(&mut config); }
        unsafe { *out=Box::into_raw(Box::new(NvVencSess {encoder,poisoned:false,config})); }
        Ok(())
    });
    if status!=0 {-2} else {0}
}
#[no_mangle]
unsafe extern "C" fn nv_venc_sess_update(s:*mut NvVencSess,u:*const NvVencSessUpdate)->i32 {
    unsafe { session_call(s, |s| {
        let u=u.as_ref().ok_or(invalid("null update"))?;
        rate_valid(s.encoder.cfg.rc_mode as u32,u)?;
        let c=&mut s.encoder.cfg;
        c.bitrate=u.bitrate as i32;c.max_bitrate=u.max_bitrate as i32;c.vbv_bits=u.vbv_bits as i32;
        c.fps=u.fps as i32;c.idr_period=u.idr_period as i32;
        Ok(())
    }) }
}
fn extent(rows:usize,stride:usize,row:usize)->Option<usize> {
    if rows==0 || stride<row {return None;}
    (rows-1).checked_mul(stride)?.checked_add(row).filter(|&n|n<=isize::MAX as usize)
}
unsafe fn plane<'a>(p:&NvVencPlane,rows:usize,row:usize,i:usize)->Result<&'a [u8],EncError> {
    let bad=||EncError::InvalidInputPlane {plane:i,reason:"null, short, overflowing or invalid stride"};
    let stride=usize::try_from(p.stride).map_err(|_|bad())?;
    let n=extent(rows,stride,row).ok_or_else(bad)?;
    if p.data.is_null() || p.length<n as u64 || p.length>isize::MAX as u64
        || (p.data as usize).checked_add(n).is_none() {return Err(bad());}
    Ok(unsafe {std::slice::from_raw_parts(p.data,n)})
}
fn unused(p:&NvVencPlane)->bool {p.data.is_null() && p.length==0 && p.stride==0}
#[no_mangle]
unsafe extern "C" fn nv_venc_sess_encode(s:*mut NvVencSess,p:*const NvVencPlane,format:u32,force:u32,out:*mut NvVencOut)->i32 {
    if out.is_null() {return report(EncError::InternalInvariant {reason:"null output"});}
    unsafe {*out=NvVencOut::default();}
    unsafe { session_call(s,|s| {
        let result=(|| {
            if p.is_null() || format>4 || force>1 {return Err(invalid("encode arguments"));}
            let p=std::slice::from_raw_parts(p,3);
            let (w,h)=(s.encoder.cfg.width as usize,s.encoder.cfg.height as usize);
            if (format==0 || format==1) && !unused(&p[2])
                || format>=3 && (!unused(&p[1]) || !unused(&p[2])) {return Err(invalid("unused plane"));}
            if format>=3 && p[0].stride<(w*4) as u64 {return Err(invalid("RGB row stride"));}
            let a=plane(&p[0],h,if format>=3 {(w-1)*4+3} else {w},0)?;
            let b=if format<3 {plane(&p[1],h/2,if format==2 {w/2}else{w},1)?}else{&[]};
            let c=if format==2 {plane(&p[2],h/2,w/2,2)?}else{&[]};
            s.encoder.load_c2(format,[a,b,c],[p[0].stride as usize,p[1].stride as usize,p[2].stride as usize])?;
            let ticket=s.encoder.submit_checked(force!=0,false,-1)?;
            let v=s.encoder.wait_view(ticket)?;
            *out=NvVencOut {data:v.data.as_ptr(),length:v.data.len() as u64,
                engine_ns:(v.frame.ms*1000000.0) as u64,key:v.frame.idr as u32,avg_qp:v.frame.avg_qp,
                bit_count:v.frame.bits,frame_num:v.frame.frame_num,poc:v.frame.poc,reserved:0};
            Ok(())
        })();
        if result.as_ref().is_err_and(|e|e.recoverable()) {s.encoder.recover_picture();}
        result
    }) }
}
#[no_mangle]
unsafe extern "C" fn nv_venc_sess_codec_config(s:*mut NvVencSess,b:*mut u8,cap:u64,len:*mut u64)->i32 {
    if len.is_null() {return report(EncError::InternalInvariant {reason:"null config length"});}
    unsafe {*len=0;session_call(s,|s| {
        if b.is_null() && cap!=0 {return Err(invalid("null config buffer"));}
        *len=s.config.len() as u64;
        if b.is_null() && cap==0 {return Ok(());}
        if cap<*len {return Err(invalid("short config buffer"));}
        ptr::copy_nonoverlapping(s.config.as_ptr(),b,s.config.len());Ok(())
    })}
}
#[no_mangle]
unsafe extern "C" fn nv_venc_sess_destroy(s:*mut NvVencSess) {
    let _=pure(|| {if !s.is_null() {unsafe {drop(Box::from_raw(s));}} Ok(())});
}
#[no_mangle]
unsafe extern "C" fn nv_venc_c2_timing(bits:u64,interval:i64,fps:*mut u32,period:*mut u32)->i32 {
    pure(|| {
        let raw=f64::from_bits(bits);let rounded=raw.round().max(1.0);
        if !raw.is_finite() || raw<=0.0 || rounded>65535.0 || fps.is_null() || period.is_null() {return Err(invalid("timing"));}
        let p=if interval<0 || interval==i64::MAX {0}else{(interval as f64*raw/1000000.0).round().clamp(1.0,134217728.0) as u32};
        unsafe {*fps=rounded as u32;*period=p;}Ok(())
    })
}
#[no_mangle]
unsafe extern "C" fn nv_venc_c2_output_size(frame:u64,prepend:u64,total:*mut u32)->i32 {
    pure(|| {let n=frame.checked_add(prepend).and_then(|n|u32::try_from(n).ok()).ok_or(invalid("output size"))?;
        if total.is_null() {return Err(invalid("null output size"));}unsafe {*total=n;}Ok(())})
}

fn input(v:&NvVencC2View,w:u32,h:u32)->Result<(u32,[NvVencPlane;3]),EncError> {
    if w==0 || h==0 || w%2!=0 || h%2!=0 || v.crop_left!=0 || v.crop_top!=0
        || v.crop_width<w || v.crop_height<h || v.crop_width>v.width || v.crop_height>v.height
        || !(1..=4).contains(&v.num_planes) || !(1..=v.num_planes).contains(&v.root_planes) {
        return Err(invalid("view dimensions/crop/planes"));
    }
    let ps=&v.planes[..v.num_planes as usize];
    let mut roots=[None;4];
    for p in ps {
        if p.data.is_null() || p.row_inc<=0 || p.col_inc<=0 || p.root_index>=v.num_planes
            || p.allocated_depth!=8 || p.bit_depth!=8 || p.right_shift!=0 {
            return Err(invalid("plane descriptor"));
        }
        let root=(p.data as usize).checked_sub(p.root_offset as usize).ok_or(invalid("root offset"))?;
        let slot=&mut roots[p.root_index as usize];
        if slot.is_some_and(|old|old!=root) {return Err(invalid("root relation"));}
        *slot=Some(root);
        let root_plane=&ps[p.root_index as usize];
        if root_plane.root_offset!=0 || root_plane.root_index!=p.root_index
            || root_plane.data as usize!=root {return Err(invalid("root descriptor"));}
    }
    if roots.iter().filter(|r|r.is_some()).count()!=v.root_planes as usize {
        return Err(invalid("root plane count"));
    }
    let make=|p:&NvVencC2Plane,rows:u32,row:u32|->Result<NvVencPlane,EncError> {
        let n=extent(rows as usize,p.row_inc as usize,row as usize).ok_or(invalid("plane extent"))?;
        if (p.data as usize).checked_add(n).is_none() {return Err(invalid("plane address overflow"));}
        Ok(NvVencPlane {data:p.data,length:n as u64,stride:p.row_inc as u64})
    };
    let mut out=[NvVencPlane::default();3];
    if v.type_==0 {
        if ps.len()!=3 {return Err(invalid("YUV plane count"));}
        let (y,u,b)=(&ps[0],&ps[1],&ps[2]);
        if y.col_sampling!=1 || y.row_sampling!=1 || y.col_inc!=1
            || u.col_sampling!=2 || u.row_sampling!=2 || b.col_sampling!=2 || b.row_sampling!=2 {
            return Err(invalid("YUV sampling"));
        }
        out[0]=make(y,h,w)?;
        if u.col_inc==1 && b.col_inc==1 {
            out[1]=make(u,h/2,w/2)?;out[2]=make(b,h/2,w/2)?;
            return Ok((2,out));
        }
        if u.col_inc!=2 || b.col_inc!=2 || u.row_inc!=b.row_inc || u.root_index!=b.root_index {
            return Err(invalid("interleaved chroma"));
        }
        let ua=u.data as usize;let va=b.data as usize;
        if ua.checked_add(1)==Some(va) {out[1]=make(u,h/2,w)?;Ok((0,out))}
        else if va.checked_add(1)==Some(ua) {out[1]=make(b,h/2,w)?;Ok((1,out))}
        else {Err(invalid("chroma adjacency"))}
    } else if v.type_==1 || v.type_==2 {
        if ps.len()!=if v.type_==2 {4}else{3} {return Err(invalid("RGB plane count"));}
        let (r,g,b)=(&ps[0],&ps[1],&ps[2]);
        for p in ps {
            if p.col_sampling!=1 || p.row_sampling!=1 || p.col_inc!=4
                || p.row_inc!=r.row_inc || p.root_index!=r.root_index {return Err(invalid("RGB descriptor"));}
        }
        let (ra,ga,ba)=(r.data as usize,g.data as usize,b.data as usize);
        let (format,base)=if ra.checked_add(1)==Some(ga) && ga.checked_add(1)==Some(ba) {(3,r)}
            else if ba.checked_add(1)==Some(ga) && ga.checked_add(1)==Some(ra) {(4,b)}
            else {return Err(invalid("RGB adjacency"));};
        if ps.len()==4 && (base.data as usize).checked_add(3)!=Some(ps[3].data as usize) {
            return Err(invalid("alpha adjacency"));
        }
        // X is ignored; require the full row stride but only the last RGB sample extent.
        if u64::from(r.row_inc as u32)<u64::from(w)*4 {return Err(invalid("RGB stride"));}
        out[0]=make(base,h,w.checked_mul(4).and_then(|n|n.checked_sub(1)).ok_or(invalid("RGB width"))?)?;
        Ok((format,out))
    } else {Err(invalid("layout type"))}
}
#[no_mangle]
unsafe extern "C" fn nv_venc_c2_input(v:*const NvVencC2View,w:u32,h:u32,format:*mut u32,out:*mut NvVencPlane)->i32 {
    pure(|| {
        if v.is_null() || format.is_null() || out.is_null() {return Err(invalid("null view/output"));}
        let (f,p)=input(unsafe {&*v},w,h)?;
        unsafe {*format=f;ptr::copy_nonoverlapping(p.as_ptr(),out,3);}Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // Ambiguous only if the private owner accidentally becomes Sync.
    trait AmbiguousIfSync<A> { fn check() {} }
    impl<T:?Sized> AmbiguousIfSync<()> for T {}
    impl<T:?Sized+Sync> AmbiguousIfSync<u8> for T {}
    const _: fn()=|| {let _=<NvVencSess as AmbiguousIfSync<_>>::check;};

    fn cfg(codec:u32)->NvVencSessCfg {
        NvVencSessCfg {abi_version:1,struct_size:size_of::<NvVencSessCfg>() as u32,
            codec,width:128,height:96,fps:30,h264_profile:if codec==0 {100}else{0},
            cabac:if codec==0 {1}else{0},..Default::default()}
    }
    #[test]
    fn config_rejects_invalid_before_rm() {
        for codec in 0..3 {assert!(config(&cfg(codec)).is_ok());}
        let mut c=cfg(3);assert!(config(&c).is_err());
        c=cfg(0);c.width=127;assert!(config(&c).is_err());
        c=cfg(0);c.fps=65536;assert!(config(&c).is_err());
        c=cfg(0);c.idr_period=134217729;assert!(config(&c).is_err());
        c=cfg(0);c.rc_mode=2;assert!(config(&c).is_err());
        c.bitrate=1;c.max_bitrate=200000000;c.vbv_bits=1;assert!(config(&c).is_ok());
        c.max_bitrate=200000001;assert!(config(&c).is_err());
    }
    #[test]
    fn timing_and_output_boundaries() {
        let(mut f,mut p)=(7,8);
        unsafe {
            assert_eq!(nv_venc_c2_timing(29.97f64.to_bits(),2000000,&mut f,&mut p),0);
            assert_eq!((f,p),(30,60));
            assert_eq!(nv_venc_c2_timing(f64::NAN.to_bits(),0,&mut f,&mut p),-1);
            assert_eq!((f,p),(30,60));
            assert_eq!(nv_venc_c2_timing(65535f64.to_bits(),i64::MAX-1,&mut f,&mut p),0);
            assert_eq!(p,134217728);
            assert_eq!(nv_venc_c2_timing(30f64.to_bits(),-1,&mut f,&mut p),0);assert_eq!(p,0);
            assert_eq!(nv_venc_c2_output_size(u64::MAX,1,&mut f),-1);
            assert_eq!(nv_venc_c2_output_size(u32::MAX as u64,1,&mut f),-1);
            assert_eq!(nv_venc_c2_output_size(100,20,&mut f),0);assert_eq!(f,120);
        }
    }
    fn view(format:u32)->NvVencC2View {
        let mut v=NvVencC2View {type_:if format<3 {0}else{2},num_planes:if format<3 {3}else{4},
            root_planes:if format==2 {3}else if format<2 {2}else{1},width:128,height:96,
            crop_width:128,crop_height:96,..Default::default()};
        for i in 0..v.num_planes as usize {
            v.planes[i]=NvVencC2Plane {data:(0x10000+i*0x10000) as *const u8,row_inc:128,
                col_inc:1,col_sampling:1,row_sampling:1,allocated_depth:8,bit_depth:8,
                root_index:i as u32,..Default::default()};
        }
        if format<3 {
            for i in 1..3 {v.planes[i].row_sampling=2;v.planes[i].col_sampling=2;}
            if format==2 {v.planes[1].row_inc=64;v.planes[2].row_inc=64;}
            else {
                let root=if format==0 {1}else{2};
                for i in 1..3 {v.planes[i].col_inc=2;v.planes[i].root_index=root;
                    v.planes[i].root_offset=if i as u32==root {0}else{1};
                    v.planes[i].data=(0x20000+v.planes[i].root_offset as usize) as *const u8;}
            }
        } else {
            let root=if format==3 {0}else{2};
            let offsets=if format==3 {[0,1,2,3]}else{[2,1,0,3]};
            for (i,off) in offsets.into_iter().enumerate() {
                v.planes[i].root_index=root;v.planes[i].root_offset=off;
                v.planes[i].data=(0x10000+off as usize) as *const u8;
                v.planes[i].row_inc=512;v.planes[i].col_inc=4;
            }
        }
        v
    }
    #[test]
    fn five_layouts_and_bad_descriptors() {
        for format in 0..5 {
            let v=view(format);let(f,p)=input(&v,128,96).unwrap();assert_eq!(f,format);
            assert_eq!(p[0].length,if format<3 {12288}else{49151});
            let mut bad=v;bad.crop_left=1;assert!(input(&bad,128,96).is_err());
            bad=v;bad.planes[0].row_inc=-1;assert!(input(&bad,128,96).is_err());
            bad=v;bad.planes[0].bit_depth=10;assert!(input(&bad,128,96).is_err());
            bad=v;bad.planes[0].data=(usize::MAX-2) as *const u8;assert!(input(&bad,128,96).is_err());
            bad=v;bad.planes[0].root_offset=1000000;assert!(input(&bad,128,96).is_err());
        }
    }
    #[test]
    fn raw_plane_short_stride_and_overflow() {
        let bytes=[0u8;64];let mut p=NvVencPlane {data:bytes.as_ptr(),length:64,stride:8};
        unsafe {assert_eq!(plane(&p,8,8,0).unwrap().len(),64);}
        p.length=63;unsafe {assert!(plane(&p,8,8,0).is_err());}
        p.length=64;p.stride=7;unsafe {assert!(plane(&p,8,8,0).is_err());}
        p.stride=u64::MAX;unsafe {assert!(plane(&p,8,8,0).is_err());}
        assert!(extent(usize::MAX,usize::MAX,8).is_none());
    }
    #[test]
    #[ignore="requires a real NVENC GPU"]
    fn session_handoff_runtime_and_recovery() {
        for codec in 0..3 {
            let owner=std::thread::spawn(move|| {
                let c=cfg(codec);let mut s=ptr::null_mut();
                unsafe {assert_eq!(nv_venc_sess_create(&c,&mut s),0);}
                let mut owner=unsafe {Box::from_raw(s)};
                let y=vec![96u8;128*96];let uv=vec![128u8;128*48];
                let mut planes=[NvVencPlane {data:y.as_ptr(),length:y.len() as u64,stride:128},
                    NvVencPlane {data:uv.as_ptr(),length:uv.len() as u64,stride:128},NvVencPlane::default()];
                let mut out=NvVencOut::default();
                let s=&mut *owner as *mut NvVencSess;
                unsafe {
                    assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,0,&mut out),0);assert_eq!(out.key,1);
                    let mut len=0;assert_eq!(nv_venc_sess_codec_config(s,ptr::null_mut(),0,&mut len),0);assert!(len>0);
                    let mut short=[0xa5u8;1];assert_eq!(nv_venc_sess_codec_config(s,short.as_mut_ptr(),1,&mut len),-1);assert_eq!(short,[0xa5]);
                    let before=owner.encoder.cfg.clone();
                    let bad=NvVencSessUpdate {fps:0,bitrate:1,max_bitrate:1,vbv_bits:1,idr_period:1};
                    assert_eq!(nv_venc_sess_update(s,&bad),-1);
                    assert_eq!(owner.encoder.cfg.fps,before.fps);assert_eq!(owner.encoder.cfg.idr_period,before.idr_period);
                    let good=NvVencSessUpdate {fps:24,bitrate:1000000,max_bitrate:1000000,vbv_bits:1000000,idr_period:1};
                    assert_eq!(nv_venc_sess_update(s,&good),0);
                    assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,0,&mut out),0);assert_eq!(out.key,1);
                    let no_period=NvVencSessUpdate {idr_period:0,..good};assert_eq!(nv_venc_sess_update(s,&no_period),0);
                    planes[0].length-=1;
                    assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,0,&mut out),-1);assert!(out.data.is_null());assert_eq!(out.length,0);
                    planes[0].length+=1;
                    assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,0,&mut out),0);assert_eq!(out.key,1);
                    assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,1,&mut out),0);assert_eq!(out.key,1);
                    for fault in if codec==2 {vec![1,2]}else{vec![1]} {
                        owner.encoder.completion_fault=fault;
                        assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,0,&mut out),-1);
                        assert!(out.data.is_null());assert_eq!(out.length,0);
                        assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,0,&mut out),0);
                        assert_eq!(out.key,1);assert!(out.length>0);
                    }
                    owner.encoder.completion_fault=3;
                    assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,0,&mut out),-2);
                    assert!(owner.poisoned);assert!(out.data.is_null());assert_eq!(out.length,0);
                    assert_eq!(nv_venc_sess_update(s,&good),-2);
                    assert_eq!(nv_venc_sess_codec_config(s,ptr::null_mut(),0,&mut len),-2);
                    assert_eq!(len,0);
                    assert_eq!(nv_venc_sess_encode(s,planes.as_ptr(),0,0,&mut out),-2);
                }
                owner
            }).join().unwrap();
            // The creating/encoding worker has joined before another thread destroys.
            std::thread::spawn(move|| {let raw=Box::into_raw(owner);unsafe {nv_venc_sess_destroy(raw);}}).join().unwrap();
        }
    }
}
