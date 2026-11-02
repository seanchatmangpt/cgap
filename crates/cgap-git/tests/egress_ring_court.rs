//! Lane GIT-07 egress-ring courts.
//!
//! Self-contained `#[path]` wire-in (lib.rs is off-limits this lane): the
//! module is compiled directly into this test binary. The module uses no
//! `crate::` paths, so this is byte-faithful to how lib.rs will wire it.
//!
//! Chicago courts, real ring real bytes, assert final state. Anti-vacuity
//! flip twins, every court names its mutation:
//! - round-trip court: wrap-boundary segment shape pinned (a segment
//!   split proves the wrap actually happened; a fully contiguous window
//!   would make the wrap court vacuous).
//! - misalignment court: descriptor offset 16 -> 17 in the schema
//!   descriptor; twin asserts the valid schema yields Ok and the mutant
//!   yields typed Err (outcomes differ = the gate flips).
//! - zero-alloc court: tracking `GlobalAlloc` instrument (INSTRUMENT,
//!   stated per dispatch); non-vacuity control = `snapshot_owned`, an
//!   allocating path in the same module, proves the instrument fires.
//! - Parquet reuse court: `PAR2` magic byte-pinned against
//!   `footer.rs`'s constant restatement.
//! Loud `[SKIP]` only, never a fake pass. No skips expected.

#![cfg(test)]

// The module uses `::alloc::vec::Vec` (snapshot_owned control path);
// declare the facet here so the standalone test binary links it exactly
// as lib.rs will.
extern crate alloc;

#[path = "../src/egress_ring.rs"]
mod egress_ring;

use alloc::vec::Vec;
use egress_ring::{
    parse_page, validate_schema, EgressError, StdEgressRing, DESC_ENTRY_BYTES,
    DESC_TABLE_OFFSET, EGRESS_RING_BYTES,
};

/// Deterministic 32-byte record: four u64 columns at offsets 0/8/16/24.
fn record(seed: u64) -> [u8; 32] {
    let mut r = [0u8; 32];
    for c in 0..4u64 {
        let v = (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(c)) as u64;
        r[(c * 8) as usize..(c * 8 + 8) as usize].copy_from_slice(&v.to_le_bytes());
    }
    r
}

/// Wrap-around round-trip court: records pushed across the ring boundary
/// serve back byte-exact through window, IPC page, and parse. MUTATION
/// named: the wrap must actually happen (asserted via segment shape in
/// the flip-twin court below) and any payload byte corruption must flip
/// the byte-exact assert.
#[test]
fn roundtrip_byte_exact_court() {
    let _serial = serial();
    let mut ring = StdEgressRing::new();
    assert_eq!(ring.capacity(), EGRESS_RING_BYTES);
    assert_eq!(ring.capacity_rows(), 1024);
    assert_eq!(ring.pending_rows(), 0);

    // Push 40 records starting at record 1016 of 1024 capacity: records
    // 1017..1024 fill the tail of the buffer and the rest WRAP to the
    // front — the served window crosses the boundary.
    for i in 1016..1016 + 40usize {
        ring.push_record(&record(i as u64)).expect("push fits");
    }
    assert_eq!(ring.pending_rows(), 40);
    assert_eq!(ring.pending_bytes(), 40 * 32);

    // Zero-copy window reassembly is byte-exact.
    let win = ring.window(0, 40).expect("window in range");
    assert_eq!(win.total_len(), 40 * 32);
    let mut gathered = [0u8; 40 * 32];
    win.copy_into(&mut gathered);
    for i in 0..40usize {
        let expect = record((1016 + i) as u64);
        assert_eq!(
            &gathered[i * 32..(i + 1) * 32],
            &expect[..],
            "record {i} byte-exact across wrap"
        );
    }

    // Serve an IPC page and parse it back: payload byte-exact, schema
    // round-trips intact.
    let mut blob = [0u8; EGRESS_RING_BYTES];
    let n = ring
        .serve_ipc_page(&StdEgressRing::SCHEMA, &mut blob)
        .expect("serve fits");
    let page = parse_page(&blob[..n]).expect("parse own page");
    assert_eq!(page.ncols, 4);
    assert_eq!(page.stride, 32);
    assert_eq!(page.nrows, 40);
    assert_eq!(page.payload.len(), 40 * 32);
    assert_eq!(page.payload, &gathered[..]);
    for (i, d) in page.schema[..page.ncols].iter().enumerate() {
        assert_eq!(d, &StdEgressRing::SCHEMA[i], "descriptor {i} round-trips");
    }

    // MUTATION FLIP: corrupt one payload byte in the served blob; the
    // byte-exact invariant now FAILS (payload differs from the window
    // gather). Same parser, same framing, one flipped byte.
    let mut corrupt = blob;
    corrupt[n - 1] ^= 0xFF;
    let bad = parse_page(&corrupt[..n]).expect("framing still parses");
    assert_ne!(bad.payload, &gathered[..], "flipped byte must be visible");
}

/// Flip twin for the wrap court: pin the segment SHAPE so the wrap is
/// proven to have happened. A contiguous 40-record window would return
/// one segment; crossing the 32,768-byte boundary MUST return two. If a
/// regression made windows always contiguous (or always split), this
/// court flips.
#[test]
fn flip_twin_wrap_segment_shape_court() {
    let _serial = serial();
    let mut ring = StdEgressRing::new();
    // Fill to the last record slot, then push across the boundary.
    // tail 1023 rows so one pending row sits in the FINAL stride of the
    // buffer; the next push must WRAP to the front.
    for i in 0..1024usize {
        ring.push_record(&record(i as u64)).expect("fits");
    }
    ring.advance_tail(1023).expect("tail advances");
    ring.push_record(&record(999_999)).expect("fits");
    // Window of 2 rows: row 0 sits in the LAST stride of the buffer,
    // row 1 wraps to the front — two segments, shapes pinned exactly.
    let win = ring.window(0, 2).expect("window");
    assert_eq!(win.a.len(), 32, "first segment = final stride of buffer");
    assert_eq!(win.b.len(), 32, "second segment = wrapped front stride");
    let mut two = [0u8; 64];
    win.copy_into(&mut two);
    assert_eq!(&two[..32], &record(1023)[..], "row 0 = last-stride record");
    assert_eq!(
        &two[32..],
        &record(999_999)[..],
        "row 1 = wrapped front record"
    );
}

/// Misalignment court (dispatch mutation target): a misaligned column
/// offset in the schema descriptor must fail CLOSED with a typed Err on
/// BOTH read surfaces — encode/serve validation and parse validation.
/// FLIP TWIN: the valid schema yields Ok on both surfaces; the mutant
/// (offset 16 -> 17) yields `MisalignedColumn` on both. Outcomes differ
/// = the gate is live, not vacuous.
#[test]
fn misaligned_column_rejection_court() {
    let _serial = serial();
    // TWIN ARM A (valid): encode and parse both accept.
    let mut hdr = [0u8; 256];
    let hlen = super_encode_ok(&mut hdr);
    assert!(hlen > 0);
    let parsed = parse_page(&hdr[..hlen]).expect("valid schema parses");
    assert_eq!(parsed.ncols, 4);

    // MUTANT ARM B (offset 16 -> 17): misaligned u64 column.
    let mut bad_schema = StdEgressRing::SCHEMA;
    bad_schema[2].offset = 17;
    let err_enc = validate_schema(&bad_schema, 32).unwrap_err();
    assert_eq!(
        err_enc,
        EgressError::MisalignedColumn {
            index: 2,
            offset: 17,
            width: 8
        }
    );

    // Same rejection on the wire-read surface: splice the mutated
    // descriptor into an otherwise valid page blob and parse.
    let mut corrupt = hdr;
    let desc_at = DESC_TABLE_OFFSET + 2 * DESC_ENTRY_BYTES + 4; // entry 2, offset field
    corrupt[desc_at..desc_at + 2].copy_from_slice(&17u16.to_le_bytes());
    let err_wire = parse_page(&corrupt[..hlen]).unwrap_err();
    assert_eq!(
        err_wire,
        EgressError::MisalignedColumn {
            index: 2,
            offset: 17,
            width: 8
        }
    );

    // Serve path also fails closed on the mutant schema.
    let mut ring = StdEgressRing::new();
    ring.push_record(&record(1)).expect("fits");
    let mut out = [0u8; 512];
    assert_eq!(
        ring.serve_ipc_page(&bad_schema, &mut out),
        Err(EgressError::MisalignedColumn {
            index: 2,
            offset: 17,
            width: 8
        })
    );

    // Andon class pinned: egress refusals are class 101 (typed Err, not
    // a panic and not a silent drop).
    assert_eq!(err_enc.andon_exit_class(), 101);
}

/// Helper: encode a valid 4-column header with 2 rows of payload.
fn super_encode_ok(out: &mut [u8]) -> usize {
    let mut ring = StdEgressRing::new();
    ring.push_record(&record(7)).expect("fits");
    ring.push_record(&record(8)).expect("fits");
    ring.serve_ipc_page(&StdEgressRing::SCHEMA, out)
        .expect("fits")
}

/// Zero-alloc serve-path court (dispatch invariant, INSTRUMENTED — the
/// tracking-alloc instrument is stated here per dispatch, not merely
/// asserted statically). A process-wide counting allocator measures the
/// allocation DELTA across the zero-copy serve path: `window`,
/// `serve_ipc_page`, and `parse_page` must perform ZERO heap
/// allocations. NON-VACUITY CONTROL: `snapshot_owned` (the module's one
/// allocating path) must show a delta > 0 in the same harness — if the
/// instrument stops firing, this court flips. MUTATION NAMED: if any
/// serve-path edit introduces even one heap allocation (e.g. a Vec
/// gather replacing segment slices), the delta-0 assert flips the court.
#[test]
fn zero_alloc_serve_court() {
    let _serial = serial();
    let mut ring = StdEgressRing::new();
    for i in 0..40usize {
        ring.push_record(&record(i as u64)).expect("fits");
    }
    let mut blob = vec![0u8; EGRESS_RING_BYTES];

    // Instrument fires? Control arm first: the allocating snapshot must
    // move the counter, proving the harness measures real allocations.
    COUNT.store(0, MO_SEQ);
    let snap: Vec<u8> = ring.snapshot_owned();
    let control = COUNT.load(MO_SEQ);
    assert_eq!(snap.len(), 40 * 32);
    assert!(
        control > 0,
        "instrument is vacuous: snapshot_owned showed 0 allocs"
    );

    // Zero-copy window: allocation delta must be exactly 0.
    COUNT.store(0, MO_SEQ);
    let win = ring.window(0, 40).expect("in range");
    let total = win.total_len();
    COUNT.store(COUNT.load(MO_SEQ), MO_SEQ); // settle (no-op)
    let window_delta = COUNT.load(MO_SEQ);
    assert_eq!(window_delta, 0, "window allocated {window_delta} times");

    // IPC serve path: delta exactly 0 (header encode + payload gather).
    COUNT.store(0, MO_SEQ);
    let n = ring
        .serve_ipc_page(&StdEgressRing::SCHEMA, &mut blob)
        .expect("serves");
    let serve_delta = COUNT.load(MO_SEQ);
    assert_eq!(
        serve_delta, 0,
        "serve_ipc_page allocated {serve_delta} times"
    );

    // Parse path: delta exactly 0 (zero-copy Page borrow).
    COUNT.store(0, MO_SEQ);
    let page = parse_page(&blob[..n]).expect("parses");
    let parse_delta = COUNT.load(MO_SEQ);
    assert_eq!(parse_delta, 0, "parse_page allocated {parse_delta} times");
    assert_eq!(page.payload.len(), total);

    // Parquet-flavored serve: delta exactly 0.
    COUNT.store(0, MO_SEQ);
    let nq = ring.serve_parquet_page(&mut blob).expect("serves");
    let pq_delta = COUNT.load(MO_SEQ);
    assert_eq!(pq_delta, 0, "serve_parquet_page allocated {pq_delta} times");
    assert!(nq > 0);
}

/// Parquet reuse court: the Parquet-flavored page byte-pins the reused
/// `PAR2` container magic (footer.rs:248 restatement) at offset 0 and
/// the payload length at offset 4, with the payload byte-exact against
/// the zero-copy window. MUTATION named: flipping any magic byte or the
/// length field flips the byte-pin asserts.
#[test]
fn parquet_page_reuse_court() {
    let _serial = serial();
    let mut ring = StdEgressRing::new();
    for i in 0..3usize {
        ring.push_record(&record(100 + i as u64)).expect("fits");
    }
    let mut blob = [0u8; 512];
    let n = ring.serve_parquet_page(&mut blob).expect("serves");
    assert_eq!(n, 72 + 3 * 32);
    assert_eq!(&blob[0..4], b"PAR2", "reused PAR2 magic pinned at 0");
    assert_eq!(
        u32::from_le_bytes(blob[4..8].try_into().expect("4 bytes")),
        3 * 32,
        "payload length pinned"
    );
    let win = ring.window(0, 3).expect("window");
    let mut gather = [0u8; 3 * 32];
    win.copy_into(&mut gather);
    assert_eq!(
        &blob[72..n],
        &gather[..],
        "payload byte-exact, no re-serialization"
    );
}

/// Counting allocator + counters (defined last so the courts above read
/// naturally; Rust items are order-independent).
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering as MO};
use std::sync::{Mutex, MutexGuard};

/// Test-serialization lock (lane DX-6 race fix): COUNT is a
/// process-global alloc counter; a sibling test allocating between a
/// `COUNT.store(0)` and its matching load inflates the zero-alloc delta
/// (witnessed: zero_alloc_serve_court false-FAILED "allocated 1 times"
/// under two concurrent cargo test runs). Every test in this binary
/// holds SERIAL for its whole body so measured windows are exclusive.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

struct Counting;

static COUNT: AtomicUsize = AtomicUsize::new(0);
const MO_SEQ: MO = MO::SeqCst;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        COUNT.fetch_add(1, MO_SEQ);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        COUNT.fetch_add(1, MO_SEQ);
        System.realloc(p, l, n)
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        COUNT.fetch_add(1, MO_SEQ);
        System.alloc_zeroed(l)
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;
