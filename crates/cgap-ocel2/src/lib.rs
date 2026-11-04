//! # cgap-ocel2
//!
//! cgap -> OCEL 2.0 projection membrane (standalone, `crates/cli`
//! pattern: own `[workspace]`, NOT a root member, `publish = false`).
//!
//! ## What this is
//!
//! A typed, refuse-closed projection of the ring-commit receipt chain
//! (`cgap_ffi::receipt_chain::RingReceiptChain`) into three OCEL 2.0
//! tables, emitted as real `cgap-arrow` [`Batch`] columnar buffers plus
//! a documented SQLite/DuckDB view DDL (`view` module) so pm4py can
//! consume the result.
//!
//! ## Mapping (approved sketch)
//!
//! - **Events** — one event per chain link. `event_id` is the chain
//!   ordinal (the receipt's zero-gap `index`). IT IS NOT A WALL CLOCK:
//!   there is deliberately no `ocel:timestamp` column and no
//!   timestamp substitution; the ordinal is a logical sequence number.
//!   Downstream tooling must not read it as time (see `view::DDL`).
//!   `activity` is the transition type (the verified link append,
//!   `"receipt.append"`; on refusal, no event row is ever emitted).
//!   Per-event receipt digest = the link hash `BLAKE3(prev_hash ||
//!   block bytes || license bytes)`.
//! - **Objects** — witness identities: lease ids and file identities
//!   under the `(dev, ino)` keying shape
//!   (`crates/cgap-git/src/ref_store.rs` register/journal shapes
//!   motivated the two-wide integer key; the adapter does NOT mint
//!   filesystem stats itself). A block-derived fallback object
//!   (`block:<hex8>` of the block digest) exists so E2O stays total
//!   even with no witness supplied.
//! - **E2O edges** — `(event_id, object_id, qualifier)` per event.
//!
//! ## Refusal law (the falsifier)
//!
//! `Ocel2Projector::project` runs the REAL `RingReceiptChain::verify`
//! over the live committed bytes FIRST. Any tamper — a flipped block
//! byte, a mutated stored receipt field, a post-hoc license change —
//! collapses to the typed [`Ocel2Refusal::Spoliation`] refusal citing
//! the chain invariant, and NO tables are emitted. Projection is
//! refuse-closed: it never emits rows for a chain it could not
//! verify. Anti-vacuity witness: removing the verify call in a /tmp
//! sandbox copy makes the tamper court FAIL (the projection accepts
//! corrupted input) — a projection that cannot flip is FAIL.
//!
//! ## HONEST SCOPING (mandatory, do not soften)
//!
//! 1. PROJECTION RATE IS A TARGET. No bench exists for this adapter.
//!    The receipted 82.24 GB/s figure is BN3-internal throughput
//!    (a different instrument, different subject); it says NOTHING
//!    about this projection's rate. Any per-sentence-per-second or
//!    GB/s claim here is UNVERIFIED until a bench lands.
//! 2. COMMIT-LEG RECEIPT WIRING IS UNSUPPORTED UPSTREAM. FIX-08
//!    declined to wire `RingReceiptChain` into `cgap_commit_parquet`
//!    (receipt_chain.rs commit-leg boundary note). Therefore "every
//!    event cryptographically backed" holds ONLY for chain-covered
//!    events — events minted through `RingReceiptChain::append` and
//!    verified through `verify`. The commit leg itself has no receipt
//!    surface; events projected from it would be unbacked.
//! 3. The link law is consumed from the public `ChainReceipt` fields
//!    (all `pub`); no private API was reached, no decode re-derived.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

use cgap_arrow::batch::{Batch, BatchBuilder, ColumnBuilder};
use cgap_arrow::primitive::PrimitiveBuilder;
use cgap_arrow::varlen::VarlenBuilder;
use cgap_ffi::receipt_chain::{ReceiptChainRefusal, RingReceiptChain};
use cgap_p28::attestation::P28LicenseAttestation;
use cgap_parquet::footer::SealedBlock;

pub mod tap;
pub mod view;

/// The activity label for a verified chain-link append (the only
/// transition type a verified chain can carry; refusals emit nothing).
pub const ACTIVITY_APPEND: &str = "receipt.append";

/// Witness identity under the `(dev, ino)` keying shape. The adapter
/// never stats a filesystem; the caller supplies real identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileIdentity {
    /// st_dev of the witnessing file.
    pub dev: u64,
    /// st_ino of the witnessing file.
    pub ino: u64,
}

/// Per-event object witness: a lease id plus an optional file
/// identity. Length must match the chain (typed refusal otherwise).
#[derive(Clone, Copy, Debug)]
pub struct LeaseWitness {
    /// The ring-commit lease id for this transition.
    pub lease_id: u64,
    /// Optional file identity witnessed at commit time.
    pub file: Option<FileIdentity>,
}

/// The projection refusal type. `Spoliation` is the ONE opaque
/// tamper class, mapped 1:1 from the chain's own single opaque
/// refusal (no failure fingerprinting, p28 SS5 law).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ocel2Refusal {
    /// Chain empty — nothing to project (shape verdict).
    EmptyChain,
    /// Chain invariant violated (tampered block bytes, receipt
    /// field, license, or position linkage). Fail-closed.
    Spoliation,
    /// `witnesses.len() != chain.len()`.
    WitnessCountMismatch,
    /// Emission invariant broken (column arity/length mismatch). Not
    /// reachable on verified input; fail-closed if it ever is.
    TableShape,
}

impl core::fmt::Display for Ocel2Refusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyChain => {
                f.write_str("receipt chain empty: nothing to project")
            }
            Self::Spoliation => {
                f.write_str("projection refused: receipt chain invariant violated (spoliation, fail-closed)")
            }
            Self::WitnessCountMismatch => {
                f.write_str("witness count does not match chain length")
            }
            Self::TableShape => {
                f.write_str("projected table shape invariant broken")
            }
        }
    }
}

/// The three projected OCEL 2.0 tables as Arrow-shaped batches.
pub struct Ocel2Tables {
    /// Events table: `event_id` (chain ordinal, NOT wall clock),
    /// `activity`, `receipt_digest`, `block_digest`, `prev_hash`.
    pub events: Batch,
    /// Objects table: witness `(dev, ino)` identities + lease ids,
    /// plus the block-derived fallback object per event.
    pub objects: Batch,
    /// E2O edges: `(event_id, object_id, qualifier)`.
    pub e2o: Batch,
}

/// Lowercase hex of a 32-byte digest.
fn hex32(d: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in d {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// `object_id` for a witness file identity: the `(dev, ino)` key.
fn file_object_id(id: &FileIdentity) -> String {
    format!("file:{}:{}", id.dev, id.ino)
}

/// Fallback object id derived from the committed block digest.
fn block_object_id(block_digest: &[u8; 32]) -> String {
    let h = hex32(block_digest);
    format!("block:{}", &h[..8])
}

/// The projector. Static facade over the real chain verify; every
/// emission is behind the verify gate.
pub struct Ocel2Projector;

impl Ocel2Projector {
    /// Project a VERIFIED receipt chain into the three OCEL 2.0
    /// tables. Order of gates:
    /// 1. witness arity (typed refusal, cheap first),
    /// 2. `RingReceiptChain::verify(blocks, license)` over the LIVE
    ///    committed bytes — any tamper is `Spoliation` and NOTHING is
    ///    emitted,
    /// 3. columnar emission (pure projection of verified state).
    ///
    /// # Errors
    /// [`Ocel2Refusal`] — see the refusal law in the crate docs.
    pub fn project(
        chain: &RingReceiptChain,
        blocks: &[&SealedBlock],
        license: &P28LicenseAttestation,
        witnesses: &[LeaseWitness],
    ) -> Result<Ocel2Tables, Ocel2Refusal> {
        if witnesses.len() != chain.len() {
            return Err(Ocel2Refusal::WitnessCountMismatch);
        }
        // THE VERIFY GATE (the falsifier's hinge). Mapping is 1:1:
        // the chain's single opaque refusal class becomes ours.
        match chain.verify(blocks, license) {
            Ok(()) => {}
            Err(ReceiptChainRefusal::EmptyChain) => {
                return Err(Ocel2Refusal::EmptyChain);
            }
            Err(ReceiptChainRefusal::Spoliation) => {
                return Err(Ocel2Refusal::Spoliation);
            }
        }
        Self::emit(chain, blocks, witnesses)
    }

    /// Pure emission over already-verified state.
    fn emit(
        chain: &RingReceiptChain,
        blocks: &[&SealedBlock],
        witnesses: &[LeaseWitness],
    ) -> Result<Ocel2Tables, Ocel2Refusal> {
        let receipts = chain.receipts();
        let mut ev = BatchBuilder::new()
            .with_column("event_id", ColumnBuilder::UInt64(PrimitiveBuilder::<u64>::new()))
            .with_column("activity", ColumnBuilder::Utf8(VarlenBuilder::<u32>::new()))
            .with_column("receipt_digest", ColumnBuilder::Utf8(VarlenBuilder::<u32>::new()))
            .with_column("block_digest", ColumnBuilder::Utf8(VarlenBuilder::<u32>::new()))
            .with_column("prev_hash", ColumnBuilder::Utf8(VarlenBuilder::<u32>::new()));
        let mut ob = BatchBuilder::new()
            .with_column("object_id", ColumnBuilder::Utf8(VarlenBuilder::<u32>::new()))
            .with_column("object_type", ColumnBuilder::Utf8(VarlenBuilder::<u32>::new()))
            .with_column("dev", ColumnBuilder::UInt64(PrimitiveBuilder::<u64>::new()))
            .with_column("ino", ColumnBuilder::UInt64(PrimitiveBuilder::<u64>::new()))
            .with_column("lease_id", ColumnBuilder::UInt64(PrimitiveBuilder::<u64>::new()));
        let mut e2o = BatchBuilder::new()
            .with_column("event_id", ColumnBuilder::UInt64(PrimitiveBuilder::<u64>::new()))
            .with_column("object_id", ColumnBuilder::Utf8(VarlenBuilder::<u32>::new()))
            .with_column("qualifier", ColumnBuilder::Utf8(VarlenBuilder::<u32>::new()));
        for (i, r) in receipts.iter().enumerate() {
            let block_digest = hex32(&r.block_digest);
            let push_utf8 = |b: &mut BatchBuilder, col: &str, v: &str| {
                if let Some(ColumnBuilder::Utf8(vb)) = b.column_mut(col) {
                    vb.push(Some(v.as_bytes()));
                }
            };
            let push_u64 = |b: &mut BatchBuilder, col: &str, v: Option<u64>| {
                if let Some(ColumnBuilder::UInt64(pb)) = b.column_mut(col) {
                    pb.push(v);
                }
            };
            push_u64(&mut ev, "event_id", Some(r.index));
            push_utf8(&mut ev, "activity", ACTIVITY_APPEND);
            push_utf8(&mut ev, "receipt_digest", &hex32(&r.hash));
            push_utf8(&mut ev, "block_digest", &block_digest);
            push_utf8(&mut ev, "prev_hash", &hex32(&r.prev_hash));

            // Fallback object: the committed block itself (total E2O).
            let bid = block_object_id(&r.block_digest);
            push_utf8(&mut ob, "object_id", &bid);
            push_utf8(&mut ob, "object_type", "sealed_block");
            push_u64(&mut ob, "dev", None);
            push_u64(&mut ob, "ino", None);
            push_u64(&mut ob, "lease_id", Some(witnesses[i].lease_id));
            push_u64(&mut e2o, "event_id", Some(r.index));
            push_utf8(&mut e2o, "object_id", &bid);
            push_utf8(&mut e2o, "qualifier", "committed_block");

            // Witness object when a real (dev, ino) identity is supplied.
            if let Some(f) = witnesses[i].file {
                let fid = file_object_id(&f);
                push_utf8(&mut ob, "object_id", &fid);
                push_utf8(&mut ob, "object_type", "file");
                push_u64(&mut ob, "dev", Some(f.dev));
                push_u64(&mut ob, "ino", Some(f.ino));
                push_u64(&mut ob, "lease_id", Some(witnesses[i].lease_id));
                push_u64(&mut e2o, "event_id", Some(r.index));
                push_utf8(&mut e2o, "object_id", &fid);
                push_utf8(&mut e2o, "qualifier", "witness");
            }
        }
        // Belt-and-braces over verified state: every projected block
        // digest must equal the live bytes' BLAKE3. A mismatch here is
        // unreachable after verify; if it ever happens, refuse closed.
        for (i, r) in receipts.iter().enumerate() {
            if r.block_digest != cgap_blake3::hash(blocks[i].bytes()) {
                return Err(Ocel2Refusal::Spoliation);
            }
        }
        Ok(Ocel2Tables {
            events: ev.finish().map_err(|_| Ocel2Refusal::TableShape)?,
            objects: ob.finish().map_err(|_| Ocel2Refusal::TableShape)?,
            e2o: e2o.finish().map_err(|_| Ocel2Refusal::TableShape)?,
        })
    }
}
