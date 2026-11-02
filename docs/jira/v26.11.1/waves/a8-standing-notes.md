# A-8 Standing-Doc Reconcile Receipt (2026-11-02)

Lane: A-8 STANDING-DOC-RECONCILE (lane-runner). Repo: /Users/sac/cgap.
Lease honored: docs/jira/v26.11.1/lanes/*.md status lines + this receipt. No git writes.

## Live crates.io witness (2026-11-02, curl -A cgap-publish-receipt, api/v1/crates/<name>)

PUBLISHED-26.11.1: cgap-blake3 (2), cgap-arrow (2), cgap-git (2),
cgap-parquet (2), cgap umbrella (2), cgap-bont (1), cgap-bosym (1),
cgap-gpu (1), cgap-crypto (1) — num_versions in parens.

NOT on crates.io (API "NOT-FOUND" this session): cgap-gate, cgap-bn3,
cgap-p28, cgap-ffi, cgap-audit, cgap-fibo, cgap-cloud, cgap-integration,
cgap-beam (publish=false by design), cgap-wasm-footprint.

## Status-line edits (one line each, verified `git diff` = +1 insertion)

| lane doc | crate | line |
|---|---|---|
| A01-arrow-slab.md | cgap-arrow | status update PUBLISHED-26.11.1, witnessed 2026-11-02 |
| A02-parquet-mmap.md | cgap-parquet | status: PUBLISHED-26.11.1, witnessed 2026-11-02 |
| A03-git-merkle.md | cgap-git | - Status bullet, PUBLISHED-26.11.1, witnessed 2026-11-02 |
| A04-triumvirate-isomorphism.md | cgap-blake3 + umbrella cgap | status update, witnessed 2026-11-02 |
| A06-bont-scalars.md | cgap-bont | - Status update bullet, witnessed 2026-11-02 |
| A07-bosym-slab.md | cgap-bosym | - Status update bullet, witnessed 2026-11-02 |
| A16-chatman-pqc.md | cgap-crypto | status update bullet, witnessed 2026-11-02 |

cgap-gpu (PUBLISHED-26.11.1, witnessed 2026-11-02) has no dedicated lane
doc (A01-A20); its publish receipt lives at
waves/p3-pub-beam-gpu.md — recorded here instead of inventing a doc.

## integration / wasm-footprint / beam

No dedicated lane doc exists in lanes/ for any of the three (grep over
lanes/*.md: mentions only, no owning doc) — per task clause "(IF the doc
exists)", no lane doc edited. States recorded here:
- cgap-integration: UNSUPPORTED-for-now (coordinator ruling; API NOT-FOUND).
- cgap-wasm-footprint: UNSUPPORTED-for-now (coordinator ruling; API NOT-FOUND).
- cgap-beam: no lane receipt; guarded publish=false
  (witnessed waves/p3-pub-beam-gpu.md:62, publish attempt not made).

## Divergences from dispatch context

1. Dispatch said gate/bn3/crypto/p28/ffi/audit/fibo/cloud "in flight or
   landed — verify live". Live check: ONLY cgap-crypto landed; gate, bn3,
   p28, ffi, audit, fibo, cloud are NOT on crates.io (NOT-FOUND). No
   PUBLISHED line was written for them.
2. cgap-gpu and the umbrella cgap have no dedicated lane doc; cgap-gpu
   state recorded in this receipt, umbrella+blake3 noted on A04 (nearest
   spec surface).

## Commands + exits

- curl loop over 19 crate names (crates.io API) exit 0
- python3 insert script exit 0 (7 files, "ok" x7)
- git diff --stat per file exit 0 (each file +1 line from this lane)
- grep -c receipt line x7 exit 0 (1 each)

Standing: ALIVE (all edits executed and byte-verified on disk this session).
