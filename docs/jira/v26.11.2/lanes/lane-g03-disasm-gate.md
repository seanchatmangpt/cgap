# Lane G03 — Disassembly-Invariants Gate Hardening

Standing: ALIVE (all commands below run this session, 2026-11-02, /Users/sac/cgap @ main)

## Scope (per dispatch)

Files written: `scripts/verify_disassembly_invariants.py` (gate hardening only;
no source edits to cgap-branchless), `docs/jira/v26.11.2/lanes/lane-g03-disasm-gate.md` (this receipt).
G01/G02 concurrently own `crates/cgap-branchless/src/{scan,transpose}.rs`; this lane
never touched them.

## Task 1 — baseline audit (symbol-matching/regex vs real mangling)

Run: `python3 scripts/verify_disassembly_invariants.py` (baseline, pre-extension)
Output persisted: /tmp/g03-full.log analysis (run 1, exit **1**).

Findings:
- Emitted-asm symbols are legacy-v0 mangling `_RNvNtCsggheNpNv8AU_15cgap_branchless...`
  with a trailing `B3_` extra-context suffix and a crate-disambiguator hash.
  `SYMBOL_RE` (`^(_RNv[A-Za-z0-9_]+)>?:$`) and the substring attribution
  (`fn in current`) tolerate both; every FORBIDDEN row named a correct
  mangled symbol. llvm-objdump `<sym>:` headers match OBJDUMP_INSTR_RE.
- Baseline hits (all in scan/transpose, all eliminated by G01/G02 later
  the same session):
  - count_leading_eq_u8: beqz x1
  - prefix_max_u8x8: bgeu x1, bltu x1
  - transpose_64x64: bne x1
  Regexp audit verdict: matcher CORRECT against the real artifacts;
  no redefinition needed. One latent bug found and fixed: the arm E
  symbol-capture regex included the trailing `>` in the captured symbol
  (`[^:+]*` -> `[^:>]*`), which broke ELF section lookup.

## Task 2 — byte-level negative-mutation arm (Arm E, M-G03-E)

Added `binary_mutant_court` (arm E): extracts the cgap_branchless object
member from the linked-witness staticlib (GNU/BSD ar parser handling
`/N` extended-name table), picks an allowlist symbol, patches the first
4-byte RV32 instruction word to `bnez x1, 8` (0x00009e63, little-endian
63 9e 00 00) in the SCRATCH copy, re-disassembles, and requires bnez
count to rise by exactly 1. Original archive sha256 compared before/after:
UNCHANGED (fail-closed restore; the shared tree was never mutated).

Flip proof (witnessed):
- pre_ops bnez=5, post_ops bnez=6 in
  `_RNvNtCsgFeXzD4EKeT_15cgap_branchless6select22checked_uniform_select`
- `[BINMUT] original archive sha256 UNCHANGED (restore verified)`
- final run: `[PASS] anti-vacuity M-G03-E ... (court can flip at byte level)`

Two real defects fixed while landing arm E:
1. Fixed scratch dir `/tmp/ae4-scratch` was a cross-lane race: concurrent
   gate runs (G01/G02 courts) `rmtree` the shared path mid-run and delete
   another run's witness. Scratch is now a unique per-run mkdtemp lease
   (`ae4-scratch-*`), deleted at lane end.
2. 4-byte instruction detection used the bytes-column token count, which
   fails on riscv llvm-objdump (bytes printed without spaces). Now:
   stripped bytes-column length >= 8.

## Task 3 — exit-code ABI, pinned and armed

Pinned as typed constants (EXIT_CLEAN/EXIT_BRANCH/EXIT_SUBJECT_MISSING/
EXIT_LOUD_SKIP). Every arm has a run witness:

| Code | Meaning | Test arm | Witnessed exit |
|---|---|---|---|
| 0 | clean | final full run after G01/G02 landed | `EXIT=0` |
| 1 | forbidden branch or vacuous court | baseline run (3 FORBIDDEN rows); vacuous arm proven by source-mutant + self-check + arm E all flipping | `EXIT=1` |
| 2 | binary missing | `--witness /nonexistent/binary.a` -> `[FAIL] witness binary missing` | `EXIT_MISSING=2` |
| 3 | loud skip | `CGAP_AE4_FORCE_SKIP=1` -> `[SKIP] forced loud-skip arm` | `EXIT_FORCE_SKIP=3` |

Exit-1 class also covers VACUOUS: mutant court, branchy self-check and
arm E each print [VACUOUS] and force exit 1 when they fail to flip.

## Task 4 — post-G01/G02 shared-tree gate

Final run: `python3 scripts/verify_disassembly_invariants.py` -> exit **0**,
`AE4 ... CLEAN (census + linked witness + mutant + self-check + binary
mutant all held)`. Zero FORBIDDEN rows. Residuals still loudly reported:
cmov_u64 (1 bnez), eytzinger_restore (1 beqz), select_slice (2) as
ALLOWED-RESIDUAL (RV32 ISA ceiling, lane-03 cites), plus REFERENCE-DRIVER
loop-back-edge rows. Log: /tmp/g03-final.log.

## Commands and exits

- baseline: python3 scripts/verify_disassembly_invariants.py -> exit 1
- python3 scripts/verify_disassembly_invariants.py --witness /nonexistent/binary.a -> exit 2
- CGAP_AE4_FORCE_SKIP=1 ... -> exit 3
- full run 3 (arm E skip) -> exit 3 (loud; scratch race, fixed)
- full run 4 -> exit 3 (loud; 4-byte detect bug, fixed)
- final full run -> exit 0

## Fences (must-not-regress surfaces this diff could break)

- Arm A census semantics: untouched (extensions additive); baseline and
  final runs produce identical ALLOWED-RESIDUAL/REFERENCE-DRIVER rows
  for unchanged symbols.
- Arm B linked witness: now returns the archive path; census rows
  unchanged (witnessed identical linked-witness lines across runs).
- Arm C/D anti-vacuity: untouched; both still PASS in the final run.
- ABI: previous callers saw exit 1 for FORBIDDEN, 2 for missing
  subject, 3 for skips — all preserved; only new behavior is the new
  typed arm E and the unique-scratch default (was a shared fixed dir;
  concurrent runs previously raced each other's scratch).

## Divergences from dispatch

- Dispatch: "witness binary" for the bnez mutation — implemented as a
  scratch copy of the linked staticlib archive member (the linked
  witness binary), extracted and byte-patched, then re-disassembled.
  The mutation subject is a /tmp copy; the original's sha256 is
  verified unchanged (fail-closed restore law), per dispatch.
- Dispatch assumed exit-1 state until G01/G02 land; G01/G02 landed
  mid-session, so the gate's final state on the shared tree is exit 0
  (witnessed), not BLOCKED-on-sibling. The three baseline FORBIDDEN
  hits (count_leading_eq_u8 beqz; prefix_max_u8x8 bgeu+bltu;
  transpose_64x64 bne) were present early-session and are gone in the
  final run — G01/G02's fix, not a gate change.
- The gate's module docstring cites p2/EX3 (lane-03) as method of
  record and notes "w6-b6 does not exist" — that pre-existing
  divergence note was already in the file; left as-is.

## Standing

ALIVE — every ABI arm and the full gate ran this session against the
exact subject with recorded exits. Perf claims: none made. The exit-0
state is conditional on the current G01/G02 tree; the residual
allowlist rows are typed and loudly reported, never silently passed.
