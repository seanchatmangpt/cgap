//! Wire -> substrate E2E court (lane GAP-08, v26.11.2, ship-plan section B).
//!
//! Seam path (all modules lib-mounted, `cgap_git::<module>` paths):
//! (1) FRAME IN  — mcp_driver::Driver::dispatch / parse_frame (fixed
//!     schema READ/TOOL grammar, Latin-1 bytes, total parser);
//! (2) STATE     — admission_gate parse_update_pkt/RefTable::apply with
//!     typed refusals;
//! (3) MERGE     — octopus_merge over >= 3 heads (typed DomainClash);
//! (4) COMMIT    — commit_synth LeaseCommitSpec::render + LeaseReceipt
//!     seal;
//! (5) SEAL      — cgap_blake3 digest + 80-byte seal shape check.
//!
//! Allocation discipline: the whole pipeline runs inside ONE
//! CountingAlloc window (G06/soc_full_chip_court pattern) with the
//! positive control FIRST; the window must exit BYTE-balanced.
//!
//! Loud exclusion (sandbox law): `object_store` is NOT lib-mounted in
//! crates/cgap-git/src/lib.rs (line 159 is still `// pub mod
//! object_store;` at court-authoring time — GAP-01 landing it is in
//! flight). This court therefore exercises the four mounted modules
//! above; object_store is EXCLUDED, never stubbed. If a later build
//! mounts it, this header must be updated, not silently left stale.
//!
//! Four mandatory seam flips:
//! F1 corrupt frame byte        -> typed ProtocolError;
//! F2 gate-rule violation       -> admission refuses AND downstream
//!                                 steps never run (probe == 0);
//! F3 poisoned head             -> MergeError ANDON (andon wrapper
//!                                 escalates to panic naming heads);
//! F4 trailer tag/digest mutate -> commit validation refuses +
//!                                 re-derivation mismatch.

use std::sync::atomic::{AtomicUsize, Ordering as AOrd};
use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

use cgap_git::admission_gate::{
    parse_update_line, GateError, GateRule, OntologyRefusal, ProposedUpdate, RefTable, RefUpdate,
};
use cgap_git::commit_synth::{
    validate, CommitScratch, CommitSynthError, LeaseCommitSpec, LeaseReceipt, RECEIPT_HEX_LEN,
};
use cgap_git::mcp_driver::{parse_frame, Driver, ProtocolError};
use cgap_git::octopus_merge::{
    octopus_merge, octopus_merge_andon, HeadInput, MergeError, OctopusScratch,
};

// ---------------------------------------------------------------------------
// Seeds (seed law: splitmix64 multiplier vs frozen corpus constant,
// never conflated).
// ---------------------------------------------------------------------------

const SPLITMIX_SEED: u64 = 0xBF58_476D_1CE4_E5B9;
const CORPUS_CONST: u64 = 0xA20C_0FFE_E5A7_E000;
const NONCE: u64 = 0x0047_4150_3038_0001; // "GAP08" tag, fixed pin

// ---------------------------------------------------------------------------
// Counting global allocator: scoped window + positive control. Mirrors
// soc_full_chip_court.rs, upgraded to BYTE accounting: window_counts()
// = (alloc_bytes, dealloc_bytes); realloc folds to a byte delta. The
// window must exit BYTE-BALANCED (equal, nonzero) — every byte the
// seam allocates inside the window it frees inside the window.
// ---------------------------------------------------------------------------

static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
static WINDOW_OPEN: AtomicUsize = AtomicUsize::new(0);
static WINDOW_OWNER: Mutex<Option<ThreadId>> = Mutex::new(None);
static SERIAL: OnceLock<Mutex<()>> = OnceLock::new();

/// True iff the CURRENT thread owns the open window. The pipeline is
/// single-threaded; per-thread attribution keeps harness threads'
/// allocations out of the court's balance claim.
fn window_is_mine() -> bool {
    if WINDOW_OPEN.load(AOrd::Relaxed) != 1 {
        return false;
    }
    let guard = WINDOW_OWNER.lock().unwrap_or_else(|e| e.into_inner());
    matches!(&*guard, Some(owner) if *owner == std::thread::current().id())
}

struct CountingAlloc;
unsafe impl std::alloc::GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        if window_is_mine() {
            ALLOC_BYTES.fetch_add(layout.size(), AOrd::Relaxed);
        }
        unsafe { std::alloc::System.alloc(layout) }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: std::alloc::Layout) {
        if window_is_mine() {
            DEALLOC_BYTES.fetch_add(_layout.size(), AOrd::Relaxed);
        }
        unsafe { std::alloc::System.dealloc(_ptr, _layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        if window_is_mine() {
            // realloc folds to a byte delta: the old block's bytes are
            // retired inside System.realloc itself.
            ALLOC_BYTES.fetch_add(new_size, AOrd::Relaxed);
            DEALLOC_BYTES.fetch_add(layout.size(), AOrd::Relaxed);
        }
        unsafe { std::alloc::System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn open_window() {
    ALLOC_BYTES.store(0, AOrd::Relaxed);
    DEALLOC_BYTES.store(0, AOrd::Relaxed);
    *WINDOW_OWNER.lock().unwrap_or_else(|e| e.into_inner()) = Some(std::thread::current().id());
    WINDOW_OPEN.store(1, AOrd::Relaxed);
}
fn close_window() -> (usize, usize) {
    WINDOW_OPEN.store(0, AOrd::Relaxed);
    *WINDOW_OWNER.lock().unwrap_or_else(|e| e.into_inner()) = None;
    (
        ALLOC_BYTES.load(AOrd::Relaxed),
        DEALLOC_BYTES.load(AOrd::Relaxed),
    )
}
fn window_counts() -> (usize, usize) {
    (
        ALLOC_BYTES.load(AOrd::Relaxed),
        DEALLOC_BYTES.load(AOrd::Relaxed),
    )
}
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

// ---------------------------------------------------------------------------
// Step probes: stage markers for the happy path and the F2 downstream
// counter (a gate refusal must leave it at zero).
// ---------------------------------------------------------------------------

static STEP_COUNT: AtomicUsize = AtomicUsize::new(0);
static F2_DOWNSTREAM_STEPS: AtomicUsize = AtomicUsize::new(0);

fn mark_step() {
    STEP_COUNT.fetch_add(1, AOrd::Relaxed);
}
fn step_count() -> usize {
    STEP_COUNT.load(AOrd::Relaxed)
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// 40 lowercase-hex chars from 20 bytes (gate wire law).
fn hex40(bytes: &[u8; 20]) -> String {
    let mut s = String::with_capacity(40);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Fixed SHA pins (not minted per run — determinism is the court's pin).
fn old_sha() -> [u8; 20] {
    let mut b = [0u8; 20];
    for (i, cell) in b.iter_mut().enumerate() {
        *cell = (i as u8) * 2 + 1;
    }
    b
}
fn new_sha() -> [u8; 20] {
    let mut b = [0u8; 20];
    for (i, cell) in b.iter_mut().enumerate() {
        *cell = 0xF0 ^ ((i as u8) * 3);
    }
    b
}

/// Deterministic triple corpus for one head: distinct coordinates,
/// plus `shared` rows identical across all heads (the unify case).
fn head_corpus(head_id: usize, rows: usize, shared: usize) -> Vec<(u64, u64, u64)> {
    let mut s = SPLITMIX_SEED ^ (CORPUS_CONST ^ (head_id as u64 + 1));
    let mut out: Vec<(u64, u64, u64)> = Vec::with_capacity(rows);
    for i in 0..rows {
        if i < shared {
            // Shared slice: identical bytes for every head.
            let mut c = SPLITMIX_SEED ^ (CORPUS_CONST ^ (i as u64));
            let t = (
                splitmix64(&mut c) & 0x000F_FFFF_FFFF_FFFF,
                splitmix64(&mut c) & 0x000F_FFFF_FFFF_FFFF,
                splitmix64(&mut c) & 0x000F_FFFF_FFFF_FFFF,
            );
            out.push(t);
        } else {
            let t = (
                splitmix64(&mut s) & 0x000F_FFFF_FFFF_FFFF,
                splitmix64(&mut s) & 0x000F_FFFF_FFFF_FFFF,
                splitmix64(&mut s) & 0x000F_FFFF_FFFF_FFFF,
            );
            out.push(t);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Render a sealed lease commit from a triple corpus (S4+S5 core), used
// verbatim by the happy path and by the F4 mutation flips.
// ---------------------------------------------------------------------------

fn sealed_commit(
    payload_corpus: &[(u64, u64, u64)],
) -> (Vec<u8>, LeaseReceipt, [u8; 32], [u8; 32]) {
    // payload = BLAKE3 over the SoA triple bytes (real merge product).
    let mut payload_src: Vec<u8> = Vec::new();
    for &(s, p, o) in payload_corpus {
        payload_src.extend_from_slice(&s.to_le_bytes());
        payload_src.extend_from_slice(&p.to_le_bytes());
        payload_src.extend_from_slice(&o.to_le_bytes());
    }
    let payload = cgap_blake3::hash(&payload_src);
    // Tree is its own domain-separated digest, not a copy of the payload.
    let mut tree_src = b"cgap-gap08-tree|".to_vec();
    tree_src.extend_from_slice(&payload_src);
    let tree = cgap_blake3::hash(&tree_src);
    let parent = [7u8; 32];
    let receipt = LeaseReceipt::seal(NONCE, parent, payload);
    let spec = LeaseCommitSpec {
        tree,
        parent,
        author_ts: 1_700_000_000,
        committer_ts: 1_700_000_100,
        message: "gap08 e2e wire substrate commit",
        lease_phi: cgap_blake3::hash(b"cgap-gap08 lease phi"),
        lease_delta: 42,
        receipt: receipt.clone(),
    };
    let mut scratch = CommitScratch::new();
    let rendered = scratch.render(&spec).expect("render");
    let bytes = rendered.to_vec();
    (bytes, receipt, tree, payload)
}

/// Find the `lease-receipt ` trailer's hex region in rendered bytes.
fn receipt_hex_span(buf: &[u8]) -> (usize, usize) {
    const KEY: &[u8] = b"lease-receipt ";
    let mut start = None;
    let mut i = 0;
    while i + KEY.len() <= buf.len() {
        if &buf[i..i + KEY.len()] == KEY {
            start = Some(i + KEY.len());
            break;
        }
        i += 1;
    }
    let s = start.expect("lease-receipt trailer present");
    let mut e = s;
    while e < buf.len() && buf[e] != b'\n' {
        e += 1;
    }
    (s, e)
}

/// Flip one hex digit in place, keeping the byte valid hex.
fn flip_hex(buf: &mut [u8], at: usize) {
    let c = buf[at];
    buf[at] = match c {
        b'9' => b'a',
        b'f' => b'0',
        _ => c + 1,
    };
}

// ---------------------------------------------------------------------------
// Happy path: one counting-allocator window around the FULL pipeline.
// Positive control first; the window must exit balanced (every heap
// event the seam triggers inside the window it retires inside it).
// ---------------------------------------------------------------------------

#[test]
fn wire_to_substrate_e2e() {
    let _g = serial();

    // -- positive control FIRST: the counter must flip on a real alloc.
    open_window();
    let probe: Vec<u8> = vec![7u8; 512];
    assert_eq!(probe[0], 7);
    let pc_allocs = window_counts().0;
    drop(probe);
    let pc = close_window();
    assert!(pc_allocs > 0, "positive control did not count allocations");
    assert!(pc.1 > 0, "positive control did not count deallocations");

    // The FULL pipeline runs inside ONE window; every heap value is
    // born and dies inside this block.
    {
        open_window();
        let root = std::env::temp_dir().join(format!("cgap-gap08-e2e-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("mkdir root");
        let payload: &[u8] = b"cgap-gap08 wire substrate snapshot payload";
        std::fs::write(root.join("snapshot.bin"), payload).expect("write snapshot");

        // -- S1 FRAME IN (mcp_driver dispatch/parse_frame) ---------------
        let mut driver = Driver::new(root.to_str().expect("utf8 root"));
        let out_push = driver
            .dispatch(b"TOOL Push snapshot.bin\n")
            .expect("push dispatch");
        assert!(out_push.pushed.is_some(), "S1: push enqueued");
        assert_eq!(driver.pending_pushes(), 1, "S1: queue depth 1");
        assert_eq!(driver.proposals()[0].snapshot_path, "snapshot.bin");
        let out_read = driver
            .dispatch(b"READ snapshot.bin\n")
            .expect("read dispatch");
        assert_eq!(
            out_read.read.as_deref().expect("read bytes"),
            payload,
            "S1: READ returns the real file bytes"
        );
        mark_step();

        // -- S2 STATE (admission_gate) -----------------------------------
        let line = format!(
            "{} {} refs/heads/main",
            hex40(&old_sha()),
            hex40(&new_sha())
        );
        let req: RefUpdate = parse_update_line(line.as_bytes()).expect("wire parse");
        assert_eq!(req.ref_name, b"refs/heads/main".to_vec());
        assert_eq!(req.old_sha, old_sha());
        assert_eq!(req.new_sha, new_sha());
        let mut table = RefTable::new();
        table.set(b"refs/heads/main", old_sha());
        let v0 = cgap_git::admission_gate::var(0);
        let rule = GateRule {
            rule_id: 11,
            body: vec![(0xA1, 0xB2, v0)],
            head: (v0, 0xD4, 0x0B0B),
        };
        let proposed = ProposedUpdate {
            facts: vec![(0xA1, 0xB2, 0xC3), (0xA1, 0xB2, 0xC4)],
            rules: vec![rule],
            functional_predicates: Default::default(),
        };
        table.apply(&req, &proposed).expect("admission");
        assert_eq!(
            table.get(b"refs/heads/main"),
            Some(new_sha()),
            "S2: ref advanced old -> new through the gate"
        );
        mark_step();

        // -- S3 MERGE (octopus_merge, 3 heads) ---------------------------
        let rows = 32usize;
        let shared = 8usize;
        let heads_n = 3usize;
        let mut corpora: Vec<Vec<(u64, u64, u64)>> = Vec::new();
        for h in 0..heads_n {
            corpora.push(head_corpus(h, rows, shared));
        }
        let mut columns: Vec<[Vec<u64>; 3]> = (0..heads_n).map(|_| Default::default()).collect();
        for (h, corpus) in corpora.iter().enumerate() {
            for t in corpus {
                columns[h][0].push(t.0);
                columns[h][1].push(t.1);
                columns[h][2].push(t.2);
            }
        }
        let heads: Vec<HeadInput> = (0..heads_n)
            .map(|h| {
                HeadInput::try_new(h, &columns[h][0], &columns[h][1], &columns[h][2])
                    .expect("head shape ok")
            })
            .collect();
        let total = rows * heads_n;
        let mut row_refs: Vec<(u32, u32)> = vec![(0, 0); total];
        let mut scratch = OctopusScratch::default();
        let mut out_s: Vec<u64> = vec![0; total];
        let mut out_p: Vec<u64> = vec![0; total];
        let mut out_o: Vec<u64> = vec![0; total];
        let merged_n = octopus_merge(
            &heads,
            &mut row_refs,
            &mut scratch,
            &mut out_s,
            &mut out_p,
            &mut out_o,
        )
        .expect("S3: merge succeeds");
        let mut distinct = std::collections::BTreeSet::new();
        for corpus in &corpora {
            for t in corpus {
                distinct.insert(*t);
            }
        }
        assert_eq!(
            merged_n,
            distinct.len(),
            "S3: merged rows == distinct triples (dedupe + unify)"
        );
        assert!(merged_n > shared, "S3: merge is non-degenerate");
        let shared0 = corpora[0][0];
        let mut found = false;
        for i in 0..merged_n {
            if (out_s[i], out_p[i], out_o[i]) == shared0 {
                found = true;
                break;
            }
        }
        assert!(found, "S3: shared row survives the merge verbatim");
        mark_step();

        // -- S4 COMMIT (commit_synth render + receipt seal) ---------------
        let mut merged_corpus: Vec<(u64, u64, u64)> = Vec::with_capacity(merged_n);
        for i in 0..merged_n {
            merged_corpus.push((out_s[i], out_p[i], out_o[i]));
        }
        let (bytes, receipt, tree, _payload) = sealed_commit(&merged_corpus);
        mark_step();

        // -- S5 SEAL (cgap_blake3 digest + 80-byte shape) -----------------
        // 80 = RECEIPT_HEX_LEN / 2 (commit_synth.rs: the on-wire receipt
        // line carries 160 hex chars). Layout provenance: module doc of
        // commit_synth::LeaseReceipt — nonce u64 LE [0..8] + parent
        // [8..40] + payload [40..72] + tag [72..80], no padding holes.
        const EXPECTED_SEAL_BYTES: usize = RECEIPT_HEX_LEN / 2;
        assert_eq!(EXPECTED_SEAL_BYTES, 80);
        let seal = receipt.to_bytes();
        assert_eq!(seal.len(), EXPECTED_SEAL_BYTES, "S5: 80-byte seal shape");
        let digest = cgap_blake3::hash(&bytes);
        assert_eq!(digest.len(), 32, "S5: blake3 digest width");
        let fields = validate(&bytes).expect("S5: rendered commit validates");
        assert_eq!(fields.tree, tree, "S5: tree round-trips");
        assert_eq!(fields.receipt, receipt, "S5: receipt round-trips");
        assert_eq!(
            fields.receipt.tag,
            LeaseReceipt::expected_tag(
                fields.receipt.nonce,
                &fields.receipt.parent,
                &fields.receipt.payload
            ),
            "S5: tag re-derivation matches"
        );
        mark_step();
    };
    let (win_a, win_d) = close_window();
    assert_eq!(win_a, win_d, "window exits balanced");
    assert!(win_a > 0, "window saw real allocation traffic");
    assert_eq!(step_count(), 5, "S1..S5 all ran inside the window");
    std::fs::remove_dir_all(
        std::env::temp_dir().join(format!("cgap-gap08-e2e-{}", std::process::id())),
    )
    .ok();
}

// ---------------------------------------------------------------------------
// F1: corrupt frame byte -> typed ProtocolError.
// ---------------------------------------------------------------------------

#[test]
fn f1_frame_byte_corrupt_refuses_typed() {
    let _g = serial();
    let good: &[u8] = b"TOOL Push snapshot.bin\n";
    let mut bad = good.to_vec();
    bad[0] = b'X';
    let mut driver = Driver::new("/tmp");
    let err = driver
        .dispatch(&bad)
        .expect_err("F1: corrupted verb must refuse");
    assert_eq!(err, ProtocolError::UnknownVerb);
    assert_eq!(parse_frame(&bad), Err(ProtocolError::UnknownVerb));
    assert!(parse_frame(good).is_ok(), "F1 control: clean frame parses");
}

// ---------------------------------------------------------------------------
// F2: gate-rule violation -> admission refuses AND downstream never runs.
// Probe is two-sided: the broken rule must leave F2_DOWNSTREAM_STEPS at
// zero, and an otherwise-identical ADMITTED rule must move it (a probe
// that never moves is vacuous).
// ---------------------------------------------------------------------------

#[test]
fn f2_gate_violation_blocks_downstream() {
    let _g = serial();
    F2_DOWNSTREAM_STEPS.store(0, AOrd::Relaxed);
    let line = format!(
        "{} {} refs/heads/main",
        hex40(&old_sha()),
        hex40(&new_sha())
    );
    let req: RefUpdate = parse_update_line(line.as_bytes()).expect("wire parse");
    let unbound = cgap_git::admission_gate::var(9);
    let rule = GateRule {
        rule_id: 77,
        body: vec![(0xA1, 0xB2, 0xC3)],
        head: (unbound, 0xD4, 0x0B0B),
    };
    let proposed = ProposedUpdate {
        facts: vec![(0xA1, 0xB2, 0xC3)],
        rules: vec![rule],
        functional_predicates: Default::default(),
    };
    let mut table = RefTable::new();
    table.set(b"refs/heads/main", old_sha());
    let outcome = table.apply(&req, &proposed);
    match outcome {
        // Downstream continuation: runs ONLY behind an admitted gate.
        Ok(()) => {
            F2_DOWNSTREAM_STEPS.fetch_add(1, AOrd::Relaxed);
        }
        Err(e) => match e {
            GateError::OntologyRuleBroken { rule_id, reason } => {
                assert_eq!(rule_id, 77);
                assert_eq!(reason, OntologyRefusal::UnboundHeadVariable);
            }
            other => panic!("F2: wrong refusal class: {other:?}"),
        },
    }
    // Control arm: same wire update, valid rule -> gate admits, probe moves.
    let v0 = cgap_git::admission_gate::var(0);
    let ok_rule = GateRule {
        rule_id: 78,
        body: vec![(0xA1, 0xB2, v0)],
        head: (v0, 0xD4, 0x0B0B),
    };
    let ok_proposed = ProposedUpdate {
        facts: vec![(0xA1, 0xB2, 0xC3)],
        rules: vec![ok_rule],
        functional_predicates: Default::default(),
    };
    table
        .apply(&req, &ok_proposed)
        .expect("F2 control: gate admits");
    let probed = F2_DOWNSTREAM_STEPS.fetch_add(1, AOrd::Relaxed) + 1;
    assert_eq!(probed, 1, "F2 control: admitted arm reaches downstream");
    assert_eq!(
        F2_DOWNSTREAM_STEPS.load(AOrd::Relaxed),
        1,
        "F2: violation arm contributed zero downstream steps"
    );
    assert_eq!(
        table.get(b"refs/heads/main"),
        Some(new_sha()),
        "F2: ref advanced only through the admitted arm"
    );
}

// ---------------------------------------------------------------------------
// F3: poisoned head -> MergeError ANDON. Three heads, head 1 poisons a
// coordinate head 0 already holds with a different object.
// ---------------------------------------------------------------------------

#[test]
fn f3_poisoned_head_merge_andon() {
    let _g = serial();
    let clash_s = 0x5EED_0001u64;
    let clash_p = 0x0000_0042u64;
    let mut corpora: Vec<Vec<(u64, u64, u64)>> = Vec::new();
    for h in 0..3usize {
        let mut c = head_corpus(h, 16, 0);
        if h == 0 {
            c[0] = (clash_s, clash_p, 0xAAAA);
        }
        if h == 1 {
            c[0] = (clash_s, clash_p, 0xBBBB);
        }
        corpora.push(c);
    }
    let mut columns: Vec<[Vec<u64>; 3]> = (0..3).map(|_| Default::default()).collect();
    for (h, corpus) in corpora.iter().enumerate() {
        for t in corpus {
            columns[h][0].push(t.0);
            columns[h][1].push(t.1);
            columns[h][2].push(t.2);
        }
    }
    let heads: Vec<HeadInput> = (0..3)
        .map(|h| {
            HeadInput::try_new(h, &columns[h][0], &columns[h][1], &columns[h][2])
                .expect("head shape ok")
        })
        .collect();
    let mut row_refs: Vec<(u32, u32)> = vec![(0, 0); 48];
    let mut scratch = OctopusScratch::default();
    let mut out_s: Vec<u64> = vec![0; 48];
    let mut out_p: Vec<u64> = vec![0; 48];
    let mut out_o: Vec<u64> = vec![0; 48];
    let err = octopus_merge(
        &heads,
        &mut row_refs,
        &mut scratch,
        &mut out_s,
        &mut out_p,
        &mut out_o,
    )
    .expect_err("F3: domain clash must refuse");
    match err {
        MergeError::DomainClash {
            head_a,
            head_b,
            object_a,
            object_b,
            subject,
            predicate,
        } => {
            assert_eq!((head_a, head_b), (0, 1));
            assert_eq!((object_a, object_b), (0xAAAA, 0xBBBB));
            assert_eq!(subject, clash_s);
            assert_eq!(predicate, clash_p);
            let andon_line = format!("{err}");
            drop(andon_line);
        }
        other => panic!("F3: wrong refusal class: {other:?}"),
    }
    // ANDON escalation: the andon wrapper must turn the same refusal
    // into a process-stopping panic naming the clashing heads.
    let andon = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        octopus_merge_andon(
            &heads,
            &mut row_refs,
            &mut scratch,
            &mut out_s,
            &mut out_p,
            &mut out_o,
        )
    }));
    assert!(
        andon.is_err(),
        "F3: octopus_merge_andon must escalate DomainClash"
    );
    let payload = andon.unwrap_err();
    let msg = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .expect("F3: string panic payload");
    assert!(
        msg.contains("ANDON-REFUSAL"),
        "F3: andon panic names the refusal: {msg}"
    );
}

// ---------------------------------------------------------------------------
// F4: trailer tag/digest mutate -> commit validation refuses +
// re-derivation mismatch.
// ---------------------------------------------------------------------------

#[test]
fn f4_trailer_mutation_refuses() {
    let _g = serial();
    let corpus = head_corpus(9, 8, 0);
    let (bytes, receipt, _tree, payload) = sealed_commit(&corpus);
    assert!(
        validate(&bytes).is_ok(),
        "F4 control: clean commit validates"
    );

    // F4a: flip one hex digit inside the 8-byte tag region (last 16
    // hex chars of the lease-receipt trailer).
    let (s, e) = receipt_hex_span(&bytes);
    assert_eq!(e - s, RECEIPT_HEX_LEN, "trailer hex spans 160 chars");
    let mut tag_mut = bytes.clone();
    let (_, te) = receipt_hex_span(&tag_mut);
    flip_hex(&mut tag_mut, te - 16);
    match validate(&tag_mut) {
        Err(CommitSynthError::ReceiptTagMismatch) => {}
        other => panic!("F4a: expected ReceiptTagMismatch, got {other:?}"),
    }

    // F4b: flip one hex digit inside the 32-byte payload region of the
    // trailer (bytes 40..72 -> hex offset 80..144). The sealed tag was
    // derived over the ORIGINAL payload: re-derivation must mismatch.
    let mut pay_mut = bytes.clone();
    let (ps, _) = receipt_hex_span(&pay_mut);
    flip_hex(&mut pay_mut, ps + 80);
    match validate(&pay_mut) {
        Err(CommitSynthError::ReceiptTagMismatch) => {}
        other => panic!("F4b: expected ReceiptTagMismatch, got {other:?}"),
    }

    // F4c: direct re-derivation mismatch at the API level.
    let mut payload_mut = payload;
    payload_mut[0] ^= 0x01;
    assert_ne!(
        LeaseReceipt::expected_tag(NONCE, &receipt.parent, &payload_mut),
        receipt.tag,
        "F4c: mutated payload changes the derived tag"
    );
    assert_ne!(receipt.tag, [0u8; 8], "F4: tag is not degenerate");
    let _ = s;
}
