# A-4 assessment receipt: cgap-beam

Standing: **ALIVE**. Purpose: BEAM/AtomVM NIF membrane — off-heap resource
handles bridging Erlang/Elixir to the cgap kernels (arrow, parquet, git,
blake3) via a cdylib (`crate-type = ["cdylib"]`, 3,512 LOC across 10 src
files; largest: shm.rs 1,166, lib.rs 661, ops_scan.rs 475). Dependency
surface: cgap-arrow, cgap-parquet, cgap-git, cgap-blake3 (path, 26.11.1).
Publish state: guarded `publish = false` (Cargo.toml line 6) — P3-9
(p3-pub-beam-gpu.md) honored the recon BLOCKED-for-publish verdict
(structural policy field, not compile); no p3-pub-aux-final.md landed as of
this receipt, guard flip remains in-flight coordinator work.

## Commands (this session)

- `cargo build -p cgap-beam` -> exit 0
- `cargo test -p cgap-beam` -> exit 0; 2/2 tests pass:
  tests/bridge_contract.rs `beam_bridge_contract_end_to_end` ok (1.03s);
  tests/kill_stress.rs `beam_kill_stress_100_iterations` ok (1.62s).
  Test binaries build via build.rs-generated harness under
  target/debug/build/cgap-beam/.../out/.
