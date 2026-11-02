# cgap

Sovereign columnar/deductive compute substrate: fixed-width columnar frames
(A04 CgapFrameHeader, 128 B), binary ontology slots (A06/B-ONT), string-free
deduction (BN3 binary N3, A11), and spoliation-enforced P28-SIL licensing
(A17, 80 B attestations). Kernels are `#![no_std]` with zero external
dependencies — each format is implemented from the byte layout up (branch-free
inner loops, 64-byte-aligned buffers, LSB-first validity bitmaps), exposed
through four sovereign C-ABI entrypoints (admit / eval / commit / verify,
A19) and a `cgap` umbrella facade. Renamed from `avec` 2026-10-31.

## Crates

22 crates. Registry status below is live crates.io API truth sampled
2026-11-02 (UA cgap-publish-receipt; receipt:
docs/jira/v26.11.1/waves/coord-publish-record.md + p3-api-verify-all.md).

Published at 26.11.1 (9):

| crate | what it is |
|---|---|
| cgap | Umbrella facade re-exporting the kernel crates. |
| cgap-arrow | Arrow C Data Interface columnar kernels, no_std. |
| cgap-blake3 | BLAKE3 tree hash, no_std; NEON/AVX2 Merkle reduction. |
| cgap-git | Packfile v2 writer: stored blobs, .idx v2, in-crate SHA-1/CRC32. |
| cgap-parquet | Parquet kernels: bit-unpacking, SWAR RLE, page decode. |
| cgap-bont | Binary ontology scalars/slots (B-ONT). |
| cgap-bosym | Binary symbol slab. |
| cgap-crypto | Keyed BLAKE3 crypto: sealed transcripts, Merkle helpers. |
| cgap-gpu | Metal zero-copy bridge for aligned frame buffers (macOS). |

Guarded, manifest `publish = true`, not yet on the registry (live API 404
at sampling; publish chain receipts under waves/):

| crate | what it is |
|---|---|
| cgap-core | CgapFrameHeader — 128 B fixed-width frame wire format (A04). |
| cgap-storage | WAL framing + persistence (A05). |
| cgap-bn3 | Binary N3 deduction engine (A11-A13). |
| cgap-gate | SHACL bitmask validation (A08/A14). |
| cgap-p28 | P28-SIL 640-bit spoliation + licensing (A16/A17). |
| cgap-ffi | Sovereign C-ABI cdylib (A19). |

Harness / structural crates (never published): cgap-beam (BEAM NIF
membrane, `publish = false`), cgap-audit (CC=1 branch-audit cdylib), bench
(line-rate benches), cgap-court (court_verdict harness), cgap-cli
(standalone ggen-generated `cgap` binary), cgap-integration (cross-crate
C-ABI roundtrip tests), wasm-footprint (wasm32-wasip1 link wrapper).

## Quickstart

```sh
cargo add cgap@26.11.1
```

The umbrella re-exports the kernel crates. Real consumer surfaces
(witnessed registry-sourced end-to-end: f3-14-consumer.md —
`cargo new` + `cargo add` + `cargo build --release` + `cargo run`,
exit 0, 4/4 PASS):

```rust
// cgap-blake3: known-answer hashes, b3sum-grounded
cgap_blake3::hash(b"abc");

// cgap-arrow: build -> freeze -> read back a batch
// BatchBuilder -> Int32 column -> finish -> row_count / format "i" slice

// cgap-git: PackBuilder -> pack + idx -> parse_idx_v2 -> checksum refusal

// cgap-parquet: RLE round-trip -> truncation refusal
```

At 26.10.29 (the version the consumer receipt exercised) the registry
surface predates the 26.11.1 frame/deduction lanes; 26.11.1 adds them.

## Morning DoD gates

Receipts: docs/jira/v26.11.1/waves/{gate-1-workspace,gate-3-ledger-proof,
f3-04-courts-green,gate-4-spoliation,gate-5-silicon,dod-five-gates}.md.
Aggregate verdict (dod-five-gates.md, first-hand re-witnessed):

| gate | standing | witness |
|---|---|---|
| G1 workspace purity | GREEN | build+test exit 0, census 1025 pass / 0 fail |
| G2 binary/symbol purity | GREEN | court_verdict 13/0, kill_rate 1.0000 (19/19) |
| G3 1M-row ledger | ALIVE | 4-of-4 seal+eval court, 14/0 exit 0 |
| G4 640-bit spoliation | ALIVE | 2560 mutations / 2516 real deaths, p28 42/0 |
| G5 bare-metal silicon | ALIVE* | ELF 150/65536 B flash, 0/32768 B SRAM |

G5*: ALIVE on the size law only — the linked ELF's production sections are
court DEMO bytes and XIP rule execution stays BLOCKED-on-owner (recorded
in gate-5-silicon.md, not laundered).

## Chain PARETOs (bench receipts, 2026-11-01)

| chain | dominant stage | receipt |
|---|---|---|
| ingress | digest fold ~85% of segment / ~91% of chain (BLAKE3) | bench-chains-ingress.md |
| sovereign | commit (A02 sealed-block Parquet) 87.6% | bench-chains-sovereign.md |
| bn3 deduction | unify join 81.2% at 65k -> 99.9% at 1M | bench-chains-bn3.md |
| seal/governance | three-way: meter 45.4% / crypto 35.2% / validate 19.3% | bench-chains-seal.md |

## Receipt trail

Full execution history: docs/jira/v26.11.1/waves/ (per-lane finish and
audit receipts, gate markers, publish chain, bench PARETO receipts,
campaign ledger v26111-campaign-ledger.md).
