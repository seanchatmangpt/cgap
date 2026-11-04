//! Purity gate for `cgap-aoq` (cgap-core `source_scan_gate.rs` model):
//! zero-allocation / no_std / no-sleep source scan over the lib target's
//! registered sources, with poison positive controls — a gate that cannot
//! fail is a VACUOUS GATE and must exit non-zero.
//!
//! Banned classes (E=0 externals + E=0 allocation; no-sleep doctrine):
//! heap allocation paths, std paths, timing assumptions, loud-unimplemented
//! holes, interior mutability without a court.

const BANNED: [(&str, &str); 9] = [
    ("alloc::", "heap allocation path"),
    ("vec!", "heap vector macro"),
    ("Box", "heap pointer type"),
    ("std::", "std path (kernel is core-only)"),
    ("sleep", "no-sleep doctrine: time is not a proxy for state"),
    ("todo!()", "unimplemented hole: use a typed refusal"),
    ("unimplemented!()", "unimplemented hole: use a typed refusal"),
    ("static mut", "unsynchronized interior mutability"),
    ("UnsafeCell", "interior mutability requires a named court"),
];

/// Registered modules of the lib target (must match src/lib.rs `pub mod`).
const REGISTERED: [&str; 12] = [
    "aead.rs",
    "chain.rs",
    "codec.rs",
    "error.rs",
    "generated_aoq_header_layout.rs",
    "header.rs",
    "kem.rs",
    "lib.rs",
    "session.rs",
    // [AF-43 SLA sync 2026-11-04] profile_guard.rs landed in src/ (registered
    // in lib.rs by its lane); adding it to the scanned set.
    "profile_guard.rs",
    // [AB-7 SLA sync 2026-11-03] AB-8 landed verify.rs/writer.rs mid-pass.
    "verify.rs",
    "writer.rs",
];

/// [FLIPPED 2026-11-03 by AB-7, disclosed in ab-7-chain-session.md] The
/// chain.rs collision ledger entry is closed: the header-agnostic rewrite
/// landed and is registered in lib.rs. No known-unregistered sources.
const KNOWN_UNREGISTERED: [&str; 0] = [];

/// Remove line/block comments keeping string/char literals intact (the
/// cgap-core scanner's proven algorithm, verbatim in structure).
fn strip_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    let mut block_depth = 0usize;
    while i < b.len() {
        if block_depth > 0 {
            if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
                block_depth += 1;
                i += 2;
            } else if b[i] == b'*' && i + 1 < b.len() && b[i + 1] == b'/' {
                block_depth -= 1;
                i += 2;
            } else {
                if b[i] == b'\n' {
                    out.push(b'\n');
                }
                i += 1;
            }
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            block_depth = 1;
            i += 2;
            continue;
        }
        if b[i] == b'"' {
            out.push(b'"');
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' && i + 1 < b.len() {
                    out.push(b[i]);
                    out.push(b[i + 1]);
                    i += 2;
                    continue;
                }
                out.push(b[i]);
                let close = b[i] == b'"';
                i += 1;
                if close {
                    break;
                }
            }
            continue;
        }
        if b[i] == b'\'' && i + 1 < b.len() {
            if b[i + 1] == b'\\' {
                out.push(b'\'');
                i += 1;
                while i < b.len() {
                    out.push(b[i]);
                    let c = b[i];
                    i += 1;
                    if c == b'\'' {
                        break;
                    }
                }
                continue;
            }
            if i + 2 < b.len() && b[i + 2] == b'\'' {
                out.push(b[i]);
                out.push(b[i + 1]);
                out.push(b[i + 2]);
                i += 3;
                continue;
            }
            out.push(b'\'');
            i += 1;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Boundary-aware token search: word-shaped tokens must not be glued into
/// longer identifiers (`Boxed`, `asleep`); punctuation-shaped tokens match
/// wherever they occur.
fn contains_token(haystack: &str, token: &str) -> bool {
    let bytes = haystack.as_bytes();
    let t = token.as_bytes();
    let word_start = t.first().is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_');
    let word_end = t.last().is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_');
    let mut from = 0;
    while let Some(pos) = haystack[from..].find(token) {
        let at = from + pos;
        let before_ok = !word_start
            || at == 0
            || (!bytes[at - 1].is_ascii_alphanumeric() && bytes[at - 1] != b'_');
        let end = at + t.len();
        let after_ok = !word_end
            || end >= bytes.len()
            || (!bytes[end].is_ascii_alphanumeric() && bytes[end] != b'_');
        if before_ok && after_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// Scan comment-stripped text; returns violating tokens.
fn scan_text(text: &str) -> Vec<&'static str> {
    let stripped = strip_comments(text);
    BANNED
        .iter()
        .filter(|(token, _)| contains_token(&stripped, token))
        .map(|(token, _)| *token)
        .collect()
}

/// Scan one registered file; violation lines name the file (fail-loud).
fn scan_file(path: &std::path::Path, text: &str) -> Vec<String> {
    scan_text(text)
        .into_iter()
        .map(|token| format!("{}: banned token {token:?}", path.display()))
        .collect()
}

#[test]
fn gate_exits_clean_on_registered_sources() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let src = std::path::Path::new(manifest).join("src");
    let mut violations = Vec::new();
    let mut scanned = 0usize;
    for name in &REGISTERED {
        let path = src.join(name);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("registered source under {name} missing: ({e})"));
        violations.extend(scan_file_exempt(&text, name));
        scanned += 1;
    }
    // Loud accounting: every .rs file under src/ must be either registered
    // or on the known unregistered ledger — anything else REFUSES.
    for entry in std::fs::read_dir(&src).expect("read src dir") {
        let path = entry.expect("dirent").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).expect("utf8 name");
        assert!(
            REGISTERED.contains(&name) || KNOWN_UNREGISTERED.contains(&name),
            "UNREGISTERED SOURCE (fail-closed): {name} is under src/ but is \
             neither a registered module nor on the collision ledger"
        );
    }
    assert_eq!(scanned, REGISTERED.len());
    assert!(
        violations.is_empty(),
        "SOURCE_SCAN_GATE_FAIL:\n{}",
        violations.join("\n")
    );
}

#[test]
fn gate_fails_on_injected_poison_file() {
    // Anti-vacuity, file level: a poisoned .rs dropped into the scanned
    // set must be found AND named by path.
    let dir = std::env::temp_dir().join(format!("cgap-aoq-gate-inject-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let clean = dir.join("clean.rs");
    std::fs::write(&clean, "pub fn f() -> u8 { 5 }\n").expect("write clean");
    let injected = dir.join("injected_vec.rs");
    std::fs::write(
        &injected,
        "pub fn g() -> Vec<u8> {\n    let v = vec![0u8; 8];\n    v\n}\n",
    )
    .expect("write poison");
    let mut violations = Vec::new();
    for name in ["clean.rs", "injected_vec.rs"] {
        let p = dir.join(name);
        let t = std::fs::read_to_string(&p).expect("read");
        violations.extend(scan_file(&p, &t));
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(violations.len(), 1, "exactly the poison file must trip");
    assert!(
        violations[0].contains("injected_vec.rs") && violations[0].contains("vec!"),
        "VACUOUS GATE: poison not named by path: {violations:?}"
    );
}

#[test]
fn gate_positive_control_poison_must_flip() {
    // Anti-vacuity: the scanner must flag EVERY banned class on poisoned
    // text; a gate that accepts the poison is vacuous.
    let poison = concat!(
        "use std::vec; ",
        "let v = vec![0u8; 4]; ",
        "let b = Box::new(0u8); ",
        "extern crate alloc; ",
        "let p = std::alloc::alloc(layout); ",
        "core::ptr::NonNull::new_unchecked(1usize as *mut u8); ",
        "thread::sleep(dur); ",
        "todo!(); ",
        "unimplemented!(); ",
        "static mut S: u8 = 0; ",
        "struct Q(core::cell::UnsafeCell<u8>); "
    );
    let hits = scan_text(poison);
    let missing: Vec<&str> = BANNED
        .iter()
        .filter(|(token, _)| !hits.contains(token))
        .map(|(token, _)| *token)
        .collect();
    assert_eq!(missing, Vec::<&str>::new(), "missed {missing:?}");
    assert_eq!(hits.len(), BANNED.len(), "partial flip: {hits:?}");
}

#[test]
fn gate_comment_stripping_is_not_a_hole() {
    // Tokens inside comments must not trip; the SAME tokens in code must.
    let commented =
        "// vec![0u8]; Box::new(1); std::mem; sleep(1); todo!(); static mut\nlet x = 5;";
    assert!(scan_text(commented).is_empty(), "comment tokens must not trip");
    let in_code = "let y = 5; Box::new(1);";
    assert_eq!(scan_text(in_code), vec!["Box"]);
}

#[test]
fn word_boundary_glue_does_not_mask_or_launder() {
    // `Boxed`/`asleep` are NOT `Box`/`sleep` (no false positive); a real
    // `Box` glued into a longer name still cannot hide a bare one.
    assert!(scan_text("let boxed_val = Boxed::new();").is_empty());
    assert!(scan_text("fn asleep() {}").is_empty());
    assert_eq!(scan_text("let p = Box::leak();"), vec!["Box"]);
}

#[test]
fn chain_rs_collision_ledger_is_closed_and_registered() {
    // [FLIPPED 2026-11-03 by AB-7] Original court asserted chain.rs was
    // unregistered pending the coordinator rewrite; the header-agnostic
    // rewrite (R-H1) landed and is registered — the ledger closes. The
    // flip itself is the named transition this court existed to gate.
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let chain = src.join("chain.rs");
    assert!(
        chain.exists(),
        "AB-7 chain.rs expected on disk (collision ledger closed)"
    );
    let lib = std::fs::read_to_string(src.join("lib.rs")).expect("read lib.rs");
    let registered_chain = lib
        .lines()
        .any(|l| l.trim() == "pub mod chain;");
    assert!(
        registered_chain,
        "LEDGER NOT CLOSED: chain.rs exists but is not registered in lib.rs"
    );
}

/// Parse `SOURCE-SCAN-EXEMPT: <token> : <justification >= 20 chars>` from
/// RAW text (cgap-core model): a valid marker exempts ONE token in ONE
/// file; an invalid marker is itself a violation (INVALID-EXEMPTION).
fn parse_exemptions(text: &str) -> Vec<(&'static str, String, bool)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim().trim_start_matches('/').trim();
        let Some(rest) = line.strip_prefix("SOURCE-SCAN-EXEMPT:") else {
            continue;
        };
        let rest = rest.trim();
        let Some((tok, just)) = rest.split_once(" : ") else {
            out.push(("", String::new(), false));
            continue;
        };
        let tok = tok.trim();
        let just = just.trim().to_string();
        let known = BANNED.iter().find(|(t, _)| *t == tok).map(|(t, _)| *t);
        let valid = known.is_some() && just.len() >= 20;
        out.push((known.unwrap_or(""), just, valid));
    }
    out
}

/// Scan one registered file honoring valid exemption markers; invalid
/// markers are reported, never silently accepted.
fn scan_file_exempt(text: &str, name: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let mut exempt: Vec<&'static str> = Vec::new();
    for (tok, just, valid) in parse_exemptions(text) {
        if valid {
            exempt.push(tok);
        } else {
            violations.push(format!(
                "{name}: INVALID-EXEMPTION: SOURCE-SCAN-EXEMPT: {tok} : {just}"
            ));
        }
    }
    for token in scan_text(text) {
        if !exempt.contains(&token) {
            violations.push(format!("{name}: banned token {token:?}"));
        }
    }
    violations
}

#[test]
fn exemption_requires_justification_and_known_token() {
    let marker_bad = "// SOURCE-SCAN-EXEMPT: alloc:: : fixme\nlet x = 1;";
    let v = scan_file_exempt(marker_bad, "t.rs");
    assert!(
        v.iter().any(|s| s.contains("INVALID-EXEMPTION")),
        "sub-20-char justification must be flagged: {v:?}"
    );
    let marker_unknown = "// SOURCE-SCAN-EXEMPT: frobnicate : fully justified long enough text\n";
    let v = scan_file_exempt(marker_unknown, "t.rs");
    assert!(
        v.iter().any(|s| s.contains("INVALID-EXEMPTION"),
        ),
        "unknown-token marker must be flagged: {v:?}"
    );
    let marker_good =
        "// SOURCE-SCAN-EXEMPT: alloc:: : path-included test-harness allocator, tests-only collaborator\n";
    let v = scan_file_exempt(marker_good, "t.rs");
    assert!(v.is_empty(), "valid marker on clean source must stay clean: {v:?}");
}

#[test]
fn exemption_is_scoped_not_a_blanket_pass() {
    let src = concat!(
        "// SOURCE-SCAN-EXEMPT: alloc:: : path-included test-harness allocator, tests-only collaborator\n",
        "let v = vec![0u8; 1];\n",
    );
    let hits = scan_file_exempt(src, "t.rs");
    assert!(
        hits.iter().any(|s| s.contains("vec!")),
        "unmarked tokens must still trip alongside a valid exemption: {hits:?}"
    );
    assert!(
        !hits.iter().any(|s| s.contains("alloc::")),
        "marked token must be exempted: {hits:?}"
    );
}
