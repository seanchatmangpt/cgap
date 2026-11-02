# Lane C11 - branchless census witness (post-REP1)

Date: 2026-11-02. Subject: /Users/sac/cgap @ main (9a730320), crates/cgap-branchless.
Command: CARGO_TARGET_DIR=/tmp/c11-target cargo test -p cgap-branchless --test branchless_census -- --nocapture
Exit 0. 3 passed / 0 failed / 0 measured. Warnings only (deprecated core::u64, dead const LO, unused f).

## Ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| census | cargo test -p cgap-branchless --test branchless_census -- --nocapture (/Users/sac/cgap) | 0 | 3/3 ok |
| host verdict | --nocapture output | - | "census host-aarch64: 10 HOT cells 0-branch, 3 RESIDUAL cells, refs labeled" |
| riscv verdict | --nocapture output | - | "census riscv32imc: 10 HOT cells 0-branch, 3 RESIDUAL cells, refs labeled" |
| asm re-witness | llvm-objdump -d /tmp/rep1-census/obj_host.o obj_riscv.o | - | bodies below |
## Census verdict output (verbatim, --nocapture)

- [PASS] branchy self-check naive_if_cmov: flagged BRANCHY (3 flow instrs) (both targets)
- [PASS-RESIDUAL] host-aarch64 cmov_u64: 0/0 documented branch(es)  [csel: HOT]
- [PASS-RESIDUAL] host-aarch64 select_slice: 1/1 documented branch(es)
- [PASS-RESIDUAL] host-aarch64 eytzinger_restore: 0/0 documented branch(es)
- [PASS-RESIDUAL] riscv32imc cmov_u64: 1/1 documented branch(es)
- [PASS-RESIDUAL] riscv32imc select_slice: 2/2 documented branch(es)
- [PASS-RESIDUAL] riscv32imc eytzinger_restore: 1/1 documented branch(es)
- [SKIP-REFERENCE] x6 rows: checked_uniform_select (cold-path refusal by design);
  eytzinger_search / to_eytzinger_layout (reference drivers) — loud skip, presence asserted
- [PASS] sandbox unsafe mask form select_slice_mask_form: 0 branches
  (forbid(unsafe_code) is the sole in-tree blocker) (both targets)
- [MUTANT] select_slice -> BRANCHY 11x ["b.eq","b.hs","b","b.hs","b","b.ne",
  "b.eq","cbz","b.ne","b.eq","b.lo"] (both targets)
- [PASS] anti-vacuity M-REP1-1: mutant flipped to BRANCHY (>1 branch) (both targets)

## Honest final grid (16/16 coverage, both targets)

| Cell | host aarch64 | riscv32imc |
|---|---|---|
| cmov_u64 | HOT 0 (csel) | RESIDUAL 1/1 (bnez) |
| pred_inc | HOT 0 | HOT 0 |
| select_slice | RESIDUAL 1/1 (cbz) | RESIDUAL 2/2 (beqz + bnez) |
| checked_uniform_select | REFERENCE (presence asserted) | REFERENCE (presence asserted) |
| saturated_subs_p28 | HOT 0 | HOT 0 |
| poison_mask | HOT 0 | HOT 0 |
| swar_eq_u64 | HOT 0 | HOT 0 |
| swar_has_zero_byte | HOT 0 | HOT 0 |
| swar_range_mask | HOT 0 | HOT 0 |
| swar_lt | HOT 0 | HOT 0 |
| swar_splat | HOT 0 | HOT 0 |
| eytzinger_next | HOT 0 | HOT 0 |
| eytzinger_restore | HOT 0 | RESIDUAL 1/1 (beqz) |
| eytzinger_search | REFERENCE | REFERENCE |
| to_eytzinger_layout | REFERENCE | REFERENCE |
| radix_bucket_12 | HOT 0 | HOT 0 |

Per-target verdict: 10 HOT cells 0-branch, 3 RESIDUAL, refs labeled (both targets).
## Residual table (ISA-ceiling framing)

| Residual | Count | Mechanism | Classification |
|---|---|---|---|
| select_slice riscv32imc | 2 | beqz empty-mask default + 1 bnez 2-way select lowering; rv32imc has no Zicond, ISel lowers every 2-way select to a branch; --disable-cgp-select2branch does not change it (witnessed REP1) | ISA ceiling, not a defect |
| cmov_u64 riscv32imc | 1 | same ISel select-lowering; arithmetic mask form folds back to a select lowered to 1 bnez | ISA ceiling, not a defect |
| select_slice aarch64 | 1 | cbz empty-mask default; branch-free unsafe mask form blocked by crate-level forbid(unsafe_code) (src/lib.rs:16) | documented by-design residual |
| eytzinger_restore riscv32imc | 1 | one beqz variable-shift guard; riscv shifts do not mask the shift amount, LLVM emits the guard around the magic-multiply ctz expansion | ISA ceiling, not a defect |

## Fresh asm witnesses (llvm-objdump, this session)

host aarch64 select_slice (obj_host.o, real symbol) - 1 branch + 2x csel:

    cbz  x1, +0x14
    ldrb w8, [x0]
    cmp  w8, #0x0
    csel x3, x3, x5, ne
    csel x2, x2, x4, ne
    mov  x0, x2
    mov  x1, x3
    ret

riscv32imc select_slice (obj_riscv.o) - 2 branches (beqz empty-mask + ISel-inherent bnez):

    mv   a6, a1 ; mv a1, a3
    beqz a6, +0x12
    lbu  a0, 0(a0)
    bnez a0, +0x12
    mv   a1, a5 ; mv a2, a4
    mv   a0, a2 ; ret

host aarch64 saturated_subs_p28 (RV1-repaired form) - 0 branches, 9 insns:

    bic  x8, x1, x0 ; orn x9, x1, x0 ; sub x10, x0, x1
    and  x9, x10, x9 ; orr x8, x9, x8 ; asr x1, x8, #63
    cmp  x8, #0x0 ; csel x0, xzr, x10, mi
    ret

riscv32imc saturated_subs_p28 - 0 branches, 25 insns (sltu/sub/not/or/and chain,
srli/srai sign broadcast, four sw stores; no branch mnemonics).

Note on the lane-brief figure: the brief said "19-insn"; the observed counts are
9 (host) / 25 (riscv), both 0-branch. The 0-branch property is the census-HOT
claim; the insn counts above are this-session measurements, not the 19 figure.

## Standing

- Census ALIVE this session against the exact subject: command + exit 0 above.
- cargo test exit 0 = census-passed (FA6 blindness fixed: the census is in-test,
  not an external grep).
- Anti-vacuity: both arms flipped (naive_if_cmov BRANCHY self-check + M-REP1-1
  loop-injection mutant, 11x branches) - the census refuses branchy mutants.
- 16/16 grid coverage asserted by the test itself (GRID + ALL_MODULES).
- Residual allowlist: tests/branchless_census.rs:300-312 (allowed()).
