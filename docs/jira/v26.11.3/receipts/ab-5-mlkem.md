# AB-5 Receipt: ML-KEM-768 kernel (fips203 + kem)

Lane: AB-5 (AoQ Wave-1 W1-2b). Crate: cgap-crypto. Subject:
`crates/cgap-crypto/src/fips203.rs`, `crates/cgap-crypto/src/kem.rs`,
`crates/cgap-crypto/tests/mlkem768_*`, `crates/cgap-crypto/tests/vectors/mlkem768/`.
FIPS 203 (final) ML-KEM-768: k=3, eta1=eta2=2, du=10, dv=4.
Standing: ALIVE

## Delivered

- `fips203.rs`: ML-KEM-768 consts with checked const asserts
  (EK=1184, DK=2400, CT=3*320+128=1088, SS=32 — includes the
  coordinator-relayed CT const-math fix), branchless Montgomery (QINV=62209
  as u16 — the i16-literal-overflow fix) / Barrett (v=20159) reductions,
  ZETAS table (pqcrystals/PQClean lineage, exact PQClean indexing: ntt
  reads ZETAS[1..=127], intt ZETAS[127..=1], basemul ZETAS[64+i]; [i16;259]
  layout documented in-code), in-place NTT/INTT (fixed 7x128 trips),
  CBD eta=2, ByteEncode/Decode d=1..12, compress/decompress du=10/dv=4,
  PRF/J (SHAKE-256), H (SHA3-256), G (SHA3-512, rate 72), SampleNTT
  rejection sampling, matrix A / A^T generation, canonical tobytes.
- `kem.rs`: deterministic keygen(d,z) / encaps(ek,m) / decaps(dk,ct);
  NO RNG in-crate; decaps infallible by type (both candidate keys computed
  unconditionally, OR-fold + byte-cmov select); `#[inline(never)]` on
  decaps, no inline(always) on any entry symbol; `decaps-only` feature
  compiles (keygen/encaps carry cfg_attr allow(dead_code) — they still
  compile under the feature; a hard cfg-out would break the lib build,
  disclosed).
- Courts in tests/: zetas PQClean entry-check + blake3 digest pin
  (1225892f302b3678d778c4ff059ca318bd2cd2c25ff98b99f3cada566c4cc4f6),
  NTT C-reference pins (ntt.o outputs captured in-session from the compiled
  PQClean MLKEM512 object), basemul-vs-schoolbook, ByteEncode/Decode d=1..12
  re-encode property, KEM determinism, round-trip, 3+3 negative
  corrupted-ct courts (rejection key != ss, deterministic rejection key),
  KAT courts (byte-exact pins vs liboqs derand probes, below),
  zero-alloc court (counting allocator: 0 allocs across keygen/encaps/
  decaps incl. corrupted-ct decaps, with non-vacuity witness), bench
  (min-of-100 us/op), liboqs interop court (both directions).

## External ground truth + provenance

- pqcrystals/kyber ref lineage via PQClean MLKEM512 clean C sources
  (~/bytestar/bytepqc/src/pqclean_kyber512_clean/); ZETAS cross-checked
  entry-by-entry against the .c literal AND byte-for-byte against the
  compiled ntt.o table (offset 4500, 128/128).
- liboqs 0.13.1-dev ML-KEM-768 (FIPS 203 final):
  - keygen differential: PQCP_MLKEM_NATIVE_MLKEM768_C_keypair_derand with
    d=[7;32], z=[9;32] -> ek byte-exact vs mine (ek[0..32] pinned 925a2700
    ...afca in mlkem768_kat_courts).
  - encaps differential: PQCP_MLKEM_NATIVE_MLKEM768_C_enc_derand with
    m=[3;32] -> ct byte-exact (459f33...40f9) and ss byte-exact
    (868c53...9266), pinned in mlkem768_courts.rs (mlkem768_kat_courts.rs).
  - interop A: my keygen -> liboqs encaps -> my decaps == liboqs ss.
  - interop B: liboqs keypair -> my encaps -> liboqs decaps == my ss.
  - Network ACVP access was unavailable (curl 404) — the liboqs derand
    differentials + interop courts are the KAT substitute; disclosed.
- FIPS 202 known answers: SHAKE-256(""), SHA3-512(""), SHA3-256("abc").
- Code assembled with python string-assembly, not hand-edits (write-corruption
  discipline from CLAUDE.md).

## Session defects found and fixed (each with its discovery receipt)

1. CT_BYTES const math: K*320+32 -> K*320+128 = 1088 (coordinator relay).
2. QINV as i16: 62209 overflows i16; typed u16 + wrapping_mul.
3. Montgomery const assert: q*QINV == +1 mod 2^16 (not -1).
4. SHA3-512 rate: 136 -> 72.
5. ZETAS off-by-one layout: first build placed the table at [1..=128];
   regenerated to exact PQClean indexing (the root cause of the first
   round-trip failure; ground truth = compiled ntt.o differential).
6. cond_add_q: unsigned shift made (1 & q) == 1 instead of arithmetic
   shift ((-1) & q) == q — broke all canonical serialization.
7. FIPS 203 canonical shat in dk: Kyber-R3 packs shat UNREDUCED
   (self-consistent with its own frombytes, non-conformant for FIPS 203);
   liboqs packs canonical ByteEncode12(shat) — added poly_reduce before
   poly_tobytes in keygen. THE round-trip + interop fix; proven by the dk
   byte-diff (302 bytes, confined to the shat region) and by full interop
   after the fix.

## Gates (this session)

- cargo test -p cgap-crypto (full crate) x2: pass 1 = 74 tests 0 failed
  exit 0; pass 2 = 76 tests 0 failed exit 0 (background receipts).
- decaps-only feature: `cargo check -p cgap-crypto --features decaps-only`
  exit 0 (feature re-added to Cargo.toml after a concurrent overwrite —
  disclosed).
- Cross targets: `cargo check -p cgap-crypto --target
  riscv32imc-unknown-none-elf` exit 0; `--target wasm32-unknown-unknown`
  exit 0.
- Corridor join (AF-1, AF1-KD-1 closure): crates/cgap-aoq corridor_join_court
  REBOUND to the real ML-KEM-768 seam (MlkemKem adapter over
  cgap_crypto::kem; TestKem fallback retired); run 1: 3 tests 0 failed
  75.7s incl. the 100-iteration J1 court on the real KEM hop; run 2: 3 tests 0 failed (exit 0).
- Final state re-read from disk: fips203.rs + kem.rs compile under all four
  surfaces (default, decaps-only, riscv32imc, wasm32); 11 court files on
  disk under tests/; receipt on disk (this file). Court file edit disclosed here
  as the AF-1 close condition named in af-1-corridor.md.
- Bench (cargo test --release --test mlkem768_bench, aarch64 darwin,
  min-of-100): keygen 29.60 us/op, encaps 30.46 us/op, decaps 33.55 us/op
  (OQ-4 rekey-cost baseline; UNVERIFIED for other targets).

## Stack-depth note (riscv32imc)

Analytical estimate (NOT measured on-target; UNVERIFIED): worst frame is
the keygen/decaps path: A matrix 9 x 512 B = 4.6 KiB + s/e/t-hat 3 x 512 B
= 1.5 KiB + XofState (Sponge 200 B + 168 B buf) + serialization scratch
~1 KiB, plus AB-2 Sponge internals — order 8-9 KiB peak, plus callee
frames. Callers on riscv32imc should budget >= 16 KiB for the KEM path.
A linked-ELF stack-depth measurement (CLAUDE.md footprint truth method)
is a Wave-2 item.

## Disclosures

- decaps-only: soft gate (dead_code allow) not hard cfg-out; rationale above.
- The sha3 seam (fips203::sha3) drives AB-2's public keccak_f1600 with
  0x06 padding — deleted when AB-2 exposes a SHA3-domain sponge.
- Sharing edits: lib.rs (module wiring, minimal), Cargo.toml
  ([features] decaps-only), both disclosed here as minimal unblocks.
- In-session scratch tests (dbg_*.rs) were removed; AF-12's kat_flip_demo.rs
  was accidentally deleted during scratch cleanup and restored byte-exact
  from HEAD via git show (read-only; no state transition).
- The session also relays a courtesy note for AF-1/AF-26/AF-27: the
  'in-flight ML-KEM' typed disclosure rows are now CLOSED - ML-KEM-768 is
  ALIVE and the corridor join can bind the real KEM hop.
