# w6-rv32imc-branchless-repair — lane RV1 receipt

Lane: RV1 (riscv branchless repair). Date: 2026-11-02. Repo:
/Users/sac/cgap, branch main.

## Files written (ownership)

- `crates/cgap-branchless/src/arithmetic.rs` — `saturated_subs_p28`
  body swapped from the u128/sbb-chain borrow to the dispatch's pure-u64
  carry kill/propagate form. Host aarch64 and rv32imc both branch-free.
- `crates/cgap-branchless/src/select.rs` — `select_slice` and
  `pred_inc` bodies branchless; loud mixed-mask refusal moved OUT of
  the hot path into a new cold fn `checked_uniform_select`
  (`#[inline(never)]`, scan + same panic text as the B1 form).
- `docs/jira/v26.11.2/waves/w6-rv32imc-branchless-repair.md` (this
  receipt). No other file touched; no git writes.

## Divergences from the lane doc

1. gate0 exit recorded as **1**, not the predicted 2: with no mode
   argument the gate prints `no mode: one of --mutate, --shim-court-
   only, --deps, --strings required` and exits 1 (a usage refusal,
   pre-existing). Regression criterion held: identical before/after
   (exit 1, same stdout) — no FM-PACK-008 regression introduced.
2. Census riscv arm: the census resolves `rustup` against its own cwd.
   Run from outside `/Users/sac/cgap` the default toolchain (stable)
   lacks riscv32imc and the court prints `[SKIP] riscv32imc target not
   installed` and censuses only 11 host cells. Run with cwd = repo
   root, the `rust-toolchain.toml` nightly override applies, riscv is
   installed, and the court censuses all 19 cells. All grids below are
   19-cell runs (cwd = /Users/sac/cgap).
3. **REJECTED PROPOSAL (recorded per dispatch)**: masking the tripwire
   with `0x5032_38FF` is refused. That constant is the ABI-frozen
   ERR_P28_SPOLIATION trap code (trap.rs:36), not a mask; ANDing it
   would break the naive-oracle contract (tripwire all-ones-iff-
   underflow) and flip B2's differential courts. Not implemented.
4. Contract note: `select_slice` is now the hot uniform-mask select
   with a caller precondition (uniform mask, equal lengths); the loud
   mixed-mask refusal lives in `checked_uniform_select`. The B1
   `#[should_panic]` test was retargeted accordingly, plus a
   hot/checked agreement test added.
