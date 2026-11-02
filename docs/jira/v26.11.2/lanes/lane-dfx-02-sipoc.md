# Lane DFX-2 — SIPOC + C4 (AX sandbox family, corrected to actual verdict)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout).

## W1 Identity

- Lane: DFX-2. Owned files: `docs/jira/v26.11.2/dflss/07-ax-sipoc-c4.md`
  (NEW by name — verified absent before write) and this receipt.
- No git commands run. No shared lib.rs/Cargo.toml touched.

## W2 Commands + Exits

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `ls docs/jira/v26.11.2/dflss/` | 0 | 16-sipoc.md + 17-c4-architecture.md landed; 07-ax-sipoc-c4.md ABSENT -> NEW |
| 2 | `ls courts/interop-chicago courts/ax-testbed` | 0 | go.mod, corpus/gen.go, ffi/ (6 files), shm/ (5 files), cmd/seal_receipt; Dockerfile, entrypoint.sh, manifests/ (5 yaml + README), runners/ (3 sh) |
| 3 | `sed -n 15,70p lanes/lane-ipc-00-ground-truth.md` | 0 | 11-symbol nm -gU table; 3 suite NO-GOs (CTQ-02/CTQ-04/shm) |
| 4 | `grep AX-1 build evidence in lanes/lane-ax-*` | 0 | no AX-1 receipt landed; only gate/02/06 receipts exist -> image stays DRAFT |
| 5 | `grep -m3 DRAFT/UNRUN lanes/lane-ax-02-manifests.md` | 0 | DRAFT-UNRUN law confirmed |
| 6 | Read `lanes/lane-ax-00-gate.md` per row | 0 | podman 127, docker 28.0.4 exit 0, cluster exit 1, ax 127 |

## W3 Diff summary

`07-ax-sipoc-c4.md` (117 lines): SIPOC corrected to the ACTUAL verdict —
suppliers = docker 28.0.4 (podman ABSENT), local corpus generator, AX
manifests DRAFT-UNRUN, 11-symbol FFI table; inputs = Task manifests
(vendor schema UNVERIFIED), frozen seed 0xA20C_0FFE_E5A7_E000, 11-symbol
C-ABI; process = docker build -> docker run LOCAL exec (NOT "AX
Provisioning gVisor Sandbox": no cluster exit 1, no gVisor witnessed,
sandbox class UNVERIFIED); outputs = per-court exits, darwin-vs-linux
delta tables PENDING until AX-4..6 container arms, typed refusals;
customers = operator / audit ledger / ship-gate table. C4 ASCII of the
actual landed state + honest-notes register (7 rows).

## W4 Courts run

None in-lane — docs-only lane; the referenced gates (AX-0 W2 probes)
were run in their owning lanes this session and are cited, not re-run.
Falsifier for this artifact: any downstream lane citing a cluster apply,
a gVisor witness, or an applied manifest contradicts the cited exits
(127/1) and is hearsay.

## W5 Verification

Files re-read from disk post-write (line counts 117 + receipt); token
sweep clean (no mojibake, table pipes intact). 07 file is NEW by name;
16/17 SIPOC/C4 remain canonical for the crate-substrate scope — no
supersession.

## W6 Fences

Docs-only diff. No gates, courts, or shared compile surfaces touched;
compile-freeze SLA not engaged. Cross-referenced fences: none of the
cited receipts edited.

## W7 Standing

ALIVE for this lane's artifact (written + verified on disk this
session); all substrate claims inherit the cited receipts' standing
(AX-0 gate ALIVE/NO-GO rows, IPC-0 ALIVE with 3 suite NO-GOs).
