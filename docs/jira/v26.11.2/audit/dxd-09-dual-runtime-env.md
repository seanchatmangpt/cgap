# dxd-09-dual-runtime-env

Joint env prerequisite matrix for the Rust+Go runtimes (2026-11-02,
read-only lane). Spec input for DXD-02's merged doctor; reconciled
against audit/dxd-02-doctor-design.md — adds llvm-tools paths,
CGO/rpath coupling, and lease-pressure rows dxd-02 omits.

## Detection surface today

- scripts/doctor_agent_lanes.py — 4 checks, exit max(0/1/2).
- crates/cli/src/bin/doctor.rs — 10 checks (c01 toolchain, c02
  wasm-targets, c03 git, c04 b3sum, c05 pyshacl, c06 workspace,
  c07 scripts, c08 receipts, c09 lsl, c10 dirty-tree). Hand-
  maintained side-bin, NOT ggen-regenerated.
- scripts/verify_lsp_branchless.py — staleness guard arm-B:
  strings|grep -c ANDON_REFUSED == 0 -> exit 3 typed skip.
- scripts/lease_janitor.py — DX-3 grammar, 60-min window,
  fail-closed refusal.
- scripts/run_coverage.sh — NIGHTBIN/llvm-profdata resolve.
- NOTHING checks go version, CGO/rpath linkage, or ipc3-target
  dylib freshness. dxd-02's 6 new checks are DESIGN only.

## Joint dependency matrix

| prereq | detection | absent -> | canonical command | witnessed |
|---|---|---|---|---|
| nightly toolchain | rust-toolchain.toml; doctor.rs c01 checks cargo/rustc NOT channel | silent wrong-toolchain builds | `rustc --version` (1.101.0-nightly) | GREEN |
| riscv32imc target | doctor.rs c02 checks wasm32 ONLY, not riscv | AE4 + enclave_budget_gate hard-fail | `rustup target list --installed | grep riscv32imc` | GREEN |
| toolchain llvm-tools bin | CLAUDE.md nightly-aarch64 lib/rustlib path; only run_coverage.sh resolves NIGHTBIN | coverage lane fails; doctor blind | `ls ~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata` | untested |
| homebrew llc riscv backend | CLAUDE.md: toolchain llc LACKS riscv; /opt/homebrew/opt/llvm has one; enclave_budget_gate.py:111 | disasm courts silently wrong llc or fail | `/opt/homebrew/opt/llvm/bin/llc --version | grep -c riscv` | GREEN |
| llvm-profdata on usable PATH | run_coverage.sh:11 loop | coverage hard-fail; only xcrun finds it | shutil.which('llvm-profdata') | WARN witnessed |
| go >= 1.22 | NOTHING | Go lanes fail at build | `go version` | GREEN 1.23.4 |
| dylib fresh | NOTHING; dxd-02 check 4 design (mtime vs src+Cargo.toml+include/cgap.h) | SILENT FALSE-PASS (frozen 11-symbol superset links; old-behavior tests exit 0 vs wrong dylib) | newest-mtime compare | GREEN (615552 B, 12:54) |
| CGO_ENABLED/rpath | NOTHING; ffi.go:18 bakes absolute -L/-rpath; do NOT add DYLD_LIBRARY_PATH docs (masks broken rpath) | rpath break fails at exec; DYLD override would false-pass | `otool -l <bin> | grep -A2 LC_RPATH` | GREEN |
| ipc3-target lease dir | NOTHING (deliberate keep, lane-ipc-03 :92-96) | cgo courts fail to link, no warning | `ls /tmp/ipc3-target/release/libcgap_ffi.dylib` | GREEN; ipc0-target DELETED |
| /tmp lease pressure | orphan-leases (13-pattern subset) + janitor full grammar | silent disk bloat; rm-denied debris | `python3 scripts/lease_janitor.py` | 12+ live *-target |

## Failure-mode classification

- HARD-FAIL: riscv target missing, llvm-profdata missing, go
  missing (cgo lane), janitor grammar refusal.
- SILENT-SKIP / FALSE-PASS (the doctor's real targets): stale
  dylib; nightly drift (c01 GREEN on any rustc); rpath masked by
  DYLD_LIBRARY_PATH; wasm-only c02 never notices riscv absence.
- LOUD-SKIP precedent: verify_lsp_branchless exit-3 guard — the
  pattern dxd-02's checks reuse.

## Reconciliation adds to dxd-02's spec

(a) nightly-channel check (c01 gap); (b) homebrew-llc riscv-backend
probe (CLAUDE.md llvm-tools law); (c) rpath presence check replacing
any DYLD normalization; (d) ipc3-target lease presence (WARN not
RED — deliberate keep); (e) adopt janitor grammar as the
orphan-lease pattern source (doctor's list is a stale subset).

## All-GREEN today

rustc 1.101.0-nightly, riscv32imc installed, go1.23.4, /opt/homebrew
llc present, target/release dylib fresh vs src.

## See Also

audit/dxd-02-doctor-design.md · audit/dxd-03-go-ergonomics.md ·
audit/dxd-08-janitor-design.md · lanes/lane-ipc-00-ground-truth.md
