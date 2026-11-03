//! A3 — RMA / fleet lifecycle state machine (R-A3.2, R-A3.3).
//!
//! Lifecycle: `Commissioned -> Healthy -> Degraded -> Retired`.
//! Every transition is an event carrying typed evidence. Retirement
//! (RMA) is legal ONLY from `Degraded` AND only with a receipt digest
//! (hash-chained into the receipt chain upstream); retiring straight
//! from `Commissioned`, or with a bare "trust me" flag, is a typed
//! refusal. Readiness is a state transition, never a clock expiry.

use core::fmt;

/// Stable per-node identity (R-A3.1 host-side view).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub [u8; 16]);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Receipt digest binding a transition to hash-chained postmortem
/// evidence (R-A6.3). A transition that requires evidence carries a
/// full 32-byte digest, not a bare flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiptDigest(pub [u8; 32]);

impl ReceiptDigest {
    /// Deterministic digest for courts (fnv-1a 32-byte expansion).
    pub fn from_seed(seed: u64) -> Self {
        let mut h = seed.wrapping_mul(0x100000001b3).wrapping_add(0xcbf29ce4);
        let mut d = [0u8; 32];
        for slot in d.iter_mut() {
            h ^= *slot as u64;
            h = h.wrapping_mul(0x100000001b3);
            *slot = (h >> 33) as u8;
        }
        ReceiptDigest(d)
    }
}

/// Node lifecycle states, ordered commissioned -> healthy -> degraded ->
/// retired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeState {
    /// Provisioned, identity receipted, not yet admitted to ingress.
    Commissioned,
    /// Serving ingress.
    Healthy,
    /// Serving degraded; RMA candidate.
    Degraded,
    /// Retired for RMA; must never answer ingress again (R-A3.3).
    Retired,
}

/// Typed transition events. Evidence fields are required per event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transition {
    /// Commissioned -> Healthy. Evidence: device-identity digest minted
    /// at provisioning (R-A3.1).
    Promote {
        /// Device-identity digest minted at provisioning (R-A3.1).
        identity_digest: ReceiptDigest,
    },
    /// Healthy -> Degraded. Evidence: observed fault reason.
    Degrade {
        /// Observed fault code driving the degradation.
        fault_code: u32,
    },
    /// Degraded -> Healthy (operator-diagnosed recovery).
    Recover,
    /// Degraded -> Retired (RMA). Evidence: RMA receipt digest,
    /// hash-chained upstream. NEVER legal from Commissioned/Healthy.
    RmaRetire {
        /// RMA receipt digest, hash-chained upstream (required evidence).
        rma_receipt: ReceiptDigest,
    },
    /// Any -> any without evidence: NEVER legal; exists only so the
    /// type system can refuse it explicitly.
    ForceRetire,
}

/// Typed refusals (fail-closed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FleetRefusal {
    /// RMA retire without a receipt digest.
    MissingRmaEvidence,
    /// Retire attempted from a state that is not Degraded.
    RetireFromWrongState(NodeState),
    /// Node already retired; no further transitions.
    AlreadyRetired,
    /// Unknown node in the fleet register.
    UnknownNode,
}

/// A single fleet node: identity + current lifecycle state.
#[derive(Clone, Debug)]
pub struct FleetNode {
    id: NodeId,
    state: NodeState,
    /// Hash-chained transition log: (seq, transition, digest-of-prev).
    /// Forensics is replay of this log, not memory (R-A6.3 pattern).
    /// Hash-chained event log: (seq, transition, chain tip).
    log: Vec<(u64, Transition, u64)>,
    /// Rolling fnv of the event sequence.
    chain_tip: u64,
}

impl FleetNode {
    /// Mint a node in the Commissioned state.
    pub fn new(id: NodeId) -> Self {
        FleetNode { id, state: NodeState::Commissioned, log: Vec::new(), chain_tip: 0xcbf29ce4_8422_2325 }
    }

    /// Stable identity.
    pub fn id(&self) -> NodeId { self.id }
    /// Current lifecycle state.
    pub fn state(&self) -> NodeState { self.state }
    /// Receipted transition log (forensics is replay).
    pub fn transition_log(&self) -> &[(u64, Transition, u64)] { &self.log }

    /// Apply a transition. Legal transitions mutate final state and
    /// append a hash-chained event; illegal ones return a typed refusal
    /// and mutate nothing.
    pub fn apply(&mut self, t: Transition) -> Result<NodeState, FleetRefusal> {
        if self.state == NodeState::Retired {
            return Err(FleetRefusal::AlreadyRetired);
        }
        // Evidence gate: RMA requires a receipt digest. `ForceRetire`
        // carries none by construction and is refused here.
        if matches!(t, Transition::ForceRetire) {
            return Err(FleetRefusal::MissingRmaEvidence);
        }
        let next = match (&t, self.state) {
            (Transition::Promote { .. }, NodeState::Commissioned) => NodeState::Healthy,
            (Transition::Degrade { .. }, NodeState::Healthy) => NodeState::Degraded,
            (Transition::Recover, NodeState::Degraded) => NodeState::Healthy,
            (Transition::RmaRetire { rma_receipt }, NodeState::Degraded) => {
                // Second evidence gate: a digest of all zeros is a bare
                // flag in disguise; refuse it.
                if rma_receipt.0 == [0u8; 32] {
                    return Err(FleetRefusal::MissingRmaEvidence);
                }
                NodeState::Retired
            }
            // Every other (event, state) pair is illegal, including
            // RmaRetire from Commissioned/Healthy.
            _ => return Err(FleetRefusal::RetireFromWrongState(self.state)),
        };
        self.chain_tip = self
            .chain_tip
            .wrapping_mul(0x100000001b3)
            ^ chain_hash(&t);
        self.log.push((self.log.len() as u64, t, self.chain_tip));
        self.state = next;
        Ok(next)
    }
}

fn chain_hash(t: &Transition) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    let mut feed = |b: u8| {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    };
    match t {
        Transition::Promote { identity_digest } => {
            feed(1);
            identity_digest.0.iter().for_each(|&b| feed(b));
        }
        Transition::Degrade { fault_code } => {
            feed(2);
            fault_code.to_le_bytes().iter().for_each(|&b| feed(b));
        }
        Transition::Recover => feed(3),
        Transition::RmaRetire { rma_receipt } => {
            feed(4);
            rma_receipt.0.iter().for_each(|&b| feed(b));
        }
        Transition::ForceRetire => feed(5),
    }
    h
}

/// Fleet register: device inventory + lifecycle transitions (R-A3.2).
/// Retired nodes leave the ingress-serving set immediately.
#[derive(Clone, Debug, Default)]
pub struct Fleet {
    nodes: std::collections::BTreeMap<[u8; 16], FleetNode>,
}

impl Fleet {
    /// Empty fleet register.
    pub fn new() -> Self { Fleet { nodes: Default::default() } }

    /// Commission a node into the register (initial state Commissioned).
    pub fn commission(&mut self, id: NodeId) -> Result<(), FleetRefusal> {
        use std::collections::btree_map::Entry;
        match self.nodes.entry(id.0) {
            Entry::Occupied(_) => Err(FleetRefusal::AlreadyRetired),
            Entry::Vacant(e) => {
                e.insert(FleetNode::new(id));
                Ok(())
            }
        }
    }

    /// Look up a registered node.
    pub fn node(&self, id: &NodeId) -> Option<&FleetNode> {
        self.nodes.get(&id.0)
    }

    /// Node states that may answer ingress. Retired never included.
    pub fn ingress_eligible(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| n.state() != NodeState::Retired)
            .count()
    }

    /// Apply a transition to a registered node (typed refusals).
    pub fn apply(&mut self, id: &NodeId, t: Transition) -> Result<NodeState, FleetRefusal> {
        match self.nodes.get_mut(&id.0) {
            None => Err(FleetRefusal::UnknownNode),
            Some(n) => n.apply(t),
        }
    }
}
