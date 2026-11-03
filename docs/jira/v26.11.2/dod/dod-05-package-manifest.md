# DOD-05 — clean-tarball `cargo package --list` inclusion audit

Date: 2026-11-02. Lane: DOD-05. Read-only over the shared tree; packaging
staged to `/tmp/dod05-target` (lane build-root lease, /tmp only). No git
commands, no registry contact (`--list` resolves the index read-only).
`--allow-dirty` required: the audit runs against the working tree with
uncommitted wave diffs — every verdict below is a working-tree verdict,
not a HEAD verdict.

Inputs: `docs/jira/v26.11.2/audit/pub-01-dry-run-matrix.md` (16
DRY-RUN-OK set) + `docs/jira/v26.11.2/01-release-gates.md` refreshed
G1-PUBLISH block.

## Method

- `CARGO_TARGET_DIR=/tmp/dod05-target cargo package --list --allow-dirty
  -p <crate>` — sequential, one crate at a time, 20 crates total (the
  16 DRY-RUN-OK set + 4 dep-blocked attempts, see divergence below).
- Per-crate file list captured at `/tmp/dod05/lists/<crate>.txt`
  (stderr at `.err`, all empty except cargo's normal warnings).
- Sizes taken from on-disk working-tree files via `stat -f %z`.
- Sweep greps per list: `docs/jira`, `.DS_Store`, `/tmp/`, license
  line, README line. Zero jira/ds_store/tmp hits across all 20 lists.

## Matrix (crate | files | largest file (B) | license | README | verdict)

| crate | files | largest file | license in list | README | verdict |
|---|---|---|---|---|---|
| cgap-arrow | 44 | src/slab.rs 83,299 | LICENSE.md | yes | PASS |
| cgap-core | 23 | src/frame.rs 29,999 | ABSENT | yes | FLAG (no license key) |
| cgap-blake3 | 15 | src/lib.rs 37,395 | LICENSE.md | yes | PASS |
| cgap-branchless | 19 | tests/branchless_census.rs 28,145 | LICENSE.md | n/a (none on disk) | PASS |
| cgap-storage | 24 | src/atomic_commit.rs 48,383 | LICENSE.md | yes | PASS |
| cgap-git | 67 | src/dag.rs 77,032 | LICENSE.md | yes | PASS |
| cgap-bn3 | 18 | src/unify.rs 124,306 | LICENSE.md | yes | PASS |
| cgap-parquet | 43 | src/footer.rs 87,619 | LICENSE.md | yes | PASS |
| cgap-p28 | 13 | src/asc450.rs 106,446 | LICENSE.md | yes | PASS |
| cgap-crypto | 12 | tests/chatman_courts.rs 30,059 | LICENSE.md | yes | PASS |
| cgap-audit | 10 | src/ocel2_slab.rs 102,188 | LICENSE.md | yes | PASS |
| cgap-gpu | 9 | src/lib.rs 3,413 | LICENSE.md | yes | PASS |
| cgap-bont | 11 | src/shacl_compiler.rs 138,888 | LICENSE.md | yes | PASS |
| cgap-bosym | 8 | src/slab.rs 62,441 | LICENSE.md | yes | PASS |
| cgap-fibo | 14 | tests/wave9_integration_courts.rs 26,984 | LICENSE.md | yes | PASS |
| cgap-cloud | 13 | src/lib.rs 28,823 | LICENSE.md | yes | PASS |
| cgap | 21 | (list-only) | LICENSE.md | yes | PASS (list mode) |
| cgap-gate | 10 | (list-only) | LICENSE.md | yes | PASS (list mode) |
| cgap-ffi | 21 | (list-only) | LICENSE.md | yes | PASS (list mode) |
| cgap-integration | 11 | (list-only) | LICENSE.md | yes | PASS (list mode) |

Every list includes `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`,
`.cargo_vcs_info.json`, src/, tests/. Every list's largest file is
< 1 MiB (largest anywhere: cgap-bont src/shacl_compiler.rs 138,888 B).
Fixture check: cgap-fibo `fibo_sec_dbt_v1.oabi` = 496 B;
cgap-cloud `tosca_cimi_v1.oabi` = 1,552 B — nowhere near the 10 MiB
test-fixture ceiling and below the 1 MiB flag line.

## Findings

1. **cgap-core ships no license artifact** (`FLAG`, the one real
   defect). `crates/cgap-core/Cargo.toml:6-14` carries the P28-SIL
   URN only as a COMMENT; there is no `license` or `license-file` key,
   so cargo includes no LICENSE file in the package
   (`/tmp/dod05/lists/cgap-core.txt` has zero license lines — verified
   grep). All 19 other audited crates include LICENSE.md. Fix: add
   `license-file = "../../LICENSE.md"` (the cgap-branchless pattern,
   `crates/cgap-branchless/Cargo.toml:5`) or a workspace-inherited
   key. NOT fixed by this lane: crates/cgap-core/Cargo.toml is not in
   the DOD-05 owned-file set. Escalate to the coordinator / a repair
   lane before first publish of cgap-core.
2. **cgap-cloud packages its internal receipt doc** (observation, not
   a defect flag): `RECEIPT.md` (24,963 B) is included in the tarball
   alongside README.md. Internal-notes criterion is `docs/jira`
   content — zero hits anywhere — but this is receipt-adjacent
   documentation in a published artifact; coordinator may want it
   excluded via `include`/`exclude` or accepted as intentional.
3. **cgap-branchless has no README** — nothing to include; not a
   defect (cargo includes a README only when one exists on disk).
4. Hygiene clean across all 20 lists: no `.DS_Store`, no `/tmp/`
   paths, no untracked scratch scripts, no `docs/jira` content.

## Divergences from the dispatch / PUB-01

- The 4 crates PUB-01 recorded as BLOCKED-DEP-ORDER (cgap, cgap-gate,
  cgap-ffi, cgap-integration; rc=101 under full `cargo package
  --no-verify --allow-dirty`) RESOLVE and LIST CLEAN under `cargo
  package --list --allow-dirty` (all rc=0 this session). `--list`
  resolves path deps from the workspace without the registry-version
  constraint walk that full packaging performs. Their dep-order
  BLOCKED standing from PUB-01 stands for full-package mode; this
  lane's list-mode PASS does NOT close it. Recorded honestly as
  mode-dependent, not as a fix.
- "4 BLOCKED-DEP-ORDER: record the dep-resolution failure" — nothing
  to record in list mode; both modes are documented above.
- cargo nightly lacks `--out-dir`; no tarballs needed for `--list`
  (none were produced; `/tmp/dod05-target/package/` absent).

## Fence law (fences this audit could break)

- Read-only over the shared tree; the single file written is this
  receipt. No Cargo.toml, lib.rs, or script edits — G1-PUBLISH,
  boundary gate, zero-string gate unaffected.
- No court flip required: this is an audit receipt, not a gate. Its
  anti-vacuity surface is the FLAG in Finding 1 — cgap-core's missing
  license key is a concrete, greppable defect a fix lane must close
  (mutation: add the key, re-run `--list`, LICENSE.md must appear).

## Commands (exit in parentheses)

- `cargo package --list --allow-dirty -p <crate>` x20 sequential
  (16x0 DRY-RUN-OK set, 4x0 blocked-set list-mode) — CARGO_TARGET_DIR
  =/tmp/dod05-target
- `grep -c license|readme|docs/jira|.DS_Store /tmp/dod05/lists/*.txt` (0)
- `stat -f %z` sweeps for largest-file and fixture sizes (0)
- `cargo metadata --no-deps` (0) — located cgap-audit at crates/audit,
  cgap-fibo/cgap-cloud at ontologies/{fibo,cloud} (dir != crate name)

## Standing

ALIVE for list-mode inclusion audit (all commands + exits this
session, above). cgap-core license FLAG: OPEN. Full-package dep-order
BLOCKED standing for the 4: unchanged (PUB-01 remains the receipt of
record for that mode).

## JSON terminus

{"lane":"DOD-05",
"files_written":["docs/jira/v26.11.2/dod/dod-05-package-manifest.md"],
"commands":["cargo package --list --allow-dirty -p <crate> x20 (20x0)",
"grep hygiene sweep x20 lists (0 hits)",
"stat size sweep (0)",
"cargo metadata --no-deps (0)"],
"courts_run":["anti-vacuity: cgap-core missing-license FLAG is the concrete defect; fix flips by LICENSE.md appearing in --list"],
"standing":"ALIVE (list mode); cgap-core license FLAG open; 4-crate full-package dep-order BLOCKED unchanged",
"divergences":["4 PUB-01 BLOCKED-DEP-ORDER crates list cleanly in --list mode — mode-dependent, full-package BLOCKED stands",
"cgap-core ships no license artifact (Cargo.toml has no license/license-file key; URN comment only)",
"cgap-cloud packages RECEIPT.md (24,963 B) — coordinator call",
"crate dirs differ from crate names for cgap-audit/cgap-fibo/cgap-cloud (crates/audit, ontologies/{fibo,cloud})"]}
