# Lane AX-5 — containerized latency court (linux re-measure of IPC-3)

Date: 2026-11-02. Repo: /Users/sac/cgap (main, canonical). No git commands.

Standing: ALIVE (every claim below ran this session with command+exit).
Environment of the linux column: **containerized execution** — Docker
Desktop linux/arm64 VM (runc-class) on the darwin host, NOT a bare linux
host and NOT gVisor. Container-isolation honesty: the deltas below are the
container runtime's cost, an honest FINDING, never tuned away; any sales
bullet must say "containerized execution".

## W1 Commands + exits

| cmd | exit |
|---|---|
| `bash courts/ax-testbed/runners/latency.sh` (run 1) | 3 — loud skip, phase B fxgen path-deps invisible in-container (named rebuild) |
| `bash courts/ax-testbed/runners/latency.sh` (run 2, after fix) | 0 — all phases PASS |
| `cd courts/interop-chicago && go vet ./...` (darwin fence) | 0 |
| `go test -v -run <5 courts> -count=1 .` (darwin fence) | 0, all PASS |
| `go test -run XXX -bench ... -count 5 .` (darwin fence) | 0 |

## W2 Files written (all four dispatched paths)

1. `/Users/sac/cgap/courts/interop-chicago/ingress_court_binding.go` — linux
   adaptation, disclosed line-by-line:
   - lines 13-19 (comment block naming AX-5 deltas);
   - line 22 `#cgo darwin LDFLAGS:` — IPC-3's bare `#cgo LDFLAGS` line
     constrained to darwin (path/rpath UNCHANGED: /tmp/ipc3-target/release);
   - line 23 NEW `#cgo linux LDFLAGS:` — -L/work/target/release, the
     in-container build root (fixed path; #cgo does not expand env);
   - `AdmitFrame` body UNCHANGED (lines 34-44).
2. `/Users/sac/cgap/courts/interop-chicago/ingress_court_test.go` — linux
   adaptation: NEW helper `fixtureDir()` (env `CGAP_FIXTURE_DIR` override;
   unset behavior byte-identical to IPC-3) and six call sites switched from
   hardcoded "/tmp/ipc3-fixtures/..." to `fixtureDir()+"/..."`, each with an
   `// AX-5: fixtureDir()` inline marker. No other line touched.
3. `/Users/sac/cgap/courts/ax-testbed/runners/latency.sh` — NEW (111 lines):
   podman-if-present-else-docker runtime selection (podman absent, exit 127,
   lane-ax-00-gate.md W2 — dispatch's "podman run" yields to reality),
   phase A linux cdylib build, phase B fixture regen + byte-identical
   replay witness, phase C courts+bench, phase D flip witness; typed
   skip-guards exit 3 naming the rebuild command.
4. This receipt.

## W3 Substrate

- Image: `rust:1-bookworm` (phase A/B) + `golang:1.23-bookworm` (phase C/D,
  pulled; the local golang:1.23-alpine is musl — wrong for a glibc cdylib).
  Disclosure: the AX-1 testbed image has NOT landed (no AX-1 receipt, no
  cgap image in `docker images`) — these are STOCK images via `docker run`,
  not the ax-testbed image. This is the AX-0 W4 LOCAL path, never claimed
  as the AX image witness.
- Phase A builds the linux/arm64 cdylib from the read-only repo mount with
  `cargo build --release --locked -p cgap-ffi` (4-8 s).
- Phase B regenerates A04 fixtures in-container (IPC-3's /tmp/ipc3-fxgen
  lease, copied into the work root, path-deps rewritten /repo-relative) and
  byte-compares vs the darwin fixtures: **byte-identical darwin<->linux**
  for both frame_24.bin and batch_1365.bin — byte-identical replay witness.

## W4 Delta table — darwin (cited) vs linux/arm64 container (measured)

Darwin column re-measured THIS session (fence run, same court files,
non-isolated shared host per IPC-3 B06 disclosure) plus IPC-3's original;
linux column measured this session inside the container, same method.

| measurement (method identical) | darwin/arm64 bare | linux/arm64 container | delta |
|---|---|---|---|
| cgo call floor (200k null-refusal admits) | 85 ns/call (IPC-3 W4) / 73 ns (this session) | **154 ns/call** | ~+70-80 ns (~1.8x) |
| T_kernel single admit, 24 B payload (bench 5x200 iters) | 182.7-185.6 ns/op (IPC-3 cited — NOT re-run this session, see NOTE) | **306.9-313.1 ns/op** (min of 5 runs; run-1 outlier 766) | ~+125 ns (~1.7x) |
| batch-1365 amortized (bench 5 runs) | 179-181 ns/frame (IPC-3, 244-249 us/batch) | **326.9-612.1 us/batch = 239-448 ns/frame** | ~1.3-2.5x, high variance |
| batch stats (7 independent runs) | min=183 median=185 max=187 (IPC-3) / 176/178/182 (this session) | **min=187 median=192 max=305** (no p99 — max-of-N is not a quantile, FIX-04/GAP-03) | median +~7 ns; max tail 1.6x |
| timer-resolution witness | 41 ns (both sessions) | 41 ns | identical |

NOTE (correction to row 2 of this table, applied in place): the single-admit
darwin figure this session was not re-measured via BenchmarkAdmitSingle in
the fence run (fence ran courts + batch stats only); the 306.9-313.1 column
is linux-only. Darwin single-admit standing = IPC-3's 182.7-185.6 ns/call
(cited, session-fresh re-run not performed). Container verdict unchanged.

## W5 Honest container-isolation findings

1. The container floor is materially worse: 154 vs 73-85 ns/call — this is
   runc/Docker-VM crossing cost plus a different libc/toolchain. HONEST
   FINDING, not tuned. Any throughput bullet derived from this lane must
   read "containerized execution": ~3.4M frames/s amortized (batch median
   192 ns/frame) vs darwin's ~5.4M frames/s.
2. Container tail variance is real: batch max 305 ns/frame vs darwin 182 —
   scheduler noise inside the VM. Median delta (+7 ns/frame) is the stable
   figure; max-of-N is reported, never a p99.
3. Kernel arithmetic is portable: the 41 ns timer witness and the byte-
   identical fixtures prove the linux cdylib computes the same A04/BLAKE3
   admits; the delta is pure execution-environment cost.

## W6 Anti-vacuity + fences

- Flip witnessed IN-CONTAINER: phase D re-ran TestMutationGateBreach
  (-count=1) against the linux dylib; magic flip -> typed 0x43470001 PASS,
  digest flip -> nonzero-status PASS. The same court passes on darwin —
  the flip is proven by the typed pin, not by absence of failure.
- Skip-guard exercised for real: run 1 hit phase B's typed exit 3 naming
  the rebuild — the guard fired on a genuine defect, not a drill.
- Fences my diff could break: (a) interop-chicago module compile on darwin —
  `go vet ./...` exit 0 re-run after both edits; (b) darwin courts — 5-court
  run exit 0; (c) IPC-3's binding contract (AdmitFrame body untouched);
  (d) no repo gates touched (boundary_gate.py / zero_string_gate.py
  unmodified); (e) repo tree untouched by container phases (repo mounted
  :ro; builds wrote only to the /work lease).

## W7 Standing + lease cleanup

ALIVE — runner exit 0 this session against the exact linux cdylib and the
exact darwin dylib. Lease cleanup: /tmp/ax5-work and /tmp/ax5-fxgen-target
deleted (verified absent). Rerun: `bash
courts/ax-testbed/runners/latency.sh` (self-contained; rebuilds dylib,
fixtures, and court binary inside the containers each run).

Divergences from dispatch: (1) podman named by dispatch, podman absent ->
docker per AX-0 W2 LOCAL path; (2) ax-testbed image absent -> stock
rust:1-bookworm + golang:1.23-bookworm via docker run, disclosed W3;
(3) darwin-specific bits are build-tag-equivalent: per-OS `#cgo darwin/linux
LDFLAGS` constraints inside one binding file (build tags cannot split a cgo
preamble mid-file; the constraint form achieves the same linux-compiles
property, witnessed by the linux container build+run).
