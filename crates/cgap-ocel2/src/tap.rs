//! AoQ tap-row consumption + sidecar chain sink (PL16 items 2-3,
//! OCEL-side lane).
//!
//! CONTRACT DISCLOSURE (cross-lane, X6 owns `cgap-ffi/src/aoq_events.rs`):
//! this module consumes X6's landed 64-byte [`AoqEventRow`] wire shape —
//! ordinal/state/kind/epoch/budget/fingerprint/reserved[20] at fixed
//! offsets (0..4 LE u32, 1B state, 1B kind, 6..8 LE u16, 8..12 LE u32,
//! 12..44 fingerprint, 44..64 reserved-zero). The two shapes are
//! mechanically tied by [`TapEvent::from_x6`] / [`TapEvent::to_x6`] and
//! the size pins below; if X6's layout moves, the size assert fires.
//!
//! ## Logical clock law (no wall clock)
//!
//! `epoch` + `ordinal` are the logical clock. This follows the project's
//! no-wall-clock precedent (`src/lib.rs:17-21`: "IT IS NOT A WALL CLOCK
//! ... the ordinal is a logical sequence number") and the AoQ counted
//! budget law (`cgap-aoq/src/session.rs:8-10`: "REKEY_BUDGET is COUNTED,
//! not timed ... No wall clock anywhere"). No timestamp exists in the
//! row; replay orders by (epoch, epoch-constant + ordinal), never by
//! time.
//!
//! ## Purity law as replay invariant
//!
//! `session.rs` law 4: no mutation on refusal; X6's caller-side law:
//! refusals emit nothing. Both collapse to ONE replay invariant: an
//! accepted step advances the ordinal by EXACTLY 1, and no row exists
//! for a refused frame — so a replayed walk with a gap, a repeat, or an
//! out-of-table (from, kind) cell is a typed refusal, never a pass.
//!
//! ## Chain binding (sidecar law)
//!
//! [`TapSink`] appends each tapped row into a SIDECAR-OWNED
//! [`RingReceiptChain`] as one real sealed cgap-parquet block: the row's
//! 64 bytes are carried verbatim as 64 u32 lane values (bits=8). The
//! writer's hybrid/RLE encoding is a deterministic function of the value
//! sequence, so re-sealing a row reproduces the committed block bytes
//! byte-exactly. Replay re-seals every logged row and matches it to its
//! chain-covered block bytes (through the full [`RingReceiptChain::verify`]
//! gate) — a tampered row log or a tampered chain collapses to the
//! typed refusal, never a silent pass.

#[cfg(test)]
use cgap_ffi::aoq_events::AoqEventRow;
#[cfg(test)]
use cgap_ffi::receipt_chain::{ReceiptChainRefusal, RingReceiptChain};
#[cfg(test)]
use cgap_p28::attestation::P28LicenseAttestation;
#[cfg(test)]
use cgap_parquet::footer::SealedBlock;
#[cfg(test)]
use cgap_parquet::mmap_writer::{SlabView, StaticPageBuffer};

/// Row byte width — mirrors [`AoqEventRow`] (X6's frozen 64B wire row).
pub const TAP_ROW_BYTES: usize = 64;

/// Activity label: one accepted `session.step` (the only transition the
/// caller-side tap can mint; refusals emit nothing).
pub const ACTIVITY_SESSION_STEP: &str = "session.step";
/// Activity label: a typed refusal observed by the caller (to-state ==
/// from-state; the session stays byte-identical — purity law). No ring
/// row ever carries this label; it is the projector-side value-list
/// entry documented in the view DDL.
pub const ACTIVITY_SESSION_REFUSE: &str = "session.refuse";
/// Activity label: a poison/boundary marker row (membrane boundary;
/// reserved projector-side value-list entry, see the view DDL).
pub const ACTIVITY_BOUNDARY_POISON: &str = "boundary.poison";

/// The replay/transition ORACLE — re-derived from the REAL 4x5 table in
/// `cgap-aoq/src/session.rs:453-519` (`Session::step`'s match arms).
/// SOURCE-OF-TRUTH CROSS-CHECK (do not re-declare independently; the
/// conformance court pins this table against the LIVE session by
/// driving real `step` calls and tapping only the `Ok` arm):
///
/// | from (state id)        | kind id           | to                |
/// |------------------------|-------------------|-------------------|
/// | 0 HandshakeInit        | 0 HandshakeEk     | 1 HandshakeWait   |
/// | 1 HandshakeWait        | 1 HandshakeFinish | 2 Established     |
/// | 2 Established          | 2 AppData         | 2 Established     |
/// | 2 Established          | 3 Rekey           | 2 Established     |
/// | any non-Closed (0,1,2) | 4 Close           | 3 Closed          |
/// | 3 Closed               | anything          | REFUSED (terminal)|
/// | anything else          | anything else     | REFUSED           |
///
/// `Some(to)` for an accepted cell; `None` for a refusal cell.
#[must_use]
#[cfg(test)]
pub(crate) fn lawful_step(from: u8, kind: u8) -> Option<u8> {
    match (from, kind) {
        (0, 0) => Some(1), // (HandshakeInit, HandshakeEk) -> HandshakeWait
        (1, 1) => Some(2), // (HandshakeWait, HandshakeFinish) -> Established
        (2, 2) => Some(2), // (Established, AppData) -> Established
        (2, 3) => Some(2), // (Established, Rekey) -> Established
        (0, 4) | (1, 4) | (2, 4) => Some(3), // (_, Close) -> Closed
        _ => None,
    }
}

/// One consumed tap row (the OCEL-side mirror of X6's [`AoqEventRow`]).
///
/// Fixed-offset 64-byte wire layout; reserved bytes are ZERO on write
/// and MUST decode as zero (forward-compat fence, refuse-closed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TapEvent {
    /// Session frame ordinal after the accepted step (wrapping u32).
    pub ordinal: u32,
    /// `SessionState::id()` AFTER the accepted step (the to-state).
    pub state: u8,
    /// `FrameKind::id()` of the accepted frame.
    pub kind: u8,
    /// Active epoch. Constant across a lawful walk (no accepted arm of
    /// `step` mutates the session's epoch).
    pub epoch: u16,
    /// Counted rekey budget used after the accepted step.
    pub budget: u32,
    /// `Session::fingerprint()` after the accepted step — verbatim.
    pub fingerprint: [u8; 32],
}

// Layout pin: 4 + 1 + 1 + 2 + 4 + 32 = 44 payload bytes; the remaining
// 20 bytes are the reserved-zero tail.
const _: () = assert!(TAP_ROW_BYTES == 64);

/// Fixed wire offsets (cross-checked against X6's `AoqEventRow` layout).
mod off {
    pub(super) const ORDINAL: usize = 0; // 0..4   LE u32
    pub(super) const STATE: usize = 4; // 4       u8
    pub(super) const KIND: usize = 5; // 5       u8
    pub(super) const EPOCH: usize = 6; // 6..8   LE u16
    pub(super) const BUDGET: usize = 8; // 8..12  LE u32
    pub(super) const FINGERPRINT: usize = 12; // 12..44
    pub(super) const RESERVED: usize = 44; // 44..64 reserved-zero
}

/// The tap-consumption refusal type (opaque per-class, p28 §5 shape).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapRefusal {
    /// Chain invariant violated (mapped 1:1 from the chain's own opaque
    /// refusal; fail-closed).
    Spoliation,
    /// Chain empty — nothing to replay (shape verdict).
    EmptyChain,
    /// Row log length != chain length.
    RowArityMismatch,
    /// A decoded row carries nonzero reserved bytes.
    ReservedNotZero,
    /// A row's (from, kind) cell is outside the session transition
    /// table, or its to-state disagrees with the table.
    IllegalTransition {
        /// The state the replay walk was in before the row.
        from: u8,
        /// The row's frame kind.
        kind: u8,
        /// The to-state the row claimed.
        got: u8,
    },
    /// Ordinal did not advance by exactly 1 (wrapping) from the prior
    /// row — a gap, a repeat, or a refusal that leaked a row.
    OrdinalLaw {
        /// The ordinal the walk expected.
        expected: u32,
        /// The ordinal the row carried.
        got: u32,
    },
    /// Budget law broken: AppData not counting, a Rekey not resetting
    /// to zero, or a budget change on a non-budget arm.
    BudgetLaw {
        /// The budget the walk expected after this row.
        expected: u32,
        /// The budget the row carried.
        got: u32,
    },
    /// Epoch changed mid-walk (no accepted arm of `step` mutates it).
    EpochLaw {
        /// The epoch established by the first row.
        expected: u16,
        /// The epoch the row carried.
        got: u16,
    },
    /// A logged row does not re-seal to its chain-covered block bytes
    /// (row log and chain desynced).
    RowBinding {
        /// The chain position whose row failed to re-seal.
        index: u64,
    },
    /// The sidecar's seal did not survive re-sealing (writer shape
    /// failure — unreachable on the real writer; fail-closed).
    ResealShape,
}

impl core::fmt::Display for TapRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Spoliation => f.write_str("tap chain spoliation (fail-closed)"),
            Self::EmptyChain => f.write_str("tap chain empty: nothing to replay"),
            Self::RowArityMismatch => f.write_str("row log length != chain length"),
            Self::ReservedNotZero => {
                f.write_str("row reserved bytes nonzero (forward-compat fence)")
            }
            Self::IllegalTransition { from, kind, got } => {
                write!(f, "illegal transition: state {from} + kind {kind} -> {got}")
            }
            Self::OrdinalLaw { expected, got } => {
                write!(f, "ordinal law: expected {expected}, row carries {got}")
            }
            Self::BudgetLaw { expected, got } => {
                write!(f, "budget law: expected budget {expected}, row carries {got}")
            }
            Self::EpochLaw { expected, got } => {
                write!(f, "epoch law: expected {expected}, row carries {got}")
            }
            Self::RowBinding { index } => {
                write!(f, "row {index} does not re-seal to its chain block bytes")
            }
            Self::ResealShape => f.write_str("sidecar re-seal shape failure"),
        }
    }
}

impl core::error::Error for TapRefusal {}

/// Values lane for a row: the 64 raw bytes as u32 values (each <= 255),
/// encoded with bits=8 by the real writer. Deterministic in, byte-exact
/// out.
#[cfg(test)]
fn row_values(row: &TapEvent) -> [u32; TAP_ROW_BYTES] {
    let bytes = row.encode();
    core::array::from_fn(|i| u32::from(bytes[i]))
}

impl TapEvent {
    /// Encode the row into the fixed-offset 64-byte wire form. Reserved
    /// bytes are written as ZERO (reserved-zero law).
    #[must_use]
    pub fn encode(&self) -> [u8; TAP_ROW_BYTES] {
        let mut b = [0u8; TAP_ROW_BYTES];
        b[off::ORDINAL..off::ORDINAL + 4].copy_from_slice(&self.ordinal.to_le_bytes());
        b[off::STATE] = self.state;
        b[off::KIND] = self.kind;
        b[off::EPOCH..off::EPOCH + 2].copy_from_slice(&self.epoch.to_le_bytes());
        b[off::BUDGET..off::BUDGET + 4].copy_from_slice(&self.budget.to_le_bytes());
        b[off::FINGERPRINT..off::FINGERPRINT + 32].copy_from_slice(&self.fingerprint);
        b
    }

    /// Decode a row from the fixed-offset wire form. Refuse-closed on
    /// nonzero reserved bytes (typed [`TapRefusal::ReservedNotZero`]).
    ///
    /// # Errors
    /// [`TapRefusal::ReservedNotZero`] on a nonzero reserved tail.
    pub fn decode(row: &[u8; TAP_ROW_BYTES]) -> Result<Self, TapRefusal> {
        if row[off::RESERVED..].iter().any(|&b| b != 0) {
            return Err(TapRefusal::ReservedNotZero);
        }
        let mut fingerprint = [0u8; 32];
        fingerprint.copy_from_slice(&row[off::FINGERPRINT..off::FINGERPRINT + 32]);
        Ok(Self {
            ordinal: u32::from_le_bytes(row[off::ORDINAL..off::ORDINAL + 4]
                .try_into()
                .expect("fixed 4-byte slice")),
            state: row[off::STATE],
            kind: row[off::KIND],
            epoch: u16::from_le_bytes(
                row[off::EPOCH..off::EPOCH + 2].try_into().expect("fixed 2-byte slice"),
            ),
            budget: u32::from_le_bytes(
                row[off::BUDGET..off::BUDGET + 4]
                    .try_into()
                    .expect("fixed 4-byte slice"),
            ),
            fingerprint,
        })
    }

    /// Consume X6's minted row verbatim (the disclosed cross-lane
    /// contract: fields carry over 1:1; reserved stays zero).
    #[must_use]
    #[cfg(test)]
    pub(crate) fn from_x6(row: &AoqEventRow) -> Self {
        Self {
            ordinal: row.ordinal,
            state: row.state,
            kind: row.kind,
            epoch: row.epoch,
            budget: row.budget,
            fingerprint: row.fingerprint,
        }
    }

    /// Project the row back onto X6's shape (round-trip seam).
    #[must_use]
    #[cfg(test)]
    pub(crate) fn to_x6(&self) -> AoqEventRow {
        AoqEventRow {
            ordinal: self.ordinal,
            state: self.state,
            kind: self.kind,
            epoch: self.epoch,
            budget: self.budget,
            fingerprint: self.fingerprint,
            reserved: [0u8; 20],
        }
    }

    /// Seal the row into a fresh sidecar block. Returns the leaked
    /// `'static` sealed block (the sidecar lifetime law: bounded count,
    /// host/test tier — see [`TapSink`] docs).
    ///
    /// # Errors
    /// [`TapRefusal::ResealShape`] if the real writer refuses the slab.
    #[cfg(test)]
    fn seal_static(&self) -> Result<&'static SealedBlock<'static>, TapRefusal> {
        let values = row_values(self);
        let lanes = [&values[..]];
        let slab = SlabView { lanes: &lanes, rows: TAP_ROW_BYTES as u32, bits: 8 };
        let buf: &'static mut StaticPageBuffer<4096> =
            Box::leak(Box::new(StaticPageBuffer::<4096>::new()));
        let mut writer = buf.writer();
        writer.write_slab(&slab).map_err(|_| TapRefusal::ResealShape)?;
        let sealed: &'static SealedBlock<'static> =
            Box::leak(Box::new(writer.seal(None).map_err(|_| TapRefusal::ResealShape)?));
        Ok(sealed)
    }

    /// Re-seal the row into a stack buffer and return its committed
    /// bytes (the row<->chain binding probe: deterministic writer, so
    /// the same row always yields byte-identical block bytes).
    ///
    /// # Errors
    /// [`TapRefusal::ResealShape`] if the real writer refuses the slab.
    #[cfg(test)]
    pub(crate) fn reseal_bytes(&self) -> Result<Vec<u8>, TapRefusal> {
        let values = row_values(self);
        let lanes = [&values[..]];
        let slab = SlabView { lanes: &lanes, rows: TAP_ROW_BYTES as u32, bits: 8 };
        let mut buf = StaticPageBuffer::<4096>::new();
        let mut writer = buf.writer();
        writer.write_slab(&slab).map_err(|_| TapRefusal::ResealShape)?;
        let sealed = writer.seal(None).map_err(|_| TapRefusal::ResealShape)?;
        Ok(sealed.bytes().to_vec())
    }
}

/// The sidecar sink: consumes tapped rows and appends each one into a
/// sidecar-owned [`RingReceiptChain`] as one real sealed block.
///
/// BLOCK LIFETIME LAW (disclosed): each row is sealed into a
/// `Box::leak`-ed [`StaticPageBuffer`] so the [`SealedBlock`] borrows
/// live for the sink's lifetime. This is the receipt-chain court fixture
/// precedent (bounded sidecar, host/test tier) — NOT a hot-path design;
/// a production sidecar would page blocks to its own storage.
#[cfg(test)]
pub(crate) struct TapSink {
    /// The sidecar-owned receipt chain (one link per tapped row).
    chain: RingReceiptChain,
    /// The chain-covered sealed blocks (parallel to the row log).
    blocks: Vec<&'static SealedBlock<'static>>,
    /// The verbatim row log (replay input).
    rows: Vec<TapEvent>,
}

#[cfg(test)]
impl TapSink {
    /// An empty sink.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            chain: RingReceiptChain::new(),
            blocks: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Total rows sunk (= chain length).
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    /// The verbatim row log (replay input).
    #[must_use]
    pub(crate) fn rows(&self) -> &[TapEvent] {
        &self.rows
    }

    /// The chain-covered sealed blocks (parallel to [`TapSink::rows`]).
    #[must_use]
    pub(crate) fn blocks(&self) -> &[&'static SealedBlock<'static>] {
        &self.blocks
    }

    /// Append one tapped row: seal it into a real block, chain-append
    /// (the license gate + sealer cross-check run at the REAL append),
    /// and log the row. On refusal the sink is UNCHANGED — a refused
    /// row mints nothing (purity law at the sink boundary too).
    ///
    /// # Errors
    /// [`TapRefusal::Spoliation`] when the chain's append gate refuses
    /// (license or seal verdict), mapped 1:1 from the chain's opaque
    /// refusal class.
    pub(crate) fn append(
        &mut self,
        row: &TapEvent,
        license: &P28LicenseAttestation,
    ) -> Result<(), TapRefusal> {
        let sealed = row.seal_static()?;
        self.chain
            .append(sealed, license)
            .map_err(|_| TapRefusal::Spoliation)?;
        self.blocks.push(sealed);
        self.rows.push(*row);
        Ok(())
    }

    /// THE REPLAY COURT CORE. Gates, in order:
    ///
    /// 1. [`RingReceiptChain::verify`] over the live committed bytes —
    ///    any tamper is fail-closed [`TapRefusal::Spoliation`],
    /// 2. row<->chain binding: every logged row re-seals byte-exactly
    ///    to its chain-covered block bytes,
    /// 3. the oracle walk in (epoch, ordinal) order through
    ///    [`lawful_step`] (re-derived from `session.rs:453-519`) with
    ///    the ordinal-step-1, counted-budget, and epoch-constancy laws.
    ///
    /// # Errors
    /// Typed [`TapRefusal`] — see the purity-law docs on this module.
    pub(crate) fn replay(&self, license: &P28LicenseAttestation) -> Result<(), TapRefusal> {
        // GATE 1: chain invariant over the live committed bytes.
        match self.chain.verify(&self.blocks, license) {
            Ok(()) => {}
            Err(ReceiptChainRefusal::EmptyChain) => return Err(TapRefusal::EmptyChain),
            Err(ReceiptChainRefusal::Spoliation) => return Err(TapRefusal::Spoliation),
        }
        if self.rows.len() != self.chain.len() {
            return Err(TapRefusal::RowArityMismatch);
        }
        // GATE 2: every logged row re-seals to its chain-covered bytes.
        for (i, row) in self.rows.iter().enumerate() {
            let bytes = row.reseal_bytes()?;
            if bytes != self.blocks[i].bytes() {
                return Err(TapRefusal::RowBinding {
                    index: self.chain.receipts()[i].index,
                });
            }
        }
        // GATE 3: the oracle walk. Rows arrive in sink order = walk
        // order; (epoch, ordinal) is the logical sort key by law, and
        // the epoch is constant across a lawful walk, so sink order IS
        // (epoch, ordinal) order — enforced by the epoch law below.
        let mut from: u8 = 0; // SessionState::HandshakeInit
        // Rows mint AFTER the accepted step, so the first row carries
        // the ordinal AFTER the first advance (1 for a fresh session).
        let mut expect_ordinal: Option<u32> = None;
        let mut expect_budget: u32 = 0;
        let mut epoch: Option<u16> = None;
        for row in &self.rows {
            if let Some(e) = epoch {
                if row.epoch != e {
                    return Err(TapRefusal::EpochLaw {
                        expected: e,
                        got: row.epoch,
                    });
                }
            } else {
                epoch = Some(row.epoch);
            }
            let want_ord = expect_ordinal.unwrap_or(row.ordinal);
            if row.ordinal != want_ord {
                return Err(TapRefusal::OrdinalLaw {
                    expected: want_ord,
                    got: row.ordinal,
                });
            }
            // Purity-as-replay: an accepted step advances EXACTLY 1.
            expect_ordinal = Some(want_ord.wrapping_add(1));
            // Transition oracle: (from, kind) -> to, session.rs:453-519.
            let to = lawful_step(from, row.kind).ok_or(TapRefusal::IllegalTransition {
                from,
                kind: row.kind,
                got: row.state,
            })?;
            if row.state != to {
                return Err(TapRefusal::IllegalTransition {
                    from,
                    kind: row.kind,
                    got: row.state,
                });
            }
            // Budget law (counted, never timed — session.rs law 3).
            let want = match row.kind {
                2 => expect_budget + 1, // AppData: count +1
                3 => 0,                 // Rekey: reset
                _ => expect_budget,     // handshake/close: unchanged
            };
            if row.budget != want {
                return Err(TapRefusal::BudgetLaw {
                    expected: want,
                    got: row.budget,
                });
            }
            expect_budget = want;
            from = to;
        }
        Ok(())
    }
}
