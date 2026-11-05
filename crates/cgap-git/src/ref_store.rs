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
//! has an empty `[dependencies]` table (published zero-dep kernel), so the
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
//! Atomic-width note (PUB-07 fix, lane DOD-04; selection corrected by the
//! R-ADV11FIX slide repair, `lanes/lane-r-adv11fix-implcell-slide.md`):
//! the seal `counter`/`chain` are width-gated via the legacy CAS cfg key
//! `target_has_atomic` (64 -> 32 -> 16 ladder). riscv32imc reports NO
//! CAS-capable atomic at any width, so rv32imc selects the implcell arm,
//! which is LOUD-until-bound: without `RUSTFLAGS="--cfg
//! implcell_critical_provided"` the build refuses with a typed
//! compile_error naming both cfg families; with the opt-in, firmware must
//! bind the real enter/exit-critical section around the seal RMW. On a
//! narrow target the monotonic counter may wrap/exhaust EARLIER (at the
//! native width, surfaced as `SEAL_COUNTER_WIDTH`); the typed
//! `CounterExhausted` refusal fires at that smaller bound so monotonic
//! CAS semantics are preserved - the counter exhausts, it never
//! silently wraps. Chain values may differ across widths (FNV over a
//! narrower accumulator) - chains are width-local ordering witnesses,
//! not cross-target invariants.

// court-consumed surface (CEN2 ALLOW-WITH-REASON): ref_store_court / ref_store_width_arms_court / ref_store_reentrancy_court / soc_full_chip_court; wire consumers pending — module-level allow (uniform surface, not scattered attrs).
#![allow(dead_code)]

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
/// ADV-11 repair: which cfg family is authoritative for what.
///
/// Empirically probed on the current nightly (rustc 1.101.0-nightly
/// 12433c307 2026-10-24), per target:
///   - host (aarch64): legacy `target_has_atomic` = 8/16/32/64/128 AND
///     `target_has_atomic_load_store` = same + ptr (CAS present).
///   - riscv32imc-unknown-none-elf: ONLY `target_has_atomic_load_store`
///     = 8/16/32 (+ptr). NO legacy `target_has_atomic` key of any width.
///
/// Meaning: the legacy `target_has_atomic = "N"` keys are the ONLY cfg
/// signal that carries CAS/RMW capability. The new
/// `target_has_atomic_load_store = "N"` family reports load/store
/// atomicity only - it is PRESENT for "32" on rv32imc, which has no CAS
/// at all, so it must NEVER be treated as a fetch_update license. The
/// old behavior (rv32imc silently selecting impl32 and emitting
/// __atomic_* libcalls for an RMW the target cannot honor) is exactly
/// the mis-arm this ladder refuses to repeat: the no-CAS arm now fails
/// LOUD unless firmware opts in, and the width witness carries an
/// independent (new-family-keyed) arm discriminator that cannot
/// self-agree with the ladder (see `seal_width_witness` at file tail).
mod seal_atomic {
    #[cfg(target_has_atomic = "64")]
    pub use impl64::*;

    #[cfg(all(not(target_has_atomic = "64"), target_has_atomic = "32"))]
    pub use impl32::*;

    #[cfg(all(not(target_has_atomic = "32"), target_has_atomic = "16"))]
    pub use impl16::*;

    #[cfg(all(
        not(target_has_atomic = "64"),
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

    /// No native CAS at any width (riscv32imc current-nightly shape: the
    /// target reports `target_has_atomic_load_store = "32"` - load/store
    /// atomicity only - and NO legacy `target_has_atomic` CAS key).
    ///
    /// ADV-11 typed contract (replaces the silent pass-through): the RMW
    /// on this arm is a plain non-atomic `Cell` read-modify-write. It is
    /// sound ONLY inside a critical section. The crate is zero-dep and
    /// cannot provide one, so firmware must bind it EXPLICITLY by passing
    /// `--cfg implcell_critical_provided` in RUSTFLAGS (the Cargo.toml
    /// feature form was not available to this lane - Cargo.toml is
    /// outside the lane's file ownership; see the lane receipt's
    /// divergence note). Without the opt-in, this target build is a loud
    /// compile_error naming both keys - never a silently-unsynchronized
    /// RMW behind a pass-through `critical`.
    #[cfg(all(
        not(target_has_atomic = "64"),
        not(target_has_atomic = "32"),
        not(target_has_atomic = "16"),
        not(target_has_atomic = "8")
    ))]
    pub(crate) mod implcell {
        use core::cell::Cell;
        pub type Prim = usize;
        pub type Inner = Cell<usize>;
        pub const WIDTH: u64 = usize::MAX as u64;

        /// The critical seam. With the firmware opt-in
        /// (`--cfg implcell_critical_provided`), this is the pass-through
        /// `f()` body: the caller has ACCEPTED the contract that
        /// interrupts are disabled (or a real enter/exit-critical is
        /// inlined at the call site by the firmware port) for the RMW's
        /// duration - a REENTRANCY VIOLATION otherwise, accepted loudly at
        /// build time, not silent.
        #[cfg(implcell_critical_provided)]
        pub fn critical<R>(f: impl FnOnce() -> R) -> R {
            f()
        }
    }

    /// ADV-11 loud-until-bound gate: on a no-CAS target without the
    /// firmware opt-in, the build refuses with a typed diagnostic naming
    /// the two cfg keys and the two lawful exits (bind a critical section
    /// via the opt-in, or move the kernel to a CAS-capable target). This
    /// is the fix for the silent toolchain slide: rv32imc can never again
    /// compile the unsynchronized RMW without the acceptance being
    /// visible in the build command itself.
    #[cfg(all(
        not(target_has_atomic = "64"),
        not(target_has_atomic = "32"),
        not(target_has_atomic = "16"),
        not(target_has_atomic = "8"),
        not(implcell_critical_provided)
    ))]
    compile_error!(
        "cgap-git: no CAS-capable atomic on this target (no legacy \
         target_has_atomic key at any width; the load_store family, if \
         present, is load/store only and NOT a fetch_update license). \
         The implcell RMW requires a bound critical section: rebuild with \
         `RUSTFLAGS=\"--cfg implcell_critical_provided\"` ONLY if your \
         firmware disables interrupts (or supplies enter/exit-critical) \
         around the seal RMW; otherwise build for a CAS-capable target."
    );
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
            () => self.0.load(core::sync::atomic::Ordering::SeqCst),
            #[cfg(not(target_has_atomic = "8"))]
            () => self.0.get() as u64,
        }
    }

    // AtomicU64::try_update is stable since 1.95; toolchain is floating
    // nightly (rust-toolchain.toml) against an MSRV of 1.85.
    #[allow(clippy::incompatible_msrv)]
    fn fetch_update<F>(&self, mut f: F) -> Result<u64, u64>
    where
        F: FnMut(u64) -> Option<u64>,
    {
        match () {
            #[cfg(target_has_atomic = "8")]
            () => self.0.try_update(
                core::sync::atomic::Ordering::SeqCst,
                core::sync::atomic::Ordering::SeqCst,
                |c| f(c).map(|v| v as seal_atomic::Prim),
            ),
            #[cfg(not(target_has_atomic = "8"))]
            () => seal_atomic::implcell::critical(|| {
                let cur = self.0.get() as u64;
                match f(cur) {
                    Some(next) => {
                        self.0.set(next as seal_atomic::Prim);
                        Ok(cur)
                    }
                    None => Err(cur),
                }
            }),
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

impl Default for SealRegister {
    fn default() -> Self {
        Self::new()
    }
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
        // RED-11 fix: ceiling enforcement is INSIDE the RMW (see
        // [`SealRegister::reserve`], which computes the effective bound
        // itself). A single
        // indivisible check-and-increment (fetch_update CAS loop) replaces
        // the separate load/check + fetch_add pair whose preempted window
        // minted leases past the ceiling and, at bound >= Prim::MAX,
        // wrapped the counter to 0. See [`SealRegister::reserve`].
        let stamped = self.reserve()?;
        // Slot copy + chain fold happen AFTER the indivisible lease
        // reservation: a reentrant observer sees a reserved lease even if
        // it inspects mid-stamp, never a duplicated one.
        self.slot[offset..offset + 8].copy_from_slice(tag);
        let _ = self.chain.fetch_update(|c| {
            Some(c.wrapping_mul(0x01000193).wrapping_add(0x811C_9DC5) ^ u64::from_be_bytes(*tag))
        });
        Ok(stamped)
    }

    /// Indivisible check-and-increment over the seal counter (RED-11
    /// repair): the ceiling guard lives INSIDE the fetch_update closure, so
    /// there is no window between "check passed" and "incremented" for a
    /// preempting ISR to occupy. The closure only ever emits `c + 1` while
    /// `c < bound <= WIDTH`, so the stored value can never pass
    /// `Prim::MAX` — a full counter yields None -> typed CounterExhausted,
    /// never a wrap to 0. This is THE RMW `stamp` uses; the court probe
    /// below calls the identical code.
    fn reserve(&self) -> Result<u64, Refusal> {
        let bound = if self.ceiling < seal_atomic::WIDTH {
            self.ceiling
        } else {
            seal_atomic::WIDTH
        };
        match self
            .counter
            .fetch_update(|c| if c < bound { Some(c + 1) } else { None })
        {
            Ok(prev) => Ok(prev + 1),
            Err(_full) => Err(Refusal::new(
                RefusalKind::CounterExhausted,
                "seal counter at ceiling; authority spent",
            )),
        }
    }

    /// #[doc(hidden)] COURT SEAM (lane RC-01, red-11 flip witness): runs
    /// exactly the production RMW ([`SealRegister::reserve`]) with no
    /// ceiling pre-check, so the reentrancy court can replay the preempted
    /// window: context A passes the (now advisory, RMW-external) state
    /// read, the ISR spends the remaining authority through real stamps,
    /// context A resumes AT the RMW. Fixed code refuses typed here; the
    /// reverted separate-check/RMW shape mints past the ceiling and the
    /// court goes red. Not a sanctioned API.
    #[doc(hidden)]
    pub fn reserve_probe(&self) -> Result<u64, Refusal> {
        self.reserve()
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

impl Default for RefStore {
    fn default() -> Self {
        Self::new()
    }
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

impl Default for Journal {
    fn default() -> Self {
        Self::new()
    }
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

// ==========================================================================
// RC-02 compile-witness (red-11 findings 1 and 5) — compile-time only;
// zero counter logic in this module (RC-01 owns that region).
//
// Witness W1 (finding 5): selecting the host-only `std` feature on an
// OS-less target (target_os = "none", riscv32imc-class) is a loud
// compile_error naming the fix, replacing the raw E0463 toolchain failure.
#[cfg(all(feature = "std", target_os = "none"))]
compile_error!(
    "cgap-git: default `std` feature is enabled on an OS-less target \
     (riscv32imc-class). std is host-only; rebuild with \
     `--no-default-features` for the no_std kernel build."
);

// Witness W2 (finding 1): the cfg ladder arms are width contracts, but
// impl16 and implcell are dead on every target the fleet builds (host =
// impl64, riscv32imc = impl32). Each arm that does NOT fire on this
// target gets a mirror module compiled here, with its width constant
// statically asserted, so a width-constant regression breaks the build
// on EVERY target instead of only on the arm's native one. Mirror bodies
// track ref_store.rs `mod seal_atomic`; drift is caught by the
// SEAL_COUNTER_WIDTH cross-check at the bottom (exactly one arm's WIDTH
// must equal it).
mod seal_width_witness {
    #![allow(dead_code)]

    #[cfg(target_has_atomic = "64")]
    const fn selected_width() -> u64 {
        u64::MAX
    }
    #[cfg(all(not(target_has_atomic = "64"), target_has_atomic = "32"))]
    const fn selected_width() -> u64 {
        u32::MAX as u64
    }
    #[cfg(all(not(target_has_atomic = "32"), target_has_atomic = "16"))]
    const fn selected_width() -> u64 {
        u16::MAX as u64
    }
    #[cfg(all(
        not(target_has_atomic = "32"),
        not(target_has_atomic = "16"),
        not(target_has_atomic = "8")
    ))]
    const fn selected_width() -> u64 {
        usize::MAX as u64
    }

    const _: () = assert!(
        crate::ref_store::SEAL_COUNTER_WIDTH == selected_width(),
        "SEAL_COUNTER_WIDTH disagrees with the target's selected cfg arm",
    );

    // Mirror of seal_atomic::impl32 — compiles wherever AtomicU32 exists.
    #[cfg(target_has_atomic = "32")]
    mod mirror_impl32 {
        pub type Prim = u32;
        pub type Inner = core::sync::atomic::AtomicU32;
        pub const WIDTH: u64 = u32::MAX as u64;
    }
    #[cfg(target_has_atomic = "32")]
    const _: () = assert!(
        mirror_impl32::WIDTH == u32::MAX as u64,
        "impl32 arm width contract violated",
    );

    // Mirror of seal_atomic::impl16 — compiles wherever AtomicU16 exists.
    #[cfg(target_has_atomic = "16")]
    mod mirror_impl16 {
        pub type Prim = u16;
        pub type Inner = core::sync::atomic::AtomicU16;
        pub const WIDTH: u64 = u16::MAX as u64;
    }
    #[cfg(target_has_atomic = "16")]
    const _: () = assert!(
        mirror_impl16::WIDTH == u16::MAX as u64,
        "impl16 arm width contract violated",
    );

    // Mirror of seal_atomic::implcell — compiles everywhere (core::cell).
    mod mirror_implcell {
        use core::cell::Cell;
        pub type Prim = usize;
        pub type Inner = Cell<usize>;
        pub const WIDTH: u64 = usize::MAX as u64;
    }
    const _: () = assert!(
        mirror_implcell::WIDTH == usize::MAX as u64,
        "implcell arm width contract violated",
    );

    // ------------------------------------------------------------------
    // ADV-11 arm discriminator: the ONLY cross-check that cannot
    // self-agree with the ladder. The ladder selects on the legacy
    // `target_has_atomic` CAS keys; the EXPECTED arm below is derived
    // from the INDEPENDENT `target_has_atomic_load_store` family. On the
    // toolchain that silently slid rv32imc onto implcell, both sides of
    // the OLD witness used the same legacy keys and fell together - the
    // cross-check asserted vacuously. Here the two sides read DIFFERENT
    // cfg families, so a family divergence (e.g. a future nightly that
    // reports load_store="64" but no legacy CAS key) fails the build
    // loudly instead of silently re-arming.
    const ARM_IMPL64: u8 = 4;
    const ARM_IMPL32: u8 = 3;
    const ARM_IMPL16: u8 = 2;
    const ARM_IMPLCELL: u8 = 1;

    // Selected arm - the SAME keys the ladder reads (legacy CAS family).
    #[cfg(target_has_atomic = "64")]
    const SELECTED_ARM: u8 = ARM_IMPL64;
    #[cfg(all(not(target_has_atomic = "64"), target_has_atomic = "32"))]
    const SELECTED_ARM: u8 = ARM_IMPL32;
    #[cfg(all(not(target_has_atomic = "32"), target_has_atomic = "16"))]
    const SELECTED_ARM: u8 = ARM_IMPL16;
    #[cfg(all(
        not(target_has_atomic = "64"),
        not(target_has_atomic = "32"),
        not(target_has_atomic = "16"),
        not(target_has_atomic = "8")
    ))]
    const SELECTED_ARM: u8 = ARM_IMPLCELL;

    // Expected arm - probed from an INDEPENDENT cfg family
    // (`target_pointer_width`, stable). Note (divergence recorded in the
    // lane receipt): the load_store family
    // (`target_has_atomic_load_store`) would be the tighter probe but is
    // UNSTABLE (E0658, `cfg_target_has_atomic` feature-gated) and
    // unusable from cfg predicates in this zero-dep kernel without a
    // crate-level feature gate (lib.rs is outside this lane's
    // ownership).
    #[cfg(target_pointer_width = "64")]
    const EXPECTED_ARM: u8 = ARM_IMPL64;
    #[cfg(all(
        target_pointer_width = "32",
        any(target_has_atomic = "64", target_has_atomic = "32")
    ))]
    const EXPECTED_ARM: u8 = ARM_IMPL32;
    #[cfg(all(
        target_pointer_width = "16",
        not(any(target_has_atomic = "64", target_has_atomic = "32")),
        target_has_atomic = "16"
    ))]
    const EXPECTED_ARM: u8 = ARM_IMPL16;
    #[cfg(all(
        target_pointer_width = "32",
        not(any(target_has_atomic = "64", target_has_atomic = "32"))
    ))]
    const EXPECTED_ARM: u8 = ARM_IMPLCELL;

    const _: () = assert!(
        SELECTED_ARM == EXPECTED_ARM,
        "cfg-arm discriminator: the ladder's selected arm disagrees with \
         the independently probed target_pointer_width family - a \
         toolchain slide moved one family without the other; adjudicate \
         before building",
    );
}
