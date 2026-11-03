# Lane FS-06: SRAM-row errata cite fix (transcription of A-02 terminus)

Status: TRANSCRIBED. This receipt is a transcription of lane A-02's
terminus from the coordinator session record; the original A-02 reply-back
was never landed as a file. Transcribed and cite-verified by lane FS-06
(2026-11-02). Owned file only: this one. No git commands run.

## 1. Landed row (11-measured-claims.md claim ID 38)

`docs/jira/v26.11.2/11-measured-claims.md:166` — claim 38, standing
MEASURED:

> SRAM static charge, gated kernel slice (blake3+branchless freestanding
> caller, riscv32imc): .text+.rodata+.data+.bss families vs 131,072 B hard
> bound, headroom 95,024 B. Complement: `--subject bn3` typed refusal
> BN3_ENCLAVE_REFUSED at 6,520,984 B = 49.75x ceiling (red-12)

Value 36,048 B; method: gate exit 0 + bn3 refusal exit 1; cites
`dod/dod-04-unsafe-sram-census.md:72-89,112-120`.

## 2. Gate-line verification (re-read this session, FS-06)

`scripts/verify_cscb_sram.py`:
- `:45` `SRAM_BUDGET = 131072  # 128 KiB hard bound (gate, not advisory)` — exact.
- `:52-56` `DEPENDENCY_CRATES` dict: cgap-blake3, cgap-branchless,
  cgap-bn3 — the gated subjects.
- `:57-60` RED-12 comment block ("cgap-bn3 is a GATED subject") +
  `BN3_SOURCE` at `:60`.
- Runtime checks: `:212` bound compare, `:218-219` refusal emission,
  `:315` blake3 subject path.

Row-38's cited gate lines (`:45`, `:52-56`, `:57-60`) all resolve exactly.

## 3. Mutation-court cites (lane-fix02-bn3-sram-gate.md:45-62)

Re-read; both flips witnessed in that window:
- bn3-pin court: NA_MAX_MATCHES 2048→1024 sandbox mutation → DIVERGED
  refusal exit 1.
- sram-bloat court: static 100 KB blob → 136068 B > 131072 B bound,
  exit 1.
- /tmp sandbox restore sha256-exact + mtime bump; corrupted-restore
  probe detected (restore-detection non-vacuous).

## 4. Divergence

A-02's dispatch cited gate lines ":51-54"; on-disk truth is `:45`
(SRAM_BUDGET), `:52-56` (subjects), `:57-60` (bn3 comment). The landed
row-38 text already carries the corrected `:45/:52-56/:57-60` lines, so
the row needs no edit; only this transcription receipt was missing.

## 5. Fences (must-not-regress surfaces this diff could break)

None. This lane wrote one NEW markdown receipt file; no code, no gates,
no shared files touched. Fence run: cite-chain re-read of
11-measured-claims.md:166, verify_cscb_sram.py:45-60,
lane-fix02-bn3-sram-gate.md:45-62 — all resolve (commands above, exit 0
per sed/grep rc).

Standing: ALIVE (cite chain re-read on disk this session).
