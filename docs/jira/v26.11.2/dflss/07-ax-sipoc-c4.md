# 07-ax-sipoc-c4

SIPOC + C4 container view for the AX/docker interop-sandbox lane family
(v26.11.2 DMEDI Define stage). Corrected to the ACTUAL verdict per
`lanes/lane-ax-00-gate.md` (W2 commands + exits, 2026-11-02). This file
is NEW by name: the landed `dflss/16-sipoc.md` / `17-c4-architecture.md`
cover the crate substrate and remain canonical for that scope; this file
covers the AX testbed / container-court lane family only. Where the
dispatch draft said "AX Provisioning gVisor Sandbox", reality is:
NO cluster (conn refused, exit 1), NO gVisor witnessed, sandbox class
UNVERIFIED — the landed process is docker build -> container exec on the
LOCAL PATH (gate W4: GO).

## Section 1 - Suppliers

| Supplier | Provides | Court / anchor |
|---|---|---|
| docker daemon 28.0.4 (podman ABSENT, exit 127) | image build + `docker run` exec substrate | `lane-ax-00-gate.md:19,44,74` |
| go 1.23.4 darwin/arm64; cargo 1.101.0-nightly | Go courts + Rust FFI dylib builds | `lane-ax-00-gate.md:23-24` |
| Local corpus generator (`courts/interop-chicago/corpus/gen.go`) | deterministic corpora from frozen seed | `corpus/gen.go` |
| AX Workspace/Task manifests (5 Task yaml + workspace-cgap.yaml) | DRAFT-UNRUN court definitions | `courts/ax-testbed/manifests/`; `lane-ax-02-manifests.md` |
| C-ABI surface, 12/12 symbols as of XC-01 (`libcgap_ffi.dylib`) | real FFI collaborators for the Go courts | `lane-xc01-ffi-surface-truth.md`; `lane-ipc-00-ground-truth.md` W3 |
| ax-testbed Dockerfile (digest-pinned two-stage) | image definition, ax-task-runner @ b82f57ee | `courts/ax-testbed/Dockerfile` |

## Section 2 - Inputs

| Input | Source spec | Anchor |
|---|---|---|
| Task manifests (vendor schema UNVERIFIED — ax CLI absent, exit 127) | ax.io/v1alpha1 vendor material, UNVERIFIED | `lane-ax-00-gate.md:36-38,73-76` |
| Corpus seed `0xA20C_0FFE_E5A7_E000` | frozen corpus seed (never conflate with SplitMix64 multiplier `0xBF58_476D_1CE4_E5B9`) | `cgap-parquet/src/bitslice.rs:687-699` |
| FFI export table | lane-ipc-00 nm -gU table (11 then; 12/12 as of XC-01) | `lane-ipc-00-ground-truth.md:22-36`; see note below |
| ax-task-runner pin b82f57ee | git ls-remote, witnessed lane AX-1 provenance comment | `courts/ax-testbed/Dockerfile:2-5` |

> FFI-census note (2026-11-02): the 11-symbol count was the nm -gU census at
> lane-ipc-00 time. As of `lanes/lane-xc01-ffi-surface-truth.md` the fresh-dylib
> census is 12/12 (cgap_stats_snapshot added by lane RC-05; `nm -gU` exit 0,
> "PASS: 12 exported cgap_* symbols == 12 header prototypes"). "11 symbols"
> elsewhere in this file is historical-as-of-IPC-0, not current truth.

## Section 3 - Process (actual, 3 stages)

| Stage | Name | Mechanism | Court | Gate |
|---|---|---|---|---|
| PX-1 | Image build | `docker build -f courts/ax-testbed/Dockerfile` (digest-pinned, two-stage) | build exit + pinned-digest echo | build must exit 0 or fail-closed |
| PX-2 | Container exec | `docker run` LOCAL exec mode (NOT k8s exec, NOT ax apply) | per-court docker run | exit 0/1/2/3 law |
| PX-3 | Court verdict + delta | Go courts bind the C-ABI surface (12/12 as of XC-01); darwin-vs-linux delta tables | go test arms | delta tables PENDING until AX-4..6 container arms run |

Explicitly NOT the process: "AX Provisioning gVisor Sandbox" — no
cluster endpoint (gate cmd #3 exit 1, conn refused), no gVisor witnessed
anywhere in the tree; sandbox class UNVERIFIED.

## Section 4 - Outputs

| Output | Customer | Class |
|---|---|---|
| Per-court exit codes (0 pass / 1 fail / 2 subject-missing / 3 loud skip) | operator, audit ledger | MEASURED per run |
| darwin-vs-linux delta tables | ship-gate table | PENDING until AX-4..6 container arms run |
| Typed refusals (NO-GO / DRAFT-UNRUN / UNVERIFIED) | audit ledger | ALIGNED (typed, per gate W4) |
| Mutation flips (each court names its anti-vacuity mutation) | court harness | court class |

## Section 5 - Customers

| Customer | Consumes | Anchor |
|---|---|---|
| Operator | per-court exits, typed refusals | gate W2 table |
| Audit ledger (lane receipts) | exits + standing vocabulary | `lanes/` receipts |
| Ship-gate table (`01-release-gates.md`) | delta tables + court verdicts | `01-release-gates.md:28-32` |

## Section 6 - C4 container diagram (ACTUAL landed state, ASCII)

```text
 Level 1 context: darwin host (arm64) runs the LOCAL PATH only.
 Cluster path: NO-GO (conn refused). gVisor: not witnessed.

      darwin host (go 1.23.4 / cargo nightly / docker 28.0.4)
        |                                       |
        | docker build                          | cargo build -p cgap-ffi
        v                                       v
 +-------------------------------+   +--------------------------+
 | ax-testbed image              |   | libcgap_ffi.dylib        |
 | (DRAFT until AX-1 build run;  |   | (REAL, 615,552 B,        |
 |  Dockerfile landed,           |   |  nm -gU 12/12 (XC-01))   |
 |  digest pins witnessed in     |   +-----------+--------------+
 |  Dockerfile comments)         |               |
 +---------------+---------------+               | dlopen / cgo
                 | docker run (LOCAL exec)       |
                 v                               v
 +----------------------------------------------------------+
 | interop-chicago module (LANDED, darwin-exercised)        |
 |  go.mod                                                  |
 |  corpus/gen.go        <- seed 0xA20C_0FFE_E5A7_E000      |
 |  ffi/  (ffi.go, blake3.go, alloc_court_test.go,          |
 |         mutation_court_test.go, linux_cgo.go,            |
 |         mmap_linux.go)                                   |
 |  shm/  (shm.go, unsafe_slice.go, shm_court_test.go,      |
 |         shm_linux.go, shm_linux_test.go)                 |
 |  cmd/seal_receipt (main.go + receipt-template.md)        |
 +----------------------------+-----------------------------+
                              | binds (C-ABI, 12/12 XC-01)
                              v
 +----------------------------------------------------------+
 | cgap-ffi sovereign surface:                              |
 |  cgap_admit_frame / cgap_eval_bn3 / cgap_commit_parquet  |
 |  / cgap_verify_p28 / cgap_i32_batch / cgap_schema_addr   |
 |  / cgap_array_addr / cgap_batch_rows / cgap_version      |
 |  / cgap_digest_of / cgap_p28_invariant_check             |
 +----------------------------------------------------------+

 Known gaps (loud): cgap_ingress_bytes, cgap_radix_join, cgap_shm_*
 ABSENT at C-ABI (IPC-0 suite NO-GOs CTQ-02/CTQ-04/shm).
```

## Section 7 - Honest-notes register

| Note | Standing | Anchor |
|---|---|---|
| AX API (ax.io/v1alpha1 Task/Workspace, apply/describe/ssh) | UNVERIFIED — vendor-README material only; ax CLI absent (exit 127), install unapproved | `lane-ax-00-gate.md:30-38` |
| Sandbox class (gVisor / other) | UNVERIFIED — no gVisor witnessed anywhere in the tree | gate W4 |
| suspend/resume (AX-9) | UNVERIFIED-by-design — NO-GO, no simulated pass permitted | `lane-ax-00-gate.md:47,66` |
| ax-testbed image | DRAFT — Dockerfile + entrypoint + runners landed; container build run NOT witnessed in a landed receipt | `courts/ax-testbed/Dockerfile` |
| Cluster path | NO-GO (ate-system svc conn refused, exit 1) | `lane-ax-00-gate.md:17,45` |
| Manifests | DRAFT-UNRUN (drafted, never applied; apply claims are hearsay) | `lane-ax-02-manifests.md` |
| darwin-vs-linux delta tables | PENDING until AX-4..6 container arms run | gate W4 |
