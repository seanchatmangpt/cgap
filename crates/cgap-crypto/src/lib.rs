//! cgap-crypto - v26.11.1 moonshot lane scaffold (PROPOSED; see docs/jira/v26.11.1/lanes/).
//!
//! no_std (W4-MW2 std-strip): core+alloc body; std re-links only under
//! `#[cfg(test)]`.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod aead;
pub mod chacha;
pub mod chatman;
pub mod fips202;
pub mod fips203;
pub mod kem;
pub mod mldsa;
pub mod pqc_seal;
pub mod lease;
pub mod poly1305;
