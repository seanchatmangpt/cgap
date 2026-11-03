# RED-11 — RISC-V 32IMC Single-Core Concurrency Poisoning (ref_store seal counter)

- Lane: RED-11, read-only over `crates/`, file written: this receipt only.
- Subject: `crates/cgap-git/src/ref_store.rs` (width-gated seal counter, DOD-04 /
  commit a0431b70 "ref_store width-gated atomics — riscv32imc no_std fix").
- Standing: **ALIVE** (all commands below run this session, exits captured).
- Verdict: **BREACH-CONFIRMED** — reentrancy-unsafe as written on riscv32imc;
  breach is a preempted check-then-act window, executable witness run on the
  real code.

## 1. Which arm actually fires

The cfg ladder (`crates/cgap-git/src/ref_store.rs:113-167`) selects **impl32**
(native `AtomicU32`) on riscv32imc — the `implcell` arm is dead on both tested
targets:

- `cfg(target_has_atomic = "32")` compiles TRUE on riscv32imc (pinned nightly,
  `riscv-toolchain.toml`-pinned `nightly` channel): /tmp probe crate
  `#[cfg(all(target_has_atomic = "32", not(target_has_atomic = "64")))] pub const IMPL32_ARM: () = ();`
  → `cargo build --target riscv32imc-unknown-none-elf` **exit 0**.
- Compile-time width witness against the REAL file: /tmp crate including
  `ref_store.rs` via `#[path]` with `const _: () = assert!(SEAL_COUNTER_WIDTH == u32::MAX as u64);`
  → **exit 0** (impl32 ceiling = u32::MAX; `usize::MAX == u32::MAX` on rv32,
  so that assert alone is not discriminating — the cfg probe is).
- `rustc --print cfg` on riscv32imc lists only `target_has_atomic_load_store`
  (+ primitive_alignment) keys while the aarch64 host lists both key
  families — the cfg ladder predates this cfg split; the ladder still
  resolves impl32 because the legacy `target_has_atomic` key is accepted.
  Noted as an upstream-cfg-migration hazard, not a current defect.

## 2. Codegen smoking gun (riscv32imc)

Built `cgap-git` `--target riscv32imc-unknown-none-elf --no-default-features`
→ **exit 0** (note: default-features build FAILS on riscv32imc — `std` feature
on; exit 101, E0463. The no_std inventory's "git: clean no_std" claim holds
only with `--no-default-features`).

Disassembly of the built rlib (`llvm-nm`/`llvm-objdump`, nightly toolchain bin
dir), 130,125 lines:

- **Zero** `lr.w` / `sc.w` / `amoswap` / `amoadd` instructions anywhere —
  no A extension, no native RMW exists on IMC.
- `Counter::fetch_add` compiles to an out-of-line libcall via `auipc`+`jalr`
  relocation (an `__atomic_fetch_add_4`-class runtime call receiving
  ptr/expected/order args at `+0x8` and `+0x2a`). Disassembly excerpt at
  /tmp/red11-disasm.txt:90564-90598.

Breach mechanism: on bare-metal rv32imc, that libcall must be supplied by
firmware; any naive `lw/add/sw` implementation is a torn read-modify-write
that a preempting ISR can interleave (duplicate leases: two stamps both
return the same lease, breaking the strict-monotonic lease uniqueness that
`RefStore::cas`'s StaleLease guard at `ref_store.rs:486-491` depends on).
Even a perfect interrupt-disabling libcall does NOT close the breach, because
the ceiling check (`:295-301`) and the RMW (`:306`) are separated by the slot
copy + chain update (`:302-305`) — the check is not part of the RMW.

## 3. Executable witness (real code, /tmp sandbox, disclosed)

Sandbox copy of the real `ref_store.rs` + appended `#[cfg(test)]
mod red11_reentrancy` (disclosed court-harness mutation; shared tree
untouched — `git status` clean except this receipt dir):

- `red11_toc_teau_ceiling_overspend`: context A reads `counter == 4`
  (`:295`), passes the ceiling check (`cur < bound`, ceiling 5); ISR performs
  the counter/chain effects of a full stamp through the REAL `Counter`
  methods (all `&self` — legal aliasing, proving `&mut self` on
  `SealRegister::stamp` gates threads, not interrupts); context A resumes at
  `:306` and mints **lease 6 > ceiling 5**. Witness PASSES, exit 0.
- `red11_control_no_interrupt_no_overspend`: identical flow, no interrupt →
  lease == 5 == ceiling, counter never exceeds, and the next stamp is
  correctly `CounterExhausted`-refused. Control PASSES, exit 0.

Both green together = non-vacuous: the corruption is caused by the preempted
window, not by trivially-broken ceiling logic. Flip law satisfied: remove the
ISR steps and the overspend court goes red.

On the actual impl32/rv32imc target with `ceiling >= WIDTH`, the same
interleaving wraps the counter to **0** (u32 fetch_add at WIDTH returns WIDTH
→ counter folds to 0), minting lease 0 after lease u32::MAX — a replayable
authority replay exactly contradicting the module's own invariant claim
("`the counter exhausts, it never silently wraps`", `ref_store.rs:33-39`,
`:110-112`; "the counter never exceeds this bound", `:169-171`).

## 4. Coverage gap (fence)

`tests/ref_store_court.rs` (5 tests, all green this session, exit 0: c1 CAS
success, c2 stale lease, c3 torn write, c4 monotonic no-wrap, c5 refusals)
has **zero** reentrancy/preemption coverage — every test assumes exclusive
`&mut` for the whole stamp. Additionally `impl16` and `implcell` arms are
never compiled by any court (host = impl64, riscv32imc = impl32): if an
implcell target ever runs this file, its `get/set` RMW path
(`ref_store.rs:196-199`, `:219-229`) is a plain non-atomic read-modify-write
with no court having ever executed it (UNVERIFIED arm).

## 5. Verdict

**BREACH-CONFIRMED** for the single-core interrupt-reentrancy threat model:
the DOD-04 width-gate preserves monotonicity only under the single-context
assumption its tests bake in. Repair shape (for the owner lane, not this
lane): move the ceiling enforcement INSIDE the RMW (`fetch_update` with a
`c < bound` guard, refuse on None) and supply/require an interrupt-safe
`__atomic_*` runtime on rv32imc (or gate the register as reentrancy-unsafe
until one exists).

## Commands (all this session)

| cmd | exit |
|---|---|
| `cargo build -p cgap-git --target riscv32imc-unknown-none-elf` (default features) | 101 (E0463 std) |
| `cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features` | 0 |
| cfg probe `target_has_atomic="32"` build on riscv32imc | 0 |
| width witness `assert!(SEAL_COUNTER_WIDTH == u32::MAX as u64)` on riscv32imc | 0 |
| `cargo test -p cgap-git --test ref_store_court` (host baseline) | 0 (5 passed) |
| `llvm-objdump -d *.rcgu.o` (rv32imc rlib) + grep AMO count | 0 AMO |
| `cargo test --manifest-path /tmp/red11-wit/Cargo.toml` (witness + control) | 0 (2 passed) |
| `git status --porcelain` (shared tree clean) | 0 |

Lease cleanup: /tmp/red11-target and /tmp/red11-probe deleted at lane end
(python shutil.rmtree). Retained for the coordinator's replay window:
/tmp/red11-arm, /tmp/red11-cfg, /tmp/red11-wit (witness crate),
/tmp/red11-extract, /tmp/red11-disasm.txt, plus build/test logs
/tmp/red11-*.log.

## ERRATA (appended 2026-11-02, lane F-01 — errata-never-rewrite; §1 above is superseded)

§1's premise "the ladder still resolves impl32 because the legacy
`target_has_atomic` key is accepted... not a current defect" (:13-29) is
SUPERSEDED. The hazard fired: ADV-11 witnessed rv32imc silently selecting
implcell with pass-through `critical` (`redteam/corrosion/adv-11-ahdclt-concurrency-hole.md`,
SPOILER-CONFIRMED, elevated to MATERIALIZED-ON-TARGET). Repair landed:
`lanes/lane-r-adv11fix-implcell-slide.md` — the ladder now refuses LOUD on
rv32imc (typed compile_error, exit 101, naming `implcell_critical_provided`)
until firmware binds the critical section via
`RUSTFLAGS="--cfg implcell_critical_provided"` (opt-in build exit 0).
Independently re-verified by lane F-01 this session: no-opt-in rv32imc
build exit 101 with the typed refusal; opt-in build exit 0; host build
exit 0; `ref_store_width_arms_court` 8/8 exit 0. §2's impl32
`__atomic_*` libcall codegen and §3's rv32imc wrap-to-0 shape describe
the PRE-slide arm; the reentrancy-breach residual for rv32imc now lives
in the implcell arm under the loud-until-bound contract (unbound =
refused, bound = firmware-owned critical section), and the impl16/
implcell execution-certification leg stays REGISTERED.
