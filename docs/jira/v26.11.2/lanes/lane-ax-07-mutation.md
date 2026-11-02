# Lane AX-7 — in-container mutation court port (4 arms)

Standing: ALIVE (host legs; container legs LOUD-BLOCKED, witnessed exit 3).

## W1 Commands + exits

| cmd | exit |
|---|---|
| `./courts/ax-testbed/runners/mutation.sh --host` | 0 (arms 1/2/4 PASS, arm 3 SKIP loud) |
| `go test ./ffi/ -run TestArm -count=3` (interop-chicago) | 0 (stable, 3 repeats) |
| `./courts/ax-testbed/runners/mutation.sh` (container mode) | 3 — [LOUD-BLOCKED], image `cgap-ax-testbed` not built (lane AX-1 Dockerfile landed but unbuilt); no podman on PATH, docker present without the image |
| `CARGO_TARGET_DIR=/tmp/ipc2-target cargo build --release -p cgap-ffi` | 0 (exact-subject dylib, 615,552 B) |
| `cargo test -p cgap-ffi --release` | 1 — 2 pre-existing/in-flight failures in `fa9_commit_gating.rs` (another lane's in-flight file, disclosed below; NOT AX-7's diff) |

## W2 Source disclosure (dispatch law)

IPC-5 did NOT land: no `lane-ipc-05*` receipt on disk at dispatch
(witnessed: lanes/ listing) and no `mutation_court_test.go` existed
anywhere under `courts/`. Ported from **POSH-5's landed refusal matrix**
(`crates/cgap-git/tests/posh_refusal_matrix_court.rs`), adapted from the
cgap-git Rust surface (Demux/RefTable/IngressRing — not exported over
C-ABI) to the C-ABI surface that actually crosses the membrane:
`cgap_admit_frame` + `cgap_eval_bn3` (IPC-0 W3 11-symbol table).

## W3 The 4x2 matrix (BEFORE/AFTER per arm, host-witnessed)

| arm | BEFORE | AFTER | verdict |
|---|---|---|---|
| 1. magic flip CGAP->XGAP | clean frame admits, status 0x00000000 | typed refusal **0x43470001 INVALID_MAGIC** | PASS (flip witnessed) |
| 2a. duplicate coordinate (1,2,100) x2 | clean store evals lanes **[1 5]** | dup store evals lanes **[1 1]** | PASS (both witnessed, differ) |
| 2b. out-of-order rows 0/1 swap | clean lanes **[1 5]** | ooo store lanes **[5 1]** | PASS (pairwise distinct asserted) |
| 3. overflow past derived bound (floor(32768/24)=1365 facts; fact 1366 = 32,784 > 32,768) | — | — | **[SKIP-MISSING-EXPORT] `cgap_ingress_bytes`** — no ingress symbol at C-ABI (IPC-0 W5: "batch ingress entry point MISSING"); loud `t.Skipf`, never a fake pass |
| 4. alloc positive control | counter baseline | alloc signal **8000 mallocs > no-op floor 0** over 8x1000-op paired rounds | PASS (escape-typed alloc, `retained[0]=999` final state) |

Container arms: LOUD-BLOCKED (exit 3); the darwin-side equivalents above
stand as the witnessed baseline, per dispatch.

## W4 Files written (owned surface only)

- `courts/interop-chicago/ffi/mutation_court_test.go` — the 4-arm port
  (package `ffi`, stdlib-only, self-contained frame builder).
- `courts/ax-testbed/runners/mutation.sh` — runner; `--host` (witnessed
  baseline) and container mode with runtime+image detection, LOUD-BLOCKED
  exit 3 paths, gate exit codes 0/1/2/3 per CLAUDE.md law.

## W5 CROSS-LANE FINDING (disclose-in-both, NOT an SLA edit — no other lane's file touched)

The landed IPC-2 court helper `wireFrame`
(`courts/interop-chicago/ffi/alloc_court_test.go:62-81`) NEVER copies
`payload` into `frame[128:]` — it builds only the 128-byte header, so
the BLAKE3 digest (computed over the real payload) mismatches the
all-zero payload bytes and `cgap_admit_frame` refuses every
wireFrame-built frame with **0x43470002 OABI_MISMATCH**. Witnessed this
session, stage-by-stage on the dumped Go frame bytes
(sha256 b0f4634a...): penstock::ingress ADMITs (0x00000000);
`CgapFrameHeader::validate` Ok; `verify_payload` -> `DigestMismatch`.
Consequence: IPC-2's own landed courts `TestAdmitPositiveControl` and
`TestEvalPositiveControl` FAIL against a fresh exact-subject dylib
(rerun this session, exit 1). AX-7's court is self-contained
(`ax7WireFrame` copies the payload — the one-line fix) and does not
call the defective helper. IPC-2's receipt should carry an ERRATA; the
one-line fix belongs to the owning lane. No mutation of another lane's
in-flight file was applied.

## W6 Fences + pre-existing failures

Must-not-regress fences for this diff: the interop-chicago Go module
(`go build ./... && go vet ./...`) and the landed IPC-2/IPC-3 Go
courts. Run this session:

- `go build ./...` -> 0; `go vet ./...` -> 0 (module clean).
- `go test ./ffi/` -> 1: the ONLY failures are IPC-2's
  `TestAdmitPositiveControl`/`TestEvalPositiveControl`, failing from
  the W5 wireFrame defect (pre-existing to AX-7, witnessed against a
  from-scratch dylib rebuild; NOT introduced by this diff — AX-7's file
  does not call wireFrame). IPC-3's latency courts: untouched.
- `cargo test -p cgap-ffi --release` -> 1 with 2 failures in
  `fa9_commit_gating.rs` (lane FA9's in-flight file, shared tree);
  the 50 sovereign unit tests + 3 other suites pass (incl. the
  admit/status-parity courts AX-7 depends on). Not AX-7's diff.

## W7 Standing

- Court file + runner: **ALIVE** (commands + exits above, this session).
- Container arms: **BLOCKED** (loud, exit 3) — image not built; the
  court file is Linux-portable (stdlib-only, darwin/linux-neutral Go)
  and mounts are pre-wired in mutation.sh; unblocks the moment
  `docker build -f courts/ax-testbed/Dockerfile -t cgap-ax-testbed`
  plus a linux `libcgap_ffi.so` at the pinned path exist.
- Arm 3: **REFUSED-by-surface** ([SKIP-MISSING-EXPORT]
  `cgap_ingress_bytes`) — named, cited, loud.
- Perf claims: none made (no perf numbers in this lane).
- Build-root leases: `/tmp/ax7-target` + probe scratch deleted at lane
  end. `/tmp/ipc2-target` is IPC-2's pinned binding path (referenced by
  ffi.go LDFLAGS), left in place deliberately.

## Divergences (dispatch vs landed)

1. Source court: IPC-5 absent -> POSH-5 refusal matrix (disclosed, W2).
2. Arm 2 semantics: POSH-5's cyclic-ontology injection has no C-ABI
   analog; adapted to duplicate/out-of-order coordinate mutation
   through the REAL eval path, per the dispatch's own arm-2 wording.
3. Arm 4 instrument: CountingAlloc (Rust global allocator) has no Go
   equivalent; the analog is runtime.MemStats.Mallocs with a
   loop-amplified ratio fence (no absolute bounds) after a witnessed
   single-pair noise flake (7 mallocs) — disclosed in-file.
4. Court lives in `courts/interop-chicago/ffi/` (package `ffi`, reuses
   the membrane bindings + BLAKE3), not a new package.

## ERRATA (appended; W5 not rewritten)

W5's wireFrame defect was FIXED mid-lane by lane AX-4 under the
compile-freeze SLA: `alloc_court_test.go:72` now carries
`copy(frame[128:], payload)` marked "AX-4 unblock" (file mtime 15:05,
2026-11-02; witnessed). Cross-cited in BOTH receipts per
disclose-in-both law: AX-4's receipt names the unblock; this receipt
supersedes its W5 consequence statement — the full `go test ./ffi/`
suite now exits 0 (witnessed this session, build 0 / vet 0 / test 0),
so the W6 fence table's "only failures are IPC-2's..." line is
SUPERSEDED by this errata. AX-7's `ax7WireFrame` remains
self-contained by design; W5's diagnosis stands as the root-cause
evidence the unblock corroborates.
