---
title: "forensic-lane-02"
lane: FA-2
status: ALIVE
date: 2026-11-02
---

# Forensic lane FA-2 — hostile audit of the cgap-branchless 300.5M dec/s claim

Subject: `docs/jira/v26.11.2/mega-bench/lane-02-lease.md` +
`crates/cgap-branchless/src/arithmetic.rs`. Read-only on the shared tree;
probes in `/tmp/fa2-probe`, asm in `/tmp/fa2-asm`, build root
`/tmp/fa2-target` (deleted at lane end). No `Process.sleep` anywhere.

## Suspicion 1 — "dead-code-eliminated loop fiction" → REFUTED (with a measured caveat)

The mb2-probe source still on disk (`/tmp/mb2-probe/src/main.rs`) confirms:
inputs `black_box`ed, both tuple outputs folded into `acc` via
`wrapping_add`, `acc` returned and XORed into a checksum PRINTED per run
(`0xd4416e9a92f3f` subs, `0xffffffffffff0bdc` poison). I rebuilt and
re-ran it: both checksums reproduce BYTE-EXACT this session. The sink is
a real observable, not returned-into-the-void.

DCE-fraction experiment (min-of-5, 1M iters, release opt-3, aarch64):

| variant | ns/op | M-ops/s |
|---|---|---|
| subs, both outputs consumed into checksum | 4.06 | 246.3 |
| subs, tripwire dropped but `new` chains state | 4.06 | 246.2 |
| subs, BOTH outputs discarded (black_box inputs only) | 0.78 | 1289.9 |
| poison, consumed | 0.52 | 1939.6 |
| poison, discarded | 0.36 | 2807.0 |

Command (cwd /tmp/fa2-probe): `CARGO_TARGET_DIR=/tmp/fa2-target cargo run --release` → exit 0.

Reading:
- The consumed-vs-discard delta is 3.28 ns of 4.06 ns (~81%): if NO output
  were consumed, LLVM eliminates the whole kernel body (remaining 0.78 ns
  is black_box + loop overhead — verified, the discard loop contains no
  arithmetic). So the DCE hazard is real and large,
  **but the mb2 harness actually consumed the outputs**, proven by the
  byte-exact checksum replay. The 3.33 ns figure was real work, not
  dead-code fiction. Suspicion REFUTED for the lease as written.
- Caveat (honest re-measurement): this session the same consumed harness
  measures 4.06 ns (246.1 M-ops/s), not 3.33 ns (300.5 M-ops/s). The
  ~18% gap is host frequency/noise (concurrent lanes), not a code change —
  source is byte-identical (checksum witness identical). Still >2.4x over
  the 100M target; the receipt's 300.5 is a single-session number and
  should be cited as such, not as a stable constant.
- poison_mask consumed-vs-discard delta is only ~0.16 ns: the kernel is so
  small that even fully eliminated it barely moves; its 0.50 ns figure is
  also genuine work.

## Suspicion 2 — "return tuple folds to nothing" → REFUTED on both targets

Rebuilt asm this session with `-C link-dead-code=yes` (census method).

### host aarch64 (rustc -O --emit asm, /tmp/fa2-asm/subs_aarch64.s, verbatim)

```asm
__RNvCs8ll05nGLuc7_4wrap6k_subs:
    bic  x8, x1, x0
    orn  x9, x1, x0
    sub  x10, x0, x1
    and  x9, x9, x10
    orr  x8, x9, x8
    asr  x1, x8, #63
    cmp  x8, #0
    csel x0, xzr, x10, mi
    ret
__RNvCs8ll05nGLuc7_4wrap8k_poison:
    cmp   x0, #0
    csetm x0, eq
    ret
```

Tuple survives in x0/x1 (contract return registers). Branchless. LLVM
contracted the pure-u64 borrow chain differently from the lease receipt
(lease quoted `subs/ngc/bic`; today it is `bic/orn/.../csel`) — same
branchless semantics, contraction is LLVM-version dependent. Not a defect;
a receipt quoting exact mnemonics should pin the rustc version.

### riscv32imc (rustup run nightly rustc 1.101.0-nightly (12433c307 2026-10-24), verbatim)

```asm
_RNvNtCsggheNpNv8AU_15cgap_branchless10arithmetic18saturated_subs_p28B3_:
    sltu a5, a1, a3
    sub  a6, a2, a4
    not  a2, a2
    sub  a5, a6, a5
    or   a6, a4, a2
    and  a2, a2, a4
    and  a4, a5, a6
    or   a2, a2, a4
    sub  a1, a1, a3
    srli a3, a2, 31
    srai a2, a2, 31
    addi a3, a3, -1
    and  a5, a5, a3
    and  a1, a1, a3
    sw   a1, 0(a0)
    sw   a5, 4(a0)
    sw   a2, 8(a0)
    sw   a2, 12(a0)
    ret
```

This is the IN-TREE crate (built via `cargo rustc --lib --release --
--emit asm -C link-dead-code=yes`, target riscv32imc, output at
/tmp/fa2-target/riscv32imc-unknown-none-elf/release/build/cgap-branchless/
63649755d8828de1/out/cgap_branchless-*.s). The (u64,u64) tuple fully
materializes through the sret pointer: four `sw` stores, all four words
live. **Zero branch instructions.**

**FINDING (receipt divergence): the w6-b6 census table
(`w6-b6-census-court.md`, per-fn census, riscv32imc row) records
`arithmetic::saturated_subs_p28 | BRANCHY (beq)` and says the court will
"flip to exit 0 when B2 lands". The current tree HAS the B2 branchless
borrow form landed (arithmetic.rs:21-27, pure-u64 kill/propagate, the
very body disassembled above), and on riscv32imc it is branchless. The
w6-b6 table is STALE for that one cell and still misleads a reader into
thinking riscv codegen is branchy today. The lease receipt quoted the
stale w6-b6 claim uncritically. Neither receipt flagged the staleness.

## Suspicion 3 — "tripwire returned into the void" → CONFIRMED (untested scaffolding)

Constant-underflow probe (/tmp/fa2-probe/src/bin/underflow.rs, exit 0):
`saturated_subs_p28(0, 100)` returns `(0x0, 0xffffffffffffffff)` — a mask,
NOT a halt. Zero halts across a 4096-case differential sweep under the
corpus seed 0xA20C_0FFE_E5A7_E000 with the SplitMix64 multiplier
0xBF58_476D_1CE4_E5B9 as DERIVE constant (seeds not conflated). The kernel
is a pure function; any halt semantics live entirely in a caller.

Caller census (grep over crates/, this session, exact):
callers of `saturated_subs_p28` / `poison_mask` exist ONLY inside
cgap-branchless itself (src/arithmetic.rs tests + tests/branchless_census.rs).
**Zero production call sites in cgap-p28, cgap-gate, cgap-crypto, cgap-ffi,
or any other crate.**

Receipt loudness check:
- w6-b8-p28-consume.md is LOUD and honest: verdict PARTIAL, "zero eligible
  swap sites", dep NOT added, divergence rows D-B8-1/D-B8-2 spelled out.
- w6-b7-gate-consume.md is loud too: dep-only landing, penstock.rs
  untouched, documents the unused-dep warning as expected.
- BUT the mega-bench lease (lane-02-lease.md) does NOT mention the zero-
  caller status at all. It frames the kernels as the "decrement kernel"
  behind the P28 lease story while no production code decrements anything
  with them. The "fail-closed entanglement" — underflow tripwire → staged
  constant → abort — has NO existing caller chain: nothing in the tree
  maps `tripwire == u64::MAX` to `spoliation_halt()`. `spoliation_halt`
  itself has exactly ONE non-test call-site reference
  (p28_invariant_courts.rs:350, a test that expects the abort). So the
  fail-closed mapping is asserted in tests of the trap machinery but the
  TRIPWIRE-to-halt edge is untested scaffolding. Verdict: the lease's
  entanglement narrative outran the wiring; w6-b8 knew and said so, the
  lease did not.

## Suspicion 4 — 0x5032_38FF: const-only vs runtime-consumed → REFUTED (it is runtime-returned and court-tested)

The three sites named in the dispatch:
- `crates/cgap-p28/src/trap.rs:36` — `ERR_P28_SPOLIATION` const, ABI-frozen
  by a `const _:` assert binding it to LICENSE_MAGIC (trap.rs:42-48), plus
  `STAGED_DISCRIMINATOR` AtomicU32 and `spoliation_halt()` (stage-then-
  abort). Design intent per doc comment: crash-handler poll value.
- `crates/cgap-gate/src/penstock.rs:319` — `CGAP_ERR_P28_SPOLIATION` const.
  NOT const-only: penstock RETURNS it at runtime from ≥7 validation
  branches (penstock.rs:574, 578, 583, 588, 592, 599, ... group-3 P28 chain
  checks). Consumed by tests: lever_courts.rs:72/90/150 assert
  CGAP_OK → SPOLIATION flips under real byte mutations.
- `crates/cgap-ffi/src/sovereign.rs:83` — `CGAP_ERR_P28_SPOLIATION: CgapStatus`
  mirror; part of the A19 register cross-check (penstock.rs ~1668-1680
  const-asserts the gate/ffi registers match). Value-level mirror,
  not an independent assertion site.

Runtime witness (this session): `cargo test -p cgap-gate --test lever_courts`
→ exit 0, 3 passed (l1_attestation_byte_flip_escalates_to_spoliation,
l1_ceiling_law_flip_is_the_named_mutant, l1_prefilter_escalation_policy_court).
A one-bit attestation mutation REALLY escalates to 0x5032_38FF at runtime.
The spoliation halt machinery (trap.rs) is a SEPARATE, process-abort path
that no production caller invokes — but the status-code path is live.

## Verdicts

| suspicion | verdict |
|---|---|
| 1. 300.5M dec/s is DCE fiction | REFUTED — outputs consumed, checksum reproduces byte-exact; BUT measured DCE hazard is real (~81% of time vanishes if outputs discarded), and this-session re-run gives 246.1 M/s not 300.5 (noise; cite as single-session) |
| 2. return tuple folds to nothing | REFUTED — tuple survives (x0/x1 on aarch64; 4x sw through sret on riscv32imc); branchless on BOTH targets |
| 3. tripwire into the void | CONFIRMED — zero production callers; underflow returns a mask, halts nothing; tripwire→spoliation_halt edge does not exist in code; w6-b8 was loud about it, the mega-bench lease was silent |
| 4. 0x5032_38FF const-only | REFUTED — penstock returns it at runtime from ≥7 sites, flip proven by lever courts exit 0 this session; trap.rs halt path is separate and uncalled in production |

## Bonus finding — stale census cell

w6-b6-census-court.md records riscv32imc `saturated_subs_p28 = BRANCHY (beq)`
against the pre-B2 tree; the current tree compiles the same kernel
branchless on riscv32imc (asm above). The B2 fix landed but the census
receipt was never re-run/annotated; downstream receipts (lane-02-lease)
still cite the BRANCHY claim as current. Register of truth is stale on
this cell.

## Receipt (commands + exits, this session)

| command (cwd) | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/fa2-target cargo run --release` (/tmp/fa2-probe) | 0 |
| `cargo run --release --bin underflow` (same) | 0 |
| `rustup run nightly rustc -O --emit asm -C link-dead-code=yes --target riscv32imc-unknown-none-elf wrap.rs` (/tmp/fa2-asm) | 0 |
| `rustc -O --emit asm -C link-dead-code=yes --target aarch64-apple-darwin wrap.rs` | 0 |
| `CARGO_TARGET_DIR=/tmp/fa2-target rustup run nightly cargo rustc --lib --release -- --emit asm -C link-dead-code=yes` (manifest-path crates/cgap-branchless) | 0 |
| `cargo test -p cgap-gate --test lever_courts` (/Users/sac/cgap) | 0 |

Shared tree: only this report file written. `rustup target add
riscv32imc-unknown-none-elf` was run but reported "already up to date"
(the target lives in the nightly toolchain; stable lacks it — dispatchers
of riscv builds must `rustup run nightly`).
