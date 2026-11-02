# W3-T2 — asm census trio (0 blr / 0 br xN)

Lane: W3-T2, gate-runner. Date 2026-11-01. Read-only except this receipt.
Toolchain: rustc nightly aarch64-apple-darwin; asm via
`CARGO_TARGET_DIR=/tmp/w3-t2-asm<crate> cargo rustc --release -p <crate> -- --emit asm`
(rlib .o members are LLVM-bitcode wrappers on this toolchain, so objdump on the
rlib object is not usable; --emit asm is the working method). Counting: python
regex over the `.s` function bodies, bounded by symbol labels
(`^__?[A-Za-z_$].*:$`).

## Ladder

| Gate | Command (cwd /Users/sac/cgap) | Exit | Evidence |
|---|---|---|---|
| build | CARGO_TARGET_DIR=/tmp/w3-t2-target cargo build --release -p cgap-gate -p cgap-bn3 -p cgap-p28 | 0 | Finished release in 1.26s |
| gate census | --emit asm, cgap_gate-3b8b0ea314fa4dc8.s | 0 | table below |
| bn3 census | --emit asm, cgap_bn3-72306c3797b03464.s | 0 | table below |
| p28 census | --emit asm, cgap_p28-379afdf09142512f.s | 0 | table below |

## Counts (per function region)

- cgap-gate `penstock::ingress` (penstock.rs:491):
  __RNvNtCsaPeMjhPWzvF_9cgap_gate8penstock7ingress — bl=0 blr=0 br xN=0 brk=0 (ret=2; 2 arms,
  hot arm is fallthrough+ret). 158 lines.
- cgap-bn3 `noalloc` eval/dispatch region (noalloc.rs): blr=0 br xN=0 brk=0 in all four
  functions; direct `bl` only:
  - na_compile bl=3, push_match bl=9, atom_local_match bl=8, na_unify bl=19
  - na_unify bl targets are all intra-region: atom_local_match (5 sites) + push_match (2 sites).
- cgap-p28 `asc450` fold region (asc450.rs): blr=0 br xN=0 brk=0 in all ten functions;
  direct `bl` only (memcmp / cgap_blake3::hash / unwrap_failed / core::fmt):
  - SealedMeterState::from_bytes bl=0, to_bytes bl=0; SignedProof::from_bytes bl=0, to_bytes bl=0
  - LeaseEpochSealer::epoch_binding bl=6; Display::fmt bl=1
  - ProofSealer seal/seal_state/verify/verify_state bl=1 each (blake3 + memcmp, direct).

## Verdict

PASS — 0 blr / 0 br xN / 0 brk across all three named regions, matching the
v26.11.1 zero-indirection law (waves/a13-finish-receipt.md:39, M15 row of
position-paper-v2.md:56). All branch targets are direct symbolic (bl) or
intra-region conditional branches (b./cb./tb.); no indirect dispatch.
This census is MUTATION-FREE, so it is a census, not a court — no vacuity
claim is made beyond the named regions.

## Notes

- L3 lever status at poll: docs/jira/v26.11.2/waves/w3-l3-semijoin.md DOES NOT EXIST;
  w2-12-l3-court.md present. cgap-bn3 built clean (exit 0, 1 unused_variables warning —
  possible mid-edit residue, unnamed file not captured). Census ran on the current tree.
- L5 `fused_row` NOT present in the p28 asm symbol table — L5 is not landed; census covers
  the pre-L5 asc450 surface (LeaseEpochSealer seal/verify path).
- bn3 non-region functions (hashbrown/ slice-sort monomorphizations, unify alloc arm)
  contain brk traps and alloc-call bl — out of scope, listed for honesty: brk #0x1 at
  .s lines 1229, 1871 (ipnsort unreachable traps).
- Build roots /tmp/w3-t2-target, /tmp/w3-t2-asmgate, /tmp/w3-t2-asmbn3,
  /tmp/w3-t2-asmp28 deleted via shutil.rmtree after receipt.

## JSON footer

{"lane":"W3-T2","date":"2026-11-01","verdict":"PASS","regions":{"gate_ingress":{"bl":0,"blr":0,"br_xn":0,"brk":0},"bn3_noalloc":{"na_compile":{"bl":3},"push_match":{"bl":9},"atom_local_match":{"bl":8},"na_unify":{"bl":19},"blr":0,"br_xn":0,"brk":0},"p28_asc450":{"blr":0,"br_xn":0,"brk":0,"direct_bl_only":true}},"method":"cargo rustc --release -- --emit asm; python label-bounded regex census","commands_exit":"build=0, emit-asm=3x0","l5_fused_row_present":false,"l3_wave_file_present":false,"standing":"ALIVE"}
