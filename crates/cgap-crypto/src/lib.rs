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

pub mod chatman;
pub mod lease;
