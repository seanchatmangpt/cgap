# Lane RC-14 — quota alloc-purity court (red-09 + red-14 hardening)

Date: 2026-11-02 · Standing: **ALIVE** · Owner file: `crates/cgap-git/tests/quota_alloc_purity_court.rs` (only file written in the tree)

## W1 Subject

FIX-04's `RefusalQuota` escalation state machine had a steady-state alloc
sentry (posh_alloc_sentry_court) but nothing courted alloc purity across
the QUOTA/backoff transitions. This lane adds one new test file,
`crates/cgap-git/tests/quota_alloc_purity_court.rs`, running a
red-14-shaped refusal-flood battery through both ring seams under an
open `CountingAlloc` window.

## W2 API basis (FIX-04 landed on disk this session)

`RefusalQuota` is present in both rings: `crates/cgap-git/src/
ingress_ring.rs:96-160` and `crates/cgap-git/src/egress_ring.rs:130-199`
(fields `max_refusals=4_096` default, `window_refusals`,
`total_refusals`, `tripped`; `note_refusal`/`note_admission`/`tripped`).
Ring accessors `refusal_quota()`. **No PENDING legs** — the quota seam
was on disk before this court ran against it.

## W3 Commands + exits (this session)

- `cargo test -p cgap-git --test quota_alloc_purity_court -- --nocapture`
  (via `scripts/run_checked.py`, build root `/tmp/rc14-target`) →
  **exit 0**, `1 passed`.
- `cargo test -p cgap-git` (full crate, fence) → **exit 0**, 38 `ok`
  test-result lines. An interleaved `mutation_conflict_injection_flip ...
  FAILED` block appears mid-log: that is the EXPECTED-FAIL child
  subprocess of the octopus court (its parent test asserts the child's
  class-101 panic) — parent result `ok. 5 passed`, overall exit 0, and
  reproducible in isolation (`cargo test -p cgap-git --test
  octopus_merge_court mutation_conflict_injection_flip` → exit 0).
  Pre-existing log artifact, not a failure and not session-introduced.

## W4 Court battery (quota_flood_alloc_purity, one test fn — allocator
state is process-global)

1. Positive control FIRST: `Vec` alloc inside an open window MUST be
   observed (allocs>=1, bytes>=512) — non-vacuity of the sentry itself.
2. Ingress flood (`IngressRing<1024>` pinned exactly full, 16x64 B):
   3,000 oversized (40,000 B) pushes = `FrameTooLarge`, 2,000 overflow
   pushes = `Overflow`; quota trips at the 4,096th refusal MID-FLOOD
   (`tripped()==true`, `total_refusals==5_000`, ring `pending()==1024`
   untouched).
3. Post-flood recovery admit still in-window: pop 64 B byte-exact
   (`(i^0xA5)` fill pattern), admit 64 B `0xAB` frame, window resets
   (`tripped()==false`, lifetime counter survives at 5,000).
4. Egress flood (`EgressRing<512,64>` pinned full, 8 records): 3,000
   wrong-stride + 2,000 overflow pushes, quota trips, recovery via
   `advance_tail(1)` + admit, window resets — all in-window.
5. Close window: **allocs==0 AND bytes==0 across the ENTIRE battery** —
   both floods, both quota-escalation transitions, both recovery
   admissions. The quota is pure field math, so zero alloc is the law;
   the assert names the breach and does not suppress a measured finding.
6. Byte-exact final state: FIFO drain of the ingress ring, every byte
   checked against the frozen fill pattern / recovery frame.

## W5 Anti-vacuity + fences

- Named mutations the court kills: (a) quota escalation allocating (any
  heap use in `note_refusal`/`note_admission` path) flips Leg 5;
  (b) flood bytes landing in the ring flips Leg 2/6 (`pending()` and
  drain asserts); (c) sentry window not counting flips Leg 1 (VACUOUS
  refusal). The court itself flipped during development: a 128 B
  recovery frame against 64 B free space failed Leg 3
  (`Overflow{head:1024, tail:64, frame_len:128}`) — real referee
  behavior witnessed, then fixed in the court, then green.
- Fences run: full `cargo test -p cgap-git` exit 0 (W3). Read-only over
  `src/` — FIX-04's `ingress_ring.rs`/`egress_ring.rs` and FIX-05's
  `pack_demux.rs` untouched by this lane.
- No Process.sleep; all asserts on final state; CountingAlloc is a
  hand-written real allocator over `System`, not a mock.

## W6 Findings

- F1 (info): `pop` on an empty ring calls `note_refusal()` (ingress
  ring Empty refusal is quota-counted) — so a naive "drain until Err"
  loop inflates the audit counter; this court drains on `pending()>0`.
  Worth knowing for future quota-telemetry consumers.
- F2 (info, cross-lane log artifact): the octopus court's expected-fail
  child subprocess dumps a verbatim FAILED block into `cargo test -p
  cgap-git` logs (see W3). Easy to misread as a regression; recorded
  here so the next reader does not re-debug it.

## W7 JSON terminus

```json
{"lane":"RC-14",
 "files_written":["/Users/sac/cgap/crates/cgap-git/tests/quota_alloc_purity_court.rs",
  "/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-rc14-quota-alloc-purity.md"],
 "commands":[
  "cargo test -p cgap-git --test quota_alloc_purity_court -- --nocapture -> exit 0 (1 passed)",
  "cargo test -p cgap-git -> exit 0 (38 ok result lines, full fence)",
  "cargo test -p cgap-git --test octopus_merge_court mutation_conflict_injection_flip -> exit 0 (isolation check of the interleaved FAILED block)"],
 "courts_run":["quota_flood_alloc_purity (6 legs, exit 0)",
  "full cgap-git suite (38 binaries ok, exit 0)"],
 "standing":"ALIVE",
 "divergences":[
  "Dispatch named the quota seam PENDING-possible; it was already landed on disk (ingress_ring.rs:96-160, egress_ring.rs:130-199), so no PENDING legs",
  "Interleaved FAILED block from the octopus court's expected-fail child subprocess is a pre-existing log artifact (W3/W6-F2), not a session-introduced failure"]}
```
