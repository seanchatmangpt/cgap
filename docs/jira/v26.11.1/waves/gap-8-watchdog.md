# GAP-8 Watchdog Receipt

Lane: GAP-8 PUBLISH-CHAIN WATCHDOG (court-auditor, read-only). Repo `/Users/sac/cgap`,
HEAD `cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e` (unchanged this lane; zero git writes).
Watch window: opened 2026-11-02T00:48:11Z, 60s cadence, hard deadline 02:03:11Z (75 min).
Watch completed early: the last required marker (f3-publish-ledger.md, item 19) landed
00:58:45Z; poller stopped 01:14:11Z with the completion condition met. Sole write of
this lane: this file. All times UTC 2026-11-02.

## Watch outcome (verdict: WATCH-COMPLETE)

1. f3-03-workspace-green.md landed 00:43:39Z standing **BLOCKED - gate RED**
   (`cargo test --workspace` exit 101; sole failure
   `cgap-ffi sovereign::xip_courts::xip_t1_riscv_staticlib_size_court` panic at
   `crates/cgap-ffi/src/sovereign.rs:2399`: T1 FLASH 264,881 B >= budget 65,536 B,
   4.04x over, deterministic on isolated rerun). Publishers must NOT proceed past it.
2. Chain f3-09 -> f3-13 landed: 09 00:41:34Z, 10 00:44:08Z, 11 00:48:13Z, 12 00:49:59Z
   all BLOCKED by chain law with publish never invoked; **13 landed 00:44:30Z OUT OF
   CHAIN ORDER (before 11 and 12)** - named FAIL below.
3. f3-14..f3-19 all landed: 15 00:51:49Z, 17 00:53:46Z, 14 00:55:10Z, 18 00:55:37Z,
   ledger (item 19) 00:58:45Z.
4. No marker claims a completed publish. Registry truth (five independent lanes:
   f3-15 two sweeps 28 min apart, f3-18, f3-13, f3-16, ledger): max_version 26.10.29
   x5 through 00:52:53Z; 26.11.1 on zero of five slots. One-attempt publish budgets
   intact chain-wide (f3-18 grep court + its own live registry reads).

## Timeline table

landed-at = file mtime UTC. Standing read first-hand at first sight by this watchdog
(poller events /tmp/gap8-watchdog/events.jsonl; 26 events captured, 0 malformed).

| marker | landed-at | standing | chain-law-ok |
|---|---|---|---|
| f3-08-registry-diff | 00:23:44Z | CLEAR x5 | n/a (pre-publish) |
| f3-01-version-lock | 00:24:46Z | ALIVE | n/a |
| f3-02-regen | 00:25:10Z | ALIVE | n/a |
| f3-07-package-audit | 00:26:00Z | PASS | n/a |
| f3-05-auth | 00:26:40Z | ALIVE (presence-level) | n/a |
| f3-06-pins | 00:27:53Z | ALIVE | n/a |
| f3-04-courts-green | 00:29:08Z | GREEN (exits-only, no standing word) | n/a |
| f3-20-terminus | 00:37:21Z | TERMINAL-LOUD (as-of-timestamp) | NO - premature (known/superseded) |
| f3-09-pub-blake3 | 00:41:34Z | BLOCKED (03 absent) | YES - publish NOT attempted |
| f3-16-replay | 00:43:05Z | BLOCKED | n/a (audit; probes disclosed) |
| f3-03-workspace-green | 00:43:39Z | BLOCKED - gate RED | YES - correctly red |
| f3-10-pub-arrow | 00:44:08Z | BLOCKED by chain law (09 BLOCKED) | YES - refused |
| f3-13-pub-umbrella | 00:44:30Z | BLOCKED (f3-12 absent) | NO - out-of-chain-order landing |
| f3-11-pub-git | 00:48:13Z | BLOCKED by chain law (10 BLOCKED) | YES - refused |
| f3-12-pub-parquet | 00:49:59Z | BLOCKED (11 absent at poll expiry) | YES - refused |
| f3-15-api-verify | 00:51:49Z | FAIL typed x5 (independent registry) | YES - corroborates |
| f3-17-hygiene | 00:53:46Z | ALIVE (clean window) | YES |
| f3-14-consumer | 00:55:10Z | BLOCKED (26.11.1 absent) + lawful branch | YES |
| f3-18-warden | 00:55:37Z | ALIVE (WAVE-PASS on audited laws) | YES |
| f3-publish-ledger (19) | 00:58:45Z | BLOCKED/UNWITNESSED publish | YES |

## Named anomalies (watchdog teeth)

1. **FAIL named: f3-13-pub-umbrella** - out-of-chain-order marker landing at 00:44:30Z
   while f3-11 (landed 00:48:13Z) and f3-12 (landed 00:49:59Z) were absent. Its
   coordinate-3 claim ("publish-leg markers f3-09..f3-12 absent") was false at its own
   write time (f3-09 on disk from 00:40:25Z per f3-18's poll log). Mitigation, from
   13's own receipt: publish NOT attempted, BLOCK cause (f3-12 absent) real, budget
   intact. Independently named by f3-15 div-2, f3-18 div-2, ledger div-3.
2. f3-20-terminus declared the wave terminal at 00:37:21Z before any publish leg
   finalized - known per dispatch context, superseded by the 09-13/15-19 landings;
   named for the record (also named by f3-12 div-3, f3-15, f3-18 div-3, ledger div-4).

## ALIVE-without-API-verify adjudication (contract clause)

- No publish-chain marker (09-13) claims ALIVE standing: the clause has no
  publish-side violation. My poller's full-text ALIVE-word heuristic flagged 09/10/11
  ("ALIVE" appears only inside their Retry-After instructions); the actual standing
  lines read BLOCKED. Adjudicated: false positives of the watchdog heuristic, not lane
  violations.
- Pre-publish legs 01/02/05/06/17 claim ALIVE without API-verify lines: not violations
  - API-verify is the post-publish evidence class; these legs have no publish to
  verify, and the registry-side pre-publish evidence is carried by f3-08/f3-15/f3-18.
  f3-05 itself discloses token VALIDITY stays UNVERIFIED by design.

## First-hand verifications (this lane's own commands, all this session)

| command | exit |
|---|---|
| baseline `ls -la` waves census (13 f3 markers; 11/12/14/15/17/18/19 absent) | 0 |
| `git status --porcelain` snapshot (87 paths, other lanes' work; lane wrote nothing) | 0 |
| `sed -n '39p' Cargo.toml` (f3-15 div-3 falsifier) | 0 -> `version = "26.11.1"` |
| `shasum -a 256` over all 20 f3 markers | 0 |
| sha cross-check vs f3-18's cited hashes (09/10/11/12/13/16) | 6/6 byte-exact match |
| poller sha256 change-detection, 60s cadence, 00:48:40Z-01:13:41Z | stopped, completion met |
| events.jsonl integrity (line-by-line json.loads) | 0 malformed / 26 lines |
| `pkill` poller + logged stop | 0 |

courts_run: none by this lane - the watchdog lease holds no mutation court. Its flip
evidence is in-vivo: a chain-law violation or a publish past the red gate would have
flipped this receipt's verdict; none occurred. Change detection is proven live by the
mid-watch landings (11 first-sight 00:48:40Z; 12/14/15/17/18/ledger BASELINE->CHANGED
stabilization sequences) and corroborated by f3-18's in-vivo f3-11 sha flip.

## Standing

WATCH-COMPLETE. The publish chain did not silently stall - it stalled LOUDLY. The
binding blocker is the red f3-03 gate (`crates/cgap-ffi/src/sovereign.rs:2399`,
xip_t1_riscv_staticlib_size_court, T1 FLASH 4.04x over budget); every publisher
refused to burn its one-attempt budget against it; downstream legs 10/11/12 BLOCKed by
chain law exactly as required (none published past a blocked predecessor); f3-13's
marker landed out of order but did not publish out of order. Unblocking path (ledger
section 6): repair sovereign.rs:2399 -> f3-03 green -> re-dispatch publishers
(dry-run -> ONE publish -> API max_version == 26.11.1) -> re-run f3-14 at 26.11.1.
As-of-write-time receipt: any later 26.11.1 publish flips every row; re-verify against
the registry before acting.

files_written: this file only.
