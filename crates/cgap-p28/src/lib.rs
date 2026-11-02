//! cgap-p28 - v26.11.1 moonshot lane scaffold (PROPOSED; see docs/jira/v26.11.1/lanes/).
//!
//! no_std (W4-MW2 std-strip): core-only body on wasm32/bare-metal; std
//! re-links on hosted targets (target_os != "none", non-wasm) so the §6.2
//! spoliation halt keeps its exact SIGABRT edge and the cfg(test) courts
//! keep fs/env/process fixtures. On wasm32 the halt lowers to
//! `core::arch::wasm32::unreachable()` (byte-equivalent to the witnessed
//! std abort lowering); on bare-metal `target_os = "none"` it lowers to a
//! panic — divergence recorded in the W4-MW2 receipt.

#![no_std]

#[cfg(all(not(target_family = "wasm"), not(target_os = "none")))]
#[macro_use]
extern crate std;

#[cfg(target_family = "wasm")]
extern crate alloc;


pub mod attestation;
pub mod trap;
pub mod asc450;
