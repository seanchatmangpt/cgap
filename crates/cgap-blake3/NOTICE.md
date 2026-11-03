# NOTICE — cgap-blake3

Crate-level notice for `crates/cgap-blake3`. The authoritative, full
upstream-attribution and clean-room record is
[`docs/NOTICE-blake3.md`](../../docs/NOTICE-blake3.md); this file only
points at it and carries the publish-relevant facts inline.

## Upstream attribution

This crate is an INDEPENDENT REIMPLEMENTATION of the BLAKE3
cryptographic hash function from the published specification. It is NOT
vendored, copied, or translated upstream reference code (evidence and
on-disk corroboration: `docs/NOTICE-blake3.md` §1, §3).

The BLAKE3 specification originates from the BLAKE3 project
(Aumasson, Neves, Wilcox-O'Hearn, Winstanley). Upstream reference code
is dedicated to the public domain with an Apache-2.0 option as
documented by the upstream project; because no upstream tree is vendored
here, the exact upstream license file bytes are **UNVERIFIED** from
disk (FIX-15 marker). No upstream license fact beyond that documented
posture is asserted.

## Vendoring rule

Any future vendoring of BLAKE3 upstream code MUST add the verbatim
upstream license text and NOTICE here and record its sha256 pin before
merge (FIX-15). Vendored-tree style precedent:
`vendor/nanoarrow/NOTICE.txt`.

## License of this crate

Distributed under the repository instrument P28-SIL v2.1.0
(`urn:avec:license:p28-sil:v2.1.0-sovereign`), declared via
`license-file = "../../LICENSE.md"` in this crate's `Cargo.toml`.
