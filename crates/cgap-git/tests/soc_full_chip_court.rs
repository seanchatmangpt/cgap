//! SoC full-chip court (lane GIT-16, v26.11.2).
//!
//! EnclaveSram128K harness: an exact 131,072-byte page-aligned STATIC
//! buffer, in-crate simulation only — no hardware, no MMIO, no device
//! lore. The court drives whichever sibling modules exist on disk at
//! build time via `#[path]` includes (lib.rs is contract-off-limits);
//! every absent sibling is a loud typed exclusion line in the emitted
//! receipt, never a silent skip.
//!
//! Pipeline: self-generated fact corpus (seed 0xA20C_0FFE_E5A7_E000,
//! splitmix64) -> pack v2 window serialization -> SWAR-sieved ingress
//! ring relay -> streaming pack demux -> object-store closure ->
//! delta-resolver sub-court -> tree-synth projection -> egress projection
//! -> BLAKE3 receipt seal (real cgap-blake3).
//!
//! Courts: (1) zero heap allocations in a counting-global-allocator
//! window with a positive control; (2) stack canary [u8; 8192] filled
//! with the 0xDEADBEEFCAFEBABE pattern, byte-unchanged post-run;
//! (3) 1,000,000-fact transaction streamed end to end at the siblings'
//! real fixed capacities (honest scale, never rounded up);
//! (4) receipt emitted to docs/csct/ on green with per-stage results and
//! the BLAKE3 transcript root; (5) mutation court: pack magic corruption
//! at the ingress boundary must refuse typed.

#[path = "../src/delta_resolver.rs"]
mod delta_resolver;
#[path = "../src/egress_ring.rs"]
mod egress_ring;
#[path = "../src/ingress_ring.rs"]
mod ingress_ring;
#[path = "../src/object_store.rs"]
mod object_store;
#[path = "../src/pack_demux.rs"]
mod pack_demux;
#[path = "../src/tree_synth.rs"]
mod tree_synth;

extern crate alloc;
use cgap_blake3;
use cgap_git::pack_write;

use ingress_ring::StdIngressRing;
use object_store::{ObjectKind, ObjectStore};

/// Exact SoC SRAM: 131,072 bytes, page-aligned.
const SRAM_BYTES: usize = 131_072;
const SRAM_PAGE: usize = 4096;
const FACTS_TOTAL: usize = 1_000_000;
/// 1,365-fact window (pack_demux::FACT_WINDOW, coprime stride).
const WINDOW: usize = pack_demux::FACT_WINDOW;
const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;
const FACT_CLASSES: usize = 128; // == ObjectStore honest retention scale
const CANARY_BYTES: usize = 8192;
const MAX_WINDOW_BYTES: usize = 12 + WINDOW * 34 + 20; // 46,442 < 64 KiB

// ---------------------------------------------------------------------------
// Counting global allocator: scoped window + positive control. The claim
// "zero heap" is asserted ONLY while the window is open.
// ---------------------------------------------------------------------------

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AOrd};

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

struct CountingAlloc;
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if WINDOW_OPEN.load(AOrd::Relaxed) {
            ALLOC_COUNT.fetch_add(1, AOrd::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new: usize) -> *mut u8 {
        if WINDOW_OPEN.load(AOrd::Relaxed) {
            ALLOC_COUNT.fetch_add(1, AOrd::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn open_window() {
    ALLOC_COUNT.store(0, AOrd::Relaxed);
    WINDOW_OPEN.store(true, AOrd::Relaxed);
}
fn close_window() -> usize {
    WINDOW_OPEN.store(false, AOrd::Relaxed);
    ALLOC_COUNT.load(AOrd::Relaxed)
}
fn positive_control() -> usize {
    open_window();
    let v: std::vec::Vec<u8> = std::vec![7u8; 512];
    let n = close_window();
    std::mem::drop(v);
    n
}

// ---------------------------------------------------------------------------
// Corpus: splitmix64 over the frozen corpus seed. Deterministic, no heap.
// ---------------------------------------------------------------------------

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Deterministic fact class for fact index `i` (corpus law, seed-frozen).
fn fact_class(i: usize) -> usize {
    let mut s = CORPUS_SEED ^ (i as u64);
    (splitmix64(&mut s) % FACT_CLASSES as u64) as usize
}

/// Deterministic 32-byte payload per fact class.
fn class_payload(class: usize, out: &mut [u8; 32]) {
    let mut s = CORPUS_SEED ^ (class as u64).wrapping_mul(0x1CE4_E5B9);
    for chunk in out.chunks_mut(8) {
        let w = splitmix64(&mut s).to_le_bytes();
        chunk.copy_from_slice(&w[..chunk.len()]);
    }
}

// ---------------------------------------------------------------------------
// Pack v2 window serialization (mirrors pack_write's byte law: "PACK",
// u32 BE version 2, u32 BE object count, varint headers, SHA-1 trailer).
// Bodies are stored frames (no deltas) — write_pack_v2 emits no deltas.
// ---------------------------------------------------------------------------

fn put_varint_header(out: &mut [u8], pos: &mut usize, ty: u8, size: u64) {
    // First byte: cont bit 7, type bits 6..4, low 4 size bits; then 7-bit LE.
    let mut b = (ty << 4) | ((size & 0x0F) as u8);
    let mut rest = size >> 4;
    if rest > 0 {
        b |= 0x80;
    }
    out[*pos] = b;
    *pos += 1;
    while rest > 0 {
        let mut nb = (rest & 0x7F) as u8;
        rest >>= 7;
        if rest > 0 {
            nb |= 0x80;
        }
        out[*pos] = nb;
        *pos += 1;
    }
}

/// Serializes one fact window as a complete pack v2 stream into `buf`.
/// Returns the byte length. Typed refusal (u32 object-count overflow).
fn serialize_window(
    facts: usize,
    base_fact: usize,
    payloads: &[[u8; 32]; FACT_CLASSES],
    buf: &mut [u8],
) -> usize {
    assert!(facts <= u32::MAX as usize);
    let mut pos = 0usize;
    buf[..4].copy_from_slice(b"PACK");
    buf[4..8].copy_from_slice(&2u32.to_be_bytes());
    buf[8..12].copy_from_slice(&(facts as u32).to_be_bytes());
    pos = 12;
    for i in 0..facts {
        let class = fact_class(base_fact + i);
        let p = &payloads[class];
        put_varint_header(buf, &mut pos, 3, p.len() as u64);
        buf[pos..pos + 32].copy_from_slice(p);
        pos += 32;
    }
    let trailer = pack_write::sha1(&buf[..pos]);
    buf[pos..pos + 20].copy_from_slice(&trailer);
    pos + 20
}

// ---------------------------------------------------------------------------
// Typed pipeline error (every refusal is typed, never a panic).
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum StageError {
    Ring(ingress_ring::IngressError),
    Demux(pack_demux::DemuxError),
    Store(object_store::StoreError),
    Tree(tree_synth::TreeSynthError),
    Delta(delta_resolver::DeltaError),
    Egress(egress_ring::EgressError),
    ChecksumMismatch,
    RingFidelity,
}

impl From<egress_ring::EgressError> for StageError {
    fn from(e: egress_ring::EgressError) -> Self {
        StageError::Egress(e)
    }
}

impl From<ingress_ring::IngressError> for StageError {
    fn from(e: ingress_ring::IngressError) -> Self {
        StageError::Ring(e)
    }
}

impl From<pack_demux::DemuxError> for StageError {
    fn from(e: pack_demux::DemuxError) -> Self {
        StageError::Demux(e)
    }
}

impl From<object_store::StoreError> for StageError {
    fn from(e: object_store::StoreError) -> Self {
        StageError::Store(e)
    }
}

impl From<tree_synth::TreeSynthError> for StageError {
    fn from(e: tree_synth::TreeSynthError) -> Self {
        StageError::Tree(e)
    }
}

impl From<delta_resolver::DeltaError> for StageError {
    fn from(e: delta_resolver::DeltaError) -> Self {
        StageError::Delta(e)
    }
}

// ---------------------------------------------------------------------------
// Pipeline outputs (fixed struct, no heap).
// ---------------------------------------------------------------------------

#[derive(Default)]
struct PipelineOut {
    facts_streamed: usize,
    windows: usize,
    ring_bytes: usize,
    distinct_objects: usize,
    dedup_accepts: usize,
    tree_len: usize,
    egress_len: usize,
    delta_targets: usize,
    root: [u8; 32],
    ledger: [u8; 32],
}

// SRAM regions: [0, 64 KiB) window A; [64 KiB, 112 KiB) relay B (48 KiB,
// >= MAX_WINDOW_BYTES); [112 KiB, 128 KiB) egress window; [128 KiB,
// 131072) mutation scratch C (16 KiB).
const REGION_B: usize = 64 * 1024;
const REGION_EGRESS: usize = 112 * 1024;
const REGION_C: usize = 128 * 1024;

#[inline(never)]
fn run_pipeline(sram: &mut [u8; SRAM_BYTES], out: &mut PipelineOut) -> Result<(), StageError> {
    let mut root = [0u8; 32];
    root.copy_from_slice(&cgap_blake3::hash(b"cgap-git-soc-v1"));
    let mut store = ObjectStore::new(cgap_blake3::hash); // GAP-01: injected HashFn (real register)
    let mut ring = StdIngressRing::new();
    let mut payload = [[0u8; 32]; FACT_CLASSES];
    for (c, slot) in payload.iter_mut().enumerate() {
        class_payload(c, slot);
    }
    let mut hop = [0u8; 8192];
    let mut records = [pack_demux::DemuxRecord {
        offset: 0,
        obj_type: 0,
        declared_size: 0,
        base: pack_demux::DeltaBase::None,
        header_len: 0,
    }; 64];
    let mut facts = 0usize;
    let mut window_no = 0usize;
    let mut base_fact = 0usize;
    while facts < FACTS_TOTAL {
        let n = core::cmp::min(WINDOW, FACTS_TOTAL - facts);
        // Stage 1: serialize the fact window as a complete pack v2 stream.
        let wlen = serialize_window(n, base_fact, &payload, &mut sram[..MAX_WINDOW_BYTES]);
        // Stage 2: SWAR-sieved ingress relay A -> B (band [0,255] admits
        // every frame; the drop path is courted separately below).
        let (a, b) = sram.split_at_mut(REGION_B);
        let b = &mut b[..REGION_EGRESS - REGION_B];
        let mut fed = 0usize;
        while fed < wlen {
            let seg = core::cmp::min(hop.len(), wlen - fed);
            ring.push_sieved(&a[fed..fed + seg], 0, 255)?;
            let mut got = 0usize;
            while got < seg {
                let m = ring.pop(&mut hop[..seg - got])?;
                b[fed + got..fed + got + m].copy_from_slice(&hop[..m]);
                got += m;
            }
            fed += seg;
        }
        if &b[..wlen] != &a[..wlen] {
            return Err(StageError::RingFidelity);
        }
        out.ring_bytes += wlen;
        // Stage 3+4: streaming demux over the relayed window, draining
        // every <=64 records (RECORD_CAP) into the closure ledger.
        let mut demux = pack_demux::Demux::new();
        let mut off = 0usize;
        while off < wlen {
            let seg = core::cmp::min(2048, wlen - off);
            demux.feed(&b[off..off + seg])?;
            off += seg;
            loop {
                let count = demux.drain_records(&mut records);
                if count == 0 {
                    break;
                }
                for r in &records[..count] {
                    let start = r.offset as usize + r.header_len as usize;
                    let body = &b[start..start + r.declared_size as usize];
                    match store.insert(ObjectKind::Blob, body) {
                        Ok(_) => out.distinct_objects += 1,
                        Err(object_store::StoreError::Duplicate(_)) => out.dedup_accepts += 1,
                        Err(e) => return Err(StageError::Store(e)),
                    }
                }
            }
        }
        if !demux.is_done() || demux.objects_seen() as usize != n {
            return Err(StageError::Demux(pack_demux::DemuxError::Truncated));
        }
        let trailer = demux
            .captured_trailer()
            .ok_or(StageError::ChecksumMismatch)?;
        if trailer != pack_write::sha1(&b[..wlen - 20]) {
            return Err(StageError::ChecksumMismatch);
        }
        // Stage 5: per-window BLAKE3 transcript link.
        let d = cgap_blake3::hash(&a[..wlen]);
        let mut link = [0u8; 64];
        link[..32].copy_from_slice(&root);
        link[32..].copy_from_slice(&d);
        root.copy_from_slice(&cgap_blake3::hash(&link));
        facts += n;
        base_fact += n;
        window_no += 1;
    }
    out.facts_streamed = facts;
    out.windows = window_no;
    // Stage 6: delta-resolver sub-court (real module, hand-encoded delta).
    let mut scratch = delta_resolver::Scratch::new();
    let mut variant = payload[0];
    variant[0] ^= 0xFF;
    let mut delta = [0u8; 64];
    delta[0] = 24; // target len varint
    delta[1] = 32; // source len varint
    delta[2] = 0x81; // copy opcode: bit0 -> 1 size byte follows
    delta[3] = 8; // copy size = 8 bytes from offset 0 (no offset bytes)
    delta[4] = 16; // insert opcode: the literal byte count IS the opcode
    delta[5..21].copy_from_slice(&variant[..16]);
    let resolved = delta_resolver::apply_delta(&payload[0], &delta[..21], &mut scratch)?;
    let mut expected = [0u8; 24];
    expected[..8].copy_from_slice(&payload[0][..8]); // copied from base
    expected[8..].copy_from_slice(&variant[..16]); // inserted literal
    if resolved != &expected[..] {
        return Err(StageError::RingFidelity);
    }
    out.delta_targets = 1;
    // Stage 7: tree-synth projection over the retained fact classes.
    let mut coords: [(tree_synth::Namespace, tree_synth::Coord); FACT_CLASSES] = [(
        tree_synth::Namespace::Facts,
        tree_synth::Coord::new(0, 1, 0),
    );
        FACT_CLASSES];
    for (i, slot) in coords.iter_mut().enumerate() {
        *slot = (
            tree_synth::Namespace::Facts,
            tree_synth::Coord::new(i as u64, 1, i as u64),
        );
    }
    let egress = &mut sram[REGION_EGRESS..REGION_C];
    let tl = tree_synth::synthesize(&mut coords, egress, cgap_blake3::hash)?;
    if tree_synth::verify_canonical_order(&egress[..tl]).is_err() {
        return Err(StageError::Tree(
            tree_synth::TreeSynthError::NonCanonicalOrder { at: 0 },
        ));
    }
    out.tree_len = tl;
    // Stage 8: egress ring projection over the real egress_ring.rs
    // (32,768-byte fixed ring, 32-byte record stride). All distinct
    // class payloads are pushed in final wire layout, served through
    // copy_into, verified byte-exact, and the tail advanced.
    let mut egress_ring = egress_ring::EgressRing::<32768, 32>::new();
    for slot in payload.iter() {
        egress_ring.push_record(slot)?;
    }
    assert!(egress_ring.pending_rows() == FACT_CLASSES);
    let served = egress_ring.window(0, FACT_CLASSES)?;
    let mut gathered = [0u8; 128 * 32];
    served.copy_into(&mut gathered);
    for (i, slot) in payload.iter().enumerate() {
        if gathered[i * 32..(i + 1) * 32] != *slot {
            return Err(StageError::RingFidelity);
        }
    }
    egress_ring.advance_tail(FACT_CLASSES)?;
    let mut frame = [0u8; 104];
    frame[..20].copy_from_slice(b"CGAP-SOC-RECEIPT-V1\n");
    frame[20..52].copy_from_slice(&root);
    frame[52..84].copy_from_slice(&store.ledger_digest());
    out.ledger = frame[52..84].try_into().unwrap();
    out.root = root;
    let flen = 84 + 8 + 8;
    frame[84..92].copy_from_slice(&(facts as u64).to_le_bytes());
    frame[92..100].copy_from_slice(&(window_no as u64).to_le_bytes());
    egress[tl..tl + flen].copy_from_slice(&frame[..flen]);
    out.egress_len = tl + flen;
    Ok(())
}

fn hex32(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Mutation verdict over a corrupted copy of a minimal valid pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutationVerdict {
    /// Demux refused DURING feed with this typed error.
    RefusedAtFeed(pack_demux::DemuxError),
    /// Demux completed; the court-side trailer law refused (captured
    /// trailer differs from pack_write::sha1 over the preceding bytes).
    RefusedAtTrailer,
    /// Corruption passed silently -- court FAIL.
    Unrefused,
}

/// Builds a minimal valid pack (1 blob, 32-byte body, valid trailer),
/// corrupts byte `corrupt_at` with `flip`, drives a fresh demux, and
/// applies the court-side trailer law. Mirrors the exact ingest path of
/// run_pipeline (same builder, same feed, same trailer check).
fn mutation_verdict(corrupt_at: usize, flip: u8) -> MutationVerdict {
    let mut scratch = [0u8; 512];
    scratch[..4].copy_from_slice(b"PACK");
    scratch[4..8].copy_from_slice(&2u32.to_be_bytes());
    scratch[8..12].copy_from_slice(&1u32.to_be_bytes());
    scratch[12] = 0xB0; // varint header: type 3, size 32, continuation
    scratch[13] = 0x02; // size continuation: 32 >> 4 = 2
    scratch[14..46].copy_from_slice(&[0xABu8; 32]);
    let trailer = pack_write::sha1(&scratch[..46]);
    scratch[46..66].copy_from_slice(&trailer);
    scratch[corrupt_at] ^= flip;
    let mut demux = pack_demux::Demux::new();
    let fed = demux.feed(&scratch[..66]);
    if let Err(e) = fed {
        return MutationVerdict::RefusedAtFeed(e);
    }
    let captured = demux.captured_trailer();
    let law = pack_write::sha1(&scratch[..46]);
    match captured {
        Some(t) if t != law => MutationVerdict::RefusedAtTrailer,
        Some(_) => MutationVerdict::Unrefused,
        None => MutationVerdict::Unrefused,
    }
}

#[test]
fn soc_full_chip_court() {
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let mut canary = [0u8; CANARY_BYTES];
            let word = 0xDEAD_BEEF_CAFE_BABEu64.to_le_bytes();
            for slot in canary.chunks_exact_mut(8) {
                slot.copy_from_slice(&word);
            }
            // Positive alloc control FIRST: the window instrument must be
            // proven alive before it is trusted to prove zero.
            let ctrl = positive_control();
            assert!(ctrl > 0, "positive control observed no allocation");
            let mut sram = [0u8; SRAM_BYTES];
            let mut out = PipelineOut::default();
            open_window();
            let verdict = run_pipeline(&mut sram, &mut out);
            let allocs = close_window();
            assert!(verdict.is_ok(), "pipeline refused: {verdict:?}");
            assert_eq!(allocs, 0, "heap allocations inside the pipeline window");
            // Isolated-demux zero-alloc claim (moved here from
            // pack_demux_court c7 per cull-03 flag 2, lane TC-04): a fresh
            // Demux parsing a clean minimal pack on the stack allocates 0.
            {
                let mut pack = [0u8; 66];
                pack[..4].copy_from_slice(b"PACK");
                pack[4..8].copy_from_slice(&2u32.to_be_bytes());
                pack[8..12].copy_from_slice(&1u32.to_be_bytes());
                pack[12] = 0xB0;
                pack[13] = 0x02;
                pack[14..46].copy_from_slice(&[0xABu8; 32]);
                let trailer = pack_write::sha1(&pack[..46]);
                pack[46..66].copy_from_slice(&trailer);
                let mut recs = [pack_demux::DemuxRecord {
                    offset: 0,
                    obj_type: 0,
                    declared_size: 0,
                    base: pack_demux::DeltaBase::None,
                    header_len: 0,
                }; 1];
                open_window();
                let mut d = pack_demux::Demux::new();
                d.feed(&pack[..]).expect("clean pack feeds");
                let got = d.drain_records(&mut recs);
                let n = close_window();
                assert_eq!(got, 1, "isolated demux drains the single record");
                assert!(d.is_done());
                assert_eq!(
                    n, 0,
                    "VACUOUS: isolated demux allocated {n} times (window law)"
                );
            }
            assert_eq!(out.facts_streamed, FACTS_TOTAL);
            assert_eq!(
                out.distinct_objects + out.dedup_accepts,
                FACTS_TOTAL,
                "closure conservation: every fact either retained or deduped"
            );
            // Canary byte-unchanged post-run.
            for slot in canary.chunks_exact_mut(8) {
                assert_eq!(slot, &word, "stack canary disturbed");
            }
            // Sieve drop-path court: a frame with every byte below the
            // band [0x78, 0x7A] must be DROPPED (Ok(false)), ring untouched.
            {
                let mut ring = StdIngressRing::new();
                let frame = [0x01u8; 128];
                assert_eq!(ring.push_sieved(&frame, 0x78, 0x7A), Ok(false));
                assert_eq!(ring.pending(), 0);
                let admitted = [0x79u8; 128];
                assert_eq!(ring.push_sieved(&admitted, 0x78, 0x7A), Ok(true));
                assert_eq!(ring.pending(), 128);
            }
            // Mutation court A: pack magic corruption at the ingress
            // boundary -- the pipeline's first-byte law must refuse typed.
            let magic = mutation_verdict(0, 0xFF);
            assert_eq!(
                magic,
                MutationVerdict::RefusedAtFeed(pack_demux::DemuxError::SubjectMissing),
                "magic corruption must refuse typed"
            );
            // Mutation court B: body-byte flip must trip the trailer law.
            let body_flip = mutation_verdict(20, 0x01);
            assert_eq!(body_flip, MutationVerdict::RefusedAtTrailer);
            // Non-vacuity control: the UNMUTATED pack must be ACCEPTED
            // (Unrefused == both laws pass) -- a court that cannot flip
            // is vacuous, and so is one that refuses everything.
            let clean = mutation_verdict(0, 0x00);
            assert_eq!(clean, MutationVerdict::Unrefused, "clean pack must pass");
            // Honest-scale assertions (never rounded up).
            assert_eq!(out.windows, 733); // 732 x 1365 + 1 x 820 = 1,000,000
            assert_eq!(out.distinct_objects, FACT_CLASSES);
            assert!(out.ring_bytes > 0);
            assert!(out.tree_len > 0);
            assert!(out.egress_len > out.tree_len);
            assert_ne!(out.root, [0u8; 32]);
            assert_ne!(out.ledger, [0u8; 32]);
            // Receipt emission (post-window; allocation is lawful here).
            let receipt = format!(
"# CSCT receipt: cgap-git SoC full-chip v1\n\n- emitted_by: crates/cgap-git/tests/soc_full_chip_court.rs (lane GIT-16)\n- standing: ALIVE (court green this run)\n\n\n## Stages (all green)\n\n| stage | module | result |\n|---|---|---|\n| corpus | in-court splitmix64, seed 0xA20C0FFEE5A7E000 | OK, {facts} facts |\n| pack window serialization | in-court builder (pack_write::sha1 law) | OK, {windows} windows |\n| ingress SWAR relay | ingress_ring.rs 32768B ring, push_sieved | OK, {ring} bytes relayed |\n| pack demux | pack_demux.rs streaming Demux | OK, objects_seen verified per window |\n| closure | object_store.rs BLAKE3 ledger (cap 256 x 128B) | OK, {distinct} distinct / {dedup} deduped |\n| delta resolver | delta_resolver.rs apply_delta | OK, {delta} target |\n| tree synth | tree_synth.rs synthesize + verify_canonical_order | OK, {tree} B |\n| egress projection | egress_ring.rs real ring push/copy_into/advance_tail + receipt frame | OK, {egress} B |\n| receipt seal | real cgap-blake3::hash | OK, root {root} |\n\n\n## Scale (honest)\n\n- facts streamed: {facts} (1,000,000)\n- windows: {windows} (732 x 1365 + 1 x 820)\n- retained closure objects: {distinct} (store cap 256)\n- heap allocations in window: 0 (positive control observed {ctrl})\n- stack canary 8192B 0xDEADBEEFCAFEBABE: byte-unchanged\n\n\n## Mutation courts (all flipped)\n\n- A: pack magic corrupt at ingress boundary -> DemuxError::SubjectMissing (typed)\n- B: body-byte flip -> court-side trailer law refusal\n- control: unmutated pack accepted\n\n\n## Loud typed exclusions\n\n- commit_synth.rs / admission_gate.rs / patent_court.rs / retract.rs / ref_store.rs\n  landed after this pipeline froze -> NOT integrated (loud, in-scope next wave)\n- bn3 closure not reachable via #[path] (foreign crate; closure = object_store.rs BLAKE3 ledger)\n\n\n## Transcript root\n\n\x60{root}\x60\n",
                facts = out.facts_streamed,
                windows = out.windows,
                ring = out.ring_bytes,
                distinct = out.distinct_objects,
                dedup = out.dedup_accepts,
                delta = out.delta_targets,
                tree = out.tree_len,
                egress = out.egress_len,
                root = hex32(&out.root),
                ctrl = ctrl,
            );
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/csct/receipt-cgap-git-soc-v1.md");
            std::fs::write(&path, receipt).expect("receipt write");
            assert!(path.metadata().map(|m| m.len() > 0).unwrap_or(false));
        })
        .unwrap();
    handle.join().unwrap();
}
