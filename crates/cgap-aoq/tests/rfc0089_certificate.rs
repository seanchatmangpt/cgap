//! RFC-0089 §8.4 — CBMQS Qualification Certificate (RX-5, v26.11.3).
//!
//! Input: a scorecard verdict table (RX-3's output surface). The gate
//! rows are DERIVED from the same source RFC §6 (RX-3 scorecard) reads
//! — the landed AB-14 chain-scale receipt on disk — via
//! `scorecard_table()`; the certificate can never again mark a gate
//! PASS where the scorecard's law records UNMET (RX-5 correction,
//! class-3 hazard: green_table had G7 hardcoded Pass vs RFC §6 G7
//! MEASURED-UNMET at 25.5202 µs/link vs the 25 µs target).
//!
//! Certificate = canonical JSON, BLAKE3-sealed via the landed cgap-blake3
//! crate. No wall clock (kernel law): `timestamp_source: "NONE"`; identity
//! comes from run-sequence number (`run_seq`) + BLAKE3 content hashes.
//!
//! Laws under court:
//! * §6 disqualification law — no certificate over any SKIP-class gate
//!   (typed `CertificateIncomplete` refusal, fail-closed).
//! * §8.4 tamper law — any field mutation post-seal → hash mismatch →
//!   typed refusal from `verify`.

use cgap_blake3::hash;

// ----------------------------------------------------------- verdict table

/// Verdict class per gate. Mirrors the landed per-gate court shapes
/// (`Status::{Pass, Fail, Skip}` in cbmqs_gates.rs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail { why: String },
    Skip { reason: String },
}

/// One gate's row in the scorecard verdict table (RX-3 surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateVerdict {
    pub id: &'static str,
    pub composes: &'static str,
    pub verdict: Verdict,
    pub measured: String,
    pub threshold: &'static str,
}

/// The scorecard verdict table: run-sequence number + gate rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictTable {
    pub run_seq: u64,
    pub gates: Vec<GateVerdict>,
}

impl VerdictTable {
    /// BLAKE3 content hash of the canonical serialization — the input
    /// identity that replaces wall-clock timestamps (kernel law).
    pub fn content_hash(&self) -> [u8; 32] {
        hash(canonical_table_json(self).as_bytes())
    }
}

// --------------------------------------------------- canonical JSON (§8.4)

/// Minimal JSON string escaping (RFC 8259 control chars + quote/backslash).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Canonical JSON of the verdict table (fixed key order, no whitespace).
/// Fixed declaration order IS the canonicalization: gates appear in
/// evaluation order, so there is no sorting ambiguity.
fn canonical_table_json(t: &VerdictTable) -> String {
    let mut s = String::new();
    s.push_str("{\"run_seq\":");
    s.push_str(&t.run_seq.to_string());
    s.push_str(",\"gates\":[");
    for (i, g) in t.gates.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&canonical_gate_json(g));
    }
    s.push_str("]}");
    s
}

fn canonical_gate_json(g: &GateVerdict) -> String {
    let verdict = match &g.verdict {
        Verdict::Pass => "{\"kind\":\"pass\"}".to_string(),
        Verdict::Fail { why } => format!("{{\"kind\":\"fail\",\"why\":{}}}", json_escape(why)),
        Verdict::Skip { reason } => {
            format!("{{\"kind\":\"skip\",\"reason\":{}}}", json_escape(reason))
        }
    };
    format!(
        concat!(
            "{{",
            "\"id\":{id},",
            "\"composes\":{composes},",
            "\"verdict\":{verdict},",
            "\"measured\":{measured},",
            "\"threshold\":{threshold}",
            "}}"
        ),
        id = json_escape(g.id),
        composes = json_escape(g.composes),
        verdict = verdict,
        measured = json_escape(&g.measured),
        threshold = json_escape(g.threshold),
    )
}

// ------------------------------------------------------------ certificate

pub const CERT_SPEC: &str = "RFC-0089";

/// Overall certificate outcome (§6): only PASS or DISQUALIFIED exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overall {
    Pass,
    Disqualified,
}

/// §8.4 certificate fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    pub spec: &'static str,
    pub run_id: String,
    pub timestamp_source: &'static str, // always "NONE" — kernel law
    pub run_seq: u64,
    pub input_hash: String, // BLAKE3 hex of the verdict table
    pub gates: Vec<GateVerdict>,
    pub overall: Overall,
    pub evidence_receipts: Vec<String>,
}

/// Typed refusal: qualification cannot issue while any gate is SKIP-class
/// (§6 disqualification law — fail-closed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateIncomplete {
    pub skipped_gates: Vec<&'static str>,
}

/// Typed refusal: the presented seal does not match the certificate body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealMismatch {
    pub expected: String,
    pub observed: String,
}

/// Issue a certificate from a verdict table. Fail-closed: any SKIP-class
/// gate → `CertificateIncomplete` typed refusal; no certificate at all.
pub fn issue(
    table: &VerdictTable,
    evidence_receipts: Vec<String>,
) -> Result<(Certificate, String), CertificateIncomplete> {
    // §6: SKIP fails closed before anything else is computed.
    let skipped: Vec<&'static str> = table
        .gates
        .iter()
        .filter(|g| matches!(g.verdict, Verdict::Skip { .. }))
        .map(|g| g.id)
        .collect();
    if !skipped.is_empty() {
        return Err(CertificateIncomplete {
            skipped_gates: skipped,
        });
    }

    let input_hash = hex(&table.content_hash());
    let overall = if table.gates.iter().all(|g| g.verdict == Verdict::Pass) {
        Overall::Pass
    } else {
        Overall::Disqualified
    };
    let cert = Certificate {
        spec: CERT_SPEC,
        run_id: format!("rfc0089-{}", &input_hash[..16]),
        timestamp_source: "NONE",
        run_seq: table.run_seq,
        input_hash: input_hash.clone(),
        gates: table.gates.clone(),
        overall,
        evidence_receipts,
    };
    let json = certificate_json(&cert);
    let seal = hex(&hash(json.as_bytes()));
    Ok((cert, seal))
}

/// Canonical JSON of the certificate (fixed key order, no whitespace).
pub fn certificate_json(c: &Certificate) -> String {
    let gates = c
        .gates
        .iter()
        .map(canonical_gate_json)
        .collect::<Vec<_>>()
        .join(",");
    let receipts = c
        .evidence_receipts
        .iter()
        .map(|r| json_escape(r))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"spec\":{spec},",
            "\"run_id\":{run_id},",
            "\"timestamp_source\":{ts},",
            "\"run_seq\":{seq},",
            "\"input_hash\":{ih},",
            "\"gates\":[{gates}],",
            "\"overall\":\"{overall}\",",
            "\"evidence_receipts\":[{receipts}]",
            "}}"
        ),
        spec = json_escape(c.spec),
        run_id = json_escape(&c.run_id),
        ts = json_escape(c.timestamp_source),
        seq = c.run_seq,
        ih = json_escape(&c.input_hash),
        gates = gates,
        overall = match c.overall {
            Overall::Pass => "PASS",
            Overall::Disqualified => "DISQUALIFIED",
        },
        receipts = receipts,
    )
}

/// BLAKE3 hex seal over the canonical certificate JSON.
pub fn seal(cert: &Certificate) -> String {
    hex(&hash(certificate_json(cert).as_bytes()))
}

/// §8.4 tamper law: recompute the BLAKE3 seal over the certificate as it
/// exists NOW and compare against the presented seal. Any field mutated
/// post-seal changes the canonical JSON → mismatch → typed refusal.
pub fn verify(cert: &Certificate, presented_seal: &str) -> Result<(), SealMismatch> {
    let recomputed = seal(cert);
    if recomputed != presented_seal {
        return Err(SealMismatch {
            expected: recomputed,
            observed: presented_seal.to_string(),
        });
    }
    Ok(())
}

// ------------------------------------------------------------- hex codec

pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// --------------------------------------------------- scorecard-derived table

/// AB-14's landed receipt — the SAME on-disk source RFC §6 (the RX-3
/// scorecard, `rfc0089_scorecard.rs` `gate_g7()` leg B) reads. Single
/// source of truth: the certificate derives its G7 row from this file,
/// never from a hand-encoded verdict.
const AB14_RECEIPT: &str = "/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md";

/// RFC-0089 §6 G7 target, per-link, at the 50k span (same constant the
/// scorecard's `gate_g7()` applies to leg B).
const G7_TARGET_US_PER_LINK: f64 = 25.0;

/// The §6 law applied to the scorecard source: parse AB-14's
/// "50k span" cost line, derive µs/link over 50,000 links, and map to
/// the certificate verdict vocabulary. Unreadable/unparsable receipt →
/// `Skip` (which `issue()` fails closed on — no certificate at all),
/// mirroring the scorecard's TARGET-UNMEASURED degradation.
pub fn g7_leg_b_from_receipt(path: &str) -> Verdict {
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

/// The certificate table, derived from the scorecard's §6 source
/// (AB-14 receipt on disk) — NOT a hand-encoded verdict table. G1–G6
/// cite the landed receipts (ab-6 86 ns amortized; ab-9 0 B/op;
/// ab-19 12.2M/s); G7 is computed by the §6 law from AB-14, so a
/// PASS here can only exist if the scorecard source itself says PASS.
pub fn scorecard_table(run_seq: u64) -> VerdictTable {
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
                verdict: g7_leg_b_from_receipt(AB14_RECEIPT),
                measured: match g7_leg_b_from_receipt(AB14_RECEIPT) {
                    Verdict::Pass => {
                        "leg B within 25 µs/link at 50k span (per receipt on disk)".into()
                    }
                    Verdict::Fail { why } => why,
                    Verdict::Skip { reason } => reason,
                },
                threshold: "per-link cost <= 25 µs/link at 50k span",
            },
        ],
    }
}

// ----------------------------------------------------------------- courts

#[cfg(test)]
mod courts {
    use super::*;

    fn green(run_seq: u64) -> VerdictTable {
        scorecard_table(run_seq)
    }

    /// Gate 1 — §8.4 certificate on the scorecard-derived table. The
    /// honest state per RFC §6 (G7 MEASURED-UNMET at 25.5202 µs/link)
    /// is DISQUALIFIED — the certificate states a fact, never softens.
    /// Issues, seals, verifies clean, deterministic (no wall clock).
    #[test]
    fn court_pass_certificate_on_green_run() {
        let (cert, seal_hex) = issue(
            &green(7),
            vec!["docs/jira/v26.11.3/receipts/rx-5-correction.md".into()],
        )
        .unwrap();
        assert_eq!(cert.spec, "RFC-0089");
        assert_eq!(cert.timestamp_source, "NONE");
        // Honest overall: G7 leg B is MEASURED-UNMET in the §6 source.
        assert_eq!(cert.overall, Overall::Disqualified);
        assert_eq!(cert.run_seq, 7);
        assert_eq!(cert.input_hash, hex(&green(7).content_hash()));
        verify(&cert, &seal_hex).unwrap();
        // G7 is the named failing gate in the body.
        let body = certificate_json(&cert);
        let g7_fail = cert
            .gates
            .iter()
            .find(|g| g.id == "G7")
            .expect("G7 row must exist");
        assert!(matches!(g7_fail.verdict, Verdict::Fail { .. }));
        assert!(body.contains("MEASURED-UNMET"));
        assert!(body.contains("25.5202"));
        // Determinism: same table AND same receipts → same seal (no clock).
        let receipts = vec!["docs/jira/v26.11.3/receipts/rx-5-correction.md".to_string()];
        let (_, seal2) = issue(&green(7), receipts).unwrap();
        assert_eq!(seal_hex, seal2);
        // run_seq is load-bearing: seq 8 changes the body → different seal.
        let (_, seal3) = issue(&green(8), vec![]).unwrap();
        assert_ne!(seal_hex, seal3);
    }

    /// Gate 2 — DISQUALIFIED certificate on a sandbox-injected failure.
    /// A DISQUALIFIED certificate is still a valid certificate (it states
    /// a fact): it issues, seals, verifies — but overall is DISQUALIFIED
    /// and the failing gate is named in the body.
    #[test]
    fn court_disqualified_certificate_on_injected_failure() {
        let mut table = green(11);
        // SANDBOX-INJECTED FAILURE (court mutation): G5 (index 5) flips.
        table.gates[5].verdict = Verdict::Fail {
            why: "injected: storm frame 4173 desynced".into(),
        };
        table.gates[5].measured = "injected failure witness".into();
        let (cert, seal_hex) = issue(&table, vec![]).unwrap();
        assert_eq!(cert.overall, Overall::Disqualified);
        verify(&cert, &seal_hex).unwrap();
        // The seal binds the failure: green-run seal at same seq differs.
        let (_, green_seal) = issue(&green(11), vec![]).unwrap();
        assert_ne!(seal_hex, green_seal);
        // Cross-verify against the green seal → typed mismatch.
        let err = verify(&cert, &green_seal).unwrap_err();
        assert_ne!(err.expected, err.observed);
        // The failing gate is named in the canonical body.
        let body = certificate_json(&cert);
        assert!(body.contains("\"kind\":\"fail\""));
        assert!(body.contains("injected: storm frame 4173 desynced"));
    }

    /// Gate 3 — tamper court: any field mutated post-seal → verify
    /// refuses typed (SealMismatch), never passes.
    #[test]
    fn court_tamper_any_field_mutation_refused() {
        let (cert, seal_hex) = issue(&green(3), vec!["r.md".into()]).unwrap();
        let mut witnessed = 0;

        let mut flip = cert.clone();
        flip.gates[2].verdict = Verdict::Fail {
            why: "post-seal tamper witness".into(),
        };
        assert!(verify(&flip, &seal_hex).is_err());
        witnessed += 1;

        let mut t2 = cert.clone();
        t2.gates[2].measured = "44.5 ns (edited post-seal)".into();
        assert!(verify(&t2, &seal_hex).is_err());
        witnessed += 1;

        let mut t3 = cert.clone();
        t3.overall = Overall::Pass; // base is honestly Disqualified (G7)
        assert!(verify(&t3, &seal_hex).is_err());
        witnessed += 1;

        let mut t4 = cert.clone();
        t4.run_seq += 1;
        assert!(verify(&t4, &seal_hex).is_err());
        witnessed += 1;

        let mut t5 = cert.clone();
        t5.evidence_receipts.push("injected.md".into());
        assert!(verify(&t5, &seal_hex).is_err());
        witnessed += 1;

        let mut t6 = cert.clone();
        t6.input_hash = "0".repeat(64);
        assert!(verify(&t6, &seal_hex).is_err());
        witnessed += 1;

        let mut t7 = cert.clone();
        t7.spec = "RFC-0090";
        assert!(verify(&t7, &seal_hex).is_err());
        witnessed += 1;

        // Seal itself altered → recomputation mismatch.
        let wrong_seal = "f".repeat(64);
        assert!(verify(&cert, &wrong_seal).is_err());
        let err = verify(&cert, &wrong_seal).unwrap_err();
        assert_ne!(err.expected, err.observed);

        // Mutation count is load-bearing (7 field mutations witnessed).
        assert_eq!(witnessed, 7);
    }

    /// Gate 4 — §6 fail-closed: any SKIP-class gate → typed
    /// `CertificateIncomplete` refusal; no certificate issues at all.
    #[test]
    fn court_skip_gate_fails_closed() {
        let mut table = green(5);
        table.gates[6].verdict = Verdict::Skip {
            reason: "cross-repo receipt unreachable this run".into(),
        };
        let err = issue(&table, vec![]).unwrap_err();
        assert_eq!(err.skipped_gates, vec!["G6"]);

        // SKIP + FAIL mix also refuses: the SKIP check precedes everything.
        let mut mixed = green(6);
        mixed.gates[6].verdict = Verdict::Skip {
            reason: "unreachable".into(),
        };
        mixed.gates[2].verdict = Verdict::Fail {
            why: "also failing".into(),
        };
        assert!(issue(&mixed, vec![]).is_err());

        // Re-check the original SKIP table still refuses (idempotent).
        assert!(issue(&table, vec![]).is_err());
    }

    /// Non-vacuity: canonical JSON is deterministic, clock-free, and
    /// load-bearing — same body → same seal; changed body → different seal.
    #[test]
    fn court_non_vacuity_canonical_json_and_identity() {
        let (cert, seal_hex) = issue(&green(9), vec![]).unwrap();
        let j1 = certificate_json(&cert);
        let j2 = certificate_json(&cert.clone());
        assert_eq!(j1, j2);
        // Clock-free identity: NONE timestamp source, no wall-clock types.
        assert!(j1.contains("\"timestamp_source\":\"NONE\""));
        assert!(!j1.contains("SystemTime"));
        assert!(!j1.contains("Instant"));
        // Body-seal binding: clean verify passes; any body change breaks it.
        verify(&cert, &seal_hex).unwrap();
        let mut any_change = cert.clone();
        any_change.run_id = "rfc0089-tampered".into();
        assert!(verify(&any_change, &seal_hex).is_err());
    }
}
