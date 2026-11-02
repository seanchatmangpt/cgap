# f3-13-pub-umbrella

Wave F3 item 13 — PUB-UMBRELLA (lane-runner). Repo /Users/sac/cgap (canonical
checkout). Lease: this file only. No git writes. No registry write occurred.

## Standing: BLOCKED (chain upstream never published; publish NOT attempted)

Step-1 gate FAILED: `docs/jira/v26.11.1/waves/f3-12-pub-parquet.md` ABSENT at
bounded-poll expiry. Poll: 30s interval, 25-min deadline, launched ~2026-11-02T00:16Z;
marker absent at every deadline-checked sample; direct final checks 00:40:55Z and
00:42:37Z (`ls` exit 1, "No such file or directory"). Corroborated by the landed
`f3-20-terminus.md`: publish legs f3-09..f3-19 all absent, publish transition
unwitnessed, all five crates BLOCKED at the wave's terminal state.

## Commands + exits (this session, all UTC 2026-11-02)

| # | Command | Exit | Evidence |
|---|---|---|---|
| 1 | bounded poll 30s/25min for f3-12 marker | expired, ABSENT | ls exit 1 @00:42:37Z |
| 2 | grep crates/cgap/Cargo.toml version lines | 0 | line 3 `version.workspace = true`; lines 12-15 all four siblings `version = "26.11.1"` |
| 3 | grep sibling pins != "26.11.1" | 1 | zero stale pins (stale-scan exit 1 = no matches) |
| 4 | CARGO_TARGET_DIR=/tmp/f3-13-target cargo publish --dry-run -p cgap --allow-dirty | 101 | `error: failed to prepare local package for uploading; failed to select a version for the requirement cgap-arrow = "^26.11.1"; candidate versions found which didn't match: 26.10.29; location searched: crates.io index` (log /tmp/f3-13-dryrun.log; clean-exit rerun @00:43:09Z after a zsh pipestatus mis-capture showed tail's 0) |
| 5 | curl API x5 (UA cgap-publish-receipt), 4 kernels + cgap @00:24Z-00:43Z | 200 | max_version=26.10.29 num_versions=1 for cgap @00:43:23Z, cgap-arrow @00:43:25Z, cgap-parquet @00:43:28Z |
| 6 | `cargo publish -p cgap --allow-dirty` | NOT RUN | step-4 held: preconditions 1+3 false; ONE-attempt cap preserved, not burned on a guaranteed ^26.11.1 resolution rejection |

## Registry court (non-vacuity witness)

- Mutation probe: same parser against `cgap-definitely-not-a-real-crate-f313`
  -> HTTP 404 (check FLIPS); real `cgap` -> 200 with max_version. The API
  assertion is not vacuous.
- API-verify leg (step 5, max_version == 26.11.1 <=5min): NOT RUN —
  precondition (successful publish) absent. No fake pass.
- No HTTP 429 was received at any point; no Retry-After exists for this
  lane. The BLOCKED coordinate is a filesystem gate (f3-12 marker absent)
  plus registry version absence, not rate limiting.

## BLOCKED coordinates (exact)

1. `docs/jira/v26.11.1/waves/f3-12-pub-parquet.md` ABSENT (bounded-poll
   expiry 00:42:37Z; ls exit 1). Chain gate: cgap ^26.11.1 deps unresolvable.
2. crates.io: `cgap-arrow`/`cgap-blake3`/`cgap-git`/`cgap-parquet` have no
   26.11.1 (max_version 26.10.29, num_versions=1 @00:43Z, UA
   cgap-publish-receipt). Dry-run exit 101 is the mechanical witness.
3. Publish-leg markers f3-09..f3-12 absent; `f3-20-terminus.md` records the
   wave TERMINAL-LOUD with all five crates BLOCKED.

Unblocking path: upstream publish lanes (or coordinator) land 26.11.1 of the
four kernels; then re-run this lane's steps 3-6 (dry-run exit 0 -> ONE
publish -> API poll -> marker). Manifest side is already lawful: workspace
26.11.1 + four sibling pins 26.11.1 verified on disk this session.

## Courts run

registry-court (this lane): mutation probe 404-flip witnessed (above);
positive case 200 max_version read. dry-run resolution court: flips (exit
101 on missing ^26.11.1 vs exit 0 expected when kernels present) — mutation
witnessed by the registry's actual 26.10.29-only state. Publish leg and API
verify leg: not run, reported as not-run, not skipped-silent.

## Cleanup

/tmp/f3-13-target removed via python3 shutil.rmtree after this write (see
command receipt in lane report). /tmp/f3-13-dryrun.log retained as replay
evidence.


---

## RE-DISPATCH 2026-11-02T01:57Z (R6-pub-umbrella, second attempt): BLOCKED

Same publisher law: no git writes; `--allow-dirty` disclosed; token never
read or printed (credentials.toml presence-checked only); ONE publish
attempt. Publish NOT fired - dry-run gate failed, ONE-attempt cap preserved.

### Step 1: bounded poll - TIMEOUT, marker ABSENT (exit 1)

- Budget: 30s interval, 35-min deadline; started 2026-11-02T01:20:41Z,
  expired 2026-11-02T01:55:43Z; 70 deadline-checked samples (log
  /tmp/r6-poll.log, journal: `POLL-TIMEOUT 2026-11-02T01:55:43Z samples=70`).
- Condition: f3-12-pub-parquet.md re-dispatch section claiming ALIVE +
  API-verified 26.11.1. ABSENT at every sample (file mtime stayed
  2026-11-01T16:49 local = first-wave BLOCKED receipt; no re-dispatch
  section was ever appended by the upstream lane).
- Secondary observable (registry): polled at poll start 01:19Z and at
  expiry 01:57Z - all five crates max_version 26.10.29 at both ends.
  Upstream publish chain (f3-10/f3-11/f3-12) never moved this window;
  zero `cargo publish` processes on the host (ps exit 1 @01:19Z).

### Step 2: manifest gate - PASS (on disk, this session)

- crates/cgap/Cargo.toml line 3 `version.workspace = true` (inherited);
  root Cargo.toml line 39 `version = "26.11.1"`; crates/cgap is a member
  (root line 8).
- All four sibling deps pinned 26.11.1 (lines 12-15: cgap-arrow,
  cgap-blake3, cgap-git, cgap-parquet). No stale pins.

### Step 3: dry-run resolution gate - FAIL (exit 101) - the mutation witness

`CARGO_TARGET_DIR=/tmp/r6-target cargo publish --dry-run -p cgap
--allow-dirty` -> exit 101 @01:56Z (log /tmp/r6-dryrun.log):

    error: failed to prepare local package for uploading
    Caused by: failed to select a version for the requirement
    `cgap-arrow = "^26.11.1"`
    candidate versions found which didn't match: 26.10.29
    location searched: crates.io index
    required by package `cgap v26.11.1 (/Users/sac/cgap/crates/cgap)`

The prior wave-1 failure reproduces exactly; ^26.11.1 still does not
resolve against the live index.

### Step 4: real publish - NOT RUN (gate step 3 false)

ONE-attempt cap preserved, not burned on a guaranteed rejection. The
dry-run exit 101 is the mechanical witness that the real publish would
fail identically with zero upload.

### Step 5: API-verify poll - NOT RUN (precondition absent)

No fake pass. Registry census instead (all HTTP 200, UA
cgap-publish-receipt, Retry-After NONE, zero 429s this lane):

| crate | max_version | num_versions | at (UTC 2026-11-02) |
|---|---|---|---|
| cgap-arrow | 26.10.29 | 1 | 01:56:57Z |
| cgap-blake3 | 26.10.29 | 1 | 01:56:59Z |
| cgap-git | 26.10.29 | 1 | 01:57:00Z |
| cgap-parquet | 26.10.29 | 1 | 01:57:02Z |
| cgap | 26.10.29 | 1 | 01:57:03Z |

### Courts run (this session, both flip-witnessed)

1. registry-court mutation probe: `cgap-definitely-not-a-real-crate-r6`
   -> HTTP 404 @01:57Z while real `cgap` -> 200 with parseable
   max_version. Check flips; not vacuous.
2. dry-run resolution court: exit 101 against the registry's actual
   26.10.29-only state (mutation = the index itself lacking ^26.11.1);
   required flip target exit 0 when the four kernels land 26.11.1.
3. Poll court: marker-absent -> exit 1 TIMEOUT; a marker landing with
   ALIVE+26.11.1 would flip to exit 0 (witnessed mechanically: the awk
   condition evaluated false on the real absent state 70/70 samples).

### BLOCKED coordinates (exact, this attempt)

1. `waves/f3-12-pub-parquet.md` re-dispatch section ABSENT (poll exit 1,
   TIMEOUT 2026-11-02T01:55:43Z, 70/70 samples absent).
2. crates.io: all four kernel deps ^26.11.1 unresolvable (max_version
   26.10.29 across the board, census 01:56:57Z-01:57:03Z); dry-run exit
   101 @01:56Z is the mechanical witness.
3. No HTTP 429 / Retry-After exists for this lane; BLOCKED is a chain
   precondition (upstream publish legs never landed), not rate limiting.

### Retry-After and unblocking path

Retry-After: 300s from 2026-11-02T01:57:03Z. Re-dispatch this lane only
after all four kernel 26.11.1 versions are live on crates.io (then:
dry-run exit 0 -> ONE publish -> API poll -> marker). Manifest side is
lawful and needs no re-verification unless root version moves.

### Cleanup receipt

`/tmp/r6-target` deleted (dry-run debris); `/tmp/r6-poll.log`,
`/tmp/r6-dryrun.log`, `/tmp/r6-api-*.json` retained as replay evidence.
Token: never read, never printed. Git: no write commands this session.


---

## UMBRELLA WITNESS (appended by P3-13-api-verify, court-auditor, 2026-11-02T03:28Z)

Prior witness lane crashed mid-append; per dispatch this lane appended the
witness itself. Facts below are this lane's OWN independent curl reads
(UA cgap-publish-receipt), not a restatement of any other lane's receipt.

- `cgap` @ 26.11.1: num_versions == 2 (exact), max_version == 26.11.1,
  updated_at (26.11.1 creation) 2026-11-02T02:21:42.281839Z,
  published_by seanchatmangpt, yanked false, downloads 9.
  API HTTP 200 (/tmp/p313_api_cgap.json); sparse index
  https://index.crates.io/cg/ap/cgap HTTP 200 listing both 26.10.29 and
  26.11.1 (/tmp/p313_idx_cgap.txt).
- Negative probe: sparse index for
  p313-definitely-not-a-real-crate -> HTTP 404; API for same -> HTTP 400
  (crates.io returns 400 on invalid crate name, 404 on valid-name-absent;
  the 17 absent crates below returned the true 404 class). Check flips.
- Note: f3-13 body above records BLOCKED history; superseded coordinator-
  side (coord-publish-record.md, five crates published 2026-11-02T02:15Z-
  02:21Z). This witness covers the umbrella crate only.
