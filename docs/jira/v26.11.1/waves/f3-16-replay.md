# F3-16 Replay Receipt

Lane F3-16 (DRY-RUN-REPLAY, gate-runner). Repo `/Users/sac/cgap`, canonical checkout, HEAD
`cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e` (session start; re-read 16:44 local,
unchanged). Sole write: this file. No git writes, no publish, no token material in any
captured output (scanned before recording).

## Verdict

BLOCKED - the replay did not witness an already-published 26.11.1, because 26.11.1 is not
on crates.io. Three independent witnesses this session:

1. cgap-blake3 dry-run exits 0 at upload-abort - no conflict exists to hit.
2. cgap umbrella dry-run exits 101 on dependency resolution against the LIVE index:
   `cgap-arrow ^26.11.1` unmatched, only `26.10.29` offered - direct registry proof the
   26.11.1 kernel versions do not exist.
3. Live API rows: max_version 26.10.29 for both subjects at 2026-11-02T00:40:09-10Z.

Corroborated by `f3-20-terminus.md` (all five crates BLOCKED; publish legs f3-09..f3-19
and the publish ledger absent at its bounded-poll expiry).

## Poll leg (loud, disclosed)

Bounded poll for `waves/f3-13-pub-umbrella.md` - 30s interval over two foreground windows
(540s + 300s) plus a background leg (stopped at its time limit): ABSENT at expiry, no
ALIVE witnessed. f3-20's census confirms the lane never landed. The dry-run legs ran
anyway (disclosed divergence, not a silent skip): their evidentiary value - registry
conflict semantics - does not mechanically depend on the f3-13 receipt, and the replayed
subject state is grounded by this lane's own live API rows, not by the receipt.

## Probe 1: cgap-blake3 (earliest-published kernel)

Command: `CARGO_TARGET_DIR=/tmp/f3-16-target cargo publish --dry-run -p cgap-blake3
--allow-dirty` - exit 0. Verbatim `/tmp/f3-16-blake3.out`:

    Updating crates.io index
   Packaging cgap-blake3 v26.11.1 (/Users/sac/cgap/crates/cgap-blake3)
    Packaged 14 files, 109.9KiB (28.0KiB compressed)
   Verifying cgap-blake3 v26.11.1 (/Users/sac/cgap/crates/cgap-blake3)
   Compiling cgap-blake3 v26.11.1 (/tmp/f3-16-target/package/cgap-blake3-26.11.1)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.12s
   Uploading cgap-blake3 v26.11.1 (/Users/sac/cgap/crates/cgap-blake3)
warning: aborting upload due to dry run

On-disk workspace version was 26.10.29 at 16:15 local and 26.11.1 by probe time (~16:39):
the concurrent f3-01 bump lane landed mid-session. The replay therefore ran at 26.11.1 -
the wave's publish candidate, correct subject.

Mechanics finding: dry-run aborts INSIDE the upload step before any server interaction
(the warning line IS the abort), so `cargo publish --dry-run` cannot witness
already-exists semantics at all - even a genuinely published version would exit 0. The
contract's expected "exit != 0 with already-exists text" is unreachable by construction
from this probe; its fallback clause "a dry-run still SUCCEEDING = BLOCKED" is the
correct reading and it fired.

## Probe 2: cgap (umbrella)

Command: `CARGO_TARGET_DIR=/tmp/f3-16-target cargo publish --dry-run -p cgap
--allow-dirty` - exit 101. Verbatim `/tmp/f3-16-cgap.out`:

    Updating crates.io index
   Packaging cgap v26.11.1 (/Users/sac/cgap/crates/cgap)
    Updating crates.io index
error: failed to prepare local package for uploading

Caused by:
  failed to select a version for the requirement `cgap-arrow = "^26.11.1"`
  candidate versions found which didn't match: 26.10.29
  location searched: crates.io index
  required by package `cgap v26.11.1 (/Users/sac/cgap/crates/cgap)`

Exit is nonzero as the contract EXPECTED, but the semantics are dependency-resolution
failure, not already-exists: the umbrella cannot even package, because its dependency
26.11.1 is absent from the live index. That failure is a positive control that fired - it
proves this probe consults the live crates.io index, so the registry-state claims above
are grounded in this session's network observables, not only cited from f3-20.

## Live registry rows (this lane, this session)

- `curl -s -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap-blake3` -> 200:
  max_version=26.10.29 newest=26.10.29 num_versions=1 at 2026-11-02T00:40:09Z
- `curl -s -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap` -> 200:
  max_version=26.10.29 newest=26.10.29 num_versions=1 at 2026-11-02T00:40:10Z
- 26.11.1 on zero of two probed slots; f3-20 extends the absence to all five at 00:35:10Z.

## Commands + exits (this lane, this session)

(a) cargo rows run with `CARGO_TARGET_DIR=/tmp/f3-16-target`, `--allow-dirty`,
    cwd `/Users/sac/cgap`.

| Command | Exit | Witness |
|---|---|---|
| bounded poll f3-13 (30s / 25min budget, bg leg + 2 fg windows) | 1 | ABSENT at expiry |
| `cargo publish --dry-run -p cgap-blake3` (a) | 0 | abort-at-upload; v26.11.1 packaged |
| `cargo publish --dry-run -p cgap` (a) | 101 | cgap-arrow ^26.11.1 unmatched |
| `curl` API x2 (UA cgap-publish-receipt) | 0 | max_version 26.10.29 x2 @ 00:40Z |
| `python3 shutil.rmtree('/tmp/f3-16-target')` | 0 | pre: 3 entries / 6.1M; post: absent |

## Hygiene

`/tmp/f3-16-target` deleted via python3 shutil.rmtree after the probes; verified absent
post-delete. Raw probe outputs kept for replay at `/tmp/f3-16-blake3.out` and
`/tmp/f3-16-cgap.out` (outside the tree). No in-tree writes outside this file.

## Standing

BLOCKED, coordinates:

1. cgap-blake3 26.11.1: dry-run exit 0 (abort-at-upload) + live API max_version 26.10.29
   at 00:40:09Z - the version is not on the registry; there is nothing to re-publish
   against, so the replay cannot prove "the published state is real".
2. cgap 26.11.1: dry-run exit 101 with resolution semantics (cgap-arrow ^26.11.1
   unmatched, only 26.10.29 offered) - the umbrella candidate is unpublishable until its
   kernel deps land on the registry.
3. Root cause (inherited, corroborated, not caused by this lane): publish legs
   f3-09..f3-19 and `f3-publish-ledger.md` are absent (f3-20 census); 26.10.29 remains
   max_version on every probed slot.

Unblocking path: land the real publish legs, then re-witness publication by live API rows
(max_version flip to 26.11.1) - NOT by a dry-run conflict error, which the mechanics
finding above shows this probe can never produce.


---

## Replay Proof v2 (R9-replay, gate-runner, 2026-11-02)

API-witnessed per the v1 mechanics finding: `cargo publish --dry-run` aborts
inside the upload step before server contact and can never witness
already-exists; the real replay witness is the live registry rows. Repo
`/Users/sac/cgap`, HEAD `cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e`
(rev-parse exit 0 at 01:19:26Z). Sole write: this append. Zero `cargo
publish` (non-dry-run) invocations from this lane - the one-attempt
immutability cap is untouched by R9. No token material in any captured
output (checked before recording; dry-run outputs contain no credentials).

### Step 1: bounded polls (30s cadence, 35-min budgets, no other sleeps)

Poll A (f3-13 re-dispatch watch), 01:24:00Z -> 01:59:00Z, 68 samples, exit 1
DEADLINE_HIT. Registry baseline at all 68 samples: both subjects max_version
26.10.29, num_versions 1. Boundary signal, loudly disclosed: f3-13 receipt
mtime changed at samples 67/68 (01:57:55Z / 01:58:23Z) - the R6-pub-umbrella
re-dispatch receipt was landing; read on disk it is BLOCKED (its own
70-sample poll timed out on the upstream kernel legs; publish NOT fired;
census 26.10.29 x5 at 01:56:57-01:57:03Z; log /tmp/r9-poll.log).

Poll B (witness, after R2 was dispatched), launched 02:12:23Z, 20 samples,
exit 0 WITNESS-ALIVE at 02:22:12Z. cgap-blake3 flipped between sample 7
(02:15:29Z, n=1) and sample 8 (02:16:00Z, n=2) - consistent with the
registry-side created_at 02:15:49.374566Z. cgap flipped between sample 19
(02:21:41Z, n=1) and sample 20 (02:22:12Z, n=2) - created_at
02:21:42.281839Z. Zero NUM>2 divergences across all 20 samples (log
/tmp/r9-poll2.log).

### Step 2: dry-run probes (CARGO_TARGET_DIR=/tmp/r9-target, --allow-dirty)

Pre-flip records (02:03-02:05Z): cgap-blake3 exit 0 ("aborting upload due to
dry run" - EXPECTED and NOT the witness, per the v1 mechanics finding);
cgap exit 101 (`cgap-arrow = "^26.11.1"` unmatched, only 26.10.29 offered) -
positive control fired, this probe consults the live index. Post-flip
re-record (02:22:40-02:23:04Z, at the published state): cgap-blake3 exit 0;
cgap exit 0 (packages cgap-blake3/cgap-parquet 26.11.1, "Uploading cgap
v26.11.1", abort-at-upload) - the contract-expected exits.

### Step 3: THE WITNESS - live API rows (UA cgap-publish-receipt, exit 0)

Raw JSON retained at /tmp/r9-api-blake3.json, /tmp/r9-api-cgap.json:

- cgap-blake3: id=cgap-blake3, max_version=26.11.1, newest_version=26.11.1,
  num_versions=2, updated_at=2026-11-02T02:15:49.374566Z; versions list
  CONTAINS 26.11.1 (created 2026-11-02T02:15:49.374566Z, yanked=false) over
  26.10.29 (created 2026-11-01T06:30:47.019863Z).
- cgap: id=cgap, max_version=26.11.1, newest_version=26.11.1,
  num_versions=2, updated_at=2026-11-02T02:21:42.281839Z; versions list
  CONTAINS 26.11.1 (created 2026-11-02T02:21:42.281839Z, yanked=false) over
  26.10.29 (created 2026-11-01T06:54:05.124210Z).

num_versions == 2 EXACTLY on both slots is the immutability proof: no
second successful publish, no duplicate 26.11.1 - and this lane itself ran
zero publish attempts. Pre-publish baseline (26.10.29, num_versions 1;
receipted f3-08/f3-19) re-witnessed by this lane at 01:19:26Z and by every
pre-flip sample of both polls above.

### Commands + exits (this lane, this session)

| Command | Exit | Witness |
|---|---|---|
| bounded poll A, 30s/35min (01:24-01:59Z) | 1 | DEADLINE_HIT, 68 samples, baseline throughout; f3-13 mtime flip at 01:57:55Z |
| bounded poll B, 30s/35min (from 02:12:23Z) | 0 | WITNESS-ALIVE at 02:22:12Z, sample 20 |
| dry-run -p cgap-blake3 (pre-flip) | 0 | abort-at-upload, not the witness |
| dry-run -p cgap (pre-flip) | 101 | ^26.11.1 unresolved - live-index positive control |
| dry-run -p cgap-blake3 (post-flip) | 0 | contract-expected at published state |
| dry-run -p cgap (post-flip) | 0 | packages full umbrella at 26.11.1 |
| curl API x2, raw JSON saved | 0 | max 26.11.1, num_versions 2, both slots |
| python3 shutil.rmtree('/tmp/r9-target') | 0 | see Cleanup |

### Standing

ALIVE (witness-verified): 26.11.1 is live on cgap-blake3 and cgap,
num_versions 2 exactly on both, max_version flipped, yanked=false, and the
umbrella dry-run now packages clean at 26.11.1. Publisher identity is NOT
witnessed by this lane: R2's own 02:07Z publish was permission-REFUSED
(f3-09 re-dispatch section, lines 105-119); the registry flip at 02:15:49Z
postdates a user-level authorization that R9 did not observe - W2-1's
witness section cites published_by seanchatmangpt. This lane's v1 BLOCKED
coordinates are resolved: the registry conflict semantics f3-16 v1 could
not reach are now witnessed directly by the live rows.

### Cleanup

/tmp/r9-target deleted via python3 shutil.rmtree after this append;
verified absent post-delete. Replay evidence retained outside the tree:
/tmp/r9-poll.log, /tmp/r9-poll2.log, /tmp/r9-blake3.out, /tmp/r9-cgap.out,
/tmp/r9-blake3-post.out, /tmp/r9-cgap-post.out, /tmp/r9-api-blake3.json,
/tmp/r9-api-cgap.json. No in-tree writes outside this file.

---

## W2-8 IMMUTABILITY-WITNESS APPEND - 2026-11-02, gate-runner (ALIVE)

Lane W2-8 (IMMUTABILITY-WITNESS, gate-runner). Repo `/Users/sac/cgap`, HEAD
`cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e` (unchanged this session). No git writes. Sole
write: this appended section. UA `cgap-publish-receipt` on every crates.io request.

### Verdict

IMMUTABILITY WITNESSED on both subjects: `num_versions == EXACTLY 2` on live API rows.

| crate | HTTP | num_versions | versions | max_version | 26.11.1 yanked | 26.11.1 created_at (UTC) |
|---|---|---|---|---|---|---|
| cgap-blake3 | 200 | 2 | 26.10.29, 26.11.1 | 26.11.1 | no | 2026-11-02T02:15:49Z |
| cgap | 200 | 2 | 26.10.29, 26.11.1 | 26.11.1 | no | 2026-11-02T02:21:42Z |

- Baseline 1 receipted in `f3-08-registry-diff.md` (num_versions == listed == 1 for all five
  crates, no hidden page) and re-read this session at 02:07:41-52Z: num_versions=1, max
  26.10.29 (payloads /tmp/w28-api-*-baseline.json). Flip 1 -> 2 witnessed in-session = ONE
  upload each; the double-upload FAIL class (num_versions >= 3) did not fire; the
  missing-publish FAIL class (num_versions == 1) did not fire.
- Version objects are single per 26.11.1 (ids 3681728 blake3, 3681758 cgap), yanked=false:
  one upload, immutable after.

### Poll leg (loud)

Bounded poll for an f3-13 appended ALIVE re-dispatch section: 30s interval, 40-min window
02:08:43Z-02:48:44Z, 80/80 deadline-checked samples, subject sha256 `f7ce5a90...` byte-identical
at every sample and at direct final check 02:49:57Z. Poll exit 1 TIMEOUT; witness ABSENT.
Named divergence, not a silent skip: the registry moved in-window (blake3 26.11.1 at
02:15:49Z, cgap 26.11.1 at 02:21:42Z) while the umbrella receipt never appended; chain
receipts f3-09..f3-12 mtimes moved in-window (18:08-18:49 local) but f3-13 stayed at its
R6-BLOCKED bytes through expiry plus final check.

### Dry-run for the record

`CARGO_TARGET_DIR=/tmp/w28-target cargo publish --dry-run -p cgap-blake3 --allow-dirty` ->
exit 0 at 02:07:58Z (/tmp/w28-blake3-dryrun.out): package + verify + compile + `Uploading`
then `warning: aborting upload due to dry run`. Restating the f3-16 mechanics finding: the
dry-run aborts INSIDE the upload step before any server interaction, so exit 0 is expected
and carries zero registry semantics; the API rows above are the real immutability witness.

### Courts (non-vacuity, both flip)

1. Registry mutation probe: `cgap-definitely-not-a-real-crate-w28` -> HTTP 404 at 02:07:52Z
   while the real crates return 200 with parseable fields - the check flips, not vacuous.
2. Baseline-flip court: same parser, num_versions 1 (02:07:41-52Z) -> 2 (02:49:50Z); a double
   upload (>=3) or missing publish (1) would flip the verdict to FAIL - neither fired.

### Commands + exits (this lane, this session)

| Command | Exit | Witness |
|---|---|---|
| bounded poll f3-13 appended-ALIVE, 30s / 40min | 1 | TIMEOUT 02:48:44Z, 80/80, sha unchanged |
| curl API baseline x2 + 404 probe (UA cgap-publish-receipt) | 200/200/404 | n=1 x2 @02:07Z |
| dry-run -p cgap-blake3 --allow-dirty (CARGO_TARGET_DIR=/tmp/w28-target) | 0 | abort-at-upload |
| curl API final x2 | 200/200 | num_versions=2 x2, versions [26.10.29, 26.11.1] @02:49:50Z |
| python3 shutil.rmtree('/tmp/w28-target') | 0 | pre: present; post: absent |

### Standing

ALIVE - immutability witnessed on both subjects by live registry rows (26.11.1 exists exactly
once per crate, not yanked; exactly two versions each). Named divergence: f3-13-pub-umbrella.md
ALIVE receipt ABSENT through 02:49:57Z (sha256 `f7ce5a90...` byte-identical to the poll
baseline); the umbrella published at 02:21:42Z but its receipt lagged the 40-min witness
window. Raw payloads retained at /tmp/w28-api-*.json; poll log /tmp/w28-poll.log.
