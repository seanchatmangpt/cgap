---
title: "lane-02-lease"
lane: MB-2
status: ALIVE
date: 2026-11-02
---

# Mega-bench lane 2 — cgap-branchless lease arithmetic

Subject: `/Users/sac/cgap/crates/cgap-branchless/src/arithmetic.rs`
(`saturated_subs_p28`, `poison_mask`). Source untouched (read-only lease).
Probe crate `/tmp/mb2-probe` (path dep, deleted after the run; build root
`/tmp/mb2-target` deleted via `shutil.rmtree` — both absent on recheck).

## Method (divan-style honesty)

- Release build, opt-level 3, Apple Silicon (Darwin 25.2.0, aarch64).
- 1,000,000 iterations/loop, 5 runs, min-of-5 reported.
- Machine: `rustc` release codegen; no divan/criterion harness — plain
  `Instant` timing around the loop (timing overhead = 1 call per 1M iters).
- **Elision guard**: every kernel result feeds `acc` (wrapping_add chain);
  the final checksum is printed per run (`0xd4416e9a92f3f` for subs,
  `0xffffffffffff0bdc` for poison). Inputs are `black_box`ed. The
  checksums were observed on stdout, so the work was not dead-code-eliminated.
- Harness branch note: the subs harness re-seeds `c` when it saturates
  (`if c == 0`) — that branch is in the harness, not the kernel. The
  poison harness alternates quota 0..=15, exercising both mask states.

## Measured (this session, command + exit 0)

CARGO_TARGET_DIR=/tmp/mb2-target cargo run --release --manifest-path
/tmp/mb2-probe/Cargo.toml → exit 0:

| kernel | min ns/op | M-ops/s/core | vs 100M target |
|---|---|---|---|
| saturated_subs_p28 | 3.33 | 300.5 | 3.0x over |
| poison_mask | 0.50 | 1987.6 | 19.9x over |

**Success criterion: throughput >= 100M decrements/s/core — MET (measured,
300.5 M/s for the decrement kernel).**

## Branchless witness (asm-structural, not counter-measured)

Compiled a standalone `/tmp/mb2-asm/wrap.rs` (copies of the two kernels,
`#[inline(never)]` wrappers) through rustc -O --emit asm on aarch64-apple-darwin:

```asm
_k_subs:
    subs  x8, x0, x1    ; wrapping diff, flags set
    ngc   x1, xzr       ; borrow-out -> all-ones tripwire (carry-in complement)
    bic   x0, x8, x1    ; saturate: clear diff where tripwire set
    ret
_k_poison:
    cmp   x0, #0
    csetm x0, eq        ; all-ones iff quota == 0
    ret
```

- **No branch instructions in either kernel body.** Matches the w6-b2
  receipt (`docs/jira/v26.11.2/waves/w6-b2-arithmetic.md`: subs+ngc/bic/csetm).
- Honest scoping: this is the **asm-structural** claim only. Misprediction
  counters (INST_RETIRED/BR_MISMATCH class PMCs) are unavailable on this
  host; "0 branch mispredictions" is NOT a measured claim. The counters
  would need perf/KPC instrumentation this lane did not run.
- Two-target story (honest): aarch64 = branchless (above). **riscv32imc =
  BRANCHY** — per the w6-b6 census
  (`docs/jira/v26.11.2/waves/w6-b6-census-court.md`), saturated_subs_p28
  compiles BRANCHY on rv32imc because the target has no conditional-move
  instruction; the branchless-by-construction Rust is not branchless
  codegen on that backend. Do not generalize the aarch64 result.

Note: at -O on aarch64, LLVM contracted the source's explicit u128
borrow chain into the flags-based `subs`+`ngc` pair — same branchless
semantics, fewer instructions than the source-level sbb-chain framing.

## Corrected entanglement semantics (vs mission framing)

The mission framed "per-decrement BLAKE3 fold" into the P28 lease tick.
The actual architecture:

- **P28 lease tick is a monotonic increment** in
  `/Users/sac/cgap/crates/cgap-crypto/src/lease.rs` (`Tick` newtype, §5;
  monotonic via type — no `set_tick`; rewind unrepresentable in the type).
  It does NOT call saturated_subs_p28 per-decrement; decrement-shaped
  saturation lives in the cgap-branchless kernel and its consumers.
- **BLAKE3 entanglement happens at attestation/commit boundaries, not
  per-decrement**: `crates/cgap-p28/src/attestation.rs:271` (parent-chain
  `cgap_blake3::hash(&p.to_bytes())` on the derivation edge),
  `:319` (`origin_authority_digest` = `cgap_blake3::hash(instrument_header)`),
  `:507` (`upstream_receipt` = `cgap_blake3::hash(&parent.to_bytes())`
  in the derivative constructor). One hash per boundary event, O(1) per
  lease lifetime stage — the throughput number above therefore does not
  amortize any per-decrement hash cost; there is none to amortize.

## Tripwire record (design intent, not a measured cycle count)

- `ERR_P28_SPOLIATION = 0x5032_38FF` is const-pinned at
  `/Users/sac/cgap/crates/cgap-p28/src/trap.rs:36` (also mirrored as
  `CGAP_ERR_P28_SPOLIATION` at
  `/Users/sac/cgap/crates/cgap-gate/src/penstock.rs:319` and
  `crates/cgap-ffi/src/sovereign.rs:83`). Fail-closed mapping:
  `spoliation_halt` stages the constant then aborts (trap.rs:57).
- Correction to the mission text: the const lives in
  `crates/cgap-p28/src/trap.rs:36` (cgap-gate has no `trap.rs`; the
  cgap-gate mirror is penstock.rs:319).
- "Halts in <= 3 CPU cycles" is **design intent, not a measurable claim
  on this host** (no PMC access; no cycle-accurate harness). The
  defensible claim is the fail-closed mapping: underflow tripwire ->
  staged constant -> abort, pinned by const-assert (`penstock.rs:1674`).

## Ladder

| Check | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| bench | `CARGO_TARGET_DIR=/tmp/mb2-target cargo run --release --manifest-path /tmp/mb2-probe/Cargo.toml` (/tmp/mb2-probe) | 0 | table above, checksums printed |
| asm witness | `rustc -O --crate-type lib --emit asm /tmp/mb2-asm/wrap.rs` (/tmp) | 0 | _k_subs/_k_poison bodies, 0 branches |
| lease cleanup | `python3 -c shutil.rmtree('/tmp/mb2-target')` | 0 | dir absent on recheck |
