//! Build script: compiles the Slint UI and keeps the Steam rpath.
//!
//! `steamworks-sys` links `libsteam_api` at build time. At runtime the loader
//! must find it: installers place it beside the executable, and this rpath
//! makes `$ORIGIN` (binary dir) / `@loader_path` resolve it there.
//!
//! The Lucide icon library (`lucide-slint`) is registered as the `@lucide`
//! Slint library so every `.slint` file can
//! `import { IconDisplay, IconSet } from "@lucide";`.

fn main() {
    let library = std::collections::HashMap::from([(
        "lucide".to_string(),
        std::path::PathBuf::from(lucide_slint::lib()),
    )]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library);
    slint_build::compile_with_config("ui/main.slint", config).expect("slint compile");
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
