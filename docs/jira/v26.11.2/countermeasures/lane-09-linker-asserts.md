# lane-09-linker-asserts receipt (AE9, v26.11.2, 2026-11-02)

Linker section pins + gate symbol cross-check on top of D3's 128 KiB
enclave budget script (`crates/cgap-ffi/enclave_128k.ld`). D3 drained;
this lane EXTENDED its file surgically (no D3 ASSERT touched, none
duplicated).

## Files written (lane-owned only)

- `crates/cgap-ffi/enclave_128k.ld` — additive block after D3's ASSERTs:
  symbol pins `_stext/_etext` (ADDR(.text), +SIZEOF) and `_sbss/_ebss`,
  direct assignments NOT PROVIDE (a PROVIDE pin only materializes when
  referenced — the gate must be able to FAIL on a missing pin, never
  pass on one). New fail-closed ASSERTs: `.text <= 98304` (96 KiB
  per-section sub-budget), `.bss <= 16384` (16 KiB sub-budget), and
  total `(text+rodata+data+bss) <= 131072`. D3's existing text<=128K,
  text+rodata<=128K, data+bss<=64K ASSERTs untouched.
- `scripts/enclave_budget_gate.py` — additive `--verify-symbols` /
  `--verify-symbols-mutant` / `--elf` modes. llvm-nm (found beside
  llvm-size) over the linked ELF: all 4 pins must exist, pin-derived
  .text/.bss sizes must match `llvm-size -A` within 1 byte. Missing
  ELF = exit 2; missing/refused pin or >1 B delta = exit 1; llvm-nm
  absent = loud [SKIP] exit 3. Mutant: zeroes one pin's name string in
  .strtab of a /tmp byte copy; the gate MUST refuse it (VACUOUS GATE
  exit 1 on a pass). Existing modes untouched (`--self-check` rerun
  exit 0 this session).
- `docs/jira/v26.11.2/countermeasures/lane-09-linker-asserts.md` —
  this receipt.

## Commands + exits (this session)

- `python3 -m py_compile scripts/enclave_budget_gate.py` — exit 0.
- Baseline fixture link (`/tmp/ae9-court/baseline`, D3 fixture shape:
  .oabi 96 B + .bn3 48 B, riscv32imc, `cargo +nightly`,
  `RUSTFLAGS="-C link-arg=-T<enclave_128k.ld>"`) — exit 0.
- `llvm-nm` over the baseline ELF: `_stext=0x10000000 _etext=0x10000006
  _sbss=_ebss=0x30000000` — all 4 pins materialized.
- `enclave_budget_gate.py --verify-symbols --elf <baseline ELF>` —
  exit 0 (pin-derived .text 6 B / .bss 0 B, delta 0, match llvm-size).
- `--verify-symbols --verify-symbols-mutant --elf <baseline ELF>` —
  exit 0; mutant refused (gate exit 1 on `_stext` absent).
- AE9 FLIP WITNESS: `/tmp/ae9-court/bssmutant` — 24 KiB
  `static mut BSS_BLOB` anchored by `read_volatile(addr_of!(BSS_BLOB))`
  (>16384 B .bss) linked against the SHARED (unmutated) ld — exit 101,
  stderr names `cgap ENCLAVE 128K: .bss exceeds the 16384-byte (16 KiB)
  per-section sub-budget`. FLIPPED on the exact new ASSERT message.
  (Two fixated iterations recorded: an immutable static landed in
  .rodata and an unreferenced static was GC-stripped — both exited 0
  and were replaced; only the static-mut+anchored mutant flips.)
- `--verify-symbols --elf /nonexistent.elf` — exit 2 (missing subject).
- `--self-check` regression (arms C/P) — exit 0; arm M loud [SKIP].
- Shared-tree sha256 of enclave_128k.ld, pinned before and after every
  leg, byte-identical (the ld was never mutated — the mutant fixture
  lives in /tmp):
  `306f5140590ffb9fc4e6b620b4a53e7e26787601d88e7130b0da6706799c01f8`

## Standing

ALIVE — every gate/flip leg above is a command+exit run this session
against the exact on-disk files.

## Divergences

1. Dispatch: "mutant overflowing .bss" implied mutating the ld; the
   flip needs no ld mutation — the /tmp fixture fixture-owns the
   oversize .bss and the shared script refuses it (stricter subject:
   the exact checked-in bytes are what flips).
2. First mutant design zeroed a symtab entry; llvm-nm kept reporting
   the pin (entry 1 is a non-pin symbol) — mutant upgraded to strtab
   name-zeroing before the court was accepted; the earlier VACUOUS-
   GATE-detection run (court refused the weak mutant, exit 1) is
   recorded as the court's own flip proof.
3. `truncate_symbol_mutant` initial ELF32 section-header offsets were
   ELF64 values (loud [SKIP] + wrong-size reads witnessed); the
   strtab approach removed the section-walk entirely.
