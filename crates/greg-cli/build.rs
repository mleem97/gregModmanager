//! Same Steam rpath handling as `greg-app` (see its `build.rs`).

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("linux") {
        println!("cargo:rustc-link-arg-bins=-Wl,-rpath,$ORIGIN");
    } else if target.contains("darwin") {
        println!("cargo:rustc-link-arg-bins=-Wl,-rpath,@loader_path");
    }
    // Windows resolves DLLs beside the executable by default.
    println!("cargo:rerun-if-changed=build.rs");
}
