# DOD-01: Workspace SemVer + Cargo Hygiene Audit

Lane: DOD-01 · date 2026-11-02 · repo /Users/sac/cgap (canonical checkout,
branch main). READ-ONLY audit lane: zero manifest edits, zero git write
commands, no registry contact. Receipt is the only written artifact.

## Subject

23 root-workspace members (`cargo metadata` workspace_members = 23) +
standalone crates: cgap-cli (cli), cgap-fleet-ops, cgap-ocel2, plus
on-disk non-members cgap-verification, csct, wasm-footprint (recorded
as-is; contract named three standalones, the extra three are recorded
for completeness — a doc-yields divergence note, not an expansion of
scope).

## Commands (this session, exits witnessed)

- `cargo metadata --format-version 1 --no-deps` -> /tmp/dod01-meta.json,
  exit 0, 23 members, clean resolution.
- `cargo tree --workspace --duplicate`
  (CARGO_TARGET_DIR=/tmp/dod01-target), exit 0, 4 duplicate groups.
- `grep -nE` field sweeps over root + all member + standalone manifests,
  exit 0.
- `cargo tree` full-graph run, exit 0 (dedup graph consistent).

## Finding class 1 — versions (26.11.2)

All 23 members resolve to 26.11.2 via `version.workspace = true` (root
[workspace.package] `Cargo.toml:47` = "26.11.2"). PASS.

Standalones (recorded as-is):
- cgap-cli `crates/cli/Cargo.toml:6` = 26.11.2 — aligned.
- cgap-fleet-ops `crates/cgap-fleet-ops/Cargo.toml:3` = 26.11.2 —
  aligned.
- cgap-ocel2 `crates/cgap-ocel2/Cargo.toml:3` = 26.11.2 — aligned.
- cgap-verification `crates/cgap-verification/Cargo.toml:4` = 26.11.2.
- csct `crates/csct/Cargo.toml:3` = **0.1.0** — DIVERGENCE
  (finding F1).
- wasm-footprint `crates/wasm-footprint/Cargo.toml:3` = 26.11.2.

## Finding class 2 — edition + rust-version (MSRV)

edition = 2021 everywhere (workspace-inherited or declared), no
divergence. PASS.

rust-version: **the root [workspace.package] does not declare
`rust-version`; zero members declare one** (grep exit 0 with one hit:
`crates/cli/Cargo.toml:8` rust-version = "1.85", standalone only).
MSRV is undeclared at workspace scope — finding F2. Publishable
crates without rust-version publish with the toolchain's MSRV as
effectively unknown to consumers; 1.85 is the de facto value (cli
declares it, edition 2021 floor is 1.56).

## Finding class 3 — license field convention

Canonical convention: per-crate `license-file` pointing at the repo-root
LICENSE.md (P28-SIL v2.1.0, URN
urn:avec:license:p28-sil:v2.1.0-sovereign). No crate carries an SPDX
`license` key — this is the A04 manifest-court design (URN pointer, no
SPDX; `crates/cgap-core/Cargo.toml` comment block cites
docs/jira/v26.11.2/lanes/lane-dod-01-spdx.md). Recorded, not a defect.

Deviations (findings):
- F3a: **cgap-core has NO license-file field** — the only member whose
  `cargo metadata` license_file is None
  (`crates/cgap-core/Cargo.toml` lines 1-9 carry no license key; the
  URN lives only in a comment). Defect: published package would carry
  no license identifier at all.
- F3b: **cgap-ocel2 `license-file = "../LICENSE.md"` resolves to
  crates/LICENSE.md — file does not exist** (`ls crates/LICENSE.md` ->
  No such file). Dangling license path.
- F3c: **csct `license-file = "../LICENSE.md"` — same dangling path**
  (crates/LICENSE.md absent).
- F3d: **cgap-cli and cgap-fleet-ops declare NO license field at all**
  (grep zero hits across both manifests).
- Form census: 21 members use explicit `license-file = "../../LICENSE.md"`;
  3 members inherit `license-file.workspace = true` (bench, cgap-beam,
  cgap-court); 2 members (cgap-core: absent; all others covered) — see
  F3a. Both forms resolve to the same root file; the mixed form is
  recorded, not flagged.

## Finding class 4 — repository / readme

repository: inherited via `repository.workspace = true` (root
`Cargo.toml:49` = https://github.com/seanchatmangpt/cgap). All 23 members
carry it (workspace-inherited or explicit: cgap-verification:2,
csct:8, wasm-footprint:6). cli + fleet-ops + ocel2 declare none
(sub-finding of F3d-class hygiene; publish=false, low severity).

readme: `cargo metadata` reports readme = "README.md" for all 23
members (cargo auto-detection of an in-crate README.md at metadata
time). No manifest declares a readme key explicitly; recorded as-is.

## Finding class 5 — publish flags (PUB-02b re-grep)

20 true / 3 false confirmed on disk this session. Per-crate cite
(Cargo.toml:line):

| member | flag | cite |
|---|---|---|
| audit (cgap-audit) | true | crates/audit/Cargo.toml:9 |
| bench | false | crates/bench/Cargo.toml:9 |
| cgap | true | crates/cgap/Cargo.toml:9 |
| cgap-arrow | true | crates/cgap-arrow/Cargo.toml:8 |
| cgap-beam | false | crates/cgap-beam/Cargo.toml:8 |
| cgap-blake3 | true | crates/cgap-blake3/Cargo.toml:8 |
| cgap-bn3 | true | crates/cgap-bn3/Cargo.toml:9 |
| cgap-bont | true | crates/cgap-bont/Cargo.toml:9 |
| cgap-bosym | true | crates/cgap-bosym/Cargo.toml:9 |
| cgap-branchless | true | crates/cgap-branchless/Cargo.toml:9 |
| cgap-core | true | crates/cgap-core/Cargo.toml:7 |
| cgap-court | false | crates/cgap-court/Cargo.toml:8 |
| cgap-crypto | true | crates/cgap-crypto/Cargo.toml:7 |
| cgap-ffi | true | crates/cgap-ffi/Cargo.toml:9 |
| cgap-gate | true | crates/cgap-gate/Cargo.toml:9 |
| cgap-git | true | crates/cgap-git/Cargo.toml:8 |
| cgap-gpu | true | crates/cgap-gpu/Cargo.toml:2 |
| cgap-p28 | true | crates/cgap-p28/Cargo.toml:7 |
| cgap-parquet | true | crates/cgap-parquet/Cargo.toml:8 |
| cgap-storage | true | crates/cgap-storage/Cargo.toml:9 |
| cgap-integration | true | crates/integration/Cargo.toml:8 |
| cgap-fibo | true | ontologies/fibo/Cargo.toml:10 |
| cgap-cloud | true | ontologies/cloud/Cargo.toml:10 |

Cross-check: `cargo metadata` renders publish=false as `"publish": []`
— exactly bench, cgap-beam, cgap-court. Independent corroboration of
the grep. MATCH with the PUB-02b table
(docs/jira/v26.11.2/audit/pub-02-flag-table-rederivation.md), which
also lists 5 publish=false standalone non-members (cli, verification,
csct, fleet-ops, ocel2 — re-grepped this session, all false) and
wasm-footprint as a publish=true non-member (line 8, true).

## Finding class 6 — cargo tree --duplicate

4 duplicate groups, ALL the same shape: cgap-arrow, cgap-blake3,
cgap-git, cgap-parquet — each v26.11.2 appearing once as the path
member and once in a dev-dependency-only graph rooted at
cgap-integration, whose dev-deps declare registry-form version reqs
(`cgap-arrow = "26.11.2"` etc., crates/integration/Cargo.toml:12-22).

Classification (recorded, not fixed — pre-existing, benign): same
version on both sides, so no version skew; the duplicate exists only
in the dev graph (cargo does not unify a registry dep and a path
member of the same name/version in dev resolution). Zero impact on
published artifacts. A follow-up could pin path+version dev-deps in
cgap-integration to collapse the groups, but this is optional.

## Finding class 7 — cargo metadata resolution

`cargo metadata --format-version 1 --no-deps`, exit 0, 23 workspace
members, zero warnings on stderr. Clean.

## Fences checked (must-not-regress surfaces for this diff)

Diff is a docs-only receipt; the fence surface is the shared tree
remained untouched. Verified: no manifest edited (git status of
crates/*/Cargo.toml unchanged from session start modulo other lanes'
pre-existing modifications — this lane wrote zero of them). A docs
diff cannot regress G1-PUBLISH, gate0, or the boundary gate; those
gates are named here as the audited fences, not run (no subject
changed). If a follow-up fix lane lands F1-F3, its fences are
scripts/release_check.py, tests/manifest_court.rs (cgap-core), and
make gate0.

## Findings ledger (summary)

- F1: csct version 0.1.0 vs fleet convention 26.11.2 (standalone;
  low severity, record-or-align decision for fix lane).
- F2: rust-version/MSRV undeclared at workspace scope; only cli
  (standalone) declares 1.85. Recommend workspace.package rust-version
  + inheritance in a fix lane; perf-neutral.
- F3a: cgap-core missing license-file (only member; cargo metadata
  license_file=None). Highest-severity finding — cgap-core is
  publish=true and would ship with no license key.
- F3b: cgap-ocel2 license-file "../LICENSE.md" dangling (crates/
  LICENSE.md absent).
- F3c: csct license-file "../LICENSE.md" dangling (same missing
  file).
- F3d: cgap-cli + cgap-fleet-ops declare no license field.
- F6 (info): 4 same-version duplicate groups in the dev graph via
  cgap-integration registry-form dev-deps; benign, recorded.

## Standing

ALIVE for the audit: every claim above was re-grepped or re-run this
session against the exact on-disk manifests, with command exits cited
in the Commands section. Publish-flag figures re-read from disk, not
from session memory of PUB-02b (no-overclaiming: receipt-cited counts
re-read at use time). No court flips required — this lane mutated
nothing; there is no mutation subject. Skips: none silent; the three
crates beyond the three named standalones are recorded, not skipped.
