# Lane AX-4 Receipt — Alloc-Purity Courts in Container (linux/arm64)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout).
Mode: LOCAL container exec (docker 28.0.4, aarch64). Cluster apply:
DRAFT-UNRUN per AX-0 gate W4 (cluster NO-GO, exit 1 conn-refused).

## W1 Identity

Owned files written this lane:
- `courts/interop-chicago/ffi/linux_cgo.go` (NEW, `//go:build linux`)
- `courts/interop-chicago/ffi/mmap_linux.go` (NEW, `//go:build linux`)
- `courts/interop-chicago/ffi/alloc_linux_test.go` (NEW, `//go:build linux`)
- `courts/ax-testbed/runners/alloc.sh` (NEW)
- `docs/jira/v26.11.2/lanes/lane-ax-04-alloc.md` (this file)

## W2 Commands + Exits (all run this session)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `docker info` | 0 | daemon 28.0.4 aarch64 ALIVE |
| 2 | `docker images` | 0 | **AX-1 image ABSENT** (no cgap-ax image in daemon) |
| 3 | `bash courts/ax-testbed/runners/alloc.sh` (run 1) | 1 inner | cargo: no cwd — fixed (`cd /work`) |
| 4 | alloc.sh run 2 | inner FAIL | courts FAIL 0x43470002 — pre-existing court defect, see W5 |
| 5 | alloc.sh run 3 | inner build FAIL | `sink declared and not used` — concurrent lane edit, unblocked per SLA |
| 6 | alloc.sh run 4 | docker ENOSPC | rustup download failed (os error 28) |
| 7 | `docker builder prune -f` | 0 | reclaimed 8.9 GB build cache |
| 8 | alloc.sh run 5 (final) | **0** | **6/6 courts PASS in linux/arm64 container** |
| 9 | darwin fence `go test` (same courts) | **0** | `ok ... 2.171s`, PASS |
| 10 | `go build ./...` (ffi pkg, darwin) | 0 | compile fence holds |

## W3 Honesty clause (first-class, per dispatch)

- The linux/arm64 cgo transition tax is a NEW measurement, taken 2026-11-02
  inside golang:1.23-bookworm (linux/arm64) against
  libcgap_ffi.so built from THIS tree by rust:1-bookworm (release).
- darwin numbers DO NOT carry over. For reference only:
  lane-ipc-03-latency-court.md W4 cites 182.7-185.6 ns/call (darwin,
  BenchmarkAdmitSingle) — a latency number, not an alloc number, and a
  DIFFERENT substrate (darwin dylib, darwin go).
- linux/arm64 column (measured this session):
  - TestSteadyStateAllocPurity: AllocsPerRun(1_000_000, admit+eval) =
    **0.0 allocs/op** (PASS; court asserted final state, count=2 lanes=[1 5])
  - TestVersionAllocPurity: **0.0 allocs/op** (PASS)
  - Anti-vacuity flip: escaping closure = **1 allocs/op**, no-op = **0**
    (both directions witnessed IN the container)
- ns/op cgo latency on linux/arm64: UNVERIFIED (no bench leg run; the
  IPC-3 bench file was not migrated in this lane).

## W4 Anti-vacuity / flip battery

- The court corpus ran against the REAL linux dylib via real cgo; no
  doubles, no mocks.
- Flip witnessed inside the container BOTH directions (W2 #8):
  escaping closure 1 allocs/op > 0; no-op 0.0. A court that cannot flip
  is FAIL — this one flips.
- Corruption leg also flipped typed: corrupt digest -> 0x43470002
  (OABI_MISMATCH), valid frame -> admit (TestAdmitPositiveControl PASS).

## W5 Defects found (pre-existing, not introduced by AX-4)

1. **wireFrame never copied the payload into the frame buffer**
   (alloc_court_test.go wireFrame, original lines 62-81): `frame :=
   make([]byte, 128+len(payload))` then only the header was written; the
   payload tail stayed zeros, so the embedded digest (over the real
   payload) could never match. Consequence: TestAdmitPositiveControl /
   TestEvalPositiveControl / TestSteadyStateAllocPurity refuse
   0x43470002. Reproduced ON DARWIN this session with the current
   /tmp/ipc2-target dylib before any linux run — this is a
   darwin-visible defect, not a linux delta. Fixed by one line
   (`copy(frame[128:], payload)`), disclosed as an AX-4 unblock.
2. **TestAntiVacuityFlip could not flip on linux go1.23** (0 allocs/op:
   the local `sink` backing array was stack-allocated). I landed a
   linux-tagged replacement witness (alloc_linux_test.go,
   TestAntiVacuityFlipLinux, package-level sink). The file owner then
   rewrote TestAntiVacuityFlip mid-wave (package-level `sink = s`), which
   first broke compile (`declared and not used`), unblocked per SLA
   (moved the decl to package level); the owner's final version flips at
   1 allocs/op on BOTH darwin and linux, superseding my
   TestAntiVacuityFlipLinux (which stays landed as the linux-tagged
   redundancy — both pass in the container).

## W6 SLA unblocks applied to ANOTHER lane's in-flight file (disclose-in-both-directions)

- File: courts/interop-chicago/ffi/alloc_court_test.go (IPC-2 lane's
  court; its receipt docs/jira/v26.11.2/lanes/lane-ipc-02-alloc-purity.md
  is ABSENT on disk — divergence recorded, W8).
- Unblock A: `copy(frame[128:], payload)` in wireFrame — without it the
  admit positive control is vacuous-false on every platform.
- Unblock B: moved `var sink []int` from function-local to package level
  to fix the owner lane's mid-flight compile break
  (`declared and not used`), per compile-freeze SLA.
- The owning lane supersedes both diffs on landing; this receipt cites
  the fix, the owner's landing must cite this file.

## W7 Fences (must-not-regress surfaces)

- darwin court suite (the IPC-2/IPC-3 surface my diff touches): re-run
  this session after both unblocks — exit 0, PASS (W2 #9). The linux
  files are `//go:build linux`-tagged, so darwin compilation is
  byte-identical in surface (go build exit 0, W2 #10).
- Shared rust workspace untouched (container-only builds under
  /tmp lease); root Cargo.toml / lib.rs never edited.
- Leases deleted: /tmp/ax4-shared, /tmp/ax4-probe, /tmp/ax4-probe-target
  (python shutil.rmtree, witnessed absent).

## W8 Divergences + standing

- Dispatch said READ lane-ipc-02-alloc-purity.md, lane-ax-01/03: NONE of
  these receipts exist on disk (lanes/ grep zero). The darwin court
  source (alloc_court_test.go) and lane-ipc-03-latency-court.md were
  used as the migration base instead.
- Dispatch said "podman run": podman ABSENT (AX-0 gate W2 #4); docker
  used per AX-0 gate W4 LOCAL verdict.
- AX-1 image absent: runner falls back to pinned stock images
  (rust:1-bookworm + golang:1.23-bookworm), disclosed in script output
  and here. The AX-1-image arm of alloc.sh is landed but UNVERIFIED.
  Missing prerequisite: the cgap-ax image (AX-1 lane, receipt absent).
- Cluster apply: DRAFT-UNRUN by design (AX-0 W4).
- Standing: container alloc-purity courts linux/arm64 **ALIVE**
  (alloc.sh exit 0, 6/6 PASS, flips witnessed); darwin fence **ALIVE**
  (exit 0); AX-1 image arm **BLOCKED** (image absent); linux ns/op cgo
  latency **UNVERIFIED**; cluster apply **DRAFT-UNRUN**.

## JSON terminus

{"lane":"AX-4","files_written":["courts/interop-chicago/ffi/linux_cgo.go","courts/interop-chicago/ffi/mmap_linux.go","courts/interop-chicago/ffi/alloc_linux_test.go","courts/ax-testbed/runners/alloc.sh","docs/jira/v26.11.2/lanes/lane-ax-04-alloc.md"],"commands":["docker info=0","docker images=0","bash courts/ax-testbed/runners/alloc.sh=0","darwin go test=0","go build=0","docker builder prune=0"],"courts_run":["TestBLAKE3OfficialEmptyVector","TestAdmitPositiveControl","TestEvalPositiveControl","TestSteadyStateAllocPurity","TestVersionAllocPurity","TestAntiVacuityFlip","TestAntiVacuityFlipLinux (linux container only)"],"standing":"ALIVE (container courts); AX-1 image arm BLOCKED; linux ns/op latency UNVERIFIED","divergences":["lane-ipc-02/ax-01/ax-03 receipts absent on disk","podman absent, docker used","AX-1 image absent, pinned fallback disclosed","two SLA unblocks to alloc_court_test.go (W6)"]}
