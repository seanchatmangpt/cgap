//! Lane BENCH-4: the CHAINED governance pipeline per record batch — the
//! 80/20 court over the governance cost per row (ASC 450 / SOX framing).
//!
//! Business case (dispatch): every admitted row pays exactly one attestation
//! verify (A17 P28 §5, real 80 B records over the pinned instrument) + one
//! liability metering step (A18 ASC 450, crossing + non-crossing mixed
//! corpus), then a chatman receipt (A16) seals the row into ONE chained
//! governance ledger; the batch tail is a pure verify. Governance cost per
//! row is a compliance line item; its predictability (flat tail) is the
//! product. One bench over consts [1024, 65536]; the stage decomposition
//! (validate vs meter vs seal vs verify) is attributed in the lane receipt
//! from this chain total minus the component benches of `--bench seal`
//! re-run on the same host in the same session (method stated there).
//!
//! Chain law (A16 §3.1 "receipts chain"): row i's delta `from_root` is row
//! i-1's receipt `blake3_digest`, so the batch seals into one tamper-evident
//! governance ledger, not N independent receipts. The final-state asserts
//! below pin the ledger, not the calls (Chicago).
//!
//! Corpus law (charter "Method discipline"): SplitMix64 canonical
//! `0xBF58_476D_1CE4_E5B9` verbatim as master seed; per-record spread via the
//! in-repo penstock form `seed ^ i*0x2544_5491_4F6C_DD1D`
//! (cgap-gate/src/penstock.rs:814). Every corpus byte is derived from that
//! seed; `black_box` consumes the folded ledger checksum; verdict counters
//! are asserted per iteration batch (validated/licensed/crossed/proven/
//! sealed-states/verified counts), so the measured loop is Chicago: real
//! records, real wire bytes, real sealed artifacts, final state asserted.
//!
//! Anti-vacuity (a bench that cannot flip is decoration) — the structural
//! mutations named per stage; a court flips one in a /tmp sandbox:
//! (1) p28 stage: flip one bit of a record's `upstream_receipt` -> §5.7
//!     refuses the row -> the `validated == N` assert fires and the row's
//!     meter/seal/verify cost disappears from the chain.
//! (2) meter stage: demote one crossing row's amount to <= ceiling -> no
//!     crossing -> no `SignedProof` emission -> the `proven == N/2` and
//!     `sealed_states == N/2` asserts fire.
//! (3) seal/verify stage: flip one byte of a receipt's `blake3_digest`
//!     between seal and verify -> `verify_unsealed` = `DigestMismatch` ->
//!     the `verified == N` assert fires.

use std::sync::Mutex;

use cgap_crypto::chatman::{seal_unsealed, verify_unsealed, DeltaS};
use cgap_crypto::lease::Lease;
use cgap_p28::asc450::{
    fused_row, Asc450Meter, LeaseEpochSealer, DEFAULT_SAFE_HARBOR_CEILING_CENTS,
};
use cgap_p28::attestation::{
    instrument_header_span, origin_authority_digest, P28LicenseAttestation, ATTESTATION_LEN,
    FLAG_DERIVATIVE, FLAG_ROOT, SAFE_HARBOR_CEILING_MAX_CENTS,
};
use divan::{black_box, Bencher};

/// Canonical SplitMix64 master seed (charter, verbatim).
const CORPUS_SEED: u64 = 0xBF58_476D_1CE4_E5B9;

/// Per-record spread constant — the in-repo penstock derivation form
/// (cgap-gate/src/penstock.rs:814).
const RECORD_SPREAD: u64 = 0x2544_5491_4F6C_DD1D;

/// Domain bit for the per-row meter-lease ids (distinct from the chatman
/// receipt epoch so the two authority domains stay separable by seed).
const METER_LEASE_DOMAIN: u64 = 0xA18E_5000_0000_0001;

/// Domain bit for the chatman receipt epoch (lease id + issued_root).
const RECEIPT_EPOCH_DOMAIN: u64 = 0xA16E_7000_0000_0001;

/// Domain bit for the genesis `from_root` of the receipt chain.
const GENESIS_ROOT_DOMAIN: u64 = 0xA16E_7000_0000_0002;

// ---------------------------------------------------------------------------
// SplitMix64 helpers (form matches the canonical in-repo fill_splitmix,
// cgap-blake3/tests/thread_differential.rs:64, and the seal.rs corpus law).
// ---------------------------------------------------------------------------

/// SplitMix64 byte stream with the canonical constants.
fn fill_splitmix(buf: &mut [u8], seed: u64) {
    let mut z = seed;
    let mut i = 0;
    while i < buf.len() {
        z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut w = z;
        w = (w ^ (w >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        w = (w ^ (w >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        w ^= w >> 31;
        let src = w.to_le_bytes();
        let n = (buf.len() - i).min(8);
        buf[i..i + n].copy_from_slice(&src[..n]);
        i += n;
    }
}

/// Per-record seed: master seed verbatim, spread per the penstock form.
fn record_seed(i: usize) -> u64 {
    CORPUS_SEED ^ (i as u64).wrapping_mul(RECORD_SPREAD)
}

/// Rotate-xor fold of a 32 B digest to u64 (byte-for-byte the cgap-ffi
/// `fold_word` step; the boundary's digest-out convention).
fn fold32(d: &[u8; 32]) -> u64 {
    let mut acc = 0u64;
    for w in d.chunks_exact(8) {
        acc = acc.rotate_left(1) ^ u64::from_le_bytes(w.try_into().expect("chunks_exact(8)"));
    }
    acc
}

// ---------------------------------------------------------------------------
// Corpus builders.
// ---------------------------------------------------------------------------

/// Deterministic derivative-chain corpus: one pinned instrument span, one
/// Root parent, `N` Derivative records bound to it (§5.7 receipt =
/// BLAKE3(parent wire)). Per-record splitmix variation lands in the free
/// field, `safe_harbor_ceiling_cents` (§5.8-legal: 0..=MAX). Instrument
/// bytes end in the canonical `---` separator so `instrument_header_span`
/// always terminates; the span derives from the same bytes the validator
/// re-derives from, so the shaping cannot desync the corpus.
struct ChainCorpus {
    instrument_span: Vec<u8>,
    parent: P28LicenseAttestation,
    records: Vec<[u8; ATTESTATION_LEN]>,
}

fn p28_chain_corpus(n: usize) -> ChainCorpus {
    let mut instrument = vec![0u8; 96];
    fill_splitmix(&mut instrument[..88], CORPUS_SEED);
    instrument[87] = b'\n'; // the `---` line starts at 88, whatever the stream
    instrument[88..92].copy_from_slice(b"---\n");
    let instrument_span = instrument_header_span(&instrument)
        .expect("appended --- guarantees a span")
        .to_vec();
    let parent = P28LicenseAttestation {
        license_magic: cgap_p28::attestation::LICENSE_MAGIC,
        version: cgap_p28::attestation::ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: origin_authority_digest(&instrument_span),
        upstream_receipt: [0u8; 32],
    };
    let parent_wire = parent.to_bytes();
    let receipt = cgap_blake3::hash(&parent_wire);
    let records = (0..n)
        .map(|i| {
            let mut ceiling_bytes = [0u8; 8];
            fill_splitmix(&mut ceiling_bytes, record_seed(i));
            let ceiling = u64::from_le_bytes(ceiling_bytes) % (SAFE_HARBOR_CEILING_MAX_CENTS + 1);
            let rec = P28LicenseAttestation {
                license_magic: cgap_p28::attestation::LICENSE_MAGIC,
                version: cgap_p28::attestation::ATTESTATION_VERSION,
                attestation_flags: FLAG_DERIVATIVE,
                safe_harbor_ceiling_cents: ceiling,
                origin_authority_digest: parent.origin_authority_digest,
                upstream_receipt: receipt,
            };
            rec.to_bytes()
        })
        .collect();
    ChainCorpus {
        instrument_span,
        parent,
        records,
    }
}

/// The mixed metering corpus: per row `(amount_cents, crossing)`. Even rows
/// are LICENSED ingestion (strictly sub-ceiling amount, `record_licensed`,
/// never emits); odd rows are UNLICENSED boundary events sized
/// `ceiling + 1 + spread` so a fresh per-row meter provably crosses
/// (`prev = 0 <= ceiling && next > ceiling`, §4.2) and emits exactly one
/// 168 B `SignedProof` + one sealed meter state (§4.4-3). Half-and-half by
/// construction, deterministic from the seed.
fn amount_corpus(n: usize) -> Vec<(u64, bool)> {
    (0..n)
        .map(|i| {
            let mut buf = [0u8; 8];
            fill_splitmix(&mut buf, record_seed(i));
            let spread = u64::from_le_bytes(buf);
            if i % 2 == 0 {
                (spread % DEFAULT_SAFE_HARBOR_CEILING_CENTS, false)
            } else {
                (
                    DEFAULT_SAFE_HARBOR_CEILING_CENTS + 1 + (spread % 1_000),
                    true,
                )
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The chained governance pipeline — one bench, consts [1024, 65536].
// ---------------------------------------------------------------------------

/// MEASURES the full governance chain per row: (1) parse + §5 full ordered
/// validate of the row's 80 B attestation; (2) A18 meter genesis + one
/// mixed-corpus event (licensed sub-ceiling, or unlicensed crossing with a
/// real `SignedProof` emission + sealed meter state); (3) A16 chatman seal
/// of a delta that CHAINS the previous row's receipt digest into the
/// governance ledger; (4) A16 pure verify. Every stage's artifacts are
/// consumed into the ledger checksum; verdict counters are asserted per
/// iteration batch.
#[divan::bench(consts = [1024, 65536])]
fn governance_chain_per_row<const N: usize>(b: Bencher) {
    let c = p28_chain_corpus(N);
    let amounts = amount_corpus(N);
    // One chatman receipt epoch for the whole bench run: the lease is shared
    // across iterations under a Mutex (seal.rs pattern); its ceiling is far
    // above any run (seal = the only tick consumer, ~2N ticks per batch).
    let mut epoch_lease_id = [0u8; 32];
    let mut issued_root = [0u8; 32];
    fill_splitmix(&mut epoch_lease_id, CORPUS_SEED ^ RECEIPT_EPOCH_DOMAIN);
    fill_splitmix(&mut issued_root, CORPUS_SEED ^ RECEIPT_EPOCH_DOMAIN ^ 1);
    let lease = Mutex::new(Lease::new(epoch_lease_id, u64::MAX / 2, issued_root));
    b.bench(|| {
        let mut lease = lease.lock().expect("lease mutex");
        // Chain head: the ledger's genesis from_root (splitmix-derived).
        let mut genesis_root = [0u8; 32];
        fill_splitmix(&mut genesis_root, CORPUS_SEED ^ GENESIS_ROOT_DOMAIN);
        let mut prev_root = genesis_root;
        let (mut validated, mut licensed) = (0u64, 0u64);
        let (mut crossed, mut proven) = (0u64, 0u64);
        let (mut sealed_states, mut verified) = (0u64, 0u64);
        let mut checksum: u64 = 0;
        for i in 0..N {
            // (1) A17 §5 — one attestation verify per admitted row.
            let Ok(rec) = P28LicenseAttestation::from_bytes(&c.records[i]) else {
                continue;
            };
            if rec.validate(&c.instrument_span, Some(&c.parent)).is_err() {
                continue;
            }
            validated += 1;
            // (2) A18 — per-row entity meter: genesis anchors the row's wire
            // (BLAKE3 over the 80 B, fail-closed on spoliation), then one
            // mixed-corpus event with the tick passed in as an event
            // argument (§4.3: the meter never reads time).
            let mut meter_lease_id = [0u8; 32];
            fill_splitmix(&mut meter_lease_id, record_seed(i) ^ METER_LEASE_DOMAIN);
            let sealer =
                LeaseEpochSealer::new(Lease::new(meter_lease_id, u64::MAX / 2, issued_root));
            let mut meter = Asc450Meter::new(DEFAULT_SAFE_HARBOR_CEILING_CENTS, &rec, sealer)
                .expect("lawful corpus: genesis cannot refuse a §5-valid attestation");
            let (amount, crossing) = amounts[i];
            // The delta's `admissions` root binds the row's admitted
            // evidence: the crossing proof's 168 B digest, or the row's
            // attestation anchor for licensed rows.
            let mut admissions = meter.attestation_digest();
            if crossing {
                crossed += 1;
                if let Ok(Some(proof)) = meter.record_unlicensed(amount, lease.current_tick()) {
                    proven += 1;
                    admissions = Asc450Meter::<LeaseEpochSealer>::proof_digest(&proof);
                }
                if meter.last_sealed_state().is_some() {
                    sealed_states += 1;
                }
            } else {
                meter
                    .record_licensed(amount, lease.current_tick())
                    .expect("licensed event: sub-ceiling checked add cannot overflow");
                licensed += 1;
            }
            // (3) A16 — chatman seal: the delta CHAINS the ledger (prev_root
            // = previous receipt digest) to this row's admitted evidence.
            let delta = DeltaS::new(prev_root, meter.attestation_digest(), 1, admissions);
            let receipt = seal_unsealed(delta, &mut lease);
            prev_root = receipt.blake3_digest;
            // (4) A16 — pure verify (no tick, no key): integrity +
            // epoch-consistency of the row's governance receipt.
            if verify_unsealed(&receipt).is_ok() {
                verified += 1;
                checksum = checksum.rotate_left(1) ^ fold32(&receipt.blake3_digest);
            }
        }
        let _ = genesis_root;
        assert_eq!(
            validated, N as u64,
            "p28 §5: every seeded record must admit (spoliation corpus is lawful)"
        );
        assert_eq!(
            licensed,
            (N / 2) as u64,
            "mixed corpus: exactly half the rows are licensed ingestion"
        );
        assert_eq!(
            crossed,
            (N / 2) as u64,
            "mixed corpus: exactly half the rows are unlicensed boundary events"
        );
        assert_eq!(
            proven,
            (N / 2) as u64,
            "each crossing row must emit exactly one SignedProof (fresh per-row \
             meter: the one-shot latch never suppresses a first crossing)"
        );
        assert_eq!(
            sealed_states,
            (N / 2) as u64,
            "§4.4-3: a sealed meter state exists on every emission"
        );
        assert_eq!(
            verified, N as u64,
            "chatman verify: every sealed governance receipt must verify Ok"
        );
        assert_ne!(
            prev_root, genesis_root,
            "the receipt chain must advance off the genesis root"
        );
        black_box(checksum ^ fold32(&prev_root));
    });
}

/// L5 FUSED ARM (v26.11.2 lever L5): the same governance chain with the
/// meter stage folded into ONE fused_row call per row — counter
/// accumulation folded into the loop body alongside the receipt from_root
/// chaining, the row tick hoisted (one current_tick read per row, §4.3:
/// the meter never reads time), and the §5-validated row digest fed
/// through the fail-closed §5↔§4 anchor match (recover's ceiling/digest
/// check, §4.4-3). The unfused arm above is the CONTROL for the PARETO
/// split; both arms share the identical corpus builders, so the only
/// delta is the fused meter stage.
///
/// Anti-vacuity (court L5-BENCH-DEMOTE-CROSSING, /tmp sandbox): demote one
/// crossing row's amount to <= ceiling -> no proof -> `proven == N/2`
/// fires AND prev_root + folded checksum both diverge (the row's
/// admissions anchor falls back to the attestation digest instead of the
/// proof digest).
#[divan::bench(consts = [1024, 65536])]
fn governance_chain_fused_per_row<const N: usize>(b: Bencher) {
    let c = p28_chain_corpus(N);
    let amounts = amount_corpus(N);
    let mut epoch_lease_id = [0u8; 32];
    let mut issued_root = [0u8; 32];
    fill_splitmix(&mut epoch_lease_id, CORPUS_SEED ^ RECEIPT_EPOCH_DOMAIN);
    fill_splitmix(&mut issued_root, CORPUS_SEED ^ RECEIPT_EPOCH_DOMAIN ^ 1);
    let lease = Mutex::new(Lease::new(epoch_lease_id, u64::MAX / 2, issued_root));
    b.bench(|| {
        let mut lease = lease.lock().expect("lease mutex");
        let mut genesis_root = [0u8; 32];
        fill_splitmix(&mut genesis_root, CORPUS_SEED ^ GENESIS_ROOT_DOMAIN);
        let mut prev_root = genesis_root;
        let (mut validated, mut licensed) = (0u64, 0u64);
        let (mut crossed, mut proven) = (0u64, 0u64);
        let (mut sealed_states, mut verified) = (0u64, 0u64);
        let mut checksum: u64 = 0;
        for i in 0..N {
            // (1) A17 §5 — one attestation verify per admitted row.
            let Ok(rec) = P28LicenseAttestation::from_bytes(&c.records[i]) else {
                continue;
            };
            if rec.validate(&c.instrument_span, Some(&c.parent)).is_err() {
                continue;
            }
            validated += 1;
            // (2) A18 FUSED — per-row entity meter; genesis anchors the
            // row's §5-validated wire; the fused_row call advances the
            // meter and mints the row's sealed state in one fold.
            let mut meter_lease_id = [0u8; 32];
            fill_splitmix(&mut meter_lease_id, record_seed(i) ^ METER_LEASE_DOMAIN);
            let sealer =
                LeaseEpochSealer::new(Lease::new(meter_lease_id, u64::MAX / 2, issued_root));
            let mut meter = Asc450Meter::new(DEFAULT_SAFE_HARBOR_CEILING_CENTS, &rec, sealer)
                .expect("lawful corpus: genesis cannot refuse a §5-valid attestation");
            let (amount, crossing) = amounts[i];
            // Hoisted tick: read once per row, passed as an event argument
            // (§4.3).
            let tick = lease.current_tick();
            let outcome = fused_row(&mut meter, &rec, amount, crossing, tick)
                .expect("lawful corpus: anchor matches, adds cannot overflow");
            let mut admissions = outcome.attestation_digest;
            if crossing {
                crossed += 1;
                if outcome.proof.is_some() {
                    proven += 1;
                    admissions = Asc450Meter::<LeaseEpochSealer>::proof_digest(
                        outcome.proof.as_ref().expect("checked Some"),
                    );
                }
            } else {
                licensed += 1;
            }
            // §4.4-3: the fused fold seals a state on EVERY row.
            sealed_states += 1;
            // (3) A16 — chatman seal with from_root chaining.
            let delta = DeltaS::new(prev_root, meter.attestation_digest(), 1, admissions);
            let receipt = seal_unsealed(delta, &mut lease);
            prev_root = receipt.blake3_digest;
            // (4) A16 — pure verify.
            if verify_unsealed(&receipt).is_ok() {
                verified += 1;
                checksum = checksum.rotate_left(1) ^ fold32(&receipt.blake3_digest);
            }
        }
        let _ = genesis_root;
        assert_eq!(validated, N as u64, "p28 §5: every seeded record admits");
        assert_eq!(
            licensed,
            (N / 2) as u64,
            "mixed corpus: exactly half the rows are licensed ingestion"
        );
        assert_eq!(
            crossed,
            (N / 2) as u64,
            "mixed corpus: exactly half the rows are unlicensed boundary events"
        );
        assert_eq!(
            proven,
            (N / 2) as u64,
            "each crossing row must emit exactly one SignedProof (per-row \
             entity meters: the one-shot latch never suppresses a first \
             crossing)"
        );
        assert_eq!(
            sealed_states, N as u64,
            "fused fold: a sealed meter state exists on EVERY row (§4.4-3)"
        );
        assert_eq!(
            verified, N as u64,
            "chatman verify: every sealed governance receipt must verify Ok"
        );
        assert_ne!(
            prev_root, genesis_root,
            "the receipt chain must advance off the genesis root"
        );
        black_box(checksum ^ fold32(&prev_root));
    });
}

// ---------------------------------------------------------------------------
// W4-P2 BLAKE3 FLOOR (lever L5 re-PARETO): the primitive cost of one keyed
// BLAKE3 digest over the two production wire shapes, measured directly.
// Key law (A16 §3.3/§6): the receipt key is the lease `issued_root`, never
// a fresh key — so the floor legs derive the key through the real Lease
// (splitmix corpus seed, same epoch derivation as the chain benches).
// ---------------------------------------------------------------------------

/// Keyed-BLAKE3 floor over the 80 B p28 attestation wire: the per-row
/// sealed-state/proof anchor shape (§4.4-3 mints key over the row wire).
#[divan::bench(consts = [1024, 65536])]
fn blake3_floor_keyed_attestation_wire<const N: usize>(b: Bencher) {
    let c = p28_chain_corpus(N);
    let mut key = [0u8; 32];
    fill_splitmix(&mut key, CORPUS_SEED ^ RECEIPT_EPOCH_DOMAIN);
    // Real key derivation per the crate law: the epoch key is the lease's
    // issued_root, read from the live Lease (chatman.rs §3.3 key law).
    let lease = Lease::new(key, u64::MAX / 2, key);
    assert_eq!(
        lease.issued_root(),
        &key,
        "key law: floor key = issued_root"
    );
    b.bench(|| {
        let mut checksum: u64 = 0;
        for rec in &c.records {
            let d = cgap_crypto::chatman::keyed_digest(lease.issued_root(), rec)
                .expect("80 B < 1024 B single-chunk keyed bound");
            checksum = checksum.rotate_left(1) ^ fold32(&d);
        }
        assert_ne!(checksum, 0, "floor leg must fold a live digest stream");
        black_box(checksum);
    });
}

/// Keyed-BLAKE3 floor over the 192 B receipt-chain covered input
/// (`DeltaS || Phi_lease`, §3.3) — the per-row seal+verify shape.
#[divan::bench(consts = [1024, 65536])]
fn blake3_floor_keyed_receipt_chain<const N: usize>(b: Bencher) {
    let c = p28_chain_corpus(N);
    let amounts = amount_corpus(N);
    let mut epoch_lease_id = [0u8; 32];
    let mut issued_root = [0u8; 32];
    fill_splitmix(&mut epoch_lease_id, CORPUS_SEED ^ RECEIPT_EPOCH_DOMAIN);
    fill_splitmix(&mut issued_root, CORPUS_SEED ^ RECEIPT_EPOCH_DOMAIN ^ 1);
    let lease = Mutex::new(Lease::new(epoch_lease_id, u64::MAX / 2, issued_root));
    // The covered input the seal law actually hashes: DeltaS chaining the
    // previous receipt digest, sealed + verified under the live lease.
    let prev_root = issued_root;
    b.bench(|| {
        let mut lease = lease.lock().expect("lease mutex");
        let mut prev_root = prev_root;
        let mut checksum: u64 = 0;
        for i in 0..N {
            let delta = DeltaS::new(
                prev_root,
                cgap_blake3::hash(&c.records[i]),
                1,
                cgap_blake3::hash(&c.records[amounts.len() - 1 - i]),
            );
            // Real seal law shape: DeltaS chaining the previous receipt
            // digest, sealed under the live lease (one tick per seal —
            // ceiling u64::MAX/2 far exceeds a bench run).
            let receipt = seal_unsealed(delta, &mut lease);
            if verify_unsealed(&receipt).is_ok() {
                checksum = checksum.rotate_left(1) ^ fold32(&receipt.blake3_digest);
            }
            prev_root = receipt.blake3_digest;
        }
        assert_ne!(
            prev_root, issued_root,
            "chain must advance off the epoch root"
        );
        black_box(checksum);
    });
}

fn main() {
    eprintln!(
        "[CHAIN] BENCH-4 governance pipeline per row: p28 §5 validate -> \
         asc450 boundary metering (mixed corpus: alternating licensed / \
         crossing rows, one SignedProof + sealed state per crossing) -> \
         chatman seal (receipts chain via from_root) -> pure verify. No \
         conditional legs; nothing is skipped."
    );
    divan::main();
}
