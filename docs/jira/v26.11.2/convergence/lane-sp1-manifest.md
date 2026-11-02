# Lane SP1 manifest — enclave self-play foundation (v26.11.2 convergence)

Standing: **ALIVE** (host check + riscv check + lib courts, all exit 0, this session).

## Files written (lane ownership, exact)

- `crates/cgap/Cargo.toml` (deps + `host` feature)
- `crates/cgap/src/lib.rs` (facade re-exports, cfg-gated)
- `crates/cgap/src/pipeline.rs` (new)
- `crates/cgap/src/bin/enclave_self_play_entry.rs` (new)
- `docs/jira/v26.11.2/convergence/lane-sp1-manifest.md` (this file)

## Commands + exits (this session)

| command | exit |
|---|---|
| `cargo check -p cgap` (host, dev) | 0 |
| `cargo check -p cgap --no-default-features --target riscv32imc-unknown-none-elf` | 0 |
| `cargo test -p cgap --lib` | 0 (4 passed / 0 failed) |
| `cargo test -p cgap` (workspace full) | 101 — NOT this lane's surface: `tests/self_play_parquet_git_commit.rs` compile-aborts (E0308, 0 tests executed) and `tests/self_play_radix_dred_cascade.rs` fails 2 semantic courts; both files are OTHER SP lanes' in-flight integrations (this lane owns no `tests/` file) |

## Feature decisions

- All 10 deps added `default-features = false`.
- `cgap-git` default-features=false was LOAD-BEARING: the pre-existing
  `cgap-git` dep line enabled `std` by default and failed E0463 on
  riscv32imc-unknown-none-elf; `cargo tree -e features -i cgap-git`
  witnessed the default→std edge from cgap itself.
- `cgap-core` + `cgap-storage` are NOT no_std-clean (95 / 198 errors on
  riscv32imc-unknown-none-elf; no `#![no_std]` declared anywhere, std
  paths implicit) — made optional behind new feature `host` (default-on).
  The none-target image excludes both until those crates gain no_std
  posture (their files are outside this lane's ownership).
- `cgap-parquet` scan is std-gated upstream (`cgap-parquet/src/lib.rs`
  `#[cfg(feature = "std")] pub mod scan`) — record: the Parquet READER is
  host-only everywhere, and is not part of the enclave entry by design.

## Host-only stage list (none-target image)

- `CascadeRetract` (cgap-storage `cascade_retract`) — compiles OUT of the
  none-target image; the entry egresses typed marker bit 0x80000000.
- A04 core-frame constants (`cgap_core::frame`) — none target carries the
  pinned literals (b"CGAP", version 1, 128 B) with an independent-trip
  note; the host build consumes the real `cgap_core::frame` pins.
- Parquet scan (reader) — host-only upstream, not a pipeline stage here.

## no_std-clean subset (passes the none-target check)

gate, p28, blake3, branchless, arrow, bn3, parquet (no-scan), git
(no-std-feature) — 8 of 10; the 2 host-only deps are core and storage.

## Wire-up inventory (pipeline.rs, construction-only)

- gate: `cgap_gate::penstock::ingress` fn-pointer seam + A19 status probe
- p28: `P28LicenseAttestation` + `validate_intrinsic` admission gate
  (typed `PipelineError::AttestationRefused`; mutation court
  `refuses_spoliated_license` names the magic-corruption mutation)
- blake3: `hash` fn seam
- arrow: `PresenceMask<256, 32>::EMPTY` + const surface
- bn3: `RuleCompiler` + `ConsequentProjector` real handles
- parquet: `FooterSealer` PhantomData seam (factory-closed per stage)
- git: `write_pack_v2` fn-pointer seam
- storage/core: host-only, `PhantomData`/cfg-gated seams
- 4 Chicago courts in `pipeline.rs #[cfg(test)]` (real collaborators, no
  mocks; two named anti-vacuity mutations: spoliated P28 magic, corrupted
  frame magic flips ingress status). All 4 pass.

## Bare-metal entry (enclave_self_play_entry.rs)

- FA10 maximal-consumer pattern, cited
  `docs/jira/v26.11.2/adversarial/forensic-lane-10.md:81`.
- `target_os = "none"`-gated body: real P28 root license, pipeline
  construction, penstock ingress probe, BLAKE3 digest, PresenceMask const,
  RuleCompiler address, (host) cascade_retract, fnv1a64, crc32+sha1 —
  results XOR-egress through an `AtomicU32` + volatile read-back.
- Bare-metal bump allocator (static-mut heap, atomic top, no free) — the
  pipeline seams carry alloc-typed signatures (git pack writer).
- Const-assert on section budget: NOT link-time ASSERTed here (a no_main
  check cannot ASSERT sections; budget ASSERTs live in the enclave_128k.ld
  ASSERT family per the cgap-ffi pattern) — recorded divergence vs the
  dispatch sketch ("where link-time allows" — it does not allow in a
  `cargo check`).

## Divergences vs dispatch

1. Dispatch named cgap-storage for cascade_retract — confirmed, plus
   cgap-core is also host-only (same no_std posture gap).
2. `LeaseTracker` (named in the dispatch constructor inventory) does not
   exist by that name in cgap-gate/cgap-core greps; the penstock ingress
   fn-pointer + PresenceMask seams were wired instead. UNVERIFIED handle
   name: no LeaseTracker type found; recorded, not invented.
3. `cargo test -p cgap` full-run exit 101 is OTHER lanes' in-flight
   integration files in crates/cgap/tests/ (one compile-abort, two
   semantic failures) — this lane owns no tests/ file; the lane's own
   surface is `cargo test -p cgap --lib` exit 0.
4. rustfmt applied to pipeline.rs/bin post-write (formatting-only).
