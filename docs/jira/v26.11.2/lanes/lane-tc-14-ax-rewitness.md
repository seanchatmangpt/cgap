# Lane TC-14 Receipt — Composed cgap-ax-testbed image + in-image court re-witness (DFX-6 clause 1)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout).
No git commands. Consumes AX-0 (docker LOCAL GO), AX-1 (Dockerfile),
AX-3 (stage fragment), AX-4/AX-7 (runners + courts), DFX-6 (falsifier).

## W1 Files written

- `courts/ax-testbed/build_image.sh` (NEW, 109 lines, gate exit codes
  0/1/2/3; no stock-image fallback — a build failure exits 2 LOUD)
- this receipt (`docs/jira/v26.11.2/lanes/lane-tc-14-ax-rewitness.md`)
  — the ONLY receipt path; no other file written.

## W2 Commands + exits (all run this session, 2026-11-02)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `bash courts/ax-testbed/build_image.sh` (attempt 1) | 0 | first composition built; in-image cgo died later (row 2) |
| 1b | `AX_IMAGE=cgap-ax-testbed:latest bash courts/ax-testbed/runners/alloc.sh` | 101 | in-image arm defect: cargo runs with no cwd — "could not find `Cargo.toml` in `/`" (runner not my file; divergence W6-1) |
| 2 | alloc arm payload run 1 (in-image) | 1 | `gcc: error: unrecognized command-line option '-m64'` — AX-1's hardcoded `go1.23.4.linux-amd64.tar.gz` over arm64 bases = mixed-arch image |
| 3 | rebuild after arm64 go substitution in the SPLICED Dockerfile only | 0 | `cgap-ax-testbed:latest`, manifest list sha256 6dcdb1cf20f6d5... |
| 4 | alloc arm payload (in-image, `cd /work` added) | **0** | **6/6 PASS**: BLAKE3 empty vector; admit + eval positive controls (corrupt digest 0x43470002 typed); steady-state 0.0 allocs/op; version 0.0; **TestAntiVacuityFlip 1 vs 0 allocs/op** |
| 5 | host seeding: `docker run cgap-ax-testbed:latest cat /opt/cgap/lib/libcgap_ffi.so > /tmp/ipc2-target/release/libcgap_ffi.so` (arm64 ELF, 669,720 B) | 0 | satisfies mutation.sh pinned-path precheck |
| 6 | `bash courts/ax-testbed/runners/mutation.sh` (container mode) | **0** | arms 1/2/4 **PASS IN-IMAGE** (magic flip 0x43470001; dup/ooo lanes [1 1]/[5 1] vs clean [1 5]; alloc signal 8000 > 0); arm 3 `[SKIP-MISSING-EXPORT] cgap_ingress_bytes` (loud) |
| 7 | darwin fence `go build ./... && go test ./ffi/` (interop-chicago) | **0** | `ok ... 2.490s` — no-regression law holds |

## W3 Composition method

- `build_image.sh` assembles programmatically: AX-1's
  `courts/ax-testbed/Dockerfile` is the source of truth for stages 1-2
  (AX-1's file NEVER edited); the AX-3 fragment's builder stage is
  spliced before the final FROM; artifacts COPYed into the final stage
  (`/opt/cgap/lib/libcgap_ffi.so` + `/opt/cgap/include/cgap.h`).
- Build context staged at /tmp/tc14-ctx (repo tree untouched; lease
  deleted at script exit; image persists by design).
- TC-14 enabling additions (disclosed): `ENV CARGO_TARGET_DIR=/tmp/target`
  and the seeded `/tmp/ipc2-target/release` dylib copy — the alloc arm
  cargo-builds into a writable target dir and the cgo link pin is
  `-L/tmp/ipc2-target/release` (ffi.go:18).
- Anti-vacuity: composition artifact courts (script step 4: non-empty
  ffi.so + header + seeded path + ax-task-runner binary + pinned-commit
  file) PLUS the in-image flips themselves: alloc flip 1 vs 0
  allocs/op (W2 row 4) and mutation arms 1/2/4 BEFORE/AFTER (W2 row 6).
  A court that cannot flip is FAIL; these flip.

## W4 DFX-6 clause 1 closure statement

DFX-6 Verify was NOT PASSED on clause 1 ("flips witnessed in stock
images, not the cgap-ax-testbed image",
`docs/jira/v26.11.2/lanes/lane-dfx-06-rollup.md` W2). CLOSED this
session: TestAntiVacuityFlip flips 1 vs 0 allocs/op INSIDE
`cgap-ax-testbed:latest` (W2 row 4) and mutation arms 1/2/4 run INSIDE
the composed image (W2 row 6), arm 3 loud SKIP. Clause 1's falsifier
("not the real image") no longer holds. Clause 1 only: DFX-6's other
open items (suspend/resume UNVERIFIED, zero cluster-apply) stay OPEN.

## W5 Standing

- Composed image build: **ALIVE** (exit 0 this session, tag
  cgap-ax-testbed:latest, manifest list sha256 6dcdb1cf...).
- In-image alloc courts: **ALIVE** (6/6 PASS, flip witnessed, exit 0).
- In-image mutation arms 1/2/4: **ALIVE** (exit 0); arm 3
  REFUSED-by-surface (loud SKIP, unchanged from AX-7's host witness).
- darwin fence: **ALIVE** (exit 0, W2 row 7).
- alloc.sh in-image arm as-written: **BLOCKED** (runner defect, W6-1);
  courts ran via its exact payload plus `cd /work`.
- AX-1's `cgap-ax-testbed:ax1` (amd64 go): cgo **BLOCKED** (mixed-arch,
  W2 row 2) — finding for the owning lane.
- DFX-6 clause 1: **CLOSED** (image dimension).

## W6 Divergences (doc yields to reality)

1. **alloc.sh in-image arm defect (AX-4's file, not modified)**: the arm
   cargo-builds with no cwd (defaults to /) → exit 101 "could not find
   Cargo.toml" (W2 row 1b). AX-4 fixed the same defect class (`cd /work`)
   in the FALLBACK arm (lane-ax-04-alloc.md W2 row 3) but the in-image
   arm never ran until now. Minimal fix = add `cd /work` at the head of
   the in-image bash payload (alloc.sh:36-45). No SLA unblock applied —
   the file is another lane's and the courts are honest; only the
   wrapper is defective. Owner (AX-4 or successor) applies the one-liner.
2. **Mixed-arch defect in AX-1's Dockerfile (finding for owner, shared
   tree untouched)**: `go1.23.4.linux-amd64.tar.gz` over arm64-pulled
   bases yields amd64 go emitting `-m64` into the aarch64 gcc; every cgo
   build in `cgap-ax-testbed:ax1` dies (W2 row 2, exit 1). I patched the
   SPLICED COPY only (documented in build_image.sh's python assembler,
   with an assert the substitution applied); AX-1's file is byte-untouched.
3. **ffi.go pin path**: `-L/tmp/ipc2-target/release` (ffi.go:18) is a
   HOST path; in-image it exists only because the composed image seeds
   it (final-stage RUN mkdir+cp). Host copy seeded at W2 row 5 from the
   image's own arm64 .so (exact subject, built from this tree).
4. Tag naming: mutation.sh pins `IMAGE=cgap-ax-testbed` (:latest) while
   alloc.sh defaults `AX_IMAGE=cgap-ax:latest`; the composed image was
   tagged `cgap-ax-testbed:latest` and alloc.sh was run with
   `AX_IMAGE=cgap-ax-testbed:latest` (env override; runner untouched).

## W7 Fences

- darwin-host courts untouched: `go build ./...` + `go test ./ffi/`
  exit 0 this session (W2 row 7). No Go/Rust source edited; shared
  tree untouched outside the two owned files. Compile-freeze SLA not
  engaged. Leases: /tmp/tc14-ctx deleted at script exit; /tmp/ax4-shared
  and /tmp/ipc2-target/release deliberately left (IPC-2/AX-4 pinned
  binding paths, per AX-7 W7 precedent).

## JSON terminus

{"lane":"TC-14","files_written":["courts/ax-testbed/build_image.sh","docs/jira/v26.11.2/lanes/lane-tc-14-ax-rewitness.md"],"commands":["bash courts/ax-testbed/build_image.sh=0","AX_IMAGE=cgap-ax-testbed:latest bash runners/alloc.sh=101 (runner defect, W6-1)","in-image alloc payload exit=0 (6/6 PASS, flip witnessed)","host .so seeding=0","bash courts/ax-testbed/runners/mutation.sh=0 (arms 1/2/4 PASS in-image, arm 3 loud SKIP)","darwin go build+test=0"],"courts_run":["TestBLAKE3OfficialEmptyVector","TestAdmitPositiveControl","TestEvalPositiveControl","TestSteadyStateAllocPurity","TestVersionAllocPurity","TestAntiVacuityFlip","TestArm1MagicFlipRefusesTyped","TestArm2CoordinateMutationResultDiffers","TestArm3OverflowTypedAtDerivedBound (loud SKIP)","TestArm4AllocPositiveControl"],"standing":"ALIVE (composed image build + in-image alloc 6/6 + mutation arms 1/2/4); DFX-6 clause 1 CLOSED; alloc.sh in-image arm BLOCKED (runner defect, one-liner fix owned by AX-4); AX-1:ax1 cgo BLOCKED (mixed-arch finding)","divergences":["alloc.sh in-image arm no-cwd defect (not modified, W6-1)","AX-1 amd64 go tarball mixed-arch defect (spliced copy patched only, W6-2)","ffi.go host-path pin seeded in-image (W6-3)","tag naming divergence alloc.sh vs mutation.sh (W6-4)"]}
