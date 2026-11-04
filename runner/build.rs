fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("android") {
        return;
    }

    let dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("litert");
    println!("cargo:rerun-if-changed={}", dir.join("libLiteRt.so").display());
    println!("cargo:rustc-link-search=native={}", dir.display());
    println!("cargo:rustc-link-lib=dylib=LiteRt");
}
