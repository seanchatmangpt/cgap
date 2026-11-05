# W3r — ML-DSA-65 KAT courts resume (eta divergence closure)

Standing: PASS

Subject: /Users/sac/cgap crates/cgap-crypto/src/mldsa.rs @ bdd93af9

## Context

W3's staged eta pack/unpack sign-flip (pack_s nibble = η − z) had been
swept into L36's fmt commit b3fae1f9 — the fix was already on main at
resume. KAT courts were still red on a sixth divergence: RejBoundedPoly
(expand_s_poly) stored negative coefficients in mod-q representation
(c + Q, witnessed as 8380413..8380416 in the court diff) rather than
centered η − nib ∈ [−4, 4]. pack_s requires centered z.

## Diff

crates/cgap-crypto/src/mldsa.rs expand_s_poly: dropped `c + neg*Q` in
both nibble lanes; coefficients now stored centered per FIPS 204
Algorithm 15.

## Commands / exits

- cargo test -p cgap-crypto --test mldsa65_kat_courts
  - BEFORE: FAILED (keygen_kat_seed_expansion_byte_exact_vs_nist_acvp
    vector 0 "s1 != NIST"; nist_sk_parse_matches_seed_law — ExpandS
    returned q−4..q−1)
  - AFTER: ok. 3 passed; 0 failed — exit 0
- cargo test -p cgap-crypto --test fips203_cbd_court
  → ok. 4 passed; 0 failed — exit 0
- cargo test -p cgap-crypto --test mldsa_ntt_courts
  → ok. 9 passed; 0 failed — exit 0
- cargo test -p cgap-crypto (full): 112 tests, 0 failures, exit 0 —
  includes pqc_seal_courts and the anti-vacuity mutation flip courts
  (meta_court_flip_demo_is_not_vacuous et al., 104s) — no re-pin needed;
  no court added this session, so no new anti-vacuity mutation required.

## Landing shape

FULL. All KAT stages green; nothing UNVERIFIED.

## Lineage

- eta pack/unpack sign flip (pack_s + dk unpack negate): b3fae1f9
- centered-rep fix + this receipt: bdd93af9
- prior divergences: f4af9188, cce95c35

## Push

Pending verification below.
