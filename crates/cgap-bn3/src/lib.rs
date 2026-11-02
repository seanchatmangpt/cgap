//! cgap-bn3 - v26.11.1 moonshot lane scaffold (PROPOSED; see docs/jira/v26.11.1/lanes/).
// Production lib is no_std (host tests stay std via cfg(test)); W6-BN3NS.
#![cfg_attr(not(test), no_std)]

pub mod arena;
pub mod builtins;
pub mod noalloc;
pub mod project;
pub mod sigil;
pub mod unify;
