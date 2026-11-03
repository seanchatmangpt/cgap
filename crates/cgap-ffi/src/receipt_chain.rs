//! Lane p2-receipts — ring-commit receipt chaining witness.
//!
//! Per-transition BLAKE3 Merkle receipts over the W3-L4 ring commit leg:
//! every committed sealed block appends one receipt to a hash chain
//! (genesis -> r0 -> r1 -> ...), and each receipt is bound to the P28
//! license verify — an attestation that fails `validate_intrinsic` is
//! refused at append time, and the license record's wire bytes are a
//! hash input, so a post-hoc license tamper flips chain verification.
//!
//! Chain law (p28 §5 shape): the genesis prev-hash is the all-zero
//! digest; receipt i binds
//! `prev_hash || sealed block bytes || license wire bytes`; every
//! tamper — of a stored receipt field, of a committed block, or of the
//! license record — collapses to ONE opaque refusal class
//! (`ReceiptChainRefusal::Spoliation`), mirroring p28's single halt
//! class (attestation.rs §5 preamble: no failure fingerprinting).
//!
//! PROSE-ONLY FORMULA (XC-03, honest status): the PRD equation
//! `ΔS_admitted + Φ_lease ≡ R_receipt (mod Σ_PQC)` has NO code referent
//! here. `Σ_PQC` is INVENTED LORE — zero Rust hits tree-wide (audit
//! `docs/jira/v26.11.2/audit/cgapprd-02-equation-grounding.md:47-55`;
//! redteam `red-13-receipt-desync-exploit.md:11-18`); ΔS/Φ live only as
//! cgap-git commit trailers (commit_synth.rs), not in any block payload
//! this chain hashes. Per dispatch law the missing quantities were NOT
//! invented. What IS code: the real link law
//! `R_i = BLAKE3(prev_hash || block_bytes || license_bytes)` and the
//! strict per-link position invariant — `index` strictly increasing with
//! zero gaps and `prev_hash == prior link's hash` — enforced in
//! `verify` (court: `chain_position_monotonicity_invariant`).
//!
//! Host surface only: the XIP `target_os = "none"` build keeps the
//! pre-existing dep set (cgap-blake3/cgap-p28/cgap-parquet are
//! cfg(not(target_os = "none")) dependencies of this crate).
#![cfg(not(target_os = "none"))]

use cgap_blake3::hash;
use cgap_p28::attestation::P28LicenseAttestation;
use cgap_parquet::footer::SealedBlock;

/// The all-zero genesis prev-hash (chain anchor).
pub const GENESIS: [u8; 32] = [0u8; 32];

/// One per-transition receipt: the Merkle link over the sealed block
/// bytes and the license record's wire bytes, chained to the previous
/// receipt's hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChainReceipt {
    /// 0-based position in the chain (also part of no hash input — the
    /// prev-hash linkage carries the ordering, not this counter).
    pub index: u64,
    /// BLAKE3 of the committed sealed-block bytes.
    pub block_digest: [u8; 32],
    /// BLAKE3 of the license attestation's 80-byte wire form.
    pub license_digest: [u8; 32],
    /// Hash of the previous receipt (`GENESIS` at index 0).
    pub prev_hash: [u8; 32],
    /// `BLAKE3(prev_hash || block bytes || license bytes)`.
    pub hash: [u8; 32],
}

/// The single opaque chain-refusal class. `EmptyChain` is a shape
/// verdict (nothing to verify); every tamper or license failure —
/// stored receipt field, committed block bytes, license record — maps
/// to `Spoliation`, never fingerprinting WHICH check failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptChainRefusal {
    /// `verify` called on a chain with zero receipts.
    EmptyChain,
    /// Any tamper or license invariant violation (opaque, p28 §5 law).
    Spoliation,
}

impl core::fmt::Display for ReceiptChainRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyChain => f.write_str("receipt chain empty"),
            Self::Spoliation => f.write_str("receipt chain spoliation (fail-closed)"),
        }
    }
}

/// The per-lane receipt chain over the ring commit leg.
#[derive(Clone, Debug, Default)]
pub struct RingReceiptChain {
    receipts: Vec<ChainReceipt>,
}

impl RingReceiptChain {
    pub fn new() -> Self {
        Self {
            receipts: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    pub fn receipts(&self) -> &[ChainReceipt] {
        &self.receipts
    }

    /// Mutable receipt access — audit/repair tooling seam (the tamper
    /// court drives mutations through this seam; production append is
    /// the only writer on the hot path).
    pub fn receipts_mut(&mut self) -> &mut [ChainReceipt] {
        &mut self.receipts
    }
}

impl RingReceiptChain {
    /// The per-transition link function: `BLAKE3(prev_hash || block
    /// bytes || license wire bytes)`.
    fn link(prev_hash: &[u8; 32], block: &[u8], license: &[u8; 80]) -> [u8; 32] {
        hash(&[prev_hash.as_slice(), block, license.as_slice()].concat())
    }

    /// Append one receipt for `sealed`, bound to `license`. Gates, in
    /// order (any refusal leaves the chain UNCHANGED — append is the
    /// only writer):
    ///
    /// 1. License verify gate: the attestation MUST pass
    ///    `validate_intrinsic` (typed refusal). RED-13 §4 repair.
    /// 2. Sealer cross-check: `cgap_parquet::footer::verify_seal`
    ///    re-runs the FULL block verdict (shape -> attestation slot ->
    ///    FNV-1a-64 checksum -> sealer content digest) over the live
    ///    committed bytes and the sealer's own digest. The chain's
    ///    BLAKE3 link alone cannot detect a post-seal byte tamper
    ///    (it hashes whatever bytes it is handed); the FNV checksum +
    ///    content digest the sealer recorded CAN — so a
    ///    tampered-bytes/sealer-digest disagreement is Spoliation,
    ///    never trusted (RED-13 §4 repair; court:
    ///    `append_cross_checks_sealer_digest`).
    ///
    /// COMMIT-LEG BOUNDARY (stated, RED-13 §3): `RingReceiptChain` is
    /// NOT wired into `cgap_commit_parquet` (sovereign.rs) — that path
    /// seals with `seal(None)` (no P28 attestation surface), keeps its
    /// lease thread-local with no chain handle, and has no FFI return
    /// path for receipts; wiring it is not a small coherent diff and
    /// was NOT scaffolded by FIX-08. Any "receipts bind commits"
    /// non-repudiation claim over the commit leg is UNSUPPORTED as
    /// implemented — the chain is a host witness type only.
    pub fn append(
        &mut self,
        sealed: &SealedBlock,
        license: &P28LicenseAttestation,
    ) -> Result<[u8; 32], ReceiptChainRefusal> {
        // License verify gate — bind to the real P28 verify path.
        if license.validate_intrinsic().is_err() {
            return Err(ReceiptChainRefusal::Spoliation);
        }
        let bytes = sealed.bytes();
        // Sealer cross-check (RED-13 §4 repair): the FULL block verdict —
        // shape -> attestation slot -> FNV checksum -> sealer content
        // digest — re-run over the live committed bytes. The chain's own
        // BLAKE3 link hashes whatever bytes it is handed, so it cannot see
        // a post-seal tamper; the sealer's recorded FNV checksum/digest
        // can. Disagreement is Spoliation, chain unchanged.
        if cgap_parquet::footer::verify_seal(bytes, sealed.digest()).is_err() {
            return Err(ReceiptChainRefusal::Spoliation);
        }
        let license_bytes = license.to_bytes();
        // NOTE: the sealer's own digest (SealedBlock::digest) is the
        // FNV-1a-64 four-lane content address (footer.rs:34-38) —
        // deliberately NON-cryptographic, so it is NOT the chain
        // digest. The chain records the recomputed BLAKE3 over the
        // committed bytes (collision-resistant binding), which
        // verify() recomputes the same way.
        let license_digest = hash(&license_bytes);
        let prev_hash = self.receipts.last().map(|r| r.hash).unwrap_or(GENESIS);
        let block_digest = hash(bytes);
        let link = Self::link(&prev_hash, bytes, &license_bytes);
        self.receipts.push(ChainReceipt {
            index: self.receipts.len() as u64,
            block_digest,
            license_digest,
            prev_hash,
            hash: link,
        });
        Ok(link)
    }

    /// Verify the whole chain against the committed blocks and the
    /// license record the chain is bound to. Recomputes every link
    /// (genesis anchor, prev-hash linkage, block digests over the
    /// actual block bytes, license digest over the live license wire
    /// bytes) plus the STRICT POSITION INVARIANT (XC-03): receipt i
    /// must carry `index == i` (strictly increasing, zero gaps) and
    /// `prev_hash == prior link's hash` (genesis anchor at i = 0); any
    /// mismatch is the one opaque refusal. A post-hoc license tamper
    /// fails either the intrinsic verify gate or the link recomputation.
    pub fn verify(
        &self,
        blocks: &[&SealedBlock],
        license: &P28LicenseAttestation,
    ) -> Result<(), ReceiptChainRefusal> {
        if self.receipts.is_empty() {
            return Err(ReceiptChainRefusal::EmptyChain);
        }
        if license.validate_intrinsic().is_err() {
            return Err(ReceiptChainRefusal::Spoliation);
        }
        if blocks.len() != self.receipts.len() {
            return Err(ReceiptChainRefusal::Spoliation);
        }
        let license_bytes = license.to_bytes();
        if hash(&license_bytes) != self.receipts[0].license_digest {
            return Err(ReceiptChainRefusal::Spoliation);
        }
        let mut prev = GENESIS;
        for (i, (r, b)) in self.receipts.iter().zip(blocks.iter()).enumerate() {
            let bytes = b.bytes();
            // Strict position invariant (XC-03): zero-gap monotone
            // index + prev-hash linkage. A rewound, gap-spliced, or
            // duplicated position is Spoliation even though every
            // digest in the receipt is internally consistent.
            if r.index != i as u64 || r.prev_hash != prev {
                return Err(ReceiptChainRefusal::Spoliation);
            }
            if r.block_digest != hash(bytes)
                || r.block_digest != hash(bytes)
                || r.license_digest != hash(&license_bytes)
                || r.hash != Self::link(&prev, bytes, &license_bytes)
            {
                return Err(ReceiptChainRefusal::Spoliation);
            }
            prev = r.hash;
        }
        Ok(())
    }
}

impl core::fmt::Display for RingReceiptChain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RingReceiptChain({} receipts)", self.receipts.len())
    }
}

// ===========================================================================
// Courts (Chicago: real collaborators — real sealed blocks from the real
// A02 writer, the real P28 attestation record, the real BLAKE3 kernel;
// assert final state; no mocks). Anti-vacuity: the tamper court runs its
// own negative control every run — the PRISTINE chain must verify OK, so
// an always-failing verify is a VACUOUS GATE, not a pass. Mutations run
// on in-memory copies of receipts only; the shared tree is never mutated.
// ===========================================================================
#[cfg(test)]
mod courts {
    use super::{ReceiptChainRefusal, RingReceiptChain, GENESIS};
    use cgap_p28::attestation::{
        P28LicenseAttestation, ATTESTATION_VERSION, FLAG_ROOT, LICENSE_MAGIC,
        SAFE_HARBOR_CEILING_MAX_CENTS,
    };
    use cgap_parquet::footer::SealedBlock;
    use cgap_parquet::mmap_writer::{SlabView, StaticPageBuffer};

    /// The valid Root license record used by every positive leg.
    fn root_license() -> P28LicenseAttestation {
        P28LicenseAttestation {
            license_magic: LICENSE_MAGIC,
            version: ATTESTATION_VERSION,
            attestation_flags: FLAG_ROOT,
            safe_harbor_ceiling_cents: 1_000_000_000,
            origin_authority_digest: [7u8; 32],
            upstream_receipt: [0u8; 32],
        }
    }

    /// Build `n` REAL sealed blocks (real writer, real seal) with
    /// distinct payloads and a chain appending all of them. Blocks are
    /// leaked `&'static` buffers so the `SealedBlock` borrows live for
    /// the court (bounded count, test-only).
    fn fixture(n: usize) -> (RingReceiptChain, Vec<&'static SealedBlock<'static>>) {
        let mut chain = RingReceiptChain::new();
        let mut blocks: Vec<&'static SealedBlock<'static>> = Vec::new();
        for i in 0..n {
            let buf: &'static mut StaticPageBuffer<65_536> =
                Box::leak(Box::new(StaticPageBuffer::<65_536>::new()));
            let lane_data: [u32; 4] = [
                i as u32,
                ((i + 1) % 256) as u32,
                ((i + 2) % 256) as u32,
                ((i + 3) % 256) as u32,
            ];
            let mut writer = buf.writer();
            let slab = SlabView {
                lanes: &[&lane_data[..]],
                rows: 4,
                bits: 8,
            };
            writer.write_slab(&slab).expect("fixture slab writes");
            let sealed: &'static SealedBlock<'static> =
                Box::leak(Box::new(writer.seal(None).expect("fixture seal")));
            chain
                .append(sealed, &root_license())
                .expect("fixture append");
            blocks.push(sealed);
        }
        (chain, blocks)
    }

    /// Positive leg + structural linkage asserts (genesis anchor,
    /// prev-hash chaining, per-receipt digest identity).
    #[test]
    fn chain_appends_and_verifies() {
        let (chain, blocks) = fixture(4);
        assert_eq!(chain.len(), 4);
        let rs = chain.receipts();
        assert_eq!(rs[0].prev_hash, GENESIS);
        for i in 0..4 {
            assert_eq!(rs[i].block_digest, cgap_blake3::hash(blocks[i].bytes()));
            if i > 0 {
                assert_eq!(rs[i].prev_hash, rs[i - 1].hash);
            }
        }
        let license = root_license();
        assert_eq!(chain.verify(&blocks, &license), Ok(()));
    }

    /// THE TAMPER COURT: every stored-field mutation, every committed
    /// block-byte mutation, and a post-hoc license mutation each flip
    /// to the ONE typed refusal class. Negative control first: the
    /// pristine chain verifies OK (a court that cannot flip is FAIL;
    /// a verify that always refuses is a VACUOUS GATE).
    #[test]
    fn tampered_receipt_or_block_is_typed_refusal() {
        let license = root_license();
        let (chain, blocks) = fixture(3);

        // Negative control (non-vacuity): pristine passes.
        assert_eq!(chain.verify(&blocks, &license), Ok(()));

        // Mutation battery over stored receipt fields (copies only).
        let mut m = chain.clone();
        m.receipts_mut()[1].hash[0] ^= 0xFF;
        assert_eq!(
            m.verify(&blocks, &license),
            Err(ReceiptChainRefusal::Spoliation)
        );
        let mut m = chain.clone();
        m.receipts_mut()[1].block_digest[5] ^= 0x01;
        assert_eq!(
            m.verify(&blocks, &license),
            Err(ReceiptChainRefusal::Spoliation)
        );
        let mut m = chain.clone();
        m.receipts_mut()[1].prev_hash[7] ^= 0x80;
        assert_eq!(
            m.verify(&blocks, &license),
            Err(ReceiptChainRefusal::Spoliation)
        );
        let mut m = chain.clone();
        m.receipts_mut()[1].license_digest[11] ^= 0x40;
        assert_eq!(
            m.verify(&blocks, &license),
            Err(ReceiptChainRefusal::Spoliation)
        );

        let mut swapped: Vec<&SealedBlock> = blocks.clone();
        swapped.swap(0, 1);
        assert_eq!(
            chain.verify(&swapped, &license),
            Err(ReceiptChainRefusal::Spoliation)
        );

        // Post-hoc license tamper: intrinsic-valid record whose bytes
        // differ from the one the receipts were minted under.
        let mut tampered_license = root_license();
        tampered_license.origin_authority_digest[0] ^= 0x01;
        assert_eq!(
            chain.verify(&blocks, &tampered_license),
            Err(ReceiptChainRefusal::Spoliation)
        );
        // License that fails intrinsic verify outright (bad magic).
        let mut bad = root_license();
        bad.license_magic = *b"XXXX";
        assert_eq!(
            chain.verify(&blocks, &bad),
            Err(ReceiptChainRefusal::Spoliation)
        );
    }

    /// License verify gates APPEND: an attestation failing
    /// validate_intrinsic is refused at mint time, chain unchanged.
    #[test]
    fn invalid_license_refused_at_append() {
        let mut chain = RingReceiptChain::new();
        let (_, blocks) = fixture(1); // blocks minted; chain separate
        let mut bad = root_license();
        bad.license_magic = *b"XXXX";
        assert_eq!(
            chain.append(blocks[0], &bad),
            Err(ReceiptChainRefusal::Spoliation)
        );
        assert!(chain.is_empty(), "refused append must not mint a receipt");

        let mut over = root_license();
        over.safe_harbor_ceiling_cents = SAFE_HARBOR_CEILING_MAX_CENTS + 1;
        assert_eq!(
            chain.append(blocks[0], &over),
            Err(ReceiptChainRefusal::Spoliation)
        );
        assert!(chain.is_empty());
    }

    /// Shape verdict: verify on an empty chain is EmptyChain, not
    /// Spoliation (nothing to verify is not tampering).
    #[test]
    fn empty_chain_verifies_as_empty() {
        let chain = RingReceiptChain::new();
        assert_eq!(
            chain.verify(&[], &root_license()),
            Err(ReceiptChainRefusal::EmptyChain)
        );
    }

    /// RED-13 §4 repair court — the sealer cross-check at append.
    /// Append must run `verify_seal(bytes, sealed.digest())` and refuse
    /// typed when the committed bytes disagree with the sealer's own
    /// FNV checksum/content digest: the chain's BLAKE3 link alone hashes
    /// whatever bytes it is handed, so WITHOUT the cross-check a
    /// post-seal tamper mints a receipt that internally verifies (the
    /// vacuous pass this court exists to kill).
    ///
    /// Anti-vacuity mutation (in-court, data-level): the tamper arm
    /// flips one committed byte through the block buffer's live bytes;
    /// the checksum channel of verify_seal must fire. Baseline arm:
    /// the PRISTINE block appends and verifies (an always-refusing
    /// append is a VACUOUS GATE, not a pass). The full structural flip
    /// witness (removing the verify_seal call from append) was also run
    /// this session as a /tmp sandbox mutation with sha256-exact
    /// restore + mtime bump — see the FIX-08 lane receipt.
    #[test]
    fn append_cross_checks_sealer_digest() {
        let mut buf = StaticPageBuffer::<65_536>::new();
        let lane_data: [u32; 4] = [9, 8, 7, 6];
        let mut writer = buf.writer();
        writer
            .write_slab(&SlabView {
                lanes: &[&lane_data[..]],
                rows: 4,
                bits: 8,
            })
            .expect("fixture slab writes");
        let sealed = writer.seal(None).expect("fixture seal");

        // BASELINE (non-vacuity): the pristine block appends and the
        // chain verifies.
        let mut chain = RingReceiptChain::new();
        chain
            .append(&sealed, &root_license())
            .expect("pristine append");
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.verify(&[&sealed], &root_license()), Ok(()));

        // MUTATION: flip one committed body byte (through the buffer the
        // block borrows — test-only tamper of the sealed artifact).
        let ptr = sealed.bytes().as_ptr() as *mut u8;
        unsafe {
            *ptr.add(2) ^= 0x01;
        }

        // AFTER: the cross-check must refuse typed and the chain must be
        // UNCHANGED (append is the only writer; refusal mints nothing).
        let mut chain2 = RingReceiptChain::new();
        assert_eq!(
            chain2.append(&sealed, &root_license()),
            Err(ReceiptChainRefusal::Spoliation),
            "tampered bytes vs sealer digest must refuse at append"
        );
        assert!(chain2.is_empty(), "refused append must not mint a receipt");
        // And the earlier pristine chain no longer verifies against the
        // mutated bytes either — tamper detection is total.
        assert_eq!(
            chain.verify(&[&sealed], &root_license()),
            Err(ReceiptChainRefusal::Spoliation)
        );
    }

    /// Lease-exhaustion court: the 64 KiB commit lease holds exactly
    /// 65,376 lanes; a larger call must refuse typed
    /// (CGAP_ERR_LEASE_EXHAUSTED), never truncate. Drives the REAL
    /// `cgap_commit_parquet` body over the ceiling.
    #[test]
    fn lease_exhaustion_is_typed_refusal() {
        use crate::sovereign::{cgap_commit_parquet, CGAP_ERR_LEASE_EXHAUSTED, CGAP_OK};
        let path = std::ffi::CString::new("/tmp/p2-lease-court.par2").unwrap();
        let lanes: Vec<i32> = (0..16).map(|r| (r % 256) as i32).collect();
        // Grant leg: an in-ceiling call commits OK (lease granted).
        let st = unsafe { cgap_commit_parquet(lanes.as_ptr(), lanes.len(), path.as_ptr()) };
        assert_eq!(st, CGAP_OK, "in-ceiling commit must commit");
        // Over-ceiling leg: 70,000 lanes exceeds the 64 KiB lease.
        let over: Vec<i32> = (0..70_000).map(|r| (r % 256) as i32).collect();
        let st = unsafe { cgap_commit_parquet(over.as_ptr(), over.len(), path.as_ptr()) };
        assert_eq!(
            st, CGAP_ERR_LEASE_EXHAUSTED,
            "over-ceiling commit must refuse typed"
        );
    }

    /// XC-03 court — the strict per-link position/monotonicity
    /// invariant (the achievable subset of the prose-only
    /// `ΔS + Φ ≡ R (mod Σ_PQC)` law; Σ_PQC has no code referent,
    /// red-13 §1 / cgapprd-02:47-55). Baseline: k pristine links
    /// verify OK (always-refusing verify = VACUOUS GATE). Mutations
    /// over in-memory receipt copies: (a) rewound index on one link,
    /// (b) gap-spliced index, (c) foreign link inserted mid-chain —
    /// each must be the ONE typed refusal, final state asserted.
    #[test]
    fn chain_position_monotonicity_invariant() {
        let license = root_license();
        let (chain, blocks) = fixture(4);

        // BASELINE (non-vacuity): pristine chain, positions 0..4.
        for (i, r) in chain.receipts().iter().enumerate() {
            assert_eq!(r.index, i as u64, "mint positions must be zero-gap");
        }
        assert_eq!(chain.verify(&blocks, &license), Ok(()));

        // MUTATION A — rewound index: link 1 re-labeled 0. Every digest
        // stays internally consistent; ONLY the position invariant can
        // see this.
        let mut rewound = chain.clone();
        rewound.receipts_mut()[1].index = 0;
        assert_eq!(
            rewound.verify(&blocks, &license),
            Err(ReceiptChainRefusal::Spoliation),
            "rewound link index must refuse typed"
        );

        // MUTATION B — gap: link 2 advanced past its position.
        let mut gap = chain.clone();
        gap.receipts_mut()[2].index = 3;
        assert_eq!(
            gap.verify(&blocks, &license),
            Err(ReceiptChainRefusal::Spoliation),
            "gap-spliced position must refuse typed"
        );

        // MUTATION C — position permutation: links 2 and 3 swap their
        // chain positions (hashes intact, ordering desynced).
        let mut permuted = chain.clone();
        let i2 = permuted.receipts_mut()[2].index;
        permuted.receipts_mut()[2].index = permuted.receipts_mut()[3].index;
        permuted.receipts_mut()[3].index = i2;
        assert_eq!(
            permuted.verify(&blocks, &license),
            Err(ReceiptChainRefusal::Spoliation),
            "position-permuted chain must refuse typed"
        );
    }
}
