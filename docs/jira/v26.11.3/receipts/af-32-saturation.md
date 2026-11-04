# AF-32 saturation soak receipt

Lane: AF-32 (Benchmark 02, zero-snaps saturation soak), v26.11.3.
Subject: `crates/cgap-aoq/tests/macro_saturation_soak.rs` (new file, this
wave). Receipt file: this one.

## What was built

One soak test file, two modes:

- `af32_saturation_soak_scaled` — default scaled mode, 10,000,000 frames
  (dossier 1B scaled to CI-adjacent minutes; `CGAP_AF32_FRAMES` env
  override). Ran green this session.
- `af32_saturation_soak_full_1b` — the dossier's 1-billion-transaction
  target as a `#[ignore]` full-scale mode:
  `cargo test -p cgap-aoq --test macro_saturation_soak -- --ignored`.
  Listed on disk (2 tests, 0 benchmarks); NOT executed this session — the
  1B run itself is UNVERIFIED. Documented runtime estimate below.

Composition (landed surfaces, no re-implementation): AB-8 writer/verify
pipeline (`AoqWriter::frame_segments` scatter writev -> real socketpair
UDS -> `verify_window` + `decrypt_in_place`, hot loop is ab8_soak's
direct descendant); AB-10 partial-accept storm (1 byte per accept) plus
Ok(0) would-block injection; AB-11 `FrameTooLarge` over a real wire-shm
ring segment; AB-17-pattern churn (per-lane worker cycles killed
mid-stream at deterministic points and respawned; respawn resumes from
durable state: persistent writer cursor + chain carry checkpoint).

## Gate runs (exit records)

Exit-record law: process exit or cargo's "N passed; 0 failed" line.

| Run | Mode | Profile | Exit record | Throughput (measured) |
|---|---|---|---|---|
| R1 | scaled 10M | debug | `test result: ok. 1 passed; 0 failed` (101.38s) | 98,815 frames/s |
| R2 | scaled 10M | debug | `test result: ok. 1 passed; 0 failed` (82.78s) | 121,044 frames/s |
| R3 | scaled 10M | RELEASE | `test result: ok. 1 passed; 0 failed` (36.04s) | 278,630 frames/s |
| S0 | 200k smoke | debug | `test result: ok. 1 passed; 0 failed` (3.56s) | 62,574 frames/s |

## Zero-snaps table (measured columns only)

From R1/R3 (10M frames each). R2's counters were identical except
throughput and RSS baseline; both runs assert the same identities.

| Column | Measured (R1 debug / R3 release) |
|---|---|
| frames requested | 10,000,000 |
| frames valid (landed in chain, exactly once) | 9,990,243 |
| corrupt refusals (typed) | 9,757 (= 10,000,000 - 9,990,243, exact identity) |
| truncation refusals (typed) | 1,260 (210 legs x 6 probes) |
| oversize refusals (typed `FrameTooLarge`) | 2,560 (160 designated cycles x 16 attempts) |
| storm write calls | 41,216,208 |
| Ok(0) retries | 19,515 (exact arithmetic identity) |
| churn killed / respawned | 690 / 690 (balanced) |
| unhandled worker exceptions | 0 |
| chain segments verified/opened | 780 / 780 (every segment verify_all green) |
| strict alloc court | 0 allocs / 0 bytes (strict window); positive control moved the counter |
| fd delta (lsof) | 9 -> 9 (delta 0) |
| RSS | debug 64,112 -> 64,192 KiB plateau; release 24,304 -> 24,400 KiB |
| chain integrity post-soak | every segment `verify_all` green; carry records link segments across kills |

Drop law: valid frames dropped = 0 (the test asserts
`VALID == total - corrupt_count` exactly; the corrupt frames are the
refusal census, not drops).

## Full-scale 1B mode

- Command: `cargo test -p cgap-aoq --test macro_saturation_soak -- --ignored`
- Listed: `cargo test -p cgap-aoq --test macro_saturation_soak -- --list`
  -> `af32_saturation_soak_full_1b`, `af32_saturation_soak_scaled`
  (2 tests, 0 benchmarks), run this session.
- Executed: NO (this session). Standing: UNVERIFIED for the 1B run.
- Runtime estimate: 1,000,000,000 / 278,630 fps (R3, RELEASE) =
  ~3,589 s = ~60 minutes. ARITHMETIC EXTRAPOLATION from the scaled rate,
  labeled as such; not a measured 1B runtime. Debug profile would be
  ~2.3-2.8 h at the two debug rates.

## Disclosures (honest boundaries)

1. Churn is thread-level kills at frame boundaries (the
   macro_defense_mesh pattern): the worker checks a deterministic kill
   point each frame and returns mid-batch; the respawn re-opens a chain
   segment with the killed segment's checkpoint as its carry record.
   Cross-repo AB-17 real-process churn court is cited via cbmqs G6
   (/Users/sac/lsp-max/docs/jira/v26.11.3/receipts/ab-17-churn-stress.md);
   it was not re-run here.
2. Chains are windowed segments (slab reused per cycle, cap 65,536
   records): whole-run slab for 1B would be ~120 GB; the windowed law
   keeps memory bounded. Cross-segment linkage is via carry records
   (each segment opens with a link whose payload is the previous
   segment's checkpoint; GENESIS for the first). Post-soak integrity =
   every segment verify_all green at close + segment census identity.
3. Strict alloc court ran inside lane 0, cycle 0 (clean warmup traffic,
   seq < 8,192, storm/corrupt suppressed there). The counting allocator
   is thread-local (cbmqs G1 method), exact under parallelism.
4. Kill points are deterministic per (lane, cycle); Ok(0) is a simulated
   would-block (adapter, not a kernel backpressure event).
5. RSS/fd sampling via external `ps`/`lsof` between wave barriers; the
   hot loop itself is allocation-free (strict court) and does not spawn.
6. Release-vs-debug: R3 is the extrapolation basis (RELEASE profile,
   optimized). R1/R2 are the CI-adjacent debug exit records.
7. Neighboring-lane state at census time (pre-existing, not session
   introduced, not AF-32 subject): `macro_p99_torture.rs` compile-aborts
   (unclosed delimiter; file actively being written by another lane
   during the census); `purity_gate` 1 fail (unregistered
   `profile_guard.rs` under src/ — another lane's in-flight source);
   `rfc0089_conformance` 2 fails (conformance/tamper-typing). All other
   cgap-aoq test targets green at census time (exit records re-run this
   session, table in the transcript; ab8_soak re-verified green).

## See Also

- `crates/cgap-aoq/tests/ab8_soak.rs` (pipeline ancestor)
- `crates/cgap-aoq/tests/cbmqs_gates.rs` (G5 storm/oversize, G6 churn
  citation)
- `crates/cgap-aoq/tests/macro_defense_mesh.rs` (churn pattern)
