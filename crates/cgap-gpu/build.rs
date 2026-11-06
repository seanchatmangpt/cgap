fn main() {
    // Check target OS dynamically from Cargo environment rather than host cfg.
    // When cross-compiling from Darwin to Linux, cfg!(target_os = "macos") evaluates
    // the build-script host (macOS), which erroneously tries to invoke gcc for the target.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("metal_shim.m")
            .flag("-fobjc-arc")
            .compile("metal_shim");

        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
