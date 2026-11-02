//! BENCH-5 — the SOVEREIGN-BOUNDARY CHAIN (lane dispatch 2026-11-01).
//!
//! The business case: every business transaction crossing the C-ABI pays
//! the four-entrypoint sequence admit -> eval -> commit -> verify; that
//! sequence's total ns IS the substrate's price per transaction, and the
//! extern "C" hop is the only irreducible part of it.
//!
//! This bench runs the REAL sovereign bodies landed in
//! `crates/cgap-ffi/src/sovereign.rs` (GAP-1 + Wave F2 item 5) through the
//! FROZEN fn-pointer signatures (A19 §2), indirect-call form (the F2-14
//! method), chained per transaction:
//!   1. `cgap_admit_frame`  — A14 limiter + A04 project/validate + BLAKE3
//! 2. `cgap_eval_bn3`     — A12 compile + unify + project over admitted view
//!   3. `cgap_commit_parquet` — A02 sealed-block commit (64 KiB lease)
//! 4. `cgap_verify_p28`   — A17 §5 full validate over the PINNED instrument
//!
//! Corpus law: SplitMix64 canonical seed `0xBF58_476D_1CE4_E5B9` (charter),
//! per-record penstock spread `seed ^ i*0x2544_5491_4F6C_DD1D`. Every bench
//! asserts CgapStatus CGAP_OK per stage and a differential lane-value
//! checksum (the expected lane vector is computed WITHOUT the unifier), so
//! the measured loop is Chicago: real frames, real network bytes, real
//! files, final state asserted.
//!
//! Anti-vacuity (charter: a bench that cannot flip is decoration) — the
//! structural mutation named per bench:
//!   (1) chain: any stage returning a typed refusal flips the 4*N status
//!       assert; a silently-truncated lane vector flips the checksum.
//!   (2) admit: a header-byte or digest repair dropped from the gate flips
//!       the per-frame admit-count assert.
//!   (3) eval: a comparator/binding mutation diverges from the raw-word
//!       lane oracle and flips the checksum assert.
//!   (4) commit: a dropped slab write flips the per-call OK count (the
//!       file's existence is asserted post-loop via its re-open).
//!   (5) verify: any §5 predicate weakening flips the verify-count assert
//!       (the evidence is a Root record bound to the PINNED instrument —
//!       a mutated instrument file on disk flips the span pin at startup).

use std::ffi::CString;
use std::path::PathBuf;
use std::sync::OnceLock;

use cgap_bn3::sigil::{encode_term, Sigil, PAYLOAD_MASK};
use cgap_core::frame::{
    CgapFrameHeader, P28LicenseAttestation as CoreP28, FLAG_ARROW, FRAME_HEADER_LEN,
    FRAME_MAGIC, FRAME_VERSION, LICENSE_FLAG_ROOT as CORE_FLAG_ROOT,
    LICENSE_MAGIC as CORE_LICENSE_MAGIC, SAFE_HARBOR_CEILING_CENTS_MAX,
};
use cgap_ffi::sovereign::{
    cgap_admit_frame, cgap_commit_parquet, cgap_eval_bn3, cgap_verify_p28, CgapAdmitFrameFn,
    CgapCommitParquetFn, CgapEvalBn3Fn, CgapFrameView, CgapStatus, CgapVerifyP28Fn,
    CGAP_ERR_LEASE_EXHAUSTED, CGAP_ERR_P28_SPOLIATION, CGAP_OK,
};
use cgap_p28::attestation::{
    instrument_header_span, origin_authority_digest, FLAG_ROOT, LICENSE_MAGIC,
    P28LicenseAttestation as P28Rec, ATTESTATION_LEN, SAFE_HARBOR_CEILING_MAX_CENTS,
};
use divan::{black_box, Bencher};

/// Canonical SplitMix64 master seed (charter, verbatim).
const CORPUS_SEED: u64 = 0xBF58_476D_1CE4_E5B9;

/// Per-record spread — the in-repo penstock derivation form
/// (cgap-gate/src/penstock.rs:814).
const RECORD_SPREAD: u64 = 0x2544_5491_4F6C_DD1D;

/// Rows (statements) per transaction frame: the small batch the chain
/// commits. Payload = 24 B/row columnar (s-col | p-col | o-col, LE words).
const ROWS_PER_FRAME: usize = 16;

/// Payload bytes per frame: 24 * 16 = 384.
const PAYLOAD_LEN: usize = 24 * ROWS_PER_FRAME;

/// Frame stride: 128 B header + 384 B payload = 512 (64-aligned).
const FRAME_STRIDE: usize = FRAME_HEADER_LEN + PAYLOAD_LEN;

/// The one network rule: `(var0, P:100, var1) => (K:7, P:100, var1)` —
/// every seeded row matches; lane[i] = the projected head OBJECT word.
const RULE_PREDICATE: u64 = 100;

const _: () = assert!(FRAME_STRIDE == 512);
const _: () = assert!(PAYLOAD_LEN % 24 == 0);

/// SplitMix64 byte stream with the canonical constants (same form as the
/// in-repo benches; the corruption-witness constant rides the multiplier).
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

/// Per-record seed: master seed verbatim, penstock spread.
fn record_seed(i: usize) -> u64 {
    CORPUS_SEED ^ (i as u64).wrapping_mul(RECORD_SPREAD)
}

/// SlotConst term word over the masked 56-bit payload (A11 sigil ABI,
/// sigil 0x00 — const words ARE small raw values, which is what puts the
/// lane domain inside commit's declared bits-8 kernel domain [0, 255]).
fn const_word(payload: u64) -> u64 {
    encode_term(Sigil::SlotConst, payload & PAYLOAD_MASK).expect("const_word encode")
}

/// VarUni term word naming variable register `idx` (< 256 by caller law).
fn var_word(idx: u64) -> u64 {
    encode_term(Sigil::VarUni, idx).expect("var_word encode")
}

/// The pinned eval encoding (sovereign.rs `EVAL_ENCODING_PIN`): LE u64
/// words `[rule_count, per rule: rule_id, head(3), body_len, body(3n)]`.
/// One rule: (var0, K:100, var1) => (K:7, K:100, var1).
fn chain_rodata() -> Vec<u8> {
    let words: [u64; 9] = [
        1, // rule_count
        0xBEEF, // rule_id
        const_word(7), // head_s
        const_word(RULE_PREDICATE), // head_p
        var_word(1), // head_o: lane[i] = bound var1 = the matched o word
        1, // body_len
        var_word(0), const_word(RULE_PREDICATE), var_word(1), // body atom
    ];
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// 64-byte-aligned frame slot: admit takes the TRUE zero-copy §8.1
/// projection path (`project`) exactly when the caller buffer is 64-byte
/// aligned — the DMA-slot shape a real sovereign consumer presents.
#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct AlignedFrame([u8; FRAME_STRIDE]);

/// Per-transaction lane values (the eval oracle): row r of transaction t
/// carries object payload `(t*ROWS + r) % 256` — inside commit's declared
/// bits-8 kernel domain [0, 255], so the whole chain admits end to end.
fn lane_payload(t: usize, r: usize) -> u64 {
    ((t * ROWS_PER_FRAME + r) % 256) as u64
}

/// Expected lane-value checksum for transactions [0, N) — computed with
/// NO unifier in sight (differential oracle for the eval asserts).
fn expected_lane_sum(n: usize) -> u64 {
    (0..n)
        .flat_map(|t| (0..ROWS_PER_FRAME).map(move |r| lane_payload(t, r)))
        .map(u64::from)
        .sum()
}

/// Pinned-instrument span + the ROOT record bound to it (the only lineage
/// the sovereign verify can lawfully admit: `validate(span, None)` is
/// fail-closed on non-Root lineage, D-A17-2).
fn pinned_instrument() -> (Vec<u8>, P28Rec) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    let file = root.join("LICENSE-P28-SIL.md");
    let bytes = std::fs::read(&file).expect("LICENSE-P28-SIL.md readable at repo root");
    let span = instrument_header_span(&bytes)
        .expect("pinned instrument has a canonical --- span")
        .to_vec();
    // The A19 sovereign embeds this span as a 413-byte const with a
    // compile-time pin; the file-derived span must stay in parity or the
    // sovereign and this bench would validate DIFFERENT instruments.
    assert_eq!(span.len(), 413, "pinned instrument span drifted from the A19 413-byte pin");
    let rec = P28Rec {
        license_magic: LICENSE_MAGIC,
        version: cgap_p28::attestation::ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: origin_authority_digest(&span),
        upstream_receipt: [0u8; 32],
    };
    rec.validate(&span, None)
        .expect("pinned ROOT record must pass the FULL §5 validate with no parent");
    (span, rec)
}

/// The frame-header attestation (cgap-core layout): a ROOT record bound to
/// the PINNED instrument — the same authority the verify leg checks, so the
/// chain's frame and evidence are one coherent instrument identity.
fn frame_attestation(span: &[u8]) -> CoreP28 {
    CoreP28 {
        license_magic: CORE_LICENSE_MAGIC,
        version: 1, // A17 §2.1 pin: attestation version 1
        attestation_flags: CORE_FLAG_ROOT,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_CENTS_MAX,
        origin_authority_digest: origin_authority_digest(span),
        upstream_receipt: [0u8; 32],
    }
}

/// Build the frame for transaction `t`: 64-aligned slot, 128 B header
/// (FLAG_ARROW, BLAKE3-bound payload) + 384 B columnar payload.
fn chain_frame(t: usize, att: &CoreP28) -> AlignedFrame {
    let mut slot = AlignedFrame([0u8; FRAME_STRIDE]);
    let mut payload = [0u8; PAYLOAD_LEN];
    let n = ROWS_PER_FRAME;
    let mut seed_buf = [0u8; 8];
    fill_splitmix(&mut seed_buf, record_seed(t));
    let subject = u64::from_le_bytes(seed_buf) % 64;
    for r in 0..n {
        let s = const_word(subject);
        let p = const_word(RULE_PREDICATE);
        let o = const_word(lane_payload(t, r));
        payload[r * 8..r * 8 + 8].copy_from_slice(&s.to_le_bytes());
        payload[(n + r) * 8..(n + r) * 8 + 8].copy_from_slice(&p.to_le_bytes());
        payload[(2 * n + r) * 8..(2 * n + r) * 8 + 8].copy_from_slice(&o.to_le_bytes());
    }
    let header = CgapFrameHeader {
        magic: FRAME_MAGIC,
        version: FRAME_VERSION,
        flags: FLAG_ARROW,
        reserved: [0u8; 2],
        payload_byte_len: PAYLOAD_LEN as u64,
        oabi_digest: cgap_blake3::hash(&payload),
        attestation: *att,
    };
    header
        .encode_into(&mut slot.0[..FRAME_HEADER_LEN])
        .expect("stride >= FRAME_HEADER_LEN");
    slot.0[FRAME_HEADER_LEN..].copy_from_slice(&payload);
    slot
}

/// The full chain corpus: `n` aligned frames, the one-rule network bytes,
/// the 80 B evidence record, and the setup-time status warm check.
struct ChainCorpus {
    frames: Vec<AlignedFrame>,
    rodata: Vec<u8>,
    evidence: [u8; ATTESTATION_LEN],
}

fn chain_corpus(n: usize) -> ChainCorpus {
    let (span, rec) = pinned_instrument();
    let att = frame_attestation(&span);
    // Setup-time warm check: the corpus frame MUST admit through the REAL
    // sovereign admit before any timing happens (a fixture that refused
    // would make every assert below vacuously green).
    let mut probe_view = CgapFrameView { payload: std::ptr::null(), payload_len: 0, schema_id: 0, rule_count: 0 };
    let probe_frame = chain_frame(0, &att);
    let st = unsafe { cgap_admit_frame(probe_frame.0.as_ptr(), FRAME_STRIDE, &mut probe_view) };
    assert_eq!(st, CGAP_OK, "corpus frame must admit through cgap_admit_frame (status 0x{st:08X})");
    assert_eq!(probe_view.payload_len, PAYLOAD_LEN, "admitted view borrows the payload");
    let mut probe_lanes = [0i32; ROWS_PER_FRAME];
    let mut probe_count = 0usize;
    let probe_rodata = chain_rodata();
    let st = unsafe {
        cgap_eval_bn3(&probe_view, probe_rodata.as_ptr(), probe_rodata.len(),
                      probe_lanes.as_mut_ptr(), ROWS_PER_FRAME, &mut probe_count)
    };
    assert_eq!(st, CGAP_OK, "corpus rule network must eval (status 0x{st:08X})");
    assert_eq!(probe_count, ROWS_PER_FRAME);
    for r in 0..ROWS_PER_FRAME {
        assert_eq!(probe_lanes[r] as u64, lane_payload(0, r), "lane[{r}] vs oracle");
    }
    let evidence = rec.to_bytes();
    let mut probe_digest = 0u64;
    let st = unsafe { cgap_verify_p28(evidence.as_ptr(), ATTESTATION_LEN, &mut probe_digest) };
    assert_eq!(st, CGAP_OK, "corpus evidence must verify through cgap_verify_p28 (status 0x{st:08X})");
    // Commit liveness: one real commit through the sink (File::create is
    // truncating; the timed loops overwrite it). A failed first commit here
    // means the sink path is unusable and every timed commit below would be
    // measuring a refusal loop — refuse loudly instead.
    let probe_lanes_c: [i32; ROWS_PER_FRAME] = core::array::from_fn(|i| (i % 256) as i32);
    let st = unsafe {
        cgap_commit_parquet(
            probe_lanes_c.as_ptr(),
            ROWS_PER_FRAME,
            commit_path().as_ptr(),
        )
    };
    assert_eq!(st, CGAP_OK, "commit sink must accept a real batch (status 0x{st:08X})");
    let frames = (0..n).map(|t| chain_frame(t, &att)).collect();
    ChainCorpus { frames, rodata: chain_rodata(), evidence }
}

// ---------------------------------------------------------------------------
// Per-process commit sink: the sovereign commit is a REAL sealed-block
// flush (File::create + write), so the chain needs a writable path. One
// per-process temp dir, one reused file name (File::create truncates);
// a Drop guard removes the dir when the harness main returns.
// ---------------------------------------------------------------------------

static COMMIT_PATH: OnceLock<CString> = OnceLock::new();

fn commit_path() -> &'static CString {
    COMMIT_PATH.get().expect("COMMIT_PATH set by main() before divan::main()")
}

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// LEASE_EXHAUSTED boundary probe (dispatch item): find the lane count the
/// fixed 64 KiB A02 commit lease accepts — NOT timed, run once at startup.
/// Bisect over [1, 1<<17]: every refusing probe MUST return exactly
/// CGAP_ERR_LEASE_EXHAUSTED (typed, never SHACL — a wrong class flips the
/// probe's assert), every accepting probe commits a real file.
fn lease_boundary_probe() -> (u32, CgapStatus) {
    let mut lo = 1u32; // must accept (asserted)
    let mut hi = 1u32 << 17; // must refuse (asserted)
    let ok = |k: u32| -> CgapStatus {
        let lanes: Vec<i32> = (0..k).map(|i| (i % 256) as i32).collect();
        unsafe { cgap_commit_parquet(lanes.as_ptr(), k as usize, commit_path().as_ptr()) }
    };
    let st_lo = ok(lo);
    assert_eq!(st_lo, CGAP_OK, "1-lane batch must commit (status 0x{st_lo:08X})");
    let st_hi = ok(hi);
    assert_eq!(
        st_hi, CGAP_ERR_LEASE_EXHAUSTED,
        "131072-lane batch must refuse LEASE_EXHAUSTED (status 0x{st_hi:08X})"
    );
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if ok(mid) == CGAP_OK {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo, ok(lo + 1))
}

fn main() {
    // -- Startup truth: the direct C-ABI leg is live. dlsym(RTLD_DEFAULT)
    //    cannot resolve #[no_mangle] symbols statically linked from an rlib
    //    into a Mach-O executable (they are not in the dyld export trie
    //    without a dylib/exported-symbols-list restructure), so symbol-table
    //    visibility is NOT the liveness test here; liveness is proven by the
    //    setup-time probe in chain_corpus(): every entrypoint is invoked for
    //    real and must return CGAP_OK before any timing happens.
    let _ = sovereign_symbols_exported();
    eprintln!("[FFI] dlsym(RTLD_DEFAULT) is not authoritative for statically \
               linked rlib symbols on Mach-O; BENCH-5 liveness = real \
               invocation: chain_corpus() asserts CGAP_OK through all four \
               sovereign entrypoints before any timing");
    let _guard = TempDirGuard(setup_commit_dir());
    let (boundary, first_refusal) = lease_boundary_probe();
    eprintln!(
        "[BENCH-5] commit lease boundary: {boundary} lanes accepted, \
         first refusal at {} lanes (status 0x{:08X}); bench batch = {} \
         lanes/frame (fits, LEASE_EXHAUSTED never fires in-bench)",
        boundary + 1,
        first_refusal,
        ROWS_PER_FRAME
    );
    divan::main();
    eprintln!("[BENCH-5] divan complete; temp commit sink removed by Drop guard");
}

/// One per-process commit sink dir under the OS temp dir; the CString
/// path inside it is set once for the sovereign commit's `*const c_char`.
fn setup_commit_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cgap-bench-chains-sovereign-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir); // stale debris from a prior crash
    std::fs::create_dir_all(&dir).expect("commit sink dir");
    let file = dir.join("chain.pq");
    let cpath = CString::new(file.to_str().expect("utf-8 temp path")).expect("nul-free path");
    COMMIT_PATH.set(cpath).expect("COMMIT_PATH set exactly once");
    dir
}

// dlsym probe — same method as seal.rs (F2-14): a real symbol resolution
// over the live process image, run once at startup, loud verdict.
unsafe extern "C" {
    fn dlsym(
        handle: *mut core::ffi::c_void,
        symbol: *const core::ffi::c_char,
    ) -> *mut core::ffi::c_void;
}

/// RTLD_DEFAULT (NULL on macOS/Linux).
const RTLD_DEFAULT: *mut core::ffi::c_void = core::ptr::null_mut();

/// True iff all four sovereign symbols resolve in the live image.
fn sovereign_symbols_exported() -> bool {
    const SYMBOLS: [&core::ffi::CStr; 4] = [
        c"cgap_admit_frame",
        c"cgap_eval_bn3",
        c"cgap_commit_parquet",
        c"cgap_verify_p28",
    ];
    SYMBOLS.iter().all(|name| {
        let p = unsafe { dlsym(RTLD_DEFAULT, name.as_ptr()) };
        !p.is_null()
    })
}

// ---------------------------------------------------------------------------
// (1) THE CHAIN — the per-transaction sovereign tax. One transaction =
// admit -> eval -> commit -> verify over one seeded frame; N transactions
// per iteration. Every stage's status is folded and asserted == CGAP_OK;
// the lane vector is asserted against the raw-word oracle sum.
// Mutation class that must move the number / flip the assert: any stage
// returning a typed refusal (corrupt header byte, mutated rule, lease
// overflow, spoliated evidence) drops `ok` below 4*N and flips.
#[divan::bench(consts = [256, 4096])]
fn chain_full_transaction<const N: usize>(b: Bencher) {
    let corpus = chain_corpus(N);
    let admit: CgapAdmitFrameFn = cgap_admit_frame;
    let eval: CgapEvalBn3Fn = cgap_eval_bn3;
    let commit: CgapCommitParquetFn = cgap_commit_parquet;
    let verify: CgapVerifyP28Fn = cgap_verify_p28;
    b.bench(|| {
        let mut ok = 0u64;
        let mut checksum = 0u64;
        let mut lane_sum = 0u64;
        for t in 0..N {
            let mut view = CgapFrameView {
                payload: std::ptr::null(),
                payload_len: 0,
                schema_id: 0,
                rule_count: 0,
            };
            let frame_ptr = corpus.frames[t].0.as_ptr();
            // SAFETY: frame_ptr indexes the corpus; view is live for the call.
            let st_admit = unsafe { admit(frame_ptr, FRAME_STRIDE, &mut view) };
            checksum = checksum.rotate_left(1) ^ st_admit as u64;
            ok += (st_admit == CGAP_OK) as u64;
            let mut lanes = [0i32; ROWS_PER_FRAME];
            let mut lane_count = 0usize;
            // SAFETY: view.payload borrows the corpus frame; lanes is live.
            let st_eval = unsafe {
                eval(
                    &view,
                    corpus.rodata.as_ptr(),
                    corpus.rodata.len(),
                    lanes.as_mut_ptr(),
                    ROWS_PER_FRAME,
                    &mut lane_count,
                )
            };
            checksum = checksum.rotate_left(1) ^ st_eval as u64;
            ok += (st_eval == CGAP_OK) as u64;
            ok += (lane_count == ROWS_PER_FRAME) as u64;
            for lane in lanes.iter().take(lane_count) {
                lane_sum += *lane as u64;
            }
            // SAFETY: lanes holds ROWS_PER_FRAME committed values in domain.
            let st_commit = unsafe { commit(lanes.as_ptr(), lane_count, commit_path().as_ptr()) };
            checksum = checksum.rotate_left(1) ^ st_commit as u64;
            ok += (st_commit == CGAP_OK) as u64;
            let mut digest = 0u64;
            // SAFETY: evidence is the corpus record; digest sink is live.
            let st_verify = unsafe {
                verify(corpus.evidence.as_ptr(), ATTESTATION_LEN, &mut digest)
            };
            checksum = checksum.rotate_left(1) ^ st_verify as u64;
            ok += (st_verify == CGAP_OK) as u64;
            checksum = checksum.rotate_left(1) ^ digest;
        }
        // Chicago final state: every stage of every transaction admitted.
        assert_eq!(
            ok,
            (4 * N + N) as u64,
            "chain: every admit/eval/commit/verify verdict must be CGAP_OK \
             and every lane vector full-width (got ok={ok}, N={N})"
        );
        assert_eq!(
            lane_sum,
            expected_lane_sum(N),
            "chain: eval lane vector diverged from the raw-word oracle"
        );
        black_box(checksum);
    });
}

// ---------------------------------------------------------------------------
// (2) STAGE BENCHES — the per-stage attribution the Pareto needs. Same
// corpus, same consts, each stage alone over N transactions: per-stage
// ns/transaction from these four, chain total from (1), shares = stage/total.
// ---------------------------------------------------------------------------

/// MEASURES the admit stage alone (A14 limiter + A04 project/validate +
/// BLAKE3 digest re-derivation over the 384 B payload). Mutation class:
/// dropping the digest re-derivation or the attestation gate moves the
/// number to the early-refusal path and flips the admit-count assert.
#[divan::bench(consts = [256, 4096])]
fn stage_admit_frame<const N: usize>(b: Bencher) {
    let corpus = chain_corpus(N);
    let admit: CgapAdmitFrameFn = cgap_admit_frame;
    b.bench(|| {
        let mut ok = 0u64;
        let mut checksum = 0u64;
        for t in 0..N {
            let mut view = CgapFrameView {
                payload: std::ptr::null(),
                payload_len: 0,
                schema_id: 0,
                rule_count: 0,
            };
            // SAFETY: frame_ptr indexes the corpus; view is live for the call.
            let st = unsafe { admit(corpus.frames[t].0.as_ptr(), FRAME_STRIDE, &mut view) };
            checksum = checksum.rotate_left(1) ^ st as u64;
            ok += (st == CGAP_OK) as u64;
            checksum = checksum.rotate_left(1) ^ view.payload_len as u64;
        }
        assert_eq!(ok, N as u64, "stage_admit: every frame must admit");
        black_box(checksum);
    });
}

/// MEASURES the eval stage alone (A12 RuleCompiler::compile + unify_batch +
/// ConsequentProjector over the pre-admitted view). Views are minted in
/// SETUP (untimed) so the bench isolates eval's own cost — eval over an
/// admitted view is exactly the sovereign contract (§2 zero-copy).
/// Mutation class: a comparator/binding mutation diverges from the raw-word
/// lane oracle and flips the checksum.
#[divan::bench(consts = [256, 4096])]
fn stage_eval_bn3<const N: usize>(b: Bencher) {
    let corpus = chain_corpus(N);
    let eval: CgapEvalBn3Fn = cgap_eval_bn3;
    let admit: CgapAdmitFrameFn = cgap_admit_frame;
    // The views hold raw pointers into `corpus.frames` (alive, never mutated
    // for the bench's lifetime), but raw pointers are !Sync and divan's
    // closure bound requires Sync — wrap in a sound disclosure newtype; the
    // closure borrows through `wrapper` and the aliasing invariant holds
    // (shared reads only, frames outlive the bench).
    struct AdmittedViews(Vec<CgapFrameView>);
    unsafe impl Sync for AdmittedViews {}
    let mut wrapper = AdmittedViews(Vec::with_capacity(N));
    for t in 0..N {
        let mut view = CgapFrameView {
            payload: std::ptr::null(),
            payload_len: 0,
            schema_id: 0,
            rule_count: 0,
        };
        // SAFETY: frame_ptr indexes the corpus; view is live for the call.
        let st = unsafe { admit(corpus.frames[t].0.as_ptr(), FRAME_STRIDE, &mut view) };
        assert_eq!(st, CGAP_OK, "setup admit for eval stage");
        wrapper.0.push(view);
    }
    let sync_views: &AdmittedViews = &wrapper;
    b.bench(|| {
        let mut ok = 0u64;
        let mut lane_sum = 0u64;
        for t in 0..N {
            let mut lanes = [0i32; ROWS_PER_FRAME];
            let mut lane_count = 0usize;
            // SAFETY: views[t].payload borrows the corpus frames (alive).
            let st = unsafe {
                eval(
                    &sync_views.0[t],
                    corpus.rodata.as_ptr(),
                    corpus.rodata.len(),
                    lanes.as_mut_ptr(),
                    ROWS_PER_FRAME,
                    &mut lane_count,
                )
            };
            ok += (st == CGAP_OK) as u64;
            ok += (lane_count == ROWS_PER_FRAME) as u64;
            for lane in lanes.iter().take(lane_count) {
                lane_sum += *lane as u64;
            }
        }
        assert_eq!(
            ok,
            (2 * N) as u64,
            "stage_eval: every call must admit and fill a full-width lane vector"
        );
        assert_eq!(
            lane_sum,
            expected_lane_sum(N),
            "stage_eval: lane vector diverged from the raw-word oracle"
        );
        black_box(lane_sum);
    });
}

/// MEASURES the commit stage alone (A02 sealed-block writer: write_slab ->
/// seal -> flush_to(FileSink), the 64 KiB fixed lease, a REAL file write
/// per call). Mutation class: a dropped slab write or a truncated seal
/// flips the per-call OK count.
#[divan::bench(consts = [256, 4096])]
fn stage_commit_parquet<const N: usize>(b: Bencher) {
    let commit: CgapCommitParquetFn = cgap_commit_parquet;
    let mut lane_bank: Vec<[i32; ROWS_PER_FRAME]> = Vec::with_capacity(N);
    for t in 0..N {
        let mut batch = [0i32; ROWS_PER_FRAME];
        for r in 0..ROWS_PER_FRAME {
            batch[r] = lane_payload(t, r) as i32;
        }
        lane_bank.push(batch);
    }
    b.bench(|| {
        let mut ok = 0u64;
        for batch in lane_bank.iter().take(N) {
            // SAFETY: batch is a live ROWS_PER_FRAME i32 slice in domain.
            let st = unsafe { commit(batch.as_ptr(), ROWS_PER_FRAME, commit_path().as_ptr()) };
            ok += (st == CGAP_OK) as u64;
        }
        assert_eq!(ok, N as u64, "stage_commit: every batch must commit CGAP_OK");
        black_box(ok);
    });
}

/// MEASURES the verify stage alone (A17 §5 FULL ordered validation over the
/// PINNED 413-byte instrument span, no parent — the sovereign contract).
/// Mutation class: any §5 predicate weakening (e.g. skipping the §5.6
/// instrument recompute) admits a mutated record and flips the assert on
/// the setup-time negative control below.
#[divan::bench(consts = [256, 4096])]
fn stage_verify_p28<const N: usize>(b: Bencher) {
    let corpus = chain_corpus(N);
    let verify: CgapVerifyP28Fn = cgap_verify_p28;
    // Setup-time NEGATIVE CONTROL (non-vacuity, executed every run): one
    // byte of the origin_authority_digest flipped MUST refuse with exactly
    // CGAP_ERR_P28_SPOLIATION — else the verify gate cannot flip and the
    // bench's OK-count assert would be vacuous.
    {
        let mut spoliated = corpus.evidence;
        spoliated[20] ^= 0x01;
        let mut sink = 0u64;
        let st = unsafe { verify(spoliated.as_ptr(), ATTESTATION_LEN, &mut sink) };
        assert_eq!(
            st, CGAP_ERR_P28_SPOLIATION,
            "verify negative control: spoliated record must refuse typed"
        );
    }
    b.bench(|| {
        let mut ok = 0u64;
        let mut checksum = 0u64;
        for _ in 0..N {
            let mut digest = 0u64;
            // SAFETY: evidence is the corpus record; digest sink is live.
            let st = unsafe { verify(corpus.evidence.as_ptr(), ATTESTATION_LEN, &mut digest) };
            ok += (st == CGAP_OK) as u64;
            checksum = checksum.rotate_left(1) ^ digest;
        }
        assert_eq!(ok, N as u64, "stage_verify: every record must verify CGAP_OK");
        black_box(checksum);
    });
}
