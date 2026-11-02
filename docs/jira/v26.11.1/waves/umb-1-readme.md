# umb-1-readme

Lane UMB-1 (UMBRELLA-README, lane-runner). Repo `/Users/sac/cgap`, branch
`main`, no git writes. Lease: `crates/cgap/README.md` (rewrite) +
`docs/jira/v26.11.1/readme-archive/cgap-umbrella-pre.md` (archive) + this
receipt.

## Standing: ALIVE

## Read-list (all sibling READMEs read in full this session)

crates/: audit, bench, cgap-arrow, cgap-beam, cgap-blake3, cgap-bn3,
cgap-bont, cgap-bosym, cgap-core, cgap-court, cgap-crypto, cgap-ffi,
cgap-gate, cgap-git, cgap-gpu, cgap-p28, cgap-parquet, cgap-storage, cgap
(pre-rewrite), cli, integration, wasm-footprint; ontologies/: cloud, fibo;
root README; plus waves receipts `f3-14-consumer.md`, `coord-publish-record.md`,
`dod-five-gates.md`, `p3-api-verify-all.md` (title-referenced).

## Archive

- Source: `crates/cgap/README.md` (105 lines, v26.10.29-era tutorial).
- Archive: `docs/jira/v26.11.1/readme-archive/cgap-umbrella-pre.md`.
- sha256 both sides: `fd7564e438b7276eaeb4d951bac7da175524f48c17133c95d5426fb288c5a2b6`
  (`shasum -a 256` on file and archive, byte-identical, exit 0).

## Rewrite section list (crates/cgap/README.md, 120 lines, max 100 cols)

1. Intro — what the substrate is (root-README factual frame; A04/A06/A11/
   A17/A19 pins; renamed avec 2026-10-31).
2. Crate map — 22 crates in 5 groups (published kernels / published
   substrate / published O-ABI packs / guarded / membranes+aux), one line
   per crate from ITS README: purpose + test count + standing.
3. Quickstart — `cargo add cgap@26.11.1` + sovereign loop from
   `waves/f3-14-consumer.md` (4/4 PASS, registry-sourced).
4. Morning DoD five gates one-liners + pointer to
   `waves/dod-five-gates.md` (G5 divergence recorded, not laundered).
5. Chain PARETOs one-liners (bench receipts 2026-11-01).
6. Links: waves/ receipt trail, `docs/diataxis/`, readme-archive provenance.

## Commands + exits

| command | exit |
|---|---|
| shasum -a 256 README + archive copy | 0 (byte-identical hashes) |
| awk/wc line-length + count sweeps (final: 120 lines, 0 over 100 cols) | 0 |
