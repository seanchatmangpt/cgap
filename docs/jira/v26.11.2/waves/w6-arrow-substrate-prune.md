# A2P — Arrow substrate prune receipt

Lane A2P (v26.11.2 w6). Repo /Users/sac/cgap, branch main. Date 2026-11-02.

## Diff summary

| File | Change |
|---|---|
| crates/cgap-arrow/Cargo.toml | +`nested-types = []` feature (default OFF) |
| crates/cgap-arrow/src/lib.rs | `timestamps` mod + re-export gated behind `all(not(target_os = "none"), feature = "nested-types")` |
| crates/cgap-arrow/src/validate.rs | NEW: fixed-width 3-column u64 validator + 5-test in-file court (raw-base API, no deref, typed fail-closed errors) |
| docs/jira/v26.11.2/waves/w6-arrow-substrate-prune.md | this receipt |

## Dependency edges (why the other candidates stayed UNGATED)

Qualified-reference sweep (grep of `mod::` + pub type names, tests/benches
counted, comments excluded):

- `varlen`, `views`, `runend` <- batch.rs:19-21 (Column::Utf8/LargeUtf8/
  Utf8View variants in the alloc-backed facade). Gating breaks the core
  facade. Also: cgap_arrow tests (validation/property/mutation_kills),
  bench bin, and `audit/src/lib.rs:10` uses `views::is_inline`.
- `slice` <- cgap-arrow tests/mutation_kills.rs:20 + bench arrow_accessors
  _bench.rs:40. Gating breaks the crate's own test suite at default features.
- `c_abi_nested` <- integration/tests/membranes.rs:32 (`export_struct`);
  cgap-ffi reference is comment-only (sovereign.rs:2046). Dev-dep on
  default features would break membranes compile.
- `timestamps` <- ZERO qualified code references outside lib.rs re-export;
  layout.rs:12/20 hits are comments. GATED (kept on disk, not deleted:
  published-crate downstream may use it under the explicit feature).

## Commands + exits (this session)

- `make gate0` -> exit 2 PRE-EXISTING (ggen.lock pack hash mismatch:
  expected blake3:e085b8d1..., on disk 22ba9f10...; pack restore/ggen.lock
  regeneration outside this lane's ownership). Before AND after — the
  cargo/test verification ladder below substitutes; not silently counted
  as a lane pass.
- `cargo check -p cgap-arrow --no-default-features --target riscv32imc-unknown-none-elf` (CARGO_TARGET_DIR=/tmp/a2p-target) -> exit 0 (pre- and post-validate.rs).
- `cargo test -p cgap-arrow` -> exit 0, 189 passed / 0 failed.
- `cargo test -p cgap-arrow --features nested-types` -> exit 0, 192 passed / 0 failed.
- `cargo test -p cgap-arrow validate` -> 5/5 courts pass (valid view, overflow,
  unaligned base refused without deref, None column, Display text).
- `cargo check -p cgap-ffi -p cgap-audit` -> exit 0 (downstream consumers).
- wasm32-unknown-unknown audit rlib CODE sums (llvm-size -A, exact
  nightly toolchain binary, awk $1=="CODE"):

| Build | CODE bytes |
|---|---|
| baseline (pre-lane, timestamps on, no validate.rs) | 84,951 (matches m1 receipt) |
| timestamps gated OFF + validate.rs | 85,608 |
| timestamps gated ON + validate.rs | 85,822 |

Honest reading: the rlib-CODE method counts ALL codegen, not linked output.
timestamps contributes ~21 bytes of rlib codegen (its impls are generic-
adjacent and mostly dead at rlib level); validate.rs adds ~657-678. The
~1,700-line prune candidate is NOT realizable as .text reduction under the
rlib method because the other five modules have live dependency edges.

## Anti-vacuity court (gate flip)

llvm-nm on the two isolated rlibs (/tmp/a2p-m-{off,on}):
- OFF: 0 `TimestampMicro` symbols; ON: 1. The gate flips; a run that still
  saw the symbol after gating would be VACUOUS and exit nonzero.

## Standing

- Prune (timestamps gate + validate.rs): ALIVE — commands + exits above,
  run this session against the exact on-disk subject.
- Full ~1,700-line reduction: UNVERIFIED as unattainable-by-evidence;
  edges recorded, re-prune requires touching batch.rs / integration
  tests / audit first.
- gate0: BLOCKED pre-existing (hash mismatch, owner: ggen/pack lane).

## Divergences

1. gate0 exit 2 pre-existing before any lane edit (ggen.lock vs pack hash).
2. Lane doc anticipated gating varlen/views/runend/slice/c_abi_nested;
   evidence says no — each has a live consumer edge listed above. Doc
   yields; recorded instead of gated.
3. tests/validation.rs (W3-VALID) predates and is unrelated to the new
   src/validate.rs module (test file not in lane ownership; left alone).
