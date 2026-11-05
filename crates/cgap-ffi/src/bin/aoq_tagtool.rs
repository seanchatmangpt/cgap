//! aoq_tagtool — the blake3 bridge bin (ZP-11 G1, PLANS row
//! ggen/docs/jira/v26.11.4/PLANS.md "ZP-11 ... + `aoq_tagtool` blake3
//! bridge bin").
//!
//! Cross-language pin tool the golden-window court uses: reads body bytes
//! on stdin, prints the 16-byte blake3 tag hex (Tag16 = blake3(body)[..16],
//! the AOQ1 wire law `cgap-arrow/src/ipc.rs` tag gate computes). Go is
//! stdlib-only (arcus zero-dependency law), so the Go-side courts pin
//! their blake3 port against THIS binary's output — one executable law
//! (cgap-blake3), many language bindings.
//!
//! Modes:
//! - (default) read all of stdin, print 32 hex chars (the 16-byte tag).
//! - `--self-test`: run the vector court in-process and exit 0/2
//!   (fail-closed: exit 2 on any vector mismatch, never a fallback).
//!
//! Vectors (pinned, R1's official cgap-blake3 vectors
//! `crates/cgap-blake3/tests/official_vectors.rs`):
//! - blake3("")        = af1349b9f5f9a1a6a0404dea36dcc9499bcb24c38... (full
//!   32B digest); Tag16 = af1349b9f5f9a1a6a0404dea36dcc949
//! - blake3(4096B)     = 015094013f57a5277b59d8475c0501042c0b642e...;
//!   Tag16 = 015094013f57a5277b59d8475c050104
//!
//! Host-surface only: XIP-safety law matches aoq_sidecar.rs (the
//! `target_os = "none"` build compiles this bin out).
// XIP safety (coordinator call 2026-11-04): the riscv32imc XIP build
// (`target_os = "none"`) compiles bin targets too; std/alloc are unresolved
// there. This bin is HOST-SURFACE ONLY, so the none-target gets an empty
// std-free bin and the whole tool body is compiled out.
#[cfg(target_os = "none")]
fn main() {}

#[cfg(not(target_os = "none"))]
fn main() {
    use std::io::Read;

    let self_test = std::env::args_os().skip(1).any(|a| a == "--self-test");

    if self_test {
        // Vector court: the two pinned vectors above, over the same crate
        // the sidecar's tag gate computes with (cgap-arrow::blake3, which
        // re-exports/delegates to cgap-blake3's kernel).
        let empty = [0u8; 0];
        let got_empty = tag16_hex(&empty);
        let got_4096 = {
            // The official corpus recipe: input byte i = (i % 251)
            // (official_vectors.rs header law) — NOT zero-filled.
            let body: Vec<u8> = (0..4096usize).map(|i| (i % 251) as u8).collect();
            tag16_hex(&body)
        };
        let want_empty = "af1349b9f5f9a1a6a0404dea36dcc949";
        let want_4096 = "015094013f57a5277b59d8475c050104";
        let ok = got_empty == want_empty && got_4096 == want_4096;
        println!("self-test empty  Tag16 = {got_empty}");
        println!("self-test 4096B  Tag16 = {got_4096}");
        if !ok {
            eprintln!("aoq_tagtool: vector mismatch (fail-closed exit 2)");
            std::process::exit(2);
        }
        println!("self-test PASS (2 vectors)");
        return;
    }

    let mut body = Vec::new();
    std::io::stdin().read_to_end(&mut body).unwrap_or_else(|e| {
        eprintln!("aoq_tagtool: stdin read fault: {e}");
        std::process::exit(2);
    });
    println!("{}", tag16_hex(&body));
}

/// Tag16 = blake3(body)[..16], hex-encoded (the AOQ1 wire tag law).
#[cfg(not(target_os = "none"))]
fn tag16_hex(body: &[u8]) -> String {
    let d = cgap_arrow::blake3::hash(body);
    let mut hex = String::with_capacity(32);
    for b in &d[..16] {
        use std::fmt::Write;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}
