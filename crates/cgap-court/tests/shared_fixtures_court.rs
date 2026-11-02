//! shared_fixtures_court (lane DX-6) — static classifier over the repo
//! tree proving zero fixed-shared-WRITE /tmp scratch dirs remain in test
//! sources (G10 root cause: a fixed /tmp/rep1-census WORK dir raced
//! across concurrent cargo invocations and false-FAILED a gate).
//!
//! Anti-vacuity: the in-tree subject must contain ZERO violations; the
//! classifier's teeth are proven by twins that run the SAME classifier
//! over embedded samples (a classifier that passes the violating sample
//! is FAIL — vacuous gate).
//!
//! Violation rule: a non-comment line containing a "/tmp/..." literal
//! that (a) names a scratch dir the test may create/write (const-str
//! binding, create_dir_all, fs::write, PathBuf/Path::new shapes),
//! (b) is NOT keyed on std::process::id, and (c) is NOT an inert label
//! seam (constructor stores the string, no fs use — Driver::new).

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    // tests/ sits at crates/cgap-court/tests/ under the repo root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// One classifier finding: file:line + trimmed offending line.
#[derive(Debug)]
struct Finding {
    loc: String,
}

fn is_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

/// Classify one line. Returns Some(loc) if the line violates the law.
fn classify_line(file: &str, lineno: usize, line: &str) -> Option<String> {
    if is_comment(line) || !line.contains("\"/tmp/") {
        return None;
    }
    // (b) per-process keyed scratch is safe (the G10 fix shape).
    if line.contains("process::id") {
        return None;
    }
    // (c) inert label seam: constructor stores the path, never touches
    // fs (witnessed: cgap-git mcp_driver Driver stores the string).
    if line.contains("Driver::new(") {
        return None;
    }
    // (a) only write-target shapes are violations.
    let const_work = line.contains("const ") && line.contains(": &str");
    let shape = const_work
        || line.contains("create_dir_all")
        || line.contains("fs::write")
        || line.contains("PathBuf::from(\"/tmp/")
        || line.contains("Path::new(\"/tmp/");
    if shape {
        Some(format!("{file}:{lineno}: {}", line.trim()))
    } else {
        None
    }
}

/// Walk crates/*/tests/*.rs and classify every line.
fn scan_tree(root: &Path) -> Vec<Finding> {
    let mut out = Vec::new();
    let crates = root.join("crates");
    for entry in fs::read_dir(&crates).expect("crates dir") {
        let entry = entry.expect("crates entry");
        let tests = entry.path().join("tests");
        if !tests.is_dir() {
            continue;
        }
        for t in fs::read_dir(&tests).expect("tests dir") {
            let t = t.expect("tests entry");
            let p = t.path();
            if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                scan_file(&p, &mut out);
            }
        }
    }
    out
}

fn scan_file(path: &Path, out: &mut Vec<Finding>) {
    let text = fs::read_to_string(path).expect("read test file");
    let name = path
        .strip_prefix(repo_root())
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    for (i, line) in text.lines().enumerate() {
        if let Some(loc) = classify_line(&name, i + 1, line) {
            out.push(Finding { loc });
        }
    }
}

/// THE COURT: zero fixed-shared-write /tmp patterns in the tree.
#[test]
fn court_zero_fixed_shared_write_tmp_dirs_in_tree() {
    let findings = scan_tree(&repo_root());
    assert!(
        findings.is_empty(),
        "[FIXTURE-RACE] fixed-shared-write /tmp patterns remain (G10 class):\n{}",
        findings
            .iter()
            .map(|f| format!("  {}", f.loc))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Teeth twin 1 (anti-vacuity): the SAME classifier MUST flag the
/// violating sample — a classifier that passes it is FAIL (vacuous).
#[test]
fn court_classifier_flags_violating_sample() {
    // Sample built via concat to avoid a contiguous "/" + "tmp/" literal
    // self-flagging inside this court file (the classifier would
    // otherwise flag its own teeth-twin source).
    let tmp = concat!("/tm", "p/fixed-shared-census");
    let sample = format!("let _ = fs::create_dir_all({:?});", tmp);
    let mut hits = Vec::new();
    for (i, line) in sample.lines().enumerate() {
        if let Some(loc) = classify_line("embedded-sample.rs", i + 1, line) {
            hits.push(loc);
        }
    }
    assert!(
        !hits.is_empty(),
        "VACUOUS CLASSIFIER: violating sample passed unflagged"
    );
    assert!(hits[0].contains("/tmp/fixed-shared-census"));
}

/// Teeth twin 2: the sanctioned G10 per-process fix shape must NOT be
/// flagged (the classifier must not bite the cure).
#[test]
fn court_per_process_pattern_not_flagged() {
    let line = "WORK.get_or_init(|| format!(\"/tmp/rep1-census-{}\", std::process::id()))";
    assert!(classify_line("x.rs", 1, line).is_none());
}
