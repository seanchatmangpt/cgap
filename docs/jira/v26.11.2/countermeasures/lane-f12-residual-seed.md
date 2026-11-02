# Lane F12 — residual grid → machine-readable seed for lsp-max

2026-11-02, repo /Users/sac/cgap (main). Owner files only:
`docs/jira/v26.11.2/countermeasures/residuals.toml` (new) and this
receipt.

## What was written

`residuals.toml` — 10 rows, parsed clean by `python3 -c "import
tomllib; ..."` (exit 0, this session). Schema documented in-file:

- `symbol` — crate-local fn path (`select::select_slice`, ...)
- `target` — `aarch64` | `riscv32imc` | `any`
- `residual_kind` — `isel_residual` (documented branch count asserted
  EXACTLY by the census), `repaired_hot` (0-branch post-repair PASS),
  `skip_reference` (SKIP-REFERENCE, presence only)
- `branch_count` — exact allowed flow-instruction count; `-1` for
  skip_reference rows (not censused)
- `provenance` — receipt path:line per row (cite, not a copy)

## Row inventory (10 rows = 7 symbols x per-target)

| symbol | target | kind | count | provenance (receipt:line) |
|---|---|---|---|---|
| select::select_slice | aarch64 | isel_residual | 1 (cbz) | repairs/repair-01:72 |
| select::select_slice | riscv32imc | isel_residual | 2 (beqz+bnez) | repairs/repair-01:72-73 |
| select::cmov_u64 | aarch64 | repaired_hot | 0 (cmp+csel) | repairs/repair-01:68 |
| select::cmov_u64 | riscv32imc | isel_residual | 1 (bnez) | repairs/repair-01:68-71 |
| eytzinger::eytzinger_restore | aarch64 | repaired_hot | 0 (rbit/clz/lsr) | repairs/repair-01:74-75 |
| eytzinger::eytzinger_restore | riscv32imc | isel_residual | 1 (beqz) | repairs/repair-01:74-76 |
| select::saturated_subs_p28 | any | repaired_hot | 0 (both targets) | repairs/repair-01:62-65 |
| eytzinger::eytzinger_search | any | skip_reference | -1 | repairs/repair-01:78-81 |
| eytzinger::to_eytzinger_layout | any | `skip_reference` | -1 | repairs/repair-01:78-81 |
| select::checked_uniform_select | any | skip_reference | -1 | repairs/repair-01:78-81 |

Cross-checked against convergence/lane-11-branchless-census-witness.md:18-23
(residual verdict lines 1/1, 2/2, 1/1 match) and
exhaustive/lane-03-branchless-complete.md:30-40 (cbz 1 host / 2 riscv,
bnez 1, beqz 1, reference drivers all match). No divergence between the
three sources on any row.

## Handoff note for lsp-max (branchless-lsp)

Copy (or reference, once lsp-max grows a docs bridge) the file to
branchless-lsp's config root as `residuals.toml`. branchless-lsp's
`attributes.rs` consumes it at attribute-expansion time: for each
`#[allow_isel_residual]` site it looks up `(symbol, target)` pairs in
this TOML; a row with `residual_kind = "isel_residual"` seeds the
allowlist with the exact `branch_count`, `repaired_hot` rows seed
count 0 (HOT, no allow needed), and `skip_reference` rows are loaded
but never grant an allow (presence-only). Rows absent from the TOML
get the default conservative behavior (no pre-seed; the census must
assert them first). Provenance strings are kept verbatim on the
allowlist entry so an lsp-max audit can trace every allowance back to
the cgap receipt that documented it. This kills the EX3/B6/REP1
duplication: three receipts converge on one machine-readable grid.

## Reads (this session)

- `docs/jira/v26.11.2/repairs/repair-01-branchless-census.md` — full
  read (130 lines), authoritative grid.
- `grep` over convergence/lane-11-branchless-census-witness.md (exit 0,
  residual verdict lines 18-23, 36-50, 58-59) and
  exhaustive/lane-03-branchless-complete.md (exit 0, rows 30-40).

## Commands

- `mkdir -p .../countermeasures && python3 - <<EOF` (assemble +
  write + tomllib parse) → printed `parsed OK, 10 rows`, exit 0.

## Standing

ALIVE for the landed scope: TOML exists on disk, parses with tomllib
(exit 0, this session), 10 rows with per-row provenance cites verified
against the source receipts read this session. UNVERIFIED: any claim
about lsp-max consumption (that integration is a downstream lane's
work, not witnessed here).

## Divergences

None between the three provenance sources. One note: repair-01's
cmov_u64 riscv residual is honestly recorded as 1 bnez even though the
mask-form source doc over-claimed 0-branch (repair-01:118-122); the
TOML records the witnessed count (1), not the stale doc claim.
