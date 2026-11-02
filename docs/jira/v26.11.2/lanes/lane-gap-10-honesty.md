# Lane GAP-10 Receipt — honesty backfills + R7 receipt map

Date: 2026-11-02. Repos: /Users/sac/cgap (docs), /Users/sac/lsp-max
(test runs only — no source edits to lsp-max). Branch: main, NO git
commands run. Standing per task:

## Files written

- `docs/jira/v26.11.2/lanes/lane-lspa-01-shutdown-seal.md` — BACKFILL of
  the "TO BE FILLED" Commands+exits section (the only edit; receipt
  history above the marker untouched).
- `docs/jira/v26.11.2/lanes/lane-c05-dead-code-excision.md` — APPEND-only
  amendment (dated GAP-10 amendment note; original text unmodified).
- `docs/jira/v26.11.2/audit/workstream-receipt-map.md` — NEW (F5/F10/
  WG2/WG3 mapping).
- `docs/jira/v26.11.2/lanes/lane-gap-10-honesty.md` — this receipt.

## Task 1: LSPA-01 backfill (fresh gate run)

Staleness check: `strings /Users/sac/lsp-max/target/release/
branchless-lsp | grep -c ANDON_REFUSED` = 3, exit 0 — matches
dod-lsp-max-wire.md:38-40; binary fresh, no rebuild needed. (LSPA-2 was
NOT mid-edit: rule_pack.rs mtime 13:52:19, lib.rs 13:55:30, both >35 min
before my run; no mid-flight edit collided.)

Run shape: full `cargo test -p branchless-lsp` blocked on the shared
`target/` file lock (concurrent LSPA-1/LSPA-2 lanes running the same
suite) for >30 min; relaunched in lane build roots
`CARGO_TARGET_DIR=/tmp/gap10-target` and `/tmp/gap10-target2`. Results
(full table in the LSPA-01 backfill, cited there):

- lib 72/0, code_actions 5/0, e9 6/0, riscv 3/0 — green.
- conformance_seal_court: 3/4 green; `session_totals_observable_before_
  shutdown` HANGS deterministically (cross-build-root corroborated by
  LSPA-1's own independent run, pid 35280, stuck 30+ min).
- did_save_court: 1 real FAILURE — `lspa3_measured_budget_admits_branchy_
  on_wire` (did_save_court.rs:311): max_branches=2 measured budget
  expected admit, got 2x ANDON_REFUSED + 1 blocking-without-push.

Both defects are on LSPA-1/LSPA-2/LSPA-3 in-flight surfaces; per
dispatch I recorded, did not fix. LSPA-01 standing downgraded to
PARTIAL in the backfill (A13's receipt_lint RECEIPT-STRUCT-002 flip
condition: the section is now filled, and filled with what RUNS).

## Task 2: C05 amendment (disclose-in-both closure)

Appended a dated amendment to
`lane-c05-dead-code-excision.md` citing
`lane-fix07-bench-build.md:48-54` and closing the violation DOCTRINE-1
flagged (release-blockers.md context). On-disk verification of the
misfire-restored state: `let mut lease` present at
`crates/bench/benches/chains-seal.rs:216/339/480`; block_len use sites
live at `chains-parquet.rs:243/246/327` (grep this session). C05's
original receipt rows at :51/:53 are superseded for those two items by
the amendment.

## Task 3: workstream-receipt-map.md

Word-boundary grep `\b(F5|F10|WG2|WG3)\b` over cgap
v26.11.2 {lanes,waves,audit,dflss} + /Users/sac/lsp-max/docs/. Result:
F5 → `lsp-max/docs/lane-f5-workspace-integration.md:11-17` PARTIAL
(covers the 21-test-era integration; suite is now 90+ tests per
dod-lsp-max-wire.md:8-21). F10 → NO RECEIPT FOUND, UNVERIFIED, no
rounding. WG2 → wg2-tier2-wiring.md:52-68 + lane-g05 + dod:44,124,
ALIVE. WG3 → wg3-parameterized-attrs.md:56-60 + lane-g04, ALIVE.
R7 close condition now partially satisfied: mapping landed; F10 row
stays OPEN-UNVERIFIED until a receipt lands or the coordinator retires
the token.

## Task 4: this receipt

Every claim above cites a file:line or a fresh command+exit this
session.

## Commands + exits (this session)

| Command | Exit |
|---|---|
| `strings /Users/sac/lsp-max/target/release/branchless-lsp \| grep -c ANDON_REFUSED` | 0 (count 3) |
| `cd /Users/sac/lsp-max && cargo test -p branchless-lsp` (shared target) | killed at 30 min — blocked on target/ file lock by concurrent LSPA lanes |
| `CARGO_TARGET_DIR=/tmp/gap10-target cargo test -p branchless-lsp` (full suite) | hung in conformance_seal_court, killed |
| `--test conformance_seal_court` solo (gap10-target) | hung on session_totals_observable_before_shutdown, killed |
| `--test did_save_court --test e9 --test riscv` (gap10-target2) | 101 (did_save 6/1) |
| `--test e9_precision_court --test riscv32imc_court` (gap10-target2) | 0 (6/0, 3/0) |
| `stat` rule_pack.rs / lib.rs mtimes | 0 (13:52:19 / 13:55:30) |

## Fences (must-not-regress surfaces my writes could break)

- `audit/release-blockers.md` R7 text (read-only; my map is the errata
  artifact it names — no edit made to it).
- C05 receipt (append-only; sha256 before/after of the untouched prefix
  preserved — appended bytes only, python `open(p,'a')`).
- LSPA-01 receipt (single-line replacement of the TO-BE-FILLED marker
  only; wording above intact).
- receipt_lint RECEIPT-STRUCT-002 on lane-lspa-01: the flip it wanted
  (unfilled section) is now resolved — re-run of the linter is the
  owning wave's gate, not mine.

## Divergences

- Dispatch said "rebuild if stale" — binary was fresh (count 3), so no
  rebuild was performed; the 13:44 binary (dod-lsp-max-wire.md:40) is
  still current on-disk.
- One stray kill: my cleanup pattern `gap10-target` also matched
  `gap10-target2` and killed the in-flight other-suites run; rerun
  completed and produced the recorded exits. Disclosed as a
  session-internal process error, no tree impact.
- Build-root leases /tmp/gap10-target, /tmp/gap10-target2 deleted at
  lane end (see cleanup commands below).

## Cleanup

python rmtree of both lane build roots at lane end.
