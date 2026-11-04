//! Unified tri-language drift court: ONE run pins the AOQ frozen status
//! band across all three language tables against cgap-arrow's producer
//! table (`cgap_arrow::status_dictionary::STATUS_BAND`):
//!
//!   1. cgap-ffi `crates/cgap-ffi/src/aoq/status_maps.rs`
//!      (source-parsed `0x... => Some("NAME")` match arms),
//!   2. `include/cgap_aoq.h` (`#define CGAP_AOQ_STATUS_* 0x...u`),
//!   3. arcus `internal/arcus/adapter/aoq_ipc.go` (`AoqStatus*` consts +
//!      the `AoqStatusName` switch), resolved via `$ARCUS_ROOT`
//!      (default: the sibling checkout `../../arcus`).
//!
//! CROSS-REPO PATH LAW (admit_test precedent, inverted): arcus reads cgap
//! via `$CGAP_ROOT` with a loud `SKIPPED-CGAP-ABSENT` skip; this court
//! inverts the direction — `$ARCUS_ROOT` with a loud `SKIPPED-ARCUS-ABSENT`
//! skip. A skip is a disclosed non-observation, never a pass.
//!
//! ANTI-VACUITY: a /tmp sandbox copy of status_maps.rs with one mutated
//! name must parse to a DIFFERENT table — the court provably flips.

#![cfg(test)]
use std::fs;
use std::path::PathBuf;

use cgap_arrow::status_dictionary::STATUS_BAND;

/// cgap-ffi source (word, name) pairs from the `aoq_status_name` arms.
fn parse_status_maps_rs(src: &str) -> Vec<(u32, String)> {
    let mut table = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        // `0x4347A001 => Some("OK"),`
        if t.starts_with("0x") && t.contains("=> Some(\"") {
            let word_hex = t
                .split(" =>")
                .next()
                .expect("line starts with 0x")
                .trim()
                .to_string();
            let word =
                u32::from_str_radix(word_hex.trim_start_matches("0x"), 16)
                    .expect("frozen-band hex word parses");
            let name = t
                .split("Some(\"")
                .nth(1)
                .expect("arm carries Some(\"...\")")
                .split('"')
                .next()
                .expect("closing quote")
                .to_string();
            table.push((word, name));
        }
    }
    table
}

/// C header (word, define-suffix) pairs from CGAP_AOQ_STATUS_* defines.
fn parse_cgap_aoq_h(src: &str) -> Vec<(u32, String)> {
    let mut table = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        // `#define CGAP_AOQ_STATUS_OK 0x4347A001u`
        if let Some(rest) = t.strip_prefix("#define CGAP_AOQ_STATUS_") {
            let mut parts = rest.split_whitespace();
            let suffix = parts.next().expect("define name").to_string();
            let hex = parts.next().expect("define value");
            let word = u32::from_str_radix(hex.trim_end_matches('u').trim_start_matches("0x"), 16)
                .expect("frozen-band hex word parses");
            table.push((word, suffix));
        }
    }
    table
}

/// Go (word, name) pairs: `AoqStatus*` consts joined with the
/// `AoqStatusName` switch (`case AoqStatusX:` / `return "NAME", true`).
fn parse_aoq_ipc_go(src: &str) -> Vec<(u32, String)> {
    let mut consts: Vec<(u32, String)> = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        // `AoqStatusOK              uint32 = 0x4347A001`
        if t.starts_with("AoqStatus") && t.contains("uint32 = 0x") {
            let mut parts = t.split_whitespace();
            let const_name = parts.next().expect("const name").to_string();
            let hex = parts
                .last()
                .expect("const value")
                .trim_start_matches("0x")
                .to_string();
            let word = u32::from_str_radix(&hex, 16).expect("frozen-band hex word parses");
            consts.push((word, const_name));
        }
    }
    // Attach each `return "NAME", true` to the pending `case AoqStatus*`.
    let mut joined: Vec<(u32, String)> = Vec::new();
    let mut pending: Option<String> = None;
    for line in src.lines() {
        let t = line.trim();
        if let Some(go_name) = t.strip_prefix("case AoqStatus") {
            pending = Some(format!("AoqStatus{}", go_name.trim_end_matches(':').trim()));
        } else if let Some(p) = pending.take() {
            if let Some(rest) = t.strip_prefix("return \"") {
                let name = rest.split('"').next().expect("closing quote").to_string();
                let word = consts
                    .iter()
                    .find(|(_, c)| *c == p)
                    .map(|(w, _)| *w)
                    .unwrap_or(0);
                joined.push((word, name));
            }
            // A case whose next line is not a plain return: not part of
            // the status switch; ignored (pending already taken).
        }
    }
    joined
}

/// Arcus checkout root: `$ARCUS_ROOT`, default sibling `../../arcus`.
fn arcus_root() -> Option<PathBuf> {
    if let Ok(r) = std::env::var("ARCUS_ROOT") {
        return Some(PathBuf::from(r));
    }
    let sibling = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../arcus");
    if sibling.join("internal/arcus/adapter/aoq_ipc.go").exists() {
        Some(sibling)
    } else {
        None
    }
}

fn producer_table() -> Vec<(u32, String)> {
    STATUS_BAND
        .iter()
        .map(|(w, n)| (*w, n.to_string()))
        .collect()
}

fn print_matrix(
    rows: &[(String, Vec<(u32, String)>)],
    producer: &[(u32, String)],
) {
    println!("TRI-COURT 8x3 matrix (producer | rust | c | go):");
    for (i, (word, name)) in producer.iter().enumerate() {
        let cell = |t: &[(u32, String)]| -> String {
            match t.get(i) {
                Some((w, n)) if w == word && n == name => "MATCH".to_string(),
                Some((w, n)) => format!("DRIFT({w:#x},{n})"),
                None => "MISSING".to_string(),
            }
        };
        let cols: Vec<String> = rows
            .iter()
                       .map(|(_, t)| cell(t))
            .collect();
        println!(
            "  [{i}] {word:#010x} {name:<17} producer=REF {}",
            cols.join(" | ")
        );
    }
}

#[test]
fn rust_and_c_tables_match_producer() {
    let producer = producer_table();
    assert_eq!(producer.len(), 8);

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().expect("crate parent").parent().expect("repo root");

    let rust_src = fs::read_to_string(
        root.join("crates/cgap-ffi/src/aoq/status_maps.rs"),
    )
    .expect("cgap-ffi status_maps.rs readable from cgap-arrow test");
    let rust_table = parse_status_maps_rs(&rust_src);
    assert_eq!(rust_table.len(), 8, "status_maps.rs arms parsed");

    let c_src = fs::read_to_string(root.join("include/cgap_aoq.h"))
        .expect("include/cgap_aoq.h readable");
    let c_table = parse_cgap_aoq_h(&c_src);
    assert_eq!(c_table.len(), 8, "CGAP_AOQ_STATUS_* defines parsed");

    // The court itself: all three columns equal the producer, in band order.
    assert_eq!(
        rust_table, producer,
        "cgap-ffi status_maps.rs DRIFTED from the producer band"
    );
    assert_eq!(
        c_table, producer,
        "include/cgap_aoq.h DRIFTED from the producer band"
    );

    print_matrix(
        &[
            ("rust".to_string(), rust_table),
            ("c".to_string(), c_table),
        ],
        &producer,
    );
}

#[test]
fn go_table_matches_producer() {
    let producer = producer_table();
    let root = match arcus_root() {
        Some(r) => r,
        None => {
            eprintln!("SKIPPED-ARCUS-ABSENT: set ARCUS_ROOT (default sibling ../../../arcus absent); arcus frozen band not observed this run");
            return;
        }
    };
    let go_src = match fs::read_to_string(root.join("internal/arcus/adapter/aoq_ipc.go")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "SKIPPED-ARCUS-ABSENT: set ARCUS_ROOT (root {:?}); aoq_ipc.go unreadable: {e}",
                root
            );
            return;
        }
    };
    let go_table = parse_aoq_ipc_go(&go_src);
    assert_eq!(go_table.len(), 8, "8 Go status pairs parsed");
    assert_eq!(
        go_table, producer,
        "arcus aoq_ipc.go DRIFTED from the producer band"
    );
}

/// ANTI-VACUITY: the court's parser must flip on a real mutation.
#[test]
fn court_flips_on_mutated_status_maps() {
    let producer = producer_table();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().expect("crate parent").parent().expect("repo root");
    let src = fs::read_to_string(root.join("crates/cgap-ffi/src/aoq/status_maps.rs"))
        .expect("status_maps.rs readable");

    // Mutate exactly one name in a /tmp sandbox copy.
    let mutated = src.replace("Some(\"OK\")", "Some(\"OK_DRIFTED\")");
    assert_ne!(mutated, src, "mutation must apply");
    let dir = std::env::temp_dir().join("cgap_tri_court_sandbox");
    fs::create_dir_all(&dir).expect("sandbox dir");
    let path = dir.join("status_maps.rs");
    fs::write(&path, &mutated).expect("sandbox write");

    let sandbox_table = parse_status_maps_rs(&fs::read_to_string(&path).expect("read back"));
    assert_ne!(
        sandbox_table, producer,
        "VACUOUS COURT: a mutated name still matched the producer band"
    );
    assert_eq!(sandbox_table[0].1, "OK_DRIFTED");

    // Sandbox cleanup: the lease is deleted in the same file that mints it.
    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir(&dir);
}

#[test]
fn court_flips_on_mutated_header() {
    let producer = producer_table();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().expect("crate parent").parent().expect("repo root");
    let src = fs::read_to_string(root.join("include/cgap_aoq.h")).expect("header readable");
    let mutated = src.replace("#define CGAP_AOQ_STATUS_OK 0x4347A001u", "#define CGAP_AOQ_STATUS_OK 0x4347A009u");
    assert_ne!(mutated, src, "mutation must apply");
    let table = parse_cgap_aoq_h(&mutated);
    assert_ne!(table, producer, "VACUOUS COURT: mutated header still matched");
}
