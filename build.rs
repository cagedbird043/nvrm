// The cdylib carries DT_SONAME libnv_bionic_native.so, so a client linked by path (Soong links a prebuilt
// shared library by its full intermediate path) still records NEEDED libnv_bionic_native.so.
fn main() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,libnv_bionic_native.so");
}
