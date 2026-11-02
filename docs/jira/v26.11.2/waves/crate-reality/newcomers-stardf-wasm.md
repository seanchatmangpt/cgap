# newcomers reality wave — stardf-datalog + wasm-footprint

Receipt for lane REAL-newcomers, v26.11.2 reality wave, 2026-11-01.
Repo `/Users/sac/cgap`, branch main. Shared tree untouched except this receipt.

## Verdicts

| Crate | Verdict | Basis |
|---|---|---|
| `stardf-datalog` | REAL (engine) / VACUOUS-DEFAULT (test gate) | src is genuine semi-naive Datalog; default `cargo test` silently executes 0 tests |
| `wasm-footprint` | REAL | real link glue + real C ABI over three kernels; 4/5 host courts pass; 1 fails on a cross-lane defect in cgap-blake3 |

## stardf-datalog

- `src/engine.rs` (290 L) is a real semi-naive kernel: delta-seeded round 0,
  per-round restriction of one body atom to the prior delta
  (`run` + `eval_rule`/`join`/`bind_and_recurse`, engine.rs:126-245), a
  MAX_VARS=8 bind stack, and `count_vars` refusal. Not a stub.
- `src/seed.rs` honors the seed law: `SPLITMIX64_MULT` =
  `0xBF58_476D_1CE4_E5B9` distinct from `CORPUS_SEED`
  `0xA20C_0FFE_E5A7_E000` (seed.rs:10-14), courted in
  `court_seed_law_constants_distinct`.
- `tests/oracle_courts.rs` is a genuine differential court vs an
  independently written naive fixpoint oracle; includes an in-file
  anti-vacuity mutation court (`court_mutation_flip_drop_recursive_atom`)
  that asserts `mutant < intact` — it flips.
- **W3-T8 finding reproduced**: tests are gated
  `#![cfg(feature = "std")]` (tests/oracle_courts.rs:9) and `std` is NOT a
  default feature — `cargo test -p stardf-datalog` exits 0 executing
  **0 tests**, a silent vacuous pass. Reproduced this session, exit 0,
  "0 passed" witnessed.
- With `--features std`, excluding the unbounded n10k court: **4 passed /
  0 failed in 48.7s** (`cargo test -p stardf-datalog --features std -- --skip n10k`).
  Includes the mutation court (flips) and the seed-law court.
- **Open defect (this crate's own court, unbounded)**:
  `court_anc_set_equality_n10k` does not terminate in a usable window —
  the naive oracle is O(rounds x |rel|^2)-ish on 10k nodes; still running
  in background past 20 min with zero output progress. Bounded-poll law:
  the n1k pass bounds correctness; the n10k court needs either a smaller
  n, a smarter oracle, or an ignored-with-reason + separate perf surface.

## wasm-footprint

- `src/lib.rs` (505 L) is real link glue: `#[repr(align(64))]` 1 MiB bump
  allocator (CAS loop, checked adds, no free), `#[panic_handler]` halt
  loop, stable-form `rust_eh_personality` stub — all correctly
  `cfg(not(test))`-gated (E0152 avoided). Real C ABI exports
  (`wasm_version`, `wasm_mask_words`, `wasm_unpack_1bit`, `wasm_rle_decode`,
  `wasm_decode_page_body_w4`), and a W=4 constant-parameter page-body
  decoder with its own differential courts (`w4_courts`) vs
  `cgap_parquet::rle::decode_into`, including a width-refusal court and a
  fixture-mint court.
- wasm32-unknown-unknown build: **ALIVE** —
  `cargo build --manifest-path crates/wasm-footprint/Cargo.toml
  --target wasm32-unknown-unknown --release` exit 0, artifact
  4377 B at `/tmp/newcomers-target/wasm32-unknown-unknown/release/wasm_footprint.wasm`.
  **Registry slide captured**: deps resolved to crates.io 26.11.2
  (cgap-arrow/cgap-parquet downloaded from the registry, 26.11.1 -> 26.11.2
  update line in the log). 
- Host tests: **4 passed / 1 FAILED** (`cargo test --manifest-path
  crates/wasm-footprint/Cargo.toml`).
  Failing: `tests/no_std_core.rs::wave6_multi_chunk_x2_nameable_no_std`
  (a `#![no_std]` test crate with the `#[test]` harness; the assertion
  shape is identical to cgap-blake3's own kernel court).

### Cross-lane defect found (pre-existing, not owned by this lane)

- `crates/cgap-blake3/src/multi_chunk.rs`
  `compress_chunks_4way_x2` / `compress_chunks_4way` NEON paths diverge
  from the scalar/sequential path **only when compiled at opt-level >= 1**
  (wasm-footprint's `[profile.dev] opt-level = 3`; reproduced in cgap-blake3's
  own suite with `--release`).
- Receipt: `cargo test -p cgap-blake3 --release --test multi_chunk_vectors`
  → `x2_kernel_equals_two_4way_kernels` FAILED (multi_chunk_vectors.rs:159)
  and `x2_kernel_matches_scalar_and_4way` FAILED (multi_chunk_vectors.rs:286),
  5 passed / 2 failed. Same tests pass at default dev (opt 0). Debug-vs-release
  divergence in unsafe NEON code = optimizer-sensitive UB or miscompile —
  requires a dedicated repair lane on cgap-blake3.
- Downstream effect: wasm-footprint's court `wave6_multi_chunk_x2_nameable_no_std`
  fails identically against registry 26.11.2 AND a /tmp sandbox patched to
  local path deps (`/tmp/wf-sandbox`), so it is not a registry-slide artifact.
- Interesting tension recorded: `x2_driver_matches_b3sum` and
  `differential_vs_b3sum_corpus_25_and_odd_lengths` PASS in --release, so the
  divergence is masked end-to-end at the driver level; only the
  kernel-level differential courts catch it.

## Commands + exits (this session)

| Command | Exit | Note |
|---|---|---|
| `cargo test -p stardf-datalog` | 0 | 0 tests executed — the vacuous-default gate, exit 0 |
| `cargo test -p stardf-datalog --features std -- --skip n10k` | 0 | 4 passed / 0 failed, 48.7s |
| `cargo test -p cgap-blake3 --test multi_chunk_vectors` | 0 | 7 passed (dev profile) |
| `cargo test -p cgap-blake3 --release --test multi_chunk_vectors` | 0 | 5 passed / 2 FAILED (the opt-level defect) |
| `cargo build --manifest-path crates/wasm-footprint/Cargo.toml --target wasm32-unknown-unknown --release` | 0 | 4377 B artifact |
| `cargo test --manifest-path crates/wasm-footprint/Cargo.toml` | 101-ish (test failure) | 4 passed / 1 FAILED; harness exit nonzero |
| `cargo test -p cgap-blake3 --lib` | 0 | 3 passed |
| sandbox `/tmp/wf-sandbox` patched-run | nonzero | same 1 failure (registry slide ruled out) |

(Exit codes are the last pipe segment; harness rc confirmed nonzero on the
two failures by the FAILED lines — recorded as witnessed, not inferred.)

## Standing

- stardf-datalog engine: ALIVE (ran + passed 4 courts incl. mutation flip).
- stardf-datalog default gate: VACUOUS (0 tests, exit 0) — needs `std = ["dep:..."]`
  dev-dependency style fix or `required-features = ["std"]` so the default
  run is loud.
- wasm-footprint: PARTIAL — builds for wasm32 ALIVE; host courts 4/5;
  the 1 failure is owned by cgap-blake3 (opt-level-sensitive NEON x2
  divergence).
