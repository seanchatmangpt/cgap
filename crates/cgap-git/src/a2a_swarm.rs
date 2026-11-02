//! Agent-to-agent swarm lease table (lane GIT-15).
//!
//! Operator lane-lease doctrine (disjoint file/coordinate ownership)
//! rendered as coordinate bitmasks over a fixed-capacity agent table.
//! Each agent lease is a `[u64; COORD_WORDS]` zone envelope granted at
//! `refs/agents/<agent-id>` registration; commit proposals carry the
//! coordinates they touch and any intersection with a foreign active
//! lease is a typed `AndonRefusal`.
//!
//! Concurrency model (stated honestly): single-threaded court,
//! single-writer-per-agent. The ref cells are plain memory because the
//! zone + namespace checks make cross-agent writes a typed refusal
//! before any cell is touched; `core::sync::atomic` would claim SMP
//! semantics this court never exercised, so it is NOT used. Zero heap:
//! every structure is a fixed-size array sized by `const`.

/// Hard ceiling on registered agents (fixed table, no heap).
pub const MAX_AGENTS: usize = 32;
/// Coordinate-space width: 4 u64 words = 256 zoned coordinates.
pub const COORD_WORDS: usize = 4;
/// Byte width of a per-agent head ref (SHA-1 oid).
pub const REF_LEN: usize = 20;

/// Opaque agent identity (registered id; `refs/agents/<id>` key).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentId(pub u64);

/// A granted coordinate zone: `COORD_WORDS` bitmask words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneMask(pub [u64; COORD_WORDS]);

impl ZoneMask {
    pub const EMPTY: ZoneMask = ZoneMask([0u64; COORD_WORDS]);

    /// Word `w` gets bit `b` set (coordinate `w * 64 + b`).
    pub const fn coord(word: usize, bit: u32) -> ZoneMask {
        let mut words = [0u64; COORD_WORDS];
        words[word] = 1u64 << bit;
        ZoneMask(words)
    }

    /// Bitwise AND across envelopes: non-zero anywhere = overlap.
    pub const fn intersects(&self, other: &ZoneMask) -> bool {
        let mut i = 0;
        while i < COORD_WORDS {
            if self.0[i] & other.0[i] != 0 {
                return true;
            }
            i += 1;
        }
        false
    }
}

/// One lease slot in the fixed table.
#[derive(Debug, Clone, Copy)]
pub struct LeaseSlot {
    pub agent: AgentId,
    pub zone: ZoneMask,
    /// Plain (non-atomic) per-agent head ref: single-writer-per-agent.
    pub head: [u8; REF_LEN],
    pub active: bool,
    pub revoked: bool,
}

impl LeaseSlot {
    const VACANT: LeaseSlot = LeaseSlot {
        agent: AgentId(0),
        zone: ZoneMask::EMPTY,
        head: [0u8; REF_LEN],
        active: false,
        revoked: false,
    };
}

/// Typed refusals. `ForeignZone` / `ForeignRefPath` are the
/// AndonRefusal-class outcomes: a proposal that would cross another
/// agent's lease is refused before any state mutates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmError {
    /// Coordinate zone intersects a foreign active lease (Andon class).
    ForeignZone { holder: AgentId },
    /// Proposal targets another agent's ref path (Andon class).
    ForeignRefPath { owner: AgentId },
    /// Id not in the table.
    UnknownAgent(AgentId),
    /// Id's lease was revoked; further pushes are refused.
    LeaseRevoked(AgentId),
    /// Registration overlaps an existing active lease.
    ZoneConflict { holder: AgentId },
    /// Table at MAX_AGENTS capacity.
    TableFull,
    /// CAS expected-ref mismatch.
    CasMismatch {
        expected: [u8; REF_LEN],
        actual: [u8; REF_LEN],
    },
    /// Duplicate registration of a live agent id.
    DuplicateAgent(AgentId),
    /// Revoke of an id that is not currently active.
    NotActive(AgentId),
}

/// Fixed-capacity agent lease table. Zero heap, no atomics.
#[derive(Debug, Clone, Copy)]
pub struct LeaseTable {
    slots: [LeaseSlot; MAX_AGENTS],
    /// Revocation tombstones: survive slot reuse (fixed capacity).
    tombstones: [AgentId; MAX_AGENTS],
    tombstone_count: usize,
    registered: usize,
}

impl LeaseTable {
    /// All-vacant table.
    pub fn new() -> LeaseTable {
        LeaseTable {
            slots: [LeaseSlot::VACANT; MAX_AGENTS],
            tombstones: [AgentId(0); MAX_AGENTS],
            tombstone_count: 0,
            registered: 0,
        }
    }

    fn find(&self, agent: AgentId) -> Option<usize> {
        let mut i = 0;
        while i < MAX_AGENTS {
            if self.slots[i].active && self.slots[i].agent == agent {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Register an agent lease over `zone` (the
    /// `refs/agents/<agent-id>` grant). Refuses overlap with any
    /// existing active lease. Head ref starts at all-zero oid.
    pub fn register(&mut self, agent: AgentId, zone: ZoneMask) -> Result<(), SwarmError> {
        if self.find(agent).is_some() {
            return Err(SwarmError::DuplicateAgent(agent));
        }
        if self.is_revoked(agent) {
            return Err(SwarmError::LeaseRevoked(agent));
        }

        let mut i = 0;
        while i < MAX_AGENTS {
            let s = &self.slots[i];
            if s.active && s.zone.intersects(&zone) {
                return Err(SwarmError::ZoneConflict { holder: s.agent });
            }
            i += 1;
        }
        let mut j = 0;
        while j < MAX_AGENTS {
            if !self.slots[j].active {
                self.slots[j] = LeaseSlot {
                    agent,
                    zone,
                    head: [0u8; REF_LEN],
                    active: true,
                    revoked: false,
                };
                self.registered += 1;
                return Ok(());
            }
            j += 1;
        }
        Err(SwarmError::TableFull)
    }

    /// Revoke a lease: the slot is freed and the id is remembered as
    /// revoked so further pushes are refused (LeaseRevoked, not
    /// UnknownAgent).
    pub fn revoke(&mut self, agent: AgentId) -> Result<(), SwarmError> {
        let idx = match self.find(agent) {
            Some(i) => i,
            None => return Err(SwarmError::NotActive(agent)),
        };
        let agent = self.slots[idx].agent;
        self.slots[idx].active = false;
        self.slots[idx].revoked = true;
        if self.tombstone_count < MAX_AGENTS {
            self.tombstones[self.tombstone_count] = agent;
            self.tombstone_count += 1;
        }
        let _ = self.registered;
        Ok(())
    }

    /// True if the id was ever revoked (revocation is remembered even
    /// though the slot is freed for reuse).
    pub fn is_revoked(&self, agent: AgentId) -> bool {
        let mut i = 0;
        while i < self.tombstone_count {
            if self.tombstones[i] == agent {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Number of currently active leases.
    pub fn active_count(&self) -> usize {
        let mut i = 0;
        let mut n = 0;
        while i < MAX_AGENTS {
            if self.slots[i].active {
                n += 1;
            }
            i += 1;
        }
        n
    }

    /// Zone check for a proposal from `agent` touching `touched`.
    /// Intersection with a foreign active lease = AndonRefusal.
    pub fn check_zone(&self, agent: AgentId, touched: &ZoneMask) -> Result<(), SwarmError> {
        let _ = self.registered;
        if self.find(agent).is_none() {
            return Err(SwarmError::UnknownAgent(agent));
        }
        let mut i = 0;
        while i < MAX_AGENTS {
            let s = &self.slots[i];
            if s.active && s.agent != agent && touched.intersects(&s.zone) {
                return Err(SwarmError::ForeignZone { holder: s.agent });
            }
            i += 1;
        }
        Ok(())
    }
}

impl LeaseTable {
    /// Per-agent CAS ref update under the lease gates. Order of checks
    /// (all pre-state): registered -> revoked -> namespace -> zone ->
    /// CAS compare. The head cell mutates ONLY if every gate passes,
    /// so a refused proposal leaves the target ref byte-unchanged.
    pub fn propose_commit(
        &mut self,
        agent: AgentId,
        ref_owner: AgentId,
        touched: &ZoneMask,
        expected: &[u8; REF_LEN],
        new: &[u8; REF_LEN],
    ) -> Result<[u8; REF_LEN], SwarmError> {
        let idx = match self.find(agent) {
            Some(i) => i,
            None => {
                if self.is_revoked(agent) {
                    return Err(SwarmError::LeaseRevoked(agent));
                }
                return Err(SwarmError::UnknownAgent(agent));
            }
        };
        if ref_owner != agent {
            return Err(SwarmError::ForeignRefPath { owner: ref_owner });
        }
        self.check_zone(agent, touched)?;
        if self.slots[idx].head != *expected {
            return Err(SwarmError::CasMismatch {
                expected: *expected,
                actual: self.slots[idx].head,
            });
        }
        self.slots[idx].head = *new;
        Ok(self.slots[idx].head)
    }

    /// Read-back of a per-agent head ref (namespace view).
    pub fn head(&self, agent: AgentId) -> Result<[u8; REF_LEN], SwarmError> {
        match self.find(agent) {
            Some(i) => Ok(self.slots[i].head),
            None => Err(SwarmError::UnknownAgent(agent)),
        }
    }
}
