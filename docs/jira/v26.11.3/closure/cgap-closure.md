# cgap v26.11.3 closure seal — wave-closing seal receipt

Date: 2026-11-04. Lane: ZD-105 (seal lane). This receipt's own
actuation touches only two files (this closure + its own receipt
`../receipts/zd-105-cgap-closure.md`); every code deliverable landed via
receipted lanes cited below. Every standing word is re-read from the
cited receipt bodies on disk, never memory.

## Exact subject

`/Users/sac/cgap` main @ `b6d685177d6d62b95708b75627fc1b617265a82d`,
pushed to origin (origin/main verified equal at seal time via
`git fetch && git rev-parse HEAD origin/main`). The untracked
`zd-8-dod006-quiet.md` receipt is landed by this very seal commit.

## Deliverables with standing (from receipt bodies)

- **AoQ kernel (cgap-aoq)**: ALIVE — macro p99 torture ×4 (release
  serial + debug parallel, 22/22 integration binaries; AF-37), RFC-0089
  conformance suite exit 0 ×4 + lib courts 14/0 (RX-2), corridor join
  court 3/0 exit 0 (AF-1, J1 100-iteration real-KEM join, J2 6-gate
  anti-vacuity battery, J3 100k-link scale leg).
- **Crypto kernels**: FIPS-202 ALIVE (AB-2), ChaCha20-Poly1305 ALIVE
  (AB-3), AEAD courts PARTIAL_ALIVE (AB-4), ML-KEM-768 ALIVE with liboqs
  interop both directions (AB-5). dod-006's load-bound flake closed
  quiet: 3× GREEN exit 0, measured min 64KB throughput 261.26 MB/s —
  ~130× the 2 MB/s floor (zd-8).
- **Wave-2 membrane: LANDED** (zd-63 status flip over the four lane
  receipts): W2-A session FFI ALIVE (zd-6), W2-C sidecar ALIVE (zd-13),
  W2-B twin + W2-D admission ALIVE (arcus-side receipts zd-29/zd-42, as
  recorded in zd-63's per-lane table). Rung 8 E2E subsequently landed
  (zd-56: sequential serve loop, E1 real-ML-KEM echo==preseal, E2
  DIGEST_MISMATCH control, E3 reuse + exit-0 janitor); rung 7 gate0 was
  BLOCKED in the zd-79 witness, then closed by the ZD-79 close commit
  `f30580ff` (recipe carries the disclosed `--skip-ea-admission`
  opt-out; byte-stable replay + header compile PASS, rung-7 witnessed).
- **DoD 12/12** per zd-36 (dod-001..dod-012 table, exits 0 across the
  MEASURED rows; dod-011/012 raw commands refuse typed FM-LAW-015,
  disclosed `--skip-ea-admission` variant exit 0 ×2, `written: []`
  determinism) with the zd-55 command-disposition ruling (cgap root
  manufactures no SBBs → pin the flag, not an empty ea.graph.json).
- **Dry-run ladder 20/20 accounted** (zd-43 + zd-97): 7 green exits 0 +
  13 registry-dep-missing self-healing (exit 101, blocked only on
  cgap-arrow ^26.11.3 not yet on crates.io), 0 unclassified, 0
  environment-BLOCKED.
- **XIP**: cfg-stub pattern (cfg-not-target-os none mod imp + fn main
  stub) preserved through the sidecar serve loop (zd-56), exercised by
  the ffi XIP size court (`cargo test -p cgap-ffi --lib
  xip_t1_riscv_elf_size_court`, per zd-56's ladder rows).
- **Corridor real-KEM**: ML-KEM-768 corridor join with liboqs interop
  (AB-5) and the AF-1 join court over a live UDS socketpair.
- **Post-wave convergence**: repo-wide rustfmt convergence (191 files,
  sync ×2 `written: []` post-fmt, corridor 3/3, ffi suite green, check
  exit 0 — HEAD commit b6d68517).

## Receipt count (re-read at seal time)

`ls docs/jira/v26.11.3/receipts | wc -l` → **78** pre-seal; **80**
after this seal lands its two files.

## BLOCKED / DEFERRED (carried, not claimed)

- **crates.io Stage 3 not run**: no real publish occurred; the dry-run
  ladder is the only publish-surface evidence (zd-43/zd-97).
- **Tags unminted**: no v26.11.3 tag minted in cgap.
- **cgap-mcp standalone workspace**: cgap-mcp is not a root workspace
  member; its cli-verify gate runs `-p cgap-cli` against the standalone
  workspace via manifest-path (XC-9).
- dod-006's fence is host-load-dependent by construction (min-of-8
  inner reps; quiet-machine closure per zd-8) — recorded, not repaired.

## Falsifiers

- Any cited receipt's standing word disagrees with its body on disk.
- A cited gate command re-run now exits non-zero (load-bound dod-006
  re-classified per zd-8 if it fires).
- Receipt count in this file disagrees with a fresh `ls | wc -l`.
- origin/main ≠ HEAD for the subject SHA recorded above.

## Verification ladder (this seal's own runs, cgap @ b6d68517)

- `cargo check --workspace` → exit 0 (13 pre-existing cgap-crypto lib
  warnings, disclosed; no errors).
- `make gate0` → exit 0 (sync ×2, sha256 byte-stable replay proof PASS,
  freestanding C header compile PASS).
- `git rev-parse HEAD origin/main` → equal (`b6d68517…`), after
  `git fetch`.

## Addendum: post-seal landings (2026-11-04, authored at c9430086)

The seal above recorded `git rev-parse HEAD origin/main` equal at seal
time (subject SHA **cfdd58ab**). Four commits landed after the seal was
written; none re-grade the seal's claims.

### Post-seal landing list

- `7459848b` — zd-103: fix-forward cgap-ffi aoq_sidecar warnings
  (zd-102 follow-up).
- `c7bfdde4` — land zd-8 receipt (dod-006 quiet-machine close, 3x GREEN
  261 MB/s).
- `4bd598d8` — zd-113: ladder plan refresh — Stage-1 license blocker
  FALSIFIED (never fired 20/20), census pinned 21, Stage 2
  COMPLETE-VERIFIED, Stage 3 admission gate restated.
- `c9430086` — zd-119: registry preflight (token present, zero 26.11.3
  collisions, cgap-aoq first-publish; census 20 publish=true manifests).

### Manifest-class verification

`git diff --stat cfdd58ab..HEAD` (run at c9430086):

```
 crates/cgap-ffi/src/bin/aoq_sidecar.rs             | 14 +---
 docs/jira/v26.11.3/plans/crates-io-ladder-plan.md  | 94 +++++++++++++++-------
 docs/jira/v26.11.3/receipts/zd-103-ffi-warnings.md | 41 ++++++++++
 .../v26.11.3/receipts/zd-119-cgap-preflight.md     | 80 ++++++++++++++++++
 docs/jira/v26.11.3/receipts/zd-8-dod006-quiet.md   | 81 +++++++++++++++++++
 5 files changed, 270 insertions(+), 40 deletions(-)
```

Four of five changed paths are docs/plans/receipts. One path is source
(`crates/cgap-ffi/src/bin/aoq_sidecar.rs`, commit 7459848b): disclosed
as warning-fix only — dead `PAD_LO`/`PAD_HI` consts + orphaned
`AOQ1_TAG_AT` import removed, `&mut pieces` → `&pieces`,
`u16::from(EPOCH)` → `EPOCH`. No behavior change (receipt zd-103:
cargo check 0 warnings in the file, clippy clean). Classified
manifest-class with that disclosure; not a pure docs-only window.

### Receipt count (re-read at addendum time)

`ls docs/jira/v26.11.3/receipts | wc -l` → **83** at c9430086 working
tree (includes zd-113/zd-116 receipt files present but uncommitted at
authoring time; seal-time count was 80).

### Falsifier note

Seal claims remain anchored at **cfdd58ab**. This addendum extends the
closure record; it does not re-grade any seal claim. The "origin/main =
HEAD" seal verification is as-of-cfdd58ab only.
