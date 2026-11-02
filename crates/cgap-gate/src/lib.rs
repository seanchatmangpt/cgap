//! cgap-gate - v26.11.1 moonshot lane scaffold (PROPOSED; see docs/jira/v26.11.1/lanes/).
//!
//! no_std (W4-MW2 std-strip): core-only body; the penstock courts module is
//! `#[cfg(test)]` and re-links std there.

#![no_std]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod penstock;
