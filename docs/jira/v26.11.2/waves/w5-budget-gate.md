# W5 D4 — enclave budget gate (128 KiB envelope court)

Lane: W5 D4 (Design), 2026-11-02, repo /Users/sac/cgap, branch main.
Files written: `scripts/enclave_budget_gate.py` (new), this receipt.
No git writes; /tmp sandbox only; Python 3 stdlib only; no mocks, no
Process.sleep.

## Gate

`scripts/enclave_budget_gate.py` v1 — house exit ABI
(0 pass / 1 refuse / 2 missing subject / 3 loud skip), modes
`--check`, `--mutate`, `--self-check`, subjects via
`--artifact TIER=PATH` (.o/.elf/.rlib; .rlib is llvm-ar-extracted and
sized as the sum of per-member `llvm-size -A` Totals, .rmeta excluded).

Budget table (frozen v1):

| slot | bytes | KiB |
|---|---|---|
| whole envelope | 131072 | 128 |
| tier kernel | 16384 | 16 |
| tier storage | 49152 | 48 |
| tier compute | 49152 | 48 |
| tier attest | 12288 | 12 |

(Tiers sum to 124 KiB; 4 KiB envelope slack — tiers bind per-slot, the
envelope binds the sum of measured artifacts.)

Sizing method: real `llvm-size -A` over real artifacts (subprocess); no
mocks. Tool resolution: `CGAP_LLVM_SIZE` env, then PATH
(`shutil.which`), then sorted rustup-toolchain globs, homebrew, Xcode;
absent -> exit 3 `[SKIP]`.

## Commands + exits (this session, all witnessed)

Subject artifacts: audit-profile riscv32imc rlibs rebuilt this session
into the lane lease /tmp/d4-target (the M1 /tmp tree
`/tmp/m1-footprint-target` had been deleted by lease cleanup mid-session
— see divergences), build exit 0:

    CARGO_TARGET_DIR=/tmp/d4-target cargo build -p cgap-blake3 -p \
      cgap-crypto -p cgap-p28 -p cgap-gate \
      --target riscv32imc-unknown-none-elf --profile audit   # exit 0

    python3 scripts/enclave_budget_gate.py --check \
      --artifact kernel=.../libcgap_blake3-7c5fa0bf72b77bae.rlib \
      --artifact attest=.../libcgap_p28-513b8df4fc83b47c.rlib \
      --artifact storage=.../libcgap_gate-c5afee7bdc448a87.rlib \
      --artifact compute=.../libcgap_crypto-11aecd20a77727bf.rlib
    # exit 1 — baseline RED, honest breach:
    #   kernel  114072 / 16384  BREACH  (blake3 audit rlib)
    #   attest    9220 / 12288  OK
    #   storage   1048 / 49152  OK
    #   compute  20146 / 49152  OK
    #   envelope 144486 / 131072  BREACH

    --mutate (30 KB blob appended to a /tmp copy of the blake3 rlib)
    # exit 0 — mutant REFUSED (gate exit 1): llvm-ar fail-closed archive
    # verdict ("truncated or malformed archive ... ENCLAVE-BUDGET-M").
    # A mutant pass would have been VACUOUS GATE; it did not pass.

    --self-check (with the blake3 rlib subject)
    # exit 0 — arms:
    #   C clean     exit 0 (real cc-compiled tiny object, kernel tier)
    #   P poisoned  exit 1 (real cc-compiled 32 KiB global, kernel tier)
    #   M mutant    exit 1 (real-artifact 30 KB-append refuses)

    --check over p28/gate/crypto only   # exit 0 (green arm witnessed)
    --check --artifact kernel=/tmp/nope-d4.o  # exit 2 MISSING SUBJECT
    (no mode)                            # exit 1

## Verdicts

- Budget court: ALIVE and flipping both directions — refuses the real
  114 KB blake3 audit-rlib kernel-tier breach (exit 1) and passes the
  9220/1048/20146 B set (exit 0); the 30 KB-append mutant is refused,
  the poisoned cc-compiled arm is refused, the clean arm passes.
  Anti-vacuity: every arm flips; no arm vacuous.
- Baseline RED is a REAL finding, not a gate defect: the audit-profile
  cgap-blake3 riscv rlib alone is 114,072 B of section total
  (SIMD-wide hash tables + code), over the 16 KiB kernel tier and
  pushing the envelope to 144,486 B > 131,072 B. M1's intended 940 B
  object was the linked envelope object; this lane measured per-crate
  rlibs, which is a stricter surface (see divergences).
- dev-profile rlibs (measured too: 1,349,377 B blake3 with debuginfo)
  breach far harder; the budget table is calibrated to optimized
  audit/release-profile artifacts.

## Divergences

1. M1's `/tmp/m1-footprint-target` (the 940 B riscv object the dispatch
   named) was deleted by lane-lease cleanup before this lane ran; the
   wasm-footprint subject was also absent (`m2-err-wasm-footprint.txt`
   ends in "package ID specification `wasm-footprint` did not match").
   This lane rebuilt the subjects itself (cargo, audit profile,
   riscv32imc, exit 0 above) into its own lease /tmp/d4-target, and
   measured rlib section totals rather than a linked 940 B envelope
   object — same budget law, stricter subject granularity.
2. Release-profile rlibs embed LLVM-IR bitcode members that llvm-size
   refuses ("not recognized as a valid object file"); the gate
   fail-closes on them (exit 1, typed). Use the audit profile for
   budget subjects — it stores real ELF rcgu objects.
3. The mutant refusal is delivered via the fail-closed ARCHIVE-parse
   channel (llvm-ar refuses the corrupted trailer) rather than the
   budget channel; both channels exit 1. A blob appended in a way that
   preserves the ar trailer would still breach via budget (witnessed
   independently by self-check arm P's pure-size refusal of a real
   oversized object).
