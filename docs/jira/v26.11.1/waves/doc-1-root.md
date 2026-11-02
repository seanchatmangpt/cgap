# DOC-1 root README rewrite receipt

Lane: DOC-1 (lane-runner). Repo /Users/sac/cgap, branch main, no git writes.
Lease: README.md, docs/jira/v26.11.1/readme-archive/, this file.

## Archive

- Source: README.md (84 lines, pre-26.11.1 content)
- sha256 README.md (pre-rewrite): cdb85b437e63f5af3de1a706c400c51e8f6598840b942e9e8deeff982c8a0a32
- sha256 archive copy (docs/jira/v26.11.1/readme-archive/root-README-pre-26.11.1.md):
  cdb85b437e63f5af3de1a706c400c51e8f6598840b942e9e8deeff982c8a0a32 (byte-identical)

## Verification (commands + exits, this session)

- `curl -A cgap-publish-receipt https://crates.io/api/v1/crates/{cgap-blake3,
  cgap-arrow,cgap-git,cgap-parquet,cgap}` — all HTTP 200, max_version 26.11.1
  for all five (published_by seanchatmangpt, created 02:15-02:21Z 2026-11-02).
- Live API truth for the other 17 crates: 26.11.1 for cgap-bont, cgap-bosym,
  cgap-crypto, cgap-gpu; HTTP 404 for cgap-core, cgap-storage, cgap-bn3,
  cgap-gate, cgap-p28, cgap-ffi, cgap-beam, wasm-footprint, cgap-cli,
  cgap-integration, cgap-audit. README publish split (9 published / 8
  guarded / 5 harness + beam) reflects this live sample, superseding
  p3-api-verify-all.md's "17 new crates 404" snapshot (later lanes published
  bont/bosym/crypto/gpu — see p3-pub-bont-bosym.md, p3-pub-aux-final.md).
- Receipts read in full: dod-five-gates.md, bench-chains-ingress.md,
  bench-chains-sovereign.md, bench-chains-bn3.md, bench-chains-seal.md,
  f3-14-consumer.md, coord-publish-record.md, p3-api-verify-all.md (head),
  p3-pub receipts (heads).
- README token-sweep: 0 NUL/control bytes, newline-terminated, all lines
  <= 100 chars (awk length check).

## README section list

1. Header paragraph (substrate description, spec anchors A04/A06/A11/A17/A19)
2. Crates: 9 published at 26.11.1 / 8 guarded (publish=true, registry 404) /
   harness + beam
3. Quickstart: cargo add cgap@26.11.1 + f3-14 consumer surfaces
4. Morning DoD gates table (G1/G2 GREEN, G3/G4 ALIVE, G5 ALIVE-with-
   recorded-divergence) with receipt pointers
5. Chain PARETOs (ingress, sovereign, bn3, seal) with bench receipt pointers
6. Receipt trail pointer to docs/jira/v26.11.1/waves/

## Standing

PARTIAL: README rewrite landed and grounded against live API + receipts;
no executable court run in this docs lane (no anti-vacuity mutation applies
to a markdown rewrite; the archive sha256 equality is the byte-exactness
witness). Divergences recorded below.
