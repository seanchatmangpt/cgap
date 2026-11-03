//! cgap-core - v26.11.1 moonshot lane scaffold (PROPOSED; see docs/jira/v26.11.1/lanes/).
// Production lib is no_std (host tests stay std via cfg(test)); mirrors the
// cgap-bn3 posture — fixes PUB-07 finding A (95 std-prelude errors on
// riscv32imc). checkpoint.rs / tracking_alloc.rs are #[path]-included
// test-harness modules, not lib members, so they are unaffected.
#![cfg_attr(not(test), no_std)]

pub mod dma_ring;
pub mod frame;
pub mod isomorphism;
