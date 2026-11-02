# f3-10-pub-arrow

Wave F3 item 10 — PUB-ARROW lane receipt. Repo `/Users/sac/cgap`. Publish
target: `cgap-arrow` 26.11.1. Lease honored: this marker is the only file
written. **Zero crates.io publish attempts** — the one-attempt immutability
budget is intact for re-dispatch.

## Standing

BLOCKED by chain law at 2026-11-02T00:44:00Z. Upstream
`f3-09-pub-blake3.md` (poll FOUND at 2026-11-02T00:40:55Z, inside the
bounded ~20 min / 30s-cadence window opened ~00:20:41Z) carries standing
**BLOCKED**: its `f3-03` gate marker never landed, so
`cargo publish -p cgap-blake3` was never invoked (budget intact there
too). BLOCKED there = BLOCK here; `cgap` umbrella depends on blake3
`^26.11.1`, so arrow-first publish would strand the facade.

## Gate evidence (this session)

- `crates/cgap-arrow/Cargo.toml:3` `version.workspace = true` (grep
  exit 0) + root `Cargo.toml:39` `version = "26.11.1"`;
  `cargo metadata --no-deps` exit 0 -> cgap-arrow 26.11.1 (deps:
  cgap-core/gate/storage, dev kind only; zero normal deps).
- f3-08 registry diff: zero pre-existing 26.11.1 on any of the five
  crates -> no immutable-version collision for this slot.

## Commands and exits

| command | exit |
|---|---|
| bounded poll for `f3-09-pub-blake3.md`, 30s cadence, ~20 min window | 0 (FOUND 2026-11-02T00:40:55Z; round 1 killed at harness 10-min background limit, restarted with remaining budget — disclosed) |
| standing parse of f3-09 content | BLOCKED (content verdict, not an exit code) |
| `grep -n '^version\|^name' crates/cgap-arrow/Cargo.toml` | 0 (line 3 `version.workspace = true`) |
| `grep -n version Cargo.toml` (root) | 0 (line 39 `version = "26.11.1"`) |
| `cargo metadata --no-deps --format-version 1` | 0 (cgap-arrow 26.11.1) |
| DIAGNOSTIC `CARGO_TARGET_DIR=/tmp/f3-10-target cargo publish --dry-run -p cgap-arrow --allow-dirty` | 0 (Packaged 38 files; `Verifying cgap-arrow v26.11.1`; `warning: aborting upload due to dry run`; 2026-11-02T00:43:24Z) |

## Publish step

NOT EXECUTED. `cargo publish -p cgap-arrow --allow-dirty` was never
invoked: the chain gate is red. No API max_version poll for 26.11.1 was
run as step 5 (nothing published). Registry baseline read at
2026-11-01T06:30:48Z + rechecks through 00:42:34Z: cgap-arrow
`max_version 26.10.29`, `num_versions 1` — unchanged.

## Retry-After

Carried from f3-09: 900 s from 2026-11-02T00:40:02Z -> retry at/after
**2026-11-02T00:55:02Z**. Re-dispatch should: confirm `f3-03` landed and
reads green; confirm f3-09 updates to ALIVE (API
`cgap-blake3 max_version == "26.11.1"`); then run this lane's steps 3-5
in order (dry-run gate -> publish ONCE -> API poll
`https://crates.io/api/v1/crates/cgap-arrow`, UA `cgap-publish-receipt`).
The diagnostic dry-run above already exits 0 at 26.11.1.

## Divergences

1. Diagnostic dry-run executed after the red chain gate (out of dispatch
   order), mirroring f3-09's disclosed diagnostic; it is packaging
   evidence, NOT step-3 pass evidence for a green run.
2. Poll round 1 was killed at the harness 10-min background limit before
   its 20-min internal deadline; restarted with the remaining budget —
   both rounds together stayed inside one ~20-min window.
3. crates.io showed cgap-blake3 `max_version 26.10.29` at every check
   through 00:42:34Z, consistent with f3-09's not-executed publish.

## Re-dispatch (R3 chain position 2, 2026-11-02T02:05:25Z)

Standing: **BLOCKED by chain law at 2026-11-02T02:05:25Z**. One bounded
poll (30s cadence, 35-min window, rounds 1-70, 2026-11-02T01:20:48Z ->
01:55:48Z, exit 2 POLL-DEADLINE):

- Gate A (`f3-09-pub-blake3.md` re-dispatch section reading ALIVE with
  API-verified 26.11.1): **RED all 70 rounds**. The file was never
  updated; re-read at 02:01:14Z it still carries standing BLOCKED
  (marker gate, 00:39:13Z) and no `## Re-dispatch` section exists.
- Gate B (`r13-quad64.md` standing ALIVE): **RED all 70 rounds**. The
  file landed at 01:25:49Z (mid-window) but carries **no standing line
  at all** (Change/Verification/Cleanup only) - see Divergences.
- Registry at 02:01:14Z: cgap-blake3 `max_version 26.10.29` (num 1),
  cgap-arrow `max_version 26.10.29` (num 1) - gate A red corroborated
  API-side; zero 26.11.1 anywhere; no immutable-version collision for
  this slot.

## Re-dispatch commands and exits

| command | exit |
|---|---|
| bounded poll A+B, 30s x 70 rounds, 01:20:48Z->01:55:48Z | 2 (POLL-DEADLINE) |
| `sed -n '39p' Cargo.toml` | 0 (`version = "26.11.1"`) |
| `grep -n version crates/cgap-arrow/Cargo.toml` | 0 (line 3 `version.workspace = true`) |
| baseline curl cgap-arrow (UA cgap-publish-receipt) | 0 (`max_version 26.10.29` @ 01:20:06Z) |
| `CARGO_TARGET_DIR=/tmp/r3-target cargo publish --dry-run -p cgap-arrow --allow-dirty` (pre-R13 subject, ~01:21Z) | 0 (Packaged 38 files 408.6KiB; `Verifying cgap-arrow v26.11.1`; aborting upload) |
| same dry-run re-run on post-R13 subject @ 02:04Z | 0 (Packaged 38 files 408.3KiB; aborting upload; exit captured clean) |
| `cargo publish -p cgap-arrow --allow-dirty` | NOT EXECUTED - never invoked; no exit code minted; one-attempt budget intact |

## Publish step (re-dispatch)

NOT EXECUTED. `cargo publish -p cgap-arrow --allow-dirty` was never
invoked this re-dispatch: gate A red (f3-09 still BLOCKED, blake3
26.10.29 on the registry) blocks the chain by law, and gate B's named
"standing ALIVE" wording is unsatisfied in `r13-quad64.md`. No API
max_version poll for 26.11.1 was run as step 5 (nothing published).

## Retry-After (re-dispatch)

900 s from 2026-11-02T02:05:25Z -> retry at/after **2026-11-02T02:20:25Z**.
Next re-dispatch should: (1) confirm the f3-09 re-dispatch lands ALIVE
with API `cgap-blake3 max_version == "26.11.1"`; (2) read `r13-quad64.md`
green-on-content (its verification legs are all exit 0) or have R13 add a
standing line; (3) the 02:04Z dry-run here is already exit 0 on the
post-R13 subject - re-run only if cgap-arrow source changes again;
(4) publish ONCE, then API-poll `https://crates.io/api/v1/crates/cgap-arrow`
(UA `cgap-publish-receipt`) for `max_version == "26.11.1"`; (5) append
the next re-dispatch section and delete the lane target dir.

## Divergences (re-dispatch)

1. Gate B wording mismatch: `r13-quad64.md` defines no standing field;
   the poll's standing-grep read RED, content is green (178 tests pass,
   exit 0, exactly 1 `struct Quad64`, dedupe landed via `pub use
   crate::layout::Quad64;` in transpose.rs). Gate A red alone forces the
   BLOCK either way, so no publish decision hinged on this reading.
2. Mid-window interleave: R13 edited `crates/cgap-arrow/src/transpose.rs`
   at 01:23:29Z, staling this lane's 01:21Z dry-run; re-run at 02:04Z on
   the post-R13 subject: exit 0. Both runs disclosed.
3. `--allow-dirty` used on both dry-runs (dirty tree, disclosed per
   dispatch); zero publishes attempted, so no published artifact carries
   it.
4. Prior receipt's poll kill at the harness 10-min background limit did
   not recur: this re-dispatch's poll ran the full 35 min in one
   background task (35-min timeout sized to the window).
5. Cleanup denial: `rm -rf /tmp/r3-target` denied by the permission
   system (2 attempts, incl. combined-with-append); the lane build root
   remains on disk (~408KiB package debris + debug build). Coordinator
   must delete `/tmp/r3-target` (and R13's `/tmp/r13-target`) at
   integration per the cleanup law.

## Witness (W2-2-witness-arrow, gate-runner, 2026-11-02)

- Who: lane W2-2-witness-arrow (chain witness, gate-runner; no publish
  authority, zero publish attempts from this lane, no git writes).
- Baseline: registry read 2026-11-02T02:04:18Z — cgap-arrow
  `max_version 26.10.29`, `num_versions 1`.
- Poll (task item 1): 30s cadence, window 02:04:30Z -> 02:44:52Z (4
  foreground legs, ~40-min contract budget). f3-09 standing REFUSED and
  f3-10 standing BLOCKED at every round — no marker flip in-window.
  Disclosure: leg-4 mtime column degraded by a stat typo; the standing
  tripwire columns stayed clean all rounds.
- Registry truth (task item 2, reads 02:46:10Z + 02:52:17Z, UA
  cgap-publish-receipt): cgap-arrow `max_version 26.11.1`,
  `num_versions 2 EXACTLY`, versions [26.11.1, 26.10.29]; version
  record `/crates/cgap-arrow/26.11.1` created_at
  2026-11-02T02:17:56.021242Z, yanked false, published_by
  seanchatmangpt. num_versions == 2 — no double upload; one-attempt
  law held.
- Chain order witnessed: blake3 26.11.1 created 02:15:49.374566Z ->
  arrow 26.11.1 created 02:17:56.021242Z (+2m07s; coord-publish-record
  order blake3 -> arrow; arrow publish resolved blake3 ^26.11.1 from
  the live index). "f3-09 first, then f3-10" holds at the registry
  layer; the marker standing lines are stale-not-false, not contradicted.
- Cross-check (task item 3): f3-10's claims (last write 02:08:17Z:
  publish NOT EXECUTED, registry 26.10.29/1 @ 02:01:14Z) were true at
  write time; the upload postdates the marker's last write, so BLOCKED
  here is stale-not-false. Ground truth: coord-publish-record.md
  (ALIVE, all five crates 26.11.1, coordinator-side per operator
  standing order after permission-system refusals of subagent publish
  authority) + the registry records. No contradiction, no FAIL named.
- Standing: **ALIVE** — cgap-arrow 26.11.1 published, witnessed
  2026-11-02T02:52:17Z.
- Divergences: (1) this marker's Standing/BLOCKED text is superseded
  by the registry + coord record and awaits an actuating-path
  amendment; (2) leg-4 stat typo as disclosed; (3) lane build root
  /tmp/r3-target still on disk per the prior receipt's cleanup denial —
  coordinator cleanup at integration.
