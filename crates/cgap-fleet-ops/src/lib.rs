#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! cgap-fleet-ops — host-side fleet membrane (A3 RMA lifecycle, A6 alerting).
//!
//! Implements FIX-14 ops register requirements R-A3.2/R-A3.3 and
//! R-A6.2: a typed fleet-node lifecycle state machine where retirement
//! is a receipted transition (RMA requires a receipt digest, not a bare
//! flag), plus a bounded-poll SLO snapshot poller — the kernel exposes
//! state, the host decides. No sleeps anywhere; readiness is a typed
//! state transition on an event.

pub mod fleet;
pub mod slo;

pub use fleet::{Fleet, FleetRefusal, NodeId, NodeState, ReceiptDigest, Transition};
pub use slo::{Alert, AlertPredicate, SloSnapshot, SloSource, poll_until};
