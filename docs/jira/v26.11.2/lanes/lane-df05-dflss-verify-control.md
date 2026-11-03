# lane-df05-dflss-verify-control receipt

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Lane: DF-05
(VERIFY/courts + control plan). Owned files: dflss/06-control-plan.md
(merge target). dflss/05-cpk-analysis.md and 07-ax-sipoc-c4.md were
READ this session and required NO changes — 05 already carries the
CPK_INJECT_FAIL flip + honest-FAIL table; 07 already carries the
0/1/2/3 exit-code law and typed-refusal outputs.

## Diff (06-control-plan.md only, appended Section 5)

- 5.1 anti-vacuity court narratives: RED-13 path-alias race (exit 7 x2,
  fa9 court vacuity hole recorded), RED-12+FIX-02 SRAM gate flip
  (all flip exits cited from lane-fix02-bn3-sram-gate.md), RED-14
  flood liveness (2,000,001-trip), mutation non-vacuity law.
- 5.1b: "18 publishable units clean dry-run matrix" marked UNRUN
  loudly — no receipt runs it (lanes/waves grep this session).
- 5.2 SPC control table: verify_cscb_sram.py (0/1/2/3, docstring
  lines 22-27), zero_string_gate.py (0/1 GATE-REFUSED, :23-27),
  ffi_surface_gate.py (EXISTS, 0/1/2/3 + --self-check), boundary_gate.py
  (consts :151-155), receipt_lint.py (0/1/2/3; KeyError('rule')
  --strict crash = open OUT-OF-CONTROL), receipt_doctor.sh +
  mint_receipt.sh (fail-closed minter), cargo test batteries.
- 5.3 standing table.

## Commands + exits (this session)

| Command | Exit |
|---|---|
| cat red-20 synthesis | 0 |
| ls docs/jira/v26.11.2/dflss/ | 0 (17 files) |
| cat 06-control-plan.md, 05-cpk-analysis.md, 07-ax-sipoc-c4.md | 0 |
| cat red-13, red-12, red-14 receipts | 0 |
| cat lane-fix01/fix-02/fix-04 receipts | 0 |
| grep exit contracts (7 scripts, per-script) | 0 |
| grep dry-run receipts across lanes/+waves/ | 0 (no publish
  dry-run matrix found) |
| awk length>100 sweep over 06-control-plan.md | 0 (2 long lines
  fixed in-session; pre-existing GATE rows left as-is) |

## Courts run

None executable — DF-05 is a documentation/restructure lane; every
landed narrative cites its owning receipt's command+exit (red-13 exit 7
x2, fix-02 mutation courts exit 0-with-flip, red-14 exit 0 flood, GATE
witnesses in 06). No court was re-run and none is claimed as re-run.

## Standing

ALIVE (documentation lane): every cited path:line re-read on disk this
session; no invented runs; UNRUN/PENDING rows marked loudly.

## Fences

- Existing GATE matrix + out-of-control log untouched (merge, never
  clobber). No scripts/, crates/, lib.rs, Cargo.toml, or git state
  touched. Other lanes' receipts cited read-only.
- Must-not-regress: GATE-02/03/05 rows unchanged; the new Section 5
  only ADDS rows and defers to the existing escalation law.

## Divergences

1. Dispatch named fix-01 as the possible alias-fix denial receipt —
   lane-fix01-core-license.md is cgap-core Cargo license metadata, NOT
   the path-alias fix; the alias fix has no receipt on disk (PENDING,
   marked in 5.1 and 5.3).
2. Dispatch named fix-04 for flood liveness — lane-fix04-cscb-p99.md is
   the csct p99-labeling repair, NOT an andon/trip-gate fix (lane-id
   collision, marked in 5.1).
3. Dispatch said SRAM flip at "6,521,032 B" — that is the riscv32imc
   linked-ELF charge; the static struct model the gate charges is
   6,520,984 B (red-12 W5 cmd 1; fix-02 pin cross-check). Both ~49.75x
   the ceiling; verdict identical.
4. Exit-101 SPC loop diagram landed as DESIGN-INTENT per red-14 (no
   latched breaker exists on disk; class-101 is a typed per-call
   refusal with zero process::exit(101) hits).
