# Lane AX-1 Receipt — AX Testbed Image (courts/ax-testbed)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout).
Reads AX-0 verdict: `lanes/lane-ax-00-gate.md` (docker local path = GO).

## W1 Identity

- Lane AX-1. Owned files (all new):
  - `/Users/sac/cgap/courts/ax-testbed/Dockerfile` (sha256 2a51bb85...)
  - `/Users/sac/cgap/courts/ax-testbed/entrypoint.sh` (sha256 30557598...)
  - `/Users/sac/cgap/courts/ax-testbed/.dockerignore` (sha256 adcf4489...)
  - this receipt.
- Image tag `cgap-ax-testbed:ax1`, built this session.

## W2 Authority

Dispatch prompt (owned-files manifest above). No git commands run against
the canonical checkout. Sandbox mutations confined to /tmp copies.

## W3 Commands + exits (all this session)

| # | Command | Exit |
|---|---|---|
| 1 | `git ls-remote https://github.com/google/ax HEAD` | 0 -> b82f57ee20f8882bb62762fc8f0c24f1c3c27f86 (pin grounded from upstream, not @latest) |
| 2 | `docker buildx imagetools inspect` golang:1.23-bookworm / debian:bookworm-slim | 0 -> digests 167053a2... / 99c0b25c... |
| 3 | `docker build -t cgap-ax-testbed:ax1 .` (first attempt) | 1 (drift court: pinned commit HAS cmd/ax-task-runner, but go.mod requires go 1.27.1; guard first mis-read this as "package absent") |
| 4 | rebuild after GOTOOLCHAIN=auto fix | 0 |
| 5 | rebuild after telemetry printf fix | 0 (final image) |
| 6 | `docker run ... go version` | 0 -> go1.23.4 linux/amd64 |
| 7 | `docker run ... rustc --version` | 0 -> 1.101.0-nightly (3e92e1a2d 2026-11-01) |
| 8 | `rustup target list --installed \| grep riscv32imc` (in-container) | 0 -> riscv32imc-unknown-none-elf |
| 9 | anti-vacuity flip court (see W5) | canonical 0 / mutated 1 |
| 10 | `entrypoint.sh --help` in-container | 0 (telemetry JSON + real runner Usage) |

## W4 Consequence

- Two-stage digest-pinned image: stage 1 golang:1.23-bookworm builds
  ax-task-runner at pinned commit b82f57e (CGO_ENABLED=0); stage 2
  debian:bookworm-slim with Go 1.23.4, rustup nightly 1.101.0-nightly +
  llvm-tools-preview + rust-src + riscv32imc target, clang/llvm, python3, jq.
- Entrypoint: exports GOROOT/CARGO_HOME/PATH, CGO_ENABLED=1, mkdir TMPDIR,
  once-per-container boot telemetry (marker file), then execs the REAL
  stage-1 binary (missing binary = loud COURT FAIL, never a placeholder).
- Fences: none touched (new files only, no lib.rs/Cargo.toml edits; no
  shared gates consume this dir). Compile-freeze SLA not engaged.

## W5 Courts table

| Court | Mutation (anti-vacuity) | Flip witness | Exit |
|---|---|---|---|
| Build drift court | pinned-commit package check in stage 1 | first build exit 1 when guard mis-fires loud | 1 -> fixed, rebuild 0 |
| Installed-target grep court | /tmp Dockerfile copy with `rustup target add riscv32imc-unknown-none-elf` replaced by `true`; rebuild tag `:mutated` | canonical grep exit 0; mutated exit 1 | 0/1 flip WITNESSED |
| Entrypoint telemetry + real-exec court | none (positive court) | telemetry JSON (ax_commit pinned SHA) + `Usage of /usr/local/bin/ax-task-runner` real flags (-port/-metadata-port/-task-file), exit 0 | 0 |

Strengthening note: the first grep court form
(`rustup target list \| grep`, without `--installed`) was VACUOUS — it
matched the target in the available list of the mutated image. Corrected
to `--installed`; the flip is witnessed only in the `--installed` form.

## W6 Standing

- Image build (docker 28.0.4): **ALIVE** (exit 0, this session, exact tag).
- go1.23.4 / rustc nightly / riscv32imc installed-target witnesses: ALIVE.
- ax-task-runner binary: ALIVE (19,503,884 B, real Usage output, pinned
  commit b82f57ee burned into /usr/local/share/ax_commit.txt).
- Unpinned surfaces (recorded, per dispatch): rustup nightly is floating
  (resolved to 1.101.0-nightly 2026-11-01 at build time); apt package set
  unpinned. Base images ARE digest-pinned. AX pin grounded via
  git ls-remote (not a local README — upstream reachable, so no
  placeholder needed and no UNVERIFIED flag on the pin).
- podman: absent (docker used; per AX-0 GO).

## W7 Divergences / disclosures

1. **go.mod divergence (doc yields to reality)**: dispatch said stage-1
   golang:1.23 builds the runner; go.mod at the pin requires go 1.27.1.
   Resolved with `GOTOOLCHAIN=auto` (documented in the Dockerfile itself),
   keeping the chartered FROM while Go bootstraps 1.27.1.
2. **First-build drift court mis-fire**: the first guard suppressed
   stderr, so a module-load failure read as "package absent". Guard
   rewritten loud (`go list ./cmd/ax-task-runner` without suppression).
3. **Vacuous first-form grep court**: strengthened to `--installed`
   (recorded in W5; first form never claimed a pass on the canonical image
   without the corrected form re-running).
4. **No SLA unblocks applied** to other lanes' files; no file outside the
   owned manifest written.
