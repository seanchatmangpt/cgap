# Lane ADV-03 — cgap-git test concurrency & shared-state corrosion

Adversarial lane over /Users/sac/cgap. READ-ONLY over crates/ (contract);
wrote only this receipt. No git commands. Build lease /tmp/adv03-target*.

## W1 Subject

Does cgap-git's test suite corrupt under concurrency (32 test-threads,
concurrent cargo binaries, CPU-hog contention) versus R-03's sequential
green (`docs/jira/v26.11.2/lanes/lane-r03-git-fence.md`)?

## W2 Commands / exits (this session, exact subject shared tree)

- Attack 1 — `CARGO_TARGET_DIR=/tmp/adv03-target cargo test -p cgap-git
  -- --test-threads=32` (attempt 1) -> exit 101: compile-abort of
  `breach_regression_court` (untracked in-flight file, another lane
  mid-edit: `use cgag_note_never as _;` at :221). COMPILE-ABORT, not a
  failure count. File was clean by the time of inspection (owner lane
  fixed under the compile-freeze SLA mid-run).
- Attack 1 (attempt 2) -> exit 101: compile-abort, `breach_regression_court`
  again (E0599 `Demux.sub`, E0433 `DemError`) — same file, still churning.
- Attack 2 — concurrent-binary storm: 4 cargo test invocations of
  ref_store_court / pack_demux_court / mcp_driver_court /
  andon_liveness_court in 4 separate /tmp target dirs, simultaneously:
  4/4 EXIT=0, zero FAILED lines, no fixture collision.
- Attack 4 — refusal-flood under contention: andon_liveness_court +
  quota_alloc_purity_court (`--test-threads=8`) while 4 /tmp/adv03-hog.sh
  spinners ran; FLOOD_EXIT=0, 2/2 `test result: ok`, hogs killed by
  pidfile (`pgrep -f adv03-hog.sh` -> clean).
- Attack 3 — determinism: after the churned file compiled, TWO full
  32-thread runs -> both exit 101 with the SAME single test failing:
  `andon_liveness_court::ingress_flood_liveness_and_quota_escalation`
  (assert "admission resets the window", andon_liveness_court.rs:53).

## W3 Fixed /tmp fixture inventory (attack-2 named paths)

- `crates/cgap-git/tests/mcp_driver_court.rs:20` —
  /tmp/cgap-git14-does-not-exist (refusal path; never created)
- `crates/cgap-git/tests/mcp_driver_court.rs:77` — /tmp/cgap-git14-court-tool
- `crates/cgap-git/tests/mcp_driver_court.rs:100` — /tmp/cgap-git14-court-lat
- `crates/cgap-git/tests/mcp_driver_court.rs` court 2 —
  `std::env::temp_dir()/cgap-git14-court-read/derived_state.bin` — fixed
  path + fixed FILENAME, but every concurrent writer writes the identical
  bytes (`SNAPSHOT-BYTES-0xA20C`), so a collision is content-benign.
- `crates/cgap-git/tests/breach_regression_court.rs:62` — /tmp/r14-leg1-target
  only as an env var value, never as a shared fixture directory.

## W4 Verdicts

- **Ordering-dependence: NOT CONFIRMED.** The sole test failure
  (`ingress_flood_liveness_and_quota_escalation`) reproduces
  single-threaded (`cargo test -p cgap-git --test andon_liveness_court --
  --test-threads=1`) with the same assert firing — it is a deterministic
  regression from another lane's in-flight semantic change to
  `crates/cgap-git/src/ingress_ring.rs` (sticky `tripped` +
  `clean_streak` hysteresis: admission no longer clears `tripped` in one
  step; `window_refusals=0` + clean-streak counter instead), while the
  court still pins the OLD semantics ("admission resets the window").
  32-thread runs and 1-thread runs fail identically => no concurrency
  channel.
- **Fixture collision: NOT CONFIRMED.** 4/4 storm exits 0; the only fixed
  /tmp writer collision is content-benign (W3).
- **R-03 perf_baseline transient: DID NOT REPRODUCE** under 32-thread
  load (0 "No such file" hits in both determinism-run logs).
- Attack 1/3 exit-101s decompose into: 2 compile-aborts (in-flight
  churn, disclosed, not failure counts) + 1 deterministic single-test
  regression owned by the ingress_ring churn lane.

## W5 Fences (must-not-regress surfaces my diff could break)

This lane wrote zero code; the only fence is receipt-only: the two
compile-aborts and the deterministic andon failure above are attributed
to in-flight lanes (breach_regression_court owner; ingress_ring owner),
NOT to cgap-git's concurrency behavior. If the ingress_ring lane lands
its hysteresis change, the andon court MUST be updated in the same
landing or the fence at
`docs/jira/v26.11.2/lanes/lane-r03-git-fence.md` (38 ok binaries) will
regress to 37-ok/1-fail deterministically — that is an in-flight owner
duty, not an ADV-03 regression.

## W6 Divergences / disclosures

- Read-only contract honored: no crates/ edits despite two opportunities
  (compile-freeze SLA unblocks on breach_regression_court were left to
  its owner, who fixed them within the SLA window mid-session).
- run_checked.py not used; exits captured un-piped per CLAUDE.md.
- Shared tree observed mid-churn by >=2 other lanes this session
  (breach_regression_court.rs, ingress_ring.rs, lib E0063
  clean_streak-window) — witness timestamps in /tmp/adv03-*.log.

## W7 Standing

**ALIVE** for every attack verdict above: each cites a command + exit run
this session against the exact shared tree. **SPOILER NOT CONFIRMED**
for test concurrency; **IMMUNITY RECEIPT** for the 4-binary storm and
CPU-hog flood (all exits 0). Clean up: /tmp/adv03-target* leases deleted
at lane end (below).
