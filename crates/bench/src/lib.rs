//! Library surface for the `bench` package.
//!
//! ZP-18 Phase C: every timing binary composes
//! `cgap_aoq::profile_guard::assert_release` INLINE as the first statement
//! of `fn main` (wrapper `profile::guard_or_exit` retired 2026-11-04 — the
//! profile-guard census predicate matches the literal
//! `profile_guard::assert_release` string in-file, which the wrapper hid).

//! Tier-2 measured/pinned provenance types (H9): fabrication of a PASS from
//! a pinned constant is unrepresentable — see [`provenance`].
pub mod provenance;
pub use provenance::{Measured, Pinned, RefusalReason, Source, Verdict};
