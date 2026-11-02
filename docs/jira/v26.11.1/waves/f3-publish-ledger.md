# F3 Publish Ledger

Lane: F3 wave item 19 - RECEIPT-AGGREGATOR (court-auditor). Date: 2026-11-02.
Repo: /Users/sac/cgap, branch main, HEAD cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e
(read-only; rev-parse exit 0 at 00:55:34Z). Lease honored: this file is the sole write
of this lane. No git writes, no publish attempts, no mocks, no sleeps outside the
bounded poll the task prescribes. All times UTC 2026-11-02 unless dated otherwise.

## 1. Bounded poll (task item 1)

30s cadence, 30-min budget, window ~00:19:40Z -> 00:49:40Z. First background leg was
killed at the harness 10-min background limit (disclosed wrinkle); restarted with the
remaining budget inside the same 30-min window. DEADLINE_HIT at ~00:49:40Z with
f3-12, f3-14, f3-15, f3-17, f3-18 missing - named loudly, per contract.

Post-deadline, pre-write landings (all read fully before inclusion): f3-12-pub-parquet
(~00:50Z), f3-15-api-verify (~00:52Z), f3-17-hygiene (~00:53Z), f3-14-consumer +
f3-18-warden (~00:55Z). At write time (00:56Z) ALL TEN required markers f3-09..f3-18
exist. The deadline state is superseded at use time, not hidden.

## 2. Per-crate rows

Publish subject: all five crates at workspace version 26.11.1 (root Cargo.toml:39 =
"26.11.1"; umbrella sibling pins 26.11.1 x4). Registry truth at every independent
observation: max_version 26.10.29, num_versions 1, zero 26.11.1.

| crate | version | dry-run exit | publish exit | API-verified ts (max_version 26.10.29) | independent-API (f3-15) | replay-proof (f3-16) |
|---|---|---|---|---|---|---|
| cgap-blake3 | 26.11.1 | 0 (f3-09 diagnostic, 00:39Z) | NOT EXECUTED | 00:52:53Z (this lane) | FAIL: 26.11.1 absent | BLOCKED (probe 1: dry-run 0 = abort-at-upload) |
| cgap-arrow | 26.11.1 | 0 (f3-10 diagnostic, 00:43:24Z) | NOT EXECUTED | 00:52:53Z (this lane) | FAIL: 26.11.1 absent | not probed by f3-16 (blake3+cgap only) |
| cgap-git | 26.11.1 | 0 (f3-11 packaging evidence) | NOT EXECUTED | 00:52:53Z (this lane) | FAIL: 26.11.1 absent | not probed by f3-16 |
| cgap-parquet | 26.11.1 | 0 (f3-12, 00:49:13Z) | NOT EXECUTED | 00:52:53Z (this lane) | FAIL: 26.11.1 absent | not probed by f3-16 |
| cgap | 26.11.1 | 101 (f3-13 00:43:09Z; f3-16 corroborates) | NOT EXECUTED | 00:52:53Z (this lane) | FAIL: 26.11.1 absent | BLOCKED (probe 2: 101 = ^26.11.1 unresolved, registry-side) |

Other independent registry witnesses: f3-15 (sweeps 00:20:42Z + 00:49:02Z, API + sparse
index, FAIL x5), f3-18 (live reads, all five 26.10.29), f3-20 (00:35:10Z), f3-16
(00:40:09-10Z), f3-13 (00:43:23-28Z). No observer disagrees with any other.

## 3. Non-crate rows

- Consumer-build (f3-14): BLOCKED at the dispatch target - `cargo add cgap@26.11.1`
  exit 101, "could not be found in registry index". Lawful BRANCH per blocker law:
  registry-sourced consumer at cgap@26.10.29 (zero path deps; checksum court:
  index cksum == Cargo.lock checksum == sha256 of cached .crate), build 0, run 0,
  4/4 surfaces PASS (blake3 KAT, arrow readback, git pack v2 + checksum-refusal,
  parquet rle round-trip + truncation refusal), inline mutation courts flipped
  (PackChecksumMismatch, LiteralOverrun). Cargo-tree registry line:
  cgap v26.10.29 -> cgap-arrow/blake3/git/parquet v26.10.29 x4, all registry-sourced.
- Hygiene (f3-17): ALIVE - clean window. HEAD unchanged across 30 polls; staged=0
  at open, every iteration, close; zero token material over 16 markers; start rc=2
  (subject-missing) disclosed.
- Warden (f3-18): ALIVE - WAVE-PASS on audited laws. Zero publish attempts chain-wide
  (one-attempt grep court + independent live registry court, 404/200 controls);
  blocked-protocol law satisfied by all six audited markers; stability court FLIPPED
  in vivo (f3-11 edit 00:47:04Z -> 00:48:13Z detected). Warden-cited sha256 of
  f3-09/f3-11-final/f3-12 re-verified byte-exact against disk by this lane.

## 4. Cross-check: publisher claims vs f3-15 independent table (task item 3)

- All 17 landed f3-* markers read fully by this lane.
- Registry facts: every publisher marker's claims (f3-09 baseline 00:22Z, f3-10 through
  00:42:34Z, f3-11 00:45:55Z, f3-12 00:48Z, f3-13 00:43Z) MATCH f3-15's independent
  table (two sweeps) and this lane's own pulls (00:22Z, ~00:27Z, 00:52:53Z). Zero
  contradiction on any registry fact.
- On-disk pins: f3-15's verification of umbrella lines 12-15 = 26.11.1 matches this
  lane's own read. Dry-run exits (0 x4 kernels, 101 umbrella) consistent across
  f3-13/f3-16 (identical error text).
- f3-15's publisher-claims court: "MATCH my independent reads exactly" - corroborated.

## 5. Named divergences

1. f3-15 divergence-3 FACT ERROR: claims root Cargo.toml:39 is still 26.10.29. False:
   f3-01 bumped it; this lane read 26.11.1 at 00:26Z and re-verified at 00:53:46Z
   (grep line 39 exit 0; sha256 2430fec1...). Mechanism also wrong: the umbrella
   ^26.11.1 resolution failure is registry-side (f3-16: "location searched: crates.io
   index"; f3-14: cargo add "could not be found in registry index"), not
   local-root-side; it persists until the four kernel 26.11.1 publishes land.
2. f3-12 coordinate 2 claimed item 11 "died mid-publish" (no processes at 00:42Z):
   falsified - f3-11 landed 00:48:00Z and its final write 00:48:13Z (warden-witnessed
   in-vivo edit). The lane was alive; process-death was an inference, not a fact.
3. f3-13 coordinate 3 ("publish-leg markers f3-09..f3-12 absent") stale at its own
   mtime: f3-09 on disk from 00:40:25Z, f3-10 from 00:44:08Z, marker written 00:44:30Z
   (named by f3-15 div-2 and f3-18 div-2; corroborated). BLOCK cause (f3-12 absent)
   still stood.
4. f3-20-terminus (00:37:21Z) declared the wave terminal before any publisher leg
   finalized; 09/10/11/12/13/15/16/17/18 landed after. As-of-timestamp rows,
   superseded (also named by f3-12 div-3, f3-15, f3-18 div-3).
5. f3-11's one-attempt grep count is 2, not 1 (both NOT-EXECUTED statements; zero real
   attempts - substance holds) and its final write postdates the warden window close
   (warden div-1; corroborated, current bytes = warden's final sha).
6. f3-05 auth: token VALIDITY stays UNVERIFIED by lane design (presence-level only).
   Recorded so no downstream lane reads it as a validity proof.
7. Poll wrinkle (this lane): first background poll leg killed at harness 10-min limit,
   restarted within the 30-min contract budget; deadline hit with 5 markers missing;
   all 10 landed before the write. Both states recorded above.

## 6. Publish-budget state and standing

- Zero crates.io publish attempts chain-wide; the one-attempt immutability budget is
  intact for all five crates (asserted by every publisher marker, audited by f3-18,
  registry-corroborated: max_version 26.10.29 x5 at 00:52:53Z).
- Wave standing: BLOCKED/UNWITNESSED publish transition. Root BLOCK coordinate:
  f3-03-workspace-green gate RED (cargo test --workspace exit 101 -
  xip_t1_riscv_staticlib_size_court, sovereign.rs:2399, T1 FLASH 264,881 B vs budget
  65,536 B, 4.04x over; deterministic on rerun). Every publisher correctly refused to
  burn its one attempt against a red gate.
- Retry chain: Retry-After timestamps (00:55:02Z / 00:56:13Z / +300s) have lapsed at
  write time; the binding blocker is the f3-03 red gate, not the clock. Unblocking
  path: repair sovereign.rs:2399 size court -> f3-03 green -> re-dispatch publishers
  (dry-run gate -> ONE publish -> API max_version == 26.11.1; note f3-16's mechanics
  finding: dry-run cannot witness already-exists, API flip is the real witness) ->
  re-run f3-14 at cgap@26.11.1.
- This ledger is an as-of-write-time aggregation (00:56Z). Any later 26.11.1 publish
  flips every row; re-verify against the registry before acting.

## 7. This lane's commands + exits

| command | exit | evidence |
|---|---|---|
| marker census `ls waves/ \| grep ^f3-` (per check) | 0/1 per state | files: 0 @00:19Z, 1 @00:24Z, 5 @00:26Z, 7 @00:28Z, 11 @00:42Z, 14 @00:48Z, 15 @00:51Z, 16 @00:52Z, 17 @00:53Z, 19 @00:55Z |
| bounded poll, 30s/30-min, f3-09..f3-18 (2 legs) | DEADLINE_HIT | missing 12,14,15,17,18 at 00:49:40Z |
| `curl` crates.io API x5, UA cgap-publish-receipt (00:22Z / ~00:27Z / 00:52:53Z) | 0 x15 (HTTP 200) | max_version 26.10.29 x5, num_versions 1 x5, all three sweeps |
| `sed -n '39p' Cargo.toml` + grep -n version (00:26Z, 00:53:46Z) | 0 | `version = "26.11.1"` line 39 |
| grep umbrella Cargo.toml:12-15 pins | 0 | four siblings `version = "26.11.1"` |
| residual `grep -rn 26\.10\.29` over 9 manifests | 0 | 2 hits, out-of-publish-set: integration:3, wasm-footprint:3 |
| `git rev-parse HEAD` | 0 | cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e |
| full Read of all 17 landed f3-* markers | 0 | cited above |
| `shasum -a 256` f3-09/f3-11/f3-12 markers | 0 | match warden-cited shas byte-exact |
| this ledger write (python assembly) + byte verify | 0 | 129 lines; corruption sweep clean (rc=1); final sha256 recorded in the lane report |

---

## Re-dispatch + coordinator-execution fold (A-1-ledger, court-auditor, 2026-11-02)

Closes the aggregation gap R18 named (f3-20-terminus v2: "the publish ledger
(item 19) never gained a re-dispatch section"). This append is the lane's sole
write. All rows below supersede the as-of-00:56Z BLOCKED rows in sections 2-6,
which are preserved byte-exact above as the pre-landing history (stale-not-false
at their own timestamps).

### Authority edge (why the subagents never published)

R2..R6 subagent publish attempts were REFUSED pre-execution by the Claude Code
permission system (f3-09 re-dispatch section, 02:07Z: "cargo never executed;
NO bytes uploaded"; authority refusal, not a registry failure, NO Retry-After).
The coordinator then ran the chain directly per the operator's standing order,
recorded in coord-publish-record.md: five publishes, published_by
seanchatmangpt, ONE invocation per crate, blake3 -> arrow -> git -> parquet ->
umbrella (15s propagation waits between legs; each dry-run resolved its
predecessors from the live sparse index). Subagents consumed zero uploads; the
coordinator consumed exactly one each - the one-attempt immutability budget is
intact for all five crates.

### Witnessed chain order (registry created_at, strictly increasing)

cgap-blake3 2026-11-02T02:15:49.374566Z -> cgap-arrow 02:17:56.021242Z ->
cgap-git 02:19:00.265596Z -> cgap-parquet 02:20:08.018411Z -> cgap
02:21:42.281839Z. Independently witnessed by: W2-1 (f3-09 witness, 02:16:09Z),
W2-2 (f3-10 witness, 02:52:17Z), W2-3 (f3-11 witness, 02:21:47/02:23:42Z),
W2-4 (f3-12 witness, 02:46:17Z), P3-13 (f3-13 umbrella witness, 03:28Z),
f3-15 re-sweep (02:09:02Z FAIL x5 -> 02:23:18Z PASS x5, bracketing the chain),
f3-16 Replay Proof v2 (R9: poll B flipped blake3 between 02:15:29Z n=1 and
02:16:00Z n=2; cgap between 02:21:41Z n=1 and 02:22:12Z n=2), R16 window-2
(f3-18: baseline 01:24Z all 26.10.29/1 -> 02:28Z all 26.11.1/2, flip court
fired IN VIVO), R18 terminus (02:47:51Z), coord-publish-record.md, and this
lane's own live re-witness (below). No observer disagrees with any other.

### Immutability law

num_versions == 2 EXACTLY on all five crates at every post-landing observation
(26.10.29 + 26.11.1, no third version, no yank) - one upload per crate, no
doubles. R9's 20-sample poll B saw zero NUM>2 divergences; R16's 80-poll window
and f3-15's two sweeps agree. This lane's own reads re-witness it live.

### This lane's own live re-witness (2026-11-02, task item 2)

curl -s -A cgap-publish-receipt https://crates.io/api/v1/crates/<crate>, x5,
HTTP 200 x5 (payloads /tmp/a1-*.json): all five max_version 26.11.1, num_versions
2 EXACTLY, newest 26.11.1, yanked false, published_by seanchatmangpt (github
username matches), created_at exactly the chain timestamps above. Exit 0 x5.
Commands + exits: `git rev-parse HEAD` -> 0 (cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e,
read-only lane, no git writes); full Read of coord-publish-record.md,
f3-09..f3-13 (REFUSED/BLOCKED + witness sections), f3-15 re-sweep, f3-16 v2,
f3-18 window-2, f3-20 v2 -> 0; curl x5 -> 0; this append (python assembly,
prior bytes preserved byte-exact) + corruption sweep -> 0.

### Consumer proof pointer

Terminal registry-sourced consumer receipt: f3-14 append (W2-6-consumer,
02:00-02:31Z, per R18's terminus census): `cargo add cgap@26.11.1` exit 0,
`cargo run --release` exit 0, 6/6 surfaces PASS incl. the new 26.11.1 git-dag
and parquet-seal surfaces, 3-way checksum court, `cargo add cgap@26.11.99`
exit 101 non-vacuity flip. The ledger's section 3 BLOCKED consumer row is
superseded by it.

### Zero-contradiction cross-check

Folded against every witness section named above: registry facts (max_version,
num_versions, created_at chain, published_by, yanked) agree across W2-1..W2-4,
P3-13, f3-15, f3-16 v2, R16 window-2, R18, coord-publish-record.md, and this
lane's live reads - zero contradictions. Prior divergences 1-7 (section 5) are
unaffected except as already superseded by the landing; f3-15 divergence-3
(root version) is mooted by the registry flip itself. Standing of the publish
transition flips from BLOCKED/UNWITNESSED to ALIVE/WITNESSED: all five crates
PUBLISHED at 26.11.1, one attempt each, chain-ordered, consumer-proven.
