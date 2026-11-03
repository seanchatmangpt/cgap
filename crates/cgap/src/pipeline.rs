//! SP1 lane: enclave self-play pipeline wiring (thin types, real subsystem
//! calls behind them).
//!
//! Construction only. Stage *methods* that drive the subsystems land with
//! the SP test lanes; this module fixes the wiring seams so those lanes
//! write against a stable surface.
//!
//! Subsystem seams (all real, zero mocks):
//! - gate:   [`cgap_gate::penstock::ingress`] (A04 CgapFrameHeader, 128 B)
//! - p28:    [`cgap_p28::attestation::P28LicenseAttestation`] (A17, 80 B)
//! - blake3: [`cgap_blake3::hash`] (free fn)
//! - arrow:  [`cgap_arrow::bitmask::PresenceMask`] column presence
//! - bn3:    [`cgap_bn3::unify::RuleCompiler`] + projector
//! - parquet: [`cgap_parquet::footer::FooterSealer`] (factory-closed per stage)
//! - git:    [`cgap_git::write_pack_v2`] (factory-closed per stage)
//! - storage: [`cgap_storage::dred::cascade_retract`] (REP4 cascade)
//! - core:   [`cgap_core::frame`] A04 frame types

use core::marker::PhantomData;

/// Pipeline stages in sovereign execution order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineStage {
    /// Penstock ingress (A04 frame admission).
    Ingress,
    /// P28 license attestation validation (A17).
    Attestation,
    /// BLAKE3 content hash.
    B3Hash,
    /// Arrow column presence build.
    ColumnBuild,
    /// BN3 rule compilation + projection.
    RuleCompile,
    /// Parquet block seal.
    Seal,
    /// Git pack write.
    PackWrite,
    /// DRed cascade retraction (cgap-storage, REP4).
    CascadeRetract,
}

/// Typed pipeline admission refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineError {
    /// P28 attestation failed intrinsic validation.
    AttestationRefused,
}

/// Enclave self-play pipeline: holds real subsystem handles.
///
/// `LIFE` carries the P28 license; construction refuses (typed, no panic)
/// when the attestation fails `P28LicenseAttestation::validate_intrinsic`.
pub struct EnclavePipeline<'att> {
    /// P28 attestation (A17, 80 B) — pipeline admission proof.
    attestation: &'att cgap_p28::attestation::P28LicenseAttestation,
    /// BN3 rule compiler (unit struct, `Copy`).
    rule_compiler: cgap_bn3::unify::RuleCompiler,
    /// BN3 consequent projector (unit struct).
    projector: cgap_bn3::project::ConsequentProjector,
    /// Arrow column presence mask (256 rows = 32 bytes, LSB-first).
    presence: cgap_arrow::bitmask::PresenceMask<256, 32>,
    /// Penstock ingress entry (A19 sovereign entrypoint seam).
    ingress: for<'b> fn(&'b [u8], usize, Option<&mut cgap_gate::penstock::FrameView<'b>>) -> u32,
    /// BLAKE3 hash entry.
    hash: fn(&[u8]) -> [u8; 32],
    /// Git pack write entry.
    pack_write: for<'a> fn(&[&'a [u8]]) -> Result<alloc::vec::Vec<u8>, cgap_git::GitError>,
    /// Parquet sealer seam (factory-closed at the sealing stage).
    _seal: PhantomData<cgap_parquet::footer::FooterSealer<'static>>,
    /// DRed cascade scratch seam (caller-leased, zero heap).
    _cascade: PhantomData<&'static ()>,
}

impl<'att> EnclavePipeline<'att> {
    /// Wire an [`EnclavePipeline`] against a real P28 attestation.
    ///
    /// Refuses (typed [`PipelineError::AttestationRefused`]) when the
    /// attestation fails intrinsic validation — no panic on the admission
    /// path.
    pub fn new(
        attestation: &'att cgap_p28::attestation::P28LicenseAttestation,
    ) -> Result<Self, PipelineError> {
        if attestation.validate_intrinsic().is_err() {
            return Err(PipelineError::AttestationRefused);
        }
        Ok(Self {
            attestation,
            rule_compiler: cgap_bn3::unify::RuleCompiler,
            projector: cgap_bn3::project::ConsequentProjector::new(),
            presence: cgap_arrow::bitmask::PresenceMask::EMPTY,
            ingress: cgap_gate::penstock::ingress,
            hash: cgap_blake3::hash,
            pack_write: cgap_git::write_pack_v2,
            _seal: core::marker::PhantomData,
            _cascade: core::marker::PhantomData,
        })
    }

    /// The admitted license (A17 bytes).
    pub fn attestation(&self) -> &cgap_p28::attestation::P28LicenseAttestation {
        self.attestation
    }

    /// Stage handles, in sovereign order.
    pub fn stage(&self, stage: PipelineStage) -> StageHandle<'_, 'att> {
        StageHandle {
            pipeline: self,
            stage,
        }
    }

    /// BN3 rule compiler (real handle).
    pub fn rule_compiler(&self) -> &cgap_bn3::unify::RuleCompiler {
        &self.rule_compiler
    }

    /// BN3 consequent projector (real handle).
    pub fn projector(&self) -> &cgap_bn3::project::ConsequentProjector {
        &self.projector
    }

    /// Arrow presence mask (real handle).
    pub fn presence(&self) -> &cgap_arrow::bitmask::PresenceMask<256, 32> {
        &self.presence
    }

    /// The BLAKE3 hash entry (real `fn` pointer, same surface as
    /// `Self::ingress_fn`).
    pub fn hash_fn(&self) -> fn(&[u8]) -> [u8; 32] {
        self.hash
    }

    /// The git pack-write entry (real `fn` pointer, same surface as
    /// `Self::ingress_fn`).
    pub fn pack_write_fn(
        &self,
    ) -> for<'a> fn(&[&'a [u8]]) -> Result<alloc::vec::Vec<u8>, cgap_git::GitError> {
        self.pack_write
    }
}

/// Borrowed view of one pipeline stage's real subsystem handle.
#[derive(Clone, Copy)]
pub struct StageHandle<'p, 'att> {
    pipeline: &'p EnclavePipeline<'att>,
    stage: PipelineStage,
}

impl<'p, 'att> StageHandle<'p, 'att> {
    /// The stage's ordinal in sovereign execution order.
    pub fn ordinal(&self) -> usize {
        match self.stage {
            PipelineStage::Ingress => 0,
            PipelineStage::Attestation => 1,
            PipelineStage::B3Hash => 2,
            PipelineStage::ColumnBuild => 3,
            PipelineStage::RuleCompile => 4,
            PipelineStage::Seal => 5,
            PipelineStage::PackWrite => 6,
            PipelineStage::CascadeRetract => 7,
        }
    }

    /// Probe penstock ingress (out=None) and return the A19 status word;
    /// 0 is the sole success status (A19 CgapStatus).
    pub fn ingress_status(&self, bytes: &[u8], len: usize) -> u32 {
        (self.pipeline.ingress)(bytes, len, None)
    }

    /// The raw gate ingress entry (full `Option<&mut FrameView>` surface
    /// for stage lanes that build a real view slot).
    pub fn ingress_fn(
        &self,
    ) -> for<'b> fn(&'b [u8], usize, Option<&mut cgap_gate::penstock::FrameView<'b>>) -> u32 {
        self.pipeline.ingress
    }
}

#[cfg(test)]
mod wiring_courts {
    use super::*;

    fn root_att(digest: [u8; 32]) -> cgap_p28::attestation::P28LicenseAttestation {
        cgap_p28::attestation::P28LicenseAttestation {
            license_magic: cgap_p28::attestation::LICENSE_MAGIC,
            version: cgap_p28::attestation::ATTESTATION_VERSION,
            attestation_flags: cgap_p28::attestation::FLAG_ROOT,
            safe_harbor_ceiling_cents: cgap_p28::attestation::SAFE_HARBOR_CEILING_MAX_CENTS,
            origin_authority_digest: digest,
            upstream_receipt: [0u8; 32],
        }
    }

    /// ADMIT: a valid root license wires the whole pipeline.
    #[test]
    fn admits_root_license() {
        let att = root_att([7u8; 32]);
        let pip = EnclavePipeline::new(&att).expect("root license admits");
        assert_eq!(
            pip.attestation(),
            &att,
            "the wired pipeline carries the admitted A17 record verbatim"
        );
        // Stage ordinals are the sovereign order (A19 entrypoint sequence).
        assert_eq!(pip.stage(PipelineStage::Ingress).ordinal(), 0);
        assert_eq!(pip.stage(PipelineStage::CascadeRetract).ordinal(), 7);
    }

    /// NAMED MUTATION (anti-vacuity): corrupt the license magic — the
    /// constructor MUST refuse with the typed PipelineError. A constructor
    /// that admits a spoliated license flips this court to FAIL.
    #[test]
    fn refuses_spoliated_license() {
        let mut att = root_att([7u8; 32]);
        att.license_magic = *b"P28X"; // mutation: magic corrupted
        assert!(
            matches!(
                EnclavePipeline::new(&att),
                Err(PipelineError::AttestationRefused)
            ),
            "spoliated magic must be a typed refusal, not an admit"
        );
    }

    /// The real gate entry runs through the wiring: a well-formed A04
    /// header prefix ADMITs; flipping the magic byte (mutation 2) must
    /// flip the status to the INVALID_MAGIC class.
    #[test]
    fn gate_ingress_admits_then_mutates_to_reject() {
        let att = root_att([7u8; 32]);
        let pip = EnclavePipeline::new(&att).expect("root license admits");
        let mut frame = [0u8; 256];
        frame[0..4].copy_from_slice(b"CGAP");
        frame[4] = 1; // A04 version
        frame[8..16].copy_from_slice(&128u64.to_le_bytes()); // payload len <= 128
        frame[16] = 0xA7; // O-ABI digest field present (nonzero; gate is a limiter)
                          // A17 attestation span: penstock OFF_ATTESTATION = 48, 80 B.
        frame[48..52].copy_from_slice(b"P28S");
        frame[52..54].copy_from_slice(&1u16.to_le_bytes()); // version
        frame[54..56].copy_from_slice(&1u16.to_le_bytes()); // FLAG_ROOT lineage
        frame[56..64].copy_from_slice(&5_000_000_000u64.to_le_bytes()); // ceiling
                                                                        // digest 64..96 + receipt 96..128 stay zero = ROOT (receipt-zero law)
                                                                        // EXCEPT the O-ABI digest field in the header (offset 16) is the
                                                                        // nonzero one; the ROOT receipt must stay zero — A17 §2.2.
        let baseline = pip
            .stage(PipelineStage::Ingress)
            .ingress_status(&frame, frame.len());
        assert_eq!(
            baseline, 0,
            "well-formed frame prefix must ADMIT (status 0)"
        );
        frame[0] = b'X'; // mutation 2: magic corrupted
        let mutated = pip
            .stage(PipelineStage::Ingress)
            .ingress_status(&frame, frame.len());
        assert_ne!(mutated, 0, "corrupted magic must NOT admit");
    }

    /// Real BLAKE3 through the wiring: digest matches the free-fn kernel.
    #[test]
    fn blake3_wiring_matches_kernel() {
        let att = root_att([7u8; 32]);
        let pip = EnclavePipeline::new(&att).expect("root license admits");
        let bytes = b"cgap self-play pipeline";
        let via_pipeline = pip.stage(PipelineStage::B3Hash);
        let _ = via_pipeline;
        // The pipeline's hash seam IS the kernel's own fn — call through
        // the wiring accessor and assert byte equality with the free fn.
        let h1 = (pip.hash_fn())(bytes);
        let h2 = cgap_blake3::hash(bytes);
        assert_eq!(h1, h2);
    }

    /// The pack-write wiring seam is the real `cgap_git::write_pack_v2`
    /// fn pointer (anti-vacuity: a seam pointing anywhere else flips).
    #[test]
    fn pack_write_wiring_is_kernel_entry() {
        let att = root_att([7u8; 32]);
        let pip = EnclavePipeline::new(&att).expect("root license admits");
        let seam = pip.pack_write_fn();
        let kernel: for<'a> fn(&[&'a [u8]]) -> Result<alloc::vec::Vec<u8>, cgap_git::GitError> =
            cgap_git::write_pack_v2;
        assert!(
            core::ptr::fn_addr_eq(seam, kernel),
            "pack_write seam must be the kernel entry"
        );
    }
}
