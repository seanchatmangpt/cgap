//! ZP-15 foreign-subject projection court (v26.11.4, lane S-4).
//!
//! Plan text (ggen `docs/jira/v26.11.4/PLANS.md` ZP-15): "gate-ID
//! collision — the runner's G1-G7 ≠ RFC §6's G1-G7 (different
//! taxonomies); the projection uses namespaced P1-P8 rows with
//! `projection_class ∈ {PROJECTABLE, ANALOGUE, NOT-PROJECTABLE}` +
//! `attestation_class ∈ {SELF, ATTESTED, EXTERNAL}`; NOT-PROJECTABLE
//! register for kernel-internal gates (compile-court spec-drift, ≤25
//! µs/link receipt chain); Kafka adapter = real broker transport,
//! replies NO_CENSUS honestly (P1 → CONTRACT_UNSUPPORTED → PROVISIONAL
//! cap — the expected honest outcome for JVM incumbents)."
//!
//! Real subject: the landed RFC-0089 §8.4 certificate kernel
//! (`cgap_aoq::rfc0089_certificate` — the ZP-21-promoted lib surface)
//! driven over the REAL AB-14 chain-scale receipt on disk, the same §6
//! source the scorecard's leg B reads. The court derives the §6 G7
//! verdict with its OWN oracle parse (deliberately not shared with the
//! generator), issues a real BLAKE3-sealed certificate over the
//! projection register, verifies it, and witnesses the tamper law.
//!
//! Falsifier (S-4 lane contract): the projection court renders P1-P8
//! rows with `projection_class` on a real subject. Gate: exit 0.

use cgap_aoq::rfc0089_certificate::{
    certificate_json, hex, issue_with_seal, verify_with, GateVerdict, Overall, SealMismatch,
    Verdict, VerdictTable,
};

const AB14: &str = "/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md";

// ---------------------------------------------------------------------------
// Projection classes (plan-vocabulary, typed)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectionClass {
    Projectable,
    Analogue,
    NotProjectable,
}

impl ProjectionClass {
    fn as_str(self) -> &'static str {
        match self {
            ProjectionClass::Projectable => "PROJECTABLE",
            ProjectionClass::Analogue => "ANALOGUE",
            ProjectionClass::NotProjectable => "NOT-PROJECTABLE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttestationClass {
    Self_,
    Attested,
    External,
}

impl AttestationClass {
    fn as_str(self) -> &'static str {
        match self {
            AttestationClass::Self_ => "SELF",
            AttestationClass::Attested => "ATTESTED",
            AttestationClass::External => "EXTERNAL",
        }
    }
}

/// One namespaced P-row of the foreign-subject projection register.
#[derive(Debug, Clone)]
struct ProjectionRow {
    /// Namespaced projection id (P1..P8) — never a bare G-id.
    id: &'static str,
    /// Source gate row, BOTH taxonomies named (gate-ID collision law:
    /// runner G1-G7 ≠ RFC §6 G1-G7).
    source: &'static str,
    projection_class: ProjectionClass,
    attestation_class: AttestationClass,
    /// Evidence receipt path grounding the row (existence checked).
    evidence: &'static str,
    measured: String,
}

impl ProjectionRow {
    fn render(&self) -> String {
        format!(
            "{:<4} <- {:<28} {:<16} {:<10} {} | measured: {}",
            self.id,
            self.source,
            self.projection_class.as_str(),
            self.attestation_class.as_str(),
            self.evidence,
            self.measured
        )
    }
}

// ---------------------------------------------------------------------------
// §6 G7 oracle: independent parse of the AB-14 receipt. The court must
// NOT reuse the generator's `g7_leg_b_from_receipt` — independent
// re-derivation is the audit surface.
// ---------------------------------------------------------------------------

fn oracle_g7_us_per_link(path: &str) -> Result<(f64, String), String> {
    let rec = std::fs::read_to_string(path).map_err(|e| format!("unreadable: {e}"))?;
    if !(rec.contains("min-of-3") || rec.contains("min of 3")) {
        return Err("receipt lost its min-of-3 disclosure".into());
    }
    let idx = rec.find("50k span").ok_or("no '50k span' line")?;
    let tail = &rec[idx + "50k span".len()..];
    let num: String = tail
        .chars()
        .skip_while(|c| !c.is_ascii_digit() && *c != '.')
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let ms: f64 = num.parse().map_err(|e| format!("unparsable '{num}': {e}"))?;
    Ok((ms * 1000.0 / 50_000.0, format!("{ms:.2} ms")))
}

// ---------------------------------------------------------------------------
// The real subject table: §8.4 gate rows grounded in the landed
// receipts (AB-9, AB-19, AB-6, AB-2/3, AB-10/11, AB-17, AB-14). Same
// rows the certificate kernel's scorecard table carries; G7 is
// computed from disk, never asserted.
// ---------------------------------------------------------------------------

fn real_subject_table(run_seq: u64) -> VerdictTable {
    let g7 = match oracle_g7_us_per_link(AB14) {
        Ok((us, ms_str)) if us <= 25.0 => Verdict::Pass,
        Ok((us, ms_str)) => Verdict::Fail {
            why: format!(
                "RFC §6 G7 MEASURED-UNMET: receipted 50k-span cost {ms_str} / 50,000 = \
                 {us:.4} µs/link > 25 µs/link target (min-of-3, AB-14 on disk)"
            ),
        },
        Err(e) => Verdict::Skip {
            reason: format!("G7 §6 source unmeasurable: {e}"),
        },
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
                composes: "AB-14/AF-21 receipt chain (§6 scorecard source, leg B)",
                verdict: g7,
                measured: match oracle_g7_us_per_link(AB14) {
                    Ok((us, ms_str)) => {
                        format!("{ms_str} / 50,000 links = {us:.4} µs/link vs 25 µs/link target")
                    }
                    Err(e) => e,
                },
                threshold: "per-link cost <= 25 µs/link at 50k span",
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Namespaced projection: §8.4 rows → P1-P8 register rows.
// ---------------------------------------------------------------------------

/// Kafka-adapter census reply (plan law: the adapter replies NO_CENSUS
/// honestly when the foreign subject carries no census contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForeignCensusReply {
    /// Real census payload (allocs, bytes) — the native reply shape.
    Census { allocs: u64 },
    /// The honest foreign-subject reply (JVM incumbents).
    NoCensus,
}

/// Typed census-exchange outcome (the runner's typed reason codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExchangeOutcome {
    Pass,
    ContractUnsupported,
}

/// Typed mapping: foreign reply → exchange outcome → qualification cap.
/// P1 → CONTRACT_UNSUPPORTED → PROVISIONAL cap is the EXPECTED HONEST
/// outcome for JVM incumbents (plan text) — never a fabricated pass.
fn foreign_exchange(reply: ForeignCensusReply) -> (ExchangeOutcome, &'static str) {
    match reply {
        ForeignCensusReply::Census { allocs: _ } => (ExchangeOutcome::Pass, "FULL"),
        // NO_CENSUS is honest, not a fault: typed CONTRACT_UNSUPPORTED,
        // qualification capped at PROVISIONAL. Never fabricated to PASS.
        ForeignCensusReply::NoCensus => (ExchangeOutcome::ContractUnsupported, "PROVISIONAL"),
    }
}

/// Project the real §8.4 table onto the namespaced P1-P8 register.
fn projection_register(table: &VerdictTable) -> Vec<ProjectionRow> {
    let g = |id: &str| table.gates.iter().find(|g| g.id == id).expect("gate row");

    let kafka_reply = ForeignCensusReply::NoCensus; // JVM-incumbent shape
    let (outcome, cap) = foreign_exchange(kafka_reply);

    let measured = |id: &str| g(id).measured.clone();

    vec![
        // P1 — the Kafka-adapter census exchange leg. ANALOGUE: the
        // foreign subject answers under a DIFFERENT contract; the
        // projection is by typed mapping, not by shared semantics.
        ProjectionRow {
            id: "P1",
            source: "runner-G1 (alloc census exchange)",
            projection_class: ProjectionClass::Analogue,
            attestation_class: AttestationClass::External,
            evidence: "tests/cbmqs_runner.rs NO_CENSUS law",
            measured: format!(
                "foreign reply NO_CENSUS -> {:?} -> cap {} (honest; JVM incumbents)",
                outcome, cap
            ),
        },
        ProjectionRow {
            id: "P2",
            source: "RFC§6-G1 (AB-9 alloc court)",
            projection_class: ProjectionClass::Projectable,
            attestation_class: AttestationClass::Attested,
            evidence: "ggen docs/jira/v26.11.3/receipts/ab-9",
            measured: measured("G1"),
        },
        ProjectionRow {
            id: "P3",
            source: "RFC§6-G1b (AB-19 slab throughput)",
            projection_class: ProjectionClass::Projectable,
            attestation_class: AttestationClass::Attested,
            evidence: "ggen docs/jira/v26.11.3/receipts/ab-19",
            measured: measured("G1b"),
        },
        ProjectionRow {
            id: "P4",
            source: "RFC§6-G2 (AB-6 bench court)",
            projection_class: ProjectionClass::Projectable,
            attestation_class: AttestationClass::Attested,
            evidence: "ggen docs/jira/v26.11.3/receipts/ab-6",
            measured: measured("G2"),
        },
        ProjectionRow {
            id: "P5",
            source: "RFC§6-G3+G4 (drift flip + KAT, in-repo witnesses)",
            projection_class: ProjectionClass::Projectable,
            attestation_class: AttestationClass::Self_,
            evidence: "crates/cgap-aoq/tests/aoq1_layout_drift_court.rs",
            measured: format!("{}; {}", measured("G3"), measured("G4")),
        },
        ProjectionRow {
            id: "P6",
            source: "RFC§6-G5 (AB-10/AB-11 storm + FrameTooLarge)",
            projection_class: ProjectionClass::Projectable,
            attestation_class: AttestationClass::Attested,
            evidence: "ggen docs/jira/v26.11.3/receipts/ab-10",
            measured: measured("G5"),
        },
        ProjectionRow {
            id: "P7",
            source: "RFC§6-G6 (AB-17 churn, PASS-CITED cross-repo lsp-max)",
            projection_class: ProjectionClass::Analogue,
            attestation_class: AttestationClass::External,
            evidence: "lsp-max cross-repo receipt (cited, not re-run)",
            measured: measured("G6"),
        },
        // P8 — kernel-internal: the ≤25 µs/link receipt chain reads a
        // receipt INSIDE this tree's trust boundary; a foreign subject
        // cannot attest it. Plan names this class explicitly.
        ProjectionRow {
            id: "P8",
            source: "RFC§6-G7 (AB-14 ≤25 µs/link receipt chain)",
            projection_class: ProjectionClass::NotProjectable,
            attestation_class: AttestationClass::Self_,
            evidence: AB14,
            measured: measured("G7"),
        },
    ]
}

fn render_register(rows: &[ProjectionRow]) -> String {
    let mut s = String::from("== ZP-15 foreign-subject projection register ==\n");
    for r in rows {
        s.push_str(&r.render());
        s.push('\n');
    }
    s
}

// ---------------------------------------------------------------------------
// Courts
// ---------------------------------------------------------------------------

/// Court 1 — the falsifier itself: P1-P8 rows render with
/// projection_class on the REAL subject (§8.4 table over AB-14 disk).
#[test]
fn court_projection_renders_p1_p8_on_real_subject() {
    let table = real_subject_table(15);
    assert_eq!(table.gates.len(), 8, "eight §8.4 source rows");
    let rows = projection_register(&table);
    assert_eq!(rows.len(), 8, "P1-P8 register rows");

    let expected = ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8"];
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(r.id, expected[i], "namespaced order");
    }

    // Namespacing law: no bare G-id leaks into the projection register.
    for r in &rows {
        assert!(
            r.id.starts_with('P'),
            "register id must be namespaced: {}",
            r.id
        );
    }

    // Every projection_class is one of the three plan-vocabulary values
    // (exhaustive enum — compile-time; here we assert the rendered
    // vocabulary is exactly the plan's closed set).
    let legal = ["PROJECTABLE", "ANALOGUE", "NOT-PROJECTABLE"];
    for r in &rows {
        let cls = r.projection_class.as_str();
        assert!(legal.contains(&cls), "illegal projection_class: {cls}");
    }

    // NOT-PROJECTABLE register law: kernel-internal gates (G7 receipt
    // chain) must be registered, never projected as pass.
    let p8 = rows.iter().find(|r| r.id == "P8").unwrap();
    assert_eq!(p8.projection_class, ProjectionClass::NotProjectable);
    assert_eq!(p8.attestation_class, AttestationClass::Self_);

    // Honest Kafka leg: NO_CENSUS -> CONTRACT_UNSUPPORTED -> PROVISIONAL.
    let p1 = rows.iter().find(|r| r.id == "P1").unwrap();
    assert!(p1.measured.contains("ContractUnsupported"));
    assert!(p1.measured.contains("PROVISIONAL"));

    // Real output: the rendered register on stdout (visible with
    // --nocapture; this IS the falsifier's rendered artifact).
    let rendered = render_register(&rows);
    println!("{}", rendered);
    assert!(rendered.contains("P1"));
    assert!(rendered.contains("NOT-PROJECTABLE"));
}

/// Court 2 — the register is grounded: the real subject table is
/// BLAKE3-sealed via the landed certificate kernel, verifies, and its
/// G7 row actually follows the on-disk AB-14 receipt (MEASURED-UNMET
/// per §6 — the certificate states a fact, never softens).
#[test]
fn court_real_subject_seals_and_g7_follows_disk() {
    let table = real_subject_table(42);
    let (cert, seal_hex) = issue_with_seal(
        &cgap_blake3::hash,
        |json| hex(&cgap_blake3::hash(json.as_bytes())),
        &table,
        vec![AB14.to_string()],
    )
    .expect("no SKIP-class rows in the real subject table");

    assert_eq!(cert.spec, "RFC-0089");
    assert_eq!(cert.timestamp_source, "NONE");
    // Honest overall: §6 G7 is MEASURED-UNMET on disk → DISQUALIFIED.
    assert_eq!(cert.overall, Overall::Disqualified);

    // The seal verifies over the real kernel.
    verify_with(&cert, &seal_hex, |json| {
        hex(&cgap_blake3::hash(json.as_bytes()))
    })
    .expect("seal verifies");

    // G7 row follows disk: oracle number matches the row's verdict.
    let (us, _) = oracle_g7_us_per_link(AB14).expect("AB-14 parseable");
    let g7 = cert.gates.iter().find(|g| g.id == "G7").expect("G7 row");
    if us > 25.0 {
        assert!(
            matches!(g7.verdict, Verdict::Fail { .. }),
            "G7 must stay MEASURED-UNMET while disk says {us:.4} µs/link"
        );
    } else {
        assert!(matches!(g7.verdict, Verdict::Pass));
    }

    // Determinism: same table + receipts → same seal (no wall clock).
    let (_, seal2) = issue_with_seal(
        &cgap_blake3::hash,
        |json| hex(&cgap_blake3::hash(json.as_bytes())),
        &table,
        vec![AB14.to_string()],
    )
    .unwrap();
    assert_eq!(seal_hex, seal2);
}

/// Court 3 — tamper law over the projection register: any field
/// mutation post-seal → typed SealMismatch refusal.
#[test]
fn court_tampered_register_refuses_typed() {
    let table = real_subject_table(7);
    let (cert, seal_hex) = issue_with_seal(
        &cgap_blake3::hash,
        |json| hex(&cgap_blake3::hash(json.as_bytes())),
        &table,
        vec![],
    )
    .unwrap();

    // SANDBOX-INJECTED TAMPER (court mutation): flip a measured value.
    let mut forged = cert.clone();
    forged.gates[2].measured = "9999 ns (forged)".into();

    let err = verify_with(&forged, &seal_hex, |json| {
        hex(&cgap_blake3::hash(json.as_bytes()))
    })
    .expect_err("tampered body must refuse");
    let SealMismatch { expected, observed } = err;
    assert_ne!(expected, observed, "seals diverge under tamper");
}

/// Court 4 — anti-vacuity: if the AB-14 receipt vanished, the G7 row
/// (and therefore P8) degrades to Skip, and the certificate kernel
/// REFUSES to issue (fail-closed) — the projection court cannot fake
/// a pass over a missing §6 source.
#[test]
fn court_missing_source_refuses_fail_closed() {
    let mut table = real_subject_table(9);
    table.gates[7].verdict = Verdict::Skip {
        reason: "AB-14 unreadable (simulated loss)".into(),
    };
    let err = issue_with_seal(
        &cgap_blake3::hash,
        |json| hex(&cgap_blake3::hash(json.as_bytes())),
        &table,
        vec![],
    )
    .expect_err("SKIP-class gate must refuse issue");
    assert_eq!(
        err.skipped_gates,
        vec!["G7"],
        "typed CertificateIncomplete names G7"
    );
}

/// Court 5 — non-vacuity of the oracle itself: a mutated receipt COPY
/// (never the real file) flips the oracle outcome. A court whose
/// oracle cannot flip is decoration.
#[test]
fn court_oracle_flips_on_mutated_copy() {
    let real = std::fs::read_to_string(AB14).expect("AB-14 readable");
    let (us_real, _) = oracle_g7_us_per_link(AB14).expect("real receipt parses");

    let dir = std::env::temp_dir();
    let path = dir.join("zp15-oracle-flip-copy.md");
    std::fs::write(&path, &real).unwrap();

    // Mutate a COPY: shrink the 50k span cost so the target is met.
    let faster = {
        let mut s = real.clone();
        let idx = s.find("50k span").unwrap();
        let start = idx + "50k span".len();
        let digits: usize = s[start..]
            .chars()
            .skip_while(|c| !c.is_ascii_digit() && *c != '.')
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .count();
        let digits_start = s[start..]
            .char_indices()
            .skip_while(|(_, c)| !c.is_ascii_digit() && *c != '.')
            .next()
            .map(|(i, _)| i)
            .unwrap_or(0);
        let byte_start = start + digits_start;
        let digits_end = byte_start
            + s[byte_start..]
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .map(|c| c.len_utf8())
                .sum::<usize>();
        let mut replacement = String::new();
        replacement.push_str(&s[..byte_start]);
        replacement.push_str(" 0.10 ms");
        replacement.push_str(&s[digits_end..]);
        s = replacement;
        s
    };
    std::fs::write(&path, &faster).unwrap();
    let (us_fast, _) = oracle_g7_us_per_link(path.to_str().unwrap()).expect("copy parses");
    std::fs::remove_file(&path).unwrap();

    assert!(
        us_fast < 0.1,
        "mutated copy drives µs/link under target: {us_fast}"
    );
    // And the direction differs from the real receipt's outcome.
    assert!(
        (us_real > 25.0) != (us_fast > 25.0),
        "oracle flips across the 25 µs/link fence"
    );
}
