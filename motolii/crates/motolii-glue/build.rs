fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let include = manifest.join("../../third_party/include/webgpu");
    let lib = manifest.join("../../third_party/lib");
    cc::Build::new()
        .file("src/macos/native_device.c")
        .include(&include)
        .flag("-fno-strict-aliasing")
        .compile("native_device");
    println!("cargo:rustc-link-search=native={}", lib.display());
    println!("cargo:rustc-link-lib=dylib=wgpu_native");
    println!("cargo:rustc-link-lib=dylib=thorvg-1.1");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib.display());
    println!("cargo:rerun-if-changed=src/macos/native_device.c");
}
