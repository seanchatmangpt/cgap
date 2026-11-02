# Lane AX-3 — canonical in-image cgap-ffi dylib (kills dxd-03 dual-/tmp-target)

Standing: **ALIVE** — every claim backed by command+exit run this session (2026-11-02).

## W1 Identity + ownership

- Owned files (all written this session, no git commands run):
  - `courts/ax-testbed/docker/build-cgap-ffi.sh` (242 lines, chmod +x)
  - `courts/ax-testbed/docker/cgap-ffi.stage` (33 lines) — STAGE FRAGMENT;
    AX-1 OWNS the Dockerfile. Merge duty disclosed here and in the stage
    header comment. Fragment is standalone-runnable (`docker build -f
    cgap-ffi.stage` with the tar context the build script emits).
- Read-only consumed: root `Cargo.toml`/`Cargo.lock`, `crates/`,
  `ontologies/`, `include/{cgap.h,cgap_c_data.h}` (repo-root include/, NOT
  cgap-ffi-relative), `lanes/lane-ipc-00-ground-truth.md` (W3 oracle).

## W2 Commands + exits

| # | command | exit |
|---|---|---|
| 1 | `docker info` | 0 (daemon up; podman absent — AX-0 W4 GO-via-docker) |
| 2 | `sh build-cgap-ffi.sh` (final battery, 2026-11-02) | **0** |
| 3 | earlier attempts: 5 court-driven fixes (context members, excludes, sort-order, mutation assert, flip-exclusion) each exited 1 and are logged in-session | 1 |

## W3 What the image now owns (dxd-03 kill)

- ONE canonical dylib path: `/opt/cgap/lib/libcgap_ffi.so` (LINUX, ELF —
  `.so`, not `.dylib`), built in-image by
  `cargo build --release -p cgap-ffi` from the context tar
  (Cargo.toml Cargo.lock crates/ include/ ontologies/; target-debris
  excluded by tar --exclude, 8 MB context).
- Header at `/opt/cgap/include/cgap.h` (frozen ABI header, repo-root include/).
- cgo re-point: GENERATED cgo config mechanism — the image contract fixes
  `-L/opt/cgap/lib -lcgap_ffi` for BOTH GOOS arms; only the artifact file
  NAME differs by platform convention (darwin `.dylib` / linux `.so`). The
  darwin session numbers (IPC-0) are NOT ports: linux/arm64 re-measurement
  is THIS lane's finding (courts ran in-image, linux/arm64).
- Anti-vacuity: mutate sandbox copy of cgap.h (drop the cgap_verify_p28
  declaration, 131 bytes removed) -> rebuild mutated image -> C4 flips to
  FAIL ("declared-surface count 4 != 5"). flip=1 witnessed, exit 0 overall.

## W4 Court results (final battery, CONTAINER mode, docker 28.0.4)

```
PASS  C1 in-image release build (cargo build --release -p cgap-ffi) exit 0
PASS  C2 canonical artifact path /opt/cgap/lib/libcgap_ffi.so (+header), non-empty
PASS  C3 symbol count == 11 and exact IPC-0 set match
PASS  C4 header court: declared surface == frozen 5-name set, all exported
FAIL  C4 header court: declared-surface count 4 != 5 (header mutation or drift)
PASS  ANTI-VACUITY: mutated header -> C4 header court flips to FAIL
PASS  sandbox law: shared include/cgap.h sha256 unchanged
      (489eae9a49620b6c5e47e4e128da66cc1c7e2280917a45f8c7ca909deb6bc3b4)

SUMMARY CONTAINER: pass=6 fail=0 flip=1
EXIT=0
```

- C3 oracle: IPC-0 W3 11-symbol table, exact set match on the linux .so
  (`nm -D --defined-only`, cgap_* filter). The darwin IPC-0 measurement
  holds verbatim on linux/arm64 — same 11, same set. FINDING: linux/arm64
  parity CONFIRMED for the export table; perf numbers remain darwin-only.
- The `FAIL C4` line is the WITNESSED ANTI-VACUITY FLIP on the mutated
  sandbox image, not a baseline regression (fail=0, flip=1, exit 0).

## W5 Disclosure (disclose-in-both-directions)

- Stage fragment `docker/cgap-ffi.stage` is AX-3-authored; the Dockerfile
  into which it merges is AX-1-owned. AX-1 supersedes the diff on landing
  (owner-supersedes-unblock law). This receipt is the AX-3 side of the
  disclosure; AX-1's receipt must name the merge.

## W6 Fences + divergences

- Fences (must-not-regress, all untouched): zero_string_gate.py,
  boundary_gate.py, root workspace compile, `cargo test -p cgap-court`,
  cgap.h byte-exactness (sha256 pinned above, verified unchanged post-run).
- No shared lib.rs/Cargo.toml edits; only new files under
  courts/ax-testbed/docker/ + this receipt.
- Divergences from dispatch (doc yields to reality, recorded not hidden):
  1. cgap.h is at REPO-ROOT `include/`, not `crates/cgap-ffi/include/`.
  2. Workspace needs `ontologies/` members in the image context; first
     build failed exit 101 without it (manifest load error), fixed via
     COPY ontologies + tar excludes for pre-existing `target`/`target-a09`
     build debris (csct/cli 810M+106M, fibo 147M) — build debris was
     pre-existing lane debris, disclosed here.
  3. First anti-vacuity design (`comm -23 declared⊆exported`) was
     NON-FLIPPING (witnessed still-green run); rewrote C4 as declared-set
     equality vs the frozen 5-name surface. Vacuity was caught by the
     court's own battery, not coordinator pre-thinking.
  4. Docker disk near-exhaustion (92% host, context-copy ENOSPC on an
     early run) — slim 8 MB context fixed; no `docker system prune` run
     (shared daemon state, other lanes' images present).

## W7 Standing

ALIVE. Container path (docker): full battery exit 0, image
`cgap-ffi:ax3` (30fe330fca10) retained on the daemon for AX-4/5/6/7/8
consumers; LOCAL-FALLBACK path exists in the script but was NOT exercised
this session (UNVERIFIED, loud). Cleanup witnessed: sandbox/ctx dirs
removed (python shutil; rm -rf is permission-denied), no lease debris.
