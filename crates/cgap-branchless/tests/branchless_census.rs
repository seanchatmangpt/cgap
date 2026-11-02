//! REP1 in-test assembly census for `cgap-branchless` (FA6 closure).
//!
//! Runs as REAL `#[test]` fns under `cargo test -p cgap-branchless`:
//! every cell asserts final state and PANICS on a BRANCHY/MISSING cell,
//! so `cargo test` exit 0 means census-passed. Compiles the crate's
//! modules with nightly rustc into /tmp objects and censuses the real
//! llvm-objdump disassembly.
//!
//! Cell classes (honest grid, no fake 19/19):
//! - HOT: 0-branch required. Mutation that flips the court:
//!   M-REP1-1 (if-injection into select.rs sandbox copy).
//! - PASS-RESIDUAL-1 (select_slice only): exactly ONE documented branch
//!   allowed — the empty-mask-default empty-check `cbz` (see select.rs
//!   module docs; the unsafe mask form that removes it is blocked by
//!   crate-level forbid(unsafe_code), src/lib.rs:16).
//! - REFERENCE-DRIVER (loud SKIP label, symbol presence REQUIRED):
//!   loops/alloc/panic-refusal by design (eytzinger_search,
//!   to_eytzinger_layout, checked_uniform_select).
//!
//! INL CALLER cells (FA6 F6-INL gap closure): `#[inline]`/
//! `#[inline(always)]` primitives censused as isolated symbols leave
//! inline CALL SITES invisible. Canonical CALLER probes compile the
//! real crate modules and inline the primitives into representative
//! hot loops; the CALLER symbols are censused per target:
//! - caller_filter_lane  (cmov_u64 + pred_inc per lane, 8-wide):
//!   HOT — exact 0 branches on host-aarch64 AND riscv32imc (the
//!   caller's loop is unrolled over the const LANE with no bounds
//!   checks, so the documented loop back-edge budget is 0).
//! - caller_swar_scan    (swar_eq_u64 + swar_splat per lane, 8-wide):
//!   HOT — exact 0 branches on both targets.
//! - caller_eytzinger_probe (eytzinger_next in-loop + eytzinger_restore
//!   tail): documented residual — the caller's OWN traversal-loop
//!   branches, bounds-check guard and panic call (plus the rv32
//!   eytzinger_restore shift guard), not primitive branches.
//! The caller inventory is printed per run so future callers are added
//! to `CALLER_GRID` explicitly.
//!
//! Anti-vacuity arms: (a) branchy reference `naive_if_cmov` MUST be
//! flagged BRANCHY; (b) mutation M-REP1-1 of a select.rs sandbox copy
//! MUST flip select_slice past its residual allowlist. A court that
//! passes while either fails to flip exits as VACUOUS (test panics).
//! (c) M-INL-1: a data-dependent branch injected into a CALLER probe
//! sandbox copy MUST flip the caller's HOT cell off 0. A caller court
//! that still reads 0-branch under the mutant is VACUOUS.
//!
//! Scanned mnemonic classes: aarch64 (b/bl/br/blr/cbz/cbnz/tbz/tbnz/
//! b.*), riscv32 (beq/bne/blt/bge/bltu/bgeu/beqz/bnez/bgez/bltz/bgtz/
//! blez/j/jal/jalr AND compressed c.beqz/c.bnez/c.j/c.jal/c.jr/c.jalr
//! — the FA6 classifier gap, now closed), x86_64 (j*/call).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const CRATE_DIR: &str = "/Users/sac/cgap/crates/cgap-branchless";
// G10 root-cause fix: per-process WORK so concurrent census invocations
// (workspace run + lane run) cannot race on shared obj artifacts.
fn work() -> &'static str {
    static WORK: OnceLock<String> = OnceLock::new();
    WORK.get_or_init(|| format!("/tmp/rep1-census-{}", std::process::id()))
}

fn tools() -> &'static (String, PathBuf) {
    static T: OnceLock<(String, PathBuf)> = OnceLock::new();
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
        let host_bin = fs::read_dir(format!("{sysroot}/lib/rustlib"))
            .expect("sysroot lib/rustlib")
            .filter_map(|e| e.ok())
            .find(|e| e.path().join("bin/llvm-objdump").exists())
            .map(|e| e.path().join("bin/llvm-objdump"))
            .expect("llvm-objdump in nightly sysroot");
        (sysroot, host_bin)
    })
}

fn run_cmd(program: &str, args: &[&str]) -> (bool, String) {
    match Command::new(program).args(args).output() {
        Ok(o) => {
            let mut t = String::from_utf8_lossy(&o.stdout).into_owned();
            t.push_str(&String::from_utf8_lossy(&o.stderr));
            (o.status.success(), t)
        }
        Err(e) => (false, format!("spawn failed: {e}")),
    }
}

fn compile_obj(src: &Path, out: &Path, extra: &[&str], opt: &str) -> bool {
    let rustc = format!("{}/bin/rustc", tools().0);
    let mut args: Vec<String> = vec!["--edition", "2021", "--crate-type", "lib"]
        .into_iter()
        .map(String::from)
        .collect();
    args.extend(
        [
            "-C",
            opt,
            "-C",
            "debuginfo=0",
            "-C",
            "llvm-args=--inline-threshold=0",
            "-C",
            "link-dead-code=yes",
            "--emit=obj",
            "-o",
            &out.to_str().unwrap(),
        ]
        .into_iter()
        .map(String::from),
    );
    args.extend(extra.iter().map(|s| String::from(*s)));
    args.push(src.to_str().unwrap().to_string());
    let argrefs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (ok, text) = run_cmd(&rustc, &argrefs);
    if !ok {
        eprintln!("[COMPILE-FAIL] {}\n{}", src.display(), text);
    }
    ok
}

fn write_wrapper(path: &Path, modules: &[&str], no_std: bool) {
    let mut body = String::new();
    if no_std {
        body.push_str("#![no_std]\n");
    }
    body.push_str("#![allow(dead_code)]\n");
    for m in modules {
        body.push_str(&format!(
            "#[path = \"{CRATE_DIR}/src/{m}\"] pub mod {};\n",
            m.trim_end_matches(".rs")
        ));
    }
    fs::write(path, body).expect("write wrapper");
}

fn disassemble(objdump: &Path, obj: &Path) -> Vec<(String, Vec<String>)> {
    let (ok, text) = run_cmd(
        objdump.to_str().unwrap(),
        &["-d", "--no-show-raw-insn", obj.to_str().unwrap()],
    );
    if !ok {
        panic!("[OBJDUMP-FAIL] {}", obj.display());
    }
    let mut blocks: Vec<(String, Vec<String>)> = Vec::new();
    let mut cur: Option<(String, Vec<String>)> = None;
    for line in text.lines() {
        if let Some(open) = line.find('<') {
            if let Some(close) = line.find(">:") {
                if open < close {
                    if let Some(c) = cur.take() {
                        blocks.push(c);
                    }
                    cur = Some((line.trim().to_string(), Vec::new()));
                    continue;
                }
            }
        }
        if let Some((_, lines)) = cur.as_mut() {
            lines.push(line.to_string());
        }
    }
    if let Some(c) = cur.take() {
        blocks.push(c);
    }
    blocks
}

fn mnemonic(line: &str) -> Option<&str> {
    let mut toks = line.split_whitespace();
    let _addr = toks.next()?;
    Some(toks.next()?.trim_end_matches(':'))
}

/// riscv32imc classifier INCLUDING the compressed forms FA6 flagged as a
/// gap: c.beqz / c.bnez / c.j / c.jal / c.jr / c.jalr (+ plain beqz /
/// bnez / b<cctype>z / j).
fn is_flow(m: &str, arch: &str) -> bool {
    match arch {
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
                || m.starts_with("blraa")
                || m.starts_with("brasl")
        }
        "riscv32" => {
            let (c, stem) = if let Some(s) = m.strip_prefix("c.") {
                (true, s)
            } else {
                (false, m)
            };
            let base = matches!(
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
            );
            base || (c && matches!(stem, "jr" | "jalr"))
        }
        "x86_64" => m.starts_with('j') || m == "call" || m == "callq",
        _ => false,
    }
}

fn census(objdump: &Path, obj: &Path, arch: &str, fns: &[&str]) -> Vec<(String, String, usize)> {
    let blocks = disassemble(objdump, obj);
    let mut verdicts = Vec::new();
    for f in fns {
        // The REAL fn symbol, not LLVM closure/instantiation clones:
        // block headers starting with closure markers (_RNC / _RINv)
        // are clones; the fn body lives under the _RNv definition.
        let real = blocks
            .iter()
            .filter(|(h, _)| h.contains(f) && !h.contains("_RNC") && !h.contains("_RINv"))
            .last();
        let Some((hdr, lines)) = real else {
            verdicts.push((f.to_string(), "MISSING".to_string(), 0));
            continue;
        };
        let hits: Vec<String> = lines
            .iter()
            .filter_map(|l| mnemonic(l))
            .filter(|m| is_flow(m, arch))
            .map(String::from)
            .collect();
        let v = if hits.is_empty() {
            format!("PASS ({})", hdr.trim())
        } else {
            format!("BRANCHY {}x {:?}", hits.len(), hits)
        };
        verdicts.push((f.to_string(), v, hits.len()));
    }
    verdicts
}

/// (target, module, fn, cell_class) inventory. Cell classes: HOT (0
/// branches), RESIDUAL1 (exactly 1, select_slice's documented empty-mask
/// default), REFERENCE (loud SKIP label, presence required).
const GRID: &[(&str, &[&str], CellClass)] = &[
    ("select.rs", &["cmov_u64"], CellClass::Residual),
    ("select.rs", &["pred_inc"], CellClass::Hot),
    ("select.rs", &["select_slice"], CellClass::Residual),
    (
        "select.rs",
        &["checked_uniform_select"],
        CellClass::Reference("cold-path refusal by design (uniformity scan + loud panic)"),
    ),
    (
        "arithmetic.rs",
        &["saturated_subs_p28", "poison_mask"],
        CellClass::Hot,
    ),
    (
        "swar.rs",
        &[
            "swar_eq_u64",
            "swar_has_zero_byte",
            "swar_range_mask",
            "swar_lt",
            "swar_splat",
        ],
        CellClass::Hot,
    ),
    ("eytzinger.rs", &["eytzinger_next"], CellClass::Hot),
    ("eytzinger.rs", &["eytzinger_restore"], CellClass::Residual),
    (
        "eytzinger.rs",
        &["eytzinger_search", "to_eytzinger_layout"],
        CellClass::Reference(
            "reference drivers: traversal loop back-edge / Vec allocation by design",
        ),
    ),
    ("radix.rs", &["radix_bucket_12"], CellClass::Hot),
];

enum CellClass {
    /// 0 branches required on every target.
    Hot,
    /// Per-target documented residual allowlist (see `allowed`).
    Residual,
    /// Loud SKIP label; symbol presence still REQUIRED.
    Reference(&'static str),
}

/// Documented per-cell residual allowlist (target-inherent; witnesses in
/// repair-01-branchless-census.md):
/// - select_slice riscv32imc = 2: the empty-mask-default check PLUS one
///   bnez select-lowering (witness: the `if` form AND the arithmetic
///   mask form both emit exactly 1 bnez; rv32imc has no Zicond, ISel
///   lowers every 2-way select to a branch; --disable-cgp-select2branch
///   does not change it — witnessed this session).
/// - cmov_u64 riscv32imc = 1 (same select-lowering witness; the
///   arithmetic mask form in select.rs folds back to a select, which
///   ISel lowers to 1 bnez — identical count to the `if` form).
/// - select_slice aarch64 = 1: the empty-mask-default check (the
///   branch-free unsafe mask form is blocked by crate-level
///   forbid(unsafe_code), src/lib.rs:16).
/// - cmov_u64 aarch64 = 0 -> HOT (csel): the mask form in select.rs.
fn allowed(arch: &str, f: &str) -> usize {
    match (arch, f) {
        ("riscv32", "select_slice") => 2,
        ("riscv32", "cmov_u64") => 1,
        // eytzinger_restore riscv32imc = 1: variable-shift guard. riscv
        // shifts do not mask the shift amount; LLVM emits one beqz guard
        // for shift-amount >= width around the magic-multiply
        // count-trailing-ones expansion (witnessed in the riscv obj).
        ("riscv32", "eytzinger_restore") => 1,
        (_, "select_slice") => 1,
        _ => 0,
    }
}

const ALL_MODULES: &[&str] = &[
    "select.rs",
    "arithmetic.rs",
    "swar.rs",
    "eytzinger.rs",
    "radix.rs",
];

/// Census one target: compile the module wrapper, run all grid cells.
fn census_target(label: &'static str, arch: &'static str, extra: &[&str], tag: &str) -> usize {
    let (_, objdump) = tools();
    fs::create_dir_all(work()).unwrap();
    write_wrapper(
        &Path::new(work()).join(format!("wrapper_{tag}.rs")),
        ALL_MODULES,
        true,
    );
    let obj = Path::new(work()).join(format!("obj_{tag}.o"));
    assert!(
        compile_obj(
            &Path::new(work()).join(format!("wrapper_{tag}.rs")),
            &obj,
            extra,
            "opt-level=3"
        ),
        "census compile failed for {label}"
    );
    let mut residual_seen = 0usize;
    let mut hot_cells = 0usize;
    for (module, fns, class) in GRID {
        let _ = module;
        let (fns, class) = (*fns, class);
        for (f, v, hits) in census(objdump, &obj, arch, fns) {
            let lim = allowed(arch, &f);
            match class {
                CellClass::Hot => {
                    assert_eq!(lim, 0, "grid consistency: HOT cell {f} must allow 0");
                    assert_eq!(hits, 0, "[CENSUS-FAIL] {label} HOT cell {f} -> {v}");
                    hot_cells += 1;
                }
                CellClass::Residual => {
                    assert!(
                        hits <= lim,
                        "[CENSUS-FAIL] {label} cell {f} -> {v} exceeds the documented residual allowlist {lim}"
                    );
                    println!("[PASS-RESIDUAL] {label} {f}: {hits}/{lim} documented branch(es)");
                    residual_seen += 1;
                }
                CellClass::Reference(reason) => {
                    assert_ne!(
                        v, "MISSING",
                        "[CENSUS-FAIL] {label} REFERENCE cell {f} symbol MISSING"
                    );
                    println!("[SKIP-REFERENCE] {label} {f}: {reason} (presence asserted)");
                }
            }
        }
    }
    println!(
        "census {label}: {hot_cells} HOT cells 0-branch, {residual_seen} RESIDUAL cells, refs labeled"
    );
    hot_cells
}

/// Build the /tmp SANDBOX TWIN of the unsafe ptr/len mask form of
/// select_slice (the form blocked in-tree by forbid(unsafe_code)) and
/// census it: MUST be 0-branch. This witnesses that the only blocker is
/// the crate-level forbid, not the form itself.
fn census_mask_form_sandbox(tag: &str) {
    let (_, objdump) = tools();
    fs::create_dir_all(work()).unwrap();
    let src = Path::new(work()).join(format!("mask_form_{tag}.rs"));
    fs::write(&src,
        "#![allow(dead_code)]\n\
         #[no_mangle]\n\
         pub unsafe fn select_slice_mask_form<'a>(cond: &[bool], a: &'a [u64], b: &'a [u64]) -> &'a [u64] {\n\
         \x20   // SAFETY: caller contract is a uniform non-empty mask (the\n\
         \x20   // checked wrapper resolves emptiness in the cold path);\n\
         \x20   // raw read of the first bool, branch-free.\n\
         \x20   let pick_a = unsafe { *cond.as_ptr() };\n\
         \x20   let mask = 0usize.wrapping_sub(pick_a as usize);\n\
         \x20   let ptr = ((a.as_ptr() as usize) & mask) | ((b.as_ptr() as usize) & !mask);\n\
         \x20   let len = (a.len() & mask) | (b.len() & !mask);\n\
         \x20   core::slice::from_raw_parts(ptr as *const u64, len)\n\
         }\n").unwrap();
    let obj = Path::new(work()).join(format!("mask_form_{tag}.o"));
    assert!(
        compile_obj(&src, &obj, &[], "opt-level=3"),
        "mask-form twin compile failed"
    );
    for (f, v, hits) in census(objdump, &obj, "aarch64", &["select_slice_mask_form"]) {
        assert_eq!(
            hits, 0,
            "[CENSUS-FAIL] unsafe mask form must be 0-branch -> {v}"
        );
        println!("[PASS] sandbox unsafe mask form {f}: 0 branches (form is branch-free; forbid(unsafe_code) is the sole in-tree blocker)");
    }
}

/// M-REP1-1: inject an extra data-dependent branch into a select.rs
/// sandbox copy; select_slice MUST flip beyond its residual allowlist.
fn census_mutation_m_rep1_1(tag: &str) {
    let (_, objdump) = tools();
    fs::create_dir_all(work()).unwrap();
    let src = Path::new(work()).join(format!("mut_{tag}.rs"));
    let orig =
        fs::read_to_string(Path::new(CRATE_DIR).join("src/select.rs")).expect("read select.rs");
    let needle = "let pick_a = cond.first().map_or(true, |&c| c);";
    let mutated = orig.replace(
        needle,
        "let pick_a = cond.first().map_or(true, |&c| c);\n    if cond.iter().filter(|c| !**c).count() > 0 { return b; }",
    );
    assert_ne!(mutated, orig, "M-REP1-1 injection needle not found");
    fs::write(&src, mutated).unwrap();
    let obj = Path::new(work()).join(format!("mut_{tag}.o"));
    assert!(
        compile_obj(&src, &obj, &[], "opt-level=3"),
        "mutant compile failed"
    );
    let mut flipped = false;
    for (_f, v, hits) in census(objdump, &obj, "aarch64", &["select_slice"]) {
        println!("[MUTANT] select_slice -> {v}");
        if hits > 1 {
            flipped = true;
        }
    }
    assert!(flipped,
        "[VACUOUS COURT] M-REP1-1 mutation did not push select_slice beyond the 1-branch residual — court cannot flip");
    println!("[PASS] anti-vacuity M-REP1-1: mutant flipped to BRANCHY (>1 branch)");
}

/// Classic branchy reference: MUST be flagged BRANCHY (classifier
/// self-check; a court that accepts it is VACUOUS).
fn census_branchy_reference(tag: &str) {
    let (_, objdump) = tools();
    fs::create_dir_all(work()).unwrap();
    let src = Path::new(work()).join(format!("branchy_{tag}.rs"));
    fs::write(
        &src,
        "#![allow(dead_code)]\n\
         #[inline(never)]\n\
         fn arm_a(x: u64) -> u64 { x.wrapping_add(7) }\n\
         #[inline(never)]\n\
         fn arm_b(x: u64) -> u64 { x.wrapping_mul(3) }\n\
         #[no_mangle]\n\
         pub extern \"C\" fn naive_if_cmov(cond: bool, t: u64, f: u64) -> u64 {\n\
         \x20   if cond { arm_a(t) } else { arm_b(f) }\n\
         }\n",
    )
    .unwrap();
    let obj = Path::new(work()).join(format!("branchy_{tag}.o"));
    assert!(
        compile_obj(&src, &obj, &[], "opt-level=1"),
        "branchy ref compile failed"
    );
    for (f, v, hits) in census(objdump, &obj, "aarch64", &["naive_if_cmov"]) {
        assert!(
            hits > 0,
            "[VACUOUS COURT] branchy reference not flagged: {f} -> {v}"
        );
        println!("[PASS] branchy self-check {f}: flagged BRANCHY ({hits} flow instrs)");
    }
}

// ===================== INL: caller-site census (FA6 F6-INL) =====================

/// Caller-probe source: `#[path]`-includes the REAL crate modules
/// (select, swar, eytzinger) and defines the canonical CALLER fns that
/// inline the primitives into representative hot loops. Compiled per
/// target WITHOUT `--inline-threshold=0` (the caller census needs the
/// `#[inline]` hints to actually inline at the call site).
fn caller_probe_src() -> String {
    let mut s = String::from("#![no_std]\n#![allow(dead_code)]\n");
    for m in ["select", "swar", "eytzinger"] {
        s.push_str(&format!(
            "#[path = \"{CRATE_DIR}/src/{m}.rs\"] pub mod {m};\n"
        ));
    }
    s.push_str("use select::{cmov_u64, pred_inc};\n");
    s.push_str("use eytzinger::{eytzinger_next, eytzinger_restore};\n");
    s.push_str("use swar::{swar_eq_u64, swar_splat};\n");
    s.push_str("\nconst LANE: usize = 8;\n");
    s.push_str(
        r#"
#[no_mangle]
pub fn caller_filter_lane(xs: &[u64; LANE], keep: u64) -> u64 {
    let mut hits = 0usize;
    let mut acc = 0u64;
    for i in 0..LANE {
        let x = xs[i];
        let pred = (x & 1) == 0;
        acc = acc.wrapping_add(cmov_u64(pred, x, keep));
        pred_inc(&mut hits, pred);
    }
    acc ^ (hits as u64)
}

#[no_mangle]
pub fn caller_swar_scan(words: &[u64; LANE], byte: u8) -> usize {
    let pat = swar_splat(byte);
    let mut hits = 0usize;
    for i in 0..LANE {
        hits += (swar_eq_u64(words[i], pat) == 0) as usize;
    }
    hits
}

#[no_mangle]
pub fn caller_eytzinger_probe(eyt: &[u64], target: u64) -> usize {
    let m = eyt.len() - 1;
    let mut k = 1usize;
    while k <= m {
        k = eytzinger_next(k, eyt[k] <= target);
    }
    eytzinger_restore(k)
}
"#,
    );
    s
}

/// Caller-cell inventory: (fn, documented residual per target). HOT
/// caller cells (filter_lane, swar_scan) allow exactly 0 branches on
/// every target: the 8-lane loop unrolls bounds-check-free, so the
/// caller's own documented loop back-edge budget is 0 and the inlined
/// primitive bodies must contribute 0 branches beyond it.
/// caller_eytzinger_probe carries the caller's OWN structural residual:
/// host-aarch64 = 4 (loop compare + bounds-check branch + loop
/// back-edge + out-of-bounds panic call), riscv32imc = 9 (same loop +
/// bounds structure plus the eytzinger_restore shift-amount guard).
/// The caller list is recorded here: a NEW caller fn gets a NEW row in
/// `CALLER_GRID` (added by whoever writes the caller).
const CALLER_GRID: &[(&str, fn(&str) -> usize)] = &[
    ("caller_filter_lane", caller_allowed_hot),
    ("caller_swar_scan", caller_allowed_hot),
    ("caller_eytzinger_probe", caller_allowed_eytzinger),
];

fn caller_allowed_hot(arch: &str) -> usize {
    match arch {
        "aarch64" | "riscv32" => 0,
        _ => 0,
    }
}

fn caller_allowed_eytzinger(arch: &str) -> usize {
    match arch {
        "riscv32" => 9,
        _ => 4,
    }
}

/// Census the CALLER symbols per target (compile WITHOUT the
/// inline-threshold=0 flag; assert per CALLER_GRID).
fn census_caller_probes(tag: &str, arch: &'static str, extra: &[&str]) {
    let (_, objdump) = tools();
    fs::create_dir_all(work()).unwrap();
    let src = Path::new(work()).join(format!("callers_{tag}.rs"));
    fs::write(&src, caller_probe_src()).expect("write caller probe src");
    let obj = Path::new(work()).join(format!("callers_{tag}.o"));
    assert!(
        compile_obj_plain(&src, &obj, extra),
        "caller probe compile failed for {tag}/{arch}"
    );
    println!(
        "[INL-CALLER-INVENTORY] {}",
        CALLER_GRID
            .iter()
            .map(|(f, _)| *f)
            .collect::<Vec<_>>()
            .join(", ")
    );
    for (f, allowed) in CALLER_GRID {
        for (cf, v, hits) in census(objdump, &obj, arch, &[*f]) {
            assert_eq!(cf, *f);
            let lim = allowed(arch);
            assert_ne!(
                v, "MISSING",
                "[CENSUS-FAIL] caller {f} symbol MISSING on {arch}"
            );
            assert!(
                hits <= lim,
                "[CENSUS-FAIL] {tag}/{arch} CALLER cell {f} -> {v} exceeds documented caller residual {lim} (inlined primitives leaked branches past the caller's own loop budget)"
            );
            if lim == 0 {
                println!("[PASS-INL-CALLER-HOT] {tag}/{arch} {f}: 0 branches (primitives inline branch-free at the call site)");
            } else {
                println!("[PASS-INL-CALLER-RESIDUAL] {tag}/{arch} {f}: {hits}/{lim} documented caller-own branch(es)");
            }
        }
    }
}

/// rustc WITHOUT --inline-threshold=0 so `#[inline]` hints fire at the
/// caller (the isolated-symbol census uses threshold=0 on purpose; the
/// caller census needs real inlining).
fn compile_obj_plain(src: &Path, out: &Path, extra: &[&str]) -> bool {
    let rustc = format!("{}/bin/rustc", tools().0);
    let mut args: Vec<String> = vec!["--edition", "2021", "--crate-type", "lib"]
        .into_iter()
        .map(String::from)
        .collect();
    args.extend(
        [
            "-C",
            "opt-level=3",
            "-C",
            "debuginfo=0",
            "-C",
            "link-dead-code=yes",
            "--emit=obj",
            "-o",
            &out.to_str().unwrap(),
        ]
        .into_iter()
        .map(String::from),
    );
    args.extend(extra.iter().map(|s| String::from(*s)));
    args.push(src.to_str().unwrap().to_string());
    let argrefs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (ok, text) = run_cmd(&rustc, &argrefs);
    if !ok {
        eprintln!("[COMPILE-FAIL] {}\n{}", src.display(), text);
    }
    ok
}

/// M-INL-1: inject a data-dependent branch with a black_box side effect
/// into a CALLER sandbox copy; the caller_filter_lane HOT cell MUST
/// flip off 0 (black_box is opaque to the optimizer, so the injected
/// branch cannot be select-lowered away).
fn census_caller_mutation_m_inl_1(tag: &str, arch: &'static str, extra: &[&str]) {
    let (_, objdump) = tools();
    fs::create_dir_all(work()).unwrap();
    let src = Path::new(work()).join(format!("callers_mut_{tag}.rs"));
    let orig = caller_probe_src();
    let needle = "        pred_inc(&mut hits, pred);\n";
    let mutated = orig.replace(
        needle,
        &format!("{needle}        if pred {{ core::hint::black_box(x); }}\n"),
    );
    assert_ne!(mutated, orig, "M-INL-1 injection needle not found");
    fs::write(&src, mutated).unwrap();
    let obj = Path::new(work()).join(format!("callers_mut_{tag}.o"));
    assert!(
        compile_obj_plain(&src, &obj, extra),
        "M-INL-1 mutant compile failed for {tag}"
    );
    let mut flipped = false;
    for (f, v, hits) in census(objdump, &obj, arch, &["caller_filter_lane"]) {
        println!("[MUTANT-INL] {tag}/{arch} {f} -> {v}");
        if hits > 0 {
            flipped = true;
        }
    }
    assert!(
        flipped,
        "[VACUOUS COURT] M-INL-1 caller mutation left caller_filter_lane at 0 branches on {arch} — caller court cannot flip"
    );
    println!("[PASS] anti-vacuity M-INL-1 ({tag}/{arch}): caller_filter_lane flipped to BRANCHY");
}

fn riscv_installed() -> bool {
    let (ok, text) = run_cmd("rustup", &["target", "list", "--installed"]);
    ok && text
        .lines()
        .any(|l| l.trim() == "riscv32imc-unknown-none-elf")
}

/// ---- the census runs as REAL tests ----

#[test]
fn census_host_hot_cells_0_branch() {
    census_target("host-aarch64", "aarch64", &[], "host");
    census_caller_probes("host", "aarch64", &[]);
    census_caller_mutation_m_inl_1("host", "aarch64", &[]);
    census_mask_form_sandbox("host");
    census_branchy_reference("host");
    census_mutation_m_rep1_1("host");
}

#[test]
fn census_riscv32imc_hot_cells_0_branch() {
    if !riscv_installed() {
        println!("[SKIP] riscv32imc target not installed — host-only census");
        return;
    }
    census_target(
        "riscv32imc",
        "riscv32",
        &["--target", "riscv32imc-unknown-none-elf"],
        "riscv",
    );
    census_caller_probes(
        "riscv",
        "riscv32",
        &["--target", "riscv32imc-unknown-none-elf"],
    );
    census_caller_mutation_m_inl_1(
        "riscv",
        "riscv32",
        &["--target", "riscv32imc-unknown-none-elf"],
    );
    census_mask_form_sandbox("riscv_twin_aarch64");
    census_mutation_m_rep1_1("riscv_twin_mutation");
}

#[test]
fn census_mutation_and_selfcheck_always_run() {
    // Independent of target inventory: the named mutations + branchy
    // self-check run even when riscv is absent.
    census_branchy_reference("indep");
    census_mutation_m_rep1_1("indep");
    census_caller_probes("indep", "aarch64", &[]);
    census_caller_mutation_m_inl_1("indep", "aarch64", &[]);
}
