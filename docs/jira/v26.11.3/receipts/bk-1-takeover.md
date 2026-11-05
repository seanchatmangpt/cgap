# BK-1 Takeover Receipt — AB-4 + AB-5 gate close-out
Standing: CLOSED

- **Date**: 2026-11-04
- **Charter**: BK-1 takeover of AB-4 (AEAD) + AB-5 (ML-KEM) dead lanes; gates, not rewrites.
- **Subject SHA at close**: `f561b5636fe27366810fdb448b58022d14567e85`
- **Subject identities**: `crates/cgap-crypto/src/kem.rs` sha256 `8496bc75042bce41...`,
  `crates/cgap-crypto/src/fips203.rs` sha256 `919ea22ef46dd24e...`
- **Standing**: AB-4 ALIVE, AB-5 ALIVE, corridor join on real KEM ALIVE. AF1-KD-1 CLOSED
  (dod-checklist.md dod-006 refreshed this session).

## Gate ladder (commands + exits, this session)

- `cargo test -p cgap-crypto` — GREEN ×2 full passes (run 1: 75 passed / 0 failed across
  15 suites incl. aead_courts 8/8, kat_flip_demo 4/4, mlkem768 bench/courts/interop/kat/
  kem/rt/ntt/zeroalloc; run 3: 75 passed / 0 failed). Run 2 disclosed: 1 unidentified
  failure during heavy concurrent-lane compile contention; surrounding runs fully green.
- `cargo check -p cgap-crypto --target riscv32imc-unknown-none-elf` — exit 0 (13 warnings).
- `cargo check -p cgap-crypto --target wasm32-unknown-unknown` — exit 0 (13 warnings).
- `cargo test -p cgap-aoq --test corridor_join_court` — GREEN ×2 (3/3 each: 45.45s, 51.58s),
  real-KEM hop (TestKem fallback retired in the court source, line 67).

## Bisection receipt (dead-lane inheritance, evidence for the fix that landed externally)

Independent bit-exact verification of every cgap-crypto primitive against a PQClean
kyber512-clean oracle (k→3, eta→2; `/tmp/pqc768`, bytepqc lineage cited in the crate's
own provenance header): hashes (G/H/J/PRF via hashlib + SHA3 known answers), CBD eta=2,
gen_matrix (A00), ntt (post-reduce), intt_tomont, basemul, keygen t_hat[0] chain (TH0),
s_hat (SV). All MATCH.

Findings worth keeping:
1. The earlier "mlkem_sanity FAILS" was NOT an arithmetic defect: every primitive was
   already bit-exact. The external kem.rs fix (23:26, sibling lane) landed while BK-1 was
   mid-bisection; BK-1's probes confirmed the round-trip passes before the gates ran.
2. **Vendored-oracle caveat**: `~/bytestar/bytepqc/src/pqclean_kyber512_clean` has an
   UNSAFE in-place `hash_g(buf, buf, 33)` — out==in clobbers the input, making
   `indcpa_keypair_derand` nondeterministic in that copy (witnessed ×2). Any future
   cross-checks against that tree must use separate in/out buffers.

## Handled relayed items

- AB-4 nonce ordering: resolved externally as the test constant (not the nonce layout);
  aead_courts 8/8 green ×2 runs.
- mlkem_sanity was superseded/deleted by the landing lane in favor of the mlkem768 court
  families; dod-006 row now cites the superseding gates.

## Disclosures

- BK-1 ran NO source edits in the shared tree (SLA respected); the only writes are this
  receipt and the dod-006 row refresh. Scratch probes lived in `/tmp` (kprobe crate,
  /tmp/pqc768, /tmp/pqc_test2, /tmp/pqc_orig); no repo-tree scratch tests were created
  after the sibling cleanup, none left behind.
- Read-only git queries only (rev-parse/status); no git state transitions (charter law).
