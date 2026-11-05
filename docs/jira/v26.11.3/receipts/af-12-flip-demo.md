# AF-12 — kat_flip_demo.rs (runnable negative-mutation demonstration court)

Standing: UNVERIFIED

Lane: AF-12, completed by BK-3 takeover (dead-lane inheritance). Date: 2026-11-03. No git commands run.

## Subject

- `crates/cgap-crypto/tests/kat_flip_demo.rs` — found complete on disk at
  takeover (632 lines, all four courts authored by the dead lane; BK-3
  made no edits to this file).

## What the court proves

- **A. SIGMA avalanche (ChaCha)**: all 128 single-bit flips of
  `expand 32-byte k`, one at a time, each compiled standalone in a /tmp
  sandbox via rustc; each must change >= 1 byte of the RFC 8439 §2.3.2
  block fixture. Exhaustive, not sampled.
- **B. fips202 RC bytes**: all 192 bytes of the RC[0..24] table flipped
  one at a time; each must make the SHAKE-256("") FIPS 202 KAT reject.
- **C. fips203 ZETAS**: low bit of entries [1, 2, 64, 127] (all consumed
  by ntt()/intt_tomont()); each must fail the ntt∘intt_tomont roundtrip
  self-test AND change the NTT output. AB-5's ZETAS table is STABLE
  (landed); the skip-typed contingency was not needed.
- **M. Meta-court**: flips_detected == flips_total (324 == 324), the
  unflipped baselines pass against the LANDED kernel pub APIs
  (`cgap_crypto::chacha::chacha20_block` vs RFC 8439 §2.3.2,
  `cgap_crypto::fips202::Shake256::xof` vs FIPS 202 KAT,
  `cgap_crypto::fips203::ntt`/`intt_tomont`/`fqmul` roundtrip), and the
  repo tree is byte-exact restored after all mutations (sandbox-only
  mutation, never the shared tree).

## Gates (commands + exits, this session)

- Run 1 (inside `cargo test -p cgap-crypto`, after BK-3's dbg5 +
  lease.rs unblocks): `test result: ok. 4 passed; 0 failed` (102.08s).
- Run 2: `cargo test -p cgap-crypto --test kat_flip_demo -- --nocapture`
  → `test result: ok. 4 passed; 0 failed` (98.46s); table output:
  `flips_total=324 flips_detected=324 baselines_ok=true restore_ok=true`.
- Zero external deps intact: the demo and sandbox drivers are std-only
  (`rustc --edition=2021` standalone compiles; no dev-dependencies).
- Composes landed AB-2/AB-3/AB-5 kernels via pub APIs (baseline legs),
  lib.rs modules all `pub`.

## Disclosures

- Takeover found the file complete; the only session edits blocking its
  gates were AF-8's inherited blockers (dbg5.rs private-module import;
  AE8 stale-artifact pairing in lease.rs) — both BK-3 fixes, see
  af-8-sandbox-e0583.md.
- `mlkem_sanity::roundtrip_sanity` was FAILING during this lane's runs —
  that surface is BK-1's in-flight fips203/kem fix (dbg10-13 churn
  observed 23:02-23:13); it is not an AF-12 subject and was not touched.
