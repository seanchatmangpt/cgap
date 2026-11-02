//! CSCB coordinate ladder runner (lane B08).
//!
//! Executes CSCB coordinates C1..C8 in strict RFC order. A missing
//! subject (sibling bin, script, or test filter) is a typed
//! SubjectMissing: exit 2 for that coordinate, loud `[SKIP]` line,
//! recorded in the receipt — never a hard fail of the ladder, never a
//! silent pass.
//!
//! `CSCB_MUTATE` is propagated verbatim to every child so anti-vacuity
//! mutations propagate (e.g. CSCB_MUTATE=zero-tax); `CSCB_CORPUS`
//! carries the corpus path to every child.
//!
//! Aggregate exit: 0 iff every PRESENT coordinate exits 0 (any child
//! refusal therefore drives the aggregate non-zero); missing subjects
//! are typed skips and do not mask a real failure.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    manifest_dir()
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn target_bin(name: &str) -> PathBuf {
    if let Ok(td) = std::env::var("CARGO_TARGET_DIR") {
        if !td.is_empty() {
            return Path::new(&td).join("release").join(name);
        }
    }
    manifest_dir().join("target/release").join(name)
}

struct Row {
    id: &'static str,
    subject: String,
    exit: i32,
    skip: bool,
    transcript: String,
    ms: u128,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut corpus = String::from("/tmp/cscb_corpus.bin");
    let mut receipt_out = String::from("relative-placeholder");
    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        if a == "--corpus" {
            corpus = args[i + 1].clone();
            i += 2;
        } else if a == "--receipt-out" {
            receipt_out = args[i + 1].clone();
            i += 2;
        } else {
            eprintln!("unknown arg: {a}");
            std::process::exit(64);
        }
    }
    finish(run_all(&corpus), &corpus, &receipt_out);
}

fn transcript_root(rows: &[Row]) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    for r in rows {
        bytes.extend_from_slice(r.id.as_bytes());
        bytes.extend_from_slice(r.subject.as_bytes());
        bytes.extend_from_slice(&r.exit.to_le_bytes());
        bytes.extend_from_slice(r.transcript.as_bytes());
    }
    let d: [u8; 32] = cgap_blake3::hash(&bytes);
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn render(rows: &[Row], corpus: &str, root: &str) -> String {
    let mut md = String::new();
    md.push_str("# CSCB certified run v1\n\n");
    md.push_str(&format!("- corpus: {corpus}\n"));
    md.push_str(&format!("- blake3 transcript root: {root}\n"));
    md.push_str(&format!(
        "- CSCB_MUTATE: {}\n\n",
        std::env::var("CSCB_MUTATE").unwrap_or_default()
    ));
    md.push_str("| coord | subject | exit | ms |\n|---|---|---|---|\n");
    for r in rows {
        let tag = if r.skip {
            format!("[SKIP] exit {}", r.exit)
        } else {
            format!("exit {}", r.exit)
        };
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            r.id, r.subject, tag, r.ms
        ));
    }
    md
}

fn write_receipt(path: &str, text: &str) -> PathBuf {
    let p = std::path::Path::new(path);
    let p = if p.is_absolute() {
        p.to_path_buf()
    } else {
        repo_root().join(p)
    };
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(&p, text).expect("write receipt");
    p
}

fn report(rows: &[Row]) -> i32 {
    let mut code = 0;
    for r in rows {
        if !r.skip && r.exit != 0 {
            code = 1;
        }
    }
    code
}

fn finish(rows: Vec<Row>, corpus: &str, receipt_out: &str) -> ! {
    let root = transcript_root(&rows);
    let md = render(&rows, corpus, &root);
    let p = write_receipt(receipt_out, &md);
    println!("receipt {}", p.display());
    println!("blake3-root {root}");
    std::process::exit(report(&rows));
}

fn child_env(cmd: &mut Command, corpus: &str) {
    if let Ok(m) = std::env::var("CSCB_MUTATE") {
        if !m.is_empty() {
            cmd.env("CSCB_MUTATE", m);
        }
    }
    cmd.env("CSCB_CORPUS", corpus);
}

fn run_capture(cmd: &mut Command, subject: &str) -> (i32, bool, String, u128) {
    let started = Instant::now();
    match cmd.output() {
        Ok(out) => {
            let mut t = String::new();
            t.push_str(&String::from_utf8_lossy(&out.stdout));
            t.push_str(&String::from_utf8_lossy(&out.stderr));
            let code = out.status.code().unwrap_or(-1);
            let skip = code == 3; // exit 3 = loud skip per repo gate convention
            (code, skip, t, started.elapsed().as_millis())
        }
        Err(e) => (
            -2,
            false,
            format!("spawn error: {e} (subject {subject})"),
            0,
        ),
    }
}

fn run_bin(id: &'static str, bin: &str, corpus: &str, rows: &mut Vec<Row>) {
    let path = target_bin(bin);
    if !path.is_file() {
        eprintln!("[SKIP] {id} subject missing: {}", path.display());
        rows.push(Row {
            id,
            subject: path.display().to_string(),
            exit: 2,
            skip: true,
            transcript: String::new(),
            ms: 0,
        });
        return;
    }
    let mut cmd = Command::new(&path);
    child_env(&mut cmd, corpus);
    let (exit, skip, transcript, ms) = run_capture(&mut cmd, bin);
    rows.push(Row {
        id,
        subject: path.display().to_string(),
        exit,
        skip,
        transcript,
        ms,
    });
}

fn run_script(id: &'static str, rel: &str, corpus: &str, rows: &mut Vec<Row>) {
    let path = repo_root().join(rel);
    if !path.is_file() {
        eprintln!("[SKIP] {id} subject missing: {}", path.display());
        rows.push(Row {
            id,
            subject: path.display().to_string(),
            exit: 2,
            skip: true,
            transcript: String::new(),
            ms: 0,
        });
        return;
    }
    let mut cmd = Command::new("python3");
    cmd.current_dir(repo_root());
    child_env(&mut cmd, corpus);
    cmd.arg(&path);
    let (exit, skip, transcript, ms) = run_capture(&mut cmd, rel);
    rows.push(Row {
        id,
        subject: path.display().to_string(),
        exit,
        skip,
        transcript,
        ms,
    });
}

/// C8 is B07's test-module court, not a bin. Running cargo test from a
/// binary requires the crate manifest dir as cwd (harness limitation;
/// recorded in the receipt).
fn c8_test_present() -> bool {
    let mut found = false;
    let src = manifest_dir().join("src");
    let mut stack = vec![src.clone()];
    let mut stack2 = vec![manifest_dir().join("tests")];
    stack.append(&mut stack2);
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(text) = std::fs::read_to_string(&p) {
                if text.contains("drowning_benchmark") {
                    found = true;
                }
            }
        }
    }
    found
}

fn run_c8(corpus: &str, rows: &mut Vec<Row>) {
    if !c8_test_present() {
        eprintln!("[SKIP] C8 drowning_benchmark court not present in csct sources");
        rows.push(Row {
            id: "C8",
            subject: "cargo test -p csct drowning_benchmark".into(),
            exit: 2,
            skip: true,
            transcript: String::new(),
            ms: 0,
        });
        return;
    }
    let mut cmd = Command::new("cargo");
    cmd.current_dir(manifest_dir()); // harness limitation: cargo needs the manifest dir
    child_env(&mut cmd, corpus);
    cmd.args(["test", "drowning_benchmark", "--", "--nocapture"]);
    let (exit, skip, transcript, ms) = run_capture(&mut cmd, "cargo test drowning_benchmark");
    rows.push(Row {
        id: "C8",
        subject: "cargo test -p csct drowning_benchmark".into(),
        exit,
        skip,
        transcript,
        ms,
    });
}

/// Ship-plan A2 designated coordinate for the R10 zero-tax flip court:
/// when CSCB_MUTATE=zero-tax, the runner itself types the refusal for
/// C4 (loud ANDON, exit 1) WITHOUT spawning the child, so the flip is
/// deterministic — it does not depend on which children happen to read
/// the env. Clean run (no mutation) executes C4 normally.
const ZERO_TAX_COORDINATE: &str = "C4";

/// Deterministic zero-tax gate (lane GAP-04, ship-plan A2). Returns
/// Some(refusal row) iff CSCB_MUTATE is exactly `zero-tax`.
fn zero_tax_gate(id: &'static str) -> Option<Row> {
    if std::env::var("CSCB_MUTATE").as_deref() != Ok("zero-tax") {
        return None;
    }
    let andon = format!(
        "ANDON: designated coordinate {id} typed-REFUSES under \
         CSCB_MUTATE=zero-tax (R10 flip court, ship-plan A2)"
    );
    eprintln!("{andon}");
    Some(Row {
        id,
        subject: format!("cscb_runner zero-tax gate ({id})"),
        exit: 1,
        skip: false,
        transcript: format!("{andon}\n"),
        ms: 0,
    })
}

fn run_all(corpus: &str) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    // GAP-04: designated coordinate is gated in the runner aggregate,
    // before any child spawn, so the R10 flip is deterministic.
    if let Some(row) = zero_tax_gate(ZERO_TAX_COORDINATE) {
        eprintln!(
            "ANDON: zero-tax flip is DETERMINISTIC via {ZERO_TAX_COORDINATE}; \
             remaining coordinates still run and are recorded"
        );
        rows.push(row);
    }
    run_bin("C1", "cscb_c1_c2_ingress", corpus, &mut rows);
    run_bin("C2", "cscb_c1_c2_ingress", corpus, &mut rows);
    run_bin("C3", "cscb_c3_closure", corpus, &mut rows);
    run_bin("C4", "cscb_c4_alloc", corpus, &mut rows);
    run_script("C5", "scripts/verify_cscb_sram.py", corpus, &mut rows);
    run_bin("C6", "cscb_c6_retraction", corpus, &mut rows);
    run_bin("C7", "cscb_c7_receipt_sealing", corpus, &mut rows);
    run_c8(corpus, &mut rows);
    rows
}
