# f3-15-api-verify

Wave F3 item 15 — API-VERIFY (court-auditor). Repo /Users/sac/cgap (canonical
checkout). Independent of publishers' claims: every registry fact below was
fetched by this lane. All times UTC 2026-11-02. UA for every curl:
cgap-publish-receipt. Only file written by this lane: this marker.

## Standing

AUDIT EXECUTED — VERDICT: FAIL (typed, all five crates). The audited subject
(crates.io 26.11.1 publish) does not exist. Bounded poll for
f3-13-pub-umbrella.md: marker landed 2026-11-02T00:44:30Z (poll attempt 47 of
50, 30s cadence, window ~00:16Z->00:46Z) but contains ZERO ALIVE tokens
(grep -c ALIVE exit 1); its standing is BLOCKED ("publish NOT attempted").
Required ALIVE: NOT MET. Registry acceptance (max_version == 26.11.1 exact):
FAILED for all five crates at two independent sweeps 28 min apart
(00:20:42Z and 00:49:02Z).

## Verdict table (final sweep 00:49:02Z; sweep 1 @ 00:20:42Z identical)

| crate        | API max_version | index vers  | yanked | downloads | verdict              |
|--------------|-----------------|-------------|--------|-----------|----------------------|
| cgap-blake3  | 26.10.29        | [26.10.29]  | false  | 13        | FAIL: 26.11.1 absent |
| cgap-arrow   | 26.10.29        | [26.10.29]  | false  | 13        | FAIL: 26.11.1 absent |
| cgap-git     | 26.10.29        | [26.10.29]  | false  | 13        | FAIL: 26.11.1 absent |
| cgap-parquet | 26.10.29        | [26.10.29]  | false  | 13        | FAIL: 26.11.1 absent |
| cgap         | 26.10.29        | [26.10.29]  | false  | 9         | FAIL: 26.11.1 absent |

Only version in registry per crate: 26.10.29 (versions list length 1,
created 2026-11-01T06:30-06:54Z). 26.11.1 absent from API versions lists and
from every sparse-index line (index.crates.io/cg/ap/<name>) at both sweeps.
yanked=false and downloads>=0 hold trivially for the sole present version.

## Commands + exits (this lane, all executed)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | initial marker census: ls+grep waves/ for f3-13/f3-15 | 1 | f3-13 absent at start |
| 2 | git status --porcelain (read-only ownership snapshot) | 0 | dirty tree = other lanes; no f3-15 files |
| 3 | bounded poll 30s x 50 / 25 min for f3-13-pub-umbrella.md | 1 | POLL_TIMEOUT; absent attempts 0-46, present-no-ALIVE 47-49 |
| 4 | curl x5 crates.io/api/v1/crates/<name> @00:20:42Z | 0 x5, HTTP 200 x5 | bytes 2669-2758 saved /tmp/f315_api_*.json |
| 5 | curl x5 index.crates.io/cg/ap/<name> @00:20:46Z | 0 x5, HTTP 200 x5 | NDJSON saved /tmp/f315_idx_*.txt |
| 6 | python3 verdict parser over the 10 responses | 0 | all five FAIL |
| 7 | grep local versions: kernels version.workspace=true; root Cargo.toml:39 = 26.10.29; crates/cgap/Cargo.toml:12-15 pins 26.11.1 | 0 | pin claim verified on disk |
| 8 | re-sweep curl x10 (API+index) @00:49:02Z | 0 x10, HTTP 200 x10 | identical FAIL state |
| 9 | mutation probes: nonexistent crate name via API + index | 0 (curl) HTTP 404 + 404 | court FLIPS; parser non-vacuous |
| 10 | stat mtimes f3-09/10/11/13/20 markers | 0 | timeline pinned (see divergences) |

## Publisher-claims comparison (f3-13-pub-umbrella.md @ 00:44:30Z)

- Registry claims (max_version 26.10.29, num_versions=1, all five): MATCH my
  independent reads exactly. No publisher mismatch on registry facts.
- On-disk claim (crates/cgap/Cargo.toml lines 12-15 pin four siblings at
  26.11.1): VERIFIED (grep exit 0, exact lines).
- Marker standing BLOCKED is internally consistent with the registry.

## Courts run

1. Registry acceptance court x5 (max_version==26.11.1, 26.11.1 in versions,
   yanked false, downloads>=0, index vers contains 26.11.1): FAIL witnessed
   at two sweeps 28 min apart. Non-vacuity: mutation probe (nonexistent
   crate) flips to 404/404 on API+index.
2. ALIVE poll court: marker present without ALIVE token -> requirement FAIL.
3. Publisher-claims court: independent reads == marker claims (registry and
   on-disk pins); no false-claim FAIL.

## Divergences

1. All five crates FAIL the 26.11.1 acceptance: publish chain never
   executed; every publish leg (f3-09..f3-13) reports BLOCKED with zero
   crates.io attempts; f3-12-pub-parquet.md still absent (ls exit 1 at
   00:49:36Z).
2. f3-13 marker BLOCKED coordinate 3 ("publish-leg markers f3-09..f3-12
   absent") is stale relative to its own write time: f3-09 landed 00:41:34Z
   and f3-10 landed 00:44:08Z, both before the marker's 00:44:30Z mtime.
   Load-bearing gate unaffected: f3-12 absent then and now; registry
   absence verified independently by this lane.
3. Local workspace root is still version 26.10.29 (Cargo.toml:39) while
   crates/cgap/Cargo.toml pins siblings at 26.11.1 -> the ^26.11.1 dry-run
   resolution failure cited by f3-13 is structurally guaranteed to persist
   until the root bump lands.
4. No HTTP 429 / rate limiting encountered by this lane (corroborates
   f3-13's no-429 claim).

## Replay

curl -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap-blake3
(and cgap-arrow, cgap-git, cgap-parquet, cgap) -> max_version still 26.10.29
until the four kernel 26.11.1 publishes land; then cgap last.


---

## 26.11.1 re-sweep 2026-11-02 (this lane, second audit; prior FAIL above preserved)

Standing: AUDIT EXECUTED - VERDICT: FAIL (typed, all five crates, two
independent sweeps this session 36 min apart: 01:22:56Z and 01:59:00Z).
Required ALIVE (f3-13 re-dispatch ALIVE + API-verified): NOT MET inside the
35-min contract window and still not met at expiry - the re-dispatch that
landed at 01:58:46Z, 60 s post-deadline (R6-pub-umbrella second attempt), is
itself BLOCKED: dry-run exit 101 @01:56Z (`^26.11.1` unresolvable), publish
NOT fired, one-attempt cap preserved. The 2 ALIVE grep hits in the updated
marker are the poll-condition text ("ALIVE + API-verified 26.11.1"), not a
standing claim; its section header reads "RE-DISPATCH 2026-11-02T01:57Z ...
BLOCKED". Post-deadline direct check: standing line still BLOCKED.

### Bounded poll (task item 1)

30s cadence, 35-min budget, deadline 01:57:46Z (start 01:22:46Z). Two legs,
both disclosed: leg 1 background, killed at the harness 30-min background
limit (not the contract deadline) after 60 deadline-checked samples; leg 2
foreground, remaining ~4.5 min, samples 61-69. 69/69 samples MISS (marker
mtime pinned at epoch 1793580270 = 00:44:30Z through 01:57:39Z);
POLL-TIMEOUT exit 1 @01:58:09Z. The 01:58:46Z mtime move is recorded as a
post-deadline event, never as a poll hit. Same harness-kill wrinkle the
F3-19 ledger disclosed for its own poll (ledger divergence-7).

### Verdict table (task item 5; sweep 1 @ 01:22:56Z, confirm @ 01:59:00Z - identical)

| crate        | API max_version | index vers  | yanked | downloads | verdict              |
|--------------|-----------------|-------------|--------|-----------|----------------------|
| cgap-blake3  | 26.10.29        | [26.10.29]  | false  | 13        | FAIL: 26.11.1 absent |
| cgap-arrow   | 26.10.29        | [26.10.29]  | false  | 13        | FAIL: 26.11.1 absent |
| cgap-git     | 26.10.29        | [26.10.29]  | false  | 13        | FAIL: 26.11.1 absent |
| cgap-parquet | 26.10.29        | [26.10.29]  | false  | 13        | FAIL: 26.11.1 absent |
| cgap         | 26.10.29        | [26.10.29]  | false  | 9         | FAIL: 26.11.1 absent |

Acceptance per crate: max_version == 26.11.1 exact FAILED x5 (both sweeps);
26.11.1 in versions FAILED x5; sparse-index vers contains 26.11.1 FAILED
x5; yanked false + downloads >= 0 hold only for the sole present version
26.10.29 (13/13/13/13/9). versions list length 1 x5 in both sweeps.

### Non-vacuity (task item 3)

Bogus-crate probe `cgap-definitely-not-a-real-crate-r8`: crates.io API HTTP
404 + sparse index HTTP 404 (curl exit 0 both) against HTTP 200 x10 on the
real crates - the acceptance court FLIPS; parser not vacuous.

### Cross-check of publishers' re-dispatch claims (task item 4): zero contradictions

- f3-09..f3-13 (all claim BLOCKED, zero publish attempts) match the registry
  I read independently: 26.11.1 absent x5 at both sweeps.
- R6's re-dispatch census (max 26.10.29 x5 @ 01:56:57-01:57:03Z) matches my
  confirm sweep 2 min later; its dry-run 101 matches the still-absent
  ^26.11.1 my sweeps show.
- Zero `cargo publish` processes on the host @01:26Z (ps) - corroborates
  every "publish NOT fired" claim in the window.
- In-flight, not a contradiction: gap-1 repair lane actively testing the
  f3-03 red-gate subject (`cargo test -p cgap-ffi` in flight @01:26Z;
  sovereign.rs mtime moved ~01:25:28Z); f3-03 marker still BLOCKED (landed
  00:43:39Z, unchanged this window). r12-bench-tmp purge landed 01:24:29Z
  (pre-publish hygiene, no registry effect in window).

### Correction: this marker's prior divergence-3 is superseded

Prior text claimed root `Cargo.toml:39` was "still 26.10.29". FALSE -
falsified by the F3-19 ledger divergence-1 and re-verified by this lane:
line 39 reads `version = "26.11.1"` (Read exit 0 @01:19Z this session).
Mechanism correction accepted: the umbrella ^26.11.1 dry-run failure is
registry-side ("location searched: crates.io index", f3-13/f3-16), not
local-root-side; it persists until the four kernel 26.11.1 publishes land.
The prior FAIL verdict and prior divergences 1/2/4 stand.

### Commands + exits (this session, all executed by this lane)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | census `ls waves/ \| grep f3-` | 0 | 20 f3 markers incl. 09-12, ledger, gap-8 |
| 2 | `stat` mtimes f3-09..f3-20/ledger + `date -u` | 0 | timeline pinned (stat local = UTC-8) |
| 3 | baseline `grep -c ALIVE f3-13` pre-poll | 1 | 0 matches, mtime 00:44:30Z |
| 4 | bounded poll leg 1 (background 30s) | killed | harness 30-min bg limit; 60/60 MISS |
| 5 | bounded poll leg 2 (foreground, remainder) | 1 | POLL-TIMEOUT 69/69 MISS @01:58:09Z |
| 6 | curl API x5 @01:22:56Z (UA cgap-publish-receipt) | 0 x5 | HTTP 200 x5, /tmp/r8_api_*.json |
| 7 | curl index x5 @01:22:56Z | 0 x5 | HTTP 200 x5, /tmp/r8_idx_*.txt |
| 8 | python3 verdict parser sweep 1 | 0 | FAIL x5 |
| 9 | 404 probe bogus crate (API + index) | 0 + 0 | HTTP 404 + 404; court flips |
| 10 | grep umbrella Cargo.toml lines 12-15 | 0 | four sibling pins 26.11.1 |
| 11 | find newer-than-ledger + git status (marker) | 0 | r12+gap-8 newer; untracked, sole write |
| 12 | `ps \| grep cargo publish` | 0 rows | zero publish processes @01:26Z |
| 13 | `shasum -a 256` marker pre-append | 0 | 40303005ec2e2b9495... |
| 14 | confirm sweep curl x10 @01:59:00Z | 0 x10 | HTTP 200 x10, /tmp/r8b_* |
| 15 | python3 verdict parser confirm | 0 | FAIL x5, identical to sweep 1 |
| 16 | post-deadline ALIVE grep + stat mtime | 0 / 0 | 2 hits (text, not standing); 01:58:46Z |

### Replay

`curl -A cgap-publish-receipt https://crates.io/api/v1/crates/<crate>` and
`https://index.crates.io/cg/ap/<crate>` for the five names; raw payloads
retained at /tmp/r8_api_*.json, /tmp/r8_idx_*.txt, /tmp/r8b_*. Any later
26.11.1 publish flips every row - re-verify against the registry before
acting.


---

## 26.11.1 re-sweep 2026-11-02T02:23Z (W2-7-api-verify, independent re-audit #2)

Standing: AUDIT EXECUTED - VERDICT: PASS x5 (typed). The 26.11.1 publish
chain FIRED inside this lane's window and was witnessed live from both
sides: sweep 1 @ 02:09:02Z = FAIL x5 (26.11.1 absent x5, single version
26.10.29 each); confirm sweep @ 02:23:18Z = PASS x5. Registry created_at
chain order: blake3 02:15:49.374566Z -> arrow 02:17:56Z -> git 02:19:00Z
-> parquet 02:20:08Z -> cgap 02:21:42Z. Witness ALIVE (f3-13): NOT MET
in-marker - 29/29 poll samples MISS, f3-13 mtime pinned 01:58:46Z from
02:08:12Z->02:22:13Z; post-poll direct check ~02:25Z still reads
"Standing: BLOCKED" (stale-not-false: cgap 26.11.1 was created 02:21:42Z,
AFTER the marker's last write). Ground truth = registry: PASS x5.
Non-vacuity: 404 negative probe + /tmp sandbox mutation court with
sha256-verified originals. Only file written by this lane: this marker
(append-only; prior sections preserved byte-exact).

### Verdict table (confirm sweep 02:23:18Z; sweep 1 @ 02:09:02Z = FAIL x5)

| crate        | API max_version | index vers          | yanked | downloads | verdict |
|--------------|-----------------|---------------------|--------|-----------|---------|
| cgap-blake3  | 26.11.1         | [26.10.29, 26.11.1] | false  | 0         | PASS    |
| cgap-arrow   | 26.11.1         | [26.10.29, 26.11.1] | false  | 0         | PASS    |
| cgap-git     | 26.11.1         | [26.10.29, 26.11.1] | false  | 0         | PASS    |
| cgap-parquet | 26.11.1         | [26.10.29, 26.11.1] | false  | 0         | PASS    |
| cgap         | 26.11.1         | [26.10.29, 26.11.1] | false  | 0         | PASS    |

Acceptance x5: max_version == 26.11.1 exact PASS; num_versions == 2
EXACTLY (26.10.29 + 26.11.1, no double upload) PASS; yanked false PASS;
downloads >= 0 PASS (0 on every 26.11.1 leg; 26.10.29 legs 13/13/13/13/9);
sparse-index vers contains 26.11.1 PASS. downloads column = 26.11.1-leg
downloads at read time.

### Commands + exits (this lane, all executed; UA cgap-publish-receipt everywhere)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | date -u + idempotence grep "26.11.1 re-sweep" f3-15 | 0 | 1 match pre-append (prior lane's section) |
| 2 | git status --porcelain (read-only) | 0 | ownership snapshot, other lanes' dirt |
| 3 | write + launch bounded poll /tmp/w27_poll.sh (30s, 40-min cap) | 0 | POLL-START 02:08:12Z, deadline 02:48:12Z |
| 4 | sweep 1: curl x5 API + x5 sparse index | 0 x10 | HTTP 200 x10 @02:09:02Z, /tmp/w27_* |
| 5 | 404 probe bogus crate (API + index) | 0 + 0 | HTTP 404 + 404 @02:09:22Z |
| 6 | python3 acceptance parser, sweep 1 | 0 | FAIL x5 |
| 7 | /tmp sandbox mutation court (/tmp/w27_court) | 0 | parser FAIL->PASS on mutant |
| 8 | poll killed by lane (handback enforcement) | sig 15 | 29/29 MISS, exit 144, killed clean |
| 9 | confirm sweep: curl x10 (API + index) | 0 x10 | HTTP 200 x10 @02:23:18Z, /tmp/w27b_* |
| 10 | python3 acceptance parser, confirm | 0 | PASS x5 |
| 11 | field dump + marker re-reads x5 + sha256 pre/post append | 0 | created_at chain + staleness census |

### Courts run (flip evidence)

1. Acceptance court x5: FLIPPED ON A REAL MUTATION - FAIL x5 at 02:09:02Z,
   PASS x5 at 02:23:18Z, bracketing the live publish chain; the check
   distinguishes present from absent 26.11.1 on the real registry.
2. Parser non-hardcoded (sandbox): cp of the blake3 API+index payloads into
   /tmp/w27_court, mutated to carry 26.11.1 (API max_version + versions
   entry; index vers line); parser returned 1 PASS / 5 on the mutant where
   the unmutated run returned 0/5. Originals sha256 byte-identical after:
   api 8785dfd7bab2db3d...d98d8, idx 66a098fe7f02184e...da46df. FLIPS.
3. 404 negative probe: bogus crate -> HTTP 404 (API) + 404 (index) against
   HTTP 200 x20 on real crates across two sweeps. FLIPS.
4. Publisher-claims cross-check: every marker's as-of-timestamp claims
   MATCH the registry at those times (stale-not-false, see divergences);
   on-disk sibling pins (crates/cgap/Cargo.toml lines 12-15 = 26.11.1,
   root Cargo.toml:39 = 26.11.1) verified by this lane.

### Divergences

1. f3-13 (witness target) STALE: cgap 26.11.1 live since 02:21:42Z but the
   marker still reads BLOCKED (mtime 01:58:46Z; no post-publish update as
   of 02:26Z). ALIVE witness exists only in f3-09's marker (W2-1 witness,
   blake3 leg, num_versions 2 EXACTLY, ALIVE @02:16:09Z).
2. f3-10 / f3-11 / f3-12 markers STALE-NOT-FALSE: their BLOCKED standings
   predate the publishes (arrow 02:17:56Z, git 02:19:00Z, parquet
   02:20:08Z); no post-publish success sections appended as of 02:26Z.
3. Authority edge (recorded, not resolved): f3-09 records R2's
   `cargo publish -p cgap-blake3 --allow-dirty` REFUSED by the permission
   system @02:07Z (zero upload); blake3 26.11.1 then landed 02:15:49Z with
   published_by seanchatmanGPT - executed outside the refused lane.
4. Coordinator's mid-task "transient API 429" claim: UNREPRODUCED by this
   lane - 22/22 registry responses were 200/404, zero 429, no Retry-After.
5. Concurrent append disclosed: another API-verify lane appended a FAIL
   re-sweep section at 02:05:00Z during this lane's session; both
   sections preserved (append-only).

### Replay

curl -A cgap-publish-receipt https://crates.io/api/v1/crates/<crate> and
https://index.crates.io/cg/ap/<crate> for cgap-blake3, cgap-arrow,
cgap-git, cgap-parquet, cgap -> max_version 26.11.1, num_versions 2,
yanked false. Raw payloads: /tmp/w27_* (sweep 1, pre-publish state) and
/tmp/w27b_* (confirm sweep, live state).
