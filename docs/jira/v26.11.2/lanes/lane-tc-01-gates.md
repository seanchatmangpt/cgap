# lane-tc-01-gates

## W1 Identity

- Lane TC-01, repo `/Users/sac/cgap` (main, no git transitions).
- Subject: `scripts/zero_string_gate.py` (DOC_EXECUTION_FAMILY retarget,
  cull-05), `scripts/zero_datafusion_gate.py` (verify-and-receipt only),
  this receipt. Working tree, post-wave source (HEAD 9a730320 + in-flight
  lane work).

## W2 Authority

- Dispatch: drop ghost keys "cli"/"integration" from DOC_EXECUTION_FAMILY
  per cull-05 RE-TARGET; verify TPS-3 symlink guard; fences boundary_gate
  --deps and the floor checker; receipt with before/after exits and flips.
- Owned files: `scripts/zero_datafusion_gate.py`,
  `scripts/zero_string_gate.py`, `docs/jira/v26.11.2/lanes/lane-tc-01-gates.md`.

## W3 Commands+exits

| # | Command (cwd /Users/sac/cgap) | Exit | Witness |
|---|---|---|---|
| 1 | `python3 scripts/zero_datafusion_gate.py --self-check` | 0 | TPS-3 landed; guard present at copy_subject :194-199 |
| 2 | `python3 scripts/zero_datafusion_gate.py` (plain) | 0 | ZERO-DATAFUSION: PASS |
| 3 | `python3 /tmp/tc01-fix/zdf_pre_guard.py --self-check --root /Users/sac/cgap` (sandbox copy, guard reverted) | 1 | shutil.Error arrow-nanoarrow recursion — lane-c06 ERRATA pathology reproduced in /tmp; 1->0 flip witnessed |
| 4 | `python3 scripts/zero_string_gate.py --self-check` | 0 | SELF-CHECK RESULT PASS (also 0 pre-edit) |
| 5 | `python3 scripts/zero_string_gate.py` (plain scan) | 1 | typed refusal sections-unreadable — PRE-EXISTING, HEAD version exits 1 identically |
| 6 | `python3 scripts/zero_string_gate.py --courts` | 0 | COURTS-SUMMARY total=13 failed=0 |
| 7 | `python3 scripts/boundary_gate.py --deps` | 0 | DEPS GATE E=0 across 15 crates |
| 8 | `python3 scripts/bench_gate.py` (floor checker) | 0 | GATE PASS, margins +80-90% |
| 9 | fixture: real `cmd_scan` over /tmp fixture ws member `cli`, zero artifacts | both | PRE=execution-crate-no-artifacts cli / POST=crate-no-artifacts cli |

## W4 Consequence (diff)

- `zero_string_gate.py` DOC_EXECUTION_FAMILY :451-461: dropped ghost keys
  `"cli"`, `"integration"`; kept cgap-cli/cgap-integration and all 9 real
  package names (cull-05 §2). Comment block updated (cull-05 RE-TARGET,
  TC-01). No behavioral key was live: no package named cli/integration
  exists as a member, so the pre-edit set never matched them (the
  fixture flip below is the class-membership witness, not a production
  behavior change).
- `zero_datafusion_gate.py`: NOT edited by this lane. Working-tree
  diff vs HEAD (symlink ignore-guard + poison-merge fix) pre-existed
  this session (TPS-3); this lane verified only.

## W5 Courts table

| Court | Mutation (anti-vacuity) | Flip witness | Exit |
|---|---|---|---|
| zdf symlink-guard self-check | guard reverted in /tmp copy (arrow-nanoarrow removed from ignore_patterns) | pre=1 shutil.Error recursion; post=0 PASS | 1->0 |
| zsg T16 fixture (family membership) | fixture member pkg named `cli`, zero artifacts, real cmd_scan | PRE refusal class execution-crate-no-artifacts; POST crate-no-artifacts | class flip |
| zsg --courts suite (13 courts incl. T16 arms 1-5) | built-in mutations M-E1..M-E5 classes | COURTS-SUMMARY failed=0 | 0 |

The retarget flip honestly read: dropping "cli"/"integration" moves those
names OUT of the doc-execution-family class — a fixture package named
`cli` was treated as an execution-family member pre-fix (obligated
must-have-artifacts) and as an ordinary member post-fix. Residual risk
is covered by the no-dead-entries law: a producer-grant key naming
`cli`/`integration` resolves no discovered artifact (standalone/non-member),
so `emitter-exempt-unresolved` still refuses it end-to-end (T16 arm 5 leg B,
witnessed green in the --courts run).

## W6 Standing

ALIVE for the retarget (command+exit this session, exact subject).
zero_datafusion symlink guard: ALIVE via verify-and-receipt (guard
present :194-199, self-check 0, pre-guard sandbox flip 1->0). Plain
zero_string scan: exit 1 is a typed PRE-EXISTING refusal, not green —
not rounded up.

## W7 Divergences/disclosures

1. Dispatch expected zero_string plain scan "exit 0 unchanged"; reality:
   exit 1 both pre- and post-edit with typed refusal
   `sections-unreadable .../.cache/tmp/a07_ar.*/lib.rmeta_` (objdump no
   sections) — a07 lane scan-cache debris; HEAD version reproduces the
   identical refusal. NOT caused by the retarget. Also note the
   working-tree file carries uncommitted lane-12 exit-ABI work (v1.4.2,
   SUBJECT_MISSING_CLASSES, exit-3 loud-skip arms) from another lane;
   this lane's hunk is disjoint (:451-461 only).
2. Dispatch expected the floor checker "still exit 1 (honest red)";
   reality: `python3 scripts/bench_gate.py` exits 0 (GATE PASS, floors
   +80-90% above floor). No red to preserve. If the dispatch meant a
   different floor checker, none named "floor" exists in scripts/
   outside bench_gate.py.
3. TPS-3 disclosure: the zero_datafusion symlink guard + poison-merge
   fix landed in this lane's owned file by another lane before TC-01
   started; per dispatch this lane verified-and-receipted without
   re-editing; sha256 on disk
   f158524ca18c74f9ce94781deccbd283465d5beb2ac828f7f9a2d0e585efe426).
4. Shared-tree mutation check: only the one Edit to zero_string_gate.py
   (:451-461 + comment) — write-integrity hook flagged STRAY_TOKENS on
   the edit; on-disk bytes re-verified (ast.parse OK, diff clean).

Fences that could regress and their results: zero_string self-check 0,
--courts 13/13, boundary_gate --deps 0, bench floor gate 0 — all green
post-edit.
