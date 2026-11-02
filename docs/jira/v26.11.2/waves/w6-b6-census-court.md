# w6-b6-census-court — B6 lane receipt

Lane: B6 (assembly census court, cgap-branchless). Date: 2026-11-02.
Repo: /Users/sac/cgap, branch main.

## Files written (ownership)

- `crates/cgap-branchless/tests/branchless_census.rs` (new, ~370 lines,
  standalone binary with own `fn main`, no test harness).
- `docs/jira/v26.11.2/waves/w6-b6-census-court.md` (this receipt).

No other file touched. `git status --porcelain -- crates/cgap-branchless`
shows the crate dir still untracked (`??`) — coordinator owns git.

## Court design

- Builds the crate's modules via a generated wrapper crate
  (`/tmp/b6-target/wrapper_host.rs`, `wrapper_riscv.rs`) with nightly
  rustc: `-C opt-level=3 -C llvm-args=--inline-threshold=0
  -C link-dead-code=yes --emit=obj` (host) / plus
  `--target riscv32imc-unknown-none-elf`, `#![no_std]` (riscv).
  `link-dead-code=yes` is load-bearing: unused `#[inline]` fns are
  otherwise dropped from the object (witnessed: 19/19 MISSING without it).
- Disassembles with the toolchain's real llvm-objdump, splits per-symbol
  blocks, extracts mnemonics, classifies flow-control per arch:
  - aarch64: `b bl br blr cbz cbnz tbz tbnz b.* blraa* brasl*`
  - riscv32: `beq bne blt bge bltu bgeu jal jalr`
  - x86_64: `j* call callq` (classifier present; host is aarch64, x86
    arm not exercised this run)
- Exit 0 only when every censused fn-target cell is PASS AND the
  deliberately-branchy reference `naive_if_cmov` is flagged BRANCHY.
  BRANCHY or MISSING on any primitive → exit 1. Skips are loud
  (`[SKIP] riscv32imc: module select.rs needs std/alloc — excluded`).

## Commands + exits (this session)

| command | exit |
|---|---|
| `rustup run nightly cargo build -p cgap-branchless` (B1 poll) | 0 |
| `rustup run nightly rustc --edition 2021 -C opt-level=1 tests/branchless_census.rs -o /tmp/b6-census-bin` | 0 |
| `/tmp/b6-census-bin` (final, full 19-cell census) | 1 (honest: 3 real BRANCHY findings vs pre-branchless bodies) |
| Sandbox GREEN witness: pred_inc branchless in /tmp copy, census restricted, `/tmp/b6-census-green` | 0 |
| Sandbox VACUOUS-flip witness: branchy ref neutralized to mask arithmetic in /tmp copy, `/tmp/b6-census-vacuous` | 1 with `[VACUOUS] branchy reference was NOT flagged — court cannot flip` |

Shared tree untouched by any mutation; flip witnesses ran entirely in
`/tmp/b6-sandbox/cgap-branchless` (copy; no restore needed, no mtime
hazard — nothing in the shared tree was rebuilt from sandbox bytes).

## Per-fn asm census (as of this run, host aarch64 + riscv32imc)

| target | fn | verdict |
|---|---|---|
| host | select::cmov_u64 | PASS |
| host | select::select_slice | BRANCHY (12: b.eq x2, cbz x3, b.ne x2, b.hs, b, b.lo, tbz) |
| host | select::pred_inc | BRANCHY (tbz) |
| host | arithmetic::saturated_subs_p28 | PASS |
| host | arithmetic::poison_mask | PASS |
| host | swar::swar_eq_u64 | PASS |
| host | swar::swar_has_zero_byte | PASS |
| host | swar::swar_range_mask | PASS |
| host | eytzinger::eytzinger_next | PASS |
| host | eytzinger::eytzinger_restore | PASS |
| host | radix::radix_bucket_12 | PASS |
| riscv32imc | select::* (3 fns) | [SKIP] module needs std/alloc |
| riscv32imc | arithmetic::saturated_subs_p28 | BRANCHY (beq) |
| riscv32imc | arithmetic::poison_mask | PASS |
| riscv32imc | swar::* (3 fns) | PASS |
| riscv32imc | eytzinger::* (2 fns) | PASS |
| riscv32imc | radix::radix_bucket_12 | PASS |

selfcheck: `naive_if_cmov BRANCHY 3x ["tbz", "b", "b"]` — court
non-vacuous. Court verdict against the current (pre-B2-branchless)
tree: exit 1, honestly reporting 3 BRANCHY cells. It will flip to
exit 0 when B2 lands branchless `select_slice` / `pred_inc` /
`saturated_subs_p28` (riscv).

## Divergences from the lane doc

1. **harness=false standalone binary, not a cargo test.** The doc
   allowed "a test (or harness=false test binary)"; the file is written
   with its own `fn main` and runs standalone via
   `rustc ... -o /tmp/b6-census-bin && /tmp/b6-census-bin`. Under
   `cargo test -p cgap-branchless` it would need a `[[test]] harness =
   false` stanza in the crate Cargo.toml (B1's file, not mine).
2. **riscv detection is toolchain-relative.** `rustup target list
   --installed` resolves against the toolchain active in cwd; run from
   /tmp the sandbox witness saw no riscv std and printed
   `[SKIP] riscv32imc target not installed`. Run from the repo (nightly
   pinned) it detects riscv32imc and runs the 8-cell riscv census.
3. **`--crate-type=lib` wrapper build** (not cargo), because the court
   must census exact per-fn symbols; CARGO_TARGET_DIR not used — the
   court's build root is `/tmp/b6-target` (lease, deleted below).
4. Early B3 in-flight swar.rs break was ridden out with a bounded
   poll (~8 min deadline); B3 landed and compiled; no unblock edit
   applied to any lane-mate file.

## Anti-vacuity record

Both directions witnessed on /tmp sandbox copies: green path exits 0
only when primitives pass, and neutralizing the branchy reference
makes the court refuse with `[VACUOUS]` exit 1. A court that passes
while the branchy fn is present does not exist in this design.

## Build-root lease cleanup

Lease: `/tmp/b6-target` (+ `/tmp/b6-sandbox`, `/tmp/b6-census-*`).
Cleanup: `python3 -c "import shutil; shutil.rmtree('/tmp/b6-target', True)"`
at lane end (same for the sandbox and binaries).
