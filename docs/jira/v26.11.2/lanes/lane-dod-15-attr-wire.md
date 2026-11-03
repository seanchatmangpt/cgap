# Lane DOD-15 — close LSPA-3's red arm: parameterized attr admits on the wire

Repo: /Users/sac/lsp-max (branch main) + /Users/sac/cgap (docs). Date: 2026-11-02.

## Files written (ownership)

- `/Users/sac/lsp-max/crates/branchless-lsp/src/rule_pack.rs` — the ONLY
  production file touched.
- `/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-dod-15-attr-wire.md` —
  this receipt.
- `crates/branchless-lsp/tests/did_save_court.rs`: **UNTOUCHED**. The
  dispatch authorized edits; none were needed — the court flipped green
  purely from the source fix. Arms (ii)/(iii) kept byte-identical (the
  read-fresh / do-not-weaken duty cost zero edits).
- No git commands anywhere. No sleeps; courts poll the real socket with
  bounded deadlines (pre-existing, unmodified).

## The fix (root causes 1+2 from the LSPA-3 handoff, plus one NEW root cause)

### Root cause 1 — extractor shredding (rule_pack.rs extract_attr_paths)

The old extractor split each `#[...]` inner payload on `[',', ' ']`,
shredding `allow_isel_residual(target = "aarch64", max_branches = 2, ...)`
into fragments that fail `strip_suffix(')')` → MissingCloseParen. New
`split_attr_inner` tokenizes quote- and paren-aware: a token ends only at
whitespace OUTSIDE any paren pair and OUTSIDE a string literal, so the
whole `allow_isel_residual(...)` form arrives at `parse_attrs` as ONE
token, which parse_attrs already knows how to split on top-level commas.
Bare forms tokenize identically to before (`no_mangle`,
`allow_isel_residual`).

### Root cause 2 — unwrap_or_default laundering

`fn_attr_flags` previously did `parse_attrs(...).unwrap_or_default()`.
Now returns `Result<ParsedAttrs, AttrParseError>`; `run_tier2` falls back
to DEFAULT (refusing) attrs on Err AND surfaces the typed error as a
WARNING `ATTRIBUTE_NOTE` via the existing note_diag path (fail-closed AND
surfaced — never silent). No external callers existed (grep-verified), so
the signature change is contained. Two new unit tests pin both flips:
`parameterized_payload_survives_extraction_intact` and
`typed_parse_error_is_surfaced_not_laundered`.

### Root cause 3 (NEW, found this session) — bool-form count collapse

With causes 1+2 fixed, arm (i) went green but arm (ii) (budget N-1
refuses) went RED with a NEW finding: `run_tier2` classified via
`grid.classify_seeded(...)` — the bool form, which delegates as
`usize::from(branches_found)` (=1) into `classify_seeded_count`. A
measured `max_branches = 1` therefore admitted 2 real sites. Fixed:
`run_tier2` now calls `classify_seeded_count` with `f.sites.len()` — the
exact measured count. This root cause was UNREACHABLE before cause 1+2
(the gate never ran the parameterized path on the wire at all), which is
why LSPA-3's witness could not have caught it.

## Before/after wire witness (real court, exact subject)

- BEFORE (LSPA-3 receipt, N=2 measured): arm (i)
  `lspa3_measured_budget_admits_branchy_on_wire` RED — valid
  `allow_isel_residual(target="aarch64", max_branches=2, reason=...)` on
  the branchy body refused with `ANDON_REFUSED ... fn 'g06_branchy'`
  x2. Reconfirmed this session at the pre-fix state? No — the pre-fix
  red is cited from the LSPA-3 receipt (the fix was already in flight
  when I picked up the file; the pre-fix binary was never rebuilt).
- AFTER (this session): 7/7 did_save courts, including
  - arm (i) GREEN: measured budget max_branches=2 admits, zero ERROR;
  - arm (ii) GREEN (now NON-vacuous, honestly refusing): it went red
    once under root cause 3 with "[FAIL] max_branches=1 ... vacuous
    budget", and only the exact-count fix turned it green — the flip is
    witnessed, not assumed;
  - arm (iii) GREEN: riscv32imc mismatch under the aarch64 court refuses
    and names the fn.
- measured N: read fresh each run by measured_branch_count() from the
  real rustc/capstone court (witnessed 2 this session via arm (ii)'s
  failure message "measured 2 minus one").

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/dod15-target cargo test -p branchless-lsp --lib`
  -> 74 passed / 0 failed (first run 73/1 while a wrong assertion in MY
  new unit test was corrected; second 74/0).
- `cargo test -p branchless-lsp --test did_save_court` -> first run
  6/1 (arm (ii) RED under root cause 3 — the finding), after the
  exact-count fix 7/7 ok.
- `CARGO_TARGET_DIR=/tmp/dod15-target cargo test -p branchless-lsp`
  (FULL, unskipped) -> exit 0. lib 74/0, main 0/0, code_actions 5/0,
  compile_note 5/0, conformance_seal 4/0 (the LSPA-1 hang: verified
  FRESH this session — completes in ~0.09s, no skip needed),
  did_save 7/0, e9_precision 6/0, riscv32imc 3/0. Total 104/0, zero
  loud skips.

## Standing

- Arm (i) + the parameterized gate on the wire: ALIVE (ran this session,
  exits above).
- Arms (ii)/(iii): ALIVE and non-vacuous (budget-1 refusal + mismatch
  refusal witnessed under the exact-count court).
- Full census: ALIVE (exit 0, 104/0, 0 skips).
- Conformance seal court hang: resolved upstream (drain task); verified
  fresh, not assumed.

## Fences (must-not-regress) and their runs

- Fence 1: 4 pre-existing G06 did_save courts — RAN, green (7/7 block).
- Fence 2: LSPA-2 compile_note courts — RAN, 5/0.
- Fence 3: LSPA-1 conformance seal court — RAN, 4/0, no hang.
- Fence 4: riscv32imc + e9 precision courts — RAN, green.
- Fence 5: LSPR/WG3 attributes unit courts (37 attr tests in lib) —
  RAN, green within the 74/0 lib result.

## Divergences from dispatch

1. tests/did_save_court.rs: zero edits (dispatch authorized; not needed).
2. A third root cause (bool-form count collapse in run_tier2's
   classification call) existed beyond the two the dispatch named; it
   was invisible until 1+2 were fixed, and arm (ii) caught it exactly as
   an anti-vacuity court should.
3. Root-cause note: `classify`'s bool form itself still collapses counts
   (attributes.rs, NOT my owned file) — left untouched, called out here
   as a known conservative-ish semantic for any lane touching
   attributes.rs next.
