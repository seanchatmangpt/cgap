# cgap

Umbrella facade for the cgap sovereign columnar/deductive compute substrate:
one dependency, zero logic of its own (four `pub use` lines, version tracks
the workspace at 26.11.1). Underneath: 128 B frames (A04), B-ONT slots (A06),
binary-N3 deduction (BN3, A11), P28-SIL licensing (A17, 80 B attestations),
Arrow C Data Interface kernels, BLAKE3, Parquet, Git packfile v2 — all
`#![no_std]`, zero external deps, byte-layout-up, behind four sovereign C-ABI
entrypoints (admit/eval/commit/verify, A19). Renamed from `avec` 2026-10-31.
Every claim cites a sibling README or a waves/ receipt; witnessed state only.

## Crate map (22 crates; witnessed 2026-11-01/02 per sibling READMEs)

Nine PUBLISHED at 26.11.1, yanked=false — `waves/coord-publish-record.md`,
`waves/p3-api-verify-all.md`.

### Published kernels (26.11.1, on crates.io)

| crate | purpose (from its README) | tests | standing |
|---|---|---|---|
| cgap-arrow | Arrow C Data Interface mask/filter/transpose | 177/0 | PUBLISHED-26.11.1 |
| cgap-blake3 | full BLAKE3 suite, KAT vs test_vectors.json | 27/0 | PUBLISHED-26.11.1 |
| cgap-git | packfile v2 + DAG, dual git/receipt registers | 88/0 | PUBLISHED-26.11.1 |
| cgap-parquet | RLE hybrid, sealed verify-chain, 0 B decode | 216/0 (236 std) | PUBLISHED-26.11.1 |

### Published substrate (26.11.1, on crates.io)

| crate | purpose | tests | standing |
|---|---|---|---|
| cgap-bont | BLAKE3-addressed B-ONT slot table (SlotId law) | 35/0 | PUBLISHED-26.11.1 |
| cgap-bosym | binary symbol slab + ingress-limiter diode | 14/0 | PUBLISHED-26.11.1 |
| cgap-crypto | Chatman sealer + lease machinery | 24/0 | PUBLISHED-26.11.1 |
| cgap-gpu | Metal zero-copy bridge, Apple Silicon | 1/1 | PUBLISHED-26.11.1 |

### Published O-ABI packs

| crate | purpose | tests | standing |
|---|---|---|---|
| cgap-cloud | TOSCA/CIMI lexemes, emitted `.oabi` (1,552 B) | 13/0 | PUBLISHED-26.11.1 |
| cgap-fibo | FIBO pack, emitted `.oabi` (496 B) | 21/0 | ALIVE; 404 at its README's witness time |

### Guarded (`publish = true`, registry 404 at each README's read time)

| crate | purpose | tests | standing |
|---|---|---|---|
| cgap-core | 128 B CgapFrameHeader, DMA ring, isomorphism | 75/0 | ALIVE |
| cgap-storage | WAL two-slot atomic commit, power-cut sweeps | 113x3 sweep | ALIVE |
| cgap-bn3 | binary N3 deduction, 24 B statement pin, arena | 111/0 | ALIVE, unpublished |
| cgap-gate | penstock ingress gate, 10M-frame 0-alloc corpus | 11/0 | ALIVE (workspace-local) |
| cgap-p28 | 80 B P28LicenseAttestation, 640-bit sweep | 42/0 | ALIVE (workspace-local) |
| cgap-ffi | sovereign C-ABI cdylib (admit/eval/commit/verify) | 54/0 | ALIVE (workspace-local) |

### Membranes and aux (never published)

| crate | purpose | tests | standing |
|---|---|---|---|
| cgap-beam | BEAM/AtomVM NIF membrane (cdylib) | 2/2 | ALIVE, `publish = false` |
| cgap-audit | CountingAlloc + POWL + OCEL2 slabs | 37/0 | workspace-local |
| cgap-court | whole-repo court_verdict harness (no src/) | 14/0 | workspace-local |
| cgap-integration | cross-crate C-ABI roundtrip suites | 20/20 | ALIVE, 404 at read |
| wasm-footprint | wasm32-wasip1 link wrapper (4,629 B) | 8/8 | ALIVE on its target |
| cgap-cli | ggen-generated `cgap` binary (standalone ws) | 19/0 | workspace-local |
| bench | Divan chain benches, PARETO attribution | benches | workspace-local |

## Quickstart

```sh
cargo add cgap@26.11.1
```

The sovereign loop below is the registry-sourced consumer program of
`waves/f3-14-consumer.md` (`cargo new` + `cargo add` + build/run, exit 0, 4/4 PASS):

```rust
let d = cgap::cgap_blake3::hash(b"abc");          // [1] KAT, b3sum-grounded
assert_eq!(d.len(), 32);

let mut pb = cgap::cgap_git::PackBuilder::new();  // [3] pack v2 -> pack+idx
pb.push(b"hello\n");
let (pack, idx) = pb.finish().unwrap();
assert_eq!(&pack[0..4], b"PACK");

let v: [u32; 5] = [7, 7, 7, 9, 9];                // [4] RLE round-trip
let e = cgap::cgap_parquet::rle::rle_encode(&v, 8);
let mut out = [0u32; 5];
cgap::cgap_parquet::rle::rle_decode(&e, &mut out).unwrap();
assert_eq!(out, v);
```

Step [2] (arrow build/freeze/readback) ran 4/4 against the 26.10.29 registry
surface; 26.11.1 adds the frame/deduction lanes. The v26.10.29-era README is
archived byte-exact at `docs/jira/v26.11.1/readme-archive/cgap-umbrella-pre.md`
(sha256 `fd7564e438b7276eaeb4d951bac7da175524f48c17133c95d5426fb288c5a2b6`).

## Morning DoD five gates (one-liners; dod-five-gates.md is authoritative)

Receipts: `waves/dod-five-gates.md` + the five gate markers.

- G1 workspace purity GREEN: build+test exit 0, census 1025 pass / 0 fail.
- G2 binary/symbol purity GREEN: court_verdict 13/0, kill_rate 1.0000 (19/19).
- G3 1M-row ledger ALIVE: 4-of-4 seal+eval court, 14/0 exit 0.
- G4 640-bit spoliation ALIVE: 2560 mutations / 2516 real deaths, p28 42/0.
- G5 bare-metal silicon ALIVE*: ELF 150/65536 B flash, 0/32768 B SRAM.
  (*size law only; XIP rule execution BLOCKED-on-owner, recorded not laundered.)

## Chain PARETOs (measured 2026-11-01, bench profile)

- chains-ingress: digest fold (BLAKE3) ~85% of segment / ~91% of chain.
- chains-sovereign: A02 sealed-block commit 87.6% of chain (N=4096).
- chains-bn3: unify join 81.2% at 65k rows, 99.9% at 1,048,576.
- chains-seal: no 80% stage - metering ~45% / seal+verify ~35% / validate ~19%.

Receipts: `waves/bench-chains-*.md`; method in `crates/bench/README.md`.

## Links

- Receipt trail: `docs/jira/v26.11.1/waves/` (lane finish/audit receipts, gate
  markers, publish chain, bench PARETOs, `v26111-campaign-ledger.md`).
- Diataxis docs: `docs/diataxis/` (receipts `waves/dia-1`..`dia-6`).
- Readme provenance: `docs/jira/v26.11.1/readme-archive/` (sha256'd, `INDEX.md`).
