# f3-11-pub-git

Wave F3 item 11 — PUB-GIT lane receipt. Repo `/Users/sac/cgap`. Publish
target: `cgap-git` 26.11.1. Lease honored: this marker is the only file
written. **Zero crates.io publish attempts** — the one-attempt immutability
budget for `cgap-git` 26.11.1 is intact for re-dispatch.

## Standing

BLOCKED by chain law at 2026-11-02T00:48:00Z. Bounded poll (30s cadence,
window 2026-11-02T00:21:05Z -> 00:41:13Z, 40 iterations) for
`f3-10-pub-arrow.md` timed out (exit 3): the file did not exist at any
iteration. It landed at 2026-11-02T00:44:00Z carrying standing **BLOCKED**
(chain-propagated from f3-09), so the gate "require ALIVE" reads red on
content as well as absence. `cargo publish -p cgap-git --allow-dirty` was
never invoked.

## Upstream state (read at 00:45Z)

- f3-03-workspace-green.md: gate RED — `cargo test --workspace` exit 101,
  sole failure `cgap-ffi sovereign::xip_courts::xip_t1_riscv_staticlib_size_court`
  panic at `crates/cgap-ffi/src/sovereign.rs:2399`: T1 FLASH 264,881 B >=
  budget 65,536 B (4.04x over). Verdict: "Publishers must NOT proceed
  past F3-03."
- f3-09 (blake3) BLOCKED at 00:39:13Z, publish never invoked, Retry-After
  00:55:02Z. f3-10 (arrow) BLOCKED at 00:44:00Z, publish never invoked.
- Registry truth at 00:45:55Z: cgap-blake3 / cgap-arrow / cgap-git all
  `max_version 26.10.29` — zero 26.11.1 publishes have fired anywhere in
  the chain.

## Gate evidence (this session)

- `crates/cgap-git/Cargo.toml:3` `version.workspace = true` (grep exit 0);
  root `Cargo.toml:39` `version = "26.11.1"` (workspace inherit).
- `~/.cargo/credentials.toml` present (57 bytes; existence-level check,
  contents never read or printed — token never printed, per publisher
  law).
- Registry baseline at 00:33Z: cgap-git `max_version 26.10.29`,
  `num_versions 1` — publish succeeds only against 26.11.1.
- cgap-git has zero normal dependencies (`[dependencies]` table empty) —
  publish is resolution-independent of the upstream kernels, but the
  chain gate still binds by dispatch law.

## Commands and exits

| command | exit |
|---|---|
| bounded poll f3-10 marker, 30s cadence, ~20 min | 3 (POLL_TIMEOUT 00:41:13Z, 40 iters) |
| `grep -n version crates/cgap-git/Cargo.toml` | 0 (line 3 `version.workspace = true`) |
| root `Cargo.toml` `[workspace.package]` read | `version = "26.11.1"` (line 39) |
| dry-run publish, CARGO_TARGET_DIR=/tmp/f3-11-target | 0 (see note A) |
| `curl -s -A cgap-publish-receipt .../crates/cgap-git` (baseline) | 0 (`max_version 26.10.29`) |

Note A: `cargo publish --dry-run -p cgap-git --allow-dirty` output —
"Packaging cgap-git v26.11.1", "Packaged 22 files, 343.4KiB",
"Verifying cgap-git v26.11.1", "warning: aborting upload due to dry
run"; target dir `/tmp/f3-11-target`, deleted post-run (confirmed
absent).
| `cargo publish -p cgap-git --allow-dirty` | NOT EXECUTED (chain gate red) |
| step-5 API max_version poll | NOT EXECUTED (nothing published) |

## Publish step

NOT EXECUTED. Exactly one attempt is reserved for re-dispatch after the
chain gate goes green; a crates.io version is immutable, and publishing
against a red F3-03 would burn `cgap-git` 26.11.1 permanently.

## Retry-After

900 s from the poll timeout 2026-11-02T00:41:13Z -> retry at/after
**2026-11-02T00:56:13Z** (aligned with the upstream chain Retry-After
00:55:02Z). Re-dispatch should: confirm f3-03 flips green (the
`sovereign.rs:2399` T1 FLASH repair lands, `cargo test --workspace`
exit 0); then confirm f3-09 and f3-10 update to ALIVE with registry
`max_version == "26.11.1"`; then run this lane's steps 3-5 in order
(dry-run gate -> publish ONCE -> bounded API poll of
`https://crates.io/api/v1/crates/cgap-git`, UA `cgap-publish-receipt`,
require `max_version == "26.11.1"`), then update this marker.

## Divergences

1. The contract's poll timed out with the marker absent; `f3-10` then
   landed at 00:44:00Z — after my window — carrying BLOCKED, so the
   "require ALIVE" gate failed on both absence and content.
2. Steps 2-3 (version grep, dry-run) were run concurrently with the
   in-flight poll rather than strictly after it (non-publishing steps;
   disclosed). The dry-run is packaging evidence for re-dispatch, not a
   step-3 gate pass on a green run.

## Re-dispatch attempt 1 (written 2026-11-02T02:05Z)

Standing: **BLOCKED** — the chain gate never went green this window;
publish NOT executed; one-attempt budget for `cgap-git` 26.11.1
PRESERVED (zero publish attempts, cumulative).

### Step 1 — bounded poll

- Gate: `f3-10-pub-arrow.md` gains a re-dispatch section reading ALIVE
  + API-verified 26.11.1.
- Window 2026-11-02T01:23:51Z -> 01:58:53Z (35-min deadline), 30s
  cadence, 70 iterations. Exit **3** (`POLL_TIMEOUT
  2026-11-02T01:58:53Z iters=70`).
- Verdict `ABSENT` at every iteration: f3-10's mtime stayed at its
  00:44:08Z generation for the whole window — no re-dispatch section
  was ever appended upstream.

### Steps 2-3 (concurrent with the poll; non-publishing, disclosed)

| command | exit |
|---|---|
| `grep -n version crates/cgap-git/Cargo.toml` | 0 (`3:version.workspace = true`) |
| `grep -n ^version Cargo.toml` (root) | 0 (`39:version = "26.11.1"`) |
| `cargo metadata --no-deps --format-version 1` | 0 (cgap-git 26.11.1; cgap-arrow 26.11.1; cgap-blake3 26.11.1) |
| `CARGO_TARGET_DIR=/tmp/r4-target cargo publish --dry-run -p cgap-git --allow-dirty` | 0 (Packaging cgap-git v26.11.1; Packaged 22 files, 343.4KiB / 96.4KiB compressed; Verifying cgap-git v26.11.1; `warning: aborting upload due to dry run`) |
| `~/.cargo/credentials.toml` existence check | present (57 bytes; existence-level only — contents never read, token never printed) |

### Steps 4-5

- `cargo publish -p cgap-git --allow-dirty`: **NOT EXECUTED** (gate red;
  one-attempt budget preserved intact for the next re-dispatch).
- Step-5 API max_version poll: NOT EXECUTED (nothing published).

### Registry truth at BLOCK (2026-11-02T02:04:22Z, UA cgap-publish-receipt)

- cgap-git: `max_version 26.10.29`, `num_versions 1`, updated_at
  2026-11-01T06:35:10Z.
- cgap-arrow: `max_version 26.10.29`, `num_versions 1`, updated_at
  2026-11-01T06:30:48Z.
- cgap-blake3: `max_version 26.10.29`, `num_versions 1`, updated_at
  2026-11-01T06:30:47Z.

The upstream chain head remains unpublished: f3-03 still carries the RED
workspace gate (T1 FLASH 264,881 B >= budget 65,536 B at
`crates/cgap-ffi/src/sovereign.rs:2399`), and f3-09 / f3-10 carry
BLOCKED with zero publish attempts. Chain law binds: `cgap-git` 26.11.1
publishes only after blake3 and arrow land 26.11.1.

### Cleanup

- `python3 -c "shutil.rmtree('/tmp/r4-target')"` -> exit 0,
  `exists_after: False` (lease target dir deleted, verified absent).
- Poll script scratch `/tmp/r4_poll.sh` removed (exit 0).

## Retry-After (re-dispatch attempt 1)

900 s from POLL_TIMEOUT 2026-11-02T01:58:53Z -> retry at/after
**2026-11-02T02:13:53Z**. Path unchanged: confirm f3-03 flips green;
confirm f3-09 + f3-10 flip ALIVE with registry `max_version ==
"26.11.1"`; then steps 3-5 in order (dry-run gate -> publish ONCE ->
bounded API poll of `https://crates.io/api/v1/crates/cgap-git`, UA
`cgap-publish-receipt`, require `max_version == "26.11.1"`); then
append to this marker.

## Witness (W2-3-witness-git, gate-runner, 2026-11-02)

- Who: lane W2-3-witness-git (chain witness for the cgap-git leg; no
  publish authority; zero `cargo publish` invocations from this lane).
  Bounded polls on the marker observables (30s cadence): v1
  02:01:45Z->02:09Z (terminated: gate parsed only the top `## Standing`,
  stale after appended re-dispatch sections), v2 02:11:30Z->02:18:30Z
  (terminated on f3-09's REFUSED last-standing; the registry then
  falsified the terminal reading), v3 from 02:19:45Z (window to
  02:59:45Z, still running at witness time).
- When: cgap-git 26.11.1 publish witnessed via API at
  2026-11-02T02:21:47Z (curl exit 0); formal witness read at
  2026-11-02T02:23:42Z (curl exit 0).
- What the registry said (UA cgap-publish-receipt):
  max_version **26.11.1**, num_versions **2 EXACTLY** — 26.11.1 created
  2026-11-02T02:19:00.265596Z (published_by seanchatmangpt, yanked
  false, crate_size 98727, checksum prefix 5a64205d008c0667);
  26.10.29 unchanged (created 2026-11-01T06:35:10Z). No double upload.
  Chain state witnessed the same window: cgap-blake3 26.11.1 (created
  02:15:49Z) and cgap-arrow 26.11.1 (created 02:17:56Z), each
  num_versions 2, read at 02:18:58Z.
- Cross-check (contract item 3): every timestamped claim in this
  marker's BLOCKED sections is consistent with the registry at its own
  timestamp, and no section claims an upload that did not happen; the
  last-standing BLOCKED lines are stale-not-false at witness time (W2-1
  precedent on f3-09). R3/R4 publisher success sections had not landed
  here as of 02:23:42Z — disclosed divergence; the registry record is
  ground truth. Witness standing: **ALIVE**. Contract item 2 gate
  PASSED: max_version == "26.11.1", num_versions == 2 EXACTLY.
