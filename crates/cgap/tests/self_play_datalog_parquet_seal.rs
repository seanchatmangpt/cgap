//! SP6 — self-play datalog -> parquet seal court (v26.11.2 starDF wave).
//!
//! Self-play loop under court: materialized delta slices (the evaluated
//! datalog lane output, one i32 lane domain [0, 255] at kernel bits 8 —
//! the same domain gate `cgap_commit_parquet` applies) drive the A02
//! sealed-block path: `StaticPageBuffer::writer` -> `write_slab` ->
//! `seal` -> `SealedBlock::flush_to(RingSink)` — the exact owner-API
//! sequence the sovereign commit executes inside its W3-L4 lease
//! (crates/cgap-ffi/src/sovereign.rs `cgap_commit_parquet`, staging
//! latch: `CGAP_OK` = STAGED, rotation drains to the spill handle).
//!
//! RECEIPT (SP6, dispatch contract):
//! (1) seal over the full i32 lane domain seals Ok (surface "exit 0"),
//!     byte-deterministic across replays;
//! (2) alloc-tracking profiler (the `rle.rs alloc_proof` counting-
//!     allocator pattern, MB7 witness class / lane P5) around the
//!     steady-state encode+seal+ring-flush loop: 0 allocs, 0 heap bytes,
//!     with a real rotation drain to a real spill file inside the window;
//! (3) metadata demand over capacity -> typed `RunOverrun`, never a
//!     panic (row-group metadata table: run_len 65 / budget 64;
//!     META_CAP=512 itself is unreachable from the public seal surface —
//!     the 64-row-group table bound fires first, see the receipt note
//!     below);
//! (4) rows > i64::MAX -> typed refusal at the seal boundary (H5 bound,
//!     budget pinned to the i64 wire bound).
//!
//! ANTI-VACUITY: each refusal arm names its mutant witness run in a /tmp
//! sandbox (sha256-exact restore + mtime bump) — see the lane report:
//!   arm (2): alloc inserted into `write_slab` -> alloc court FLIPS;
//!   arm (3): MAX_ROW_GROUPS guard removed -> capacity court FLIPS
//!            (typed refusal becomes a spans[64] index panic);
//!   arm (4): rows bound removed from `FooterSealer::new` -> H5 court
//!            FLIPS (landed court: crates/cgap-parquet/tests/
//!            seal_rows_bound_court.rs, doc-names the same mutation).
//! A court that cannot flip is FAIL; none of these mutants touches the
//! shared tree.
//!
//! DIVERGENCE (recorded): dispatch said "drive the FFI fns directly or
//! re-export". `cgap-ffi` is not a dependency of the `cgap` umbrella
//! crate (crates/cgap/Cargo.toml is outside this lane's ownership), so
//! the court drives the identical owner-API sequence the FFI body
//! executes (sovereign.rs lines 795-807: writer() -> write_slab -> seal
//! -> flush_to ring), with the i32->u32 bit-identity reborrow mirrored
//! explicitly in `i32_lane_domain()`. Same code path, zero copies, real
//! collaborators.
//!
//! No mocks. No Process.sleep. Final state asserted.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use cgap::cgap_parquet;
use cgap_parquet::footer::{BlockSummary, FooterSealer};
use cgap_parquet::mmap_writer::{RingSink, SlabView, StaticPageWriter, MAX_ROW_GROUPS};
use cgap_parquet::ParquetError;

/// The materialized delta slice: the FULL i32 lane domain the sovereign
/// commit gates to [0, 255] (bits 8). i32 bit patterns are reborrows as
/// u32 (value-identity for the gated range, exactly the sovereign path).
fn i32_lane_domain() -> Vec<u32> {
    let mut vals: Vec<u32> = Vec::with_capacity(256);
    for v in 0..=255i32 {
        vals.push(v as u32);
    }
    vals
}

// A counting global allocator (rle.rs `alloc_proof` pattern): thread-local
// counters, so concurrently running sibling tests on other threads cannot
// pollute this court's measurement window. Tracks allocation COUNT and
// total BYTES (the MB7 witness class reports both).
thread_local! {
    static ALLOCS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
    static IN_ALLOC: Cell<bool> = const { Cell::new(false) };
}

struct Counting;

impl Counting {
    fn bump(layout: &Layout) {
        if IN_ALLOC.with(Cell::get) {
            return;
        }
        IN_ALLOC.with(|c| c.set(true));
        ALLOCS.with(|c| c.set(c.get() + 1));
        BYTES.with(|c| c.set(c.get().wrapping_add(layout.size())));
        IN_ALLOC.with(|c| c.set(false));
    }
    fn snapshot() -> (usize, usize) {
        (ALLOCS.with(Cell::get), BYTES.with(Cell::get))
    }
}

// SAFETY: pure pass-through to `System` with a counted bump; each method's
// contract is delegated unchanged.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Self::bump(&layout);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        Self::bump(&layout);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        Self::bump(&layout);
        unsafe { System.alloc_zeroed(layout) }
    }
}

#[global_allocator]
static SP6_COUNTING_ALLOC: Counting = Counting;

/// One steady-state seal transaction: mint the writer from the PRE-LEASED
/// static block, write the slab, seal, flush into the PRE-LEASED ring sink
/// (the per-iteration sequence of the sovereign commit lease). Nothing is
/// created inside the transaction — the lease grant happened in setup.
fn seal_tx<'b>(
    buf: &'b mut [u8],
    lanes: &[u32],
    ring: &mut RingSink,
) -> Result<usize, ParquetError> {
    let slab = SlabView {
        lanes: &[lanes],
        rows: lanes.len() as u32,
        bits: 8,
    };
    let mut writer: StaticPageWriter<'_> = StaticPageWriter::new(buf);
    writer.write_slab(&slab)?;
    let sealed = writer.seal(None)?;
    let n = sealed.bytes().len();
    sealed.flush_to(ring)?;
    Ok(n)
}

/// (1) SELF-PLAY SEAL: full i32 lane domain seals Ok — the surface-level
/// "exit 0" — and is byte-deterministic across replays (byte-identical
/// replay law). Also stages through a real pre-leased ring + real spill
/// file: the staging latch the Python witness observed (0-byte spill file
/// pre-rotation) becomes observable here after a forced rotation drain.
#[test]
fn self_play_seal_over_i32_lane_domain() {
    let domain = i32_lane_domain();
    assert_eq!(domain.len(), 256);
    // Widen the slice: 4096 rows drawn from the domain with a literal-
    // heavy stride (a plain repeat would run-code to ~4 KiB), so each
    // sealed slab is ~16 KiB and the loop crosses the 64 KiB ring.
    let lanes: Vec<u32> = (0..16384).map(|i| domain[(i * 7 + 1) % 256]).collect();
    // 8 slabs x ~16 KiB sealed each > the 64 KiB ring: a real rotation
    // drain to the real spill file happens inside the loop.
    let mut block = vec![0u8; 65_536];
    let spill_path = std::env::temp_dir().join("sp6-self-play-seal.spill");
    // Fresh spill file per run (the lease grant's only File::create).
    let _ = std::fs::remove_file(&spill_path);
    let spill = std::fs::File::create(&spill_path).expect("spill file");
    let mut ring = RingSink::new(Some(spill));

    let mut tx_bytes = 0usize;
    for _ in 0..8 {
        tx_bytes += seal_tx(&mut block, &lanes, &mut ring).expect("seal over i32 domain");
    }
    assert!(
        tx_bytes > RingSink::CAP,
        "loop must exceed the ring to force rotation (tx_bytes={tx_bytes})"
    );

    // Final state: rotation drained committed bytes to the spill file.
    let spilled = std::fs::metadata(&spill_path)
        .expect("spill metadata")
        .len();
    assert!(
        spilled > RingSink::CAP as u64,
        "rotation must have drained past one ring capacity to the spill file (spilled={spilled})"
    );

    // Byte-identical replay: the same slab seals to the same bytes+digest.
    let mut b1 = vec![0u8; 65_536];
    let mut b2 = vec![0u8; 65_536];
    let s1 = {
        let mut w = StaticPageWriter::new(&mut b1);
        w.write_slab(&SlabView {
            lanes: &[&domain],
            rows: 256,
            bits: 8,
        })
        .expect("write 1");
        w.seal(None).expect("seal 1")
    };
    let (bytes1, digest1) = (s1.bytes().to_vec(), *s1.digest());
    let s2 = {
        let mut w = StaticPageWriter::new(&mut b2);
        w.write_slab(&SlabView {
            lanes: &[&domain],
            rows: 256,
            bits: 8,
        })
        .expect("write 2");
        w.seal(None).expect("seal 2")
    };
    assert_eq!(*s2.digest(), digest1, "digest must replay byte-identically");
    assert_eq!(
        s2.bytes(),
        bytes1.as_slice(),
        "sealed bytes must replay byte-identically"
    );
    assert!(bytes1.len() > 128, "sealed block carries body + trailer");
    let _ = std::fs::remove_file(&spill_path);
}

/// (2) ZERO-ALLOC STEADY STATE (MB7 witness class, lane P5): around the
/// steady-state encode+seal+ring-flush loop the profiler must observe 0
/// allocations and 0 heap bytes. The lease grant (static block, ring,
/// spill handle, slab domain vector) is ALL inside the setup phase, before
/// the measurement window; a real rotation drain to the real spill file
/// happens inside the window.
#[test]
fn seal_steady_state_is_zero_alloc() {
    let domain = i32_lane_domain();
    // 16384 literal-heavy rows per slab: ~16.4 KiB sealed per
    // transaction, so each window includes a real rotation drain.
    let lanes: Vec<u32> = (0..16384).map(|i| domain[(i * 7 + 1) % 256]).collect();
    let mut block = vec![0u8; 65_536];
    let spill_path = std::env::temp_dir().join("sp6-zero-alloc.spill");
    let _ = std::fs::remove_file(&spill_path);
    let spill = std::fs::File::create(&spill_path).expect("spill file");
    let mut ring = RingSink::new(Some(spill));
    // Warm one transaction OUTSIDE the window (lazy paths paid here).
    seal_tx(&mut block, &lanes, &mut ring).expect("warm-up tx");

    let (a0, b0) = Counting::snapshot();
    let mut tx_bytes = 0usize;
    for _ in 0..16 {
        tx_bytes += seal_tx(&mut block, &lanes, &mut ring).expect("steady-state tx");
    }
    let (a1, b1) = Counting::snapshot();
    assert!(
        tx_bytes > RingSink::CAP,
        "window must include a real rotation drain"
    );
    assert_eq!(
        a1 - a0,
        0,
        "allocations inside the steady-state seal window (MB7/P5 zero-alloc law)"
    );
    assert_eq!(
        b1.wrapping_sub(b0),
        0,
        "heap bytes inside the steady-state seal window"
    );
    let _ = std::fs::remove_file(&spill_path);
}

/// (3) CAPACITY COURT (P5 class): metadata demand over capacity is a
/// typed `RunOverrun`, never a panic. The row-group metadata table
/// (`spans: [u32; MAX_ROW_GROUPS]`, 64 slots) is the reachable capacity
/// bound from the public seal surface; the raw META_CAP=512 scratch is
/// not reachable from it — with at most 64 row groups the emitted meta
/// block is arithmetically capped at ~380 bytes (fixed fields ~34 +
/// 64 spans x ~5 varint bytes), so the row-group bound fires first.
/// Recorded as a receipt finding, not a silent scope drop.
#[test]
fn metadata_capacity_overrun_is_typed() {
    let lanes = vec![7u32; 64]; // a small lawful slab per row group
    let mut block = vec![0u8; 65_536];
    let mut last: Option<ParquetError> = None;
    let mut accepted = 0usize;
    let slab = SlabView {
        lanes: &[&lanes],
        rows: 64,
        bits: 8,
    };
    // ONE writer accumulating row groups — the state carrier is the
    // writer's row-group table, not a per-iteration re-mint.
    let mut writer = StaticPageWriter::new(&mut block);
    for _ in 0..=MAX_ROW_GROUPS {
        match writer.write_slab(&slab) {
            Ok(()) => accepted += 1,
            Err(e) => {
                last = Some(e);
                break;
            }
        }
    }
    let err = last.expect("row-group capacity must refuse past MAX_ROW_GROUPS");
    assert_eq!(
        accepted, MAX_ROW_GROUPS,
        "exactly MAX_ROW_GROUPS row groups must be accepted"
    );
    // Typed, distinguishable: the table bound, not a buffer bound.
    match err {
        ParquetError::RunOverrun { run_len, budget } => {
            assert_eq!(
                run_len,
                MAX_ROW_GROUPS + 1,
                "run_len reports the offending row group"
            );
            assert_eq!(
                budget, MAX_ROW_GROUPS,
                "budget pins the metadata table capacity"
            );
        }
        e => panic!("unexpected error shape: {e:?}"),
    }
}

/// (4) H5 BOUND: rows > i64::MAX must be a typed refusal at the seal
/// boundary — never Ok, never a wrap-negative wire value. Same arm as the
/// landed H5 court (crates/cgap-parquet/tests/seal_rows_bound_court.rs),
/// re-driven here through the umbrella crate's re-export.
#[test]
fn seal_refuses_rows_above_i64_wire_bound() {
    let mut buf = [0u8; 128];
    let summary = BlockSummary {
        row_groups: 1,
        rows: i64::MAX as u64 + 1,
        spans: &[4u32],
    };
    let err = FooterSealer::new(&mut buf, 4, summary, None)
        .err()
        .expect("rows > i64::MAX must be refused, never Ok");
    match err {
        ParquetError::RunOverrun { run_len, budget } => {
            assert_eq!(budget, i64::MAX as usize, "budget pins the i64 wire bound");
            assert!(run_len > budget, "run_len reports the offending row count");
        }
        e => panic!("unexpected error shape: {e:?}"),
    }
}
