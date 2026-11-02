//! closure — semi-naive transitive closure on arrow columns (v26.11.2 lane p1-closure).
//!
//! Rule: R(x,z) <- R(x,y), B(y,z) over edge columns (x,y) and (y,z),
//! u32 node ids, columnar in / columnar out. Terminate when the round
//! emits zero new pairs. Dedup via a packed-pair seen-set: the planned
//! bit-plane seen-set lane (p1-bitplane) had NOT landed at authoring
//! time (no `p1-bitplane-*.md` receipt in `docs/jira/v26.11.2/waves/`,
//! witnessed this session), so the seen-set is a packed `u64` hash set
//! keyed `(x << 32) | z`. An N^2 bit-plane is infeasible at bench size
//! N=1048576 (4 TiB), so the packed hash seen-set is the design, not a
//! fallback. Splitmix64 seeded from the canonical corpus seed
//! `0xA20C_0FFE_E5A7_E000` (never wall-clock).
//!
//! Semi-naive discipline: each round joins ONLY the previous delta with
//! the base relation B (the second atom). Round k emits pairs whose
//! first derivation is a (k-1)-hop pair plus one base edge, so the
//! fixpoint is complete: every derivable pair is emitted exactly once,
//! at its shortest-path round. Termination = delta empty.

/// Canonical corpus seed (frozen pin; never conflated with the
/// splitmix64 multiplier 0xBF58_476D_1CE4_E5B9 which lives in
/// dred::splitmix64 / cgap-parquet bitslice.rs:687-699).
pub const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

/// A fully packed (x << 32) | z pair key.
#[inline]
pub fn pair_key(x: u32, z: u32) -> u64 {
    ((x as u64) << 32) | (z as u64)
}

/// Packed-pair seen-set. `insert` returns true iff the pair was new.
pub struct SeenSet {
    words: std::collections::HashSet<u64>,
}

impl SeenSet {
    pub fn new() -> Self {
        Self {
            words: std::collections::HashSet::new(),
        }
    }

    #[inline]
    pub fn insert(&mut self, key: u64) -> bool {
        self.words.insert(key)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

impl Default for SeenSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Index the base relation once: y -> (z...). Built once, reused every
/// round (columnar adjacency; no per-round full re-scan).
struct Adjacency {
    /// sorted base (x=y here) column, parallel to `targets`
    sources: Vec<u32>,
    targets: Vec<u32>,
}

impl Adjacency {
    fn build(edges_x: &[u32], edges_z: &[u32]) -> Self {
        let mut idx: Vec<(u32, u32)> = Vec::with_capacity(edges_x.len());
        for i in 0..edges_x.len() {
            idx.push((edges_x[i], edges_z[i]));
        }
        idx.sort_unstable();
        let (sources, targets): (Vec<u32>, Vec<u32>) = idx.into_iter().unzip();
        Self { sources, targets }
    }

    /// Stream all z with edges_x[j] == y (via binary search range).
    fn stream(&self, y: u32) -> impl Iterator<Item = u32> + '_ {
        let start = self.sources.partition_point(|&s| s < y);
        let end = self.sources.partition_point(|&s| s <= y);
        self.targets[start..end].iter().copied()
    }
}

/// Semi-naive transitive closure over base edge columns.
/// Returns (x column, z column, rounds executed). Includes the base
/// edges themselves (R is seeded with B).
pub fn semi_naive_closure(edges_x: &[u32], edges_z: &[u32]) -> (Vec<u32>, Vec<u32>, usize) {
    debug_assert_eq!(edges_x.len(), edges_z.len());
    let adj = Adjacency::build(edges_x, edges_z);
    let mut seen = SeenSet::new();
    let mut all_x: Vec<u32> = Vec::with_capacity(edges_x.len());
    let mut all_z: Vec<u32> = Vec::with_capacity(edges_z.len());

    // Seed: base edges are the round-0 delta.
    let mut delta_x: Vec<u32> = Vec::with_capacity(edges_x.len());
    let mut delta_z: Vec<u32> = Vec::with_capacity(edges_z.len());
    for i in 0..edges_x.len() {
        if seen.insert(pair_key(edges_x[i], edges_z[i])) {
            all_x.push(edges_x[i]);
            all_z.push(edges_z[i]);
            delta_x.push(edges_x[i]);
            delta_z.push(edges_z[i]);
        }
    }

    // join(dR, B): stream dR x B, emit unseen (x,z) into the builder.
    // loop exits when the delta is empty (emitted == 0).
    let mut rounds = 0usize;
    while !delta_x.is_empty() {
        rounds += 1;
        let mut next_x: Vec<u32> = Vec::new();
        let mut next_z: Vec<u32> = Vec::new();
        for i in 0..delta_x.len() {
            let x = delta_x[i];
            for z in adj.stream(delta_z[i]) {
                if seen.insert(pair_key(x, z)) {
                    all_x.push(x);
                    all_z.push(z);
                    next_x.push(x);
                    next_z.push(z);
                }
                // Semi-naive: continue streaming, dedup is the seen-set.
            }
        }
        delta_x = next_x;
        delta_z = next_z;
    }
    (all_x, all_z, rounds)
}

/// Naive BTreeSet fixpoint oracle (court collaborator, real). Same
/// semi-naive-free fixpoint: full known-set x base-adjacency per round.
pub fn naive_closure(edges_x: &[u32], edges_z: &[u32]) -> std::collections::BTreeSet<u64> {
    let mut adj_src: Vec<(u32, u32)> = (0..edges_x.len())
        .map(|i| (edges_x[i], edges_z[i]))
        .collect();
    adj_src.sort_unstable();
    let srcs: Vec<u32> = adj_src.iter().map(|p| p.0).collect();
    let dsts: Vec<u32> = adj_src.iter().map(|p| p.1).collect();
    let stream = |y: u32, out: &mut Vec<u32>| {
        let a = srcs.partition_point(|&s| s < y);
        let b = srcs.partition_point(|&s| s <= y);
        out.extend_from_slice(&dsts[a..b]);
    };
    let mut known: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    for i in 0..edges_x.len() {
        known.insert(pair_key(edges_x[i], edges_z[i]));
    }
    let mut changed = true;
    while changed {
        changed = false;
        let snapshot: Vec<u64> = known.iter().copied().collect();
        for k in snapshot {
            let x = (k >> 32) as u32;
            let y = k as u32;
            let mut zs: Vec<u32> = Vec::new();
            stream(y, &mut zs);
            for z in zs {
                if known.insert(pair_key(x, z)) {
                    changed = true;
                }
            }
        }
    }
    known
}

/// Court-only mutant (anti-vacuity, p1-closure court 3): the rule with
/// the SECOND ATOM (B) dropped — R(x,z) <- R(x,y) alone derives nothing,
/// so the closure collapses to the deduplicated base. The court asserts
/// the full closure count differs from this mutant's count; if it does
/// not, the join contributed zero pairs and the court is VACUOUS.
pub fn mutant_base_only_count(edges_x: &[u32], edges_z: &[u32]) -> usize {
    let mut seen = SeenSet::new();
    let mut n = 0usize;
    for i in 0..edges_x.len() {
        if seen.insert(pair_key(edges_x[i], edges_z[i])) {
            n += 1;
        }
    }
    n
}

/// Bitplane-parity variant (p1-closure court 2): the same semi-naive
/// engine loop with the dedup collaborator swapped to the landed
/// `bitplane_seen::BitPlaneSeenSet` (fixed capacity). Returns the
/// closure pair count, or `Err(SeenFull)` loudly on capacity overflow
/// (never a silent drop). Divergence (disclosed): the engine's primary
/// packed-hash seen-set drives the bench path because
/// `BitPlaneSeenSet::contains` is O(slots) — infeasible at 1M nodes.
pub fn semi_naive_closure_bitplane_count<const SLOTS: usize>(
    edges_x: &[u32],
    edges_z: &[u32],
) -> Result<usize, crate::bitplane_seen::SeenFull> {
    debug_assert_eq!(edges_x.len(), edges_z.len());
    let mut seen = crate::bitplane_seen::BitPlaneSeenSet::<SLOTS>::new();
    let mut all_x: Vec<u32> = Vec::new();
    let mut delta_x: Vec<u32> = Vec::new();
    let mut delta_z: Vec<u32> = Vec::new();
    for i in 0..edges_x.len() {
        if seen.insert(edges_x[i] as u64, edges_z[i] as u64)? {
            all_x.push(edges_x[i]);
            delta_x.push(edges_x[i]);
            delta_z.push(edges_z[i]);
        }
    }
    // Adjacency over the base relation (sorted, binary-search streamed).
    let mut idx: Vec<(u32, u32)> = (0..edges_x.len())
        .map(|i| (edges_x[i], edges_z[i]))
        .collect();
    idx.sort_unstable();
    let srcs: Vec<u32> = idx.iter().map(|p| p.0).collect();
    let dsts: Vec<u32> = idx.iter().map(|p| p.1).collect();
    while !delta_x.is_empty() {
        let mut nx: Vec<u32> = Vec::new();
        let mut nz: Vec<u32> = Vec::new();
        for i in 0..delta_x.len() {
            let x = delta_x[i];
            let y = delta_z[i];
            let a = srcs.partition_point(|&s| s < y);
            let b = srcs.partition_point(|&s| s <= y);
            for z in &dsts[a..b] {
                if seen.insert(x as u64, *z as u64)? {
                    nx.push(x);
                    nz.push(*z);
                }
            }
        }
        delta_x = nx;
        delta_z = nz;
        let _ = &mut all_x;
    }
    Ok(seen.len())
}
