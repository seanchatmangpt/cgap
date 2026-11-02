# 01 Project Charter — AX Testbed (DFLSS DMADV, v26.11.2)

DFLSS Define-phase charter for the AX testbed lane family (AX-0..AX-10)
of the v26.11.2 starDF campaign. Extends the campaign charter
`01-project-charter.md` (commercial lanes) and the campaign rollup
`00-dmadv-rollup.md`; per-lane truth is the receipt set
`../lanes/lane-ax-*.md`. Companion registers in this directory:
`02-ax-charter-ctq.md`, `08-ax-pfmea.md`, `09-ax-cpk-msa.md`,
`10-ax-control-plan.md`.

## Sponsor and executor

Sean Chatman (portfolio control plane). Executor: cgap lane fleet, one
canonical checkout at `/Users/sac/cgap`, branch main. Lane leases and
git transitions belong to the coordinator. This charter was landed by
lane AX-10 (receipt `../lanes/lane-ax-10-index.md`), which owns only
this file and that receipt.

## Gate verdict (AX-0, binding on the whole family)

`../lanes/lane-ax-00-gate.md` (W3-W4) is the authority: CLUSTER PATH
NO-GO this session (`kubectl get svc api -n ate-system` exit 1,
connection refused 127.0.0.1:64378); `ax` CLI absent (`ax version`
exit 127); `ax install` NOT APPROVED (no operator statement on disk,
docs grep zero hits). LOCAL PATH GO: docker 28.0.4 daemon ALIVE after
a bounded poll (exit 0), podman absent (exit 127). No lane may claim
an `ax apply` / describe / ssh / suspend / resume witness this
session.

## Vendor-claim discipline (mandatory)

The AX README claims — millions of concurrent tasks, sub-second task
activation, gVisor isolation — are VENDOR-CLAIM-UNVERIFIED. None was
witnessed this session. Per-claim witness status:

| vendor claim | witness this session | standing |
|---|---|---|
| millions of concurrent tasks | none — no cluster, no ax CLI | VENDOR-CLAIM-UNVERIFIED |
| sub-second task activation | none — no cluster, no ax CLI | VENDOR-CLAIM-UNVERIFIED |
| gVisor isolation | none — witnessed runtime is runc under
  Docker Desktop linux/arm64 VM (AX-5 W1) | VENDOR-CLAIM-UNVERIFIED |
| suspend/resume lossless | refusal witness only (AX-9 exit 3,
  prereq-absent) | VENDOR-CLAIM-UNVERIFIED |

The claims in this table are never adopted as fact anywhere in the
campaign docs. The only claims admitted this wave carry receipt cites:
docker-local image build (AX-1), container court migrations
(AX-4/5/6/7 host legs), darwin-vs-linux delta tables (AX-5/AX-6).

## Scope

IN scope (witnessed or drafted this wave):
- courts/ax-testbed/ image scaffold (Dockerfile, entrypoint.sh,
  .dockerignore) and the built image `cgap-ax-testbed:ax1` (AX-1).
- AX Workspace/Task manifests as DRAFT-UNRUN (AX-2) — drafted, YAML
  well-formedness checked (python yaml.safe_load exit 0), never
  applied.
- Container court migrations via `docker run` LOCAL exec:
  alloc-purity (AX-4, 6/6 PASS linux/arm64), latency re-measure
  (AX-5), SHM transport (AX-6, 6/6 PASS incl. the /dev/shm tmpfs
  arm), mutation-matrix port (AX-7, host legs exit 0; container legs
  LOUD-BLOCKED exit 3 at its run time).
- Suspend/resume probe script landed, refusal witnessed (AX-9
  exit 3).

OUT of scope this session (typed, not silent):
- cluster apply, k8s exec, ax describe/ssh/suspend/resume (AX-0 W4
  NO-GO rows; AX-2 W3; AX-9 W2 row 3).
- gVisor runtime measurements (never executed; runc-class witnessed).
- perf generalization beyond the two measured hosts (AX-5/AX-6 state
  their method and refuse to generalize).

## Claim ceiling (verbatim, binding on all AX prose)

"AX scaffold + image + container court migrations landed; cluster-apply
path NO-GO this session (ax install NOT APPROVED, Substrate
unreachable); darwin-host courts remain the measured perf baseline
(container 1.7-1.8x worse per AX-5's delta table); linux/arm64
alloc/shm numbers landed."

Any AX claim stronger than this ceiling is hearsay and must cite a
receipt that does not yet exist.

## Measured facts admitted this wave (receipt-cited)

- Image `cgap-ax-testbed:ax1` built 2026-11-02, two-stage
  digest-pinned, ax-task-runner at pinned commit b82f57ee,
  in-container go1.23.4 / rustc 1.101.0-nightly / riscv32imc target
  witnesses (AX-1 W2-W3, exit 0 rows; W6 standing ALIVE).
- linux/arm64 container alloc purity: 0.0 allocs/op (steady-state and
  version courts), 6/6 courts PASS; anti-vacuity flip witnessed both
  directions in-container (AX-4 W3).
- Container isolation cost, linux/arm64 vs darwin/arm64 bare
  (AX-5 W4 delta table, identical method both columns):
  - cgo call floor: 73-85 ns/call (darwin) vs 154 ns/call
    (container), ~1.8x
  - single admit: 182.7-185.6 ns/op (IPC-3 cited) vs 306.9-313.1
    ns/op, ~1.7x
  - batch median: ~178-185 ns/frame vs ~192 ns/frame (+~7 ns);
    tail max 182-187 vs 305 ns/frame (1.6x)
  - batch-1365 amortized throughput: ~5.4M frames/s darwin vs ~3.4M
    frames/s containerized execution (AX-5 W5.1)
  - timer-resolution witness: 41 ns on both (AX-5 W5.3) — kernel
    arithmetic portable, delta is execution-environment cost
- SHM transport (AX-6 W4, min-of-3): mmap zero-copy read 4.52-4.54
  ns/op darwin vs 4.574-4.656 ns/op container (+1-3%); pread
  fallback 1377-1405 vs 1185-1203 ns/op (-15..-16%); allocs 0 both
  arms; mmap:pread ratio ~1:260-1:310.
- darwin-host courts REMAIN the measured perf baseline; container
  numbers are the isolated-environment column, honestly worse
  (1.7-1.8x on the latency surfaces per AX-5).

## Falsifier (verbatim, binding)

"the AX testbed Verify passes iff every migrated court re-witnesses
its flip inside the container AND the darwin-vs-linux delta table is
published AND suspend/resume is either witnessed or loudly
UNVERIFIED."

Current standing against the three conjuncts:
1. flip-in-container: witnessed for alloc purity (AX-4 W4, both
   directions), latency mutation gate (AX-5 W6, in-container phase
   D), SHM magic corruption (AX-6 W3). AX-7's container legs were
   LOUD-BLOCKED (exit 3, image-absent at its run time) with host
   legs witnessed — conjunct 1 is PARTIAL until AX-7's container
   mode re-runs against the built image.
2. delta table published: YES (AX-5 W4, AX-6 W4; restated above).
3. suspend/resume: loudly UNVERIFIED-by-design (AX-9 W2 exit 3
   refusal witness; AX-0 W4 NO-GO row).

## Tollgates (AX family)

| Tollgate | Exit criterion | Standing |
|---|---|---|
| T1 Define | charter + CTQ/PFMEA/MSA/control registers landed | LANDED (this file) |
| T2 Measure | darwin-vs-linux delta tables published | DONE (AX-5/AX-6 W4) |
| T3 Analyze | container cost attributed (runc/VM crossing + libc/toolchain,
  not kernel drift) | DONE (AX-5 W5) |
| T4 Design | cluster path GO: ax install approved + reachable
  ate-system cluster | OPEN (AX-0 W4 NO-GO; install NOT APPROVED) |
| T5 Verify | the verbatim falsifier above, all three conjuncts |
  PARTIAL (2 done; 1 partial; 3 loudly UNVERIFIED) |

## Loud-skip ledger (typed, never silent)

| lane | skip | typed evidence |
|---|---|---|
| AX-2 | apply DRAFT-UNRUN | `ax apply --dry-run` exit 127 this session (AX-2 W2 #3) |
| AX-5 | phase B first run | exit 3 typed skip naming the rebuild command (AX-5 W1 run 1) |
| AX-7 | container legs | exit 3 LOUD-BLOCKED, image absent at run
  time (AX-7 W1); arm 3 loud `t.Skipf` —
  `cgap_ingress_bytes` unexported (AX-7 W3) |
| AX-9 | suspend/resume | exit 3 `REFUSED[PREREQ-ABSENT]` (AX-9 W2 #3) |

## Charter corrections against the AX README (dispatch mandate)

Where the vendor README and landed reality disagree, reality wins:

- "millions of tasks": UNWITNESSED. Nothing this session exercised
  task scheduling at any scale. Stays VENDOR-CLAIM-UNVERIFIED.
- "sub-second activation": UNWITNESSED. No task was ever created —
  `ax version` exit 127, zero applies. Stays
  VENDOR-CLAIM-UNVERIFIED.
- "gVisor isolation": UNWITNESSED and CONTRADICTED-in-part: the
  witnessed runtime class is runc under Docker Desktop's linux/arm64
  VM (AX-5 W1 honesty clause). Stays VENDOR-CLAIM-UNVERIFIED; any
  isolation claim must say "containerized execution (runc-class)".
- suspend/resume checkpoint story: UNWITNESSED; the probe script
  (`courts/ax-testbed/runners/suspend_probe.sh`) is the executable
  spec and its refusal witness (exit 3) is the only run.

## Fences this charter could break

Documentation-only diff: no source, gate, or manifest touched.
Fences: (a) the AX receipt set it cites — every cite above was
re-read from disk this session (W2/W3 of the AX-10 receipt);
(b) the campaign claim ceiling — any AX prose exceeding it must cite
a new receipt; (c) the vendor-claim table — a future cluster-GO wave
may CLOSE rows only with fresh witnesses, never by editing this
table's standing retroactively.

## Index

The per-lane stitch index (AX-0..AX-9: standing, load-bearing fact,
PENDING peers) lives in the landing receipt
`../lanes/lane-ax-10-index.md`, the machine-checkable companion to
this charter.
