//! L19 BEAM contract court (aq-nif-boundary-spec.md first surface).
//!
//! One Rust integration test drives the REAL dylib in a REAL BEAM via
//! escript: mmap open (+ refusals), zero-copy egress projection, windows,
//! stamp ingress, lease bitmask lifecycle with GC-dtor law, the CrcMismatch
//! negative leg (file pwrite corruption of a mapped frame), the alloc-spy
//! zero-alloc egress census, and the SHIM-COURT anti-vacuity mutation
//! (AQ_MUTATE_CRC_SKIP=1): running the same court against the mutated
//! validator MUST exit 1 — a green run is VACUOUS. No mocks.

use std::path::PathBuf;
use std::process::Command;

/// Locate (building if needed) the NIF dylib; mirrors bridge_contract.rs.
fn nif_dylib_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            manifest_dir
                .join("../../target")
                .canonicalize()
                .unwrap_or(manifest_dir.join("../../target"))
        });
    for profile in ["debug", "release"] {
        for ext in ["dylib", "so"] {
            let p = target_root.join(profile).join(format!("libcgap_beam.{ext}"));
            if p.exists() {
                return p;
            }
        }
    }
    panic!("build the dylib first: cargo build -p cgap-beam");
}

/// Write the escript to a temp file.
fn write_escript(src: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("aq_slab_court_{}.erl", std::process::id()));
    std::fs::write(&p, src).expect("write escript");
    p
}
