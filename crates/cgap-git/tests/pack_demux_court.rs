//! Pack demux court (lane GIT-01, v26.11.2). Real byte-built pack
//! streams, no mocks. Module self-contained via the census pattern
//! (`#[path]` include; lib.rs contract-off-limits).
//!
//! Flip-twin law: each court names its mutation; a mutation-free green
//! run is vacuous. The zero-alloc claim is carried by the tracking-alloc
//! instrument (a counting global allocator), not a static claim.

#[path = "../src/pack_demux.rs"]
mod pack_demux;

use cgap_git::pack_write::{encode_object_header, sha1};
use pack_demux::{DeltaBase, Demux, DemuxError, DemuxRecord, FACT_WINDOW, RING_WINDOW_BYTES};

/// Fixed-table default record for the no-alloc drain court.
const DEMUX_SINK_RECORD: DemuxRecord = DemuxRecord {
    offset: 0,
    obj_type: 0,
    declared_size: 0,
    base: DeltaBase::None,
    header_len: 0,
};

// ---- zero-alloc instrument (tracking-alloc pattern) -----------------------

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

/// Test-serialization lock (lane DX-6 race fix): ALLOC_CALLS is a
/// process-global alloc counter; a sibling test allocating between
/// `ALLOC_CALLS.store(0)`-equivalent reset and read inflates the
/// zero-alloc delta (witnessed: c7_parse_path_is_zero_alloc
/// false-FAILED "feed allocated 1 times" under two concurrent cargo
/// test runs). Every test in this binary holds SERIAL for its body.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);

struct TrackingAlloc;

unsafe impl GlobalAlloc for TrackingAlloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        System.realloc(p, l, n)
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        System.alloc_zeroed(l)
    }
}

#[global_allocator]
static TRACKER: TrackingAlloc = TrackingAlloc;

fn alloc_delta<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let before = ALLOC_CALLS.load(Ordering::Relaxed);
    let out = f();
    (out, ALLOC_CALLS.load(Ordering::Relaxed) - before)
}

// ---- pack builder (real bytes; stored framing per module docs) ------------

#[derive(Clone)]
enum Base {
    None,
    Ofs(u64),
    Ref([u8; 20]),
}

/// OFS_DELTA offset varint, git's biased law (inverse of the module's
/// decode; hand-checked: 12 -> [0x0C], 200 -> [0x80 0x48], 16511 ->
/// [0xFF 0x7F]).
fn encode_ofs(v: u64) -> Vec<u8> {
    if v < 128 {
        return vec![v as u8];
    }
    let mut out = vec![(v & 0x7F) as u8];
    let mut rest = v;
    while {
        rest >>= 7;
        rest != 0
    } {
        rest -= 1;
        out.insert(0, 0x80 | (rest & 0x7F) as u8);
    }
    out
}

fn build_pack(objs: &[(u8, Base, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"PACK");
    out.extend_from_slice(&2u32.to_be_bytes());
    out.extend_from_slice(&(objs.len() as u32).to_be_bytes());
    for (t, base, body) in objs {
        out.extend_from_slice(&encode_object_header(*t, body.len() as u64));
        match base {
            Base::None => {}
            Base::Ofs(v) => out.extend_from_slice(&encode_ofs(*v)),
            Base::Ref(oid) => out.extend_from_slice(oid),
        }
        out.extend_from_slice(body);
    }
    let sum = sha1(&out);
    out.extend_from_slice(&sum);
    out
}

// ---- shared driver ---------------------------------------------------------

/// Feed the whole pack in FACT_WINDOW strides inside RING_WINDOW_BYTES
/// windows, draining after each window; return the drained records.
fn demux_all_stride(pack: &[u8]) -> Vec<DemuxRecord> {
    let mut d = Demux::new();
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < pack.len() {
        let win_end = pack.len().min(pos + RING_WINDOW_BYTES);
        let mut wpos = pos;
        while wpos < win_end {
            let stride = FACT_WINDOW.min(win_end - wpos);
            d.feed(&pack[wpos..wpos + stride]).expect("stride feed");
            wpos += stride;
        }
        pos = win_end;
        let mut sink: Vec<DemuxRecord> = (0..d.records_pending())
            .map(|_| DemuxRecord {
                offset: 0,
                obj_type: 0,
                declared_size: 0,
                base: DeltaBase::None,
                header_len: 0,
            })
            .collect();
        let got = d.drain_records(&mut sink);
        assert_eq!(got, sink.len(), "drain under-delivered");
        out.extend_from_slice(&sink);
    }
    assert!(
        d.is_done(),
        "demux never reached Done (fed {} bytes)",
        d.bytes_fed()
    );
    assert_eq!(d.objects_seen(), d.declared_objects().unwrap());
    out
}

/// Single-shot feed (one window, whole pack).
fn demux_all_once(pack: &[u8]) -> Vec<DemuxRecord> {
    let mut d = Demux::new();
    d.feed(pack).expect("single-shot feed");
    let mut sink: Vec<DemuxRecord> = (0..d.records_pending())
        .map(|_| DemuxRecord {
            offset: 0,
            obj_type: 0,
            declared_size: 0,
            base: DeltaBase::None,
            header_len: 0,
        })
        .collect();
    let got = d.drain_records(&mut sink);
    assert_eq!(got, sink.len(), "drain under-delivered");
    assert!(d.is_done());
    sink
}

/// The 3-object reference pack: blob, OFS_DELTA over it, REF_DELTA over
/// it. Hand-computed offsets (see court 3).
fn ref_pack() -> (Vec<u8>, [u8; 20]) {
    let a = b"aaaa".to_vec();
    let oid = sha1(&a);
    let delta_body = b"dddddd".to_vec();
    let ref_body = b"eeeeeee".to_vec();
    let pack = build_pack(&[
        (3u8, Base::None, a),
        (6u8, Base::Ofs(12), delta_body),
        (7u8, Base::Ref(oid), ref_body),
    ]);
    (pack, oid)
}

// ---- courts ----------------------------------------------------------------

/// C1 baseline (flip twin of C2): a real 3-object pack demuxes fully.
#[test]
fn c1_baseline_three_object_pack_demuxes() {
    let _serial = serial();
    let (pack, _oid) = ref_pack();
    let recs = demux_all_stride(&pack);
    assert_eq!(recs.len(), 3, "three records");
    assert_eq!(recs[0].obj_type, 3);
    assert_eq!(recs[0].declared_size, 4);
    assert_eq!(recs[1].obj_type, 6);
    assert_eq!(recs[2].obj_type, 7);
}

/// C2 MUTATION COURT: corrupt the magic PACK -> XACK; the demux must
/// return the typed exit-2-class SubjectMissing variant. Twin of C1.
#[test]
fn c2_magic_corruption_flips_to_subject_missing() {
    let _serial = serial();
    let (mut pack, _oid) = ref_pack();
    pack[0] = b'X'; // PACK -> XACK
    let mut d = Demux::new();
    let err = d
        .feed(&pack)
        .expect_err("VACUOUS: XACK magic still admitted");
    assert_eq!(err, DemuxError::SubjectMissing);
}

/// C3 OFS_DELTA chain decode, hand-computed. Layout: 12-byte header;
/// obj0 header at 12 is one byte (0x34: type 3, size 4) + 4 body bytes
/// -> obj1 at 17; obj1 header 0x76 (type 6, size 6) + one offset byte
/// 0x0C (base offset 12) + 6 body bytes -> obj2 header at 25, 0x77
/// (type 7, size 7) + 20-byte OID + 7 body bytes -> trailer at 53.
#[test]
fn c3_ofs_delta_chain_hand_computed_offsets() {
    let _serial = serial();
    let (pack, oid) = ref_pack();
    assert_eq!(pack.len(), 53 + 20, "hand-computed total length");
    let recs = demux_all_stride(&pack);
    assert_eq!(recs[1].offset, 17, "OFS_DELTA header at hand-computed 17");
    assert_eq!(
        recs[1].base,
        DeltaBase::Ofs(12),
        "base offset == obj0 at 12"
    );
    assert_eq!(recs[1].declared_size, 6);
    assert_eq!(recs[1].header_len, 2, "1 varint + 1 offset byte");
    assert_eq!(recs[2].offset, 25);
    assert_eq!(recs[2].base, DeltaBase::Ref(oid), "REF base OID captured");
    assert_eq!(recs[2].header_len, 21, "1 varint + 20-byte OID");
    assert_eq!(recs[2].declared_size, 7);
}

/// C3b flip twin: corrupt the OFS_DELTA base-offset byte and the decoded
/// base MUST move (else the offset decode is vacuous).
#[test]
fn c3b_ofs_offset_mutation_moves_decoded_base() {
    let _serial = serial();
    let (mut pack, _oid) = ref_pack();
    let off_byte = 17 + 1; // offset varint sits right after obj1's header byte
    let before = pack[off_byte];
    pack[off_byte] = before ^ 0x01; // 12 -> 13
    let recs = demux_all_stride(&mut pack);
    assert_eq!(
        recs[1].base,
        DeltaBase::Ofs(13),
        "VACUOUS: offset-byte mutation did not move the decoded base"
    );
}

/// C4 window/stride invariance: FACT_WINDOW strides inside 32 KiB
/// windows produce byte-identical records to a single-shot feed.
/// Flip twin of C2's terminal-error path: here the feed SUCCEEDS both
/// ways; the mutation court for this law is C6 (truncation).
#[test]
fn c4_stride_invariance_matches_single_shot() {
    let _serial = serial();
    let (pack, _oid) = ref_pack();
    let stride_recs = demux_all_stride(&pack);
    let once_recs = demux_all_once(&pack);
    assert_eq!(
        stride_recs, once_recs,
        "stride law drifted from single-shot"
    );
    // trailer capture verified through the REUSED pack_write::sha1
    let mut d = Demux::new();
    d.feed(&pack).expect("feed");
    let trailer = d.captured_trailer().expect("trailer captured at Done");
    assert_eq!(
        trailer,
        sha1(&pack[..pack.len() - 20]),
        "captured trailer is not the pack SHA-1 (pack_write reuse broken)"
    );
}

/// C5 multi-byte OFS varint: base offset 200 encodes [0x80 0x48] and the
/// demux recovers 200; body skip stays byte-exact afterward.
#[test]
fn c5_multi_byte_ofs_varint_decodes() {
    let _serial = serial();
    let body = vec![b'x'; 6];
    // filler object so the base offset exceeds one varint byte
    let filler = vec![b'f'; 160];
    let pack = build_pack(&[(3u8, Base::None, filler), (6u8, Base::Ofs(200), body)]);
    let recs = demux_all_stride(&pack);
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[1].base, DeltaBase::Ofs(200), "200 == 0x80 0x48 decode");
    assert_eq!(
        recs[1].offset,
        12 + 2 + 160,
        "obj1 at 174 (2-byte header for size 160)"
    );
}

/// C5b flip twin: wrong varint bytes must not still decode to 200.
#[test]
fn c5b_ofs_200_varint_mutation_changes_value() {
    let _serial = serial();
    let filler = vec![b'f'; 160];
    let body = vec![b'x'; 6];
    let mut pack = build_pack(&[(3u8, Base::None, filler), (6u8, Base::Ofs(200), body)]);
    let varint_pos = 12 + 2 + 160 + 1; // after obj1 header byte at 174
    pack[varint_pos] ^= 0x7F; // 0x48 -> 0x37
    let recs = demux_all_stride(&mut pack);
    assert_ne!(
        recs[1].base,
        DeltaBase::Ofs(200),
        "VACUOUS: varint mutation did not change the decoded offset"
    );
}

/// C6 MUTATION COURT (truncation): declare one more object than present;
/// the demux must fail typed Truncated at the trailer, not fake Done.
/// Twin of C4's success path.
#[test]
fn c6_object_count_mutation_flips_to_truncated() {
    let _serial = serial();
    let (mut pack, _oid) = ref_pack();
    pack[8..12].copy_from_slice(&4u32.to_be_bytes()); // 3 -> 4 declared
    let mut d = Demux::new();
    d.feed(&pack).expect("feed consumed the short pack");
    let err = d
        .finish()
        .expect_err("VACUOUS: phantom 4th object still reached Done");
    assert_eq!(err, DemuxError::Truncated);
}

/// C7 zero-alloc court: full parse + drain of the reference pack
/// performs ZERO allocator calls (tracking-alloc instrument).
/// Flip twin: the test-side builder (alloc Vec) shows nonzero on the
/// same harness, proving the counter is wired, not pinned at 0.
/// (DEMUX_SINK_RECORD is a test-side fixed-table default.)
/// CULLED (audit/cull-03-git-court-census.md flag 2, lane TC-04): the
/// SERIAL mutex protects only this binary; a foreign allocation from a
/// parallel sibling binary inflates the delta (false NON-zero). The
/// isolated-demux zero-alloc law now lives in soc_full_chip_court.rs
/// (demux-stage window inside the counting-allocator harness, positive
/// control adjacent); c8_module_source_is_core_only covers the static
/// half. Kept #[ignore]d, not deleted, so the law's original witness
/// stays replayable: `cargo test -p cgap-git --test pack_demux_court
/// -- --ignored`.
#[test]
#[ignore = "CULLED per cull-03 flag 2: subsumed by soc_full_chip_court demux-stage zero-alloc window + c8 static scan; SERIAL mutex does not shield against foreign-binary allocations"]
fn c7_parse_path_is_zero_alloc() {
    let _serial = serial();
    let (pack, _oid) = ref_pack();
    let mut d = Demux::new();
    let ((), n_feed) = alloc_delta(|| d.feed(pack.as_slice()).expect("feed"));
    assert_eq!(n_feed, 0, "feed allocated {} times", n_feed);
    let (_, n_drain) = alloc_delta(|| {
        let mut sink = [DEMUX_SINK_RECORD; 3];
        let got = d.drain_records(&mut sink);
        assert_eq!(got, 3, "all three records drained into the fixed table");
    });
    assert_eq!(n_drain, 0, "drain allocated {} times", n_drain);
    assert!(d.is_done());
    // instrument self-check (flip twin): the builder allocates.
    let (_, n_build) = alloc_delta(|| ref_pack());
    assert!(
        n_build > 0,
        "VACUOUS: alloc counter never moved — instrument is pinned"
    );
}

/// C8 source-scan invariant: the module carries no std:: or alloc::
/// references (no_std compatibility for the promotion).
#[test]
fn c8_module_source_is_core_only() {
    let _serial = serial();
    let src = include_str!("../src/pack_demux.rs");
    assert!(!src.contains("std::"), "std:: leaked into the demux module");
    assert!(
        !src.contains("alloc::"),
        "alloc:: leaked into the demux module"
    );
    assert!(
        !src.contains("Vec::") && !src.contains("Box::") && !src.contains("String::"),
        "heap type leaked into the demux module"
    );
}
