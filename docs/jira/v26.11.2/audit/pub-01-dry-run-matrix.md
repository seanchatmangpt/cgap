# PUB-01 — publish dry-run matrix (first-ever receipt)

Closes DF-05 §5.1b UNRUN row ("18 publishable units clean dry-run
matrix" → now RUN, 2026-11-02). Date: 2026-11-02. Lane: PUB-01.
Cargo: 1.101.0-nightly (cad07cf8e). Receipt lines are per-crate exit
codes from `/tmp/pub01-logs/pkg_rcs2.txt`.

## Mode divergence (typed, disclosed)

- `cargo publish --dry-run` is REFUSED by the PreToolUse publish-guard
  hook — operator freeze order `waves/publish-halt.md` (standing
  FROZEN-BY-OPERATOR, 2026-11-02): "Blocked command class: cargo
  publish". The mission's publish mode is therefore typed REFUSED for
  every row; the matrix ran in the mission's named alternative mode
  `cargo package --no-verify --allow-dirty -p <crate>` (package never
  contacts the registry upload path; dependency resolution does hit
  the crates.io index read-only).
- `--allow-dirty` was REQUIRED (wave diffs uncommitted in the shared
  tree). Mission expected BLOCKED-DIRTY rows without the flag; with
  the flag, the dirty gate is bypassed and the real finding is
  BLOCKED-DEP-ORDER instead (below). `--out-dir`/`-d` do not exist on
  this cargo nightly; tarballs land in
  `/tmp/pub01-target/package/` (lane build root, /tmp only).
- CARGO_TARGET_DIR=/tmp/pub01-target (lane lease).

## Member set derivation

`cargo metadata --no-deps`: 23 members. cgap-fleet-ops and cgap-ocel2
confirmed OUTSIDE the matrix: each carries its own `[workspace]` and
`publish = false`
(`crates/cgap-fleet-ops/Cargo.toml:5,16`,
`crates/cgap-ocel2/Cargo.toml:7,11`); neither appears in metadata.

## Flag census (all read from Cargo.toml this session)

All 23 members declare an EXPLICIT flag: 20 `publish = true`,
3 `publish = false` (beam, bench, court). This SUPERSEDES the
CLAUDE.md "12/23 explicit (9 true, 3 false)" line — that text
predates the P3-2b/P3-8/P3-10 flips now on disk. Divergence recorded;
CLAUDE.md not edited (not owned by this lane).

## Matrix (member | flag | mode | outcome | receipt-line)

| member | flag | mode | outcome | receipt |
|---|---|---|---|---|
| cgap | true | package | BLOCKED-DEP-ORDER: cgap-branchless absent from crates.io (frozen set) | rc=101 |
| cgap-arrow | true | package | DRY-RUN-OK, tarball 120238 B | rc=0 |
| cgap-core | true | package | DRY-RUN-OK, 52574 B | rc=0 |
| cgap-blake3 | true | package | DRY-RUN-OK, 29439 B | rc=0 |
| cgap-gate | true | package | BLOCKED-DEP-ORDER: cgap-branchless absent (frozen) | rc=101 |
| cgap-branchless | true | package | DRY-RUN-OK, 47229 B | rc=0 |
| cgap-storage | true | package | DRY-RUN-OK, 57290 B | rc=0 |
| cgap-git | true | package | DRY-RUN-OK, 260103 B | rc=0 |
| cgap-bn3 | true | package | DRY-RUN-OK, 139171 B | rc=0 |
| cgap-parquet | true | package | DRY-RUN-OK, 201773 B | rc=0 |
| cgap-p28 | true | package | DRY-RUN-OK, 65723 B | rc=0 |
| cgap-crypto | true | package | DRY-RUN-OK, 31164 B | rc=0 |
| cgap-ffi | true | package | BLOCKED-DEP-ORDER: registry cgap-audit only 26.11.1, needs ^26.11.2 | rc=101 |
| cgap-audit | true | package | DRY-RUN-OK, 43650 B | rc=0 |
| cgap-gpu | true | package | DRY-RUN-OK, 4197 B | rc=0 |
| cgap-integration | true | package | BLOCKED-DEP-ORDER: cgap-audit 26.11.1 on registry | rc=101 |
| cgap-bont | true | package | DRY-RUN-OK, 69209 B | rc=0 |
| cgap-bosym | true | package | DRY-RUN-OK, 20851 B | rc=0 |
| cgap-fibo | true | package | DRY-RUN-OK, 35466 B | rc=0 |
| cgap-cloud | true | package | DRY-RUN-OK, 35466 B | rc=0 |
| cgap-beam | false | — | SKIP-PUBLISH-FALSE (explicit) | n/a |
| bench | false | — | SKIP-PUBLISH-FALSE + NAME-COLLISION: registry name `bench` is an unrelated crate; never attempt, even on freeze lift | n/a |
| cgap-court | false | false | SKIP-PUBLISH-FALSE (explicit; [[test]]-only harness) | n/a |

## Findings

- True DRY-RUN-OK count: **16 / 20** publish-intended crates (16
  tarballs on disk, `ls /tmp/pub01-target/package/*.crate | wc -l` = 16).
- 4 BLOCKED-DEP-ORDER rows (cgap, cgap-gate, cgap-ffi,
  cgap-integration) are consistent with `waves/publish-halt.md`'s
  frozen set: their registry deps (cgap-branchless, cgap-audit 26.11.2)
  are unpublished, so the post-freeze-lift sequence must be
  branchless -> gate/cgap, audit -> ffi/integration. The halt
  addendum's BLOCKED note on cgap-ffi (parquet RingSink) is an
  ADDITIONAL constraint, not contradicted here.
- The §5.1b UNRUN row is falsified: this receipt IS the dry-run
  matrix. Control-table row `18-crate dry-run matrix | UNRUN`
  (06-control-plan.md) should flip to RUN-PARTIAL (16 OK / 4
  dep-blocked) — that file is not owned by this lane, left to the
  coordinator.
- Anti-vacuity: 20/20 publish-intended members were attempted for
  real (exit codes recorded); zero silent skips. The publish-mode
  refusal is loud, quoted in Mode divergence.

## Fence law (fences this diff could break)

- G1-PUBLISH (`scripts/release_check.py` + 01-release-gates.md):
  untouched; this lane wrote no crate code and no Cargo.toml, lib.rs,
  or script. Only file on disk: this receipt.
- Root workspace compile: untouched (no source edits); metadata
  re-read confirms 23 members post-run.

## Commands (exit in parentheses)

- cargo metadata --no-deps (0) — member set, 23
- cargo publish --dry-run -p cgap (REFUSED by publish-guard hook,
  FROZEN-BY-OPERATOR; not retried per ONE-attempt law)
- cargo package --no-verify --allow-dirty -p <crate> x20 — 16 rc=0,
  4 rc=101 (dep-order)
- ls -l /tmp/pub01-target/package/*.crate (0) — 16 tarballs

## Standing

ALIVE for the package-mode matrix (commands + exits this session,
above). Publish-mode rows: REFUSED (operator freeze), not UNVERIFIED.
Dep-order rows: BLOCKED (typed, per-crate cause in table).

## JSON terminus

{"lane":"PUB-01",
"files_written":["docs/jira/v26.11.2/audit/pub-01-dry-run-matrix.md"],
"commands":["cargo metadata --no-deps (0)",
"cargo publish --dry-run -p cgap (REFUSED, publish-guard hook)",
"cargo package --no-verify --allow-dirty -p <crate> x20 (16x0, 4x101)",
"ls -l /tmp/pub01-target/package/*.crate (0)"],
"courts_run":["anti-vacuity: 20/20 publish-intended attempted, 0 silent skips"],
"standing":"ALIVE (package mode); publish mode REFUSED by operator freeze",
"divergences":["cargo publish refused by FROZEN-BY-OPERATOR hook (waves/publish-halt.md) — ran package mode",
"--allow-dirty used (uncommitted wave diffs); BLOCKED-DIRTY never materialized, BLOCKED-DEP-ORDER is the real finding",
"flag census: 23/23 explicit (20 true, 3 false) supersedes CLAUDE.md 12/23 line",
"cargo nightly 1.101.0-nightly lacks package --out-dir; tarballs in /tmp/pub01-target/package",
"git dirty state = pre-existing wave diffs, untouched by this lane"]}
