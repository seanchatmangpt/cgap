//! TS4 differential courts for `transpose_8x8` / `transpose_64x64`
//! (cgap-branchless). Oracle: naive scalar per-bit transpose (real
//! collaborator, no mocks). Corpus: frozen splitmix64 seed
//! `0xA20C_0FFE_E5A7_E000` plus patterned and boundary matrices.
//!
//! Anti-vacuity: a single-bit-flip mutant of transpose.rs in a /tmp
//! sandbox twin MUST fail the differential court (the twin binary exits
//! nonzero), and the unmutated twin MUST exit 0. A court that cannot
//! flip is FAIL.
//!
//! Census: HOT cell `transpose_8x8` compiled with nightly rustc into
//! /tmp objects and disassembled; 0 flow-control branches required on
//! host-aarch64 and riscv32imc (loud [SKIP] if the target is not
//! installed). `transpose_64x64` carries a documented 6-loop-back-edge
//! residual (data-independent control flow; bodies branch-free).

use cgap_branchless::transpose::{transpose_64x64, transpose_8x8};

/// Frozen corpus seed (never varied; cgap CLAUDE.md seeds law).
const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn oracle_8x8(x: u64) -> u64 {
    // bit (r*8+c) -> (c*8+r), fully explicit scalar relocation.
    let mut out = 0u64;
    for r in 0..8u32 {
        for c in 0..8u32 {
            let bit = (x >> (r * 8 + c)) & 1;
            out |= bit << (c * 8 + r);
        }
    }
    out
}

fn oracle_64x64(rows: [u64; 64]) -> [u64; 64] {
    // out[c] bit r = rows[r] bit c.
    let mut out = [0u64; 64];
    for r in 0..64usize {
        for c in 0..64u32 {
            let bit = (rows[r] >> c) & 1;
            out[c as usize] |= bit << r;
        }
    }
    out
}

fn corpus_8x8() -> Vec<u64> {
    let mut v = Vec::new();
    // boundary matrices
    v.push(0);
    v.push(u64::MAX);
    for b in 0..64u32 {
        v.push(1u64 << b); // every single-bit matrix
    }
    for r in 0..8u32 {
        v.push(0xFFu64 << (r * 8)); // one full row
        v.push(0x0101_0101_0101_0101u64 << r); // one full column
    }
    // byte-replicated exhaustive sweep: every (row-pattern, col-pattern)
    // 16-bit pair, replicated across all 8 rows/cols of the matrix.
    let mut p: u32 = 0;
    while p < 1 << 16 {
        let byte = p as u64 & 0xFF;
        let row = (((p >> 8) as u64 & 0xFF) << 8) | byte;
        v.push(row.wrapping_mul(0x0101_0101_0101_0101));
        p += 1;
    }
    // frozen splitmix corpus
    let mut s = CORPUS_SEED;
    for _ in 0..4096 {
        v.push(splitmix(&mut s));
    }
    v
}

fn corpus_64x64() -> Vec<[u64; 64]> {
    let mut v: Vec<[u64; 64]> = Vec::new();
    // all-zero / all-one
    v.push([0u64; 64]);
    v.push([u64::MAX; 64]);
    // boundary rows: each single full row / zero row / full column
    for r in 0..64usize {
        let mut m = [0u64; 64];
        m[r] = u64::MAX;
        v.push(m); // one full row
        let mut z = [u64::MAX; 64];
        z[r] = 0;
        v.push(z); // one zero row
        let mut c = [0u64; 64];
        for w in c.iter_mut() {
            *w = 1u64 << r; // one full column r
        }
        v.push(c);
    }
    // diagonal / anti-diagonal / every single-bit 64x64 matrix
    let mut dg = [0u64; 64];
    for r in 0..64usize {
        dg[r] = 1u64 << r;
    }
    v.push(dg);
    let mut ad = [0u64; 64];
    for r in 0..64usize {
        ad[r] = 1u64 << (63 - r);
    }
    v.push(ad);
    for b in 0..64u32 {
        let mut m = [0u64; 64];
        m[(b >> 3) as usize] = 1u64 << b; // single bit: element (b>>3, b)
        v.push(m);
        let mut m2 = [0u64; 64];
        m2[b as usize] = 1u64 << (b % 64); // single bit: element (b, b>>3)
        v.push(m2);
    }
    // block-structured: 8x8-block-constant matrices built from the 8x8
    // corpus patterns (exercises the quadrant-swap/block interface).
    let mut s8 = CORPUS_SEED ^ 0x11;
    for _ in 0..256 {
        let b = splitmix(&mut s8);
        let mut m = [0u64; 64];
        for r in 0..64usize {
            let mut row = 0u64;
            for c in 0..8u32 {
                row |= (b >> ((r % 8) * 8)) << (c * 8); // 8x8-constant rows
            }
            m[r] = row;
        }
        v.push(m);
    }
    // frozen splitmix corpus of full blocks
    let mut s = CORPUS_SEED ^ 0x5EED_5EED_5EED_5EED;
    for _ in 0..256 {
        let mut m = [0u64; 64];
        for w in m.iter_mut() {
            *w = splitmix(&mut s);
        }
        v.push(m);
    }
    v
}

#[test]
fn court_8x8_differential_oracle() {
    for x in corpus_8x8() {
        assert_eq!(transpose_8x8(x), oracle_8x8(x), "8x8 mismatch x={x:#018x}");
    }
}

#[test]
fn court_8x8_involution() {
    for x in corpus_8x8() {
        assert_eq!(transpose_8x8(transpose_8x8(x)), x, "involution x={x:#018x}");
    }
}

#[test]
fn court_64x64_differential_oracle() {
    for m in corpus_64x64() {
        assert_eq!(
            transpose_64x64(m),
            oracle_64x64(m),
            "64x64 mismatch m[0..4]={:?}",
            &m[0..4]
        );
    }
}

#[test]
fn court_64x64_involution_and_stage_selfinverse() {
    for m in corpus_64x64() {
        let twice = transpose_64x64(transpose_64x64(m));
        assert_eq!(twice, m, "64x64 involution m[0..4]={:?}", &m[0..4]);
    }
    // each harvested 8x8 stage is self-inverse: dswap(dswap(x)) == x.
    for x in corpus_8x8() {
        for (sh, mask) in [
            (7u32, 0x00AA_00AA_00AA_00AAu64),
            (14, 0x0000_CCCC_0000_CCCC),
            (28, 0x0000_0000_F0F0_F0F0),
        ] {
            let t = (x ^ (x >> sh)) & mask;
            let once = x ^ t ^ (t << sh);
            let t2 = (once ^ (once >> sh)) & mask;
            assert_eq!(once ^ t2 ^ (t2 << sh), x, "stage {sh} not self-inverse");
        }
    }
}

// ===================== census (branch-free house law) =====================

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CRATE_DIR: &str = "/Users/sac/cgap/crates/cgap-branchless";
/// Per-process scratch (G10 race fix, lane DX-6): fixed shared /tmp work
/// dirs raced across concurrent cargo invocations; each process gets its
/// own dir keyed on process::id (branchless_census.rs pattern).
fn work() -> &'static String {
    static WORK: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    WORK.get_or_init(|| format!("/tmp/ts4-census-{}", std::process::id()))
}

fn nightly_rustc() -> PathBuf {
    let out = Command::new("rustup")
        .args(["run", "nightly", "rustc", "--print", "sysroot"])
        .output()
        .expect("rustup run nightly rustc --print sysroot");
    let sysroot = String::from_utf8_lossy(&out.stdout).trim().to_string();
    PathBuf::from(sysroot).join("bin/rustc")
}

fn objdump() -> PathBuf {
    let out = Command::new("rustup")
        .args(["run", "nightly", "rustc", "--print", "sysroot"])
        .output()
        .expect("sysroot probe");
    let sysroot = String::from_utf8_lossy(&out.stdout).trim().to_string();
    fs::read_dir(format!("{sysroot}/lib/rustlib"))
        .expect("rustlib dir")
        .filter_map(|e| e.ok())
        .find(|e| e.path().join("bin/llvm-objdump").exists())
        .map(|e| e.path().join("bin/llvm-objdump"))
        .expect("llvm-objdump in nightly sysroot")
}

fn compile_obj(src: &Path, out: &Path, extra: &[&str]) {
    let args: Vec<String> = [
        "--edition",
        "2021",
        "--crate-type",
        "lib",
        "-C",
        "opt-level=3",
        "-C",
        "debuginfo=0",
        "-C",
        "llvm-args=--inline-threshold=0",
        "-C",
        "link-dead-code=yes",
        "--emit=obj",
    ]
    .iter()
    .chain(extra.iter())
    .map(|s| s.to_string())
    .chain(["-o".to_string()])
    .chain([out.to_str().unwrap().to_string()])
    .chain([src.to_str().unwrap().to_string()])
    .collect();
    let o = Command::new(nightly_rustc())
        .args(&args)
        .output()
        .expect("spawn rustc");
    assert!(
        o.status.success(),
        "census compile failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

fn branch_count(obj: &Path, arch: &str, f: &str) -> usize {
    let o = Command::new(objdump())
        .args(["-d", "--no-show-raw-insn", obj.to_str().unwrap()])
        .output()
        .expect("objdump");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    let mut in_fn = false;
    let mut hits = 0usize;
    for line in text.lines() {
        if line.contains('<') && line.contains(">:") {
            in_fn = line.contains(f) && !line.contains("_RNC") && !line.contains("_RINv");
            continue;
        }
        if !in_fn {
            continue;
        }
        let Some(m) = line.split_whitespace().nth(1) else {
            continue;
        };
        let m = m.trim_end_matches(':');
        let flow = match arch {
            "aarch64" => {
                m == "b"
                    || m == "bl"
                    || m == "br"
                    || m == "blr"
                    || m == "cbz"
                    || m == "cbnz"
                    || m == "tbz"
                    || m == "tbnz"
                    || m.starts_with("b.")
            }
            "riscv32" => {
                let stem = m.strip_prefix("c.").unwrap_or(m);
                matches!(
                    stem,
                    "beq"
                        | "bne"
                        | "blt"
                        | "bge"
                        | "bltu"
                        | "bgeu"
                        | "beqz"
                        | "bnez"
                        | "bgez"
                        | "bltz"
                        | "bgtz"
                        | "blez"
                        | "j"
                        | "jal"
                        | "jalr"
                )
            }
            _ => m.starts_with('j') || m == "call",
        };
        if flow {
            hits += 1;
        }
    }
    hits
}

/// transpose_8x8 caller cell: HOT — 0 branches (the `#[inline]`d
/// delta-swap body, witness on host-aarch64 and riscv32imc). Documented
/// residual for the transpose_64x64 caller cell: exactly 2 — the
/// `quad_mask(h)` match-dispatch compare branches (b.ls/b.hi on aarch64,
/// equivalent compare-branch pair on rv32imc); the six quadrant-pass
/// loops fully unroll at opt-level 3 on both targets. Any body branch
/// injected into the delta-swap stages would push past this budget.
const T64_LOOP_RESIDUAL: usize = 2;

fn symbol_present(obj: &Path, f: &str) -> bool {
    let o = Command::new(objdump())
        .args(["-d", "--no-show-raw-insn", obj.to_str().unwrap()])
        .output()
        .expect("objdump");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    // the REAL fn symbol (not LLVM clone instantiations)
    text.lines()
        .filter(|l| l.contains('<') && l.contains(">:"))
        .any(|l| l.contains(f) && !l.contains("_RNC") && !l.contains("_RINv"))
}

fn census_target(label: &'static str, arch: &'static str, extra: &[&str]) {
    let wrapper = Path::new(work().as_str()).join(format!("wrapper_{label}.rs"));
    fs::create_dir_all(work()).unwrap();
    fs::write(
        &wrapper,
        format!(
            "#![no_std]\n#![allow(dead_code)]\n\
             #[path = \"{CRATE_DIR}/src/transpose.rs\"] pub mod transpose;\n\
             #[no_mangle] #[inline(never)] pub fn census_t8(x: u64) -> u64 {{ core::hint::black_box(transpose::transpose_8x8(core::hint::black_box(x))) }}\n\
             #[no_mangle] #[inline(never)] pub fn census_t64(rows: [u64; 64]) -> [u64; 64] {{ core::hint::black_box(transpose::transpose_64x64(core::hint::black_box(rows))) }}\n"
        ),
    )
    .unwrap();
    let obj = Path::new(work().as_str()).join(format!("obj_{label}.o"));
    compile_obj(&wrapper, &obj, extra);
    assert!(
        symbol_present(&obj, "census_t8"),
        "[CENSUS-FAIL] {label} census_t8 symbol MISSING"
    );
    assert!(
        symbol_present(&obj, "census_t64"),
        "[CENSUS-FAIL] {label} census_t64 symbol MISSING"
    );
    let v8 = branch_count(&obj, arch, "census_t8");
    let v64 = branch_count(&obj, arch, "census_t64");
    assert_eq!(
        v8, 0,
        "[CENSUS-FAIL] {label} HOT cell census_t8 (inlined transpose_8x8) -> {v8} branches"
    );
    assert!(
        v64 <= T64_LOOP_RESIDUAL,
        "[CENSUS-FAIL] {label} census_t64 -> {v64} branches exceeds loop residual {T64_LOOP_RESIDUAL}"
    );
    println!("[PASS-CENSUS] {label} census_t8 (transpose_8x8 inlined): 0 branches (HOT)");
    println!("[PASS-CENSUS] {label} census_t64 (transpose_64x64 inlined): {v64}/{T64_LOOP_RESIDUAL} documented loop-control branches");
}

// ===================== anti-vacuity: single-bit-flip mutant =====================

/// Build a /tmp sandbox twin: a standalone binary that differential-tests
/// the REAL transpose.rs (via #[path]) against the naive oracle over a
/// fixed corpus. `mutate` flips ONE bit in a stage mask of the /tmp COPY
/// (never the shared tree). Returns (exit_code, twin_dir).
fn run_twin(mutate: bool) -> i32 {
    let dir = format!(
        "/tmp/ts4-{}-{}",
        if mutate { "mutant" } else { "pristine" },
        std::process::id()
    );
    let _ = fs::create_dir_all(dir.as_str());
    let mod_src = Path::new(dir.as_str()).join("transpose.rs");
    let orig = fs::read_to_string(Path::new(CRATE_DIR).join("src/transpose.rs"))
        .expect("read transpose.rs");
    if mutate {
        // M-TS4-1: single-bit flip in the stage-7 mask (0x...AA -> 0x...A8).
        let needle = "0x00AA_00AA_00AA_00AA";
        let flipped = "0x00AA_00AA_00AA_00A8";
        let mutated = orig.replace(needle, flipped);
        assert_ne!(mutated, orig, "M-TS4-1 mutation needle not found");
        fs::write(&mod_src, mutated).unwrap();
    } else {
        fs::write(&mod_src, &orig).unwrap();
    }
    // twin main: differential over boundary + splitmix corpus; exit 1 on
    // any mismatch.
    let main = Path::new(dir.as_str()).join("twin_main.rs");
    fs::write(
        &main,
        format!(
            "#[path = \"{dir}/transpose.rs\"] mod transpose;\n\
             use transpose::{{transpose_8x8, transpose_64x64}};\n\
             fn oracle8(x: u64) -> u64 {{\n\
             \x20   let mut out = 0u64;\n\
             \x20   for r in 0..8u32 {{ for c in 0..8u32 {{\n\
             \x20       let b = (x >> (r * 8 + c)) & 1;\n\
             \x20       out |= b << (c * 8 + r);\n\
             \x20   }} }}\n\
             \x20   out\n\
             }}\n\
             fn oracle64(rows: [u64; 64]) -> [u64; 64] {{\n\
             \x20   let mut out = [0u64; 64];\n\
             \x20   for r in 0..64usize {{ for c in 0..64u32 {{\n\
             \x20       out[c as usize] |= ((rows[r] >> c) & 1) << r;\n\
             \x20   }} }}\n\
             \x20   out\n\
             }}\n\
             fn main() {{\n\
             \x20   let mut xs: Vec<u64> = vec![0, u64::MAX, 1, 0xAA, 0x55];\n\
             \x20   let mut s: u64 = 0xA20C_0FFE_E5A7_E000;\n\
             \x20   for _ in 0..4096 {{\n\
             \x20       s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);\n\
             \x20       let mut z = s;\n\
             \x20       z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);\n\
             \x20       z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);\n\
             \x20       xs.push(z ^ (z >> 31));\n\
             \x20   }}\n\
             \x20   for x in &xs {{\n\
             \x20       assert_eq!(transpose_8x8(*x), oracle8(*x), \"8x8 mutant mismatch\");\n\
             \x20   }}\n\
             \x20   let mut s2: u64 = 0xA20C_0FFE_E5A7_E000 ^ 0x5EED;\n\
             \x20   for _ in 0..64 {{\n\
             \x20       let mut m = [0u64; 64];\n\
             \t     for w in m.iter_mut() {{ *w = {{ s2 = s2.wrapping_add(0x9E37_79B9_7F4A_7C15);\n\
             \x20           let mut z = s2;\n\
             \x20           z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);\n\
             \x20           z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);\n\
             \x20           z ^ (z >> 31) }}; }}\n\
             \x20       assert_eq!(transpose_64x64(m), oracle64(m), \"64x64 mutant mismatch\");\n\
             \x20   }}\n\
             \x20   println!(\"TWIN-PASS\");\n\
             }}\n"
        ),
    )
    .unwrap();
    let exe = Path::new(dir.as_str()).join("twin");
    let o = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
        .args(["--edition", "2021", "-C", "opt-level=2", "-o"])
        .args([&exe])
        .arg(&main)
        .output()
        .expect("spawn rustc twin");
    assert!(
        o.status.success(),
        "twin compile failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let run = Command::new(&exe).output().expect("run twin");
    run.status.code().expect("twin exit code")
}

#[test]
fn anti_vacuity_mutant_flips_differential_court() {
    // Baseline: pristine twin MUST pass (exit 0).
    let pristine = run_twin(false);
    assert_eq!(pristine, 0, "pristine twin must pass, got exit {pristine}");
    println!("[PASS] pristine twin differential: exit 0");
    // Mutant: single-bit-flip twin MUST FAIL (exit nonzero) — a court
    // that cannot flip is FAIL.
    let mutant = run_twin(true);
    assert_ne!(mutant, 0, "[VACUOUS COURT] M-TS4-1 mask bit-flip did NOT flip the differential court (mutant exit {mutant})");
    println!("[PASS] M-TS4-1 single-bit-flip mutant: differential court FLIPPED (exit {mutant})");
}

#[test]
fn census_host_hot_cells() {
    census_target("host", "aarch64", &[]);
}

#[test]
fn census_riscv32imc_hot_cells() {
    let ok = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("riscv32imc-unknown-none-elf"))
        .unwrap_or(false);
    if !ok {
        println!("[SKIP] riscv32imc target not installed — host-only census");
        return;
    }
    census_target(
        "riscv32imc",
        "riscv32",
        &["--target", "riscv32imc-unknown-none-elf"],
    );
}
