# ZP-18b: Purity-Gate Registration Receipt (rfc0089_certificate.rs, U2 landing)

Date: 2026-11-04. Scope: docs-only materialization of ZP-18's purity-gate
registration surface for `rfc0089_certificate.rs` in `cgap-aoq`. No lib or
gate code edited by this receipt.

## Subject

- `/Users/sac/cgap/crates/cgap-aoq/tests/purity_gate.rs` (589 lines)
- `/Users/sac/cgap/crates/cgap-aoq/src/rfc0089_certificate.rs` (687 lines,
  registered in lib.rs behind `#[cfg(feature = "cert")]`)

## The 13-entry Registered{name, exempt} table

`tests/purity_gate.rs:38` declares `REGISTERED: [Registered; 13]`:
aead.rs, chain.rs, codec.rs, error.rs, generated_aoq_header_layout.rs,
header.rs, kem.rs, lib.rs, session.rs, profile_guard.rs, verify.rs,
writer.rs, rfc0089_certificate.rs. Twelve entries carry `exempt: &[]`
(the strict default); one carries the reviewed exemption set.

## The 3 token exemptions on rfc0089_certificate.rs

`tests/purity_gate.rs:65-81`: `rfc0089_certificate.rs` is exempted for
exactly three banned tokens, each with the identical 76-character
justification "cert feature is alloc-using by design; default build
remains zero-alloc":

- `alloc::` — the module is `extern crate alloc;` with String/Vec
  certificate types, gated behind the `cert` feature.
- `vec!` — test-gated scorecard/court scaffolding only.
- `std::` — `#[cfg(test)]` host-side scaffolding, never the shipped
  no_std surface.

The default (no-`cert`) build stays core-only and zero-alloc because the
whole module is cfg'd out without `cert` (witness: `cert = []`, no
default features, in Cargo.toml).

## validate_registration law

`tests/purity_gate.rs:93-115`: a registration entry is lawful only if
(1) every exempt token IS a token in the `BANNED` table and (2) every
justification is >= 20 chars. Any violation is collected and REFUSES the
gate (named `INVALID-REGISTRATION`), never silently passed. The main
gate test (`gate_exits_clean_on_registered_sources`, line 277) runs
`validate_registration(&REGISTERED)` as a court surface before scanning.

## Mutation witnesses (anti-vacuity)

1. **D1 — unregistered-alloc-source named fail-closed**
   (`d1_unregistered_alloc_source_is_named_fail_closed`, line 532): a
   /tmp fixture with a registered file plus a stray alloc-using
   `stray_cert.rs` registered NOWHERE must be named by the directory
   walk — the exact red that opened the lane (the gate stayed green
   while rfc0089_certificate.rs was unregistered). Asserts
   `strays == ["stray_cert.rs"]`.
2. **Exemption-attribute flips both ways**
   (`registration_exemption_attribute_flips_both_ways`, line 554): the
   same alloc-using text must FAIL with an empty exempt list (all three
   tokens trip), PASS under REGISTERED[12]'s reviewed exemptions, leave
   `vec!`+`std::` loud under a partial (alloc::-only) exemption, and the
   live registered file must pass under its own row.

## cfg-interaction finding

The loud-accounting walk (`unregistered_sources`, line 253) reads the
directory: it sees on-disk `.rs` files regardless of feature cfg. A
module registered only behind `#[cfg(feature = "cert")]` in lib.rs is
still a physical file under src/ — so it MUST be in the registration
table in BOTH states (registered with exemptions; omitting it fails the
walk whenever the file exists on disk, cfg on or off). Registered in
both states on disk; recorded here as a standing law.

## Suite outcome

`cargo test -p cgap-aoq --test purity_gate` re-run THIS session
(2026-11-04):

- First attempt compile-ABORTED: cgap-blake3/src/lib.rs:1017
  "cannot find macro `vec`" — a transient mid-edit state of the shared
  tree (another lane's in-flight edit to the std-feature path; on-disk
  file now carries `::std::vec!`). Compile-abort, not a test failure;
  disclosed per census law.
- Second attempt: **10 passed; 0 failed** (exit 0), including
  `d1_unregistered_alloc_source_is_named_fail_closed ... ok` and
  `registration_exemption_attribute_flips_both_ways ... ok`.

Standing: ALIVE (command + exit run this session against the exact
subject).

## See Also

`zp-18-profile-guard-census.md`, `zp-17-gate10-settling.md` (same
directory).
