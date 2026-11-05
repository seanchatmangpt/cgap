# Decision: AAD Ruling (Option b adopted)
Standing: ALIVE

Date: 2026-11-04. Coordinator decision receipt; execution receipt at
`aad-ruling-executed.md` (same directory).

## Ruling

Option (b) ADOPTED: var-AAD (`mac_tag_var_aad`) promoted to production API;
`AAD_LEN = 36` demoted from "fixed AAD width" wire law to a wrapper constant
of `seal_detached`/`open_detached`.

## Evidence (Q3 evidence brief)

- Zero breakage: grep-verified — no consumer depends on the promoted seam.
- Real bound is `u32::MAX` (RFC 8439 uint32 length encoding); the 1MiB-bound
  premise was FALSE on disk.
- `mac_tag_var_aad` is KAT-proven via the RFC 8439 §2.8.2 published vector
  (12-byte AAD), plus a new flip court `var_aad_aad_byte_flip_diverges_tag`
  in `crates/cgap-crypto/tests/aead_courts.rs` as the promoted surface's
  anti-vacuity witness.

## Standing

ALIVE — executed this cycle; see `aad-ruling-executed.md` for the diff and
court witness.
