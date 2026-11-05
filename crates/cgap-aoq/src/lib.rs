//! cgap-aoq — AoQ (Append-only Queue) kernel.
//!
//! Registration history (disclosed per fanout SLA law):
//! - `lib.rs` was absent at AB-7 lane start (AB-1 skeleton ownership);
//!   AB-7 created it minimally (error only), then AB-6 landed
//!   header.rs/codec.rs/generated_aoq_header_layout.rs alongside.
//! - Reconciled by AB-7 (this revision): ALL landed modules registered —
//!   AB-1 supersedes at integration.

#![no_std]
#![forbid(unsafe_code)]

// [2026-11-03 AB-7] chain.rs header-agnostic rewrite (R-H1) landed;
// re-registered by AB-7. Disclosed in ab-7-chain-session.md.
pub mod aead;
pub mod chain;
pub mod codec;
pub mod error;
pub mod generated_aoq_header_layout;
pub mod header;
pub mod kem;
// session.rs landed (AB-7, 21:58); registration restored by AB-1.
pub mod session;
// Q6 M2b (P-f scope): peer identity pin-table over the fp/2 fingerprint
// image; TOFU / Pinned / typed fail-closed refusal, no new deps.
pub mod pin_table;
// RX-4: reusable release-profile detector + anti-DCE fence (AF-25 axis 2
// loud-SKIP pattern + AF-10 G1b black_box, generalized).
pub mod profile_guard;
// AB-8 (PG-A2 wire seam): window verification + 5-segment scatter writer.
pub mod verify;
pub mod writer;
// ZP-21 (RFC-0089 §8.4): CBMQS qualification certificate types +
// canonical JSON emitters + issue/verify law, behind the `cert`
// feature. Zero dependencies; the BLAKE3 substrate is injected
// (`*_with` seam forms). alloc-using under the feature; the default
// embedded surface stays no-dep / no-alloc.
#[cfg(feature = "cert")]
pub mod rfc0089_certificate;
// L56 (v26.11.5): PQC composite-seal facade — typed delegation to
// cgap-crypto's pqc_seal kernel (R-K1: the crypto body stays in
// cgap-crypto; cgap-aoq hosts the facade only, zero crypto logic here).
// `pqc_seal` returns an owned buffer, so this surface is alloc-using and
// rides the `pqc` feature (same gating precedent as `cert`); the default
// embedded surface stays no-alloc / no_std.
#[cfg(feature = "pqc")]
extern crate alloc;
#[cfg(feature = "pqc")]
pub mod pqc_seal;
