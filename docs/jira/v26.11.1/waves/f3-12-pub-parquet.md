# F3-12 Pub-Parquet Receipt

Lane: Wave F3 item 12 — PUB-PARQUET (lane-runner), chain position 4. Repo
`/Users/sac/cgap`, branch main, HEAD `cfd085a9fd230696b9dcdb437aa3e6b0aea2e60e`
(read-only). Sole write: this file. No git writes. Registry token never read
or printed (`~/.cargo/credentials.toml` presence-checked only). ONE publish
attempt: NOT fired — reserved for re-dispatch.

## Verdict: BLOCKED (precondition gate)

Step 1 failed: `waves/f3-11-pub-git.md` absent after bounded-poll expiry.
Publish leg not executed (chain order 11 -> 12 preserved; cgap-parquet itself
is publishable, see dry-run below). Retry-After: 300s.

## Coordinates

1. Poll: 30s interval, 40-iteration budget (<=20 min), started
   2026-11-02T00:19:40Z; exited 1 TIMEOUT at 2026-11-02T00:42:16Z
   (drift ~2 min under load; deadline-checked). Marker ABSENT every check.
2. cgap-git (item 11's subject) never moved: `max_version 26.10.29` at
   2026-11-02T00:39:23Z and `cgap` census at 00:46:05Z shows all five
   crates 26.10.29. Item 11's lane process: zero matching processes at
   00:42Z; `/tmp/f3-11-target/package` stalled 2026-11-01T16:31:48 local
   (00:31:48Z). Root cause: item 11 died mid-publish; f3-09..f3-19 absent
   (corroborated: `f3-20-terminus.md`, `f3-16-replay.md`).
3. cgap-parquet registry slot unchanged: `max_version 26.10.29`
   num_versions 1 at 2026-11-02T00:48Z (HTTP 200, UA cgap-publish-receipt,
   payload `/tmp/f3-12-api-cgap-parquet.json`).

## Gates that PASSED (as-of-use-time)

- Version: root `Cargo.toml:39` = `26.11.1`; `cargo metadata --no-deps`
  resolves `cgap-parquet 26.11.1` (exit 0; re-read 00:49Z). Note: the
  crate manifest carries `version.workspace = true`, so the dispatch's
  "grep cgap-parquet/Cargo.toml = 26.11.1" is satisfied by inheritance,
  not by a literal in that file (divergence recorded below).
- Dry-run: `CARGO_TARGET_DIR=/tmp/f3-12-target cargo publish --dry-run
  -p cgap-parquet --allow-dirty` -> exit 0 at 00:49:13Z; packaged 39
  files 653.3KiB; `warning: aborting upload due to dry run` — nothing
  uploaded.

## Commands + exits (this lane, this session)

| Command | Exit | Note |
|---|---|---|
| bounded poll f3-11 (30s/20min) | 1 | TIMEOUT 00:42:16Z, marker ABSENT |
| `sed -n '39p' Cargo.toml` + `cargo metadata --no-deps` | 0 | 26.11.1 resolved |
| `cargo publish --dry-run -p cgap-parquet --allow-dirty` | 0 | abort-at-upload |
| `curl` API cgap-parquet (UA cgap-publish-receipt) | 0 | HTTP 200, 26.10.29 |
| `python3 shutil.rmtree('/tmp/f3-12-target')` | 0 | pre True -> post False |
| `cargo publish -p cgap-parquet --allow-dirty` (real) | NOT RUN | gate failed |

## Divergences

1. Version-gate form: `crates/cgap-parquet/Cargo.toml` holds no version
   literal (`version.workspace = true`); gate satisfied via root
   `[workspace.package]` inheritance + `cargo metadata` resolution.
2. Dry-run leg ran although step 1 failed (f3-16 precedent: safe leg with
   disclosed divergence, evidentiary value for retry). Real publish
   remained gated and unfired.
3. Terminus receipt `f3-20-terminus.md` (00:34-00:38Z) declared the wave
   terminal before several lanes (incl. this one) finished their budgets;
   its BLOCKED rows are as-of-timestamp. This receipt supersedes its
   cgap-parquet row with grounded 00:49Z evidence.

Standing: BLOCKED. Coordinates: f3-11 marker ABSENT (poll exit 1,
00:42:16Z) + item 11 process dead + registry 26.10.29 on all five slots.
Retry-After 300s: re-dispatch this lane only after
`f3-11-pub-git.md` lands ALIVE; dry-run + ONE publish then replayable
as-is.

## Re-dispatch (R5 chain position 4, written 2026-11-02T02:44Z)

Standing: **ALIVE** — `cgap-parquet` 26.11.1 live on crates.io; chain
order preserved; this lane's publish invocation withheld (superseded,
see step 4). Token never read: `~/.cargo/credentials.toml`
existence-check only (57 bytes, contents never read/printed).

### Step 1 — gate (f3-11 marker ALIVE + API-verified 26.11.1)

- Poll v1 01:25:07Z -> killed ~02:04Z (harness 30-min background limit;
  script deadline was 35 min; zero content flips, registry 26.10.29
  throughout). Resumed per coordinator order as poll v2 02:11:02Z.
- Gate satisfied on evidence, not by the poll parser:
  - Registry: `cgap-git max_version 26.11.1` witnessed by this lane
    (curl exit 0) at 02:19:14Z and independently by W2-3-witness-git
    (02:21:47Z, 02:23:42Z): num_versions 2 EXACTLY, created
    02:19:00.265596Z, yanked false, published_by seanchatmangpt.
  - Marker: `f3-11-pub-git.md` mtime 02:27:39Z; appended W2-3 witness
    section reads "Witness standing: **ALIVE**. Contract item 2 gate
    PASSED."
  - Chain order witnessed: blake3 26.11.1 created 02:15:49Z, arrow
    02:17:56Z, git 02:19:00Z, parquet 02:20:08Z (crates.io created_at).

### Step 2 — version verify (workspace inheritance)

- `crates/cgap-parquet/Cargo.toml:3` `version.workspace = true` (grep
  exit 0); root `Cargo.toml` `[workspace.package] version = "26.11.1"`;
  `cargo metadata --no-deps` resolves `cgap-parquet 26.11.1` (exit 0,
  01:20Z).

### Step 3 — dry-run gate

- `CARGO_TARGET_DIR=/tmp/r5-target cargo publish --dry-run -p
  cgap-parquet --allow-dirty` -> exit 0 at 01:24Z, re-run exit 0 at
  02:39:12Z immediately before the publish step ("Packaging
  cgap-parquet v26.11.1", 39 files 653.3KiB, verify green,
  `warning: aborting upload due to dry run` — nothing uploaded).

### Step 4 — publish: SUPERSEDED, not fired

- Pre-flight 02:39:20Z: `cgap-parquet max_version` was ALREADY 26.11.1,
  num_versions 2. Full record: created 2026-11-02T02:20:08.018411Z,
  published_by seanchatmangpt, yanked false, crate_size 181371,
  checksum 028649a3...31b44c.
- `cargo publish -p cgap-parquet --allow-dirty` was NOT invoked by this
  lane: the deliverable already existed (published by a parallel
  authorized actor at 02:20:08Z, in chain order 68s after cgap-git);
  a duplicate invocation is a guaranteed HTTP-400 with zero
  consequence. This lane's one-attempt budget: unspent.
  Cumulative attempts on 26.11.1: 1 (successful).
- Content identity PROVEN byte-exact: local
  `cargo package --no-verify -p cgap-parquet --allow-dirty` of the
  shared tree -> sha256
  `028649a34bc76401a802ed745fc44b191c11363c25bd6cb7a3d455f8ac31b44c`,
  181371 B — EQUALS the registry checksum and size exactly.

### Step 5 — bounded API poll

- 3 consecutive passes 02:42:31Z -> 02:42:53Z (10s cadence, 10-iter
  budget, UA cgap-publish-receipt): `max_version == "26.11.1"`.
  API_POLL_PASS.

### Commands + exits (this lane, this session, UTC)

| Command | Exit | Note |
|---|---|---|
| poll v1 f3-11 gate (30s/35min) | killed | harness bg limit ~02:04Z, 0 flips |
| poll v2 (30s/35min, resumed) | killed | parser-miss disclosed; gate met on evidence |
| `grep -n version crates/cgap-parquet/Cargo.toml` | 0 | `3:version.workspace = true` |
| `cargo metadata --no-deps` | 0 | cgap-parquet 26.11.1 |
| `cargo publish --dry-run -p cgap-parquet --allow-dirty` | 0 | 01:24Z and 02:39:12Z |
| pre-flight `curl` crates.io cgap-parquet | 0 | 26.11.1 num 2 (02:39:20Z) |
| `cargo package --no-verify ... --allow-dirty` | 0 | sha256 == registry, 181371 B |
| `cargo publish -p cgap-parquet --allow-dirty` | NOT INVOKED | superseded; budget unspent |
| bounded API poll (3 consecutive) | 0 | API_POLL_PASS 02:42:53Z |
| `shutil.rmtree('/tmp/r5-target')` | 0 | below |

### Divergences

1. Publish leg SUPERSEDED, not fired: the deliverable landed at
   02:20:08Z via a parallel authorized actor; firing a knowingly-doomed
   duplicate would carry zero consequence. Coordinator may order a
   verification-only close if an invocation receipt is required.
2. Poll parser narrowness: gate read via last capital-S "Standing:"
   line; the witness section's phrasing ("Witness standing: ALIVE")
   was invisible to it. Gate passed on registry + witness evidence.
3. Poll v1 killed by the harness 30-min background limit (script
   deadline 35 min); resumed as v2 on coordinator order after a
   transient API 429.
4. --allow-dirty disclosed: publish legs carry it; the working tree is
   dirty (coordinator owns git transitions). Registry record for
   26.11.1: yanked=false.
5. Poll v1/v2 f3-11 content flips: zero while windowed; the marker
   flipped at 02:27:39Z (v1 already killed, v2 parser-missed it).

Standing: ALIVE (deliverable live, chain order preserved, byte-exact
content identity witnessed). Retry-After: none — no retry owed; the
26.11.1 slot is immutable and occupied.

## Witness (W2-4-witness-parquet, gate-runner, 2026-11-02)

- Who: lane W2-4-witness-parquet (chain witness, cgap-parquet leg; no
  publish authority; zero `cargo publish` invocations from this lane).
- Bounded poll: 30s cadence, window 2026-11-02T02:04:18Z -> 02:44:19Z
  (40-min deadline; exit 3 POLL_TIMEOUT; log /tmp/f3w4-poll.log). Both
  markers read NOT_ALIVE at every iteration — parser miss disclosed:
  f3-11's ALIVE witness line ("Witness standing: **ALIVE**", landed
  02:27:39Z) does not match a line-start standing/verdict anchor, and
  this marker's own re-dispatch section landed 02:44:38Z, 19s after
  the deadline. Phase 2 never opened; gate evaluated on the registry.
- When: registry read 2026-11-02T02:46:17Z (curl exit 0, HTTP 200, UA
  cgap-publish-receipt, payload /tmp/f3w4-parquet-final.json).
- What the registry said: max_version **26.11.1**, num_versions **2
  EXACTLY** (26.11.1 id 3681753 + 26.10.29 id 3672933; no others).
  26.11.1: created 2026-11-02T02:20:08.018411Z, published_by
  seanchatmangpt, yanked false, crate_size 181371, checksum
  028649a34bc76401a802ed745fc44b191c11363c25bd6cb7a3d455f8ac31b44c.
  No double upload; this lane's baseline at 02:02:37Z was still
  26.10.29 / num_versions 1.
- Cross-check (contract item 3): every timestamped claim in this
  marker's re-dispatch section (created 02:20:08.018411Z, num_versions
  2, published_by seanchatmangpt, yanked false, crate_size 181371,
  checksum 028649a3...31b44c) MATCHES the live registry exactly; the
  earlier "Standing: BLOCKED" lines are stale-not-false at their own
  timestamps (consistent with the 02:02:37Z baseline read). Publish-
  SUPERSEDED disclosure corroborated: exactly one successful publish
  occupies the 26.11.1 slot (cumulative attempts 1). Witness standing:
  **ALIVE**. Contract item 2 gate PASSED: max_version == "26.11.1",
  num_versions == 2 EXACTLY.
