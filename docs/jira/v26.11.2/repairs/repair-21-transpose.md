# Repair-21: Bit-Matrix Transpose Primitives (Lane TS4)

Date: 2026-11-02 · Lane: TS4 (BH2 survival-YES candidate) · Crate:
`cgap-branchless` · Standing: **ALIVE**

## Subject

- `crates/cgap-branchless/src/transpose.rs` (new, harvested)
- `crates/cgap-branchless/src/lib.rs` (module registration only)
- `crates/cgap-branchless/tests/transpose_courts.rs` (new)

## Harvest (cited, never imported)

- `~/bcinr/crates/bcinr-logic/src/algorithms/bit_matrix_transpose_8x8.rs:22`
  — single-word 8x8 delta-swap transpose.
- `~/bcinr/crates/bcinr-const... ` — correct path
  `~/bcinr/crates/bcinr-logic/src/algorithms/bit_matrix_transpose_64x64.rs:24-34`
  — three-stage masks `0x00AA00AA00AA00AA`/shift 7,
  `0x0000CCCC0000CCCC`/shift 14, `0x00000000F0F0F0F0`/shift 28. The
  bcinr "64x64" fn is itself an 8x8-bit transpose over one u64 (the
  masks are byte-periodic); a true 64x64 bit matrix (4096 bits) cannot
  fit in `[u64;8]` (512 bits), so the landed `transpose_64x64` uses the
  honest `[u64;64]` (word = row) layout with a six-pass quadrant
  delta-swap decomposition (32/16/8/4/2/1, masks
  `0x00000000FFFFFFFF` … `0x5555555555555555`), and `transpose_8x8`
  carries the harvested three-stage masks re-derived as const-friendly
  no_std Rust under `forbid(unsafe_code)`. Divergence recorded here
  rather than silently redefining "64x64".
- bcinr's `aux` perturbation input is dropped: the court needs a pure
  transpose, not a perturbed one.

## Courts (all real collaborators, final-state asserts)

Command: `CARGO_TARGET_DIR=/tmp/ts4-target cargo test -p cgap-branchless`
exit 0 — 34 lib + 3 census + 8 scan + **7 transpose courts** passed,
0 failed (all suites this session).

1. `court_8x8_differential_oracle` — naive scalar per-bit relocation
   oracle over: boundaries (0, u64::MAX, all 64 single bits, one full
   row / column per index), exhaustive 2^16 (row-pattern, col-pattern)
   byte-replicated sweep, 4096-value frozen splitmix corpus from
   `0xA20C_0FFE_E5A7_E000` (canonical corpus seed; splitmix multiplier
   `0xBF58_476D_1CE4_E5B9` kept distinct). PASS.
2. `court_8x8_involution`. PASS.
3. `court_64x64_differential_oracle` — same oracle discipline over
   boundary rows/cols, diagonal/anti-diagonal, all 128 single-bit
   matrices, block-structured corpus, 256 frozen splitmix blocks. PASS.
4. `court_64x64_involution_and_stage_selfinverse` — transpose-twice
   = identity over the full 64x64 corpus + per-stage self-inverse
   identity for each harvested mask. PASS.
5. Census (nightly rustc → /tmp obj → llvm-objdump), caller-cell
   precedent (branchless_census CALLER_GRID):
   - `census_t8` (transpose_8x8 inlined): **0 branches (HOT)** on
     host-aarch64 AND riscv32imc.
   - `census_t64`: **2/2 documented residual** on both targets — the
     `quad_mask` match-dispatch compare branches; the six quadrant
     loops fully unroll at opt-level 3.
6. ANTI-VACUITY `anti_vacuity_mutant_flips_differential_court`
   (M-TS4-1): pristine twin binary (real transpose.rs via #[path],
   real oracle, fixed corpus incl. x=1) exits **0**; the /tmp sandbox
   twin with a single-bit flip in the stage-7 mask
   (`0x00AA...AA` → `0x00AA...A8`) exits **nonzero** — the
   differential court flips. Shared tree untouched by the mutant.
7. Integration probe (honest, /tmp only): `cgap-storage`'s
   bitplane_seen stores 128 vertical planes × `[u64; SLOTS]` and
   reconstructs keys plane-wise (bitplane_seen.rs:25-38) — a 64x64
   bit-matrix transpose has no natural consumer there (128 planes, not
   64; planes indexed by bit, not by matrix row). **Verdict: the
   primitive lands as library-capability; the consumer question is
   posed** (a consumer would be a plane-major → slot-major repack with
   a different arity — future lane).

## Census budget

`T64_LOOP_RESIDUAL = 2` (observed 2/2 on host + riscv32imc), documented
in-file; `census_t8` HOT budget 0, observed 0.

## Standing

ALIVE: `cargo test -p cgap-branchless` exit 0 this session against the
exact landed subject; 52 tests, 0 failed, includes pre-existing suites
(34/3/8) untouched.