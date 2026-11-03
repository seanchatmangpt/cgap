# cgap — umbrella facade for the sovereign columnar/deductive substrate

Umbrella facade for the cgap sovereign columnar/deductive compute
substrate: one dependency, zero logic of its own (four `pub use`
lines, version tracks the workspace at 26.11.2). Underneath: 128 B
frames (A04), B-ONT slots (A06), binary-N3 deduction (BN3, A11),
P28-SIL licensing (A17, 80 B attestations), Arrow C Data Interface
kernels, BLAKE3, Parquet, and the 23-module Git substrate kernel —
all `#![no_std]`, zero external deps, byte-layout-up, behind four
sovereign C-ABI entrypoints (admit/eval/commit/verify, A19). Renamed
from `avec` 2026-10-31. Every claim cites a sibling README or a
waves/ receipt; witnessed state only.

## Crate map (23 workspace packages; `cargo metadata` re-derived 2026-11-02)

Root members: 21 crates + the cgap umbrella + one ontology crate
(`cgap-fibo`/`cgap-cloud` count within the 21). NOT members:
wasm-footprint, cgap-verification, cli, csct (standalone
`[workspace]` tables — `audit/pub-01-manifest-census.md`).
Publish flags: all 21 crate members declare explicit flags, 17 true /
4 false (bench, beam, court, branchless); the default-publishable set
is empty (`audit/pub-01-manifest-census.md`). Prior publish record:
nine PUBLISHED at 26.11.1, yanked=false
(`waves/coord-publish-record.md`, `waves/p3-api-verify-all.md`); the
26.11.2 republish is sequenced post-window (publish law, one attempt
per crate). Test counts below are the 26.11.2 crate-reality census
(`waves/crate-reality/`) unless marked as-of-date; rows without a
26.11.2 census carry their 26.11.1 receipt with an as-of marker.

### Published kernels

| crate | purpose (from its README) | tests (census) | standing |
|---|---|---|---|
| cgap-arrow | Arrow C Data Interface mask/filter/transpose | 134/0 | prior PUBLISHED-26.11.1 |
| cgap-blake3 | full BLAKE3 suite, KAT vs vectors.json | 27/0, 1 ignored | prior PUBLISHED-26.11.1 |
| cgap-git | 23-module Git kernel: pack v2, rings, DAG | 89/0, 2 ignored | prior PUBLISHED-26.11.1 |
| cgap-parquet | RLE hybrid, sealed verify-chain, 0 B decode | 222/0 | prior PUBLISHED-26.11.1 |

### Published substrate

| crate | purpose | tests (census) | standing |
|---|---|---|---|
| cgap-bont | BLAKE3-addressed B-ONT slot table (SlotId law) | 35/0 | prior PUBLISHED-26.11.1 |
| cgap-bosym | binary symbol slab + ingress-limiter diode | 14/0 | prior PUBLISHED-26.11.1 |
| cgap-crypto | Chatman sealer + lease machinery | 25 total (1+18+6) | prior PUBLISHED-26.11.1 |
| cgap-gpu | Metal zero-copy bridge, Apple Silicon | membrane census REAL | prior PUB-26.11.1 |

### Published O-ABI packs

| crate | purpose | tests | standing |
|---|---|---|---|
| cgap-cloud | TOSCA/CIMI lexemes, emitted `.oabi` | 13/0 (26.11.1 as-of) | prior PUB-26.11.1 |
| cgap-fibo | FIBO pack, emitted `.oabi` | 21/0 (26.11.1 as-of) | prior PUBLISHED-26.11.1 |

### Guarded surface (`publish = true`, registry-absent at last read)

| crate | purpose | tests (census) | standing |
|---|---|---|---|
| cgap-core | 128 B CgapFrameHeader, DMA ring, isomorphism | 24/0 | ALIVE |
| cgap-storage | WAL two-slot atomic commit, power-cut sweeps | 5/0 (target readout) | ALIVE |
| cgap-bn3 | binary N3 deduction, 24 B statement pin, arena | 119/0 | ALIVE |
| cgap-gate | penstock ingress gate, 10M-frame 0-alloc corpus | BROKEN-MID-EDIT | 22 errors, 0 ran |
| cgap-p28 | 80 B P28LicenseAttestation, 640-bit sweep | 8/0 (census target readout) | ALIVE |
| cgap-ffi | sovereign C-ABI cdylib (admit/eval/commit/verify) | 56/0 | ALIVE |

### Membranes and aux (never published)

| crate | purpose | tests | standing |
|---|---|---|---|
| cgap-beam | BEAM/AtomVM NIF membrane (cdylib) | real escript run | ALIVE, `publish = false` |
| cgap-audit | CountingAlloc + POWL + OCEL2 slabs | 37/0 | workspace-local |
| cgap-court | whole-repo court_verdict harness (no src/) | 14 courts, 3,313 LOC | workspace-local |
| cgap-branchless | branchless ALU/SWAR/bit-plane mechanics | no census | local, `publish = false` |
| cgap-integration | cross-crate C-ABI roundtrip suites | 20/20 as-of 26.11.1 | ALIVE, no-registry |
| cgap-cli | ggen-generated `cgap` binary (standalone ws) | cli census | workspace-local |
| bench | Divan chain benches, PARETO attribution | benches | workspace-local |

Non-member crates on disk: cgap-verification (7/0 courts,
`crate-reality/verification.md`), wasm-footprint (4/5 host courts;
1 fail = cross-lane blake3 defect, `crate-reality/
newcomers-stardf-wasm.md`), stardf-datalog (REAL engine /
VACUOUS-DEFAULT test gate, same census), cli, csct.

## Quickstart

```sh
cargo add cgap@26.11.2
```

The sovereign loop below is the registry-sourced consumer program of
`waves/f3-14-consumer.md` (`cargo new` + `cargo add` + build/run,
exit 0, 4/4 PASS):

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

Step [2] (arrow build/freeze/readback) ran 4/4 against the 26.10.29
registry surface; the 26.11.x frames/deduction lanes are cited in the
crate map above. The v26.10.29-era README is archived byte-exact at
`docs/jira/v26.11.1/readme-archive/cgap-umbrella-pre.md` (sha256
`fd7564e438b7276eaeb4d951bac7da175524f48c17133c95d5426fb288c5a2b6`).

## Honest failures and caveats

- cgap-gate is BROKEN-MID-EDIT at 26.11.2: 22 lib-test compile
  errors, 0 tests executed (compile-abort is not a failure count) —
  `waves/crate-reality/cgap-gate.md`.
- stardf-datalog's default `cargo test` silently executes 0 tests
  (VACUOUS-DEFAULT gate) — `crate-reality/newcomers-stardf-wasm.md`.
- wasm-footprint: 1 of 5 host courts fails on a cross-lane cgap-blake3
  defect — `crate-reality/newcomers-stardf-wasm.md`.
- cgap-storage and cgap-p28 census rows are per-target readouts; the
  26.11.1 totals (113x3 sweep; 42/0) are not re-totalled here.
- cloud/fibo/integration rows carry 26.11.1 as-of-date counts; no
  26.11.2 re-census exists for them.

## docs.rs build matrix (audit pub-06 §4)

Stable toolchain, edition 2021, no `.cargo/config.toml` rustflags:
docs.rs x86_64-linux default-features `--docs` builds clean with no
target features required across the published set. Rows: cgap
(default = host; core+storage gated behind host), arrow
(cfg(target_arch) fast paths), audit + ffi (cdylib+rlib; docs via
rlib, riscv courts cfg'd out), blake3 (NEON/AVX2 cfg-gated), bn3
(no_std body, std under cfg(test)), crypto (core+alloc body, std
re-links under cfg), gate (penstock courts cfg-gated), git (no_std,
SHA-1/CRC32 in-crate), gpu (Metal cfg-gated to Apple; docs.rs builds
fallback), parquet (aarch64 cfg-gated), p28 (wasm32/bare-metal
split), wasm-footprint (wasm32-wasip1 real target; docs.rs host
fallback).

## Morning DoD five gates (one-liners; dod-five-gates.md is authoritative)

Receipts: `waves/dod-five-gates.md` + the five gate markers.

- G1 workspace purity GREEN: build+test exit 0, census 1025 pass /
  0 fail (26.11.1 marker; see crate-reality census for 26.11.2
  per-crate state — cgap-gate compile-abort disclosed above).
- G2 binary/symbol purity GREEN: court_verdict 13/0, kill_rate 1.0000
  (19/19).
- G3 1M-row ledger ALIVE: 4-of-4 seal+eval court, 14/0 exit 0.
- G4 640-bit spoliation ALIVE: 2560 mutations / 2516 real deaths,
  p28 42/0 (26.11.1 as-of-date).
- G5 bare-metal silicon ALIVE*: ELF 150/65536 B flash, 0/32768 B
  SRAM. (*size law only; XIP rule execution BLOCKED-on-owner,
  recorded not laundered.)

## Chain PARETOs (measured 2026-11-01, bench profile)

- chains-ingress: digest fold (BLAKE3) ~85% of segment / ~91% of
  chain; receipted speedup 10.1x vs P3-16 baseline, fresh
  re-derivation 9.6x (`audit/tps-01-reality-ledger.md:24`).
- chains-sovereign: A02 sealed-block commit 87.6% of chain (N=4096).
- chains-bn3: unify join 81.2% at 65k rows, 99.9% at 1,048,576.
- chains-seal: no 80% stage — metering ~45% / seal+verify ~35% /
  validate ~19%.

Receipts: `waves/bench-chains-*.md`; method in `crates/bench/README.md`.

## Links

- Receipt trail: `docs/jira/v26.11.1/waves/` and
  `docs/jira/v26.11.2/{waves,lanes,audit}/` (lane finish/audit
  receipts, gate markers, publish chain, bench PARETOs,
  `v26111-campaign-ledger.md`).
- Diataxis docs: `docs/diataxis/` (receipts `waves/dia-1`..`dia-6`).
- Readme provenance: `docs/jira/v26.11.1/readme-archive/` (sha256'd,
  `INDEX.md`).
