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
    // Keep the same generated library path available to the Slint language
    // server and to the Rust build. The language server cannot evaluate this
    // build script itself, so it needs a stable workspace-relative path.
    let manifest_dir = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"),
    );
    let editor_library = manifest_dir.join("ui/.generated/lucide.slint");
    std::fs::create_dir_all(editor_library.parent().expect("library has a parent"))
        .expect("create generated Slint library directory");
    std::fs::copy(lucide_slint::lib(), &editor_library)
        .expect("copy lucide.slint for the Slint language server");

    let library = std::collections::HashMap::from([("lucide".to_string(), editor_library)]);
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
