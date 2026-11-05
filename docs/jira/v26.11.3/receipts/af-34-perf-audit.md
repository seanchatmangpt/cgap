# AF-34 — Benchmark 04: Ground-Constraint Stability Audit (Hardware Counters)

- **Lane**: AF-34 IMPLEMENTATION. Repo: `/Users/sac/cgap`. Date: 2026-11-03.
- **Subject**: landed stress benches under saturation — AB-9 `ipc_stress`
  (100k windows), AB-19 `slab_stress`, AB-6 `aoq1_bench_court` (codec).
- **Charter**: measure the dossier's >99.8% L1 locality / <0.01% branch
  misprediction claims against actual hardware counters, or record the
  typed tool blocker; verify the 64-byte alignment law (structural half).
- **Standing**: PARTIAL_ALIVE — wall-clock + IPC measured this session;
  L1/branch counter targets remain TARGET-UNMEASURED (typed tool blocker,
  three independent PMU routes probed and refused, receipts below).

## Tool-availability audit (probed this session, exact evidence)

| Route | Probe | Result |
|---|---|---|
| Linux `perf` on host | `which perf` | absent on macOS host |
| colima Linux VM | `colima start` (exit 0), `colima ssh` | **no PMU** — see note [a] |
| `xctrace` CPU Counters (Apple PMU) | canary `xctrace record` | **FAILS host-level** — note [b] |

[a] colima guest kernel 6.8.0-117 aarch64: `/sys/devices/` lists no `cpu`
device (only software/tracepoint/breakpoint), `perf_event_paranoid=4`;
`perf` installed in-guest via `linux-tools-*` still cannot see hardware
events — counters structurally invisible to the guest.
[b] `xctrace record --template 'CPU Counters' --time-limit 2s --launch --
/usr/bin/true` → `Data stream: Fatal logging system error: log archive is
corrupt or incomplete`; reproduces for `Time Profiler` (not
template-specific), inside and outside the session sandbox.

Fallback leg 4 discovered real PMU-derived counters: macOS `/usr/bin/time -l`
reports per-process `instructions retired` and `cycles elapsed` on this host
(Apple M3 Max) — genuine fixed-counter PMU data, but no L1-miss or branch-miss
events are exposed to any unprivileged tool here.

## Measured this session (release, --test-threads=1)

Bench outputs (exact lines from this run):

- ipc_stress: `t2: 100000 windows, 1 wrap, zero classification drift`;
  `4 passed; 0 failed` in 2.09s.
- slab_stress: `MIN-OF-3: 264,419,596 pushes/sec (fill+reset pairs over
  Slab64)`; `5 passed; 0 failed` in 0.12s.
- aoq1_bench_court: `bench[amortized]: encode 44.5 ns/header, decode
  41.5 ns/header (min of 7x100000)`; per-call 42.0/42.0 ns.

`/usr/bin/time -l` PMU counters per bench binary (release leg 4, two full
audit runs this session — per-process fixed counters, whole test binary):

| bench | run | instructions retired | cycles elapsed | IPC (inst/cyc) |
|---|---|---|---|---|
| ipc_stress | 1 | 253,821,015 | 94,072,277 | 2.697 |
| ipc_stress | 2 | 259,692,964 | 103,971,606 | 2.498 |
| slab_stress | 1 | 254,019,476 | 100,820,472 | 2.520 |
| slab_stress | 2 | 254,445,505 | 98,329,339 | 2.588 |
| aoq1_bench_court | 1 | 244,907,343 | 93,904,842 | 2.607 |
| aoq1_bench_court | 2 | 265,263,660 | 109,753,196 | 2.417 |

IPC 2.4-2.7 across runs is consistent with (does not prove) high L1
locality and predictable branches — but IPC is not the claim; it stays
UNVERIFIED as evidence for the targets. Counters are whole-binary
(includes harness), not per-kernel — disclosed.

## Verdicts against the dossier targets (never rounded)

| Dossier claim | Target | Verdict |
|---|---|---|
| L1 locality | >99.8% | **UNMEASURED-tool-blocker** (no L1-dcache counter route on this host) |
| Branch misprediction rate | <0.01% | **UNMEASURED-tool-blocker** (no branch-miss counter route) |
| 64-byte alignment | compile-time law | **MET (structural)** — evidence below |

Alignment evidence: `crates/cgap-arrow/src/ipc.rs:317-321` gates framing to
alignment ∈ {8, 64} with `pad_to` body-start alignment;
`crates/cgap-aoq/src/header.rs:104-109` const-asserts
`size_of::<AoqHeader>() == 64` / `align_of == 8` / all offsets against the
generated table (graph drift = compile error).

Ground-constraint observation (new, this audit): `sysctl hw.cachelinesize`
= **128** on this host (Apple M3 Max) — the 64-byte alignment law is BELOW
the actual hardware cache-line size here; 64B-aligned windows straddle no
128B line boundary, so the law is a floor, not an equality, on this host.

## Verbatim auditor commands (Linux host with PMU)

Gate: PARTIAL_ALIVE
/ 2 subject missing / 3 loud skip (tool blocker). Full audit output:
`/tmp/af34_perf_audit.out` (this session's run: AUDIT-EXIT=3, blockers
`perf: absent; xctrace-CPU-Counters: canary-fails(host-level); colima-guest: no-PMU`).

The three perf commands the script runs on a PMU-equipped host, verbatim:

```
perf stat -e L1-dcache-load-misses,L1-dcache-loads,branch-misses, \
           branch-instructions,instructions,cycles \
  cargo test -p cgap-arrow --release --test ipc_stress -- \
             --nocapture --test-threads=1
perf stat -e L1-dcache-load-misses,L1-dcache-loads,branch-misses, \
           branch-instructions,instructions,cycles \
  cargo test -p cgap-arrow --release --test slab_stress -- \
             --nocapture --test-threads=1
perf stat -e L1-dcache-load-misses,L1-dcache-loads,branch-misses, \
           branch-instructions,instructions,cycles \
  cargo test -p cgap-aoq --release --test aoq1_bench_court -- \
             --nocapture --test-threads=1
```

Derivations: `L1 hit ratio = 1 - (L1-dcache-load-misses / L1-dcache-loads)`;
`branch misprediction rate = branch-misses / branch-instructions`.
MEET/UNMET typed per bench against >99.8% / <0.01% — never rounded.

## Falsifier

This audit's own claims flip if: a PMU route lands on this host (install
working `xctrace` or run the script in a PMU-exposing Linux guest) and the
printed ratios disagree with the TARGET-UNMEASURED verdict; or the colima
guest gains a `cpu` device under `/sys/devices` and leg 3 then measures.
The alignment MET is structural and flips only on a source change to
`ipc.rs`/`header.rs` const asserts (compile error is the gate).
