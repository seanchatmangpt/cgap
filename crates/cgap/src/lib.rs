//! cgap umbrella facade: one dependency for the published cgap kernel crates.
//!
//! Re-exports the published surfaces verbatim — this crate carries no logic
//! of its own (zero extra abstraction over the kernels it names).
//!
//! SP1 lane (v26.11.2 convergence): adds the enclave self-play pipeline
//! wiring ([`pipeline`]) on the same no_std posture.
#![no_std]

extern crate alloc;

pub mod pipeline;
pub use pipeline::{EnclavePipeline, PipelineError, PipelineStage, StageHandle};

pub use cgap_arrow;
pub use cgap_blake3;
/// HOST-ONLY facade re-exports (feature `host`): cgap-core / cgap-storage
/// are not no_std-clean; compiled OUT of every none-target image (SP1
/// feature decision, host-only stage list in the lane manifest).
#[cfg(feature = "host")]
pub use cgap_core::frame;
pub use cgap_git;
pub use cgap_parquet;
#[cfg(feature = "host")]
pub use cgap_storage::dred;
