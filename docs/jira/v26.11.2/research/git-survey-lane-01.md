# git-survey-lane-01.md

Lane GS1 (read-only vs source/manifests; sole write = this file). Subject:
`crates/cgap-git`. All commands run 2026-11-02, cwd reset per call.

## Commands + exits

| Command | Exit | Result |
|---|---|---|
| `cargo check -p cgap-git --target riscv32imc-unknown-none-elf` | 101 | E0463 std, lib.rs:13:1 |
| same, `--no-default-features` (debug) | 101 | 4 errs: E0433 ladder.rs:31:5, E0425 x3 |
| same, `--no-default-features --release` | 0 | clean |
| `cargo build` (no-def, release, riscv) | 0 | rlib emitted |
| `cargo tree -p cgap-git --target riscv32imc-unknown-none-elf` | 0 | 0 runtime deps |
| `llvm-ar t <rlib>` | 0 | 3 members: lib.rmeta, lib.rmeta-link, 1 .rcgu.o |
| `llvm-size -A <rlib>` | 0 | only .rmeta (142 B) / .rmeta-link (81 B) — no code sections |
| `llc -mtriple=riscv32imc-unknown-none-elf` | 1 | no RISC-V backend in llc |

## (1) Cargo.toml inventory

`crates/cgap-git/Cargo.toml` — 24 lines:

- `[dependencies]` is EMPTY (lines 15-16): zero path deps, zero external
  deps at runtime. Published as `publish = true` (line 8).
- `[features]` (lines 11-13): `default = ["std"]`, `std = []`.
- `[dev-dependencies]` (lines 22-23): `cgap-blake3`, `cgap-parquet`
  workspace path deps, dev-only per Cargo.toml:18-21 comment (v26.11.2
  lane p1-lineage); add nothing to the published kernel.
- std/alloc posture: the `std` feature gates `extern crate std` +
  `std::error::Error` impls; everything else rides `alloc`
  (lib.rs:15) / `core`.

## (2) riscv32imc compile + footprint

AS-OF-NOW state: the `extern crate std;` line cited by
m2-nostd-census.md:21 is STILL PRESENT at
`crates/cgap-git/src/lib.rs:13`, but it is cfg-gated:
`#[cfg(feature = "std")]` (lib.rs:12). The census recorded it as
"explicit ... despite #![no_std]" without noting the gate; the as-now
error is therefore feature-conditional, not unconditional.

- Default features (std on), riscv32imc: FAILS, exit 101, E0463
  can't find std at lib.rs:13:1 (target has no std).
- `--no-default-features`, debug profile: FAILS, exit 101, 4 errors —
  `std::thread_local!` at `src/ladder.rs:31` inside
  `#[cfg(debug_assertions)] mod probe_count` (ladder.rs:27-32); the
  three E0425 `PROBES` errors are cascade from that module.
- `--no-default-features --release` (debug_assertions off):
  BUILDS CLEAN, exit 0. Divergence vs m2 census: the census ran
  default-features and read a feature-gated line as unconditional;
  cgap-git is no_std-clean in release with the std feature off, but a
  DEBUG-profile no_std build still fails at ladder.rs:31 (ungated
  `std::thread_local!` — should be `core::prologue`-free; a fix would
  move probe_count behind `#[cfg(all(debug_assertions, feature =
  "std"))]` or use a core-only counter).

### Footprint: [NOT-MEASURABLE: method]

Method attempted: built release rlib (exit 0), `llvm-ar t` shows
lib.rmeta + 1 `cgap_git-...rcgu.o`; llvm-size -A on the rlib reports
only .rmeta (142 B) and .rmeta-link (81 B); extracting the member, the
.rcgu.o is **LLVM IR bitcode** (`file` output), not an ELF object, and
the pinned toolchain's `llc` has no RISC-V backend ("unable to get
target for 'riscv32imc-unknown-none-elf'"), so no .text/.rodata/.data/
.bss could be produced. Per lane doc, recorded rather than forced.
Host-side proxy: rlib on disk 229,928 B; rmeta 323,964 B. Perf/size
claims stay TARGETs until a `cargo build -Z build-std=core` +
`rust-lld` link route exists.

## (3) lib.rs root audit

`crates/cgap-git/src/lib.rs` (133 lines):

- `#![no_std]` — lib.rs:9
- `#![forbid(unsafe_code)]` — lib.rs:10
- `#[cfg(feature = "std")] extern crate std;` — lib.rs:12-13
- `extern crate alloc;` — lib.rs:15
- no other `extern crate` sites; module decls at lib.rs:19-20
  (deflate, pack_write); re-exports lib.rs:22.

Non-lib alloc/std usage (non-test code only):

- `use alloc::...` sites: deflate.rs:19-20, dag.rs:110, pack_write.rs:10-11,
  inflate.rs:9-10.
- Test-only std sites (inside `#[cfg(test)]` or std-gated): deflate.rs:572-582,
  dag.rs:1248-1250, inflate.rs:453-464, 586-642, idx_read.rs:224,
  ladder.rs:156, pack_write.rs:548-549, 719, pack.rs:39.
- std-gated non-test: `impl std::error::Error for InflateError {}` —
  inflate.rs:103-104 (`#[cfg(feature = "std")]`).
- UNGATED defect: `std::thread_local!` at ladder.rs:31 (see section 2).

## (4) LOC per src file (wc -l)

| File | LOC |
|---|---|
| dag.rs | 1893 |
| deflate.rs | 716 |
| pack_write.rs | 743 |
| inflate.rs | 679 |
| idx_read.rs | 521 |
| ladder.rs | 287 |
| pack.rs | 124 |
| lib.rs | 133 |
| **Total** | **5096** |

## Standing

ALIVE for the compile/dep findings (commands+exits above, this
session). Footprint numbers remain UNVERIFIED/TARGET —
[NOT-MEASURABLE: no RISC-V llc backend in pinned toolchain].

## Divergences vs lane doc

- Lane doc said "verify whether the single error line is still present
  (may have been fixed)". It is present but cfg-gated
  (lib.rs:12-13); the m2 census over-read it. The genuinely ungated
  no_std blocker is `std::thread_local!` (ladder.rs:31), which only
  fails debug-profile builds.
- m2 census recorded cgap-integration at 3 errors; out of lane scope
  (not re-run here), UNVERIFIED as-of-now.
