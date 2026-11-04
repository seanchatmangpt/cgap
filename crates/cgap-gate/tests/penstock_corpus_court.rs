//! RC-12 penstock malformed-corpus differential court (hardening wave).
//!
//! Fills the gap the red-team battery shape named for the penstock: the
//! six-row fail-closed taxonomy (crates/cgap-gate/src/penstock.rs:505-511,
//! ingress at :513) was courted one-row-at-a-time but had no systematic
//! malformed-corpus differential court. This file owns ONLY that court; it
//! is read-only over penstock.rs.
//!
//! Battery:
//! 1. Deterministic corpus: every taxonomy row mapped to its EXPECTED
//!    class per the doc table (:505-511).
//! 2. Bit-flip differential: flip bit 0 of every header byte 0..128 of a
//!    valid frame; expected class from a documented per-range mapping.
//! 3. Truncation sweep: a valid frame truncated at EVERY offset 0..128.
//! 4. Cross-product legs (red-09 D3 class): byte-at-a-time prefix feeding
//!    and split-frame (chunked reassembly) feeding must converge to the
//!    one-shot class.
//! 5. Anti-vacuity: flipping ONE expected-class mapping in a /tmp copy of
//!    this corpus table must fail the court (executed in the lane
//!    receipt, not in-tree).
//!
//! Chicago discipline: real `ingress` calls over real buffers, final-state
//! asserts only, no mocks, no sleeps, deterministic (fixed corpus, no
//! rng). Any (input, expected, observed) mismatch is a loud FAIL row,
//! never papered over.

use cgap_gate::penstock::{
    ingress, FrameView, HEADER_LEN, OABI_DIGEST_LEN, OFF_ATTESTATION, OFF_FLAGS, OFF_OABI_DIGEST,
    OFF_PAYLOAD_BYTE_LEN, OFF_VERSION, P28_FLAG_ROOT, P28_LICENSE_MAGIC,
    P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Exp {
    Ok,
    InvalidMagic,
    OabiMismatch,
    Spoilation,
}

fn code_of(e: Exp) -> u32 {
    use cgap_gate::penstock::{
        CGAP_ERR_INVALID_MAGIC, CGAP_ERR_OABI_MISMATCH, CGAP_ERR_P28_SPOLIATION, CGAP_OK,
    };
    match e {
        Exp::Ok => CGAP_OK,
        Exp::InvalidMagic => CGAP_ERR_INVALID_MAGIC,
        Exp::OabiMismatch => CGAP_ERR_OABI_MISMATCH,
        Exp::Spoilation => CGAP_ERR_P28_SPOLIATION,
    }
}

/// Observed classification: the register status the real ingress returns.
fn classify(bytes: &[u8], len: usize) -> u32 {
    ingress(bytes, len, None)
}
// ---------------------------------------------------------------------------
// Valid-frame builder: minimal ROOT-lineage frame, 128 B header + 8 B
// payload. Every corpus case starts from this and breaks exactly the law
// its taxonomy row names.
// ---------------------------------------------------------------------------

const PAYLOAD_LEN: usize = 8;
const FRAME_LEN: usize = HEADER_LEN + PAYLOAD_LEN;
const BUF_LEN: usize = 512;

fn valid_frame(buf: &mut [u8]) {
    buf.fill(0);
    buf[0..4].copy_from_slice(b"CGAP");
    buf[OFF_VERSION] = 1;
    buf[OFF_FLAGS] = 0x01; // FLAG_ARROW only
    buf[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
        .copy_from_slice(&(PAYLOAD_LEN as u64).to_le_bytes());
    buf[OFF_OABI_DIGEST] = 0xA5; // nonzero digest field (coherence law only)
    buf[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&P28_LICENSE_MAGIC);
    buf[OFF_ATTESTATION + 4..OFF_ATTESTATION + 6].copy_from_slice(&P28_VERSION.to_le_bytes());
    buf[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&P28_FLAG_ROOT.to_le_bytes());
    buf[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
        .copy_from_slice(&P28_SAFE_HARBOR_CEILING_CENTS.to_le_bytes());
    // ROOT lineage: upstream_receipt (header bytes 96..128) stays all-zero
    // per the A17 receipt law.
}

// ---------------------------------------------------------------------------
// Corpus cases. Deterministic: fixed cases plus counted loops, no rng.
// ---------------------------------------------------------------------------

struct Case {
    name: String,
    bytes: Vec<u8>, // buffer handed to ingress
    len: usize,     // caller-declared length (the transport truth)
    exp: Exp,
}

impl Case {
    fn one_shot(&self) -> u32 {
        classify(&self.bytes, self.len)
    }
}

/// Build a case from a fresh valid frame, then break exactly the law its
/// taxonomy row names.
fn mk(name: &str, exp: Exp, mutate: &mut dyn FnMut(&mut [u8], &mut usize)) -> Case {
    let mut bytes = vec![0u8; BUF_LEN];
    valid_frame(&mut bytes);
    let mut len = FRAME_LEN;
    mutate(&mut bytes, &mut len);
    Case {
        name: name.to_string(),
        bytes,
        len,
        exp,
    }
}

fn corpus() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    // Row 0: valid frame admits (the differential anchor).
    cases.push(mk("row0_valid_admit", Exp::Ok, &mut |_, _| {}));
    // Row 1: truncated header at EVERY offset 0..127 (declared len < 128).
    for n in 0..HEADER_LEN {
        let name = format!("row1_truncated_len_{n}");
        cases.push(mk(&name, Exp::InvalidMagic, &mut |_, len| *len = n));
    }
    // Row 1 class: lying caller - declared len exceeds the actual slice.
    {
        let mut c = mk("row1_lying_caller", Exp::InvalidMagic, &mut |_, len| {
            *len = 500;
        });
        c.bytes.truncate(200); // the slice lies short of the declared len
        cases.push(c);
    }
    // Row 2: magic families + version drift.
    let magics: [&[u8; 4]; 5] = [b"XGAP", b"cgap", b"CGAx", &[0u8; 4], b"P28S"];
    for (i, m) in magics.iter().enumerate() {
        let name = format!("row2_bad_magic_family_{i}");
        cases.push(mk(&name, Exp::InvalidMagic, &mut |b, _| {
            b[0..4].copy_from_slice(*m);
        }));
    }
    for v in [0u8, 2, 255] {
        let name = format!("row2_bad_version_{v}");
        cases.push(mk(&name, Exp::InvalidMagic, &mut |b, _| b[OFF_VERSION] = v));
    }
    // Row 3a: all-zero digest field.
    cases.push(mk(
        "row3a_digest_all_zero",
        Exp::OabiMismatch,
        &mut |b, _| {
            b[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].fill(0);
        },
    ));
    // Row 3b: declared-length lies - payload beyond the declared buffer.
    let lies: [u64; 4] = [9, 264, 1 << 20, u64::MAX];
    for (i, d) in lies.iter().enumerate() {
        let name = format!("row3b_declared_len_lie_{i}");
        cases.push(mk(&name, Exp::OabiMismatch, &mut |b, _| {
            b[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8].copy_from_slice(&d.to_le_bytes());
        }));
    }
    // Zero-length payload leg: declared payload 0 ADMITS (fits exactly).
    cases.push(mk("row0_zero_len_payload_admits", Exp::Ok, &mut |b, _| {
        b[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8].copy_from_slice(&0u64.to_le_bytes());
    }));
    // Oversized-buffer leg: extra slice slack beyond the declared len is
    // never read; verdict unchanged on the declared geometry.
    cases.push(mk(
        "row0_oversized_buffer_slack_ignored",
        Exp::Ok,
        &mut |_, _| {},
    ));
    cases
}
fn corpus_row4() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    let att_magics: [&[u8; 4]; 3] = [b"P29\0", &[0u8; 4], b"p28s"];
    for (i, m) in att_magics.iter().enumerate() {
        let name = format!("row4_bad_att_magic_{i}");
        cases.push(mk(&name, Exp::Spoilation, &mut |b, _| {
            b[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(*m);
        }));
    }
    cases
}

fn corpus_row5() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    // (name, patch relative to OFF_ATTESTATION, bytes to write)
    let att: Vec<(&str, usize, Vec<u8>)> = vec![
        ("att_version_2", 4, vec![2, 0]),
        ("att_reserved_bit", 6, vec![8, 0]),
        ("att_zero_lineage", 6, vec![0, 0]),
        ("att_two_lineage_bits", 6, vec![3, 0]),
        (
            "att_ceiling_over",
            8,
            5_000_000_001u64.to_le_bytes().to_vec(),
        ),
        ("att_ceiling_u64_max", 8, u64::MAX.to_le_bytes().to_vec()),
    ];
    for (name, off, val) in att {
        let full = format!("row5_{name}");
        cases.push(mk(&full, Exp::Spoilation, &mut |b, _| {
            b[OFF_ATTESTATION + off..OFF_ATTESTATION + off + val.len()].copy_from_slice(&val);
        }));
    }
    cases
}
fn corpus_row5_receipts() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    // A17 receipt law: ROOT xor receipt-zero. Break it in both directions.
    // ROOT with a nonzero receipt (header bytes 96..128 written nonzero).
    cases.push(mk(
        "row5_root_with_receipt",
        Exp::Spoilation,
        &mut |b, _| {
            b[96..128].fill(0xFF);
        },
    ));
    // DERIVATIVE with a zero receipt: set lineage to DERIVATIVE; the base
    // frame's receipt is already all-zero.
    cases.push(mk(
        "row5_deriv_zero_receipt",
        Exp::Spoilation,
        &mut |b, _| {
            b[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&2u16.to_le_bytes());
        },
    ));
    // TRANSPILED with a zero receipt.
    cases.push(mk(
        "row5_transpiled_zero_receipt",
        Exp::Spoilation,
        &mut |b, _| {
            b[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&4u16.to_le_bytes());
        },
    ));
    cases
}

fn corpus_row6() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    // Row 6 divergence (penstock.rs ESCALATE_MASK note): header flags
    // bits 4-7 stay ADMITTED (A04 6.3 "ignored on read"); the gate marks
    // escalate=true in the view instead of refusing.
    for bits in [0x10u8, 0xF0] {
        let name = format!("row6_reserved_flags_{bits:#04x}_admitted");
        cases.push(mk(&name, Exp::Ok, &mut |b, _| {
            b[OFF_FLAGS] |= bits;
        }));
    }
    cases
}
// ---------------------------------------------------------------------------
// Bit-flip differential corpus: flip bit 0 of every header byte 0..128 of
// the valid base frame. Expected class from the DOCUMENTED per-range
// mapping of the penstock table; any divergence between observed and
// documented is a FAIL row, never papered over.
// ---------------------------------------------------------------------------

/// Documented expected class for a bit-0 flip of header byte `off` of the
/// valid base frame (mapping reasoned per range against the :505-511
/// table; base = CGAP magic, v1, flags 0x01, declared 8, digest 0xA5 at
/// byte 16 then zeros, ROOT attestation, ceiling 5e9, zero receipt).
fn flip_exp(off: usize) -> Exp {
    match off {
        // Bytes 0..4 magic (any flip breaks "CGAP"); byte 4 version 1->0:
        // row-2 class. Bytes 5..8 flags/reserved are NOT refusal
        // conditions in the table (flags bits 4-7 are row-6 ADMIT, bits
        // 0-3 and reserved bytes are never gated) -> still Ok.
        0..=4 => Exp::InvalidMagic,
        5..=7 => Exp::Ok,
        // Bytes 8..16: declared payload 8 ^ bit0 of byte k (k in 0..8)
        // = 8 + 2^(8k) > 8 = budget -> row 3b.
        8..=15 => Exp::OabiMismatch,
        // Bytes 16..48 digest: byte 16 is 0xA5 -> 0xA4 (still nonzero);
        // zero bytes -> 0x01 (digest stays nonzero) -> still admits.
        16..=47 => Exp::Ok,
        // Bytes 48..56 attestation: magic 48..52, version 52..54,
        // flags 54..56 (bit0 flip = zero lineage / reserved bit) ->
        // row 4/5 class.
        48..=56 => Exp::Spoilation,
        // Bytes 56..64 ceiling (5e9 LE = 00 F2 05 2A 01 00 00 00): a
        // bit0 flip ADDS to bytes 56,57,59,61,62,63 (over ceiling ->
        // Spoilation) but SUBTRACTS from bytes 58 (0x05 -> 0x04) and
        // 60 (0x01 -> 0x00) (back under ceiling -> Ok).
        58 | 60 => Exp::Ok,
        57 | 59 | 61..=63 => Exp::Spoilation,
        // Bytes 64..96 origin digest: never gated on the admit path.
        64..=95 => Exp::Ok,
        // Bytes 96..128 upstream_receipt: zero -> nonzero with ROOT ->
        // receipt law violated.
        96..=128 => Exp::Spoilation,
        _ => unreachable!(),
    }
}

fn flip_cases() -> Vec<Case> {
    (0..HEADER_LEN)
        .map(|off| {
            let name = format!("flip_bit0_header_byte_{off}");
            mk(&name, flip_exp(off), &mut |b, _| {
                b[off] ^= 0x01;
            })
        })
        .collect()
}
/// Full corpus = rows 0-6 + bit-flip differential (128 cases).
fn full_corpus() -> Vec<Case> {
    let mut v = corpus();
    v.extend(corpus_row4());
    v.extend(corpus_row5());
    v.extend(corpus_row5_receipts());
    v.extend(corpus_row6());
    v.extend(flip_cases());
    v
}

/// FAIL-row collector: any (input, expected, observed) mismatch is a loud
/// listed row, never papered over.
fn fail_rows(rows: &[(String, Exp, u32)]) {
    if !rows.is_empty() {
        let mut msg = format!("CORPUS MISMATCHES ({} rows):", rows.len());
        for (name, exp, obs) in rows {
            msg.push_str(&format!(
                "\n  FAIL row: input={name} expected={exp:?} observed={obs:#010x}"
            ));
        }
        panic!("{}", msg);
    }
}

// ---------------------------------------------------------------------------
// Court 1: one-shot differential — every corpus input through the real
// ingress, observed class must equal the documented class.
// ---------------------------------------------------------------------------

#[test]
fn corpus_differential_one_shot_matches_documented_classes() {
    let cases = full_corpus();
    assert!(
        cases.len() >= 150,
        "corpus degenerate: only {} cases",
        cases.len()
    );
    let mut rows: Vec<(String, Exp, u32)> = Vec::new();
    for c in &cases {
        let obs = c.one_shot();
        if obs != code_of(c.exp) {
            rows.push((c.name.clone(), c.exp, obs));
        }
    }
    fail_rows(&rows);
}
#[test]
fn corpus_prefix_feeding_converges_to_one_shot() {
    let cases = full_corpus();
    let mut rows: Vec<(String, Exp, u32)> = Vec::new();
    for c in &cases {
        for k in 1..=c.len.max(1) {
            let Some(exp) = prefix_expected(c, k) else {
                continue;
            };
            let obs = classify(&c.bytes[..k], k);
            if obs != code_of(exp) {
                rows.push((format!("{}_prefix_{k}", c.name), exp, obs));
            }
        }
    }
    fail_rows(&rows);
}
// ---------------------------------------------------------------------------
// Court 2: byte-at-a-time prefix-feeding convergence (red-09 D3 class).
// ingress is a one-shot API, so byte-at-a-time feeding = handing prefixes
// bytes[..k] with declared len k for k = 1..=frame len and asserting the
// documented classification as a function of k, converging to the one-shot
// class once the declared geometry fits. (Lying-caller cases have len >
// slice and no lawful prefix domain - excluded, loudly, by construction.)
// ---------------------------------------------------------------------------

fn prefix_expected(c: &Case, k: usize) -> Option<Exp> {
    if c.len > c.bytes.len() {
        return None; // no lawful prefix domain for a lying caller
    }
    if k > c.len {
        return None;
    }
    if k < HEADER_LEN {
        return Some(Exp::InvalidMagic);
    }
    // Table order (first failure wins): a row-1/2 class case stays
    // InvalidMagic at every prefix length once the header is present; a
    // later-group case degrades to row 3b (OABI) until the declared
    // payload fits the prefix geometry.
    if c.exp == Exp::InvalidMagic {
        return Some(Exp::InvalidMagic);
    }
    let declared = u64::from_le_bytes(
        c.bytes[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
            .try_into()
            .unwrap(),
    );
    if declared > (k - HEADER_LEN) as u64 {
        Some(Exp::OabiMismatch)
    } else {
        Some(c.exp)
    }
}

// ---------------------------------------------------------------------------
// Court 3: split-frame feeding convergence (red-09 D3 class). The same
// bytes delivered as chunked reassembly (deterministic 1,2,3..7-byte
// chunks) and over a buffer with trailing slack must classify identically
// to the one-shot feed.
// ---------------------------------------------------------------------------

#[test]
fn corpus_split_frame_and_slack_converge_to_one_shot() {
    let cases = full_corpus();
    let mut rows: Vec<(String, Exp, u32)> = Vec::new();
    for c in &cases {
        // Chunked reassembly leg (skip the lying caller: no lawful split).
        if c.len <= c.bytes.len() {
            let mut reassembled = Vec::new();
            let mut i = 0;
            let mut chunk = 1usize;
            while i < c.len {
                let end = (i + chunk).min(c.len);
                reassembled.extend_from_slice(&c.bytes[i..end]);
                i = end;
                chunk = chunk % 7 + 1;
            }
            let obs = classify(&reassembled, c.len);
            if obs != c.one_shot() {
                rows.push((format!("{}_split_reassembly", c.name), c.exp, obs));
            }
        }
        // Trailing-slack leg: same declared geometry over a longer buffer.
        let mut slack = c.bytes.clone();
        slack.extend_from_slice(&[0x5Au8; 64]);
        let obs = classify(&slack, c.len);
        if obs != c.one_shot() {
            rows.push((format!("{}_slack64", c.name), c.exp, obs));
        }
    }
    fail_rows(&rows);
}

// ---------------------------------------------------------------------------
// Court 4: admit-path view law on every Ok-classified corpus case — real
// ingress with Some(&mut view), payload geometry zero-copy into the
// caller's bytes at the declared length, escalate bit only when the
// row-6 reserved-flag family is set.
// ---------------------------------------------------------------------------

#[test]
fn corpus_admit_path_view_law() {
    for c in full_corpus().iter().filter(|c| c.exp == Exp::Ok) {
        let mut view = FrameView {
            payload: &[],
            prefilter: 0,
            escalate: false,
        };
        let st = ingress(&c.bytes, c.len, Some(&mut view));
        assert_eq!(st, code_of(Exp::Ok), "admit leg regressed on {}", c.name);
        let declared = u64::from_le_bytes(
            c.bytes[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
                .try_into()
                .unwrap(),
        );
        assert_eq!(
            view.payload.len() as u64,
            declared,
            "view geometry on {}",
            c.name
        );
        assert_eq!(
            view.payload.as_ptr(),
            c.bytes[HEADER_LEN..].as_ptr(),
            "view not zero-copy on {}",
            c.name
        );
        let escalate_expected = c.bytes[OFF_FLAGS] & 0xF0 != 0;
        assert_eq!(
            view.escalate, escalate_expected,
            "escalate bit law on {}",
            c.name
        );
    }
}

// ---------------------------------------------------------------------------
// Court 5: coverage pin — the corpus must genuinely cover every taxonomy
// row family and every expected class; a degenerate corpus proves nothing
// (anti-vacuity at the table level).
// ---------------------------------------------------------------------------

#[test]
fn corpus_table_covers_every_taxonomy_row_and_class() {
    let cases = full_corpus();
    for row in [
        "row0_", "row1_", "row2_", "row3a_", "row3b_", "row4_", "row5_", "row6_",
    ] {
        assert!(
            cases.iter().any(|c| c.name.starts_with(row)),
            "corpus missing taxonomy family {row}"
        );
    }
    for cls in [
        Exp::Ok,
        Exp::InvalidMagic,
        Exp::OabiMismatch,
        Exp::Spoilation,
    ] {
        assert!(
            cases.iter().any(|c| c.exp == cls),
            "corpus missing expected class {cls:?}"
        );
    }
    assert_eq!(
        (0..HEADER_LEN)
            .filter(|off| flip_exp(*off) == Exp::Spoilation)
            .count(),
        46,
        "flip mapping spoliation count drifted"
    );
    assert!(
        cases.iter().any(|c| c.name == "row1_lying_caller"),
        "lying-caller case missing"
    );
}
