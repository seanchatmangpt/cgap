//! CLI-4 tests: the v26.11.1 crate-surface noun-verb grid.
//!
//! Law (extends tests/zero_code_cli.rs): every admitted surface RESOLVES
//! (`cgap <noun> <verb> --help` exits 0 with usage), every landed handler's
//! smoke path executes against real crate kernels with final-state asserts,
//! and the unknown-surface refusal still fires. Chicago discipline: real
//! binary, real kernels, no mocks.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

const BINARY: &str = env!("CARGO_BIN_EXE_cgap");

/// The 17 v26.11.1 crate-surface pairs admitted by the CLI graph (CLI-1
/// stubs; CLI-2 ported the kernel bodies).
const NEW_PAIRS: &[(&str, &str)] = &[
    ("bont", "slot-lookup"),
    ("bont", "pack-verify"),
    ("bn3", "unify"),
    ("bn3", "eval"),
    ("gate", "dam-ingest"),
    ("parquet", "pack-emit"),
    ("parquet", "pack-check"),
    ("p28", "attest-verify"),
    ("crypto", "seal"),
    ("crypto", "receipt-verify"),
    ("audit", "powl-stats"),
    ("audit", "ocel-stats"),
    ("git", "dag-verify"),
    ("arrow", "slab-stats"),
    ("gpu", "info"),
    ("beam", "info"),
    ("court", "verdict-run"),
];

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the test clock must be after the Unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("cgap-{label}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("create isolated CLI test workspace");
        Self { root }
    }

    fn run(&self, noun: &str, verb: &str, arguments: &[&str]) -> Output {
        let mut command = Command::new(BINARY);
        command
            .current_dir(&self.root)
            .arg(noun)
            .arg(verb)
            .args(arguments);
        command.output().expect("execute generated CLI binary")
    }

    fn json(&self, noun: &str, verb: &str, arguments: &[&str]) -> (Output, Value) {
        let output = self.run(noun, verb, arguments);
        assert!(
            output.status.success(),
            "{noun} {verb} must exit 0; stderr={:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        let parsed: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "{noun} {verb} stdout must be JSON: {error}; stdout={:?}",
                String::from_utf8_lossy(&output.stdout)
            )
        });
        (output, parsed)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn every_new_noun_verb_pair_resolves_to_usage() {
    let workspace = TestWorkspace::new("new-verb-help");
    for (noun, verb) in NEW_PAIRS {
        let output = workspace.run(noun, verb, &["--help"]);
        assert!(
            output.status.success(),
            "{noun} {verb} --help must resolve; stderr={:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Usage"),
            "{noun} {verb} --help must print usage; stdout={stdout:?}"
        );
    }
}

#[test]
fn unknown_surface_refusal_still_fires() {
    let workspace = TestWorkspace::new("unknown-surface");
    let output = workspace.run("ggen-absent-noun", "ggen-absent-verb", &[]);
    assert!(
        !output.status.success(),
        "unknown command surface must fail closed"
    );
    let bad_verb = workspace.run("bont", "not-a-verb", &[]);
    assert!(
        !bad_verb.status.success(),
        "unknown verb on an admitted noun must fail closed"
    );
}

#[test]
fn self_version_reports_manifest_version() {
    let workspace = TestWorkspace::new("self-version");
    let (_, parsed) = workspace.json("self", "version", &[]);
    assert_eq!(
        parsed.as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "self version must echo the manifest version"
    );
}

#[test]
fn align_test_asserts_full_alignment_surface() {
    let workspace = TestWorkspace::new("align-test");
    let (_, parsed) = workspace.json("align", "test", &[]);
    let total = parsed["total"].as_u64().expect("total must be a number");
    let aligned = parsed["aligned"].as_u64().expect("aligned must be a number");
    assert!(total > 0, "align test must probe a nonzero surface");
    assert_eq!(aligned, total, "every probed Buf size must be 64B aligned");
}

#[test]
fn bench_mask_reports_positive_line_rate() {
    let workspace = TestWorkspace::new("bench-mask");
    let (_, parsed) = workspace.json("bench", "mask", &[]);
    let rate = parsed["mask_gib_per_s"]
        .as_f64()
        .expect("mask_gib_per_s must be a number");
    assert!(rate > 0.0, "mask bench must report a positive rate");
}

#[test]
fn bench_parquet_reports_both_kernel_rates() {
    let workspace = TestWorkspace::new("bench-parquet");
    let (_, parsed) = workspace.json("bench", "parquet", &[]);
    let fast = parsed["rle_encode_fast_gib_per_s"]
        .as_f64()
        .expect("fast rate must be a number");
    let base = parsed["rle_encode_gib_per_s"]
        .as_f64()
        .expect("base rate must be a number");
    assert!(fast > 0.0 && base > 0.0, "both RLE kernels must report rates");
}

// ---- v26.11.1 crate-surface smoke courts (CLI-2 handlers) ----------------

#[test]
fn bont_slot_lookup_slot_rederives_independently() {
    let workspace = TestWorkspace::new("bont-slot");
    let (_, parsed) = workspace.json("bont", "slot-lookup", &[]);
    let lexeme = parsed["lexeme"].as_str().expect("lexeme present");
    let slot_hex = parsed["slot"]
        .as_str()
        .expect("slot present")
        .trim_start_matches("0x");
    let slot = u64::from_str_radix(slot_hex, 16).expect("slot parses as hex");
    // Independent re-derivation: BLAKE3 over BONT_NS_V1 ++ lexeme, first 8B LE.
    let mut buf = Vec::with_capacity(cgap_bont::primitives::BONT_NS_V1.len() + lexeme.len());
    buf.extend_from_slice(cgap_bont::primitives::BONT_NS_V1);
    buf.extend_from_slice(lexeme.as_bytes());
    let digest = cgap_blake3::hash(&buf);
    let expected = u64::from_le_bytes(digest[0..8].try_into().expect("32B digest has 8B"));
    assert_eq!(slot, expected, "printed slot must re-derive from the lexeme");
    assert_eq!(parsed["detector_len"].as_u64(), Some(1));
}

#[test]
fn bont_pack_verify_digest_is_64_hex_and_slots_rederive() {
    let workspace = TestWorkspace::new("bont-pack");
    let (_, parsed) = workspace.json("bont", "pack-verify", &[]);
    assert_eq!(parsed["pack_len"].as_u64(), Some(8));
    assert_eq!(parsed["all_slots_rederived"].as_bool(), Some(true));
    let digest = parsed["pack_digest"].as_str().expect("pack_digest present");
    assert_eq!(digest.len(), 64, "BLAKE3 hex digest is 64 chars");
    assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn bn3_unify_binds_all_three_subjects() {
    let workspace = TestWorkspace::new("bn3-unify");
    let (_, parsed) = workspace.json("bn3", "unify", &[]);
    assert_eq!(parsed["matches"].as_u64(), Some(3));
    assert_eq!(parsed["bindings_checked"].as_u64(), Some(3));
    assert_eq!(parsed["var0_bound_to_subjects"].as_bool(), Some(true));
}

#[test]
fn bn3_eval_latches_final_register_state() {
    let workspace = TestWorkspace::new("bn3-eval");
    let (_, parsed) = workspace.json("bn3", "eval", &[]);
    assert_eq!(parsed["sum"].as_u64(), Some(12), "sum register final state");
    assert_eq!(parsed["counters_latched"].as_u64(), Some(3));
    let trace = parsed["trace"].as_array().expect("trace is an array");
    assert_eq!(trace.len(), 1, "one trace write latched");
    assert_eq!(trace[0].as_u64(), Some(12));
}

#[test]
fn gate_dam_ingest_frame_law_holds() {
    let workspace = TestWorkspace::new("gate-dam");
    let (_, parsed) = workspace.json("gate", "dam-ingest", &[]);
    assert_eq!(parsed["frame_len"].as_u64(), Some(160), "128B header + body");
    assert_eq!(parsed["status"], serde_json::json!("0x00000000"));
    assert_eq!(parsed["corrupt_refused"].as_bool(), Some(true));
}

#[test]
fn parquet_pack_emit_verifies_its_own_bytes() {
    let workspace = TestWorkspace::new("parquet-emit");
    let (_, parsed) = workspace.json("parquet", "pack-emit", &[]);
    assert_eq!(parsed["body_len"].as_u64(), Some(128));
    assert_eq!(parsed["rows"].as_u64(), Some(3));
    let digest = parsed["digest"].as_str().expect("digest present");
    assert_eq!(digest.len(), 64);
}

#[test]
fn parquet_pack_check_refuses_tampered_block() {
    let workspace = TestWorkspace::new("parquet-check");
    let (_, parsed) = workspace.json("parquet", "pack-check", &[]);
    assert_eq!(parsed["tamper_refused"].as_bool(), Some(true));
    assert_eq!(parsed["rows"].as_u64(), Some(3));
}

#[test]
fn arrow_slab_stats_roundtrip_is_byte_exact() {
    let workspace = TestWorkspace::new("arrow-slab");
    let (_, parsed) = workspace.json("arrow", "slab-stats", &[]);
    assert_eq!(parsed["pushed"].as_u64(), Some(8));
    assert_eq!(parsed["sealed_bytes"].as_u64(), Some(64 + 64 * 64));
    assert_eq!(parsed["reload_roundtrip"].as_bool(), Some(true));
}

#[test]
fn git_dag_verify_refuses_wrong_parent() {
    let workspace = TestWorkspace::new("git-dag");
    let (_, parsed) = workspace.json("git", "dag-verify", &[]);
    assert_eq!(parsed["blocks"].as_u64(), Some(4));
    assert_eq!(parsed["proof_siblings"].as_u64(), Some(2));
    assert_eq!(parsed["wrong_parent_refused"].as_bool(), Some(true));
}

#[test]
fn audit_powl_stats_conforms_and_refuses_violator() {
    let workspace = TestWorkspace::new("audit-powl");
    let (_, parsed) = workspace.json("audit", "powl-stats", &[]);
    assert_eq!(parsed["conforms"].as_bool(), Some(true));
    assert_eq!(parsed["violating_log_refused"].as_bool(), Some(true));
}

#[test]
fn audit_ocel_stats_refuses_two_corruptions() {
    let workspace = TestWorkspace::new("audit-ocel");
    let (_, parsed) = workspace.json("audit", "ocel-stats", &[]);
    assert_eq!(parsed["events"].as_u64(), Some(5));
    assert_eq!(parsed["monotonic_id_refused"].as_bool(), Some(true));
    assert_eq!(parsed["reserved_activity_refused"].as_bool(), Some(true));
}

#[test]
fn p28_attest_verify_wire_is_80_bytes() {
    let workspace = TestWorkspace::new("p28-attest");
    let (_, parsed) = workspace.json("p28", "attest-verify", &[]);
    assert_eq!(parsed["wire_len"].as_u64(), Some(80), "P28LicenseAttestation is 80B");
    assert_eq!(
        parsed["license_urn"].as_str(),
        Some("urn:avec:license:p28-sil:v2.1.0-sovereign"),
        "frozen license URN pin"
    );
    assert_eq!(parsed["over_ceiling_refused"].as_bool(), Some(true));
    assert_eq!(parsed["roundtrip_ok"].as_bool(), Some(true));
}

#[test]
fn crypto_seal_produces_a_digest_receipt() {
    let workspace = TestWorkspace::new("crypto-seal");
    let (_, parsed) = workspace.json("crypto", "seal", &[]);
    assert!(parsed["receipt_len"].as_u64().unwrap_or(0) > 0, "receipt emitted");
    assert!(parsed["tick_after_seal"].as_u64().unwrap_or(0) >= 1);
    assert_eq!(parsed["receipt_digest"].as_str().map(str::len), Some(64));
}

#[test]
fn crypto_receipt_verify_refuses_bit_tamper() {
    let workspace = TestWorkspace::new("crypto-verify");
    let (_, parsed) = workspace.json("crypto", "receipt-verify", &[]);
    assert_eq!(parsed["verified"].as_bool(), Some(true));
    assert_eq!(parsed["tamper_refused"].as_bool(), Some(true));
}

#[test]
fn gpu_info_reports_observed_availability() {
    let workspace = TestWorkspace::new("gpu-info");
    let (_, parsed) = workspace.json("gpu", "info", &[]);
    assert!(
        parsed["available"].is_boolean(),
        "availability must be observed (bool), never assumed"
    );
}

#[test]
fn beam_info_reports_real_spsc_harness_counts() {
    let workspace = TestWorkspace::new("beam-info");
    let (_, parsed) = workspace.json("beam", "info", &[]);
    assert_eq!(parsed["exit"].as_i64(), Some(0));
    assert!(parsed["passed"].as_u64().unwrap_or(0) > 0, "real SPSC tests ran");
    assert_eq!(parsed["failed"].as_u64(), Some(0));
}

#[test]
fn court_verdict_run_reports_real_harness_counts() {
    let workspace = TestWorkspace::new("court-verdict");
    let (_, parsed) = workspace.json("court", "verdict-run", &[]);
    assert_eq!(parsed["exit"].as_i64(), Some(0));
    assert!(parsed["passed"].as_u64().unwrap_or(0) > 0, "real court tests ran");
    assert_eq!(parsed["failed"].as_u64(), Some(0));
}
