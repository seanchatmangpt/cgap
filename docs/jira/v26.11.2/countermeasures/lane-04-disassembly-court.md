# Lane AE4 — disassembly-invariants court (countermeasures wave)

Lane AE4, 2026-11-02. Subject: `scripts/verify_disassembly_invariants.py`
(new) + this receipt. No git writes; shared tree untouched except the two
owned files. /tmp build scratch (deleted at lane end).

## Design

`scripts/verify_disassembly_invariants.py` (stdlib-only python3), four
arms:

- **A. census (method of record)** — EX3 emit-asm build of
  `crates/cgap-branchless` for `riscv32imc-unknown-none-elf`
  (cargo +nightly, `RUSTFLAGS="--emit=asm -C opt-level=3 -C
  link-dead-code=yes -C debuginfo=0"`,
  `CARGO_TARGET_DIR=/tmp/ae4-scratch/census-target`), per-symbol census of
  the emitted `.s`: region = symbol-label-bounded, opcode blacklist =
  beq/bne/blt/bge/bltu/bgeu + z-forms beqz/bnez + RVC c.beqz/c.bnez/c.j/
  c.jal/c.jr.
- **B. linked witness** — no_std shim staticlib (panic handler +
  GlobalAlloc shim + `pub use cgap_branchless::*`) linked for
  riscv32imc; `llvm-objdump -d` (path resolved via `rustc --print
  sysroot`) over the archive; surviving allowlist symbols re-scanned in
  the linked object stream.
- **C. mutant court** — if-injection (`if r > 0 { r } else { true_val }`,
  the B6/REP1 pattern) into `select::cmov_u64` on a /tmp sandbox copy of
  the crate; census of the mutant emit MUST flag cmov_u64 — a court that
  cannot flip is FAIL.
- **D. self-check** — branchy reference `naive_if_cmov` compiled via
  `rustc --emit=obj` for riscv32imc, disassembled with llvm-objdump -d,
  MUST be flagged (bnez witnessed).

Exit ABI: 0 clean / 1 forbidden-branch or vacuous court / 2
subject-missing / 3 loud skip.

## Method grounding (why the census is on the .s emit)

Object/rlib emission PRUNES unreferenced pub fns of a dependency rlib
(witnessed this session: shim staticlib archive carried only 5 of 19
cgap_branchless symbols; rlib members are bitcode, llvm-objdump refuses
them). `--emit=asm` preserves all 19 allowlist fns (EX3:78-83). The
census therefore runs on the emitted assembly; llvm-objdump -d is
exercised on the linked staticlib witness (arm B, 145,280 disassembly
lines) and on the self-check object (arm D).

## Residual allowlist (provenance — never silent)

RV32 ISA ceilings, cited to
`docs/jira/v26.11.2/exhaustive/lane-03-branchless-complete.md` rows; each
run emits explicit ALLOWED-RESIDUAL rows:

| fn | observed | cite |
|---|---|---|
| select::select_slice | 2 branches (beqz len-guard + bnez cond byte) | lane-03 row
  select::select_slice |
| select::cmov_u64 | 1 bnez (RV32 has no cmov/Zicond) | lane-03 row select::cmov REF1/lane-03 |
| eytzinger::eytzinger_restore | 1 beqz (index/remainder guard) | lane-03 rows
  eytzinger::eytzinger_restore |

REFERENCE-DRIVER rows (EX3 classification): checked_uniform_select,
penstock_prefilter8, swar_all_zero_vl, eytzinger_search,
to_eytzinger_layout — loop back-edges, cited to lane-03 rows.

Honesty rows (verdict-neutral, loud): FOREIGN-INSTANTIATION (std
alloc/core monomorphizations instantiated by cgap-branchless — p2-asm
receipt out-of-region class) and OWN-CRATE-PRIVATE (eytzinger padded_at
bgeu x2 — EX3 precedent: censused, not a pub-fn row).

## Scenario matrix (all ran this session)

| scenario | command | exit | outcome |
|---|---|---|---|
| clean run | `python3 scripts/verify_disassembly_invariants.py` | 0 | 19/19 fns censused; 13 PASS
  + 3 ALLOWED-RESIDUAL + 5 REFERENCE-DRIVER rows; witness, mutant, self-check held |
| mutant flip | arm C (if-injection into cmov_u64, /tmp sandbox) | 0 (arm flipped) | `[PASS]
  anti-vacuity M-AE4-1: if-injection into cmov_u64 flipped census to branchy` |
| branchy self-check | arm D (naive_if_cmov, rustc --emit=obj + llvm-branchy objdump -d) | 0 (arm
  flipped) | `[PASS] self-check naive_if_cmov flagged BRANCHY: ['bnez']` |
| forbidden-branch arm | classification (exit 1) | verified via mutant flip | classification code
  path exercised by arm C flip |
| subject missing | classification (exit 2) | not exercised | UNVERIFIED (path-existence check
  only, code-read) |
| loud skip | classification (exit 3) | not exercised | UNVERIFIED (code-read only) |

## Divergences (dispatch vs reality, doc yields recorded)

1. "w6-b6/EX3" — no w6-b6 file exists in the repo; EX3
   (`docs/jira/v26.11.2/exhaustive/lane-03-branchless-complete.md`) is the
   method of record and is reused verbatim.
2. Dispatch said run llvm-objdump -d as the census surface; the working
   census surface is the emitted .s (llvm-objdump refuses the rlib's
   bitcode members; objects prune unreferenced pub fns). llvm-objdump -d
   IS run — on the linked staticlib witness and the self-check object.
3. "Branchless set" rows with loop back-edges (the 5 reference-driver
   fns) are classified REFERENCE-DRIVER with lane-03 cites rather than
   forced into the forbidden-branch verdict — forcing them would make the
   court fail on the clean tree, i.e. vacuous-by-inversion.
4. Shared-tree edit disclosure (SLA-unblock): none.
