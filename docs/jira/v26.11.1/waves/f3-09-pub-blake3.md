# f3-09-pub-blake3

Wave F3 item 9 — PUB-BLAKE3 lane receipt. Repo `/Users/sac/cgap`. Publish
target: `cgap-blake3` 26.11.1. Lease honored: this marker is the only file
written. **Zero crates.io publish attempts** — the one-attempt immutability
budget is intact for re-dispatch.

## Standing

BLOCKED at the marker gate, 2026-11-02T00:39:13Z. Poll window
2026-11-02T00:18:45Z -> 2026-11-02T00:39:13Z (~20 min, 30s cadence):
`f3-03-*.md` never landed as a stable file. Marker inventory at deadline:
01 version-lock, 02 regen, 04 courts-green, 05 auth, 06 pins,
07 package-audit, 08 registry-diff all present; `03` ABSENT.
One transient artifact matched `f3-03-*.md` at ~00:30:10Z (a mid-write
directory whose `ls` printed `benches`; gone by 00:30:40Z — see
Divergences). Because 03 of 03/04/05 gates never arrived, the lane BLOCKed
before publishing, by design.

## Gate evidence (read at 00:39Z)

- 01 version-lock: workspace `Cargo.toml:39` version = "26.11.1";
  `cargo pkgid -p cgap-blake3` -> `...#26.11.1` (exit 0).
- 04 courts-green: no `standing` line; green expressed as all legs exit 0,
  kill_rate 1.0, VACUOUS GATE negative control fired, "No leg red".
- 05 auth: "ALIVE (presence-level)"; `~/.cargo/credentials.toml` present
  (existence checked; contents never read or printed).
- 07 package-audit: "Standing: PASS (two flagged rows, no contamination)".
- 02 regen / 06 pins / 08 registry-diff: present, no gate role for blake3.

## Commands and exits

| command | exit |
|---|---|
| poll loop `ls waves/f3-0N-*.md` for N=01..08, 30s cadence | 2 (deadline, 03 missing) |
| `grep -n version crates/cgap-blake3/Cargo.toml` | 0 (`version.workspace = true`, line 3) |
| `cargo pkgid -p cgap-blake3` | 0 (`#26.11.1`) |
| `curl -s -A cgap-publish-receipt crates.io/api/v1/crates/cgap-blake3` | 0 (baseline) |
| `cargo publish --dry-run -p cgap-blake3 --allow-dirty` | 0 (DIAGNOSTIC) |

Baseline curl (00:22Z): crates.io `max_version: 26.10.29`, `num_versions:
1` — publish succeeds only against 26.11.1. Dry-run ran with
`CARGO_TARGET_DIR=/tmp/f3-9-target`.

The dry-run is DIAGNOSTIC ONLY (run after the gate went red, out of
sequence — disclosed): it proves the crate packages and verifies clean at
26.11.1 (`Verifying cgap-blake3 v26.11.1` ... `warning: aborting upload due
to dry run`). It is NOT step-3 pass evidence. `/tmp/f3-9-target` deleted
post-run (confirmed absent).

## Publish step

NOT EXECUTED. `cargo publish -p cgap-blake3 --allow-dirty` was never
invoked: the gate (01..08 present, 03/04/05 ALIVE) never went green, and a
published crates.io version is immutable — exactly one attempt is reserved
for the re-dispatch.

## Retry-After

900 s from 2026-11-02T00:40:02Z -> retry at/after 2026-11-02T00:55:02Z.
Re-dispatch should: confirm `f3-03-*.md` landed and reads green; re-check
04/05; then dry-run (step 3 gate), publish ONCE, API-verify
`max_version == "26.11.1"` via
`curl -s -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap-blake3`,
update this marker.

## Divergences

1. `f3-03` absent at deadline — the sole BLOCK cause.
2. `f3-04-courts-green.md` carries no `standing`/ALIVE wording; green is
   expressed as all-exit-0 legs + "No leg red". Treated as green on
   content, disclosed here.
3. Torn read: the poll matched a transient `f3-03-*` path at ~00:30:10Z
   (`grep: benches: Is a directory`) that vanished by 00:30:40Z — read as
   mid-write debris from a sibling lane, not a red standing.
4. Diagnostic dry-run executed after the red gate (out of dispatch order).

## Re-dispatch (R2, 2026-11-02)

Gate poll: bash loop, 30s cadence, window 2026-11-02T01:22:08Z ->
2026-11-02T01:57:10Z (deadline hit, poll exit 2). All four markers LANDED
AND CLAIMED ALIVE IN-WINDOW (citations; file mtimes 01:24-01:42Z):
- gate-1-workspace.md:9 "Verdict: **GREEN**" (mtime 01:42:32Z; poll ALIVE 01:42:39Z)
- gap-1-sovereign.md:12-16 three sovereign entrypoints ALIVE (mtime 01:31:13Z; poll ALIVE 01:31:08Z)
- r12-bench-tmp.md:49 "ALIVE: ... R2 publish unblocked" (mtime 01:24:29Z)
- r14-pin-final.md:3 "Status: ALIVE" (mtime 01:27:31Z)
Divergence 1: r12/r14 phrase ALIVE across lines ("## Standing" heading then
"ALIVE:" line; "Status: ALIVE"), unmatched by a single-line
standing-ALIVE regex, so the poll logged them PRESENT-NOT-ALIVE through
deadline; judged GREEN on content by direct read (same treatment as f3-04
in the prior attempt). Divergence 2: first poll launch (01:21:13Z) killed
-- zsh does not word-split `for f in $FILES`; relaunched under bash at
01:22:08Z.

Pre-publish checks: crates/cgap-blake3/Cargo.toml:3
`version.workspace = true`; root Cargo.toml:39 `version = "26.11.1"`;
`cargo pkgid -p cgap-blake3` -> ...#26.11.1 exit 0;
crates/cgap-blake3/tests/bench_tmp.rs ABSENT (5 real suites only);
~/.cargo/credentials.toml presence-level only (contents never read).

Step 3 (dry run): `CARGO_TARGET_DIR=/tmp/r2-target cargo publish --dry-run
-p cgap-blake3 --allow-dirty` -> exit 0 at ~02:05Z (13 files, 109.3KiB,
Verifying clean, "aborting upload due to dry run").

Step 4 (publish, single attempt): `cargo publish -p cgap-blake3
--allow-dirty` issued ONCE at 2026-11-02T02:07Z and REFUSED pre-execution
by the Claude Code permission system (auto-mode classifier: publish to the
public crates.io registry is irreversible; only user input or permission
settings authorize it -- coordinator/task messages are not user consent).
cargo never executed; NO bytes uploaded; the one-attempt invocation is
consumed for this lane dispatch (whether the budget survives, given zero
upload, is a coordinator decision recorded here). NO Retry-After exists --
this is an authority refusal, not a registry failure; retry is gated on
USER-level publish authorization (e.g. an explicit user-issued publish or
a user-added Bash permission rule for `cargo publish -p cgap-blake3
--allow-dirty`), never on time.

Consequence-free proof: crates.io re-read at 2026-11-02T02:12:48Z ->
max_version 26.10.29, num_versions 1 (unchanged from baseline).

Standing: REFUSED (authority edge). Chain R3-R6/witnesses/terminus remain
gated behind this publish. Cleanup: /tmp/r2-target removed via
python3 shutil.rmtree, verified absent.

## Witness (W2-1-witness-blake3, gate-runner, 2026-11-02)

- Who: lane W2-1-witness-blake3 (chain witness, no publish authority;
  zero publish attempts from this lane). Poll: 30s cadence from 02:00:30Z,
  Re-dispatch section detected 02:13:34Z reading REFUSED; bounded window
  honored (deadline ~02:39Z, not needed).
- When: publish witnessed via API at 2026-11-02T02:16:09Z (curl exit 0);
  marker re-read 02:17:57Z at unchanged hash
  e1f182df5ca12c2aef0dc628e4f10c6d.
- What the registry said: max_version **26.11.1**, num_versions **2
  EXACTLY** (26.11.1 created 2026-11-02T02:15:49.374566Z, published_by
  seanchatmangpt, yanked false; 26.10.29 created 2026-11-01T06:30:47Z),
  via `curl -s -A cgap-publish-receipt
  https://crates.io/api/v1/crates/cgap-blake3`. num_versions == 2, not 3
  -- no double upload. Lane standing: **ALIVE**.
- Cross-check: every timestamped marker claim verified -- dry-run exit 0
  (~02:05Z, diagnostic-only as labeled); REFUSED-at-02:07Z consistent with
  registry 26.10.29/1 at 02:05:32Z and 02:10:33Z (this lane's own poll
  reads) and with the marker's 02:12:48Z proof; upload created_at
  02:15:49.374566Z postdates the marker's last write, so "Standing:
  REFUSED" above is stale-not-false at witness time. No contradiction, no
  FAIL.
- Divergence: the actuating lane's success entry had not been appended to
  this marker as of 02:17:57Z; the registry record above is the ground
  truth. Baseline for this witness: 26.10.29 / num_versions 1 at
  2026-11-02T01:58:59Z.
