# Lane DOD-13 Receipt — AX-4 alloc.sh no-cwd fix + AX-1 mixed-arch fix

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout).
No git commands. Closes the two defects TC-14 witnessed
(`lanes/lane-tc-14-ax-rewitness.md` W6-1 and W6-2); that receipt is the
consuming evidence — read before this one.

## W1 Files written

- `courts/ax-testbed/runners/alloc.sh` (MODIFIED, in-image arm only):
  added `cd /work` at the head of the in-image bash payload and corrected
  the dylib copy source `/tmp/target-release/` → `/tmp/target/release/`
  (the image ENV is `CARGO_TARGET_DIR=/tmp/target`; the old path never
  existed and silently fell to the `find` fallback).
- `courts/ax-testbed/Dockerfile` (MODIFIED, final go-install RUN):
  replaced the hardcoded `go1.23.4.linux-amd64.tar.gz` with a TARGETARCH
  case map (`amd64|arm64`), loud `exit 1` on any other arch, tarball URL
  derived from `GO_TARBALL`.
- this receipt — the only other file.

## W2 Commands + exits (all run this session, 2026-11-02)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `bash courts/ax-testbed/build_image.sh` | **0** | composed image rebuilt; step #22 ran the TARGETARCH case with `arm64`, `go version go1.23.4 linux/arm64` printed IN BUILD; artifact courts PASS; manifest list sha256 66a575e0feb1... |
| 2 | `AX_IMAGE=cgap-ax-testbed:latest bash courts/ax-testbed/runners/alloc.sh` (run 1) | 1 | cargo refused the workspace: `crates/cgap/Cargo.toml:10` had string-form `categories` (another lane's IN-FLIGHT publish edit; see W6-2) |
| 3 | same command (run 2, after the owning lane's concurrent fix landed on disk) | **0** | cargo build 5.54s; **6/6 PASS** incl. TestAntiVacuityFlip (`escaping closure = 1 allocs/op, no-op = 0`); lease cleaned |
| 4 | `docker run --rm --platform linux/arm64 cgap-ax-testbed:latest bash -c 'go version && uname -m && cc /tmp/c.c -o /tmp/c && /tmp/c'` | **0** | `go1.23.4 linux/arm64`, `aarch64`, native cgo link OK |

## W3 Defect closures (each names its falsifier)

1. **AX-4 alloc.sh no-cwd (TC-14 W6-1)**: falsifier was "cargo runs with
   no cwd → exit 101 could not find Cargo.toml". Fixed with `cd /work`;
   the falsifier no longer reproduces (W2 row 3, exit 0, cargo built).
2. **AX-1 mixed-arch go tarball (TC-14 W6-2)**: falsifier was "amd64 go
   emits `-m64` into aarch64 gcc → every cgo build dies". Fixed with
   TARGETARCH derivation; in-image `go version` is now `linux/arm64` and
   CGO_ENABLED=1 courts pass end-to-end (W2 rows 1, 3, 4). The amd64
   fallback branch remains for genuine amd64 builds; unsupported arches
   fail the build LOUD at that RUN (anti-vacuity: a wrong-arch base can
   no longer produce a silently mixed image).

## W4 Anti-vacuity

- Build court: the `*) exit 1` case arm — a hypothetical future base
  arch mismatch aborts the build, not the courts.
- Runtime court: TestAntiVacuityFlip flips 1 vs 0 allocs/op INSIDE the
  rebuilt image (W2 row 3) — the fixed runner is not vacuously green.

## W5 Standing

- Composed image build (post-fix): **ALIVE** (exit 0, W2 row 1,
  manifest list 66a575e0...).
- alloc.sh in-image arm as-written: **ALIVE** (exit 0, 6/6, W2 row 3).
- In-container go/arm64 + cgo link: **ALIVE** (exit 0, W2 row 4).
- TC-14's BLOCKED rows for these two defects: **CLOSED**.

## W6 Divergences

1. **Spliced-copy case collapse (build_image.sh is not my file)**:
   build_image.sh's assembler replaces the literal
   `go1.23.4.linux-amd64.tar.gz` → `linux-arm64` in the SPLICED context
   copy, which turned my `amd64)` case branch into `arm64)` in the
   built image (both branches arm64 there). Harmless on this arm64
   host, assert passed, build honest — but the composed image is
   arm64-only until build_image.sh's substitution is retired by its
   owner (TC-14). The canonical `courts/ax-testbed/Dockerfile` is
   arch-correct for amd64 and arm64.
2. **Concurrent inter-lane edit (disclose-in-both-directions)**: W2 row
   2 failed on `crates/cgap/Cargo.toml:10` (string-form `categories`)
   — another lane's in-flight publish-metadata edit to a file I do not
   own; I did NOT touch it. The owning lane landed the array-form fix
   (mtime 16:11:50, `git diff` shows `categories = ["no-std", ...]`)
   before my row 3 run. I cite it; the owner supersedes. Shared
   workspace parse is green again (row 3 cargo build exit 0 proves it).
3. First alloc run's exit code was mis-read by my own `${PIPESTATUS}`
   (zsh uses `pipestatus`); row 3 re-ran with a correct capture — the
   reported exits are the corrected ones.

## W7 Fences

- darwin-host fence: no Go/Rust source edited; `courts/interop-chicago`
  untouched; the darwin `go test ./ffi/` gate is unaffected by
  construction (container-only changes). Not re-run here — TC-14 W2
  row 7 is the last witness and no host file changed since.
- build_image.sh untouched (read-only for this lane); its spliced-copy
  substitution divergence is recorded, not patched (W6-1).
- Leases: /tmp/tc14-ctx deleted by the script; /tmp/ax4-shared deleted
  by alloc.sh; /tmp/dod13-alloc.log retained as row-3 evidence.
- Compile-freeze SLA engaged once as OBSERVER only (W6-2); no unblock
  applied by this lane.

## JSON terminus

{"lane":"DOD-13","files_written":["courts/ax-testbed/runners/alloc.sh","courts/ax-testbed/Dockerfile","docs/jira/v26.11.2/lanes/lane-dod-13-ax-fixes.md"],"commands":["bash courts/ax-testbed/build_image.sh=0","AX_IMAGE=cgap-ax-testbed:latest alloc.sh run1=1 (foreign in-flight Cargo.toml, W6-2)","alloc.sh run2=0 (6/6 PASS, flip 1 vs 0 allocs/op)","docker run go version + cgo link witness=0 (go1.23.4 linux/arm64, aarch64)"],"courts_run":["TestBLAKE3OfficialEmptyVector","TestAdmitPositiveControl","TestEvalPositiveControl","TestSteadyStateAllocPurity","TestVersionAllocPurity","TestAntiVacuityFlip","composition artifact courts (build step 4)","TARGETARCH loud-refuse case arm"],"standing":"ALIVE (image build exit 0; alloc.sh in-image arm exit 0, 6/6 with flip; go/arm64 cgo witness exit 0); both TC-14 BLOCKED rows CLOSED","divergences":["build_image.sh spliced-copy substitution collapses amd64 case branch to arm64 in the composed image (owner TC-14 to retire, W6-1)","concurrent foreign edit to crates/cgap/Cargo.toml broke row-2 run; owner fixed on disk before row 3 (disclosed both directions, W6-2)","zsh PIPESTATUS mis-read corrected by re-run (W6-3)"]}
