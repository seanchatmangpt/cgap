//! cgap-branchless: centralized branchless ALU / SWAR / bit-plane mechanics
//! for the cgap sovereign substrate (`cgap-gate`, `cgap-p28`, `cgap-arrow`,
//! `cgap-bn3`).
//!
//! Census target: 0 blr / 0 brk in the hot kernels. Lane B1 lands the
//! compiling NAIVE reference implementations; the branchless forms land in
//! lanes B2 (select/arithmetic), B3 (swar), B4 (eytzinger), B5 (radix).
//!
//! - `select`     — conditional move / slice select / predicated increment
//! - `arithmetic` — p28 saturated subtraction with underflow tripwire masks
//! - `swar`       — SWAR byte equality, zero-byte detection, range masks
//! - `eytzinger`  — eytzinger (van Emde Boas) layout step/restore
//! - `radix`      — 12-bit radix bucket fold for partitioned scans
//! - `scan`       — prefix-scan networks (TS2): or/and/xor u64x8, sum u32x8,
//!   segmented sum, max u8x8, count_leading_eq; 0-branch HOT cells
//! - `penstock`   — gate hot-path primitives (prefilter8 mix, SWAR
//!   all-zero variable-length detect; lane REP5)

#![no_std]
#![forbid(unsafe_code)]

pub mod arithmetic;
pub mod eytzinger;
pub mod penstock;
pub mod radix;
pub mod scan;
pub mod select;
pub mod swar;
pub mod transpose;
