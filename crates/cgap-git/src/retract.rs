//! retract — lane GIT-11: revert(commit-delta) -> cascade delete + re-derive
//! over the coordinate frame (v26.11.2 starDF).
//!
//! REALITY FENCE (disclosed): the REAL DRed engine is
//! `crates/cgap-storage/src/dred.rs` (B05 receipt,
//! docs/jira/v26.11.2/lanes/lane-b05-cscb-c6.md: measured min 248M facts/s
//! full cascade vs the 800M aspiration; dred.rs:73-76 cites 839-944M
//! facts/s for the keep-mask path ALONE, different corpus/method — units
//! NOT conflated here). This module is a git-frame ADAPTER: it maps a
//! reverted commit delta onto the same least-fixpoint cascade law dred.rs
//! implements, and the court cross-checks against `dred::cascade_oracle`
//! via a `#[path]` include of the real source in the std test crate
//! (disclosed reuse; cgap-git Cargo.toml untouched).
//!
//! Why self-contained (dispatch disclosure): cgap-git is zero-dep; a path
//! dep on cgap-storage needs Cargo.toml (not this lane's file), so the
//! module is alloc-free (caller-leased buffers only) and the court file
//! does the include, leaving the shared `lib.rs` untouched.
//!
//! Law (identical to dred::cascade_retract): fact F is cascade-purged iff
//! F is a deleted root, OR F is reachable from a purged fact through the
//! lineage edge relation with alt_support\[F] == 0 (least fixpoint). A
//! dependent with an alive alternative derivation survives and does NOT
//! propagate the purge.
//!
//! Coordinate frame: facts are linearized chain coordinates (u32). A
//! reverted commit delta is the set of fact coordinates the commit
//! introduced (its delta roots). No wall-clock: corpora come from
//! canonical splitmix64 (multiplier 0xBF58_476D_1CE4_E5B9).

/// Bits per SWAR word / lanes per compaction word.
pub const WORD_BITS: u32 = 64;

/// Canonical corpus seed (corpus constant family 0xA20C_0FFE_E5A7_E000,
/// stepped by splitmix64).
pub const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

/// splitmix64 — canonical deterministic generator. Returns (state, value).
#[inline]
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Typed failure of the retraction driver (exit-1-class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetractError {
    /// worklist lease smaller than the fact count.
    WorklistCapacity { capacity: usize, facts: usize },
    /// adjacency lease smaller than the edge count.
    AdjacencyLease { needed: usize, provided: usize },
    /// counts lease smaller than facts + 1.
    CountsLease { needed: usize, provided: usize },
    /// an edge endpoint is not a valid fact coordinate.
    EdgeOutOfRange {
        edge_index: usize,
        endpoint: u32,
        facts: usize,
    },
    /// ANTI-PHANTOM INTEGRITY FAILURE: post-retraction, at least one
    /// surviving derived fact has NO surviving derivation parent (an
    /// orphan derivation). Raised by the integrity driver when the
    /// cascade was suppressed or corrupted; never by the correct path.
    OrphanDerivation { orphans: u32, first: u32 },
}

/// Build the caller-leased CSR adjacency (dependents grouped by parent)
/// from `(parent, dependent)` lineage edges. `counts` (len facts+1)
/// receives exclusive-prefix-sum start offsets; `fill` (len facts) is
/// scratch; `adj` (len >= edges.len()) receives dependents. Zero
/// allocation; typed refusals on bad leases or bad edges.
pub fn build_csr(
    edges: &[(u32, u32)],
    facts: usize,
    counts: &mut [u32],
    fill: &mut [u32],
    adj: &mut [u32],
) -> Result<(), RetractError> {
    if counts.len() < facts + 1 {
        return Err(RetractError::CountsLease {
            needed: facts + 1,
            provided: counts.len(),
        });
    }
    if adj.len() < edges.len() {
        return Err(RetractError::AdjacencyLease {
            needed: edges.len(),
            provided: adj.len(),
        });
    }
    for c in counts[..=facts].iter_mut() {
        *c = 0;
    }
    for (i, &(p, d)) in edges.iter().enumerate() {
        if p as usize >= facts || d as usize >= facts {
            return Err(RetractError::EdgeOutOfRange {
                edge_index: i,
                endpoint: u32::max(p, d),
                facts,
            });
        }
        counts[p as usize] += 1;
    }
    let mut run = 0u32;
    for slot in counts[..facts].iter_mut() {
        let c = *slot;
        *slot = run;
        run = run.wrapping_add(c);
    }
    counts[facts] = run;
    fill[..facts].copy_from_slice(&counts[..facts]);
    for &(p, d) in edges.iter() {
        let fp = fill[p as usize] as usize;
        adj[fp] = d;
        fill[p as usize] += 1;
    }
    Ok(())
}

/// Bit-parallel difference masking for the keep-set: SWAR over u64 lanes.
/// Real scalar-host math (64 logical bit-lanes per word, word-parallel
/// AND/AND-NOT/OR); this is NOT SIMD and no SIMD claim is made — the
/// host is scalar, the lanes are logical.
/// keep\[f] == 1 iff fact f SURVIVES the static pass:
/// keep = !(derived & del) | (alt_support != 0).
pub fn keep_mask_swar(
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
        let suspect = derived_deps[w] & del_mask[w];
        let mut k = !suspect;
        let lo = w * 64;
        let hi = core::cmp::min(lo + 64, alt_support.len());
        for (bit, &support) in alt_support[lo..hi].iter().enumerate() {
            if support != 0 {
                k |= 1u64 << bit;
            }
        }
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

/// Branchless lane compaction (per lane: out[base + run + set] = fact,
/// run += set). CONTRACT: `out` needs ONE slack slot past the worst case;
/// the returned count is the only authoritative length.
pub fn compact_lanes(keep: &[u64], base: u32, out: &mut [u32]) -> u32 {
    debug_assert!(out.len() > base as usize + keep.len() * 64);
    let mut run = base;
    for (w, &k) in keep.iter().enumerate() {
        let wbase = w * 64;
        for lane in 0..64 {
            let set = ((k >> lane) & 1) as u32;
            out[run as usize] = (wbase + lane) as u32;
            run += set;
        }
    }
    run
}

/// ONE cascade step of the BFS fixpoint (factored so the court's mutation
/// twin can suppress exactly one step): every unpurged alt-less dependent
/// of every purged parent in worklist[head..tail] is marked + enqueued.
/// Fail-closed on worklist overflow (typed Err, never a silent drop).
pub fn cascade_step(
    alt_support: &[u32],
    counts: &[u32],
    adj: &[u32],
    purged: &mut [u64],
    worklist: &mut [u32],
    head: usize,
    tail: usize,
) -> Result<usize, RetractError> {
    let mut new_tail = tail;
    for i in head..tail {
        let f = worklist[i] as usize;
        let lo = counts[f] as usize;
        let hi = counts[f + 1] as usize;
        for &d32 in &adj[lo..hi] {
            let d = d32 as usize;
            let word = d / 64;
            let lane = d % 64;
            if purged[word] & (1u64 << lane) == 0 && alt_support[d] == 0 {
                purged[word] |= 1u64 << lane;
                if new_tail >= worklist.len() {
                    return Err(RetractError::WorklistCapacity {
                        capacity: worklist.len(),
                        facts: purged.len() * 64,
                    });
                }
                worklist[new_tail] = d as u32;
                new_tail += 1;
            }
        }
    }
    Ok(new_tail)
}

/// Drive cascade_step to the least fixpoint (bounded: each fact enqueues
/// at most once, so total enqueues <= facts).
fn cascade_steps(
    alt_support: &[u32],
    counts: &[u32],
    adj: &[u32],
    purged: &mut [u64],
    worklist: &mut [u32],
    head: &mut usize,
    tail: &mut usize,
) -> Result<(), RetractError> {
    while *head < *tail {
        let nt = cascade_step(alt_support, counts, adj, purged, worklist, *head, *tail)?;
        *head = *tail;
        *tail = nt;
    }
    Ok(())
}

/// Retraction hot path: cascade-purge the transitive closure of the
/// reverted commit delta's roots through the caller-leased CSR, rescue
/// every dependent with an alternative derivation, apply the SWAR static
/// keep pass to everything the cascade did not purge, and compact the
/// surviving fact coordinates. Appends to `out` after `base`; returns the
/// new total or a typed [`RetractError`] on an under-provisioned lease.
/// Zero heap allocation inside the hot path.
// CSR cascade lease takes 11 slice/scalar parameters by design; do not
// refactor the public hot-path signature.
#[allow(clippy::too_many_arguments)]
pub fn revert_cascade(
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
) -> Result<u32, RetractError> {
    let words = derived_deps.len();
    if worklist.len() < facts {
        return Err(RetractError::WorklistCapacity {
            capacity: worklist.len(),
            facts,
        });
    }
    if purged.len() < words || keep.len() < words {
        return Err(RetractError::CountsLease {
            needed: words,
            provided: purged.len().min(keep.len()),
        });
    }
    if counts.len() < facts + 1 {
        return Err(RetractError::CountsLease {
            needed: facts + 1,
            provided: counts.len(),
        });
    }
    purged[..words].fill(0);
    // Seed: every deleted fact is a cascade root, enqueued exactly once
    // (marked before push; tail <= facts by construction).
    let mut head = 0usize;
    let mut tail = 0usize;
    for w in 0..words {
        let mut bits = del_mask[w];
        while bits != 0 {
            let lane = bits.trailing_zeros() as usize;
            let f = w * 64 + lane;
            if f < facts {
                purged[w] |= 1u64 << lane;
                worklist[tail] = f as u32;
                tail += 1;
            }
            bits &= bits - 1;
        }
    }
    cascade_steps(
        alt_support,
        counts,
        adj,
        purged,
        worklist,
        &mut head,
        &mut tail,
    )?;
    // keep = static SWAR keep pass AND-ed with !purged.
    keep_mask_swar(derived_deps, del_mask, alt_support, facts, keep);
    for w in 0..words {
        keep[w] &= !purged[w];
    }
    Ok(compact_lanes(keep, base, out))
}

/// Map a reverted commit delta onto the coordinate frame: `delta_facts`
/// (the fact coordinates the reverted commit introduced) set bits in
/// `del_mask`. Revert(commit-delta) -> delete-set projection; idempotent
/// bit-OR per coordinate.
pub fn project_delta_roots(delta_facts: &[u32], del_mask: &mut [u64]) {
    for &f in delta_facts {
        del_mask[(f as usize) / 64] |= 1u64 << (f as usize % 64);
    }
}

/// Full-recompute scalar ORACLE (court referee): rounds-based mask
/// propagation to the least fixpoint; no CSR, no BFS, no shared state
/// with the fast path. Same least fixpoint as dred::cascade_oracle.
/// Fills `out` (capacity >= words*64+1) with surviving coordinates and
/// returns the count. Fixed 4096-word arena: facts <= 262,144.
pub fn cascade_oracle_rounds(
    derived_deps: &[u64],
    del_mask: &[u64],
    alt_support: &[u32],
    facts: usize,
    edges: &[(u32, u32)],
    out: &mut [u32],
) -> u32 {
    let words = derived_deps.len();
    let mut arena = [0u64; 4096];
    assert!(
        words <= 4096,
        "oracle arena is a fixed 4096-word lease (facts <= 262,144)"
    );
    let purged = &mut arena[..words];
    for w in purged.iter_mut() {
        *w = 0;
    }
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
    let mut karena = [0u64; 4096];
    let keep = &mut karena[..words];
    keep_mask_swar(derived_deps, del_mask, alt_support, facts, keep);
    for w in 0..words {
        keep[w] &= !purged[w];
    }
    compact_lanes(keep, 0, out)
}

/// ANTI-PHANTOM INTEGRITY (post-retraction): an orphan derivation is a
/// SURVIVING derived fact whose EVERY derivation parent was purged (it
/// survives on nothing). Returns (orphan_count, first_orphan_coord);
/// (0, u32::MAX) is the law. Takes the raw edge list (court corpus sizes;
/// production callers would lease a reverse CSR).
pub fn count_orphans(
    keep: &[u64],
    derived_deps: &[u64],
    alt_support: &[u32],
    edges: &[(u32, u32)],
    facts: usize,
) -> (u32, u32) {
    let alive = |f: usize| keep[f / 64] & (1u64 << (f % 64)) != 0;
    let mut orphans = 0u32;
    let mut first = u32::MAX;
    for f in 0..facts {
        // An alt-supported survivor is LEGAL (rescued by its alternative
        // derivation), never an orphan; only an alt-less survivor with no
        // alive parent survives on nothing.
        if !alive(f) || derived_deps[f / 64] & (1u64 << (f % 64)) == 0 || alt_support[f] != 0 {
            continue;
        }
        let mut any_parent_alive = false;
        for &(p, d) in edges {
            if d as usize == f && alive(p as usize) {
                any_parent_alive = true;
                break;
            }
        }
        if !any_parent_alive {
            orphans += 1;
            if first == u32::MAX {
                first = f as u32;
            }
        }
    }
    (orphans, first)
}

/// MUTANT (court-only, doc-hidden): anti-vacuity witness for GIT-11.
/// Identical to [`revert_cascade`] except ONE cascade step is SUPPRESSED:
/// the fixpoint loop runs only its FIRST round. On a chain corpus this
/// leaves transitive dependents unpurged — injected orphan derivations
/// the integrity driver must report as typed
/// [`RetractError::OrphanDerivation`].
// CSR cascade lease takes 11 slice/scalar parameters by design; do not
// refactor the public hot-path signature.
#[allow(clippy::too_many_arguments)]
pub fn revert_cascade_mutant_one_step(
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
) -> Result<u32, RetractError> {
    let words = derived_deps.len();
    if worklist.len() < facts {
        return Err(RetractError::WorklistCapacity {
            capacity: worklist.len(),
            facts,
        });
    }
    purged[..words].fill(0);
    let mut tail = 0usize;
    for w in 0..words {
        let mut bits = del_mask[w];
        while bits != 0 {
            let lane = bits.trailing_zeros() as usize;
            let f = w * 64 + lane;
            if f < facts {
                purged[w] |= 1u64 << lane;
                worklist[tail] = f as u32;
                tail += 1;
            }
            bits &= bits - 1;
        }
    }
    // THE SUPPRESSED STEP: exactly one cascade round, no fixpoint loop.
    let _ = cascade_step(alt_support, counts, adj, purged, worklist, 0, tail)?;
    keep_mask_swar(derived_deps, del_mask, alt_support, facts, keep);
    for w in 0..words {
        keep[w] &= !purged[w];
    }
    Ok(compact_lanes(keep, base, out))
}
