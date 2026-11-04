//! AB-4 (CG-1-shaped) constant-time/source-hygiene scan court over the
//! three kernel files: `src/poly1305.rs`, `src/aead.rs`, `src/lib.rs`.
//!
//! Laws: zero `unsafe` outside the crate-level `#![forbid(unsafe_code)]`
//! attribute in src/lib.rs; zero `asm!` / `core::arch` anywhere in the
//! three files; fail-closed restore witness after the sandbox phase.
//!
//! ANTI-VACUITY (sandbox-add-token kill): injecting each forbidden token
//! into /tmp SANDBOX copies (never the repo tree) must flip the scan --
//! a scan that accepts its own token is a vacuous gate.

use std::path::{Path, PathBuf};

const KERNEL_FILES: [&str; 3] = ["poly1305.rs", "aead.rs", "lib.rs"];
const FORBIDDEN_ASM_TOKENS: [&str; 2] = ["asm!", "core::arch"];

fn kernel_paths(manifest: &str) -> std::vec::Vec<PathBuf> {
    KERNEL_FILES
        .iter()
        .map(|f| Path::new(manifest).join("src").join(f))
        .collect()
}

/// Scan one file: (unsafe_lines, asm_hits) reported as violations.
fn scan(text: &str, path: &Path) -> std::vec::Vec<String> {
    let mut violations = std::vec::Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.contains("unsafe") && !line.contains("forbid(unsafe_code)") {
            violations.push(format!("{:?}:{}: unsafe outside forbid attr: {line}", path, n + 1));
        }
        for tok in FORBIDDEN_ASM_TOKENS {
            if line.contains(tok) {
                violations.push(format!("{:?}:{}: forbidden token {tok:?}: {line}", path, n + 1));
            }
        }
    }
    violations
}

fn read(path: &Path) -> String {
    String::from_utf8_lossy(&std::fs::read(path).expect("read kernel source")).into_owned()
}

/// The court: over the three kernel files, the scans must report zero
/// violations; the sandbox phase (below) proves the scans flip on their
/// own tokens; a restore witness pins the repo tree byte-exact.
#[test]
fn kernel_scan_zero_unsafe_zero_asm_with_sandbox_kills() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let paths = kernel_paths(manifest);
    assert_eq!(paths.len(), 3, "three kernel files scanned");
    let before: std::vec::Vec<(PathBuf, std::vec::Vec<u8>)> = paths
        .iter()
        .map(|p| (p.clone(), std::fs::read(p).expect("read source")))
        .collect();

    for p in &paths {
        let v = scan(&read(p), p);
        assert!(
            v.is_empty(),
            "kernel hygiene law violated:\n{}",
            v.join("\n")
        );
    }

    // SANDBOX ADD-TOKEN KILLS: each injected token must flip the scan.
    let mut kills = 0;
    let cases: [(&str, &str); 3] = [
        ("unsafe { let _x: u32 = 1; }", "unsafe-injection"),
        ("let _ = core::arch::asm!(\"nop\");", "asm-injection"),
        ("use core::arch::x86::_mm_add_epi8;", "core-arch-injection"),
    ];
    for (token, label) in cases {
        let sandbox =
            std::env::temp_dir().join(format!("ab4-ct-scan-{}-{label}.rs", std::process::id()));
        let mutated = format!("{token}\n{}", read(&paths[0]));
        std::fs::write(&sandbox, mutated.as_bytes()).expect("write sandbox");
        let v = scan(&read(&sandbox), &sandbox);
        let _ = std::fs::remove_file(&sandbox);
        assert!(
            !v.is_empty(),
            "VACUOUS SCAN: {label} token undetected"
        );
        kills += 1;
    }
    assert_eq!(kills, 3, "all three injection kills fired");

    // Fail-closed restore witness.
    for (p, before_bytes) in &before {
        let after = std::fs::read(p).expect("re-read source");
        assert_eq!(before_bytes, &after, "{:?} untouched (fail-closed restore)", p);
    }
}
