# Receipt: AF-1 Corridor Join Court (Systemic Integration Gate)

- **Lane**: AF-1, coordinator-dispatched.
- **Date**: 2026-11-03
- **Subject**: `crates/cgap-aoq/tests/corridor_join_court.rs` (new, ~470 lines),
  `crates/cgap-aoq/Cargo.toml` (+3 lines dev-dep, disclosed below),
  this receipt.
- **Standing**: ALIVE for the compiled subset; one typed BLOCKED row (KEM).
  All gates run this session, exit 0.

## Delivered

The DFLSS dossier's end-to-end JOIN, as three courts in
`corridor_join_court.rs`:

- **J1 corridor join (100 iterations)**: real ArrowArray (cgap-arrow
  `c_abi::export` — live `ArrowArray` with wired release fn, released
  exactly once) -> ML-KEM key hop -> ChaCha20-Poly1305 detached seal (key
  from the KEM shared secret via `session::derive(CTX_HANDSHAKE, ss, ek)`)
  -> AoqWriter six-segment vectored scatter over a LIVE socketpair UDS
  (writev, ZeroProgress asserted, commit only after full accept) -> live
  `read_exact` -> `verify_window` (receiver recomputes the tamper-tier
  wire tag `blake3(body)[..16]` from RECEIVED bytes — the gate that refuses
  a flipped ciphertext byte) -> `decrypt_in_place` (ChaCha20 keystream,
  counter start 1, matching seal's `keystream_xor`) -> `chain.append`
  (payload_digest composed over the DECRYPTED plaintext) with
  per-iteration `verify_all` + `verify_span`. Per iteration: byte-identity
  (decrypted == pre-seal array bytes), cursor/chain lockstep
  (ordinal == chain index == seq), 100 links, head != GENESIS.
- **J2 anti-vacuity battery** (6 gates, each observed firing): ct-byte
  flip -> `DigestMismatch`; ordinal regression -> `FrameOrdinalRegression`
  and seq drift -> `SeqGap`; wrong tag -> `DigestMismatch`; tampered AEAD
  tag -> `open_detached` `Mismatch`; tampered chain COPY -> typed refusal
  (chain itself append-only); short keystream -> `HeaderTooShort` with the
  window bytes proven untouched (no-plaintext-on-error).
- **J3 100k-link scale leg**: 100k appends, `verify_all`, prefix/suffix/
  middle span verifies, resumption token at head, tampered-copy refusal
  at link 77777, originals still verify.

## Typed BLOCKED row (KD register)

| id | hop | divergence | evidence | close condition |
|---|---| court | | |
|---|---|---|---|---|
| AF1-KD-1 | KEM key hop (AB-5) | cgap-crypto ML-KEM-768 encaps/decaps shared-secret DISAGREEMENT — `cargo test -p cgap-crypto --test mlkem_sanity` FAILS at HEAD (`round-trip failed`, ss mismatch). Note: cgap-crypto is mid-flight (AB-5 rewrote poly1305 `absorb_block` to 3 args mid-session, froze the shared lib ~75 s; fips203 `decaps` may also be affected). | `cargo test -p cgap-crypto --test mlkem_sanity` exit 1, this session | rebind `TestKem` -> real ML-KEM behind `KemSeam` and rerun the court green ×2 |

The corridor therefore executes the key hop over the compiling subset: a
deterministic test-KEM (dk IS ek, session.rs inline-test construction)
disclosed in the court header and here. All other hops are REAL kernels:
ChaCha20-Poly1305 (RFC 8439-vector-verified in AB-3), Shake256 (FIPS 202
KAT-verified in AB-2), blake3 (cgap-arrow), ArrowArray c_abi (cgap-arrow
courts), AoqWriter/verify_window/decrypt_in_place/chain (cgap-aoq) over a
live UDS socketpair.

## Dev-dep disclosure

`crates/cgap-aoq/Cargo.toml` gained `cgap-crypto = { path = "../cgap-crypto" }`
(dev-dep) — required to bind the REAL kernels to the R-K1 host seams.
AB-8's sweep/soak and AF-10's wire-shm dev-dep were already in place; no
other lane's entry was touched. Also disclosed: two shared-tree freezes
from concurrent lanes were absorbed by bounded polls (~2 min AF-14
cbmqs_runner manifest entry before its file landed; ~75 s AB-4/AB-5
poly1305 rewrite), per the compile-freeze SLA — no unblock edits applied.

## Verification ladder (all run this session, exact subject)

| Gate | Command | Result |
|---|---|---|
| Corridor gate run 1 | `cargo test -p cgap-aoq --test corridor_join_court` | 3 passed / 0 failed (25.8 s) |
| Corridor gate run 2 | same | 3 passed / 0 failed (28.2 s) |
| Regression sweep | `cargo test -p cgap-aoq --test {ab8_courts,ab8_sweep,ab8_soak,aoq1_bench_court,aoq1_codec_court,aoq1_layout_drift_court,aoq1_stress_court,chain_courts,chain_bench,purity_gate,corridor_join_court}` | all `ok` (14+2+1+1+7+6+2+7+1+8+3 tests, 0 failures) |
| Pre-existing failure (NOT mine, AB-5-owned) | `cargo test -p cgap-crypto --test mlkem_sanity` | exit 1, `round-trip failed` — recorded as AF1-KD-1 |

Note: full `cargo test -p cgap-aoq` (workspace default target set) is
currently frozen by AF-14's in-flight `cbmqs_runner` harness (missing
main, mid-flight, not this lane); the per-target sweep above is the
regression evidence.

## Falsifier

A corridor hop that decrypts to something other than the pre-seal array
bytes, or a tamper battery whose gates do not fire, makes J1/J2 red; a
100k chain that fails `verify_all` or accepts a tampered copy makes J3
red. AF1-KD-1 closes only when the real ML-KEM round-trip goes green and
the corridor re-runs green ×2 with the real KEM bound.
