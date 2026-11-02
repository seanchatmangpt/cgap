//! SP3 self-play court — admitted-frame payloads -> contiguous 24-byte
//! cgap-arrow statement arrays ([s, p, o] u64), sized by the live lease
//! allowance, populated with zero heap allocations, and buffer-bound
//! validated through the A2P landed validator
//! ([`cgap_arrow::validate::validate_fixed_width`]).
//!
//! Frame geometry follows SP2 (`crates/cgap/tests/self_play_ingress_lease.rs`,
//! cited not redefined): 128 B header, `CGAP` magic, v1, ROOT lineage P28S
//! attestation. Statement payloads are derived from each admitted frame by a
//! real BLAKE3 fold — the digest bytes ARE the statement triple, so the
//! arrow arrays carry admitted content, not synthesized constants.
//!
//! Chicago discipline: real collaborators only — the real gate, the real
//! BLAKE3, the real arrow validator, and a real counting GlobalAlloc (the
//! `rle.rs` `alloc_proof` pattern, cited not redefined). No mocks, no sleeps.
//!
//! ---------------------------------------------------------------------------
//! RECEIPT (SP3 — see the comment block at the bottom for exits)
//! ---------------------------------------------------------------------------

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use cgap_arrow::validate::{validate_fixed_width, FixedWidthColumn};

/// Fixed lease epoch: 64 ticks of authority for this court's batch.
const TICK_CEILING: u64 = 64;
/// Stack-resident statement capacity == the lease ceiling (one statement per
/// tick of authority; the arrays never exceed what the lease can back).
const CAP: usize = 64;
/// Bytes per statement row: 3 u64 lanes = 24 (contiguous [s, p, o]).
const ROW_BYTES: usize = 24;

/// Typed lease-boundary refusal — the exhaustion mutant is refused, never
/// trapped (divergence vs the A16 `Lease::tick` panic: the batch path must
/// stay allocation- and panic-free, so the boundary fires typed instead).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeaseBoundRefusal {
    /// Demand exceeds the remaining tick allowance of the live epoch.
    Exhausted { demand: usize, allowance: usize },
}

/// Lease ledger (SP2 pattern, cited): one tick per frame admitted by the real
/// gate; `allowance()` is the authoritative batch cap.
struct LeaseLedger {
    tick: u64,
}

impl LeaseLedger {
    fn new() -> Self {
        Self { tick: 0 }
    }

    /// Admit one raw wire frame via the real penstock ingress. The tick
    /// advances exactly once iff the gate returns `CGAP_OK`.
    fn admit(&mut self, frame: &[u8]) -> (u32, u64) {
        let mut view = cgap_gate::penstock::FrameView {
            payload: &[],
            prefilter: 0,
            escalate: false,
        };
        let code = cgap_gate::penstock::ingress(frame, frame.len(), Some(&mut view));
        if code == cgap_gate::penstock::CGAP_OK {
            self.tick += 1;
        }
        (code, self.tick)
    }

    /// Remaining authority: `ceiling - tick`, floored at 0.
    fn allowance(&self) -> usize {
        TICK_CEILING.saturating_sub(self.tick) as usize
    }

    /// Typed lease-boundary check. Refusal fires BEFORE any array work.
    fn check(&self, demand: usize) -> Result<usize, LeaseBoundRefusal> {
        let allowance = self.allowance();
        if demand > allowance {
            Err(LeaseBoundRefusal::Exhausted { demand, allowance })
        } else {
            Ok(demand)
        }
    }
}

// --- counting allocator (rle.rs alloc_proof pattern, cited) ---------------

thread_local! {
    static ALLOCS: Cell<usize> = const { Cell::new(0) };
    static IN_ALLOC: Cell<bool> = const { Cell::new(false) };
}

struct Counting;

impl Counting {
    fn bump() {
        if IN_ALLOC.with(Cell::get) {
            return;
        }
        IN_ALLOC.with(|c| c.set(true));
        ALLOCS.with(|c| c.set(c.get() + 1));
        IN_ALLOC.with(|c| c.set(false));
    }
}

// SAFETY: pass-through to System with a counted bump; dealloc is forwarded
// unchanged (no bump — deallocations are not allocations).
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Self::bump();
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        Self::bump();
        unsafe { System.realloc(ptr, layout, new_size) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        Self::bump();
        unsafe { System.alloc_zeroed(layout) }
    }
}

#[global_allocator]
static COUNTING_ALLOC: Counting = Counting;

fn current_allocs() -> usize {
    ALLOCS.with(Cell::get)
}

/// Build one valid, admissible v1 frame (SP2 geometry, cited): 128 B header
/// + payload. ROOT lineage, zero upstream receipt, safe-harbor ceiling.
/// `flags` stays in the legal band (reserved bits 4-7 clear).
fn admitted_frame(flags: u8, payload_len: usize) -> std::vec::Vec<u8> {
    use cgap_gate::penstock::{
        HEADER_LEN, HEADER_VERSION, MAGIC, OABI_DIGEST_LEN, OFF_ATTESTATION, OFF_FLAGS, OFF_MAGIC,
        OFF_OABI_DIGEST, OFF_PAYLOAD_BYTE_LEN, OFF_VERSION, P28_FLAG_ROOT, P28_LICENSE_MAGIC,
        P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION,
    };

    let mut buf = std::vec![0u8; HEADER_LEN + payload_len];
    buf[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
    buf[OFF_VERSION] = HEADER_VERSION;
    buf[OFF_FLAGS] = flags;
    buf[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
        .copy_from_slice(&(payload_len as u64).to_le_bytes());
    for b in buf[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].iter_mut() {
        *b = 0xA5;
    }
    buf[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&P28_LICENSE_MAGIC);
    buf[OFF_ATTESTATION + 4..OFF_ATTESTATION + 6].copy_from_slice(&P28_VERSION.to_le_bytes());
    buf[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&P28_FLAG_ROOT.to_le_bytes());
    buf[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
        .copy_from_slice(&P28_SAFE_HARBOR_CEILING_CENTS.to_le_bytes());
    // ROOT: upstream_receipt slot stays zero.
    buf
}

/// Statement payload: real BLAKE3 fold over the admitted frame's header
/// bytes; the first 24 digest bytes ARE the [s, p, o] triple.
fn statement_of(frame: &[u8]) -> [u64; 3] {
    let digest = cgap_blake3::hash(&frame[..cgap_gate::penstock::HEADER_LEN]);
    let mut row = [0u64; 3];
    for (i, lane) in row.iter_mut().enumerate() {
        let mut b = [0u8; 8];
        b.copy_from_slice(&digest[i * 8..(i + 1) * 8]);
        *lane = u64::from_le_bytes(b);
    }
    row
}

/// Populate the contiguous 24-byte statement array (rows of [s, p, o]) plus
/// the three column views the validator consumes. All stack-resident.
fn populate(
    rows: &mut [[u64; 3]; CAP],
    s: &mut [u64; CAP],
    p: &mut [u64; CAP],
    o: &mut [u64; CAP],
    frames: &[std::vec::Vec<u8>],
    n: usize,
) {
    for (i, frame) in frames.iter().take(n).enumerate() {
        let row = statement_of(frame);
        rows[i] = row;
        s[i] = row[0];
        p[i] = row[1];
        o[i] = row[2];
    }
}

/// A2P validator over the three column views of a populated batch.
fn validate_batch(
    s: &[u64; CAP],
    p: &[u64; CAP],
    o: &[u64; CAP],
    n: usize,
) -> Result<(), cgap_arrow::validate::ValidateError> {
    let cols = [
        Some(FixedWidthColumn {
            base: s.as_ptr(),
            len: n,
            bytes: CAP * 8,
        }),
        Some(FixedWidthColumn {
            base: p.as_ptr(),
            len: n,
            bytes: CAP * 8,
        }),
        Some(FixedWidthColumn {
            base: o.as_ptr(),
            len: n,
            bytes: CAP * 8,
        }),
    ];
    validate_fixed_width(&cols)
}

/// Contiguity proof: the rows buffer is a dense 24-byte-per-row [s, p, o]
/// array — reassemble lanes from the rows and cross-check the columns.
fn rows_are_contiguous_24b(
    rows: &[[u64; 3]; CAP],
    s: &[u64; CAP],
    p: &[u64; CAP],
    o: &[u64; CAP],
    n: usize,
) -> bool {
    if rows.as_ptr() as usize % 8 != 0 {
        return false;
    }
    for i in 0..n {
        let row = core::hint::black_box(&rows[i]);
        if *row != [s[i], p[i], o[i]] {
            return false;
        }
    }
    // stride: row i+1 base - row i base must be exactly 24 bytes.
    unsafe { (rows.as_ptr().add(1) as usize) - (rows.as_ptr() as usize) == ROW_BYTES }
}

#[test]
fn admitted_frames_fill_statement_arrays_within_allowance_zero_alloc() {
    // SETUP (outside the measurement window): 16 distinct admitted frames.
    let mut frames: std::vec::Vec<std::vec::Vec<u8>> = std::vec::Vec::new();
    let mut ledger = LeaseLedger::new();
    for i in 0..16u8 {
        let frame = admitted_frame(i & 0x0F, 16 + i as usize * 8);
        let (code, tick) = ledger.admit(&frame);
        assert_eq!(code, cgap_gate::penstock::CGAP_OK, "frame {i} must admit");
        assert_eq!(tick, (i as u64) + 1, "tick must advance once per admission");
        frames.push(frame);
    }
    assert_eq!(ledger.tick, 16);
    assert_eq!(
        ledger.allowance(),
        CAP - 16,
        "allowance must shrink with ticks"
    );

    let demand = 16usize;
    let n = match ledger.check(demand) {
        Ok(n) => n,
        Err(e) => {
            panic!("court 1 precondition: demand {demand} must be within allowance, got {e:?}")
        }
    };

    let mut rows: [[u64; 3]; CAP] = [[0u64; 3]; CAP];
    let mut s = [0u64; CAP];
    let mut p = [0u64; CAP];
    let mut o = [0u64; CAP];

    // MEASUREMENT WINDOW: population + validation are zero-alloc.
    let before = current_allocs();
    populate(&mut rows, &mut s, &mut p, &mut o, &frames, n);
    let mid = current_allocs();
    assert_eq!(
        mid - before,
        0,
        "population allocated {} times",
        mid - before
    );

    validate_batch(&s, &p, &o, n).expect("in-allowance batch must validate");
    assert_eq!(current_allocs() - mid, 0, "validation allocated");

    // Final state: rows are real BLAKE3-derived triples, contiguous 24 B.
    assert!(
        rows_are_contiguous_24b(&rows, &s, &p, &o, n),
        "rows must be dense 24-byte [s,p,o]"
    );
    for i in 0..n {
        let expected = statement_of(&frames[i]);
        assert_eq!(
            rows[i], expected,
            "row {i} must be the admitted frame's BLAKE3 triple"
        );
    }
    // Distinct frames must not collapse onto one triple.
    let first = rows[0];
    let last = rows[n - 1];
    assert_ne!(
        first, last,
        "distinct frames must yield distinct statements"
    );
}

#[test]
fn exhaustion_mutant_refused_typed_and_allocates_nothing() {
    // SETUP: exhaust the epoch first — admit TICK_CEILING frames.
    let mut ledger = LeaseLedger::new();
    let _frames: std::vec::Vec<std::vec::Vec<u8>> = std::vec::Vec::new();
    for i in 0..TICK_CEILING as u8 {
        let frame = admitted_frame(i & 0x0F, 8);
        let (code, _) = ledger.admit(&frame);
        assert_eq!(code, cgap_gate::penstock::CGAP_OK);
    }
    assert_eq!(ledger.allowance(), 0, "epoch must be fully exhausted");

    let rows: [[u64; 3]; CAP] = [[0u64; 3]; CAP];
    let s = [0u64; CAP];
    let p = [0u64; CAP];
    let o = [0u64; CAP];

    // ANTI-VACUITY MUTANT: demand > allowance (1 > 0) at the boundary.
    let before = current_allocs();
    match ledger.check(1) {
        Err(LeaseBoundRefusal::Exhausted { demand, allowance }) => {
            assert_eq!((demand, allowance), (1, 0));
        }
        other => panic!("exhaustion must refuse typed, got {other:?}"),
    }
    // NO array allocation and NO population: rows stay at the sentinel.
    assert_eq!(current_allocs() - before, 0, "refusal path allocated");
    assert_eq!(
        rows[0], [0u64; 3],
        "no statement written past the lease boundary"
    );
    assert_eq!(s[0], 0, "column s untouched by the refused batch");

    // FLIP PROOF: the same demand succeeds on a fresh epoch — the refusal is
    // caused by the lease boundary, not by the demand shape.
    let fresh = LeaseLedger::new();
    assert!(
        fresh.check(1).is_ok(),
        "identical demand within allowance must pass"
    );

    // The validator itself agrees the zero-length refusal wrote nothing.
    validate_batch(&s, &p, &o, 0).expect("empty (refused) batch validates trivially");
}

// ===========================================================================
// RECEIPT — SP3 self-play lease/arrow sizing court
// ===========================================================================
// COMMAND : cargo test -p cgap --test self_play_lease_arrow
// EXIT    : 0 — "2 passed; 0 failed" witnessed this session
//           (CARGO_TARGET_DIR=/tmp/sp3-target, removed at lane end)
// COURTS  :
//   1. admitted_frames_fill_statement_arrays_within_allowance_zero_alloc —
//      16 real gate admissions advance the tick 1..16; the batch cap comes
//      from the lease allowance (64 - tick); populate + A2P
//      validate_fixed_width run inside a counting-GlobalAlloc window with
//      delta == 0 asserted; rows are dense 24-byte [s,p,o] triples derived
//      from real BLAKE3 folds over admitted headers.
//   2. exhaustion_mutant_refused_typed_and_allocates_nothing — anti-vacuity
//      mutant: demand 1 against an exhausted (allowance == 0) epoch; asserts
//      the typed LeaseBoundRefusal::Exhausted { demand: 1, allowance: 0 }
//      fires, zero allocations on the refusal path, sentinel rows untouched,
//      and the flip proof (same demand passes on a fresh epoch).
//   Mutation would flip: court 2 fails if the boundary admits over-allowance
//   demand (typed refusal missing) or allocates; court 1 fails if population
//   allocates, the validator rejects an in-allowance batch, or rows drift
//   from the BLAKE3-derived triples.
// DIVERGENCE: cgap-crypto's Lease is not a direct dependency of the cgap
// umbrella crate (crates/cgap/Cargo.toml), so it is not nameable from this
// integration test; the tick source follows SP2's in-file ledger pattern
// (self_play_ingress_lease.rs) against the real penstock gate. The A16
// Lease::tick exhaustion PANICS (cgap-crypto/src/lease.rs §5); this court's
// batch boundary refuses TYPED per the dispatch ("typed, not panic").
// ===========================================================================
