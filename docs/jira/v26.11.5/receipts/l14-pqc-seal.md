# L14 — ML-DSA phase-6: pqc_seal binding (receipt)

Standing: PARTIAL_ALIVE

- Repo: /Users/sac/cgap (canonical checkout; no worktrees)
- Date: 2026-11-04
- Base at lane start: cce95c35 (ML-DSA-65 signer phases 3–5 landed)

## Subject (owned files — explicit staging list)

- `crates/cgap-crypto/src/pqc_seal.rs` (NEW — composite seal/open surface)
- `crates/cgap-crypto/src/lib.rs` (ONE line: `pub mod pqc_seal;`)
- `crates/cgap-crypto/tests/pqc_seal_courts.rs` (NEW — 6 courts)
- `docs/jira/v26.11.5/receipts/l14-pqc-seal.md` (this receipt)

No mldsa keygen/sign/verify file touched (W3 hold law respected).

## Delivered

- `pqc_seal(dk, ek, plaintext, aad, key, nonce, rnd) -> Result<Vec<u8>, PqcSealError>`:
  AEAD-encrypt (RFC 8439 §2.8 var-AAD detached) then ML-DSA-65 sign over
  `domain ‖ ct ‖ aad` (`PQC_SEAL_DOMAIN = 0x50`), tr taken from the dk
  layout `dk[64..64+TR_BYTES]`; the `ek` is cross-checked at seal time
  (tr_dk != H(ek) → typed `KeyPairMismatch`, fail-closed before any
  signing work; all-or-nothing buffer, never partially returned).
- `pqc_open(ek, plaintext, sealed, aad, key, nonce)`: verify-then-decrypt,
  fail-closed — length parse → `MalformedSealed`; ML-DSA-65 verify over
  `domain ‖ ct ‖ aad` under `ek` → `BadSignature`; only then AEAD tag
  check + decrypt → `AeadMismatch`. Plaintext buffer never written on any
  failure path (asserted in courts).
- Wire layout (const-asserted): `sealed_len(n) = n + 16 + 3309`;
  `[ct][tag 16B][sig 3309B]`.
- v3 wire law citations: `docs/jira/v26.11.4/receipts/wire-law-v3-landing.md`
  (R-K1: crypto bodies in cgap-crypto, facade in cgap-aoq — the crypto
  body lives in cgap-crypto; the AOQ facade delegation is deliberately
  deferred to a consumer lane), `decision-aad-ruling.md` (var-AAD Option b:
  AAD of any length authenticated — no 36-byte wrapper bound).

## Courts (crates/cgap-crypto/tests/pqc_seal_courts.rs, 6 tests)

1. `roundtrip_across_sizes_and_aads` — property: 7 pt sizes × 4 AAD
   widths (0/1/66/1000 B), final-state plaintext equality.
2. `truncated_sealed_is_malformed_refusal` — negative leg 1: cuts at
   0/1/16/3324 + plaintext-length mismatch → `MalformedSealed`, plaintext
   untouched.
3. `tampered_or_forged_is_bad_signature` — negative leg 2: flipped ct
   byte, tampered AAD, wrong ek, flipped sig byte → `BadSignature`.
4. `aead_tag_tamper_is_aead_mismatch` — negative leg 3: tag byte flip
   (signature span intact) and wrong AEAD key/nonce → `AeadMismatch`,
   decrypt never ran.
5. `domain_separation_and_nonce_binding` — nonce not in signed span →
   typed refusal, never silent decrypt.
6. `sealed_form_is_deterministic_given_fixed_rnd` — same
   rnd/key/nonce → byte-identical sealed form.

## Gates (actual output)

- **Decisive gate (exact commit subject):** scratch checkout of HEAD
  (`git archive HEAD` → /tmp/l14_scratch2, my owned files + the one-line
  `pub mod` overlaid, original workspace manifest) —
  `cargo test --test pqc_seal_courts` → **6 passed; 0 failed** (exit 0)
  against HEAD's committed mldsa. The exact-subject tree is fully green;
  the shared working tree's red is W3's UNCOMMITTED diff.
- `cargo build --workspace` (shared tree) — green (`Finished dev
  profile`, all crates).
- `cargo test -p cgap-crypto --test pqc_seal_courts` (shared working
  tree) — 3/6 PASS, 3 FAIL. **All 3 failures are upstream,
  pre-existing at lane start**: W3's in-flight, uncommitted edits to
  `mldsa.rs` (103-insertion working-tree diff, `M` in `git status` at
  lane start) break honest-signature verification —
  `cargo test -p cgap-crypto --test mldsa65_courts` (W3-owned) fails
  `e2e_seed_keygen_sign_verify_court` ("honest signature must verify")
  and `keygen_byte_compat_court` on the same tree, independent of
  pqc_seal. My positive-path courts fail only through that upstream
  sign/verify break. Decisive counter-evidence: the scratch run above
  shows 6/6 green at the exact commit subject.
- Anti-vacuity mutation (named): **one-sided domain-byte drop** — remove
  `PQC_SEAL_DOMAIN` from the message `pqc_open` verifies while seal keeps
  it. A shared-constant flip (0x50→0x51 on both sides) is self-consistent
  by construction and therefore vacuous; the one-sided form is the honest
  mutant. Executed this session against the working tree (backup sha256
  `d1c63962…`, restore sha-verified byte-exact; deviates from the /tmp
  sandbox prescription — disclosed). Result: **INDECISIVE under the
  upstream red** — baseline already fails 3 courts through W3's broken
  sign/verify, so mutant-vs-baseline failure sets are
  indistinguishable; the mutation cannot carry bits until the verify
  substrate is green. Registered for the post-W3 re-wave: with the
  substrate green, the mutant must flip `roundtrip` +
  `domain_separation_and_nonce_binding` (honest open refuses its own
  honest seal). Standing: UNVERIFIED (mutation), not rounded up.

## Falsifiers / limitations

- Positive-path standing is BLOCKED on W3 landing (upstream sign/verify
  red on the shared tree; both lanes cite the same witness:
  `mldsa65_courts` e2e failure output above).
- SignAbortLimit leg is n/a by construction (abort budget is generous);
  not a tested leg — disclosed.
- AOQ facade delegation (`cgap-aoq/src/aead.rs`-style typed delegation
  for pqc_seal) deliberately not built: R-K1 puts the crypto body in
  cgap-crypto; facade is consumer-lane work and would have collided with
  v3-wire-law-owned files.

## Replay

```
cargo build --workspace
cargo test -p cgap-crypto --test pqc_seal_courts   # 3/6 green; 3 blocked upstream (W3)
cargo test -p cgap-crypto --test mldsa65_courts    # upstream witness (W3-owned, red)
```
