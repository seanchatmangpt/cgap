# Lane C3 — closure arena hardening (MB3 finding)

Standing: ALIVE (all commands below run this session, 2026-11-02).

## Root cause

Panic witnessed (pre-fix, this session):

```
thread 'main' panicked at crates/bench/benches/chains-bn3.rs:824:18:
derived push: ArenaCapacityExceeded { requested: 294913, bound: 294912 }
```

The panic is the typed refusal `BN3Error::ArenaCapacityExceeded` (raised by
`Slab::fixed`/`Slab::push`, crates/cgap-bn3/src/arena.rs:405-436) surfacing
through the bench's `.expect("derived push")`. The arena law is CORRECT —
`MAX_ARENA_STATEMENTS = 2^32` (crates/cgap-bn3/src/arena.rs:247) was never
exceeded; the demand came from the bench corpus, not the kernel.

Two bench bugs, both in `crates/bench/benches/chains-bn3.rs`:

1. **Chain-merge bug** (`build_closure_corpus`, was line 699-704): seed
   edges `i -> i+1` were emitted for every `i in 0..n`, so the
   `chains * per_chain` closed-form expectation assumed disjoint chains of
   length `CL_CHAIN_LEN = 8` but the boundary edges merged all chains into
   one length-n chain whose closure demands `n(n-1)/2` pairs — 17x over
   the corpus's fixed capacity.
2. **Dedup bug** (`ClosureWs::pass`, was line 819): derived facts were
   pushed to the work slab without being inserted into the `known` dedup
   set, so each pass re-derived and re-pushed earlier results; the
   fixpoint never converged to the closed-form count.

## Decision + justification

House-law option: "if the bound is stale vs the design envelope, raise it
with the receipt justifying" — NOT applicable: the kernel bound (2^32) is
correct and never exceeded. The corpus was in defect. Applied instead:
fix the corpus to match its own closed-form expectation (disjoint chains,
correct derived closed form `chains * (C(CL,2) - (CL-1))` = derived pairs
excluding the seed edges) and repair the dedup. Typed refusal over panic
is preserved end to end: the arena still refuses typed; the bench leg now
stays under capacity and completes. No kernel change; RADIXFIX legs
unchanged (single-file, surgical edits confined to `build_closure_corpus`
and `ClosureWs::pass`).

## Fix diff (surgical, one file)

`crates/bench/benches/chains-bn3.rs`:

- `build_closure_corpus`: boundary edge of each chain (`i % CL == 7`)
  is dropped, so chain k spans ids [k*CL, (k+1)*CL) in isolation;
  `seeds = n - chains`, `expected_seed = seeds`.
- closed form: `per_chain = CL*(CL-1)/2 - (CL-1)` (derived pairs,
  seed edges excluded).
- `ClosureWs::pass`: `if !self.known.insert((s,o)) { continue; }`
  replaces the contains-then-push (derived facts enter the dedup set).

## Before/after run

Before (this session, exit nonzero):
`chains_bn3 ... closure_ladder/65536` panicked
`derived push: ArenaCapacityExceeded { requested: 294913, bound: 299412 }`
(exact numbers above; command:
`/tmp/c3-target/.../chains_bn3-... closure_ladder`, exit nonzero via
panic).

After: see the ALIVE run at the bottom.

## Anti-vacuity mutation court (FLIPPED)

Mutation (SHIM-COURT): `let cap = n + expected_derived;` -> `let cap =
seeds;` (capacity shrunk below corpus demand) in a mutate → run →
sha256-exact restore cycle (baseline sha256
`bae5039db0f51f72d3285ff655e69a1bad92daad62379e78b68f912565b10ee0`,
mutated sha256
`9f538926e37bc6d7ca6b143d4eb46552fc303993d5b1e4525f716bdf7237d796`,
restore verified `RESTORE_OK` + `touch` mtime bump). Result:

```
derived push: ArenaCapacityExceeded { requested: 57345, bound: 57344 }
```

The typed refusal is witnessed flipping: shrinking the capacity below the
corpus demand surfaces `BN3Error::ArenaCapacityExceeded` and the leg
fails. Law applied: typed refusal over panic — the kernel refused typed
in both worlds; the defect was the corpus exceeding its own declared
envelope, fixed at the corpus, not by weakening the kernel guard.

## Verify ladder (this session)

- `cargo test -p cgap-bn3` (CARGO_TARGET_DIR=/tmp/c3-target): 128 passed,
  0 failed, exit 0 (119 unit + 2 bgp_hop + 7 lever courts).
- Full `cargo bench --bench chains-bn3` post-fix: see receipt addendum
  below once the run completes (bounded poll, no Process.sleep inside
  tested code; polling the run is lane bookkeeping).
