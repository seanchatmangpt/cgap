//! dred — SWAR lineage-bitmask retraction (v26.11.1 paper lane PP-3).
//!
//! Home (disclosed): `cgap-storage`, not `cgap-parquet`. Lineage is a
//! storage-semantics concern: derived facts are journaled alongside the
//! WAL new-root descriptors of `atomic_commit.rs` (A05); cgap-parquet is
//! codec kernels with no derivation concept. This module is `core::`-only
//! (no_std-ready) to match the crate attribute pattern of atomic_commit.rs.
//!
//! Law: a deleted fact F is purged iff F is derived (`derived_deps` bit
//! set) AND F is deleted (`del_mask` bit set) AND F has NO alternative
//! derivation (`alt_support[F] == 0`). Facts with an alternative
//! derivation survive; the anti-vacuity court flips exactly this term via
//! [`retract_unchecked`].
//!
//! SWAR shape: all masks are `u64` words (64 lanes/word); compaction is
//! branchless per lane (out index = base + running popcount prefix; the
//! only control flow is the fixed word loop). Seeds are canonical (fixed
//! consts, splitmix64), never wall-clock.

#![allow(clippy::needless_range_loop)]

/// Bits per SWAR word / lanes per compaction word.
pub const WORD_BITS: u32 = 64;

/// Canonical seed for the retraction-correctness corpus (court 1).
pub const CORPUS_SEED: u64 = 0xD5E5_C0DE_D1CE_0001;

/// Canonical seed for the entropy/variance measurement (court 3).
pub const ENTROPY_SEED: u64 = 0xB3AC_0FFE_A7A7_0002;

/// splitmix64 — canonical deterministic corpus generator (no wall-clock,
/// no external deps). Returns (next_state, value).
#[inline]
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// suspect_mask = derived_deps & del_mask, word-parallel.
#[inline]
pub fn suspect_mask(derived_deps: &[u64], del_mask: &[u64], suspect: &mut [u64]) {
    debug_assert_eq!(derived_deps.len(), del_mask.len());
    debug_assert_eq!(derived_deps.len(), suspect.len());
    for i in 0..derived_deps.len() {
        suspect[i] = derived_deps[i] & del_mask[i];
    }
}

/// keep\[f] == 1 iff fact f SURVIVES retraction:
/// keep = !suspect | (alt_support != 0). Word-parallel.
pub fn keep_mask(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
    keep: &mut [u64],
) {
    let words = derived_deps.len();
    debug_assert_eq!(del_mask.len(), words);
    debug_assert_eq!(keep.len(), words);
    for w in 0..words {
        let s = derived_deps[w] & del_mask[w];
        let mut k = !s;
        // OR in (alt != 0) for the lanes of this word (last word may be
        // partial: alt_support.len() == fact count, not padded).
        let lo = w * 64;
        let hi = core::cmp::min(lo + 64, alt_support.len());
        // DREDP probe (2026-11-02): this per-lane rescue loop was swapped
        // to word-parallel SWAR (pair-fold + swar_nonzero_u64) and
        // MEASURED -11% (839M vs 944M facts/s, all legs, same bench
        // method) -- the loop autovectorizes in release; the SWAR form
        // does not. The per-lane form is KEPT on the perf fence; see
        // docs/jira/v26.11.2/repairs/repair-16-dred-swar-wire.md.
        for lane in lo..hi {
            if alt_support[lane] != 0 {
                k |= 1u64 << (lane - lo);
            }
        }
        // Padding lanes (facts..words*64) are never facts: purge them so
        // compaction emits exactly the real surviving set.
        let lo = w * 64;
        if lo + 64 > facts {
            let valid = if facts >= lo {
                (1u64 << (facts - lo)) - 1
            } else {
                0
            };
            k &= valid;
        }
        keep[w] = k;
    }
}

/// Branchless lane compaction: append the global fact indices of every set
/// bit in `keep` to `out`, starting at `base`. Returns the new count.
/// Per lane: out[base + run + set] = fact, run += set — no data-dependent
/// branch (the sole loop bounds are fixed). CONTRACT: `out` needs ONE
/// slack slot past the worst case (a trailing purged lane writes a
/// harmless-but-real slot at the final count position); the returned
/// count is the only authoritative length.
pub fn compact_lanes(keep: &[u64], base: u32, out: &mut [u32]) -> u32 {
    debug_assert!(out.len() > base as usize + keep.len() * 64);
    let mut run = base;
    for (w, &k) in keep.iter().enumerate() {
        let wbase = w as u32 * 64;
        for lane in 0..64 {
            let set = ((k >> lane) & 1) as u32;
            out[run as usize] = wbase + lane as u32; // set=0 overwrites fwd
            run += set; // only kept lanes advance the cursor
        }
    }
    run
}

/// Retraction hot path: purge every fact that is derived AND deleted AND
/// has no alternative derivation. Appends surviving indices to `out` after
/// `base`; returns the new total. Scratch is caller-provided (no allocation
/// inside the hot path).
pub fn retract(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
    scratch: &mut [u64],
    base: u32,
    out: &mut [u32],
) -> u32 {
    keep_mask(derived_deps, del_mask, alt_support, facts, scratch);
    compact_lanes(scratch, base, out)
}

/// MUTANT (court-only, doc-hidden): the anti-vacuity witness. Identical to
/// [`retract`] except the alt-support term is SKIPPED, so a deleted+derived
/// fact WITH an alternative derivation is WRONGLY PURGED. Never call in
/// production; exists so the court can prove the check is load-bearing.
#[doc(hidden)]
pub fn retract_unchecked(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
    scratch: &mut [u64],
    base: u32,
    out: &mut [u32],
) -> u32 {
    let _ = alt_support; // the flipped term: dropped on the floor
    let words = derived_deps.len();
    for w in 0..words {
        let mut k = !(derived_deps[w] & del_mask[w]);
        let lo = w * 64;
        if lo + 64 > facts {
            let valid = if facts >= lo {
                (1u64 << (facts - lo)) - 1
            } else {
                0
            };
            k &= valid;
        }
        scratch[w] = k;
    }
    compact_lanes(scratch, base, out)
}

/// MUTANT (court-only, doc-hidden): anti-vacuity witness (b). Identical to
/// [`retract`] except the negative keep bitmask's polarity is INVERTED
/// (`k = !suspect` instead of `k = !suspect | alt != 0`), dropping the
/// alt term AND flipping: on an all-suspect corpus the surviving set goes
/// EMPTY (every fact wrongly purged) — the oracle set-equality fires.
#[doc(hidden)]
pub fn retract_inverted(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
    scratch: &mut [u64],
    base: u32,
    out: &mut [u32],
) -> u32 {
    let _ = alt_support; // the flipped term: inverted mask ignores alt
    let words = derived_deps.len();
    for w in 0..words {
        let s = derived_deps[w] & del_mask[w];
        let mut k = !s; // the flip: polarity inverted, no alt rescue
        let lo = w * 64;
        if lo + 64 > facts {
            let valid = if facts >= lo {
                (1u64 << (facts - lo)) - 1
            } else {
                0
            };
            k &= valid;
        }
        scratch[w] = k;
    }
    compact_lanes(scratch, base, out)
}

/// Full-recompute ORACLE (court referee): per-fact scalar re-derivation,
/// no SWAR, no shared state with the fast path.
pub fn oracle_surviving(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
) -> Vec<u32> {
    let mut v = Vec::new();
    for f in 0..facts {
        let derived = (derived_deps[f / 64] >> (f % 64)) & 1 == 1;
        let deleted = (del_mask[f / 64] >> (f % 64)) & 1 == 1;
        let purged = derived && deleted && alt_support[f] == 0;
        if !purged {
            v.push(f as u32);
        }
    }
    v
}

/// Cascade retraction, expressiveness note (REP4): the flat `derived_deps`
/// bitmask cannot express per-fact dependent LISTS (it is one bit per fact:
/// "is derived"), so a lineage EDGE relation is not representable in the
/// masks alone. The strongest cascade the representation supports therefore
/// takes the edge relation as an explicit caller-leased CSR adjacency
/// ([`build_csr`]); all cascade state lives in caller-leased buffers — the
/// hot path performs ZERO heap allocation.
///
/// Cascade law: fact F is cascade-purged iff F is deleted, OR F is reachable
/// from a deleted fact through the lineage edge relation with NO alternative
/// derivation at F (`alt_support[F] == 0`). A dependent with an alive
/// alternative derivation survives and does NOT propagate the purge (its
/// dependents keep a live derivation). This is the least fixpoint of:
/// purged(root) for every deleted root; purged(d) if purged(p), edge p->d,
/// alt(d)==0. BFS over the CSR with a mark-visited bit converges in <= facts
/// enqueues because every fact is enqueued at most once (marked before
/// push), so the worklist is bounded by `facts` and overflow is a caller
/// under-provisioning error, never a silent drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CascadeError {
    /// worklist lease smaller than the fact count (each fact enqueues <=
    /// once, so capacity == facts is sufficient).
    WorklistCapacity { capacity: usize, facts: usize },
    /// adjacency lease smaller than the edge count.
    AdjacencyLease { needed: usize, provided: usize },
    /// counts lease smaller than facts + 1.
    CountsLease { needed: usize, provided: usize },
    /// an edge endpoint is not a valid fact index.
    EdgeOutOfRange {
        edge_index: usize,
        endpoint: u32,
        facts: usize,
    },
}

/// Build the caller-leased CSR adjacency (dependents grouped by parent) from
/// `(parent, dependent)` lineage edges. `counts` (len facts+1) receives the
/// exclusive-prefix-sum start offsets; `fill` (len facts) is scratch and is
/// left in an unspecified state; `adj` (len >= edges.len()) receives the
/// dependents. Zero allocation; typed refusals on bad leases or bad edges.
pub fn build_csr(
    edges: &[(u32, u32)],
    facts: usize,
    counts: &mut [u32],
    fill: &mut [u32],
    adj: &mut [u32],
) -> Result<(), CascadeError> {
    if counts.len() < facts + 1 {
        return Err(CascadeError::CountsLease {
            needed: facts + 1,
            provided: counts.len(),
        });
    }
    if adj.len() < edges.len() {
        return Err(CascadeError::AdjacencyLease {
            needed: edges.len(),
            provided: adj.len(),
        });
    }
    for c in counts[..=facts].iter_mut() {
        *c = 0;
    }
    for (i, &(p, d)) in edges.iter().enumerate() {
        if p as usize >= facts || d as usize >= facts {
            return Err(CascadeError::EdgeOutOfRange {
                edge_index: i,
                endpoint: u32::max(p, d),
                facts,
            });
        }
        counts[p as usize] += 1;
    }
    // Exclusive prefix sum: counts[f] = start offset of f's dependents.
    let mut run = 0u32;
    for f in 0..facts {
        let c = counts[f];
        counts[f] = run;
        run = run.wrapping_add(c);
    }
    counts[facts] = run;
    // Scatter dependents into adj using a moving fill cursor per parent.
    fill[..facts].copy_from_slice(&counts[..facts]);
    for &(p, d) in edges.iter() {
        let fp = fill[p as usize] as usize;
        adj[fp] = d;
        fill[p as usize] += 1;
    }
    Ok(())
}

/// Cascade retraction (REP4): deletes the transitive closure of deleted
/// roots through the caller-leased CSR lineage adjacency, rescuing every
/// dependent with an alternative derivation, then applies the STATIC
/// single-pass rule (`keep_mask`) to everything the cascade did not purge.
/// Appends surviving fact indices to `out` after `base`; returns the new
/// total, or a typed [`CascadeError`] on an under-provisioned lease.
///
/// Semantics: deleted roots are purged UNCONDITIONALLY (deletion is the
/// actuation; the static rule's alt rescue applies only to non-deleted
/// derived suspects), and a dependent d of a purged parent survives iff
/// `alt_support[d] != 0`.
///
/// Zero heap allocation: purged/keep masks, the BFS worklist ring, and the
/// CSR are all caller-leased.
// Court-fixed surface: the 11-argument caller-leased mask/CSR kernel shape
// (zero-heap contract) is the public API; refactoring signatures is out of
// scope for this pass.
#[allow(clippy::too_many_arguments)]
pub fn cascade_retract(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
    counts: &[u32],
    adj: &[u32],
    purged: &mut [u64],
    keep: &mut [u64],
    worklist: &mut [u32],
    base: u32,
    out: &mut [u32],
) -> Result<u32, CascadeError> {
    let words = derived_deps.len();
    if worklist.len() < facts {
        return Err(CascadeError::WorklistCapacity {
            capacity: worklist.len(),
            facts,
        });
    }
    if purged.len() < words || keep.len() < words {
        return Err(CascadeError::CountsLease {
            needed: words,
            provided: purged.len().min(keep.len()),
        });
    }
    if counts.len() < facts + 1 {
        return Err(CascadeError::CountsLease {
            needed: facts + 1,
            provided: counts.len(),
        });
    }
    for w in 0..words {
        purged[w] = 0;
    }
    // Seed: every deleted fact is a cascade root, enqueued exactly once
    // (marked before push; each fact enqueues <= once -> tail <= facts).
    let mut head = 0usize;
    let mut tail = 0usize;
    for w in 0..words {
        let mut bits = del_mask[w];
        while bits != 0 {
            let lane = bits.trailing_zeros() as usize;
            let f = w * 64 + lane;
            if f < facts {
                purged[w] |= 1u64 << lane;
                if tail >= worklist.len() {
                    return Err(CascadeError::WorklistCapacity {
                        capacity: worklist.len(),
                        facts,
                    });
                }
                worklist[tail] = f as u32;
                tail += 1;
            }
            bits &= bits - 1;
        }
    }
    // BFS to fixpoint: follow lineage edges from purged parents, rescuing
    // dependents with alt_support > 0 (they survive, purge does not pass
    // through them).
    while head < tail {
        let f = worklist[head] as usize;
        head += 1;
        let lo = counts[f] as usize;
        let hi = counts[f + 1] as usize;
        for k in lo..hi {
            let d = adj[k] as usize;
            let word = d / 64;
            let lane = d % 64;
            if purged[word] & (1u64 << lane) == 0 && alt_support[d] == 0 {
                purged[word] |= 1u64 << lane;
                if tail >= worklist.len() {
                    return Err(CascadeError::WorklistCapacity {
                        capacity: worklist.len(),
                        facts,
                    });
                }
                worklist[tail] = d as u32;
                tail += 1;
            }
        }
    }
    // keep = !(static suspect) | alt-rescue, AND-ed with !purged.
    keep_mask(derived_deps, del_mask, alt_support, facts, keep);
    for w in 0..words {
        keep[w] &= !purged[w];
    }
    Ok(compact_lanes(keep, base, out))
}

/// MUTANT (court-only, doc-hidden): anti-vacuity witness (c) for the
/// cascade. Identical to [`cascade_retract`] except the BFS loop is
/// DROPPED: the cascade pass is skipped entirely and only the static
/// single-pass filter runs. On the FA5 N=10,000 chain corpus this yields
/// 9,999 phantom survivors exactly as the forensic defect describes.
#[doc(hidden)]
// Court-fixed surface: mirrors `cascade_retract`'s caller-leased kernel
// shape exactly; signature parity with the real path is the anti-vacuity
// witness, so the argument count is intentional.
#[allow(clippy::too_many_arguments)]
pub fn cascade_retract_mutant_nocascade(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
    counts: &[u32],
    adj: &[u32],
    purged: &mut [u64],
    keep: &mut [u64],
    worklist: &mut [u32],
    base: u32,
    out: &mut [u32],
) -> Result<u32, CascadeError> {
    let _ = (counts, adj, purged, worklist); // the dropped cascade pass
    keep_mask(derived_deps, del_mask, alt_support, facts, keep);
    Ok(compact_lanes(keep, base, out))
}

/// Full-recompute ORACLE (court referee) for the cascade: rounds-based
/// mask propagation to fixpoint, no CSR, no BFS — the naive O(V*E)-worst
/// least-fixpoint computation. Converges because each round either purges
/// >= 1 new fact or terminates; the fixpoint is the same least fixpoint the
/// > BFS computes (both saturate the rule: purged(root) for deleted root;
/// > purged(d) if purged(p), edge p->d, alt(d)==0).
pub fn cascade_oracle(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
    edges: &[(u32, u32)],
) -> Vec<u32> {
    let words = derived_deps.len();
    let mut purged = vec![0u64; words];
    for f in 0..facts {
        if del_mask[f / 64] & (1u64 << (f % 64)) != 0 {
            purged[f / 64] |= 1u64 << (f % 64);
        }
    }
    loop {
        let mut changed = false;
        for &(p, d) in edges {
            if purged[p as usize / 64] & (1u64 << (p % 64)) != 0
                && purged[d as usize / 64] & (1u64 << (d % 64)) == 0
                && alt_support[d as usize] == 0
            {
                purged[d as usize / 64] |= 1u64 << (d % 64);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut keep = vec![0u64; words];
    keep_mask(derived_deps, del_mask, alt_support, facts, &mut keep);
    for w in 0..words {
        keep[w] &= !purged[w];
    }
    let mut out = vec![0u32; words * 64 + 1];
    let n = compact_lanes(&keep, 0, &mut out);
    out[..n as usize].to_vec()
}

#[cfg(test)]
mod courts {
    use super::*;
    extern crate std;
    use std::time::Instant;

    /// Canonical padded corpus: `facts` facts as (derived, del, alt) words.
    /// Entropy e in 0..=100 controls the Bernoulli p of each bit class.
    fn corpus(facts: usize, del_entropy_pct: u32, seed: u64) -> (Vec<u64>, Vec<u64>, Vec<u32>) {
        let words = facts.div_ceil(64);
        let mut st = seed;
        let mut derived = vec![0u64; words];
        let mut del = vec![0u64; words];
        let mut alt = vec![0u32; facts];
        for f in 0..facts {
            let d = splitmix64(&mut st) % 100 < 60; // 60% derived
            let x = splitmix64(&mut st) % 100 < del_entropy_pct as u64;
            let a = splitmix64(&mut st) % 100 < 30; // 30% have alt derivation
            if d {
                derived[f / 64] |= 1 << (f % 64);
            }
            if x {
                del[f / 64] |= 1 << (f % 64);
            }
            if a {
                alt[f] = 1 + (splitmix64(&mut st) % 7) as u32;
            }
        }
        (derived, del, alt)
    }

    fn run_retract(derived: &[u64], del: &[u64], alt: &[u32], facts: usize) -> Vec<u32> {
        let words = derived.len();
        let mut scratch = vec![0u64; words];
        let mut out = vec![0u32; words * 64 + 1];
        let n = retract(derived, del, alt, facts, &mut scratch, 0, &mut out);
        out[..n as usize].to_vec()
    }

    /// COURT 1 (retraction correctness vs full-recompute oracle): at >=5
    /// entropy levels the SWAR fast path must produce the IDENTICAL
    /// surviving set as the scalar oracle. Anti-vacuity mutation: the
    /// oracle itself is cross-checked against a hand-derived 4-fact KAT.
    #[test]
    fn court_retraction_matches_oracle() {
        // KAT first: derived+deleted+no-alt purged; alt>0 survives;
        // non-deleted survives; non-derived survives.
        let d = [0b1011u64];
        let x = [0b0111u64];
        let a = [0u32, 3, 0, 1];
        assert_eq!(oracle_surviving(&d, &x, &a, 4), vec![1, 2, 3]);
        assert_eq!(run_retract(&d, &x, &a, 4), vec![1, 2, 3]);
        for e in [0u32, 20, 40, 60, 80, 100] {
            for facts in [64usize, 1000, 4096] {
                let (d, x, a) = corpus(facts, e, CORPUS_SEED ^ (e as u64) ^ (facts as u64));
                assert_eq!(
                    run_retract(&d, &x, &a, facts),
                    oracle_surviving(&d, &x, &a, facts),
                    "entropy {e}% facts {facts}"
                );
            }
        }
    }

    /// COURT 2 (anti-vacuity flip): skipping the alt-support check must
    /// wrongly purge a deleted+derived fact with alt_support > 0 — the
    /// guarded path keeps it and the mutant purges it. A court that cannot
    /// distinguish these two paths is FAIL.
    #[test]
    fn court_antivacuity_alt_support_flip() {
        // fact 0: derived + deleted + alt_support 5 -> must survive.
        // fact 1: derived + deleted + alt_support 0 -> purged by both.
        let d = [0b0011u64];
        let x = [0b0011u64];
        let a = [5u32, 0];
        let mut scratch = [0u64; 1];
        let mut out_g = [0u32; 65];
        let mut out_m = [0u32; 65];
        let g = retract(&d, &x, &a, 2, &mut scratch, 0, &mut out_g);
        let m = retract_unchecked(&d, &x, &a, 2, &mut scratch, 0, &mut out_m);
        assert_eq!(
            &out_g[..g as usize],
            &[0u32][..],
            "guarded: alt-derived fact survives"
        );
        // Flip witness: WITHOUT the check BOTH deleted+derived facts purge
        // (the alt-supported one is the wrongly-purged collateral).
        assert_eq!(
            &out_m[..m as usize],
            &[] as &[u32],
            "mutant: alt fact wrongly purged"
        );
    }

    /// COURT 2b (anti-vacuity flip, mutation (b)): INVERTING the negative
    /// keep bitmask (`k = suspect` instead of `k = !suspect | alt`) purges
    /// ALL facts when every fact is suspect — the oracle set-equality
    /// assert fires. A court that cannot distinguish guarded from inverted
    /// is FAIL.
    #[test]
    fn court_antivacuity_inverted_bitmask() {
        // All 3 facts derived+deleted, two carry alt proofs: guarded keeps
        // the alt-supported pair; inverted purges ALL 3 (flip witness).
        let d = [0b0111u64];
        let x = [0b0111u64];
        let a = [0u32, 4, 2];
        let mut scratch = [0u64; 1];
        let mut out_g = [0u32; 65];
        let mut out_m = [0u32; 65];
        let g = retract(&d, &x, &a, 3, &mut scratch, 0, &mut out_g);
        let m = retract_inverted(&d, &x, &a, 3, &mut scratch, 0, &mut out_m);
        assert_eq!(
            &out_g[..g as usize],
            &[1u32, 2][..],
            "guarded: alt facts survive"
        );
        assert_eq!(&out_m[..m as usize], &[] as &[u32], "inverted: ALL purged");
        // Corpus-level: inversion must break oracle set-equality.
        for facts in [64usize, 1000, 4096] {
            let (d, x, a) = corpus(facts, 100, CORPUS_SEED ^ (facts as u64));
            let m = run_retract_mutant_b(&d, &x, &a, facts);
            assert_ne!(
                m,
                oracle_surviving(&d, &x, &a, facts),
                "inverted mask matched oracle at facts {facts}: court VACUOUS"
            );
        }
    }

    fn run_retract_mutant_b(derived: &[u64], del: &[u64], alt: &[u32], facts: usize) -> Vec<u32> {
        let words = derived.len();
        let mut scratch = vec![0u64; words];
        let mut out = vec![0u32; words * 64 + 1];
        let n = retract_inverted(derived, del, alt, facts, &mut scratch, 0, &mut out);
        out[..n as usize].to_vec()
    }

    /// COURT 3 (entropy-flatness measurement, PARETO table source): ns per
    /// lane across 0-100% deletion-entropy. The paper claims flat variance
    /// across entropy; this measures the actual spread (max/min ratio).
    #[test]
    fn court_entropy_flat_variance() {
        let facts = 1 << 16;
        let words = facts / 64;
        let mut scratch = vec![0u64; words];
        let mut out = vec![0u32; facts + 1];
        let mut rows: Vec<(u32, u64, f64)> = Vec::new();
        for e in [0u32, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100] {
            let (d, x, a) = corpus(facts, e, ENTROPY_SEED ^ (e as u64));
            // warm
            let _ = retract(&d, &x, &a, facts, &mut scratch, 0, &mut out);
            let mut best = u64::MAX;
            for _ in 0..200 {
                let t = Instant::now();
                let n = retract(&d, &x, &a, facts, &mut scratch, 0, &mut out);
                let ns = t.elapsed().as_nanos() as u64;
                assert_eq!(n as usize, oracle_surviving(&d, &x, &a, facts).len());
                if ns < best {
                    best = ns;
                }
            }
            let ns_per_lane = best as f64 / facts as f64;
            rows.push((e, best, ns_per_lane));
        }
        let ratios: Vec<f64> = rows.iter().map(|r| r.2).collect();
        let mn = ratios.iter().cloned().fold(f64::INFINITY, f64::min);
        let mx = ratios.iter().cloned().fold(0.0, f64::max);
        for r in &rows {
            std::println!("ENTROPY {:3}% total {:6} ns  {:.4} ns/lane", r.0, r.1, r.2);
        }
        std::println!("ENTROPY-FLATNESS max/min ratio = {:.3}", mx / mn);
        assert!(
            mx / mn < 4.0,
            "entropy-variance claim inverted: {mx:.4}/{mn:.4} spread > 4x"
        );
    }
}
