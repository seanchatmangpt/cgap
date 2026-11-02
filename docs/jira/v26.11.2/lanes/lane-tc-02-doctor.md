# lane-tc-02-doctor

Date: 2026-11-02. Branch: main. NO git write commands run.
Files written (full ownership): `scripts/doctor_agent_lanes.py`,
`crates/cli/src/bin/doctor.rs`, this receipt. Nothing else touched.

## W1 Subject

CULL-02 completion (per `audit/cull-02-doctor-consolidation.md` +
`audit/dxd-02-doctor-design.md`): scripts/doctor_agent_lanes.py is the
ONE canonical doctor; cli doctor.rs demotes to a redirect stub.
TPS-3 fresh-read first: lane-tps-03-gates.md confirms c01/c03/c04/c05
already ported WARN-degrade with poison arms — verified on disk
(doctor_agent_lanes.py:162-257) — and c10 already DIES (never ported;
subsumed by the pre-existing dirty-tree census, check (d) at :142).
Task (2) therefore reduced to witnessing the fence (W3).

## W2 Diff

doctor_agent_lanes.py (401 lines, was 274):

1. c06-workspace-check (doctor.rs:285 port) — FLAG-GATED behind
   `--full` per cull-02 open question 2 (runtime vs DXD-02 ~5s budget).
   Absent flag -> loud `SKIP` line naming the budget, mapped to exit 0
   (loud, never silent; SKIP->0 keeps the fence exit 0/1-informational).
   Under `--full`: real `cargo check --workspace`, WARN-degrade on
   nonzero exit or warnings (never RED). Poison arm DX1_POISON_C06=1.
2. c07-scripts (doctor.rs:337 port) — WARN-degrade (cull-02 open
   question 1 resolved: WARN-degrade, not park), DX1_CGAP_ROOT-relative
   10-name list. Missing OR not-executable -> WARN. Poison arm: any
   DX1_CGAP_ROOT lacking scripts/ -> WARN.
3. c08-receipts (doctor.rs:395 port) — WARN-degrade,
   DX1_CGAP_ROOT-relative; v26.10.31 frozen receipts path; staleness =
   newest crates/ mtime > newest receipt mtime -> WARN.
4. c09-lsl-ratified (doctor.rs:449 port) — WARN-degrade,
   DX1_CGAP_ROOT-relative; parses `const LSL_RATIFIED: Option<f64>`.
   None/missing -> WARN; `Some(...)` -> PASS.
5. main() runs the 12 checks (4 DX-1 + 4 TPS-3 ports + 4 TC-02 ports);
   exit ABI 0/1/2 unchanged; SKIP->0 added to the exit map.

crates/cli/src/bin/doctor.rs (663 -> 25 lines): main() replaced by a
5-line redirect stub exec'ing
`python3 /Users/sac/cgap/scripts/doctor_agent_lanes.py` (arg-forwarding
via skip(1), exit code mirrored, spawn failure -> panic with named
remediation). Header names DXD-02 + CULL-02 and records the ggen-provenance
witness (below).

## W3 Commands / exits (this session)

- Fresh-read TPS-3: `grep` + read of lane-tps-03-gates.md — c01/c03/c04/c05
  ports confirmed on disk at doctor_agent_lanes.py:162-257. Verified, not
  re-ported.
- `python3 scripts/doctor_agent_lanes.py` BEFORE: exit 1 (baseline
  captured /tmp/tc02-before.txt).
- AFTER (no flag): exit 1. DX-1 4-check lines byte-identical (diff
  before/after shows ONLY appended c06-c09 lines; zero changed lines in
  the DX-1 block). Fence witness: exit stays 0/1-informational.
- `python3 scripts/doctor_agent_lanes.py --full`: exit 1;
  `c06-workspace-check WARN exit 0 with 15 warning(s)` — real subject,
  real flip.
- Poison arms (each names its flip):
  `DX1_POISON_C06=1 ... --full` -> `c06-workspace-check WARN poison arm`.
  `DX1_CGAP_ROOT=/tmp/tc02-poison-root` (mkdir -p'd empty dir) ->
  `c07-scripts WARN missing: <all 10>`, `c08-receipts WARN receipts dir
  unreadable`, `c09-lsl-ratified WARN ctq_report.rs unreadable` — each
  flips; /tmp sandbox, never the shared tree; cleaned up after
  (shutil.rmtree). Note: the poisoned-root run also showed
  workspace-count RED (pre-existing DX-1 cargo-metadata behavior under
  a nonexistent-project root, expected, unchanged by this lane).
- `cd crates/cli && cargo build --bin doctor`: exit 0 (Finished dev
  profile). `./target/debug/doctor`: prints the full 12-check table,
  mirrors exit 1 — redirect ALIVE end-to-end.
- `CLAUDE_PROJECT_DIR=/Users/sac/cgap bash .claude/hooks/flip-tests.sh`:
  exit 0, 9/9 PASS (publish-guard fire/quiet + sandbox-mutation,
  cwd-guard x3, mtime-restore-guard x2, rep1-census-gate passed=3).

## W4 Verification ladder

Narrow (poison-arm env flips per check) -> integration (stub binary
execs the Python surface, mirrors exit) -> host gate (flip-tests 9/9).
No Process.sleep anywhere; exits observed directly.

## W5 Standing

ALIVE for: c06/c07/c08/c09 ports (real + poison flips witnessed this
session), redirect stub (built + smoke-run exit 1), flip-tests 9/9,
DX-1 byte-identical fence. doctor.rs c10 deletion: N/A-already-done by
TPS-3 (witnessed via receipt re-read + on-disk check, not re-executed).

## W6 Fences

- flip-tests.sh (9/9 exit 0, run this session — the diff touches no
  hook, but the dispatch names it as the regression fence).
- Root `cargo build --workspace` NOT run (cli crate is a standalone
  workspace; doctor.rs is a cli bin, not a root member — cli crate
  built instead, exit 0).
- ggen regen: ggen.toml + gen-templates/ grep for `doctor` = zero
  matches; repo CLAUDE.md:51 names doctor a hand-maintained side-bin.
  Stub landed anyway per dispatch; recorded: it MAY be clobbered if a
  future pack projects bin/doctor.rs — the stub's spawn-panic header
  makes that loud, never silent.

## W7 Divergences

1. Task (2) "delete the c10 check" was already executed by TPS-3
   (lane-tps-03-gates.md: "doctor.rs c10 NOT ported — DIES per plan");
   TC-02 witnessed the subsumer instead of re-deleting. Receipt cites
   the unblocker (disclose-in-both-directions).
2. SKIP exit mapping: dispatch said "exit stays 0/1-informational";
   SKIP is mapped to 0 so a gated c06 cannot push exit past 1 — a
   loud-skip-as-0 reading of that instruction, recorded here.
3. Stub hardcodes the absolute script path (no repo-root walk) — the
   old doctor.rs had a repo_root() walk; the redirect's contract is a
   fixed canonical subject per DXD-02.
