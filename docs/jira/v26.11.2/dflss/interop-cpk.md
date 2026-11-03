# Interop membrane Cpk study (IPC-7, DOD-14)

Layer law (AX-GATE-03, `dflss/10-ax-control-plan.md:27`): the admission
MEMBRANE (go cgo -> cgap-ffi AdmitFrame) measures ~179-192 ns/frame
amortized — NOT 10 ns. The 10 ns USL is kernel-layer-only. This study
gates on the DECIDED membrane budget 250 ns/frame (MV-5/R1 pattern:
measured-band + headroom, provenance below) and REPORTS the aspirational
10 ns Cpu (negative — printed, never gated).

## Method

- Runner: `courts/ax-testbed/runners/cpk.sh` (N=30 sequential trials,
  bounded, zero sleeps; `go test -bench BenchmarkAdmitBatch1365
  -benchtime 200x -count 30`, darwin membrane, fixture
  `/tmp/ipc3-fixtures/batch_1365.bin`, dylib
  `/tmp/ipc3-target/release/libcgap_ffi.dylib`).
- Per-trial sample = ns/op / 1365 = amortized ns/frame.
- Cpu = (USL - mean) / (3 sigma), sigma = sample stdev (n-1).
- Exit contract (mirrors `crates/bench/src/bin/cpk_study.rs`):
  decided Cpu < 1.33 -> exit 1 (honest FAIL, G09 law);
  jitter flip failing to drop Cpu below 1.33 -> exit 2 (VACUOUS);
  dylib/fixture/go missing -> exit 3 (loud skip naming the rebuild).
- Anti-vacuity flip: additive scheduler-jitter injection
  (+U(0,60) ns every sample, +U(0,120) ns every 3rd, seed
  0xA20C0FFEE5A7E000) MUST drive Cpu below 1.33.

## Decided budget provenance (MV-5/R1 pattern)

- Witnessed darwin membrane band: 179-192 ns/frame (IPC-3 W4; AX-5 W4
  darwin column; re-witnessed this session: 181.7-187.7 over N=30).
- DECIDED USL = 250 ns/frame: measured band top + engineering headroom.
  NOT the 10 ns kernel USL — that stays kernel-layer-only per the
  layer law; gating the membrane on it would be tuning-to-chase, a
  REFUSED control action (AX-GATE-03).
- Aspirational 10 ns Cpu is reported every run and is negative; the
  gate NEVER reads it.

## Baseline result (2026-11-02, this session)

> **SCOPE — ambient/quiet-host caveat (lane A-05, 2026-11-02):** ambient-load
> state was not recorded at measurement time; the n=30 membrane ns/frame
> figures carry no CONTENTION-SUSPECT guard witness. Consume under the
> control-plan quiet-host consumption rule
> (`dflss/06-control-plan.md:151-164`). Membrane-layer contention
> discussion register pointer: row 31 D1 (in-process vs cgo).

- n=30, mean 184.294 ns/frame, sigma 1.270, min 181.71, max 187.67.
- Cpu(decided 250 ns) = **17.252** -> PASS (>= 1.33 floor).
- Cpu(aspirational 10 ns) = **-45.762** — printed loudly, not gated.
- Flip witnessed: Cpu 17.252 -> **0.157** (< 1.33). Runner exit 0.
- Thermals: `pmset -g therm` clean before and after (no warnings).
- Linux container column: skipped loud this run — Docker Desktop VM
  disk quota exhausted during the rust image build (`os error 28`);
  context-gap, never a gate. Cited context for that column remains
  AX-5 W4 (239-448 ns/frame, high variance).

## Honest layer statement

Any claim derived from this study must read: "membrane amortized
~184 ns/frame (M3 Max darwin, containerized column unmeasured this
session); kernel-layer 10 ns USL NOT claimed at the membrane layer."
