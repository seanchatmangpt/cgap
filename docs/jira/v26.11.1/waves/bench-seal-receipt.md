# bench-seal-receipt

Lane F2-14 BENCH-SEAL, wave F2, repo /Users/sac/cgap (canonical checkout).
Charter: `docs/jira/v26.11.1/bench-campaign-charter.md` ("Receipt = cargo
bench command + exit + full Divan table text, per lane"). Status vocabulary:
ALIVE only where a command+exit ran this session; the direct C-ABI leg is
[SKIP]-pending, loudly.

## Lease

- `crates/bench/benches/seal.rs` (NEW) — the divan bench target `seal`.
- `crates/bench/Cargo.toml` — NOT written by this lane. The `[[bench]]
  name = "seal"` declaration (lines 50-52) and the dev-dep graph
  (cgap-p28/cgap-crypto/cgap-ffi) were landed by the concurrent membrane
  lane before this lane's first build; observed, not edited.
- `docs/jira/v26.11.1/waves/bench-seal-receipt.md` (this file).

## Commands + exits (this session)

| # | Command | Exit |
|---|---|---|
| 1 | `CARGO_TARGET_DIR=/tmp/lane-f2-14-target cargo bench -p bench --bench seal --no-run` | manifest error: `can't find bn3 bench` (sibling F2-10..13 files not yet landed; bounded-poll 120 s until all four files appeared, no stubs created by this lane) |
| 2 | `CARGO_TARGET_DIR=/tmp/lane-f2-14-target cargo bench -p bench --bench seal --no-run` (after files landed) | 3 compile errors in seal.rs (fixed in-lane; see "Fix log") |
| 3 | same `--no-run` | 1 Sync-bound error (fixed: ffi bench state moved inside the closure) |
| 4 | same `--no-run` | 0 (17.51 s, executable emitted) |
| 5 | `CARGO_TARGET_DIR=/tmp/lane-f2-14-target cargo bench -p bench --bench seal` (run 1) | exit NOT captured (pipe trap): crypto table printed, then the ffi corpus panicked ("appended --- guarantees a span"); p28 never ran. Fixed in-lane |
| 6 | run 2 after fix | full green table, all 3 benches x 2 consts, observed live with no pipe (cargo reported success; exact code re-witnessed in run 7) |
| 7 | `CARGO_TARGET_DIR=/tmp/lane-f2-14-target cargo bench -p bench --bench seal > /tmp/lane-f2-14-bench-run.txt 2>&1` (canonical receipt run) | CARGO_BENCH_EXIT=0, zero panics (full output captured) |

Exit-code hygiene note: commands 1-6 piped through `tail`, so the echoed
`EXIT=` there is tail's, not cargo's (the CLAUDE.md pipe trap); the
canonical run (7) captures the real cargo exit with no pipe: 0.

## Full Divan table (canonical run 7, verbatim)

```text
[SKIP] FFI sovereign C-ABI leg PENDING: cgap_admit_frame / cgap_eval_bn3 /
cgap_commit_parquet / cgap_verify_p28 are not exported by cgap-ffi in this
link (F2 items 5/6 unlanded at build time). The A19 leg measures the
Rust-side boundary pair (ffi_rust_boundary_pair_roundtrip) through the
frozen CgapAdmitFrameFn / CgapVerifyP28Fn signatures; the direct-leg bench
lands with a re-run after the exports land.
Timer precision: 41 ns
seal                                 fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ crypto_seal_verify_roundtrip                    │               │               │               │         │
│  ├─ 1024                           854.5 µs      │ 914.2 µs      │ 859.1 µs      │ 861.8 µs      │ 100     │ 100
│  ╰─ 65536                          54.89 ms      │ 56.95 ms      │ 55.15 ms      │ 55.26 ms      │ 100     │ 100
├─ ffi_rust_boundary_pair_roundtrip                │               │               │               │         │
│  ├─ 1024                           140.9 µs      │ 147.8 µs      │ 141.8 µs      │ 142.3 µs      │ 100     │ 100
│  ╰─ 65536                          8.984 ms      │ 9.549 ms      │ 9.035 ms      │ 9.046 ms      │ 100     │ 100
╰─ p28_attestation_verify                          │               │               │               │         │
   ├─ 1024                           470.9 µs      │ 472.7 µs median, 474 µs mean, 100 samples
   ╰─ 65536                          30.09 ms      │ 30.47 ms median, 30.47 ms mean, 100 samples
```

(The last two rows abbreviate; the verbatim columns for them are: 1024 ->
fastest 470.9 µs, slowest 491.6 µs, median 472.7 µs, mean 474 µs; 65536 ->
fastest 30.09 ms, slowest 30.89 ms, median 30.47 ms, mean 30.47 ms.)

## TARGET -> MEASURED conversions

| Bench | Per-op derivation | MEASURED (fastest) | Converts |
|---|---|---|---|
| `p28_attestation_verify` [1024] | 470.9 µs / 1024 | ~460 ns per 80 B record | A17 charter row "attestation verify (640-bit) ns — spoliation-check cost": the full §5 ordered validate (parse + intrinsic + BLAKE3 instrument-span recompute + BLAKE3 parent-wire binding + ceiling) is MEASURED, not a TARGET |
| `p28_attestation_verify` [65536] | 30.09 ms / 65536 | ~459 ns per record | cross-checks the [1024] number (flat per-record cost, 100-sample tables both) |
| `crypto_seal_verify_roundtrip` [1024] | 854.5 µs / 1024 | ~835 ns per receipt | A16 §6 unsealed round trip: seal (keyed BLAKE3 + tick) + pure verify, MEASURED |
| `crypto_seal_verify_roundtrip` [65536] | 54.89 ms / 65536 | ~837 ns per receipt | stable at both consts |
| `ffi_rust_boundary_pair_roundtrip` [1024] | 140.9 µs / 1024 | ~138 ns per admit+verify pair | A19 "sovereign boundary overhead" — Rust-side boundary-pair leg (frozen extern "C" signatures, indirect fn-pointer calls) MEASURED; direct C-ABI leg [SKIP]-pending |
| `ffi_rust_boundary_pair_roundtrip` [65536] | 8.984 ms / 65536 | ~137 ns per pair | stable at both consts |

Stability witness: run 5 (pre-fix) already showed crypto 1024 = 854.7 µs
fastest and 65536 = 54.82 ms fastest (independent processes); run 6 crypto
1024 = 855.8 µs. Min-of-N across the three live runs is within 0.2%.

## FFI leg status (loud, not silent)

Dispatch condition: "if the four sovereign symbols are exported by the time
you build ... else bench the Rust-side boundary function pair and record the
FFI leg [SKIP]-pending with the reason."

- Bounded-poll 1 (300 s deadline, 10 s cadence): bench files
  kernels/ingress/parquet/bn3 landed at t+120 s; sovereign exports in
  `crates/cgap-ffi/src/sovereign.rs` = 1/4 (`cgap_verify_p28`).
- Bounded-poll 2 (180 s deadline, 8 s cadence): still 1/4 at deadline.
- At the canonical build/run: still 1/4 (re-checked immediately before the
  run). The dispatch's else-branch executed: the A19 leg measures the
  Rust-side boundary pair through the frozen `CgapAdmitFrameFn` /
  `CgapVerifyP28Fn` signatures; the direct C-ABI leg is [SKIP]-pending.
- The bench binary re-probes at every start via real
  `dlsym(RTLD_DEFAULT, ...)` over the live image and prints the [SKIP]
  banner above the table — a re-run after F2 items 5/6 land resolves the
  leg without another edit.

## Method (charter compliance)

- Corpus: SplitMix64 canonical `0xBF58_476D_1CE4_E5B9` verbatim as master
  seed; per-record spread via the in-repo penstock form
  `seed ^ i*0x2544_5491_4F6C_DD1D` (cgap-gate/src/penstock.rs:814).
  Instrument corpora carry two deterministic shape bytes (a newline at
  index 87 and the literal `---\n` separator) so A17 §3 span extraction
  always terminates; the span is derived from the same bytes the validator
  re-derives from, so the shaping cannot desync the corpus.
- `black_box` on every consumed checksum; verdict checksums asserted per
  iteration batch (admit/verify counts asserted == N per batch).
- No Process.sleep in bench code; polls were lane-side bounded
  deadline-checked loops on real observables (files on disk, source text,
  build artifacts).
- Host numbers (macOS, arm64, nightly-aarch64 toolchain); Gate 5 silicon
  fit stays an `llvm-size` claim, not a Divan claim (charter caveat).

## Anti-vacuity (named per bench; flips are the court-auditor pass)

Charter: "Court-auditor flips one bench per lane in a /tmp sandbox" — the
mutations are named here and in the code header; the flip execution belongs
to the auditor lane per the charter's dispatch shape.

1. `p28_attestation_verify`: flip one bit of a record's `upstream_receipt`
   -> §5.7 refuses; admit-count assert (== N per batch) fires and the
   per-record cost moves to the early-refusal path.
2. `crypto_seal_verify_roundtrip`: flip one byte of a sealed receipt's
   `blake3_digest` -> `verify_unsealed` = `DigestMismatch`; verified-count
   assert fires.
3. `ffi_rust_boundary_pair_roundtrip`: corrupt one header byte before admit
   -> `CGAP_ERR_INVALID_MAGIC` typed refusal; ok-count assert (== 2N) fires.

## Fix log (all in-lane, all before the canonical run)

1. `copy_from_slice` length bug: instrument separator written into an 8 B
   slice with a 4 B literal (2 sites).
2. p28 corpus collected structs where wire bytes were declared; verdict
   binding was `Result<(), Spoliation>` (field access on `()`).
3. `Mutex<(CgapFrameView, u64)>` not `Sync` (raw pointers in the view);
   ffi-bench state moved inside the bench closure (fresh per batch).
4. Run-5 panic "appended --- guarantees a span": byte 87 of the instrument
   was not forced to a newline, so the `---` line could fail to start;
   deterministic newline pin added (2 sites).

## Divergences (recorded, none silent)

1. FFI direct C-ABI leg: [SKIP]-pending (1/4 sovereign exports at build
   time). Not a silent pass; banner re-probes every run.
2. The first bench executable panicked in the ffi corpus setup (fix-log
   item 4); runs 1 and 5 are discarded, the canonical run is exit 0.
3. The A19 boundary numbers are the RUST-SIDE pair through the frozen
   signatures, NOT the linked-sovereign C-ABI leg; the receipt table says
   so in-place and the binary's banner restates it every run.
4. `crates/bench/Cargo.toml` was not written by this lane (see Lease);
   sibling bench files were awaited, never stubbed.

## Cleanup

Lane build-root lease deleted per the fanout cleanup law, after the
canonical run was captured:

```bash
python3 -c "import shutil; shutil.rmtree('/tmp/lane-f2-14-target')"
```

## Identity (GAP-6 backfill)

Backfill lane GAP-6, 2026-11-01, closing the F2 schema-drift finding
witnessed by F2-15: this receipt originally cited no subject sha256 —
identity rode on lease statements alone (R_missing_identity-lite).
Backfilled identity for the exact on-disk subject:

- **Subject**: `crates/bench/benches/seal.rs`, sha256
  `0bc7e0b738d609c0b18356ed4647d98e83b863d2162f4bf5d056d8d2963ef3f4`,
  19,320 bytes, as on disk 2026-11-01.
- **F2-15 flip verdict (this bench)**: mutation class "flip one byte of a
  sealed receipt's `blake3_digest`" realized as a forced BLAKE3 recompute
  — **+12% with controls <1%, FLIPPED**; sandbox restore sha-exact.
  Executed by the F2-15 court-auditor; GAP-6 transcribes the audit's
  verdict into `f2-15-bench-flips.md`, it does not re-measure.



