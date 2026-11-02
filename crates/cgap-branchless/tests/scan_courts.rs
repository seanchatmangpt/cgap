//! Lane TS2 differential courts + 0-branch census for `scan` primitives.
//!
//! Chicago discipline: real oracles (naive scalar reference implementations
//! computed in the test), assert final state. Seeded corpus uses the frozen
//! corpus constant 0xA20C_0FFE_E5A7_E000. No mocks.
//!
//! Census: compiles the real `src/scan.rs` with nightly rustc into a /tmp
//! object and censuses the real llvm-objdump disassembly. Every HOT
//! primitive symbol must carry 0 flow-control mnemonics on the host
//! (aarch64). Anti-vacuity arms:
//! (a) classifier-alive: the loop-based `naive_serial_prefix_sum`
//!     REFERENCE-DRIVER cell MUST be flagged BRANCHY (a census that cannot
//!     see a real back-edge is vacuous);
//! (b) mutation M-TS2-1: a sandbox copy of scan.rs with a runtime-bound
//!     loop injected into prefix_xor_u64x8 MUST flip that symbol to
//!     BRANCHY. A court that passes while either arm fails exits nonzero.
//!
//! Loop back-edges belonging to a caller driver are REFERENCE-DRIVER class
//! per house law; no scan primitive contains a loop.

use cgap_branchless::scan::{
    count_leading_eq_u8, prefix_and_u64x8, prefix_max_u8x8, prefix_or_u64x8, prefix_sum_u32x8,
    prefix_xor_u64x8, segmented_prefix_sum_u32x8,
};

/// Frozen corpus constant (never varied; see repo CLAUDE.md seeds law).
const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

/// splitmix64 driver for the seeded corpus (reference-driver loop class).
fn corpus(mut state: u64) -> impl Iterator<Item = u64> {
    core::iter::from_fn(move || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Some(z ^ (z >> 31))
    })
}

// --- naive scalar oracles (REFERENCE-DRIVER: loops by design) ---

fn naive_prefix_or(a: [u64; 8]) -> [u64; 8] {
    let mut out = [0u64; 8];
    let mut acc = 0u64;
    for i in 0..8 {
        acc |= a[i];
        out[i] = acc;
    }
    out
}

fn naive_prefix_and(a: [u64; 8]) -> [u64; 8] {
    let mut out = [0u64; 8];
    let mut acc = u64::MAX;
    for i in 0..8 {
        acc &= a[i];
        out[i] = acc;
    }
    out
}

fn naive_prefix_xor(a: [u64; 8]) -> [u64; 8] {
    let mut out = [0u64; 8];
    let mut acc = 0u64;
    for i in 0..8 {
        acc ^= a[i];
        out[i] = acc;
    }
    out
}

fn naive_prefix_sum(a: [u32; 8]) -> [u32; 8] {
    let mut out = [0u32; 8];
    let mut acc = 0u32;
    for i in 0..8 {
        acc = acc.wrapping_add(a[i]);
        out[i] = acc;
    }
    out
}

fn naive_segmented(values: [u32; 8], flags: [bool; 8]) -> [u32; 8] {
    let mut out = [0u32; 8];
    let mut acc = 0u32;
    for i in 0..8 {
        if flags[i] {
            acc = 0;
        }
        acc = acc.wrapping_add(values[i]);
        out[i] = acc;
    }
    out
}

fn naive_prefix_max(a: [u8; 8]) -> [u8; 8] {
    let mut out = [0u8; 8];
    let mut acc = 0u8;
    for i in 0..8 {
        acc = acc.max(a[i]);
        out[i] = acc;
    }
    out
}

fn naive_leading_eq(bytes: [u8; 8], target: u8) -> usize {
    let mut n = 0usize;
    for b in bytes.iter() {
        if *b != target {
            break;
        }
        n += 1;
    }
    n
}

// --- differential courts ---

#[test]
fn court_prefix_or_u64x8() {
    // Exhaustive-ish edge corpus + seeded random corpus vs naive oracle.
    let edges: [u64; 8] = [
        0,
        u64::MAX,
        1,
        1 << 63,
        0x5555_5555_5555_5555,
        0xAAAA_AAAA_AAAA_AAAA,
        7,
        u64::MAX - 1,
    ];
    assert_eq!(prefix_or_u64x8(edges), naive_prefix_or(edges));
    let mut draws = corpus(CORPUS_SEED);
    for _ in 0..4096 {
        let a = [
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
        ];
        assert_eq!(
            prefix_or_u64x8(a),
            naive_prefix_or(a),
            "corpus draw mismatch"
        );
    }
}

#[test]
fn court_prefix_and_u64x8() {
    let edges: [u64; 8] = [
        0,
        u64::MAX,
        1,
        1 << 63,
        0x5555_5555_5555_5555,
        0xAAAA_AAAA_AAAA_AAAA,
        7,
        u64::MAX - 1,
    ];
    assert_eq!(prefix_and_u64x8(edges), naive_prefix_and(edges));
    let mut draws = corpus(CORPUS_SEED);
    for _ in 0..4096 {
        let a = [
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
        ];
        assert_eq!(
            prefix_and_u64x8(a),
            naive_prefix_and(a),
            "corpus draw mismatch"
        );
    }
}

#[test]
fn court_prefix_xor_u64x8() {
    // Gray-code property + seeded corpus vs naive oracle.
    let gray = prefix_xor_u64x8([0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(gray[0], 0);
    assert_eq!(gray[1], 1);
    assert_eq!(gray[2], 3);
    let edges: [u64; 8] = [
        0,
        u64::MAX,
        1,
        1 << 63,
        0x5555_5555_5555_5555,
        0xAAAA_AAAA_AAAA_AAAA,
        7,
        u64::MAX - 1,
    ];
    assert_eq!(prefix_xor_u64x8(edges), naive_prefix_xor(edges));
    let mut draws = corpus(CORPUS_SEED);
    for _ in 0..4096 {
        let a = [
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
            draws.next().unwrap(),
        ];
        assert_eq!(
            prefix_xor_u64x8(a),
            naive_prefix_xor(a),
            "corpus draw mismatch"
        );
    }
}

#[test]
fn court_prefix_sum_u32x8() {
    // Wrapping-overflow corpus: values drawn to overflow freely.
    let ones = [1u32; 8];
    assert_eq!(prefix_sum_u32x8(ones), [1, 2, 3, 4, 5, 6, 7, 8]);
    let mut draws = corpus(CORPUS_SEED);
    for _ in 0..4096 {
        let a = [
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
        ];
        assert_eq!(
            prefix_sum_u32x8(a),
            naive_prefix_sum(a),
            "corpus draw mismatch"
        );
    }
}

#[test]
fn court_segmented_prefix_sum_u32x8() {
    // EXHAUSTIVE over all 2^8 flag patterns x seeded value vectors.
    let mut draws = corpus(CORPUS_SEED);
    let mut vecs: Vec<[u32; 8]> = Vec::new();
    for _ in 0..64 {
        vecs.push([
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
            draws.next().unwrap() as u32,
        ]);
    }
    for fbits in 0u8..=255 {
        let flags = [
            fbits & 1 != 0,
            fbits & 2 != 0,
            fbits & 4 != 0,
            fbits & 8 != 0,
            fbits & 16 != 0,
            fbits & 32 != 0,
            fbits & 64 != 0,
            fbits & 128 != 0,
        ];
        for v in &vecs {
            assert_eq!(
                segmented_prefix_sum_u32x8(*v, flags),
                naive_segmented(*v, flags),
                "flags={fbits:08b}"
            );
        }
    }
}

#[test]
fn court_prefix_max_u8x8() {
    let mut draws = corpus(CORPUS_SEED);
    for _ in 0..4096 {
        let a = [
            draws.next().unwrap() as u8,
            draws.next().unwrap() as u8,
            draws.next().unwrap() as u8,
            draws.next().unwrap() as u8,
            draws.next().unwrap() as u8,
            draws.next().unwrap() as u8,
            draws.next().unwrap() as u8,
            draws.next().unwrap() as u8,
        ];
        assert_eq!(
            prefix_max_u8x8(a),
            naive_prefix_max(a),
            "corpus draw mismatch"
        );
    }
    // Monotonicity property on edge corpus incl. 0 and 255.
    for a in [[0u8; 8], [255u8; 8], [0, 255, 0, 255, 1, 254, 2, 253]] {
        let out = prefix_max_u8x8(a);
        assert_eq!(out, naive_prefix_max(a));
        for w in out.windows(2) {
            assert!(w[1] >= w[0]);
        }
    }
}

#[test]
fn court_count_leading_eq_u8() {
    // EXHAUSTIVE over all 256 match-mask patterns x target {0, 1, 0x7F, 0xFF}.
    for mbits in 0u16..=255 {
        for target in [0u8, 1, 0x7F, 0xFF] {
            let mut bytes = [0u8; 8];
            for i in 0..8 {
                let matched = mbits & (1 << i) != 0;
                // Matched lanes carry target; unmatched lanes carry a
                // guaranteed-different byte.
                bytes[i] = if matched { target } else { target ^ 0x55 };
                if bytes[i] == target {
                    bytes[i] = target.wrapping_add(1);
                }
            }
            assert_eq!(
                count_leading_eq_u8(bytes, target),
                naive_leading_eq(bytes, target),
                "mask={mbits:08b} target={target}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 0-branch census (real nightly rustc + llvm-objdump, host aarch64)
// ---------------------------------------------------------------------------

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const CRATE_DIR: &str = "/Users/sac/cgap/crates/cgap-branchless";
/// Per-process scratch (G10 race fix, lane DX-6): fixed shared /tmp work
/// dir raced across concurrent cargo invocations; keyed on process::id
/// (branchless_census.rs pattern).
fn work() -> &'static String {
    static WORK: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    WORK.get_or_init(|| format!("/tmp/ts2-census-{}", std::process::id()))
}

const HOT_PRIMITIVES: [&str; 6] = [
    "prefix_or_u64x8",
    "prefix_and_u64x8",
    "prefix_xor_u64x8",
    "prefix_sum_u32x8",
    "segmented_prefix_sum_u32x8",
    "count_leading_eq_u8",
];

fn tools() -> &'static (String, String) {
    static T: OnceLock<(String, String)> = OnceLock::new();
    T.get_or_init(|| {
        let sysroot = String::from_utf8_lossy(
            &Command::new("rustup")
                .args(["run", "nightly", "rustc", "--print", "sysroot"])
                .output()
                .expect("rustup run nightly rustc --print sysroot")
                .stdout,
        )
        .trim()
        .to_string();
        let cand = format!("{sysroot}/lib/rustlib/aarch64-apple-darwin/bin/llvm-objdump");
        let host = if fs::metadata(&cand).is_ok() {
            cand
        } else {
            fs::read_dir(format!("{sysroot}/lib/rustlib"))
                .expect("sysroot lib/rustlib")
                .filter_map(|e| e.ok())
                .find(|e| e.path().join("bin/llvm-objdump").exists())
                .map(|e| e.path().to_str().unwrap().to_string())
                .expect("llvm-objdump in nightly sysroot")
        };
        (sysroot, host)
    })
}

fn run_cmd(program: &str, args: &[&str]) -> (bool, String) {
    let libdir = format!("{}/lib/rustlib/aarch64-apple-darwin/lib", tools().0);
    match Command::new(program)
        .args(args)
        .env("DYLD_LIBRARY_PATH", &libdir)
        .output()
    {
        Ok(o) => {
            let mut t = String::from_utf8_lossy(&o.stdout).into_owned();
            t.push_str(&String::from_utf8_lossy(&o.stderr));
            (o.status.success(), t)
        }
        Err(e) => (false, format!("spawn failed: {e}")),
    }
}

fn compile_obj(src: &str, out: &str) -> bool {
    let rustc = format!("{}/bin/rustc", tools().0);
    let args = [
        "--edition",
        "2021",
        "--crate-type",
        "lib",
        "-C",
        "opt-level=3",
        "-C",
        "debuginfo=0",
        "-C",
        "link-dead-code=yes",
        "--emit=obj",
        "-o",
        out,
        src,
    ];
    let (ok, text) = run_cmd(&rustc, &args);
    if !ok {
        eprintln!("[COMPILE-FAIL] {src}\n{text}");
    }
    ok
}

fn branch_counts(obj: &str) -> Vec<(String, usize)> {
    let objdump = &tools().1;
    let (ok, text) = run_cmd(objdump, &["-d", "--no-show-raw-insn", obj]);
    assert!(ok, "[OBJDUMP-FAIL] {obj}: {text}");
    let mut per_sym: Vec<(String, usize)> = Vec::new();
    let mut cur: Option<(String, usize)> = None;
    for line in text.lines() {
        if let Some(open) = line.find('<') {
            if let Some(close) = line.find(">:") {
                if open < close {
                    if let Some(c) = cur.take() {
                        per_sym.push(c);
                    }
                    cur = Some((line.trim().to_string(), 0));
                    continue;
                }
            }
        }
        if let Some((_name, count)) = cur.as_mut() {
            let toks: Vec<&str> = line.split_whitespace().collect();
            if toks.len() >= 2 {
                let m = toks[1].trim_end_matches(':');
                let flow = m == "b"
                    || m == "bl"
                    || m == "br"
                    || m == "blr"
                    || m == "cbz"
                    || m == "cbnz"
                    || m == "tbz"
                    || m == "tbnz"
                    || m.starts_with("b.");
                if flow {
                    *count += 1;
                }
            }
        }
    }
    if let Some(c) = cur.take() {
        per_sym.push(c);
    }
    per_sym
}

fn symbol_branches(symbols: &[(String, usize)], needle: &str) -> Option<usize> {
    let hits: Vec<usize> = symbols
        .iter()
        .filter(|(name, _)| name.contains(needle))
        .map(|(_, c)| *c)
        .collect();
    if hits.is_empty() {
        None
    } else {
        Some(hits.iter().sum())
    }
}

fn census_work_dir() -> PathBuf {
    let d = PathBuf::from(work().as_str());
    let _ = fs::create_dir_all(&d);
    d
}

#[test]
fn census_scan_hot_cells_zero_branch() {
    let d = census_work_dir();
    let src = d.join("scan_wrapper.rs");
    let obj = d.join("scan.o");
    fs::write(
        &src,
        format!(
            "#![no_std]\n#![allow(dead_code)]\n#[path = \"{CRATE_DIR}/src/scan.rs\"] pub mod scan;\n"
        ),
    )
    .expect("write wrapper");
    assert!(
        compile_obj(src.to_str().unwrap(), obj.to_str().unwrap()),
        "scan.rs must compile standalone"
    );

    let syms = branch_counts(obj.to_str().unwrap());
    let mut report = String::from("census grid (host aarch64):\n");
    for needle in HOT_PRIMITIVES.iter() {
        let count = symbol_branches(&syms, needle).unwrap_or_else(|| {
            panic!("[MISSING] symbol {needle} not found in scan.o — census cannot run, not pass")
        });
        report.push_str(&format!("  {needle}: {count} flow mnemonics\n"));
        assert_eq!(
            count, 0,
            "HOT cell {needle} is BRANCHY — 0-branch law violated"
        );
    }

    // Anti-vacuity arm (a): the naive serial reference in THIS test binary is
    // loop-based; prove the classifier sees real back-edges by censusing a
    // compiled loop reference.
    let ref_src = d.join("naive_ref.rs");
    fs::write(
        &ref_src,
        "#![no_std]\n#![allow(dead_code)]\n#[inline(never)]\npub fn naive_serial_prefix_sum(a: [u32; 8], n: usize) -> [u32; 8] {\n    let mut out = [0u32; 8];\n    let mut acc = 0u32;\n    let mut i = 0usize;\n    while i < n {\n        acc = acc.wrapping_add(a[i]);\n        out[i] = a[i];\n        acc = acc.wrapping_sub(a[i]);\n        out[i] = acc;\n        i += 1;\n    }\n    out\n}\n",
    )
    .expect("write naive ref");
    let ref_obj = d.join("naive_ref.o");
    assert!(compile_obj(
        ref_src.to_str().unwrap(),
        ref_obj.to_str().unwrap()
    ));
    let nsyms = branch_counts(ref_obj.to_str().unwrap());
    let ncount: usize = nsyms.iter().map(|(_, c)| *c).sum();
    assert!(
        ncount > 0,
        "VACUOUS: classifier reads 0 branches in a real loop reference object"
    );

    // Anti-vacuity arm (b): mutation M-TS2-1 — sandbox copy with a
    // runtime-bound loop injected into prefix_xor_u64x8 MUST flip.
    let real = fs::read_to_string(format!("{CRATE_DIR}/src/scan.rs")).expect("read scan.rs");
    let mutated = real.replace(
        "pub const fn prefix_xor_u64x8(a: [u64; 8]) -> [u64; 8] {\n    let mut v = a;",
        "pub const fn prefix_xor_u64x8(a: [u64; 8]) -> [u64; 8] {\n    let mut v = a;\n    let n = (a[0] & 0xf) as usize;\n    let mut k = 0usize;\n    let mut s = 0u64;\n    while k < n { s = s.wrapping_add(k as u64); k += 1; }\n    v[7] = v[7].wrapping_add(s);\n    let _ = n;",
    );
    assert!(
        mutated != real,
        "M-TS2-1 did not apply — mutation court cannot run"
    );
    let mut_src = d.join("scan_mutant.rs");
    fs::write(
        &mut_src,
        format!(
            "#![no_std]\n#![allow(dead_code)]\n#[path = \"{}/scan.rs\"] pub mod scan;\n",
            d.join("scan_sandbox_src").display()
        ),
    )
    .expect("write mutant wrapper");
    // write the mutated source where the wrapper points
    let sandbox_dir = d.join("scan_sandbox_src");
    let _ = fs::remove_dir_all(&sandbox_dir);
    let _ = fs::create_dir_all(&sandbox_dir);
    fs::write(sandbox_dir.join("scan.rs"), &mutated).expect("write sandbox scan.rs");
    let mut_obj = d.join("scan_mutant.o");
    assert!(
        compile_obj(mut_src.to_str().unwrap(), mut_obj.to_str().unwrap()),
        "mutant must compile"
    );
    let msyms = branch_counts(mut_obj.to_str().unwrap());
    let mcount = symbol_branches(&msyms, "prefix_xor_u64x8").expect("mutant symbol missing");
    assert!(
        mcount > 0,
        "VACUOUS: M-TS2-1 mutant stays 0-branch — mutation court cannot flip"
    );

    eprintln!("{report}");
    eprintln!("anti-vacuity: naive ref {ncount} branches (BRANCHY, correct); M-TS2-1 mutant {mcount} branches (FLIPPED)\n");
}
