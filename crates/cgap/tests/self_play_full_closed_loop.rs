//! SP9 self-play court — FULL CLOSED LOOP, 50 iterations, alternating
//! topologies (chain vs fan-out), each iteration the entire sovereign
//! pipeline over real collaborators:
//!
//!   frame admission (penstock ingress, A04 128B)
//!     -> lease tick (monotonic advance per admitted frame)
//!     -> arrow statement arrays (delta s/p/o + PresenceMask)
//!     -> bn3 radix join over the WHOLE store (JoinProbeScratch, radix
//!        default-true, caller-leased scratch)
//!     -> cascade retract on a derived chain (cgap-storage dred, leased
//!        CSR + worklist; topology alternates chain/fan-out per iter)
//!     -> parquet seal (FooterSealer, i32 lane-domain witness, FA9
//!        0-byte pre-rotation latch witnessed below)
//!     -> git loose-object commit binding tick + receipt (blob -> tree ->
//!        commit; commit trailer carries the running digest)
//!
//! State conservation (honest form): the receipt chain binds every
//! admitted delta — each iteration's commit metadata carries the running
//! digest, `R_i = BLAKE3(R_{i-1} || commit_bytes_i)`, and the chain is
//! tamper-evident: flipping one iteration's bytes forces a mismatch.
//!
//! Allocation assertion (FA4, honest scope): a counting GlobalAlloc
//! witnesses that bytes_allocated_per_iter does NOT scale with corpus
//! size from iteration 2 — leased scratch, CSR, seal buffers and store
//! capacity are all provisioned OUTSIDE the measured region, so the
//! per-iteration budget is a constant bounded strictly below the cost of
//! ONE store-sized copy (that inequality is the anti-vacuity witness:
//! any store-sized copy inside the loop trips the budget).
//!
//! ANTI-VACUITY: corrupt one iteration's payload -> chain mismatch
//! (`sp_chain_tamper_flip`); the unmutated recompute must agree (the
//! flip, not the pass, is the proof).
//!
//! SEED LAW: splitmix64 multiplier `0xBF58_476D_1CE4_E5B9` drives the
//! corpus stream; the corpus constant `0xA20C_0FFE_E5A7_E000` is the
//! parquet bitslice spread constant and is deliberately NOT used here
//! (the two seeds never conflate).
//!
//! ---------------------------------------------------------------------------
//! RECEIPT (SP9 — filled at run time; see the comment block at the bottom)
//! ---------------------------------------------------------------------------

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use cgap::cgap_arrow::bitmask::{mask_bytes, PresenceMask};
use cgap::cgap_blake3;
use cgap::cgap_git::pack_write::{git_object_oid, sha1};
use cgap::cgap_parquet::footer::{verify_seal, BlockSummary, FooterSealer};
use cgap_bn3::noalloc::{na_compile, na_unify_radix, NaEngine, NaMatches};
use cgap_bn3::sigil::{encode_term, Sigil};
use cgap_bn3::unify::{BodyAtom, JoinProbeScratch, Rule, StatementColumns};
use cgap_gate::penstock::{
    ingress, CGAP_OK, HEADER_LEN, HEADER_VERSION, MAGIC, OABI_DIGEST_LEN, OFF_ATTESTATION,
    OFF_FLAGS, OFF_MAGIC, OFF_OABI_DIGEST, OFF_PAYLOAD_BYTE_LEN, OFF_VERSION, P28_FLAG_ROOT,
    P28_LICENSE_MAGIC, P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION,
};
use cgap_storage::dred;

// --- Counting allocator (FA4 allocation witness) --------------------------

static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);
// Thread-local measuring gate: only allocations made on a thread that
// armed the gate are counted. The libtest harness runs courts in
// parallel against ONE process-global allocator; without the gate a
// sibling court's allocations land inside this court's measured window
// and the budget assert flakes (witnessed this session).
thread_local! {
    static MEASURE: Cell<bool> = const { Cell::new(false) };
}

fn alloc_arm() {
    MEASURE.with(|m| m.set(true));
}
fn alloc_disarm() {
    MEASURE.with(|m| m.set(false));
}

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if MEASURE.with(Cell::get) {
            ALLOC_BYTES.fetch_add(l.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        if MEASURE.with(Cell::get) {
            let d = n as isize - l.size() as isize;
            if d > 0 {
                ALLOC_BYTES.fetch_add(d as usize, Ordering::Relaxed);
            }
        }
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn alloc_reset() {
    alloc_arm();
    ALLOC_BYTES.store(0, Ordering::Relaxed);
}
fn alloc_taken() -> usize {
    alloc_disarm();
    ALLOC_BYTES.load(Ordering::Relaxed)
}

// --- Loop constants --------------------------------------------------------

const ITERS: u64 = 50;
const TICK_CEILING: u64 = 128; // > ITERS; the lease never exhausts lawfully
const PRELOAD: usize = 4_096; // store rows before the loop
const DELTA: usize = 64; // admitted statements per iteration
const STORE_MAX: usize = PRELOAD + ITERS as usize * DELTA; // 7,296 < NA_MAX_ROWS(8192)
const FACTS: usize = 8; // cascade window per iteration
const S_POOL: u64 = 8;
const P_POOL: u64 = 4;
const O_POOL: u64 = 4096;
const BASE: u64 = 0x000A_0000_0000_0000;
/// FA4 budget: constant, N-independent, strictly below one store-sized
/// copy (STORE_MAX * 24 bytes ~ 175 KiB) — the inequality below is the
/// anti-vacuity witness for the allocation court.
const PER_ITER_BUDGET: usize = 32 * 1024;

// --- Corpus RNG (canonical splitmix64 stream) ------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_490B_BADD_03);
        z ^ (z >> 31)
    }
    fn pooled(&mut self, pool: u64) -> u64 {
        BASE + (self.next() % pool)
    }
}

fn cst(v: u64) -> u64 {
    encode_term(Sigil::SlotConst, v).expect("in-range payload")
}
fn vr(r: u64) -> u64 {
    encode_term(Sigil::VarUni, r).expect("in-range register")
}

/// Two-atom join rule: (?x P ?y) . (?y Q ?z) — the lever_courts shape.
fn join_rule() -> Rule {
    Rule {
        rule_id: 9,
        body: vec![
            BodyAtom::new(vr(0), cst(BASE + 1), vr(1)),
            BodyAtom::new(vr(1), cst(BASE + 2), vr(2)),
        ],
        head: [vr(0), cst(BASE + 3), vr(2)],
    }
}

// --- Frame builder (SP2 geometry, cited not redefined) ---------------------

fn admitted_frame(payload_len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; HEADER_LEN + payload_len];
    buf[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
    buf[OFF_VERSION] = HEADER_VERSION;
    buf[OFF_FLAGS] = 0;
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
    buf
}

// --- Lease ledger (monotonic tick; exhaustion typed at the batch boundary --

struct LeaseLedger {
    tick: u64,
}

impl LeaseLedger {
    fn advance(&mut self) -> u64 {
        self.tick += 1;
        assert!(self.tick <= TICK_CEILING, "lease exhausted unlawfully");
        self.tick
    }
}

// --- Parquet seal of one iteration's delta ---------------------------------

fn seal_delta(
    seal_buf: &mut Vec<u8>,
    body: &[u8],
    tick: u64,
    running_digest: &[u8; 32],
) -> (Vec<u8>, [u8; 32]) {
    // i32 lane-domain witness: the delta span must sit inside the §6.2
    // field-4 zigzag i32 wire domain BEFORE sealing (fail-closed witness).
    let span = (body.len() / 2) as u32;
    assert!(span <= i32::MAX as u32, "span exceeds the i32 lane domain");
    seal_buf.clear();
    seal_buf.extend_from_slice(body);
    // Reserve footer headroom INSIDE the leased buffer so the sealer sees
    // a slice strictly larger than the body (the footer appends in place).
    let body_len = body.len();
    seal_buf.resize(body_len + 4096, 0);
    let summary = BlockSummary {
        row_groups: 2,
        rows: DELTA as u64,
        spans: &[span, span],
    };
    let att = p28_attestation(tick, running_digest);
    let sealer = FooterSealer::new(seal_buf, body_len, summary, Some(&att))
        .expect("lawful seal: non-empty body, spans sum to body_len");
    let sealed = sealer.seal().expect("seal completes");
    let digest = *sealed.digest();
    verify_seal(sealed.bytes(), &digest).expect("sealed block verifies");
    (sealed.bytes().to_vec(), digest)
}

/// P28-shaped 80-byte attestation slot binding tick + running receipt.
fn p28_attestation(tick: u64, running: &[u8; 32]) -> [u8; 80] {
    let mut a = [0u8; 80];
    a[0..4].copy_from_slice(b"P28S");
    a[4..6].copy_from_slice(&1u16.to_le_bytes());
    a[6..8].copy_from_slice(&1u16.to_le_bytes()); // flags bit0
    a[8..16].copy_from_slice(&5_000_000u64.to_le_bytes());
    let mut bind = Vec::with_capacity(40);
    bind.extend_from_slice(&tick.to_le_bytes());
    bind.extend_from_slice(running);
    a[16..48].copy_from_slice(&cgap_blake3::hash(&bind));
    a
}

// --- Git loose-object commit (blob -> tree -> commit) ----------------------

fn envelope(kind: &str, content: &[u8]) -> Vec<u8> {
    let mut env = Vec::with_capacity(kind.len() + 20 + content.len());
    env.extend_from_slice(kind.as_bytes());
    env.push(b' ');
    env.extend_from_slice(content.len().to_string().as_bytes());
    env.push(0);
    env.extend_from_slice(content);
    env
}

/// Independent recompute of a git object OID (bypasses git_object_oid).
fn independent_oid(kind: &str, content: &[u8]) -> [u8; 20] {
    sha1(&envelope(kind, content))
}

fn hex20(oid: &[u8; 20]) -> String {
    oid.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex32(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Build the iteration's commit object: tree of one blob entry, metadata
/// carrying the tick and the running receipt digest. Returns
/// (commit_content, commit_oid).
fn commit_object(
    blob_oid: [u8; 20],
    iter: u64,
    tick: u64,
    running_hex: &str,
) -> (Vec<u8>, [u8; 20]) {
    let name = format!("delta-{iter:04}.parquet");
    let mut tree = Vec::new();
    tree.extend_from_slice(b"100644 ");
    tree.extend_from_slice(name.as_bytes());
    tree.push(0);
    tree.extend_from_slice(&blob_oid);
    let tree_oid = git_object_oid("tree", &tree);
    assert_eq!(
        tree_oid,
        independent_oid("tree", &tree),
        "tree OID: crate vs independent envelope recompute"
    );
    let mut commit = Vec::new();
    commit.extend_from_slice(&format!("tree {}\n", hex20(&tree_oid)).into_bytes());
    commit.extend_from_slice(
        &format!("author cgap-sp9 <sp9@cgap> 0 +0000\ncommitter cgap-sp9 <sp9@cgap> 0 +0000\n\n")
            .into_bytes(),
    );
    commit.extend_from_slice(&format!("cgap-iter {iter}\n").into_bytes());
    commit.extend_from_slice(&format!("cgap-p28-tick {tick}\n").into_bytes());
    commit.extend_from_slice(&format!("cgap-receipt {running_hex}\n").into_bytes());
    let oid = git_object_oid("commit", &commit);
    assert_eq!(
        oid,
        independent_oid("commit", &commit),
        "commit OID: crate vs independent envelope recompute"
    );
    (commit, oid)
}

// --- Leased-scratch provisioning helpers -----------------------------------

/// Heap-direct zeroed Box for the multi-MiB fixed noalloc buffers (the
/// crate's own pattern, cited from lever_courts.rs).
fn boxed_zeroed<T>() -> Box<T> {
    let v = vec![0u8; core::mem::size_of::<T>()];
    unsafe { Box::from_raw(Box::into_raw(v.into_boxed_slice()) as *mut T) }
}

/// Alternating cascade topologies: even iterations a linear derived
/// chain (depth FACTS), odd iterations a fan-out (root -> FACTS-1).
fn topology_edges(iter: u64) -> Vec<(u32, u32)> {
    if iter % 2 == 0 {
        (0..FACTS as u32 - 1).map(|k| (k, k + 1)).collect()
    } else {
        (1..FACTS as u32).map(|k| (0, k)).collect()
    }
}

struct Artifacts {
    commits: Vec<Vec<u8>>,
    final_digest: [u8; 32],
    final_tick: u64,
    final_rows: usize,
    max_per_iter_alloc: usize,
}

/// The full closed loop. `check_alloc=true` asserts the FA4 budget from
/// iteration 2 (index 1) onward.
fn run_closed_loop(check_alloc: bool) -> Artifacts {
    // The leased noalloc buffers are multi-MiB fixed constructs and the
    // debug-profile frames that touch them overflow the harness test
    // thread's stack (the crate's own witnessed overflow class, cited
    // from noalloc.rs). Run the loop on a dedicated thread that escapes
    // that constrained stack; the measuring gate is thread-local, so the
    // FA4 court is unaffected by the hop.
    std::thread::scope(|sc| {
        sc.spawn(move || run_closed_loop_inner(check_alloc))
            .join()
            .unwrap_or_else(|e| std::panic::resume_unwind(e))
    })
}

fn run_closed_loop_inner(check_alloc: bool) -> Artifacts {
    // ---- Provisioning OUTSIDE the measured region (FA4 honest scope) ----
    let mut rng = Rng(0xBF58_476D_1CE4_E5B9);
    let mut store_s = Vec::with_capacity(STORE_MAX);
    let mut store_p = Vec::with_capacity(STORE_MAX);
    let mut store_o = Vec::with_capacity(STORE_MAX);
    for _ in 0..PRELOAD {
        store_s.push(rng.pooled(S_POOL));
        store_p.push(rng.pooled(P_POOL));
        store_o.push(rng.pooled(O_POOL));
    }
    let rule = join_rule();
    let na_rule = na_compile(&rule).expect("na compile");
    let mut eng = boxed_zeroed::<NaEngine>();
    let mut na_out = boxed_zeroed::<NaMatches>();
    let probe = JoinProbeScratch::default();
    assert!(probe.radix, "L3 fast path must be the DEFAULT (FA3)");
    let mut mask = PresenceMask::<
        { STORE_MAX.next_power_of_two() },
        { mask_bytes(STORE_MAX.next_power_of_two()) },
    >::EMPTY;
    let facts_words = (FACTS + 63) / 64;
    let mut del_mask = vec![0u64; facts_words];
    let mut derived = vec![0u64; facts_words];
    for f in 0..FACTS {
        derived[f / 64] |= 1u64 << (f % 64);
    }
    let alt = vec![0u32; FACTS];
    let mut counts = vec![0u32; FACTS + 1];
    let mut fill = vec![0u32; FACTS];
    let mut adj = vec![0u32; FACTS * (FACTS - 1)];
    let mut purged = vec![0u64; facts_words];
    let mut keep = vec![0u64; facts_words];
    let mut worklist = vec![0u32; FACTS];
    let mut cascade_out = vec![0u32; facts_words * 64 + 1];
    let mut body_buf = Vec::with_capacity(DELTA * 24 + 64);
    let mut seal_buf = Vec::with_capacity(DELTA * 24 + 4096);
    let mut ledger = LeaseLedger { tick: 0 };
    let mut commits: Vec<Vec<u8>> = Vec::with_capacity(ITERS as usize);
    let mut per_iter_allocs = Vec::with_capacity(ITERS as usize);
    let mut running: [u8; 32] = cgap_blake3::hash(b"cgap-sp9-genesis");
    let mut last_joins = 0usize;
    del_mask[0] = 1; // cascade root: fact 0 of the iteration window

    for iter in 0..ITERS {
        if check_alloc {
            alloc_reset();
        }
        // 1. Frame admission (penstock, A04 128 B header).
        let frame = admitted_frame(DELTA + iter as usize);
        let mut view = cgap_gate::penstock::FrameView {
            payload: &[],
            prefilter: 0,
            escalate: false,
        };
        assert_eq!(
            ingress(&frame, frame.len(), Some(&mut view)),
            CGAP_OK,
            "iter {iter}: frame must be admitted"
        );
        // 2. Lease tick advances monotonically per admitted frame.
        let tick = ledger.advance();
        // 3. Arrow statement arrays: delta rows appended, presence set.
        let start = store_s.len();
        for _ in 0..DELTA {
            store_s.push(rng.pooled(S_POOL));
            store_p.push(rng.pooled(P_POOL));
            store_o.push(rng.pooled(O_POOL));
        }
        for r in start..start + DELTA {
            mask.set(r).expect("row within mask capacity");
        }
        assert_eq!(mask.count() >= DELTA, true, "presence count grows");
        // 4. BN3 radix join over the WHOLE store (leased scratch).
        let cols = StatementColumns::new(&store_s, &store_p, &store_o).expect("columns");
        na_unify_radix(&na_rule, &cols, &mut eng, &mut na_out).expect("na radix join ok");
        assert!(na_out.len > 0, "iter {iter}: store must yield joins");
        assert!(
            na_out.len >= last_joins,
            "iter {iter}: join yield must not shrink as the store grows",
        );
        last_joins = na_out.len;
        // 5. Cascade retract on the alternating derived topology.
        let edges = topology_edges(iter);
        dred::build_csr(&edges, FACTS, &mut counts, &mut fill, &mut adj)
            .expect("CSR build from the iteration topology");
        let total = dred::cascade_retract(
            &derived,
            &del_mask,
            &alt,
            FACTS,
            &counts,
            &adj,
            &mut purged,
            &mut keep,
            &mut worklist,
            0,
            &mut cascade_out,
        )
        .expect("cascade on a fully-provisioned lease");
        let purged_n: usize = purged.iter().map(|w| w.count_ones() as usize).sum();
        assert_eq!(
            purged_n, FACTS,
            "iter {iter}: cascade purges the full window"
        );
        assert_eq!(total, 0, "iter {iter}: zero phantom survivors");
        // 6. Parquet seal (i32 lane-domain witness inside seal_delta).
        body_buf.clear();
        for r in start..start + DELTA {
            body_buf.extend_from_slice(&store_s[r].to_le_bytes());
            body_buf.extend_from_slice(&store_p[r].to_le_bytes());
            body_buf.extend_from_slice(&store_o[r].to_le_bytes());
        }
        let (sealed, _seal_digest) = seal_delta(&mut seal_buf, &body_buf, tick, &running);
        // 7. Git loose-object commit binding tick + receipt.
        let blob_oid = git_object_oid("blob", &sealed);
        assert_eq!(
            blob_oid,
            independent_oid("blob", &sealed),
            "blob OID recompute"
        );
        let running_hex = hex32(&running);
        let (commit, commit_oid) = commit_object(blob_oid, iter, tick, &running_hex);
        assert_eq!(
            commit_oid,
            independent_oid("commit", &commit),
            "commit OID recompute"
        );
        // State conservation: R_{i} = BLAKE3(R_{i-1} || commit_bytes_i).
        let mut chain_input = Vec::with_capacity(32 + commit.len());
        chain_input.extend_from_slice(&running);
        chain_input.extend_from_slice(&commit);
        running = cgap_blake3::hash(&chain_input);
        commits.push(commit);
        if check_alloc {
            per_iter_allocs.push(alloc_taken());
        }
    }

    assert_eq!(ledger.tick, ITERS, "final tick equals admitted frames");
    assert_eq!(store_s.len(), STORE_MAX, "every admitted delta landed");
    assert_eq!(commits.len() as u64, ITERS, "one commit per iteration");
    if check_alloc {
        // Anti-vacuity: the budget must sit STRICTLY BELOW the cost of a
        // single store-sized copy — otherwise the assertion is vacuous.
        assert!(
            PER_ITER_BUDGET < STORE_MAX * 24,
            "budget {} must be below one store copy {}",
            PER_ITER_BUDGET,
            STORE_MAX * 24
        );
        for (i, &b) in per_iter_allocs.iter().enumerate().skip(1) {
            assert!(
                b <= PER_ITER_BUDGET,
                "FA4: iter {} allocated {} B > budget {} B (must not scale with corpus)",
                i + 1,
                b,
                PER_ITER_BUDGET
            );
        }
    }
    Artifacts {
        commits,
        final_digest: running,
        final_tick: ledger.tick,
        final_rows: store_s.len(),
        max_per_iter_alloc: per_iter_allocs.iter().copied().max().unwrap_or(0),
    }
}

/// Independent receipt-chain recompute from raw commit bytes.
fn recompute_chain(commits: &[Vec<u8>]) -> [u8; 32] {
    let mut running = cgap_blake3::hash(b"cgap-sp9-genesis");
    for c in commits {
        let mut input = Vec::with_capacity(32 + c.len());
        input.extend_from_slice(&running);
        input.extend_from_slice(c);
        running = cgap_blake3::hash(&input);
    }
    running
}

// --- Courts ----------------------------------------------------------------

#[test]
fn sp_full_closed_loop_50_iters() {
    let a = run_closed_loop(true);
    assert_eq!(a.final_tick, ITERS);
    assert_eq!(a.final_rows, STORE_MAX);
    // State conservation: independent recompute over the commit bytes
    // equals the loop's running digest (the chain binds every delta).
    assert_eq!(
        recompute_chain(&a.commits),
        a.final_digest,
        "receipt chain must bind every admitted delta"
    );
    // FA4 witness read-out: the peak per-iteration allocation is reported
    // and must sit in (0, budget] — zero would mean the gate never armed.
    assert!(a.max_per_iter_alloc > 0, "measurement must be non-vacuous");
    assert!(
        a.max_per_iter_alloc <= PER_ITER_BUDGET,
        "peak per-iter alloc {} B > budget",
        a.max_per_iter_alloc
    );
}

#[test]
fn sp_chain_tamper_flip() {
    // Anti-vacuity: corrupt ONE iteration's payload bytes -> the
    // recomputed chain diverges from the recorded final digest. The
    // UNMUTATED recompute must agree first (a flip that fires on an
    // already-diverging chain proves nothing).
    let a = run_closed_loop(true);
    assert_eq!(recompute_chain(&a.commits), a.final_digest);
    let mut tampered = a.commits.clone();
    let mid = tampered.len() / 2;
    let byte_at = tampered[mid].len() / 2;
    tampered[mid][byte_at] ^= 0xFF;
    assert_ne!(
        recompute_chain(&tampered),
        a.final_digest,
        "ANTI-VACUITY: flipping one iteration's payload byte must break the chain"
    );
    // Byte-flip back: byte-exact restore restores the chain.
    tampered[mid][byte_at] ^= 0xFF;
    assert_eq!(
        recompute_chain(&tampered),
        a.final_digest,
        "restore restores the chain"
    );
}

#[test]
fn sp_zero_byte_seal_latch_fa9() {
    // FA9 witnessed semantics: the parquet seal latches the 0-byte body
    // PRE-rotation — FooterSealer::new refuses a 0-byte body with a typed
    // RunOverrun (W6-F1), so a zero-body block can never rotate into a
    // sealed artifact that verify_seal would refuse downstream.
    let mut buf = vec![0u8; 512];
    let summary = BlockSummary {
        row_groups: 1,
        rows: 1,
        spans: &[0],
    };
    let err = match FooterSealer::new(&mut buf, 0, summary, None) {
        Ok(_) => panic!("FA9: 0-byte body must be refused at the seal boundary"),
        Err(e) => e,
    };
    let msg = format!("{err:?}");
    assert!(
        msg.contains("RunOverrun"),
        "typed refusal expected, got {msg}"
    );
}

// ===========================================================================
// RECEIPT — SP9 full closed loop
// ===========================================================================
// COMMAND : cargo test -p cgap --test self_play_full_closed_loop
// EXIT    : (filled in the lane report JSON terminus after the run below)
// COURTS  :
//   1. sp_full_closed_loop_50_iters — 50 iterations of the full sovereign
//      pipeline (ingress -> tick -> arrow -> radix join -> alternating
//      cascade -> seal -> git commit); final tick == 50, store == 7,296
//      rows, one commit per iteration, independent receipt-chain recompute
//      equals the running digest, FA4 per-iter allocation <= 32 KiB
//      constant from iter 2 (budget < one store-sized copy = the flip).
//   2. sp_chain_tamper_flip — anti-vacuity: flipping ONE byte of ONE
//      iteration's commit payload breaks the chain (recompute != recorded);
//      byte-exact restore restores it. A court whose flip does not fire is
//      a FAIL.
//   3. sp_zero_byte_seal_latch_fa9 — FA9 witnessed semantics: the seal
//      latches the 0-byte body pre-rotation with a typed RunOverrun
//      refusal (W6-F1); recorded as witnessed, not worked around.
// DIVERGENCES:
//   D1: cgap-crypto's PhiLease (A16) is NOT in cgap's dependency closure
//       (Cargo.toml off-limits to this lane) — the lease tick is a
//       monotonic ledger over the real penstock admission result, the
//       same form the landed SP2 court uses. Tick monotonicity + ceiling
//       are asserted; lease_id/issued_root bytes are not bound here.
//   D2: "arrow statement array" is carried by the bn3 StatementColumns
//       u64 lanes + cgap-arrow PresenceMask (the arrow-side array cgap
//       actually ships); cgap-arrow has no statement-array kernel.
// ===========================================================================
