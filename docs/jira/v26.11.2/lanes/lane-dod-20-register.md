# lane-dod-20-register receipt

## W1 Identity

Lane DOD-20, DEFECT-REGISTER refresh (append-only). Repo /Users/sac/cgap,
branch main, 2026-11-02. Files written (ONLY these):

- `docs/jira/v26.11.2/DEFECT-REGISTER.md` (appended `D-DOD-20 refresh`
  section, lines 129-210; zero history rows rewritten)
- `docs/jira/v26.11.2/lanes/lane-dod-20-register.md` (this receipt)

## W2 Authority

Dispatch: append dated status lines per row of DEFECT-REGISTER.md,
citing tc-10-open-recensus.md + landed lane receipts re-read from disk.
No git commands run. No source edits. cgap-core src/ read-only (lib.rs:1-11
read to verify FR-06 mount posture).

## W3 Consequence (append diff, per row)

- D-G08-1/R4 G2-PARETO: OPEN, measured-but-failing (lane-gap-07:50-58,90,14-17)
- D-G08-2 FA10: CLOSED stands; fix (b) gate arm OPEN (lane-fa10r:46-48)
- D-G08-3 FA1 framing: OPEN — no edited chains-ingress.rs:24 receipt on disk
- D-G08-4 FA2 Cpk: OPEN — no 30-trial receipt on disk
- D-G08-5 RV1: OPEN, law unchanged
- FR-06 checkpoint: DEFECT-OPEN -> PARTIAL — prototype exists, #[path]-mounted
  test-harness-only (crates/cgap-core/src/lib.rs:4-5,
  lane-cgap-prd-06-checkpoint.md:11,29); owed: lib mount + fixpoint-scope
  (lane-posh-04-sentry.md:21)
- R1 CLOSED (lane-mv-05-sla-reconcile.md:22-27; mutants flip exit 1)
- R2 CLOSED (lane-fix02-parquet-features.md:7-13)
- R8 CLOSED (lane-gap-11-hygiene.md:13-20)
- R9 CLOSED (lane-mv-08-doc-refresh.md:7-15,34)
- R10 OPEN-NARROWED (lane-gap-04:56,81 vs release-blockers:248-255 conflict
  flagged; DOD-11 receipt ABSENT on disk -> UNVERIFIED, not cited)
- R11 CLOSED (lane-mv-08-doc-refresh.md:34)
- NEW-7 dod10_no_spdx: CLOSED — DOD-01 fix landed (lane-dod-01-spdx.md:28-35)
  + DOD-20 independent re-witness this session
- NEW-5 POSH-2: decision pending (lane-posh-02-batch.md:29-32,38);
  DOD-12 receipt ABSENT on disk -> UNVERIFIED
- AX-family gap: registered per DFX-6 disclosure (lane-dfx-06-rollup.md:21-22)

## W4 Replay (commands + exits, this session)

| command | exit | note |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/dod20-target cargo test -p cgap-core manifest_court` | 0 | dod10_no_spdx passing |
| `CARGO_TARGET_DIR=/tmp/dod20-target cargo test -p cgap-core` (full suite) | 0 | 13 test binaries, all ok, 0 failed — NEW-7 fix witnessed independently |
| grep sweeps over docs/jira/v26.11.2/lanes/ for chains-ingress edit, 30-trial receipt, lane-dod-11*, lane-dod-12* | 1 / no-match | absence evidence for OPEN rows; DOD-11/DOD-12 UNVERIFIED |
| `tail`/`wc -l` on-disk verification of the appended register | 0 | 210 lines, tail intact, no mid-file corruption |

## W5 Verification ladder

Register re-read fresh before append (Read tool); tc-10-open-recensus.md
re-read fresh; each cited receipt grepped at file:line (not from memory);
cgap-core suite re-run as an independent witness of the DOD-01 claim
instead of citing it blindly; on-disk bytes re-checked after write.

## W6 Standing

ALIVE for the appended refresh itself (writes verified on disk, cargo
witness exit 0 this session). Per-row standing as stated in W3: CLOSED
rows carry receipt file:line + this-session re-witness where claimed
(NEW-7); OPEN rows carry absence evidence (grep exit 1); DOD-11/DOD-12
closures are UNVERIFIED — receipts not on disk, so the dispatch's
"GAP-04+DOD-11" and "DOD-12's package" citations could not be grounded
and are recorded as divergences instead of fabricated.

## W7 Divergences

- Dispatch said R10 closes via "GAP-04+DOD-11": no lane-dod-11 receipt
  exists on disk; R10 left OPEN-NARROWED with the absence recorded.
- Dispatch said POSH-2 decision = "DOD-12's package": no lane-dod-12
  receipt exists on disk; NEW-5 left pending/UNVERIFIED.
- lane-cgap-prd-06 receipt is cited as lane-cgap-prd-06-checkpoint.md
  (actual filename); tc-10 cited it as lane-cgap-prd-06:5-8 — line
  numbers map to the actual file (:5-8 verified: exit 101, only failure).
