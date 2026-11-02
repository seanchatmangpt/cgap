# Lane H8 — BN3 zero-alloc fixpoint verification + closure alloc scope

Date: 2026-11-02. Lane contract: read-only + probe runs; only write is this
receipt. /tmp scratch only (`/tmp/h8-probe`, `/tmp/h8-target*`), counting
`GlobalAlloc`, no source edits, no `Process.sleep`.

## 1. Grep evidence — W6-BFX to_vec sweep

`grep -n "to_vec" crates/cgap-bn3/src/unify.rs` — **0 matches** (grep exit 1).
W6-BFX markers present: `unify.rs:414` (`Borrowed/owned row source ... W6-BFX`),
`:441` (SplitStore), `:1505` ("the caller's columns are NEVER copied to the
heap"), `:1527` (one JoinProbeScratch hoisted to driver scope). The borrowed
prefix + owned delta tail is in place; no `.to_vec()` copies remain in the
fixpoint driver.

## 2. Counting-allocator probe (FA4 class)

Probe crate `/tmp/h8-probe` (path dep on `crates/cgap-bn3`), counting
`GlobalAlloc` wrapping `System`, counting alloc+realloc calls. Corpus:
directed ring + transitive-closure rule (`x P y, y P z => x P z`); the
**store axis** pads the store with predicate-99 self-loops the rule cannot
match, so delta/rounds are constant while store scales 1x→101x.

Two runs (release, 2026-11-02):

- First run (ring axis only, budget 64): `ring_n=3 rounds=3 allocs=137`;
  `ring_n=30 rounds=6 allocs=596`; `ring_n=300 rounds=10 allocs=15901`.
- Final run:

```
-- delta axis (store=delta, rounds grow) --
ring_n=3 rounds=3 emitted=3 allocs=137
ring_n=30 rounds=6 emitted=6 allocs=596
ring_n=300 rounds=10 emitted=10 allocs=15,901
-- store axis (delta constant: 3-cycle, pad facts pred-99) --
pad=0      store=3      rounds=3 emitted=3 allocs=137
pad=1000   store=1003   rounds=3 emitted=3 allocs=231  (+94)
pad=10000  store=10003  rounds=3 emitted=3 allocs=1,058  (+827)
pad=100000 store=100003 rounds=3 emitted=3 allocs=9,307 (+8,249)
store_ratio=101x alloc_ratio=67.93x (a0=137 a100k=9307)
```

**The MUST-FLIP falsifier fired in the direction the W6-BFX claim did not
anticipate.** The asserted strong form — "allocs scale with delta, not store"
— was REFUTED by the probe: `alloc_ratio 67.93x` for a `101x` store at
constant delta, exit 101 (assert "MUST-FLIP falsifier: allocs scaled with
STORE, not delta"). This is not a probe bug — counter sanity:

```
collected=100000 allocs=9095
SANITY: PASS   (100k BTreeSet<(u64,u64,u64)> inserts = 9,095 allocs;
                B-tree fanout ~11 elems/node -> ~9.1k nodes, matches)
```

The 100k-fact seen-set seed allocates ~9.1k B-tree nodes — exactly the
+8,249 delta from pad 10k→100k. Root cause (unify.rs:1518-1521):

```rust
let mut seen: BTreeSet<(u64,u64,u64)> = (0..cols.len())
    .map(|i| (cols.s[i], cols.p[i], cols.o[i])).collect();
```

The eval path itself is store-independent (no per-round store copy: at pad
100k, 3 rounds cost 9,307 allocs total, where the retired full-store-copy
arm would allocate store×rounds row buffers), but the seen-set seed is
**O(store/11) B-tree node allocations, once per fixpoint run**.

## Verdict (honest)

1. **NOT literally zero-alloc.** Delta buffers (`ds/dp/do_`), the `batches`
   vec, and the seen set allocate. BN3NS (BTreeSet seen, no hasher) removed
   the hasher TLS but not the allocation.
2. **Per-round store copying is gone.** Alloc cost is not O(store × rounds)
   — the retired full-store-copy arm's signature. At 100k-store, 3 rounds:
   9,307 allocs total vs the retired arm's store×rounds row-buffer class.
3. **The strong "allocs scale with delta not store" claim is refuted.** The
   seen-set seed is O(store/11) node allocations per run. Remaining alloc
   profile: O(store/11) seed + O(delta/11) insert-nodes + amortized Vec
   doubling + per-rule scratch hoisted once.

## 3. riscv32imc check (BN3NS landed)

`cargo check -p cgap-bn3 --target riscv32imc-unknown-none-elf` (target dir
/tmp/h8-target): **exit 0** (`Checking cgap-bn3 v26.11.2 ... Finished dev
profile ... 0.48s`).

## 4. Host suite regression witness

`cargo test -p cgap-bn3` (CARGO_TARGET_DIR=/tmp/h8-target): **exit 0** —
lib unittests 119 passed / 0 failed; tests/bgp_hop_courts.rs 2 passed / 2
ignored (expensive debug runs, loudly ignored); tests/lever_courts.rs 7 passed;
doc-tests 0. Total 128 passed, 0 failed, 2 loud skips.

## 5. cgap-storage closure.rs is the real remaining alloc site (not bn3)

The 1,117-alloc / 1.2GB figure belongs to `crates/cgap-storage/src/closure.rs`
(p1-closure lane), NOT bn3. Witnessed this session:

- `closure.rs:33` — `SeenSet.words: std::collections::HashSet<u64>` (std hash
  set with RandomState; per-insert node alloc, no capacity bound).
- `closure.rs:118-119` — fresh `Vec::new()` for `next_x`/`next_z` **per round**
  (alloc ∝ rounds × round-delta), replaced by `delta_x = next_x` at :129-130.
- `closure.rs:154` naive oracle uses `std BTreeSet known`.

Follow-up spec (named, not implemented here): **closure-alloc-fix**
(cgap-storage). Persistent delta buffers + reused seen-set:

1. Hoist `next_x`/`next_z` to loop scope, `next_x.clear()`/`next_z.clear()` per round
   (amortized capacity, never re-allocate).
2. Replace `SeenSet`'s `std HashSet<u64>` with a `Vec<bool>`-backed direct
   N×N bit-plane... infeasible at bench N=1048576 (4 TiB, per closure.rs:11-13
   doc comment — the module's own header documents why bit-plane is out).
   Instead: keep packed-pair keys in a single arena-owned open-addressing
   table with a reused occupancy buffer across rounds (one alloc, reused;
   typed loud refusal on capacity overflow, following the fixed-capacity
   pattern of cgap-bn3 `noalloc.rs`).
3. Naive oracle (closure.rs:154) stays std (court collaborator, not hot path).

Falsifier for the follow-up: with the counting-allocator probe pointed at
`closure::transitive_closure` at N=1M edges, alloc count must drop to
O(rounds)-plus-arena, i.e. **allocs(N=1M) < 1000** (vs current class:
1,117 allocs / 1.2GB), and must not scale with N. Mutation that must flip the
court: re-introducing a per-round `Vec::new()` (or a fresh HashSet per round)
must multiply the alloc count by rounds and flip the court.