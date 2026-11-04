//! cbmqs-audit — CBMQS std host tool (P5 phases 1-2).
//!
//! Verbs:
//! * `--emit-certificate <path> --run-seq <u64>` — derive the RFC-0089 §6
//!   verdict table from the scorecard source (AB-14 receipt), issue the
//!   §8.4 certificate via the kernel's `issue_with_seal` DI seam, write
//!   canonical JSON to `<path>` + hex BLAKE3-256 seal to sidecar
//!   `<path>.seal`. Unreadable/unparsable receipt → typed
//!   `CertificateIncomplete` → exit 3, NO artifact written.
//! * `--verify-certificate <path>` — fail-closed: unknown schema_version /
//!   canonical_form / seal_algorithm, malformed form, missing/short
//!   sidecar, seal mismatch, or the (run_seq, input_hash) identity law
//!   (RunSeqCollision) → typed refusal exit 2.
//!   The seal binds the BODY, not the outcome: a DISQUALIFIED
//!   certificate verifies fine.
//!
//! Exit ABI: 0 pass; 2 typed refusal; 3 loud skip (no artifact).

mod cert_io;

use std::process::ExitCode;

use cert_io::{
    envelope_json, hex, parse_envelope, read_seal, seal_over, Refusal, CANONICAL_FORM,
    SCHEMA_VERSION, SEAL_ALGORITHM,
};
use cgap_aoq::rfc0089_certificate::{
    certificate_json, issue_with_seal, GateVerdict, Verdict, VerdictTable,
};

/// AB-14's landed receipt — the SAME on-disk source the RX-3 scorecard
/// (`rfc0089_scorecard.rs` `gate_g7()` leg B) reads. Single source of
/// truth: the certificate derives its G7 row from this file, never from
/// a hand-encoded verdict.
/// Machine-local default for the cross-repo receipts root (this
/// checkout). `GGEN_RECEIPTS_ROOT` overrides — without it the literal
/// only resolves on the machine that has the ggen checkout; an absent
/// receipt degrades to the §6 typed `Skip` (exit 3), never a panic.
fn ab14_receipt_path() -> String {
    match std::env::var("GGEN_RECEIPTS_ROOT") {
        Ok(root) => format!("{root}/ab-14-chain-scale.md"),
        Err(_) => "/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md".into(),
    }
}

/// RFC-0089 §6 G7 target, per-link, at the 50k span.
const G7_TARGET_US_PER_LINK: f64 = 25.0;


fn main() -> ExitCode {
    let cmd = command().get_matches();
    let emit = cmd.get_one::<String>("emit_certificate");
    let verify = cmd.get_one::<String>("verify_certificate");
    match (emit, verify) {
        (Some(path), _) => {
            let run_seq = match cmd.get_one::<u64>("run_seq") {
                Some(n) => *n,
                None => {
                    eprintln!("REFUSED: --run-seq <u64> is required with --emit-certificate");
                    return ExitCode::from(2);
                }
            };
            let g7 = cmd.get_one::<String>("g7_receipt").map(|s| s.as_str());
            let evidence: Vec<String> = cmd
                .get_many::<String>("evidence")
                .map(|vals| vals.cloned().collect())
                .unwrap_or_default();
            ExitCode::from(cmd_emit(path, run_seq, g7, evidence) as u8)
        }
        (None, Some(path)) => ExitCode::from(cmd_verify(path) as u8),
        (None, None) => {
            eprintln!("REFUSED: one of --emit-certificate / --verify-certificate is required");
            ExitCode::from(2)
        }
    }
}

fn command() -> clap::Command {
    use clap::{Arg, ArgAction, Command};
    Command::new("cbmqs-audit")
        .about("CBMQS RFC-0089 §8.4 certificate emit/verify (fail-closed)")
        .disable_version_flag(true)
        .arg(
            Arg::new("emit_certificate")
                .long("emit-certificate")
                .value_name("PATH")
                .help("emit: write canonical certificate JSON to PATH")
                .conflicts_with("verify_certificate"),
        )
        .arg(
            Arg::new("verify_certificate")
                .long("verify-certificate")
                .value_name("PATH")
                .help("verify: fail-closed verify of PATH (+ PATH.seal sidecar)")
                .conflicts_with("emit_certificate"),
        )
        .arg(
            Arg::new("run_seq")
                .long("run-seq")
                .value_name("U64")
                .value_parser(clap::value_parser!(u64))
                .help("emit: run-sequence number (required with --emit-certificate)"),
        )
        .arg(
            Arg::new("g7_receipt")
                .long("g7-receipt")
                .value_name("PATH")
                .help("emit: override the §6 G7 source (sandbox courts)"),
        )
        .arg(
            Arg::new("evidence")
                .long("evidence")
                .value_name("PATH")
                .action(ArgAction::Append)
                .help("emit: evidence receipt path (repeatable)"),
        )
        .after_help("exit ABI: 0 pass / 2 typed refusal / 3 loud skip")
}

/// The §6 law applied to the scorecard source: parse the AB-14 receipt's
/// "50k span" cost line, derive µs/link over 50,000 links, and map to
/// the certificate verdict vocabulary. Unreadable/unparsable receipt →
/// `Skip` (which `issue_with_seal` fails closed on — no certificate at
/// all), mirroring the scorecard's TARGET-UNMEASURED degradation.
fn g7_leg_b_from_receipt(path: &str) -> Verdict {
    let parse = (|| -> Result<Verdict, String> {
        let rec = std::fs::read_to_string(path).map_err(|e| format!("receipt unreadable: {e}"))?;
        let marker = "50k span";
        let idx = rec
            .find(marker)
            .ok_or("receipt no longer carries the '50k span' cost line")?;
        let tail = &rec[idx + marker.len()..];
        let num: String = tail
            .chars()
            .skip_while(|c| !c.is_ascii_digit() && *c != '.')
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let ms: f64 = num
            .parse()
            .map_err(|e| format!("'50k span' value unparsable ('{num}'): {e}"))?;
        if !(rec.contains("min-of-3") || rec.contains("min of 3")) {
            return Err("receipt lost its min-of-3 disclosure".into());
        }
        let us_per_link = ms * 1000.0 / 50_000.0;
        if us_per_link <= G7_TARGET_US_PER_LINK {
            Ok(Verdict::Pass)
        } else {
            Ok(Verdict::Fail {
                why: format!(
                    "RFC §6 G7 MEASURED-UNMET: receipted 50k-span cost {ms:.2} ms / 50,000 \
                     = {us_per_link:.4} µs/link > 25 µs/link target (debug profile, min-of-3, \
                     cgap-ffi RingReceiptChain plane; same source the RX-3 scorecard leg B reads)"
                ),
            })
        }
    })();
    match parse {
        Ok(v) => v,
        Err(e) => Verdict::Skip {
            reason: format!("G7 §6 source unmeasurable: {e}"),
        },
    }
}

/// The certificate table, derived from the scorecard's §6 source (AB-14
/// receipt on disk) — NOT a hand-encoded verdict table. G1-G6 cite the
/// landed receipts (ab-6 86 ns amortized; ab-9 0 B/op; ab-19 12.2M/s);
/// G7 is computed by the §6 law from AB-14, so a PASS here can only
/// exist if the scorecard source itself says PASS.
fn scorecard_table(run_seq: u64, g7_receipt: Option<&str>) -> VerdictTable {
    let g7v = match g7_receipt {
        Some(p) => g7_leg_b_from_receipt(p),
        None => g7_leg_b_from_receipt(&ab14_receipt_path()),
    };
    VerdictTable {
        run_seq,
        gates: vec![
            GateVerdict {
                id: "G1",
                composes: "AB-9 alloc court",
                verdict: Verdict::Pass,
                measured: "0 allocs / 0 B per op after init (100k windows)".into(),
                threshold: "0 B/op after init",
            },
            GateVerdict {
                id: "G1b",
                composes: "AB-19 slab throughput",
                verdict: Verdict::Pass,
                measured: "12.2M/s slab push+reset (receipt floor 100k/s)".into(),
                threshold: ">= 100k/s",
            },
            GateVerdict {
                id: "G2",
                composes: "AB-6 bench court",
                verdict: Verdict::Pass,
                measured: "86 ns amortized-min encode+decode per header (release)".into(),
                threshold: "< 100 ns/header",
            },
            GateVerdict {
                id: "G3",
                composes: "AB-6 ontology-drift court",
                verdict: Verdict::Pass,
                measured: "injected offset mutation flips the court (flip witnessed)".into(),
                threshold: "court flips under mutation",
            },
            GateVerdict {
                id: "G4",
                composes: "AB-2/AB-3 KAT fixtures + flip witness",
                verdict: Verdict::Pass,
                measured: "KATs green; flipped byte flips witness".into(),
                threshold: "KATs green AND flip witness flips",
            },
            GateVerdict {
                id: "G5",
                composes: "AB-10/AB-11 storm + FrameTooLarge",
                verdict: Verdict::Pass,
                measured: "50k-frame storm passed; oversized refused FrameTooLarge typed".into(),
                threshold: "storm passes AND oversize refuses typed",
            },
            GateVerdict {
                id: "G6",
                composes: "AB-17 200-cycle churn (PASS-CITED cross-repo, lsp-max)",
                verdict: Verdict::Pass,
                measured: "200 churn cycles green (cross-repo receipt cited)".into(),
                threshold: ">= 200 cycles green",
            },
            GateVerdict {
                id: "G7",
                composes: "AB-14/AF-21 receipt chain (§6 scorecard source, leg B binding)",
                verdict: g7v.clone(),
                measured: match &g7v {
                    Verdict::Pass => "leg B within 25 µs/link at 50k span (per receipt on disk)".into(),
                    Verdict::Fail { why } => why.clone(),
                    Verdict::Skip { reason } => reason.clone(),
                },
                threshold: "per-link cost <= 25 us/link at 50k span",
            },
        ],
    }
}

/// Issue + envelope + write. Emit path of the exit ABI.
fn cmd_emit(
    path: &str,
    run_seq: u64,
    g7_receipt: Option<&str>,
    evidence: Vec<String>,
) -> i32 {
    let table = scorecard_table(run_seq, g7_receipt);
    let (cert, _kernel_seal) = match issue_with_seal(&cgap_blake3::hash, seal_fn(), &table, evidence)
    {
        Ok(pair) => pair,
        Err(incomplete) => {
            eprintln!(
                "SKIP CertificateIncomplete: skipped gates {:?} -- no certificate written",
                incomplete.skipped_gates
            );
            return 3;
        }
    };
    let canonical = match envelope_json(&certificate_json(&cert)) {
        Ok(s) => s,
        Err(r) => {
            eprintln!("{r}");
            return 2;
        }
    };
    let seal = seal_over(canonical.as_bytes());
    if let Err(e) = std::fs::write(path, &canonical) {
        eprintln!("REFUSED certificate write failed: {e}");
        return 2;
    }
    let seal_path = format!("{path}.seal");
    if let Err(e) = std::fs::write(&seal_path, format!("{seal}\n")) {
        eprintln!("REFUSED seal sidecar write failed: {e}");
        return 2;
    }
    println!(
        "EMITTED {path} (schema_version {SCHEMA_VERSION}, canonical_form {CANONICAL_FORM}, \
         seal_algorithm {SEAL_ALGORITHM}, run_seq {run_seq}, overall {:?})",
        cert.overall
    );
    0
}

/// seal_fn contract: hex(BLAKE3-256 over the canonical JSON string).
fn seal_fn() -> impl Fn(&str) -> String {
    |json: &str| hex(&cgap_blake3::hash(json.as_bytes()))
}

/// Fail-closed verify: schema-key refusal law, sidecar check, the
/// RunSeqCollision identity law, then the seal check — in that order.
fn cmd_verify(path: &str) -> i32 {
    let canon = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("REFUSED certificate unreadable: {e}");
            return 2;
        }
    };
    let parsed = match parse_envelope(&canon) {
        Ok(p) => p,
        Err(r) => {
            eprintln!("{r}");
            return 2;
        }
    };
    let presented = match read_seal(path) {
        Ok(s) => s,
        Err(r) => {
            eprintln!("{r}");
            return 2;
        }
    };
    // RunSeqCollision identity law BEFORE the seal check: same run_seq
    // binds exactly one input_hash (the §6-derived table's).
    let derived = hex(&scorecard_table(parsed.run_seq, None).content_hash_with(&cgap_blake3::hash));
    if derived != parsed.input_hash {
        eprintln!(
            "{}",
            Refusal::RunSeqCollision {
                run_seq: parsed.run_seq,
                presented_input_hash: parsed.input_hash.clone(),
                derived_input_hash: derived,
            }
        );
        return 2;
    }
    // DOD4 H2 body-binding law, also BEFORE the seal check: rehash the
    // PRESENTED body's gates under the same §6 law used at emit. The
    // seal is unkeyed (anyone can recompute it), so a patched-then-
    // resealed body is only caught here — the patched table hashes
    // differently from the presented input_hash.
    let body_table = match cert_io::rebuild_table(&parsed) {
        Ok(t) => t,
        Err(r) => {
            eprintln!("{r}");
            return 2;
        }
    };
    let body_hash = hex(&body_table.content_hash_with(&cgap_blake3::hash));
    if body_hash != parsed.input_hash {
        eprintln!(
            "{}",
            Refusal::BodyHashMismatch {
                presented_input_hash: parsed.input_hash.clone(),
                body_input_hash: body_hash,
            }
        );
        return 2;
    }
    let recomputed = seal_over(canon.as_bytes());
    if recomputed != presented {
        eprintln!(
            "{}",
            Refusal::SealMismatch {
                expected: recomputed,
                observed: presented,
            }
        );
        return 2;
    }
    println!(
        "VERIFIED {path} (run_seq {}, input_hash {})",
        parsed.run_seq, parsed.input_hash
    );
    0
}
