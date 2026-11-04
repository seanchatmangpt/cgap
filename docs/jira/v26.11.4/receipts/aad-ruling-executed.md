# AAD Ruling Executed (Option b — var-AAD promoted to production)

Date: 2026-11-04. Lane: IMPLEMENTATION. Ownership held: `aead.rs` +
`aead_courts.rs` only. No git actions.

## The Ruling

Option (b) ADOPTED: promote `mac_tag_var_aad` from `#[doc(hidden)]`
fixture seam to production API. Evidence: Q2 brief — zero breakage
(grep-verified); the 1MiB-bound premise was FALSE on disk (real bound is
`u32::MAX` per RFC 8439's uint32 length encoding); `mac_tag_var_aad` is
KAT-proven via the RFC 8439 §2.8.2 published vector (12-byte AAD).

## Diff

### `crates/cgap-crypto/src/aead.rs` (zero behavioral change; body untouched)

1. Module doc: `AAD_LEN = 36` demoted from "fixed AAD width" wire law to
   WRAPPER constant of `seal_detached`/`open_detached`; the mac_data
   layout authenticates AAD of any length via `mac_tag_var_aad`.
2. `AAD_LEN` const doc: same demotion, wrapper-constant wording.
3. `mac_tag_var_aad`: `#[doc(hidden)]` REMOVED; doc rewritten as
   production API (variable-length AAD, real bound `u32::MAX` practice /
   `u64` in the length encoding, KAT + flip-court provenance named).

### `crates/cgap-crypto/tests/aead_courts.rs`

New court `var_aad_aad_byte_flip_diverges_tag` (anti-vacuity for the
promoted surface): over AAD length classes 0/12/15/16/17/36, flipping ONE
absorbed AAD byte at first/mid/last positions MUST diverge the tag from
the byte-exact deterministic baseline; the RFC §2.8.2 published
12-byte-AAD vector is KAT-anchored, and a flip on that AAD must diverge
the published tag `1ae10b59...0600691`. Kills any mac_data change that
stops absorbing AAD.

## Commands + Exits (this session)

```
cargo test -p cgap-crypto --test aead_courts   # exit 0 — 9 passed, 0 failed
cargo build -p cgap-crypto                     # exit 0 — zero warnings
cargo build -p cgap-aoq                        # exit 0 — facade consumer green
```

Test run (exact): `test result: ok. 9 passed; 0 failed; 0 ignored; 0
measured; 0 filtered out`. All pre-existing courts green — including
`aead_rfc8439_2_8_2_fixture` (KAT byte-identical post-promotion), the
36B-wrapper courts (`flip_kills_are_mismatch`, `length_sweep_round_trips`,
`cross_check_rustcrypto_byte_identical_both_directions`), and the
count-law scan (restore witness intact).

## Standing

ALIVE — all three commands executed against the exact subject this
session, exits captured un-piped. Existing 36B wrapper surface unchanged
(`seal_detached`/`open_detached` signatures and bodies untouched);
`cgap-aoq` builds clean, confirming the facade consumer is unaffected.
