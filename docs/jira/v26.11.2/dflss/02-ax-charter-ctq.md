# 02 AX-Integration Charter + VOC->CTQ Table (DFLSS DMADV, AX wave)

Corrective charter for the AX (ax.io/v1alpha1 substrate) integration
lane family AX-1..AX-10. This file REWRITES the dispatch draft that
circulated for the AX wave: figures asserted without a this-session
command+exit are demoted to ASPIRATIONAL / PENDING / DESIGN-CLAIM, and
the "deterministic phase transition" objective for suspend/resume is
RECLASSIFIED as ASPIRATIONAL with the AX-0 NO-GO citation
(`../lanes/lane-ax-00-gate.md:47` — AX-9 suspend/resume NO-GO, ax CLI
absent exit 127, cluster unreachable exit 1).

Citation law: every cell cites its lane receipt file:line. A figure with
no landed receipt is PENDING or UNVERIFIED, never rounded up. Register
of truth for the wave verdict: `../lanes/lane-ax-00-gate.md`.

## Register table (per-cell status)

Receipt column uses lane shorthand; full file:line cites live in each
section below.

| CTQ | Objective | Target | Witnessed | Receipt | Status |
|---|---|---|---|---|---|
| 01 | isolation | 0 shared w-paths | claim; real witness /tmp races | ax-00:37 | CLAIM-UNVER |
| 02 | suspend/resume | <=500 ms | unexecuted; cluster NO-GO | ax-00:15,47 | ASPIRATIONAL |
| 03 | membrane allocs | 0 B/op | darwin 0 allocs; cgo unmeasured | ipc-04:48 | PARTIAL |
| 04 | frame latency | <=10 ns/fr | kernel p99 6 ns; memb 85+ ns | b01:19 ipc-03:44 | VERIF-COND |
| 05 | SRAM fit | <=131,072 B | 36,048 B; mutant 136,068 flips | b04:35,40 | VERIFIED |
| 06 | mutation arms | 4/4 flips | darwin 4/4; container PENDING | posh-05 | PART-VERIFIED |

## CTQ-01 cross-lane isolation — DESIGN-CLAIM-UNVERIFIED

- Target: 0 shared writable paths between concurrently executing lanes.
- Design claim: the AX Workspace sandbox (per the vendor README) gives
  each Task an isolated workspace. This claim is VENDOR-CLAIM-UNVERIFIED:
  the ax CLI is absent (`ax version` exit 127,
  `../lanes/lane-ax-00-gate.md:15`) and no operator approval to install
  exists on disk (`../lanes/lane-ax-00-gate.md:30-38`). The AX API
  surface is vendor-README material, UNVERIFIED
  (`../lanes/lane-ax-00-gate.md:76`).
- The session's ACTUAL witness is the opposite class: the darwin fleet
  itself raced on shared writable paths. Named witnesses:
  - G10: the census hook pinned a FIXED shared `WORK=/tmp/rep1-census`
    with fixed filenames; concurrent `cargo test` invocations raced and
    all three arms failed with missing `.o` files
    (`../lanes/lane-g10-flip-tests-reconcile.md:14-20`); a second
    hook-vs-hook target race was found while fixing
    (`:29-39`).
  - DX-6: classified inventory of fixed-shared-write /tmp literals
    across `crates/*/tests/*.rs` (e.g. `/tmp/ts4-census`,
    `/tmp/ts2-census`) plus a process-global ALLOC-counter race between
    sibling tests in one binary
    (`../lanes/lane-dx-06-fixture-races.md:16-28`).
- Honest reading: isolation is NOT witnessed at 0 shared writable paths
  anywhere in this fleet yet; the AX sandbox claim cannot be admitted
  until an executed `ax apply`/describe witness exists (cluster path
  NO-GO, `../lanes/lane-ax-00-gate.md:50-52`).
- Close condition: an executed court that runs >= 2 concurrent lanes
  against an AX Workspace and asserts disjoint write sets. Until then
  this cell stays DESIGN-CLAIM-UNVERIFIED. Mutation for that court:
  co-locate a shared writable path -> the disjoint-write assert must
  FAIL.

## CTQ-02 suspend/resume <= 500 ms — ASPIRATIONAL-UNVERIFIED

- Target: suspend/resume round trip <= 500 ms.
- RECLASSIFICATION (mandatory): the dispatch draft phrased this as a
  "deterministic phase transition" objective. That phrasing is
  DEMOTED. Per the gate receipt, AX-9 is NO-GO -> UNVERIFIED-by-design:
  `ax` absent (exit 127) and no reachable cluster (exit 1, connection
  refused 127.0.0.1:64378); "No simulated pass permitted"
  (`../lanes/lane-ax-00-gate.md:47`).
- No measurement exists. No simulated or modeled number is admitted.
- Close condition: live cluster + approved ax CLI; a real
  suspend->resume witness with a bounded-poll timing method. The peer
  register row agrees: `dflss/07-ax-sipoc-c4.md:113` records
  "suspend/resume (AX-9) UNVERIFIED-by-design — NO-GO, no simulated
  pass permitted".


## CTQ-03 membrane allocs 0 B/op — PARTIAL

- Target: 0 B/op through the membrane (no heap reassembly on the
  admit/eval/verify hot path).
- Measured (darwin/arm64, go1.23.4, this-session per
  `../lanes/lane-ipc-04-shm-transport.md:55-59`): the Go-side zero-copy
  mmap transport read path measured **0 B/op, 0 allocs/op** (3-run
  min range 4.52-4.54 ns/op), proven zero-cgo by the
  `CGO_ENABLED=0` run (exit 0, `:22`).
- NOT measured: the cgo membrane itself (through `cgap_admit_frame`).
  The IPC-2 lane's receipt is ABSENT on disk (`ls lanes/lane-ipc-02*`
  exit 1, witnessed this session); IPC-3's disclosure is the surviving
  record of that lane's in-flight state
  (`../lanes/lane-ipc-03-latency-court.md:56-64`). The dispatch for THIS
  charter named "lane-ipc-02's darwin measurement"; no such receipt
  exists — see Divergences.
- linux/arm64: PENDING — no arm64 receipt landed for any membrane arm.
- Close condition: an AllocsPerRun court over the cgo admit path with
  the dylib loaded, plus an arm64 re-run. Mutation: force one heap
  alloc into the read path -> the 0 B/op assert must FAIL (witnessed
  flip in the transport court, `../lanes/lane-ipc-04-shm-transport.md:49`).

## CTQ-04 frame latency <= 10 ns/frame — VERIFIED-CONDITIONAL (kernel layer only)

- Target: <= 10 ns/frame.
- IN-PROCESS KERNEL layer (the 10 ns target applies HERE and ONLY here):
  C2 p99 = 6 ns/frame vs 10 ns max, release build, exit 0
  (`../lanes/lane-b01-cscb-c1-c2.md:19`).
- Disclosed method (the CONDITION): per-frame Instant timing cannot
  meet any 10 ns SLA on macOS (~40 ns clock-call overhead, witnessed:
  raw per-frame p99 42 ns, dominated by clock calls); the gate
  implements ONE Instant pair amortized over a fixed batch of 64
  frames (~1.2 ns/frame residual clock overhead)
  (`../lanes/lane-b01-cscb-c1-c2.md:53-61`). Anti-vacuity: the
  `CSCB_MUTATE=branchy` mutant lifts p99 6 -> 28 ns, gate FAILs exit 1
  (`../lanes/lane-b01-cscb-c1-c2.md:20-23,34-37`).
- MEMBRANE layer (distinct, do NOT conflate): the cgo membrane floor
  alone is **85 ns/call** (null-refusal short-circuit, 200k calls), and
  a full frame through `cgap_admit_frame` measures **179-187 ns/frame**
  (batch-1365 amortized, 7-run min/median/max 183/185/187)
  (`../lanes/lane-ipc-03-latency-court.md:44-47`). The membrane is off
  the 10 ns target by >18x at the floor alone; a 10 ns claim made
  across the membrane is FALSE and this charter forbids it.
- Host disclosure: figures measured on a NON-ISOLATED shared host under
  concurrent lane builds (`../lanes/lane-ipc-03-latency-court.md:71-73`);
  environment-dependent, not portable targets.
- Close condition for the membrane layer: either a kernel-side figure
  that keeps the full admit path under 10 ns, or a charter re-scope
  that states per-layer budgets explicitly (>= 2 layers, 2 targets).


## CTQ-05 SRAM <= 131,072 B — VERIFIED

- Target: linked riscv32imc ELF SRAM (`.text+.rodata+.data+.bss`)
  <= 131,072 B.
- Measured: **36,048 B** <= 131,072 B, headroom 95,024 B — gate exit 0
  (`../lanes/lane-b04-cscb-c5.md:35`).
- Anti-vacuity: the `--mutate=sram-bloat` mutant measures
  **136,068 B > 131,072 B** — the overflow flips
  (`../lanes/lane-b04-cscb-c5.md:40`). An earlier mutant that failed to
  flip (dead-stripped `[u8;100_000]`, 36,048 B == baseline) was
  correctly refused as VACUOUS GATE
  (`../lanes/lane-b04-cscb-c5.md:31`).
- Re-derived this session (lane DFX-1, 2026-11-02), not receipt-copied:
  `python3 scripts/verify_cscb_sram.py` -> exit 0, "36048 B <= 131072 B";
  `python3 scripts/verify_cscb_sram.py --mutate=sram-bloat` -> exit 0,
  "mutant measured 136068 B > 131072 B — overflow flips".
- Bound provenance: the 131,072 B figure is the enclave physical
  envelope floor (`waves/w6-budget-elf-profile.md:13`).

## CTQ-06 4/4 mutation arms — VERIFIED (darwin) / container re-witness PENDING

- Target: 4/4 mutation arms flip in the refusal-matrix court.
- Darwin witness: `posh_refusal_matrix_court` 4/4 passed, exit 0, run
  this session by POSH-5 against the real mounted cgap-git lib APIs
  (`../lanes/lane-posh-05-matrix.md:28-29,53-56`): arm 1 wire-magic
  flip -> `DemuxError::SubjectMissing`; arm 2 cyclic injection ->
  `GateError::CyclicDerivation`; arm 3 stride overflow ->
  `IngressError::Overflow` (bound DERIVED: 1,365 x 24 = 32,784 >
  32,768, `:47-50`); arm 4 alloc control -> sentry observes b' > b.
- Container re-witness: PENDING AX-4..7. The gate permits the container
  path as GO via local docker exec (`docker run`), daemon ALIVE
  (`../lanes/lane-ax-00-gate.md:44,46`); each container court must
  still name its anti-vacuity mutation. No container re-run has
  executed; no container figure is admitted.
- Close condition: the same 4 arms re-run inside a container
  (`docker run`), 4/4 exit 0 with named flips.

## Divergences from dispatch (doc yields to reality)

1. **lane-ipc-02 receipt absent.** Dispatch directed "cite lane-ipc-02's
   darwin measurement" for CTQ-03. `ls docs/jira/v26.11.2/lanes/
   lane-ipc-02*` exits 1 (witnessed this session, W-R table). The only
   landed darwin 0 B/op measurement is IPC-4's transport court
   (`../lanes/lane-ipc-04-shm-transport.md:48-49`); IPC-3 confirms no
   IPC-2 receipt was on disk at its close
   (`../lanes/lane-ipc-03-latency-court.md:62`). CTQ-03 cites IPC-4 and
   marks the cgo-membrane alloc arm UNMEASURED.
2. **"deterministic phase transition" wording.** The dispatch draft's
   phrase appears in NO landed file (grep over dflss/ + lanes/ zero
   matches this session); it is corrected here to ASPIRATIONAL with the
   AX-0 NO-GO citation as directed.


## W-R commands + exits (this session, lane DFX-1)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `python3 scripts/verify_cscb_sram.py` | 0 | 36,048 B in bound |
| 2 | `... --mutate=sram-bloat` | 0 | mutant 136,068 B flips |
| 3 | `ls lanes/lane-ipc-02*` (v26.11.2/) | 1 | no ipc-02 receipt on disk (divergence 1) |
| 4 | `grep "deterministic phase" dflss/ lanes/` | 1 | zero matches (divergence 2) |
| 5 | Read-tool re-reads of all cited receipts | 0 | every figure re-read from disk this session |

## Standing (W7)

- CTQ-05: ALIVE (commands 1-2, exit 0 both, this session).
- CTQ-04 kernel layer: ALIVE as of B01's receipt
  (`lane-b01-cscb-c1-c2.md:19`); figures receipt-cited, re-read at use
  time.
- CTQ-04 membrane layer, CTQ-03 cgo arm, CTQ-01, CTQ-02, CTQ-06
  container arm: UNVERIFIED / PENDING per cells above — never rounded
  up.
- Shared tree untouched except the two owned files; no git commands.

## See Also

`../lanes/lane-ax-00-gate.md` (register of truth) ·
`01-project-charter-interop.md` · `02-voc-ctq-matrix.md` ·
`07-ax-sipoc-c4.md:113`
