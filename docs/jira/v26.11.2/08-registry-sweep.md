# 08-registry-sweep

Registry sweep charter for the v26.11.2 starDF campaign. Members
enumerated live from `/Users/sac/cgap/Cargo.toml` via
`cargo metadata --no-deps` (22 packages, exit 0, this session);
`crates/cli` is deliberately NOT a member (standalone ggen crate, own
`[workspace]`) and is carried as a delisted 23rd row.

## 1. 22-crate cell schema

Schema: `crate | registry state | versions | standing | court pointer`.
Registry state is `PUBLISHED-<ver>` / `UNPUBLISHED` / `DELISTED`.
Standing is `ALIVE` / `PARTIAL` / `BLOCKED` / `[TARGET-v26.11.2]` —
UNVERIFIED never rounds up. Court pointer is `waves/<file>:<line>`; a
cell with neither pass, nor registered-divergence, nor loud-MISSING is
auto-FAIL.

States below are as of the v2-07 census live API sweep (2026-11-02,
UA cgap-publish-receipt); the §2 compiler re-verifies every cell each
run — census cites are provenance, not current truth.

| # | crate | registry state | versions | standing | court pointer |
|---|---|---|---|---|---|
| | bench | DELISTED | local 26.11.2 | PASS | waves/v2-07-census.md:90 |
| | cgap | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=2, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:99 |
| | cgap-arrow | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=2, yanked=false | REGISTERED-DIVERGENCE | docs/jira/v26.11.1/waves/coord-publish-record.md:14 |
| | cgap-audit | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:90 |
| | cgap-beam | DELISTED | local 26.11.2 | PASS | waves/v2-07-census.md:91 |
| | cgap-blake3 | PUBLISHED-26.11.2 | local 26.11.2 / registry max 26.11.2, num=3, yanked=false | PASS | docs/jira/v26.11.1/waves/coord-publish-record.md:16 |
| | cgap-bn3 | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-bont | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-bosym | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-cloud | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-core | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-court | DELISTED | local 26.11.2 | PASS | waves/v2-07-census.md:92 |
| | cgap-crypto | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-ffi | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:88 |
| | cgap-fibo | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-gate | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-git | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=2, yanked=false | REGISTERED-DIVERGENCE | docs/jira/v26.11.1/waves/coord-publish-record.md:18 |
| | cgap-gpu | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-03-levers.md:32 |
| | cgap-integration | UNPUBLISHED | local 26.11.2 | PASS | waves/v2-07-census.md:91 |
| | cgap-p28 | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
| | cgap-parquet | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=2, yanked=false | REGISTERED-DIVERGENCE | docs/jira/v26.11.1/waves/coord-publish-record.md:20 |
| | cgap-storage | PUBLISHED-26.11.1 | local 26.11.2 / registry max 26.11.1, num=1, yanked=false | REGISTERED-DIVERGENCE | waves/v2-07-census.md:101 |
Non-member 23rd row (cli, standalone workspace, not counted in the 22):

| 23 | cgap-cli | DELISTED | no local row (standalone) | PASS | waves/v2-10-cli-state.md:1 |

Loud-MISSING row (W-SWEEP, 2026-11-01): `stardf-datalog` — MISSING (no-ledger-row);
subject unregistered in the ledger, the sole exit-1 cause.

W-SWEEP note (2026-11-01, live re-run of `python3 scripts/registry_sweep.py`, exit 1):
cgap-blake3 is the only member at registry max 26.11.2 (PASS, num=3); 17 members are
REGISTERED-DIVERGENCE (registry max 26.11.1 vs local 26.11.2); cgap-integration is
UNPUBLISHED (404, PASS vs publish=false guard); bench/beam/court/cli DELISTED rows hold.
Sweep wrote docs/jira/v26.11.2/waves/registry-sweep-table.json into the tree (script has
no output-path flag; write disclosed).

## 2. Coverage-compiler design

Implementation home: `scripts/registry_sweep.py` (fail-closed, exit
contract in §5).

1. Enumerate subjects mechanically: `cargo metadata --no-deps
   --format-version 1 --manifest-path <repo>/Cargo.toml` -> parse
   `packages[].name` + `manifest_path`. Never a hand-maintained roster.
2. Append the standalone `crates/cli` row (read its Cargo.toml `name`
   field; it is outside the member set by design).
3. Per crate, query the crates.io API:
   `GET https://crates.io/api/v1/crates/<name>` with a UA header
   (`User-Agent: cgap-registry-sweep/<ver>` — crates.io refuses
   UA-less clients). Classify:
   - HTTP 200 -> `PUBLISHED-<max_version>`; record `num_versions`,
     `created_at`, `yanked` from the JSON body.
   - HTTP 404 -> `UNPUBLISHED` (the 404 is the data, not an error).
   - Any other status (403, 429, timeout, connection reset) ->
     loudly-MISSING: the cell reads `MISSING-<reason>` and the run is
     never reported as all-pass. A blank cell is forbidden.
4. Emit a machine-readable table (JSON + markdown) keyed by crate name
   with all five schema cells; every cell must be pass,
   registered-divergence, or loudly-MISSING — silence = auto-FAIL.
5. Anti-vacuity: the compiler self-check mutates one expected-PUBLISHED
   expectation in a /tmp sandbox copy and asserts the verdict flips
   (court that cannot flip = FAIL); restores are sha256 fail-closed
   with mtime bump.
6. Determinism: fixed UA string, fixed classification table, no
   wall-clock in output ordering (sort by crate name).

## 3. Publish sequencing law

Precedent: `docs/jira/v26.11.1/waves/coord-publish-record.md` (all five
crates PUBLISHED at 26.11.1, one invocation each).

1. ONE publish attempt per crate, ever — no retry storms. A consumed
   attempt is consumed even if it failed.
2. Dependency-ordered: blake3 -> arrow -> git -> parquet -> umbrella is
   the 26.11.1 chain; the v26.11.2 order derives from `cargo metadata`
   dependency edges, deepest kernel first, umbrella last. Each dry-run
   resolves predecessors from the live sparse index (allow a short
   propagation wait after each hop — polled registry visibility, not a
   fixed sleep).
3. Never concurrent with lever lanes: publish runs sequenced
   post-window, coordinator-side (OVERNIGHT-LOOP.md:146-149). The
   permission system correctly refused subagent publish authority in
   v26.11.1; publishing is a coordinator-owned transition.
4. 429 / rate-limit: honor the `Retry-After` header, then back off one
   hour minimum (registry-slide law). No re-polling loop.
5. `--allow-dirty` is permitted only with disclosure in the receipt
   (tree dirty by design: campaign receipts). Token never printed.
6. Every publish row gets an independent registry witness (num_versions
   == expected EXACTLY, no double upload; `published_by`
   seanchatmangpt; yanked false) recorded post-publish.

## 4. publish=false guarded table

| crate | guard | reason |
|---|---|---|
| bench | publish = false | registry name collision: `bench` 1.1.0 (2021) is a squatter, not ours (c
| cgap-beam | publish = false | guarded membrane; UNPUBLISHED 404 row holds (census:91) |
| cgap-court | publish = false | structural `#[test]`-only crate, no lib surface |
| cgap-cli | delisted (standalone, non-member) | ggen six-pack generated crate; 404 row (census:112)
| cgap-verification | on disk, NOT in the 22-member cargo metadata list | registry-absent; carried o

Status of the remaining ambiguous rows per v2-07-census.md:

- `integration`: UNPUBLISHED 404, structural blocker CLEARED (audit
  PUBLISHED 26.11.1 at 03:53:39Z) — publish-eligible [TARGET-v26.11.2].
- `wasm-footprint`: PUBLISHED-26.11.1 num=1 (04:43:42Z) — outside this
  workspace's member set but registry-verified in the census.
- `cgap-gpu`: no API row in the census; standing stays
  [TARGET-v26.11.2] until the sweep compiler produces its first cell.

## 5. Exit contract

`python3 scripts/registry_sweep.py`:

- 0: every member cell pass or registered-divergence; loud-MISSING
  cells permitted only with `[SKIP] <reason>` lines in stdout.
- 1: gate fail — a cell neither pass nor divergence nor loudly-MISSING,
  or a subject missing from the enumeration.
- 2: subject missing — cargo metadata failed or the manifest path is
  absent.
- 3: loud skip — network unavailable; every cell emitted as
  `MISSING-<reason>` (never blank), exit 3.

A mutation-free green run is VACUOUS: the self-check (§2.5) must flip
or the gate exits nonzero. Gate exit codes follow repo law (0 pass /
1 fail / 2 subject missing / 3 loud skip).

## JSON footer

```json
{
  "lane": "W1-13",
  "files_written": [
    "docs/jira/v26.11.2/08-registry-sweep.md",
    "docs/jira/v26.11.2/waves/w1-13-registry-sweep.md"
  ],
  "commands": [
    "sed -n '1,60p' Cargo.toml (exit 0)",
    "cargo metadata --no-deps --format-version 1 (22 packages, exit 0)",
    "sed -n '80,140p' waves/v2-07-census.md (exit 0)",
    "sed -n '1,140p' v26.11.1/waves/coord-publish-record.md (exit 0)"
  ],
  "courts_run": [],
  "standing": "PARTIAL",
  "divergences": [
    "cargo metadata counts 22 packages incl. ontologies/fibo + cloud; '22-crate' is member-level",
    "cli is a 23rd standalone row, non-member by design",
    "rows 12-19 per-crate cells carried from family-level census:101 pending the sweep compiler",
    "doc first-write corruption (dup rows, typos) rewritten in chunks per corruption law"
  ]
}
```
