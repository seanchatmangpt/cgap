# Lane E-02 — R-ADV11FIX rv32imc ladder verification (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
verification executed by the E-02 Explore lane in the coordinator session;
read-only, could not land its own file — landed by the coordinator).

## W1 cfg probe — VERIFIED (re-grounded read-only)

`rustc --print cfg --target riscv32imc-unknown-none-elf` (1.101.0-nightly,
12433c307): legacy `target_has_atomic` keys ABSENT at every width;
`target_has_atomic_load_store` family PRESENT (8/16/32/ptr + bare; no "64").
Host legacy keys present through "128" — the slide is rv32imc-specific, not
global removal.

## W2 ladder structure — VERIFIED (crates/cgap-git/src/ref_store.rs)

- impl64 :140 / impl32 :143 / impl16 :146 / implcell cfg-block :148-153
  (transcript's :149-153 off by one — corrected at landing).
- Opt-in-gated `critical`: #[cfg(implcell_critical_provided)] :212-215.
- Loud refusal gate :225-239 (compile_error! :232-239) naming both lawful
  exits (opt-in bind, or CAS-capable target).
- SELECTED_ARM :824-836 / EXPECTED_ARM :847-863, mismatch assert ~:866-871;
  E0658 note :841-843 (load_store family UNSTABLE, rejected for stable
  target_pointer_width).
- Width witness anchor: SEAL_COUNTER_WIDTH = seal_atomic::WIDTH, witness
  block :732+.

## W3 doc truthfulness — VERIFIED

- Module doc :30-39 truthful post-slide (names R-ADV11FIX + receipt).
- PY-5 row (dflss/08-ax-pfmea.md:128): LANDED-CODE-REPAIRED, impl32 premise
  marked STALE, F-01 re-verification recorded.
- red-11 ERRATA landed at red-11-riscv-reentrancy-corruption.md:124 (§1
  superseded; ADV-11 MATERIALIZED-ON-TARGET :130; F-01 re-verification :135).

## W4 standing

TRANSCRIBED, landed by the coordinator. Original verification: E-02 Explore
lane (read-only). Corrections at landing: implcell range :148-153 (was
:149-153); receipt path refs resolve under docs/jira/v26.11.2/.
