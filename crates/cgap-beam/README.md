# cgap-beam

BEAM/AtomVM NIF membrane for cgap: off-heap resource handles bridging
Erlang/Elixir hosts to the cgap kernels (arrow, parquet, git, blake3)
through a cdylib.

## What ships

- `src/` — 10 files, 3,512 LOC; largest: `shm.rs` (1,166),
  `lib.rs` (661), `ops_scan.rs` (475).
- `crate-type = ["cdylib"]`; build.rs generates the test harness under
  `target/debug/build/cgap-beam/.../out/`.
- Dependency surface: cgap-arrow, cgap-parquet, cgap-git, cgap-blake3
  (path, 26.11.1).
- `tests/bridge_contract.rs` (`beam_bridge_contract_end_to_end`) and
  `tests/kill_stress.rs` (`beam_kill_stress_100_iterations`) — 2/2 pass,
  witnessed this session: `cargo test -p cgap-beam` -> exit 0
  (1.03s + 1.62s), per `waves/assess-beam.md`.

## Status

- Publish: guarded `publish = false` (Cargo.toml line 6) — P3-9
  (`waves/p3-pub-beam-gpu.md`) honored the recon BLOCKED-for-publish
  verdict (structural policy field, not compile); guard flip is
  in-flight coordinator work. Live check this session:
  `curl -A cgap-publish-receipt https://crates.io/api/v1/crates/cgap-beam`
  -> `crate cgap-beam does not exist` (404).
- Standing: ALIVE (`cargo build -p cgap-beam` exit 0 and 2/2 tests, per
  `waves/assess-beam.md`, this session).

## See Also

- Assessment: `docs/jira/v26.11.1/waves/assess-beam.md`
- Publish guard: `docs/jira/v26.11.1/waves/p3-pub-beam-gpu.md`
- Sibling membrane: `crates/cgap-gpu`
