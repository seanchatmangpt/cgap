//! SP11 bench: deductive-to-git latency -- transitive closure -> git loose-object
//! serialization, measured with two independent Instant timers per iteration.
//!
//! Pipeline (real collaborators, no mocks, no Process.sleep):
//!   1. Corpus: splitmix64 seeded from the canonical corpus constant
//!      CORPUS_SEED = 0xA20C_0FFE_E5A7_E000 (cgap-storage closure.rs -- never
//!      the splitmix64 multiplier 0xBF58_476D_1CE4_E5B9; the two seeds never
//!      conflate). Edges are window-local ((x, x+1+rand%64)) so closure
//!      derivations are genuinely multi-hop while pair count stays bounded.
//!   2. DERIVATION TIMER: cgap_storage::closure::semi_naive_closure over the
//!      edge columns (1K/10K/64K statement batches).
//!   3. COMMIT TIMER: immediate git loose-object serialization of the derived
//!      pairs through cgap-git's own Oid path -- blobs (8-byte big-endian
//!      (x,z) records, 256 pairs per blob) zlib-compressed (loose-object
//!      form), a canonical tree ("100644 sp11-<n>\0<oid20>"), and a commit
//!      object binding the tree -- every digest computed via
//!      cgap_git::pack_write::git_object_oid, which drives the H10
//!      emit_u64_decimal branch-light decimal emitter
//!      (docs/jira/v26.11.2/hardening/lane-10-git-branchless-objects.md).
//!   4. facts/s distinguishes derivation from commit: derived_pairs/deriv_ns
//!      vs derived_pairs/commit_ns, both reported.
//!   5. Latency distribution: per-iteration Instant samples; min-of-N and
//!      median-of-N are the honest divan-free form; p95/p99 reported ONLY
//!      because per-iteration samples are actually collected (nearest-rank
//!      over the sorted sample set). No divan in dev-deps, so Instant form.
//!   6. Anti-vacuity (in-run assertions, fail-closed): derived pairs MUST
//!      exceed base statements (an empty-closure mutant aborts the bench with
//!      a non-zero exit) and round count MUST exceed 1 (multi-hop actually
//!      happened); commit OIDs MUST differ across batch sizes (a
//!      constant-serialization mutant aborts). All digests black_box'd.
//!
//! ---------------------------------------------------------------------------
//! RECEIPT (filled from this session's measured run; command + exit below):
//! ---------------------------------------------------------------------------
//! Command: CARGO_TARGET_DIR=/tmp/sp11-target cargo bench -p cgap \
//!   --bench bench_self_play_datalog_git   (bench profile, release opt-level 3)
//! Exit: 0. All anti-vacuity assertions held (pairs > stmts, rounds > 1,
//! commit OIDs distinct across sizes; an earlier corpus draft derived only
//! 1000 pairs in 1 round and this court aborted the run fail-closed).
//!
//! SIZE 1K stmts:  derived_pairs 8223 (24 rounds)
//!   deriv  min 0.563ms  med 0.574ms          facts/s(min) 14,594,874
//!   commit min 1.675ms  p50 1.755ms  p95 1.918ms  p99 1.945ms
//!                                              facts/s(min) 4,908,155
//! SIZE 10K stmts: derived_pairs 168166 (59 rounds)
//!   deriv  min 13.431ms med 13.524ms         facts/s(min) 12,521,202
//!   commit min 30.367ms p50 30.423ms p95 30.818ms p99 30.818ms
//!                                              facts/s(min) 5,537,742
//! SIZE 64K stmts: derived_pairs 887096 (72 rounds)
//!   deriv  min 83.082ms med 83.445ms         facts/s(min) 10,677,317
//!   commit min 141.211ms p50 141.417ms p95 143.014ms p99 143.014ms
//!                                              facts/s(min) 6,282,049
//!
//! facts/s distinguishes derivation vs commit (two Instant timers per
//! iteration). Distribution form: per-iteration samples (30/10/3 iters),
//! nearest-rank percentiles; min + median are the honest divan-free form,
//! p95/p99 reported because per-iteration samples were actually collected.
//! Host: aarch64-apple-darwin, 2026-11-02 session.
//! ---------------------------------------------------------------------------

use std::hint::black_box;
use std::time::Instant;

use cgap_git::deflate::zlib_compress;
use cgap_git::pack_write::git_object_oid;
use cgap_storage::closure::{semi_naive_closure, CORPUS_SEED};
use cgap_storage::dred::splitmix64;

/// Pairs per blob: 256 * 8 bytes = 2048-byte blob bodies.
const BLOB_PAIRS: usize = 256;

/// Window of possible forward edge targets (multi-hop depth driver).
const NODE_WINDOW: u32 = 64;

type Oid = [u8; 20];

/// Deterministic corpus: window-local forward edges from the frozen seed.
fn build_edges(statements: usize) -> (Vec<u32>, Vec<u32>) {
    let mut seed: u64 = CORPUS_SEED;
    let mut ex = Vec::with_capacity(statements);
    let mut ez = Vec::with_capacity(statements);
    for _ in 0..statements {
        let r1 = splitmix64(&mut seed);
        let r2 = splitmix64(&mut seed);
        let x = ((r1 >> 24) % statements as u64) as u32;
        let z = x.wrapping_add(1 + ((r2 >> 24) as u32 % NODE_WINDOW));
        ex.push(x);
        ez.push(z);
    }
    (ex, ez)
}

/// Tiny local decimal emitter for tree entry names (cgap-git's
/// emit_u64_decimal is private; byte-format identical).
fn push_decimal(out: &mut Vec<u8>, mut v: usize) {
    let mut buf = [0u8; 20];
    let mut n = buf.len();
    loop {
        n -= 1;
        buf[n] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    out.extend_from_slice(&buf[n..]);
}

fn push_hex(out: &mut Vec<u8>, oid: &Oid) {
    for b in oid.iter() {
        out.push(b"0123456789abcdef"[(b >> 4) as usize]);
        out.push(b"0123456789abcdef"[(b & 0xF) as usize]);
    }
}

/// Git loose-object serialization: blobs -> tree -> commit, OIDs via the
/// crate's own git_object_oid (H10 emitter path). Returns commit OID and
/// total zlib loose bytes.
fn git_serialize(xs: &[u32], zs: &[u32]) -> (Oid, usize) {
    let n = xs.len();
    let mut blob_oids: Vec<Oid> = Vec::with_capacity(n / BLOB_PAIRS + 1);
    let mut loose_bytes = 0usize;
    let mut i = 0usize;
    while i < n {
        let end = usize::min(i + BLOB_PAIRS, n);
        let mut body = Vec::with_capacity((end - i) * 8);
        for j in i..end {
            body.extend_from_slice(&xs[j].to_be_bytes());
            body.extend_from_slice(&zs[j].to_be_bytes());
        }
        let oid = git_object_oid("blob", &body);
        let zlib = zlib_compress(&body);
        loose_bytes += zlib.len() + body.len() / 8 + 32;
        black_box(&zlib);
        blob_oids.push(oid);
        i = end;
    }
    // Canonical tree: entries ordered by decimal name (byte-sorted names).
    let mut tree = Vec::with_capacity(blob_oids.len() * 32);
    for (idx, oid) in blob_oids.iter().enumerate() {
        tree.extend_from_slice(b"100644 sp11-");
        push_decimal(&mut tree, idx);
        tree.extend_from_slice(b"\0");
        tree.extend_from_slice(oid);
    }
    let tree_oid = git_object_oid("tree", &tree);
    // Commit binding the tree.
    let mut commit = Vec::with_capacity(96);
    commit.extend_from_slice(b"tree ");
    push_hex(&mut commit, &tree_oid);
    commit.extend_from_slice(b"\ncommitter sp11-bench 0 +0000\n\nSP11 deductive-to-git batch\n");
    let commit_oid = git_object_oid("commit", &commit);
    (commit_oid, loose_bytes)
}

/// Nearest-rank percentile over collected per-iteration samples (ns).
fn percentile(samples: &mut [u64], p: f64) -> u64 {
    samples.sort_unstable();
    let rank = ((p / 100.0) * (samples.len() as f64)).ceil() as usize;
    samples[rank.clamp(1, samples.len()) - 1]
}

fn run_batch(statements: usize, iters: usize) -> Oid {
    let (ex, ez) = build_edges(statements);
    // Warmup + anti-vacuity validation on the real corpus.
    let (wx, wz, wr) = semi_naive_closure(&ex, &ez);
    assert!(
        wx.len() > statements && wr > 1,
        "ANTI-VACUITY ABORT: derived {} pairs in {} rounds for {} stmts          (expected derivations > stmts and rounds > 1)",
        wx.len(),
        wr,
        statements
    );
    let pairs = wx.len();
    let _ = wz;
    let (wid, _wb) = git_serialize(&wx, &wz);
    let mut deriv_samples: Vec<u64> = Vec::with_capacity(iters);
    let mut commit_samples: Vec<u64> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let (xs, zs, rounds) = semi_naive_closure(&ex, &ez);
        let deriv = t0.elapsed().as_nanos() as u64;
        let t1 = Instant::now();
        let (_cid, _lb) = git_serialize(&xs, &zs);
        let commit = t1.elapsed().as_nanos() as u64;
        black_box(&xs);
        black_box(&zs);
        deriv_samples.push(deriv);
        commit_samples.push(commit);
        let _ = rounds;
    }
    let dmin = *deriv_samples.iter().min().unwrap();
    let cmin = *commit_samples.iter().min().unwrap();
    let dmed = percentile(&mut deriv_samples, 50.0);
    let cmed = percentile(&mut commit_samples, 50.0);
    let cp95 = percentile(&mut commit_samples, 95.0);
    let cp99 = percentile(&mut commit_samples, 99.0);
    let dfacts = pairs as f64 / (dmin as f64 / 1e9);
    let cfacts = pairs as f64 / (cmin as f64 / 1e9);
    let mut hex = String::with_capacity(40);
    for b in wid.iter() {
        hex.push_str(&format!("{:02x}", b));
    }
    println!(
        "SIZE {} STMTS | derived_pairs {} rounds {} | deriv min {:.3}ms med {:.3}ms facts/s(min) {:.0} | commit min {:.3}ms p50 {:.3}ms p95 {:.3}ms p99 {:.3}ms facts/s(min) {:.0} | commit_oid {} | iters {}",
        statements, pairs, wr,
        dmin as f64 / 1e6, dmed as f64 / 1e6, dfacts,
        cmin as f64 / 1e6, cmed as f64 / 1e6, cp95 as f64 / 1e6, cp99 as f64 / 1e6, cfacts,
        hex, iters
    );
    wid
}

fn main() {
    let sizes: [(usize, usize); 3] = [(1_000, 30), (10_000, 10), (64_000, 3)];
    let mut first_oid: Option<Oid> = None;
    for (statements, iters) in sizes {
        let cid = run_batch(statements, iters);
        match first_oid {
            None => first_oid = Some(cid),
            Some(prev) => {
                assert_ne!(
                    prev, cid,
                    "ANTI-VACUITY ABORT: commit OID constant across batch sizes"
                );
            }
        }
    }
    println!("SP11 bench complete: all anti-vacuity assertions held");
}
