//! Lane GIT-13 - loose+packed ref table with lease-sealed atomic CAS.
//!
//! Additive, self-contained, core-only (no alloc, no std, no unsafe). The
//! test harness includes this file directly (`#[path]`), so `lib.rs` is
//! untouched.
//!
//! Provenance note (honest, no invented hardware lore): the dispatch named
//! a "128-byte hardware seal register at 0x0001_FF80". A search of the tree
//! (cgap-ffi/sovereign.rs, cgap-gate/penstock.rs, docs) found NO existing
//! referent for that address. The seal register is therefore implemented as
//! a caller-provided fixed 128-byte slot abstraction ([`SealRegister`]);
//! `SEAL_REGISTER_ADDR` is a named constant whose provenance is recorded
//! here - it binds to no hardware map that exists in-tree.
//!
//! The monotonic counter mirrors the semantics of cgap-crypto `lease.rs`
//! (80-byte PhiLease, SS5 monotonic tick, exhaustion trap) - but cgap-git
//! has an empty [dependencies] table (published zero-dep kernel), so the
//! semantics are mirrored, not imported. Lease binding is a per-ref u64
//! lease counter stamped atomically via `core::sync::atomic`.
//!
//! Crash consistency is MODELED as sequential write + commit marker
//! (in-memory journal with rollback on missing marker). This is an honest
//! in-memory court simulation, not a durable WAL (A05 owns the real WAL
//! framing).
//!
//! AndonRefusal-class: typed refusal matching the RFC-0042 Rule 3 shape
//! used by `crates/cgap-git/src/ingress_ring.rs` (exit-101-class family,
//! deterministic, in-process `Err`).
//!
//! Atomic-width note (PUB-07 fix, lane DOD-04): riscv32imc provides no
//! 64-bit atomics, so the seal `counter`/`chain` are width-gated via
//! `cfg(target_has_atomic = ...)` (64 -> 32 -> 16 ladder). On a narrow
//! target the monotonic counter may wrap/exhaust EARLIER (at the native
//! width, surfaced as `SEAL_COUNTER_WIDTH`); the typed
//! `CounterExhausted` refusal fires at that smaller bound so monotonic
//! CAS semantics are preserved - the counter exhausts, it never
//! silently wraps. Chain values may differ across widths (FNV over a
//! narrower accumulator) - chains are width-local ordering witnesses,
//! not cross-target invariants.

/// No hardware map exists in-tree binding this address; see module
/// provenance note. Named only so the constant is citable and falsifiable.
pub const SEAL_REGISTER_ADDR: u64 = 0x0001_FF80;

/// Seal slot width: 128 bytes.
pub const SEAL_SLOT_LEN: usize = 128;
/// Max ref name length (e.g. `refs/heads/main`).
pub const REF_NAME_LEN: usize = 64;
/// Fixed table capacity, no heap.
pub const REF_TABLE_CAP: usize = 16;
/// Journal record capacity, no heap.
pub const JOURNAL_CAP: usize = 32;

const HEADS: &[u8] = b"refs/heads/";
const AGENTS: &[u8] = b"refs/agents/";

/// AndonRefusal-class typed refusal (RFC-0042 Rule 3 shape).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refusal {
    /// Refusal class.
    pub kind: RefusalKind,
    /// Static diagnostic (AndonRefusal `msg` shape).
    pub msg: &'static str,
}

/// Refusal classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalKind {
    /// CAS expected-counter mismatch: out-of-order update under a stale
    /// parent lease.
    StaleLease,
    /// Ref name not present in the table.
    UnknownRef,
    /// Ref name outside `refs/heads/*` and `refs/agents/*`.
    BadNamespace,
    /// Table or journal at fixed capacity.
    TableFull,
    /// Journal replay hit a written record with no commit marker.
    TornWrite,
    /// Seal-register counter ceiling reached.
    CounterExhausted,
}

impl RefusalKind {
    /// Short class tag for court assertions.
    pub fn tag(&self) -> &'static str {
        match self {
            RefusalKind::StaleLease => "STALE_LEASE",
            RefusalKind::UnknownRef => "UNKNOWN_REF",
            RefusalKind::BadNamespace => "BAD_NAMESPACE",
            RefusalKind::TableFull => "TABLE_FULL",
            RefusalKind::TornWrite => "TORN_WRITE",
            RefusalKind::CounterExhausted => "COUNTER_EXHAUSTED",
        }
    }
}

impl Refusal {
    fn new(kind: RefusalKind, msg: &'static str) -> Self {
        Refusal { kind, msg }
    }
}

/// Width-gated native-atomic backend for the seal counter/chain.
///
/// riscv32imc has NO 64-bit atomics (`core::sync::atomic::AtomicU64`
/// does not exist on that target - PUB-07 failure B); the widest native
/// primitive is addressed by the cfg ladder below (64 -> 32 -> 16).
/// On a narrow target the monotonic counter ceiling collapses to the
/// primitive width: the typed CounterExhausted refusal fires at that
/// smaller bound instead of wrapping (monotonicity is preserved; the
/// counter exhausts EARLIER, never silently wraps).
mod seal_atomic {
    #[cfg(target_has_atomic = "64")]
    pub use impl64::*;

    #[cfg(all(not(target_has_atomic = "64"), target_has_atomic = "32"))]
    pub use impl32::*;

    #[cfg(all(not(target_has_atomic = "32"), target_has_atomic = "16"))]
    pub use impl16::*;

    #[cfg(all(
        not(target_has_atomic = "32"),
        not(target_has_atomic = "16"),
        not(target_has_atomic = "8")
    ))]
    pub use implcell::*;

    #[cfg(target_has_atomic = "64")]
    mod impl64 {
        pub type Prim = u64;
        pub type Inner = core::sync::atomic::AtomicU64;
        /// Widest value the native counter can hold (ceiling bound).
        pub const WIDTH: u64 = u64::MAX;
    }

    #[cfg(all(not(target_has_atomic = "64"), target_has_atomic = "32"))]
    mod impl32 {
        pub type Prim = u32;
        pub type Inner = core::sync::atomic::AtomicU32;
        pub const WIDTH: u64 = u32::MAX as u64;
    }

    #[cfg(all(not(target_has_atomic = "32"), target_has_atomic = "16"))]
    mod impl16 {
        pub type Prim = u16;
        pub type Inner = core::sync::atomic::AtomicU16;
        pub const WIDTH: u64 = u16::MAX as u64;
    }

    /// No native atomics at all (riscv32imc: no A extension). Sound
    /// here because every mutating entry point ([`SealRegister::stamp`])
    /// already requires `&mut self` (exclusive access) and the type is
    /// NOT `Sync` on this arm - single-hart targets only.
    #[cfg(all(
        not(target_has_atomic = "32"),
        not(target_has_atomic = "16"),
        not(target_has_atomic = "8")
    ))]
    mod implcell {
        use core::cell::Cell;
        pub type Prim = usize;
        pub type Inner = Cell<usize>;
        pub const WIDTH: u64 = usize::MAX as u64;
    }
}

/// Widest counter value the target's native atomics can address. The
/// seal counter never exceeds this bound: exhaustion refuses first.
pub const SEAL_COUNTER_WIDTH: u64 = seal_atomic::WIDTH;

/// Monotonic counter backed by the widest native atomic on the target.
/// All values cross the API in `u64`; storage is width-narrowed.
struct Counter(seal_atomic::Inner);

impl Counter {
    fn new(v: u64) -> Self {
        Counter(seal_atomic::Inner::new(v as seal_atomic::Prim))
    }

    fn load(&self) -> u64 {
        match () {
            #[cfg(target_has_atomic = "8")]
            () => self.0.load(core::sync::atomic::Ordering::SeqCst) as u64,
            #[cfg(not(target_has_atomic = "8"))]
            () => self.0.get() as u64,
        }
    }

    fn fetch_add(&self) -> u64 {
        match () {
            #[cfg(target_has_atomic = "8")]
            () => self.0.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as u64,
            #[cfg(not(target_has_atomic = "8"))]
            () => {
                let cur = self.0.get() as u64;
                self.0.set(cur.wrapping_add(1) as seal_atomic::Prim);
                cur
            }
        }
    }

    fn fetch_update<F>(&self, mut f: F) -> Result<u64, u64>
    where
        F: FnMut(u64) -> Option<u64>,
    {
        match () {
            #[cfg(target_has_atomic = "8")]
            () => self
                .0
                .fetch_update(
                    core::sync::atomic::Ordering::SeqCst,
                    core::sync::atomic::Ordering::SeqCst,
                    |c| f(c as u64).map(|v| v as seal_atomic::Prim),
                )
                .map(|v| v as u64)
                .map_err(|v| v as u64),
            #[cfg(not(target_has_atomic = "8"))]
            () => {
                let cur = self.0.get() as u64;
                match f(cur) {
                    Some(next) => {
                        self.0.set(next as seal_atomic::Prim);
                        Ok(cur)
                    }
                    None => Err(cur),
                }
            }
        }
    }
}

/// Caller-provided fixed 128-byte slot + atomic monotonic counter.
pub struct SealRegister {
    slot: [u8; SEAL_SLOT_LEN],
    counter: Counter,
    ceiling: u64,
    /// FNV-1a chain over every stamp (tamper-evident ordering witness).
    chain: Counter,
}

impl SealRegister {
    /// Fresh register: zeroed slot, counter at 0, unbounded ceiling.
    pub fn new() -> Self {
        SealRegister {
            slot: [0u8; SEAL_SLOT_LEN],
            counter: Counter::new(0),
            ceiling: u64::MAX,
            chain: Counter::new(0),
        }
    }

    /// Fresh register with an explicit counter ceiling (exhaustion is a
    /// typed refusal, never a wrap).
    pub fn with_ceiling(ceiling: u64) -> Self {
        let mut r = SealRegister::new();
        r.ceiling = ceiling;
        r
    }

    /// Reads the current monotonic counter (pure, atomic load).
    pub fn counter(&self) -> u64 {
        self.counter.load()
    }

    /// Reads the FNV chain (pure, atomic load).
    pub fn chain(&self) -> u64 {
        self.chain.load()
    }

    /// 128-byte slot read (court observability).
    pub fn slot(&self) -> &[u8; SEAL_SLOT_LEN] {
        &self.slot
    }

    /// Atomically increments the counter and stamps `tag` into the slot at
    /// `offset`. Returns the post-increment counter. Refuses
    /// CounterExhausted when the ceiling is already reached, and
    /// TableFull-class range refusal when tag would run past the slot.
    pub fn stamp(&mut self, offset: usize, tag: &[u8; 8]) -> Result<u64, Refusal> {
        if offset + 8 > SEAL_SLOT_LEN {
            return Err(Refusal::new(
                RefusalKind::TableFull,
                "seal stamp runs past the 128-byte slot",
            ));
        }
        // Effective ceiling collapses to the target's native counter
        // width on narrow targets (exhaustion refuses there, no wrap).
        let bound = if self.ceiling < seal_atomic::WIDTH {
            self.ceiling
        } else {
            seal_atomic::WIDTH
        };
        let cur = self.counter.load();
        if cur >= bound {
            return Err(Refusal::new(
                RefusalKind::CounterExhausted,
                "seal counter at ceiling; authority spent",
            ));
        }
        self.slot[offset..offset + 8].copy_from_slice(tag);
        let _ = self.chain.fetch_update(|c| {
            Some(c.wrapping_mul(0x01000193).wrapping_add(0x811C_9DC5) ^ u64::from_be_bytes(*tag))
        });
        Ok(self.counter.fetch_add() + 1)
    }
}

/// One loose ref entry (fixed-size, no heap).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefEntry {
    /// Full ref name, `REF_NAME_LEN`-padded with zeros.
    pub name: [u8; REF_NAME_LEN],
    /// Name length in bytes.
    pub name_len: usize,
    /// 20-byte object name this ref points at.
    pub oid: [u8; 20],
    /// Lease counter of the last accepted update (CAS expected value).
    pub lease: u64,
}

impl RefEntry {
    fn name_bytes(&self) -> &[u8] {
        &self.name[..self.name_len]
    }
}

/// Commit-marker journal record (sequential write + marker model).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalRecord {
    pub seq: u64,
    pub name: [u8; REF_NAME_LEN],
    pub name_len: usize,
    pub new_oid: [u8; 20],
    pub lease: u64,
    /// Commit marker: false = written but not committed (torn write).
    pub committed: bool,
}

/// In-memory crash-consistency journal (court simulation, honestly
/// labeled; not a durable WAL).
pub struct Journal {
    records: [Option<JournalRecord>; JOURNAL_CAP],
    len: usize,
    /// Set when replay refused on a missing commit marker.
    last_refusal: Option<RefusalKind>,
}

/// Loose+packed ref table, fixed capacity, no heap.
pub struct RefStore {
    entries: [Option<RefEntry>; REF_TABLE_CAP],
    len: usize,
}

impl RefStore {
    /// Empty table.
    pub fn new() -> Self {
        RefStore {
            entries: [None; REF_TABLE_CAP],
            len: 0,
        }
    }

    /// Number of live refs.
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when the table holds no refs.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// FNV-1a fingerprint over all live entry bytes: the byte-unchanged
    /// assertion surface for refusal and rollback courts.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0x811C_9DC5;
        for e in self.entries.iter().flatten() {
            for b in e.name.iter() {
                h = (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01B3);
            }
            h = (h ^ (e.name_len as u64)).wrapping_mul(0x0100_0000_01B3);
            for b in e.oid.iter() {
                h = (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01B3);
            }
            h = (h ^ e.lease).wrapping_mul(0x0100_0000_01B3);
        }
        h
    }

    /// Looks a ref up by name.
    pub fn get(&self, name: &[u8]) -> Option<&RefEntry> {
        self.entries
            .iter()
            .flatten()
            .find(|e| e.name_bytes() == name)
    }

    fn find_slot(&self, name: &[u8]) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| matches!(e, Some(en) if en.name_bytes() == name))
    }

    /// Validates the namespace: `refs/heads/*` or `refs/agents/*`.
    pub fn namespace_ok(name: &[u8]) -> bool {
        name.starts_with(HEADS) || name.starts_with(AGENTS)
    }

    /// Pads a name into the fixed-width form; refuses names over
    /// `REF_NAME_LEN` or outside both namespaces.
    pub fn pad_name(name: &[u8]) -> Result<([u8; REF_NAME_LEN], usize), Refusal> {
        if !Self::namespace_ok(name) {
            return Err(Refusal::new(
                RefusalKind::BadNamespace,
                "ref name outside refs/heads/* and refs/agents/*",
            ));
        }
        if name.len() > REF_NAME_LEN {
            return Err(Refusal::new(
                RefusalKind::BadNamespace,
                "ref name exceeds the fixed 64-byte width",
            ));
        }
        let mut buf = [0u8; REF_NAME_LEN];
        buf[..name.len()].copy_from_slice(name);
        Ok((buf, name.len()))
    }

    /// Creates a ref at lease 0. Refuses TableFull when the table is at
    /// capacity, BadNamespace on a bad name, UnknownRef on a duplicate.
    pub fn create(&mut self, name: &[u8], oid: [u8; 20]) -> Result<(), Refusal> {
        if Self::pad_name(name).is_err() {
            return Err(Refusal::new(
                RefusalKind::BadNamespace,
                "create: bad ref name or namespace",
            ));
        }
        if self.find_slot(name).is_some() {
            return Err(Refusal::new(
                RefusalKind::UnknownRef,
                "create: ref already exists",
            ));
        }
        if self.len == REF_TABLE_CAP {
            return Err(Refusal::new(
                RefusalKind::TableFull,
                "ref table at fixed capacity",
            ));
        }
        let (namep, n) = Self::pad_name(name).expect("checked above");
        self.entries[self.len] = Some(RefEntry {
            name: namep,
            name_len: n,
            oid,
            lease: 0,
        });
        self.len += 1;
        Ok(())
    }

    /// Atomic compare-and-swap over one ref, sealed by the register:
    /// succeeds only when the ref's current lease equals `expected_lease`.
    /// On success the ref's oid is updated, its lease becomes the new
    /// register counter, and the register stamps once (strict monotonic).
    /// On lease mismatch the table is untouched and the refusal is
    /// StaleLease (out-of-order update under a stale parent lease).
    pub fn cas(
        &mut self,
        name: &[u8],
        expected_lease: u64,
        new_oid: [u8; 20],
        seal: &mut SealRegister,
    ) -> Result<u64, Refusal> {
        let idx = match self.find_slot(name) {
            Some(i) => i,
            None => {
                return Err(Refusal::new(
                    RefusalKind::UnknownRef,
                    "cas: ref not present",
                ))
            }
        };
        let cur = self.entries[idx].expect("idx from live entries");
        if cur.lease != expected_lease {
            return Err(Refusal::new(
                RefusalKind::StaleLease,
                "cas: stale parent lease (expected-counter mismatch) - out-of-order update refused",
            ));
        }
        let tag = lease_tag(expected_lease, &new_oid);
        let stamped = seal.stamp(0, &tag)?;
        let e = self.entries[idx].as_mut().expect("idx from live entries");
        e.oid = new_oid;
        e.lease = stamped;
        Ok(stamped)
    }
}

/// 8-byte seal tag derived from the parent lease and the new oid.
fn lease_tag(lease: u64, oid: &[u8; 20]) -> [u8; 8] {
    let mut t = [0u8; 8];
    t[..8].copy_from_slice(&lease.to_le_bytes());
    for i in 0..20 {
        t[i % 8] ^= oid[i];
    }
    t
}

impl Journal {
    /// Empty journal.
    pub fn new() -> Self {
        Journal {
            records: [None; JOURNAL_CAP],
            len: 0,
            last_refusal: None,
        }
    }

    /// Appends an uncommitted record (sequential write, marker pending).
    pub fn write(
        &mut self,
        seq: u64,
        name: &[u8],
        new_oid: [u8; 20],
        lease: u64,
    ) -> Result<(), Refusal> {
        let (namep, n) = RefStore::pad_name(name)?;
        if self.len == JOURNAL_CAP {
            return Err(Refusal::new(RefusalKind::TableFull, "journal full"));
        }
        self.records[self.len] = Some(JournalRecord {
            seq,
            name: namep,
            name_len: n,
            new_oid,
            lease,
            committed: false,
        });
        self.len += 1;
        Ok(())
    }

    /// Sets the commit marker on record `seq`.
    pub fn commit(&mut self, seq: u64) -> Result<(), Refusal> {
        for r in self.records.iter_mut().flatten() {
            if r.seq == seq && !r.committed {
                r.committed = true;
                return Ok(());
            }
        }
        Err(Refusal::new(
            RefusalKind::UnknownRef,
            "commit: no pending record with that seq",
        ))
    }

    /// Replays committed records into the store IN ORDER. A written-but-
    /// uncommitted (torn) record stops replay: everything at and after the
    /// missing marker is rolled back (never applied), and the refusal is
    /// TornWrite. Returns the number of records applied.
    pub fn replay(&mut self, store: &mut RefStore) -> Result<usize, Refusal> {
        self.last_refusal = None;
        let mut applied = 0;
        for i in 0..self.len {
            let rec = match self.records[i] {
                Some(r) => r,
                None => break,
            };
            if !rec.committed {
                self.last_refusal = Some(RefusalKind::TornWrite);
                return Err(Refusal::new(
                    RefusalKind::TornWrite,
                    "replay: missing commit marker - rollback at this record",
                ));
            }
            // In-order apply through the real CAS path: the recorded lease
            // is the expected lease; the register must be driven to that
            // counter by earlier records, so no seal stamp happens here
            // (replay is idempotent state restore, not a new authority
            // event). Direct entry mutation, lease taken from the record.
            let idx = match store.entries.iter().position(
                |e| matches!(e, Some(en) if en.name[..en.name_len] == rec.name[..rec.name_len]),
            ) {
                Some(i) => i,
                None => {
                    return Err(Refusal::new(
                        RefusalKind::UnknownRef,
                        "replay: record names a ref the table does not hold",
                    ))
                }
            };
            if let Some(e) = store.entries[idx].as_mut() {
                if e.lease != rec.lease {
                    return Err(Refusal::new(
                        RefusalKind::StaleLease,
                        "replay: journal lease does not match table lease",
                    ));
                }
                e.lease = rec.lease + 1;
                e.oid = rec.new_oid;
            }
            applied += 1;
        }
        Ok(applied)
    }

    /// Last replay refusal class, if any.
    pub fn last_refusal(&self) -> Option<RefusalKind> {
        self.last_refusal
    }
}
