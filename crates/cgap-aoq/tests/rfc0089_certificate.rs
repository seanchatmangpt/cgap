//! RFC-0089 §8.4 certificate courts (ZP-21 promotion shim).
//!
//! The certificate implementation moved INTO the lib
//! (`crates/cgap-aoq/src/rfc0089_certificate.rs`, feature `cert`); this
//! test target now INCLUDES that exact source, so the module's embedded
//! courts (5 landed + 1 ZP-21 schema court) run against the same bytes
//! the lib ships — no duplicated body.

#[path = "../src/rfc0089_certificate.rs"]
mod rfc0089_certificate;
