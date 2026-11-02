# Lane G10 — flip-tests reconcile receipt (2026-11-02)

Lane: G10, repo /Users/sac/cgap, branch main. Files written:
`.claude/hooks/flip-tests.sh` (edited, defect found),
`docs/jira/v26.11.2/lanes/lane-g10-flip-tests-reconcile.md` (this receipt).

## Verdict

Agent 13's early 8/9 was a REAL defect, witnessed and mechanically
reproduced this session — not stale target artifacts, not cargo lock
contention. Two colliding surfaces, both concurrency-shaped:

1. **Census-vs-census collision (primary).** `branchless_census.rs:57`
   pins a FIXED shared `WORK=/tmp/rep1-census` with fixed filenames
   (`obj_host.o`, `obj_riscv.o`, `mut_indep.o`, ...). Two concurrent
   census invocations (e.g. a lane running `cargo test -p
   cgap-branchless` while another runs `cargo test --workspace
   --all-targets`) race on those files; witnessed exact panic:
   `error: could not copy "...obj_host.wrapper_host.*-cgu.0.rcgu.o" to
   "/tmp/rep1-census/obj_host.o": No such file or directory` → all 3
   fns panic → rc=101 → flip-tests false [FAIL]. Reproduced
   deterministically: 4 rounds of two concurrent census runs gave
   a=101/b=101 x3, a=0/b=101 x1; single-instance loops ran 12/12 ok.
2. **Mid-edit shared tree (secondary, correct fail-loud).** The census
   compiles live shared-tree module sources by absolute `#[path]`
   (`branchless_census.rs:56,133`). A concurrent lane mid-editing
   `scan.rs` (E0658 const-trait error, later self-fixed) fails the lib
   compile → census rc=101. This arm is the gate working as designed.
3. **Hook-vs-hook target race (found while fixing).** The hook's
   `CENSUS_TARGET` was a FIXED shared path; one instance's
   end-of-run `rm` deletes another instance's target mid-census
   (witnessed: `error writing dependencies to .../c8-flip-census-
   target/...: No such file or directory` during a 3x battery).

## Hook edit (justified, defect-supported)

`.claude/hooks/flip-tests.sh`, census arm (lines 76-100 region):
- `CENSUS_TARGET` now `mktemp -d`-unique per invocation (kills the
  hook-vs-hook rm race).
- Census invocation wrapped in a BLOCKING flock on
  `${TMPDIR:-/tmp}/rep1-census.lock` via a perl one-liner
  (`flock LOCK_EX`, `shift @ARGV` then `exec`; no sleep-polling —
  kernel wait). This serializes census invocations that take the
  lock (hook-vs-hook). NOT a retry: a genuine census failure still
  exits nonzero inside the lock.
- One self-inflicted bug during the edit, caught by the gate itself:
  the first wrapper exec'd the lock file as the program (missing
  `shift`); perl's ENOEXEC fallback ran it as an empty shell script —
  silent rc=0 with zero output, which the gate correctly read as
  `[FAIL] passed='0'` (compile-abort-vs-failure law: an empty capture
  never reads as pass).

## Divergence (outside lane ownership)

The primary surface is NOT fully fixed by the hook: direct
`cargo test -p cgap-branchless` / `cargo test --workspace` invocations
from other lanes do NOT take the hook's lock. Full fix is one line in
`crates/cgap-branchless/tests/branchless_census.rs:57` (NOT lane G10's
file): make `WORK` per-process, e.g.
`std::env::temp_dir().join(format!("rep1-census-{}", std::process::id()))`.
Recommended for the owning lane. Until it lands, the census gate can
still false-[FAIL] under concurrent foreign census load — the receipt's
9/9 runs below were witnessed with no foreign census process active
(foreign `cargo test --workspace` PID 33638 had exited; only `cargo
test -p cgap-git` remained, which does not touch cgap-branchless).

## Commands + exits (this session)

- `bash -n flip-tests.sh` → 0 (syntax, after each edit)
- Pre-fix batteries (documented failures, all rc captured):
  battery1 9/9 → 8/9(rc1) → 8/9(rc1); battery2 (after bounded mtime-
  settle poll) 9/9 → 8/9 → 8/9; battery3 9/9 → 8/9 → 9/9; battery4
  (hook logs /tmp/g10-hook-{1..3}) rc 0/1/0; battery5 rc 1/1/1 with
  mid-run target-deletion signature.
- Concurrent-census witness: 4 rounds two-instance → a=101/b=101 x3,
  a=0/b=101 x1 (panic text quoted above).
- Single-instance isolation: 6/6 ok, then 12/12 ok census loop.
- **Post-fix: 5 consecutive flip-tests.sh runs, all 9/9 exit 0**
  (batteries: 3 + 2 runs 4-5), census arm
  `[PASS] rep1-census-gate rc=0 passed=3` in every run.

## Anti-vacuity witness

- Hook-level: the publish-guard sandbox-mutation arm ran and PASSED in
  every battery (mutant flips block(2)→allow(0); sed-neutered copy,
  sha256 before/after compared — shared tree untouched).
- Census-level: the always-run mutation arm
  (`census_mutation_and_selfcheck_always_run`, M-REP1-1 + M-INL-1 +
  branchy-reference self-check) passed in every green run and
  PANICS fail-loud on real corruption — witnessed live in the
  collision reproductions above (compile-abort panics → rc=101 → gate
  [FAIL], never a silent pass).
- Gate flips when the court breaks: every witnessed corruption
  (mid-edit tree, file race, exec bug) produced [FAIL] with
  nonzero exit. No silent-pass path exists: `passed='0'` on rc=0
  reads [FAIL].

## Standing

ALIVE — 5 consecutive 9/9 exit-0 runs of the exact post-edit hook this
session; collision mechanism witnessed with exact panic text; post-fix
cleanup done (all /tmp/g10-* build-root leases deleted via python
shutil.rmtree). Must-not-regress fences: the 8 pre-existing hook courts
(publish-guard x2 + sandbox-mutation, cwd-guard x3, mtime-restore-guard
x2) all PASS in every post-fix run; hook is fail-loud (no retry arm
added).
