# KD-1 Version-Refusal Witness Receipt (lane R-KD1-CLOSE)

Standing: CLOSED

Date: 2026-11-06. Lane R-KD1-CLOSE, repo `/Users/sac/cgap`.
Closes divergence registered in `docs/jira/v26.11.4/KNOWN_DIVERGENCES.md` KD-1 and `docs/jira/v26.11.6/open-items.md` OI-3.

## Divergence Closed
- Previously, `AoqError::UnsupportedVersion` projected onto `DIGEST_MISMATCH (0x4347A005)`, creating an unresolvable collision on the wire between a version skew and a payload corruption fault.
- Close condition: Extend the frozen AOQ status band with `UNSUPPORTED_VERSION (0x4347A009)`, update mappings across `aoq_session.rs` and `aoq_sidecar.rs`, and prove discrimination in the `aoq_flip_ladder_court.rs` L2 leg.

## Witness Evidence
- Test file: `crates/cgap-ffi/tests/aoq_flip_ladder_court.rs`
- Court: `l2_version_skew_dies_at_version_law`
- Command: `cargo test -p cgap-ffi --test aoq_flip_ladder_court l2_version_skew_dies_at_version_law`
- Observed: Exit 0. Frame with version skewed 3 -> 2 and recomputed FNV-1a header digest dies with word `0x4347A009` (`AOQ_STATUS_UNSUPPORTED_VERSION`).
- Anti-vacuity witness: Tampered body bytes with valid version die at L1 with `0x4347A005` (`AOQ_STATUS_DIGEST_MISMATCH`), distinguishing both failure modes on the wire.
