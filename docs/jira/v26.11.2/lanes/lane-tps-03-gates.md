# lane-tps-03-gates

Date: 2026-11-02. Branch: main. NO git write commands run.
Files written (full ownership): `scripts/zero_datafusion_gate.py`,
`scripts/doctor_agent_lanes.py`, this receipt. Nothing else touched;
`crates/cli/src/bin/doctor.rs` untouched (ggen regen hazard, reference only).

## W1 Subject

S200-1 (cull-01 KEEP+fix): zero_datafusion_gate symlink ignore-guard.
S200-3 step 1 (cull-02): port doctor.rs c01/c03/c04/c05 into
scripts/doctor_agent_lanes.py as WARN-degrade checks with poison arms.

## W2 Diff

zero_datafusion_gate.py (2 edits):
1. copy_subject ignore set (~:194): added `"arrow-nanoarrow"` to
   `shutil.ignore_patterns` — skips the self-recursive symlink
   `vendor/nanoarrow/python/subprojects/arrow-nanoarrow -> ../..`
   (the lane-c06 ERRATA loop). One-line guard as planned.
2. UNPLANNED but required (disclosed): self_check poison injection
   appended a second `[workspace.dependencies]` table; TOML rejects
   duplicate tables, so the poison arm produced probes ERROR (exit 2),
   not HIT (exit 1) — the planned AFTER state (self-check exit 0) was
   unreachable without fixing it. Now merges `datafusion = "1"` into
   the existing section (:41) when present, falls back to append.

doctor_agent_lanes.py:
- Imports: added `shutil`.
- Ported checks appended (doctor.rs c01:186 / c03:244 / c04:252 /
  c05:261 as reference; doctor.rs untouched):
  c01-toolchain (cargo+rustc version probe), c03-git (rev-parse in
  CGAP_ROOT), c04-b3sum (which+--version), c05-pyshacl (import probe).
  All missing/broken = WARN, never RED (cull-02 DXD-02 degrade law).
- main() runs them after the 4 DX-1 checks; exit ABI 0/1/2 unchanged.
- doctor.rs c10 (dirty-tree) NOT ported — DIES per plan; subsumer:
  pre-existing `check_dirty_tree` census (check (d), same
  `git status --porcelain` input, richer per-dir output).

## W3 Commands / exits (this session)

- `python3 scripts/zero_datafusion_gate.py --self-check` BEFORE:
  exit 1 — shutil.Error traceback, copytree infinite recursion in the
  arrow-nanoarrow symlink loop (witnessed pre-fix, transcript above in
  session log; 707KB persisted output).
- `--self-check` AFTER guard, pre-poison-fix: exit 3 VACUOUS —
  `poison probes={'metadata': 'ERROR', 'tree': 'ERROR', 'literal': 'HIT'}`
  (latent duplicate-table defect surfaced).
- `--self-check` AFTER both: exit 0, `ZERO-DATAFUSION: PASS`.
- plain scan: exit 0, `ZERO-DATAFUSION: PASS`.
- `python3 scripts/doctor_agent_lanes.py` before: exit 1; after: exit 1.
  DX-1 4-check output diff: only orphan-leases byte count moved
  (4901.7 -> 4978.5 MB, same 11 dirs) — a concurrent lane minted a
  /tmp lease dir between the two runs; live-state drift, not a
  behavior change. stale-server / workspace-count / dirty-tree lines
  byte-identical.
- Poison arms (each flips its check to WARN):
  `DX1_POISON_C01=1` -> `c01-toolchain WARN poison arm`;
  `DX1_POISON_C03=1` -> `c03-git WARN poison arm`;
  `DX1_B3SUM=/nonexistent/b3sum` -> `c04-b3sum WARN override not found`;
  `DX1_PYTHON3=/nonexistent/python3` -> `c05-pyshacl WARN not available`.
  Stripped-PATH arm (`env -i PATH=/usr/bin:/bin`): c01 WARN, c04 WARN;
  c05 remained PASS (system /usr/bin/python3 has pyshacl on this host)
  — the c05 flip is the interpreter-override arm, recorded honestly.
- `bash .claude/hooks/flip-tests.sh`: 9/9 PASS, exit 0.

## W4 Fences run

- tests/gates/test_gate_exits.py: 29 passed, 1 failed. The 3
  zero_datafusion pins (nominal exit 0 :66, missing-root exit 2 :101,
  poisoned-workspace exit 2 :167) all PASSED on my diff. The failure
  is `test_gap6_zero_string_dep_direction_emitter_exempt_lines` —
  subject `scripts/zero_string_gate.py`, ANOTHER lane's in-flight
  1.4.2 normalization edit (57 insertions vs HEAD, last commit
  6801affa); its `--dep-direction` run GATE-REFUSED (exit 1) on
  `sections-unreadable /Users/sac/.cache/tmp/.../lib.rmeta_ (objdump
  -h yielded no sections)` — stale build residue in the shared
  target, unrelated to my diff. Not a session-introduced regression
  of my files; owner lane must reconcile (compile-freeze-style
  disclosure to that owner).
- flip-tests.sh 9/9 (above).
- S200-1 plan row "flip tests/gates pin `--self-check == 1` to `== 0`":
  NOT done — tests/gates/test_gate_exits.py is not in my owned file
  list and no pin on zdf self-check exists on disk (grep: only
  zero_string + boundary self-check pins; zdf rows are nominal/
  missing-root/poisoned-workspace, already exit-0/2/2 expectations).
  Plan row is vacuous against the live file; recorded as divergence,
  no edit needed, fence green.

## W5 Standing

ALIVE for both S200-1 and S200-3 step 1 (all commands + exits above
run this session against the exact on-disk subjects).
Divergences recorded:
- Poison injection fix (W2 item 2) beyond the one-line plan.
- test_gate_exits.py pin-flip plan row is vacuous (W4).
- c05 stripped-PATH arm does not flip on this host (W3).
- gate-exit test failure in another lane's in-flight file (W4 fence
  note; disclosed to the owner lane via this receipt).
- S200-4 not in scope this lane (c06 workspace-check port + budget
  remain open, per plan S200-4).
