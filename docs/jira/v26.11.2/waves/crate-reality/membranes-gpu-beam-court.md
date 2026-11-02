# REAL Receipt: membranes — cgap-gpu / cgap-beam / cgap-court

Milestone: v26.11.2 starDF reality wave. Lane: REAL-membranes. Date: 2026-11-01.
Canonical checkout /Users/sac/cgap; no git writes; no /Users/sac/stardf access;
no pack edits (ggen-marketplace read-only).

## Verdicts

| crate | class test | verdict |
|---|---|---|
| cgap-gpu | REAL-MEMBRANE | real Metal command surface, exercised this session |
| cgap-beam | REAL-MEMBRANE | real BEAM NIF dylib, real escript collaborator |
| cgap-court | REAL-MEMBRANE | real [[test]]-only harness, 3,313 LOC / 14 courts |

## cgap-gpu (105 LOC src/lib.rs + 33 LOC metal_shim.m + build.rs)

Not a stub. The Rust side is a thin FFI wrapper over a real Objective-C shim
(`crates/cgap-gpu/metal_shim.m`) compiled by build.rs with `-fobjc-arc`,
linking the Metal and Foundation frameworks. The shim calls
`MTLCreateSystemDefaultDevice()` and
`newBufferWithBytesNoCopy:length:options:storageModeShared` — the actual
zero-copy Unified Memory wrap. `MetalSharedBuffer::from_buf` over a 16 KiB
`cgap_arrow::Buf` ran green this session, which means the default Metal device
was really created and a shared-mode MTLBuffer really wrapped caller-owned
bytes (exit 0). Non-vacuity mutation candidate: neutralize the shim's
`newBufferWithBytesNoCopy` to return null — the unit test asserts
`metal_buf.is_some()` and would flip to fail.

## cgap-beam (3,617 LOC src, publish=false)

Audit's "3,512-LOC membrane, 2/2 tests" is stale-but-kind: actual src is 3,617
LOC across 10 files (shm 1,166; lib 661; ops_scan 475; resources 376;
ops_persist 220; resbin 224; spsc_ring 167; pointer 107; alloc_spy 76;
dtor_stats 40). Tests: 11 passing, not 2/2 — unit 9 (incl. shm
create/dup/roundtrip, cross-thread throughput/latency pingpong), plus
`bridge_contract` 1 and `kill_stress` 1. `bridge_contract` is a real BEAM
collaborator: it loads the cdylib via `erlang:load_nif/2` in a real escript
and drives load/ramp round-trip/zero-copy projection/envelope lifecycle/
badarg-vs-error split/owner-kill safety; `kill_stress` runs 100 kill
iterations over real processes. LOC delta from audit (3,512 → 3,617) is recorded, not investigated.

## cgap-court (0 src LOC, publish=false)

`crates/cgap-court/` has NO src/ and NO tests/ dir — this is by design, not
hollowness. The Cargo.toml declares `[[test]] court_verdict path =
../../tests/court_verdict.rs` (3,313 lines at repo root tests/), with 11
dev-deps on the sibling kernels (audit, storage, core, blake3, bn3, gate,
parquet, p28, crypto, ffi) so the courts consume real public APIs. 14 tests
passed in 26.62s this session (exit 0): a04_identity_court, a05_interleave_engine,
a12_unifier_join_court, a14_dam_ingress_court, a19_seal 1m ledger + eval,
alloc_admission_budget, alloc_verify_zero, corpus_determinism_receipt, and
peers. Verdict: the court harness is a REAL surface — a crate whose entire
body is an integration-test binary over real collaborators, exactly its
declared class. Not a shell.

## rdflib p:CrateManifest drift-check (all three)

Graph: /Users/sac/ggen-marketplace/packs/cgap-pack/ontology.ttl §8 (read
only). Query: `?m a p:CrateManifest ; p:crateName; p:crateDescription;
p:cratePublish; p:crateVersion` filtered to the three names, compared field-
by-field against each crates/<name>/Cargo.toml `[package]` (version inherited
from workspace.package = 26.11.2). Result — DRIFT: NONE on all nine fields
(3 crates x name/description/publish/version):

- cgap-gpu: desc "GPU execution membrane ... zero-copy buffer bridging.",
  publish true, ver 26.11.2 — byte-equal both sides.
- cgap-beam: desc "BEAM/AtomVM NIF membrane for cgap (off-heap resource
  handles).", publish false, ver 26.11.2 — byte-equal.
- cgap-court: desc "", publish false, ver 26.11.2 — byte-equal (manifest has
  no description key; graph faithfully mirrors the empty string).

## Proof chain (commands + exits, this session)

| command | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/real-membranes-target cargo test -p cgap-gpu` | 0 (1 passed) |
| `CARGO_TARGET_DIR=/tmp/real-membranes-target cargo build -p cgap-beam` | 0 (cdylib 1.1 MB) |
| `CARGO_TARGET_DIR=/tmp/real-membranes-target cargo test -p cgap-beam` | 0 (9 unit + 1 bridge + 1 kill_stress) |
| `CARGO_TARGET_DIR=/tmp/real-membranes-target cargo test -p cgap-court` | 0 (14 courts, 26.62s) |
| rdflib p:CrateManifest query + tomllib manifest compare | 0 |
| build root lease deleted | done |

## Standing

ALIVE for all three crates — every verdict carries a command + exit run this
session against the exact subject.

## Divergences

1. cgap-beam bridge_contract test: when `CARGO_TARGET_DIR` is set but the
   dylib is absent, the test spawns `cargo build` with the env var REMOVED
   (builds into repo-root target/) but asserts the dylib at the env-set path
   — guaranteed failure under a lane build-root lease. Workaround (no source
   edit): pre-build with the same CARGO_TARGET_DIR so the pre-check loop
   finds it. Test-side path bug, disclosed here, not fixed (file not mine).
2. Audit figure "3,512 LOC / 2/2 tests" for cgap-beam is stale: actual
   3,617 LOC / 11 tests passing.

```json
{"lane":"REAL-membranes","files_written":["docs/jira/v26.11.2/waves/crate-reality/membranes-gpu-beam-court.md"],
"commands":[["cargo test -p cgap-gpu",0],["cargo build -p cgap-beam",0],["cargo test -p cgap-beam",0],["cargo test -p cgap-court",0],["rdflib p:CrateManifest drift-check",0]],
"courts_run":["cgap-gpu metal zero-copy unit test","cgap-beam 9 unit + bridge_contract (real escript load_nif) + kill_stress 100-iter","cgap-court court_verdict 14 courts"],
"standing":"ALIVE","divergences":["bridge_contract CARGO_TARGET_DIR path bug (test spawns build without env but asserts env path; worked around by pre-building dylib, no source edit)","audit stale 3,512-LOC/2-2-tests figure for cgap-beam (actual 3,617 LOC / 11 tests)"]}
```
