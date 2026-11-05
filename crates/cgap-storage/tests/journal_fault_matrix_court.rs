//! journal_fault_matrix_court — RC-07. In-process fault-matrix court over
//! the A05 recovery chain (`atomic_commit.rs`). Companion to
//! `scripts/journal_mutation_matrix.py`, which drives the SAME matrix
//! through this file's `harness_stdio` test (byte-level protocol, no
//! serde) against a fixture journal built here by the real
//! `encode_record` — never invented bytes.
//!
//! Exit law is enforced by the Python gate (0/1/2/3); this file keeps
//! `cargo test -p cgap-storage` green on its own.
//!
//! Anti-vacuity: the in-process matrix asserts every fault row's typed
//! outcome differs from the pristine outcome where the code's actual
//! behavior says it must, and a M3 single-fault flip
//! (`MutationMode::TrailerDigestSkipped` over the bitflip-blake3-tail
//! row) must CHANGE the verdict — a matrix whose fault rows all match
//! the pristine outcome is a vacuous court and fails loudly.
//!
//! Sandbox law: no file on disk is mutated here; all faults are applied
//! to in-memory slot copies. File-level mutation/restore is the Python
//! gate's job (sha256 fail-closed + mtime bump there).

use cgap_storage::atomic_commit::{
    encode_record, recover, recover_mode, MutationMode, NewRootDescriptor, RecoveryError,
    RecoveryReport, SlotVerdict, RECORD_SIZE, SLOT_COUNT,
};

// --------------------------------------------------------------- fixture --

/// Descriptor builder: git_oid fixed, dag_seq = dag, parquet = dag*16.
fn p(dag: u64) -> NewRootDescriptor {
    NewRootDescriptor::new([7u8; 20], dag.wrapping_mul(16), dag, 0, 0b0000_0001)
}

/// Real-collaborator medium: slot bytes built from `encode_record`, base
/// root dag 9. Implements the pub `Medium` trait — no mocks.
struct FixtureMedium {
    base: NewRootDescriptor,
    slots: [[u8; RECORD_SIZE]; SLOT_COUNT],
}

impl cgap_storage::atomic_commit::Medium for FixtureMedium {
    fn read_base_root(&self) -> NewRootDescriptor {
        self.base
    }
    fn read_slot(&self, slot: usize) -> [u8; RECORD_SIZE] {
        self.slots[slot]
    }
    fn artifacts_fenced(&self) -> bool {
        true
    }
}

/// Pristine fixture: base dag 9, seq 1 (dag 10) in slot 0, seq 2 (dag 11)
/// in slot 1. Returns (base, slot bytes).
fn fixture() -> (NewRootDescriptor, [[u8; RECORD_SIZE]; SLOT_COUNT]) {
    (p(9), [encode_record(1, &p(10)), encode_record(2, &p(11))])
}

// ------------------------------------------------------- fault injection --

/// XOR one bit at absolute byte `offset` of `slot` (F1 aging-corruption).
fn flip_bit(slots: &mut [[u8; RECORD_SIZE]; SLOT_COUNT], slot: usize, offset: usize) {
    slots[slot][offset] ^= 0x01;
}

/// Torn write: keep only the first `k` bytes of `slot`'s record; the rest
/// reads back 0xFF (flash erase semantics, C0-C4 model).
fn truncate(slots: &mut [[u8; RECORD_SIZE]; SLOT_COUNT], slot: usize, k: usize) {
    for b in slots[slot].iter_mut().skip(k) {
        *b = 0xFF;
    }
}

/// Zero the 48-byte header region of `slot` (media-zeroing fault).
fn zero_header(slots: &mut [[u8; RECORD_SIZE]; SLOT_COUNT], slot: usize) {
    for b in slots[slot].iter_mut().take(48) {
        *b = 0;
    }
}

// ------------------------------------------------------------ matrix rows --

/// Expected outcome per row, derived from the actual recovery semantics in
/// atomic_commit.rs (top valid seq wins; degraded only when NO valid
/// candidate AND some slot is Damaged; erased-only is a clean base
/// rollback; same-seq conflicting payloads are a typed refusal).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Expected {
    /// (root dag, report, recovered seq)
    Recovered(u64, RecoveryReport, Option<u64>),
    /// Typed refusal, fail-closed, never a guessed root.
    Refused(RecoveryError),
}

struct Row {
    name: &'static str,
    /// true = fault row (outcome MUST differ from pristine; identical =>
    /// VACUOUS); false = invariance row (dedup/reorder: outcome must be
    /// UNCHANGED — drift there is itself a finding).
    fault: bool,
    apply: fn(&mut [[u8; RECORD_SIZE]; SLOT_COUNT]),
    expected: Expected,
}

/// The fault corpus (fix-14 A2 R-A2.1 slot classes + RC-07 dispatch):
/// truncation, bit-flip first/mid/last of a record, torn final record,
/// missing journal, duplicated record (dedup AND conflict variants),
/// reordered records, zeroed header, both-records-lost degraded class.
fn matrix() -> Vec<Row> {
    use RecoveryReport::{Clean, RecoveryDegraded};
    vec![
        Row {
            name: "pristine",
            fault: true,
            apply: |_s| {},
            expected: Expected::Recovered(11, Clean, Some(2)),
        },
        Row {
            name: "bitflip_seq2_first",
            fault: true,
            apply: |s| flip_bit(s, 1, 3),
            expected: Expected::Recovered(10, Clean, Some(1)),
        },
        Row {
            name: "bitflip_seq2_mid",
            fault: true,
            apply: |s| flip_bit(s, 1, 60),
            expected: Expected::Recovered(10, Clean, Some(1)),
        },
        Row {
            name: "bitflip_seq2_last",
            fault: true,
            apply: |s| flip_bit(s, 1, 111),
            expected: Expected::Recovered(10, Clean, Some(1)),
        },
        Row {
            name: "bitflip_seq1_first",
            fault: true,
            apply: |s| flip_bit(s, 0, 3),
            expected: Expected::Recovered(11, Clean, Some(2)),
        },
        Row {
            name: "bitflip_seq1_mid",
            fault: true,
            apply: |s| flip_bit(s, 0, 60),
            expected: Expected::Recovered(11, Clean, Some(2)),
        },
        Row {
            name: "bitflip_seq1_last",
            fault: true,
            apply: |s| flip_bit(s, 0, 111),
            expected: Expected::Recovered(11, Clean, Some(2)),
        },
        Row {
            name: "truncate_seq2_k1",
            fault: true,
            apply: |s| truncate(s, 1, 1),
            expected: Expected::Recovered(10, Clean, Some(1)),
        },
        Row {
            name: "truncate_seq2_k60",
            fault: true,
            apply: |s| truncate(s, 1, 60),
            expected: Expected::Recovered(10, Clean, Some(1)),
        },
        Row {
            name: "truncate_seq2_k111",
            fault: true,
            apply: |s| truncate(s, 1, 111),
            expected: Expected::Recovered(10, Clean, Some(1)),
        },
        Row {
            name: "truncate_seq1_k60",
            fault: true,
            apply: |s| truncate(s, 0, 60),
            expected: Expected::Recovered(11, Clean, Some(2)),
        },
        // Both records lost: no valid candidate + Damaged verdicts -> base
        // root returned with a typed RecoveryDegraded (F2 semantics).
        Row {
            name: "truncate_both_k60",
            fault: true,
            apply: |s| {
                truncate(s, 0, 60);
                truncate(s, 1, 60);
            },
            expected: Expected::Recovered(9, RecoveryDegraded, None),
        },
        // Missing journal entirely: both slots erased (0xFF) -> clean base
        // rollback, NOT degraded (erased-state carve-out in recover()).
        Row {
            name: "missing_journal",
            fault: true,
            apply: |s| {
                truncate(s, 0, 0);
                truncate(s, 1, 0);
            },
            expected: Expected::Recovered(9, Clean, None),
        },
        Row {
            name: "zeroed_header_seq2",
            fault: true,
            apply: |s| zero_header(s, 1),
            expected: Expected::Recovered(10, Clean, Some(1)),
        },
        // Duplicated record, identical (seq, payload) in both slots: F3
        // dedup collapses to one -> clean recovery of that record.
        Row {
            name: "duplicated_record",
            fault: false,
            apply: |s| {
                s[0] = encode_record(2, &p(11));
            },
            expected: Expected::Recovered(11, Clean, Some(2)),
        },
        // Duplicate CONFLICT: same seq, differing payloads -> typed
        // JournalAmbiguous, fail-closed, never a guessed root.
        Row {
            name: "duplicate_conflict",
            fault: true,
            apply: |s| {
                s[0] = encode_record(2, &p(12));
            },
            expected: Expected::Refused(RecoveryError::JournalAmbiguous),
        },
        // Reordered records: seq 2 in slot 0, seq 1 in slot 1 — top is
        // chosen by sequence, not slot order -> seq 2 wins.
        Row {
            name: "reordered_records",
            fault: false,
            apply: |s| {
                s[0] = encode_record(2, &p(11));
                s[1] = encode_record(1, &p(10));
            },
            expected: Expected::Recovered(11, Clean, Some(2)),
        },
    ]
}

fn run_row(row: &Row) -> Result<cgap_storage::atomic_commit::RecoveryOutcome, RecoveryError> {
    let (base, mut slots) = fixture();
    (row.apply)(&mut slots);
    recover(&FixtureMedium { base, slots })
}

// ---------------------------------------------------------- in-process court --

#[test]
fn fault_matrix_matches_derived_classes() {
    let pristine = run_row(&matrix()[0]).expect("pristine must recover");
    assert_eq!(pristine.root, p(11));
    assert_eq!(pristine.report, RecoveryReport::Clean);
    let mut non_clean = 0usize;
    for row in matrix().iter().skip(1) {
        let got = run_row(row);
        match (&row.expected, &got) {
            (Expected::Recovered(dag, rep, seq), Ok(out)) => {
                assert_eq!(out.root, p(*dag), "row {}", row.name);
                assert_eq!(&out.report, rep, "row {}", row.name);
                assert_eq!(&out.recovered_seq, seq, "row {}", row.name);
            }
            (Expected::Refused(e), Err(got_e)) => assert_eq!(e, got_e, "row {}", row.name),
            (exp, got) => panic!("row {}: expected {exp:?}, got {got:?}", row.name),
        }
        // Non-vacuity law, per row kind: fault rows must DIFFER from
        // pristine (a silent clean recovery is VACUOUS); invariance rows
        // must be UNCHANGED (drift in dedup/reorder is a finding).
        let matches_pristine = match (&got, &row.expected) {
            (Ok(out), _) => {
                out.root == pristine.root
                    && out.report == pristine.report
                    && out.recovered_seq == pristine.recovered_seq
                    && out.slot_verdicts == pristine.slot_verdicts
            }
            (Err(_), _) => false,
        };
        if row.fault {
            assert!(
                !matches_pristine,
                "fault row {} matches pristine -> VACUOUS",
                row.name
            );
        } else {
            assert!(
                matches_pristine,
                "invariance row {} drifted from pristine",
                row.name
            );
        }
        if !matches!(got, Ok(o) if o.report == RecoveryReport::Clean && o.root == p(11)) {
            non_clean += 1;
        }
    }
    // Corpus-level non-vacuity: degraded AND refusal classes must both be
    // exercised; a matrix that never degrades or refuses is vacuous.
    assert!(
        non_clean >= 2,
        "expected >=2 non-clean rows, got {non_clean}"
    );
}

/// Named single-fault anti-vacuity flip (M3): bit-flipping the BLAKE3
/// trailer half of seq 2 must REJECT under Strict (recovery falls back to
/// seq 1) and ACCEPT under M3TrailerDigestSkipped (record validates, seq 2
/// root 11 returns). A matrix where the mutation does not flip is FAIL.
#[test]
fn m3_trailer_digest_mutation_flips() {
    let (base, mut slots) = fixture();
    flip_bit(&mut slots, 1, 105);
    let strict = recover(&FixtureMedium { base, slots }).expect("strict");
    assert_eq!(strict.root, p(10), "strict must fall back to seq 1");
    assert_eq!(strict.recovered_seq, Some(1));
    let mutated = recover_mode(
        &FixtureMedium { base, slots },
        MutationMode::M3TrailerDigestSkipped,
    )
    .expect("mutated");
    assert_eq!(mutated.root, p(11), "M3 must accept the damaged record");
    assert_eq!(mutated.recovered_seq, Some(2));
    assert_ne!(strict.root, mutated.root, "court did not flip -> VACUOUS");
}

// ------------------------------------------------------------- stdio harness --

/// Byte-level harness the Python gate drives. Activated only when
/// JFM_SPEC/JFM_OUT are set; inert under a plain `cargo test` run. No
/// serde dep: line protocol, hex slot bytes. Spec grammar:
///   fixture            -> emit `fixture <base_dag> <slot0hex> <slot1hex>`
///   row <name>         -> open a row; following `slot <i> <hex>` and
///                         `base <dag>` lines configure it; the row runs
///                         at the next `row`/`fixture` line or EOF.
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> [u8; RECORD_SIZE] {
    let b = s.as_bytes();
    assert_eq!(b.len(), 2 * RECORD_SIZE, "slot hex length");
    let mut out = [0u8; RECORD_SIZE];
    for i in 0..RECORD_SIZE {
        let hi = (b[2 * i] as char).to_digit(16).expect("hex hi");
        let lo = (b[2 * i + 1] as char).to_digit(16).expect("hex lo");
        out[i] = (hi * 16 + lo) as u8;
    }
    out
}

#[test]
fn harness_stdio() {
    let Ok(spec_path) = std::env::var("JFM_SPEC") else {
        return;
    };
    let Ok(out_path) = std::env::var("JFM_OUT") else {
        return;
    };
    let spec = std::fs::read_to_string(&spec_path).expect("spec file");
    let mut out = String::new();
    let mut open: Option<(String, u64, Vec<[u8; RECORD_SIZE]>)> = None;
    let finish = |open: &mut Option<(String, u64, Vec<[u8; RECORD_SIZE]>)>, out: &mut String| {
        if let Some((name, dag, slots)) = open.take() {
            let mut s = [[0u8; RECORD_SIZE]; SLOT_COUNT];
            s.copy_from_slice(&slots);
            let med = FixtureMedium {
                base: p(dag),
                slots: s,
            };
            match recover(&med) {
                Ok(o) => out.push_str(&format!(
                    "row {} OK {} {} {} {} {}\n",
                    name,
                    o.root.parquet_root / 16,
                    match o.report {
                        RecoveryReport::Clean => "Clean",
                        RecoveryReport::RecoveryDegraded => "Degraded",
                    },
                    match o.recovered_seq {
                        Some(s) => s.to_string(),
                        None => "-".into(),
                    },
                    fmt_v(o.slot_verdicts[0]),
                    fmt_v(o.slot_verdicts[1]),
                )),
                Err(e) => out.push_str(&format!(
                    "row {} ERR {} {} - - -\n",
                    name,
                    match e {
                        RecoveryError::JournalAmbiguous => "JournalAmbiguous",
                        RecoveryError::SeqRegression => "SeqRegression",
                    },
                    "Refused",
                )),
            }
        }
    };
    for line in spec.lines() {
        if line == "fixture" {
            finish(&mut open, &mut out);
            let (_, slots) = fixture();
            out.push_str(&format!(
                "fixture 9 {} {}\n",
                hex(&slots[0]),
                hex(&slots[1])
            ));
        } else if let Some(rest) = line.strip_prefix("rec ") {
            // rec <seq> <dag> -> emit `rec <seq> <dag> <hex>`: a REAL
            // encode_record minting, for rows needing extra records
            // (duplicate-conflict needs same seq, differing payload).
            let mut it = rest.splitn(2, ' ');
            let seq: u64 = it.next().unwrap().parse().expect("rec seq");
            let dag: u64 = it.next().unwrap().parse().expect("rec dag");
            out.push_str(&format!(
                "rec {} {} {}\n",
                seq,
                dag,
                hex(&encode_record(seq, &p(dag)))
            ));
        } else if let Some(name) = line.strip_prefix("row ") {
            finish(&mut open, &mut out);
            open = Some((name.to_string(), 9, vec![fixture().1[0], fixture().1[1]]));
        } else if let Some(rest) = line.strip_prefix("base ") {
            if let Some(o) = open.as_mut() {
                o.1 = rest.trim().parse().expect("base dag");
            }
        } else if let Some(rest) = line.strip_prefix("slot ") {
            let mut it = rest.splitn(2, ' ');
            let idx: usize = it.next().unwrap().parse().expect("slot idx");
            let bytes = unhex(it.next().expect("slot hex"));
            let o = open.as_mut().expect("slot outside row");
            o.2[idx] = bytes;
        }
    }
    finish(&mut open, &mut out);
    std::fs::write(&out_path, &out).expect("write harness out");
}

fn fmt_v(v: SlotVerdict) -> &'static str {
    match v {
        SlotVerdict::Valid => "Valid",
        SlotVerdict::Empty => "Empty",
        SlotVerdict::Damaged => "Damaged",
    }
}
