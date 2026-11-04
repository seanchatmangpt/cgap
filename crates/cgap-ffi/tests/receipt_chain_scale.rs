//! AB-14 chain-scale stress — `RingReceiptChain` at 100k links.
//!
//! Subject (read-only): `cgap_ffi::receipt_chain::RingReceiptChain`
//! (`crates/cgap-ffi/src/receipt_chain.rs`) — per-transition BLAKE3
//! Merkle receipts over the ring commit leg.
//!
//! What is measured, not asserted:
//! - O(n) append + full-chain verify cost at 100k links (min-of-3).
//! - Span-verify cost from 20 checkpoints at span sizes 1k/10k/50k:
//!   cost tracks the span, not the chain length (the O(span) fast-path
//!   shape a checkpointed verifier would buy).
//! - Tamper-at-position-50k: the prefix-integrity law witnessed at
//!   scale (spans before the tamper verify clean; the covering span and
//!   `verify` refuse typed).
//! - Alloc/wall census per 1k links via the real `cgap-audit`
//!   `CountingAlloc` global allocator (linked by this test binary).
//!
//! DISCLOSURES (read-only subject, stated not hidden):
//! 1. The existing impl has NO `verify_span`. The span curve is measured
//!    through a TEST-LOCAL span-recompute helper that runs the SAME link
//!    law (`BLAKE3(prev || block || license)`) over `receipts[s..e]`
//!    against the live block/license bytes — it measures what a
//!    checkpointed fast path costs, it does not add a production API.
//! 2. Block payloads cycle over a 256-block pool of real sealed blocks
//!    (real writer, real `seal(None)`, distinct payloads) instead of
//!    100k distinct 64 KiB buffers — 100k leaked 64 KiB page buffers is
//!    ~6.4 GB; the per-link hash cost is identical either way, and every
//!    link still recomputes over real committed bytes.
//! 3. Alloc tracking: linking `cgap-audit` installs `CountingAlloc` for
//!    this test binary only (same pattern as tests/alloc_courts.rs).

#![cfg(not(target_os = "none"))]

use std::time::Instant;

use cgap_audit::alloc_counter::with_alloc_tracking;
use cgap_blake3::hash;
use cgap_ffi::receipt_chain::{ReceiptChainRefusal, RingReceiptChain, GENESIS};
use cgap_p28::attestation::{
    P28LicenseAttestation, ATTESTATION_VERSION, FLAG_ROOT, LICENSE_MAGIC,
};
use cgap_parquet::footer::SealedBlock;
use cgap_parquet::mmap_writer::{SlabView, StaticPageBuffer};

const N_LINKS: usize = 100_000;
const POOL: usize = 256;
const BUCKET: usize = 1_000;

fn root_license() -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: 1_000_000_000,
        origin_authority_digest: [7u8; 32],
        upstream_receipt: [0u8; 32],
    }
}

/// Pool of `POOL` distinct REAL sealed blocks (real writer, real seal).
fn block_pool() -> Vec<&'static SealedBlock<'static>> {
    let mut blocks = Vec::with_capacity(POOL);
    for i in 0..POOL {
        let buf: &'static mut StaticPageBuffer<65_536> =
            Box::leak(Box::new(StaticPageBuffer::<65_536>::new()));
        let lane_data: [u32; 4] = [
            i as u32,
            ((i + 1) % 256) as u32,
            ((i + 2) % 256) as u32,
            ((i + 3) % 256) as u32,
        ];
        let mut writer = buf.writer();
        writer
            .write_slab(&SlabView {
                lanes: &[&lane_data[..]],
                rows: 4,
                bits: 8,
            })
            .expect("pool slab writes");
        let sealed: &'static SealedBlock<'static> =
            Box::leak(Box::new(writer.seal(None).expect("pool seal")));
        blocks.push(sealed);
    }
    blocks
}

/// One full append run over `n` links against the cyclic block pool.
/// Returns the chain, the blocks slice reference list, and per-bucket
/// (wall_ns, allocs, bytes) census.
fn build_chain(
    n: usize,
    pool: &[&'static SealedBlock<'static>],
    license: &P28LicenseAttestation,
) -> (
    RingReceiptChain,
    Vec<&'static SealedBlock<'static>>,
    Vec<(u128, usize, usize)>,
) {
    let mut chain = RingReceiptChain::new();
    let mut blocks: Vec<&'static SealedBlock<'static>> = Vec::with_capacity(n);
    let mut census = Vec::with_capacity(n / BUCKET);
    for i in 0..n {
        let b = pool[i % pool.len()];
        let t = Instant::now();
        let (_t_window, report) = with_alloc_tracking(|| {
            chain
                .append(b, license)
                .expect("scale append must mint a receipt");
        });
        if (i + 1) % BUCKET == 0 {
            census.push((t.elapsed().as_nanos(), report.count, report.bytes));
        }
        blocks.push(b);
    }
    (chain, blocks, census)
}

/// TEST-LOCAL span verify (disclosure 1): recompute the link law over
/// `receipts[start..end]` against the live block/license bytes, checking
/// prev-hash linkage back to the stored neighbor (or GENESIS at 0).
fn verify_span(
    chain: &RingReceiptChain,
    blocks: &[&SealedBlock],
    license: &P28LicenseAttestation,
    start: usize,
    end: usize,
) -> Result<(), ReceiptChainRefusal> {
    let receipts = chain.receipts();
    if receipts.is_empty() || end > receipts.len() || start >= end {
        return Err(ReceiptChainRefusal::EmptyChain);
    }
    if license.validate_intrinsic().is_err() {
        return Err(ReceiptChainRefusal::Spoliation);
    }
    let license_bytes = license.to_bytes();
    if hash(&license_bytes) != receipts[0].license_digest {
        return Err(ReceiptChainRefusal::Spoliation);
    }
    let mut prev = if start == 0 {
        GENESIS
    } else {
        receipts[start - 1].hash
    };
    for (i, r) in receipts[start..end].iter().enumerate() {
        let i = start + i;
        if r.index != i as u64 || r.prev_hash != prev {
            return Err(ReceiptChainRefusal::Spoliation);
        }
        let bytes = blocks[i].bytes();
        if r.block_digest != hash(bytes)
            || r.license_digest != hash(&license_bytes)
            || r.hash != hash(&[prev.as_slice(), bytes, license_bytes.as_slice()].concat())
        {
            return Err(ReceiptChainRefusal::Spoliation);
        }
        prev = r.hash;
    }
    Ok(())
}

fn ms(ns: u128) -> f64 {
    ns as f64 / 1_000_000.0
}

#[test]
fn scale_append_verify_100k_census() {
    let license = root_license();
    let pool = block_pool();
    let mut best = None;
    let mut verify_runs: Vec<f64> = Vec::new();
    let mut span10_runs: Vec<f64> = Vec::new();
    for run in 0..3 {
        let t_all = Instant::now();
        let (chain, blocks, census) = build_chain(N_LINKS, &pool, &license);
        let append_ms = ms(t_all.elapsed().as_nanos());
        let t_verify = Instant::now();
        let verdict = chain.verify(&blocks, &license);
        let verify_ms = ms(t_verify.elapsed().as_nanos());
        assert_eq!(verdict, Ok(()), "run {run}: full-chain verify must pass");
        assert_eq!(chain.len(), N_LINKS);
        // Genesis law intact at scale: link 0 anchors the all-zeros digest.
        assert_eq!(chain.receipts()[0].prev_hash, GENESIS);
        // O(n) witness: full verify >= 5x a 10k prefix span (a linear scan
        // over 10x the links). Paired min-of-3 ratio fence: each side takes
        // its OWN best-of-3, so a load spike on a single run cannot break
        // the fence one-sidedly (no-sleep doctrine: relative ratio fence
        // against an independent best-case baseline, never an absolute ms
        // bound or a single-shot ratio).
        let t10 = Instant::now();
        verify_span(&chain, &blocks, &license, 0, 10_000).expect("prefix span");
        let span10_ms = ms(t10.elapsed().as_nanos());
        span10_runs.push(span10_ms);
        verify_runs.push(verify_ms);
        eprintln!(
            "[run {run}] append 100k: {append_ms:.1}ms | verify_all: {verify_ms:.1}ms | \
             span(0..10k): {span10_ms:.3}ms"
        );
        if best.as_ref().map_or(true, |(a, _)| append_ms < *a) {
            best = Some((append_ms, (census, verify_ms)));
        }
    }
    let min_verify = verify_runs.iter().cloned().fold(f64::INFINITY, f64::min);
    let min_span10 = span10_runs.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(
        min_verify >= 5.0 * min_span10,
        "O(n) law: best verify_all ({min_verify:.1}ms) must dominate the best \
         10k prefix span ({min_span10:.3}ms) across min-of-3 paired runs"
    );
    let (append_ms, (census, _)) = best.expect("min-of-3 runs");
    let first = census[0];
    let last = *census.last().expect("non-empty census");
    eprintln!(
        "[MIN-OF-3] append 100k: {append_ms:.1}ms | verify_all: {min_verify:.1}ms | \
         per-1k wall first {}/last {} ms (x{:.2}) | per-1k allocs first {} -> last {} | \
         per-1k bytes first {} -> last {}",
        ms(first.0),
        ms(last.0),
        ms(last.0) as f64 / ms(first.0).max(1e-9),
        first.1,
        last.1,
        first.2,
        last.2,
    );
    // The append is a flat per-link law: bucket 100's wall cost must not
    // blow up past 20x bucket 1's (Vec growth dominates; caught here only
    // as a gross-antilinear tripwire, never a tight bound).
    let b1 = census[0];
    let b100 = census[99];
    assert!(
        ms(b100.0) < 20.0 * ms(b1.0).max(1e-6),
        "late append buckets must not blow up vs early buckets"
    );
}

#[test]
fn verify_span_from_checkpoints_o_span_curve() {
    let license = root_license();
    let pool = block_pool();
    let (chain, blocks, _) = build_chain(N_LINKS, &pool, &license);
    assert_eq!(chain.verify(&blocks, &license), Ok(()));

    let spans = [1_000usize, 10_000, 50_000];
    // 20 checkpoints for the small spans; the 50k span samples every 4th
    // (debug-profile wall clock, bounded suite time).
    let all_checkpoints: Vec<usize> = (0..20).map(|c| c * (N_LINKS - spans[2]) / 19).collect();
    let mut curve = std::collections::BTreeMap::new();
    for span in spans {
        let mut best = u128::MAX;
        let stride = if span == 50_000 { 4 } else { 1 };
        for &cp in all_checkpoints.iter().skip(0).step_by(stride) {
            if cp + span > N_LINKS {
                continue;
            }
            // min-of-3 per (checkpoint, span) cell.
            for _ in 0..3 {
                let t = Instant::now();
                verify_span(&chain, &blocks, &license, cp, cp + span).expect("clean span");
                best = best.min(t.elapsed().as_nanos());
            }
        }
        curve.insert(span, ms(best));
    }
    for (span, t) in &curve {
        eprintln!("[SPAN-CURVE] span {span}: best {:.3}ms (20 checkpoints, min-of-3)", t);
    }
    // O(span) law measured: 50k-span cost must dominate 1k-span cost on
    // the same chain (>= 20x for a 50x span; loose floor).
    let t1k = curve[&1_000];
    let t50k = curve[&50_000];
    assert!(
        t50k >= 20.0 * t1k,
        "span cost must track the span: 50k span {:.3}ms vs 1k span {:.3}ms",
        t50k,
        t1k
    );
    // Position-independence: a 1k span costs the same near the head as
    // near the tail (loose 5x ceiling; the helper re-evaluates only its
    // window, never the prefix).
    let mut tail = u128::MAX;
    let cp = N_LINKS - 1_000;
    for _ in 0..3 {
        let t = Instant::now();
        verify_span(&chain, &blocks, &license, cp, cp + 1_000).expect("tail span");
        tail = tail.min(t.elapsed().as_nanos());
    }
    assert!(
        ms(tail) < 5.0 * t1k,
        "1k span near tail ({:.3}ms) must not cost more than 5x near head ({t1k:.3}ms)",
        ms(tail)
    );
}

#[test]
fn tamper_at_50k_prefix_integrity_at_scale() {
    let license = root_license();
    let pool = block_pool();
    let (chain, blocks, _) = build_chain(N_LINKS, &pool, &license);

    // Negative control (non-vacuity): pristine full verify + spans on
    // BOTH sides of the future tamper point verify clean.
    assert_eq!(chain.verify(&blocks, &license), Ok(()));
    verify_span(&chain, &blocks, &license, 0, 50_000).expect("pristine prefix");
    verify_span(&chain, &blocks, &license, 49_999, 50_001).expect("pristine window");

    // TAMPER: flip one stored byte of the receipt at position 50k
    // (through the documented audit/repair seam, copies-of-one field).
    let mut tampered = chain.clone();
    tampered.receipts_mut()[50_000].hash[0] ^= 0xFF;

    // verify_all refuses typed at scale.
    assert_eq!(
        tampered.verify(&blocks, &license),
        Err(ReceiptChainRefusal::Spoliation),
        "single flipped link at 50k must refuse full-chain verify"
    );
    // The minimal covering span refuses too (2 links).
    assert_eq!(
        verify_span(&tampered, &blocks, &license, 49_999, 50_001),
        Err(ReceiptChainRefusal::Spoliation),
        "covering span must refuse"
    );
    // PREFIX-INTEGRITY LAW AT SCALE: every span BEFORE the tamper
    // verifies clean — the tamper at 50k does not retroactively spoil
    // positions 0..50k's stored links, and the covering-span refusal
    // localizes the damage to exactly one window.
    verify_span(&tampered, &blocks, &license, 0, 50_000)
        .expect("prefix before the tamper must verify clean");
    // And a 10k prefix spot-check near the boundary from a checkpoint.
    verify_span(&tampered, &blocks, &license, 40_000, 50_000)
        .expect("prefix checkpoint span must verify clean");
}
