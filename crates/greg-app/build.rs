//! Build script: compiles the Slint UI and keeps the Steam rpath.
//!
//! `steamworks-sys` links `libsteam_api` at build time. At runtime the loader
//! must find it: installers place it beside the executable, and this rpath
//! makes `$ORIGIN` (binary dir) / `@loader_path` resolve it there.

fn main() {
    slint_build::compile("ui/main.slint").expect("slint compile");
    println!("cargo:rerun-if-changed=ui/");

    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("linux") {
        println!("cargo:rustc-link-arg-bins=-Wl,-rpath,$ORIGIN");
    } else if target.contains("darwin") {
        println!("cargo:rustc-link-arg-bins=-Wl,-rpath,@loader_path");
    }
    // Windows resolves DLLs beside the executable by default.
    println!("cargo:rerun-if-changed=build.rs");
}
