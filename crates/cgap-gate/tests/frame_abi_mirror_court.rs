//! Frame-ABI mirror court (CG-1 gate G6, v26.11.3).
//!
//! The Wave-1 frozen mirror structs in `src/penstock.rs`
//! (`CgapFrameHeaderAbi` / `P28LicenseAttestationAbi`) are now diffed
//! against the CANONICAL cgap-core types (`crates/cgap-core/src/frame.rs`
//! `CgapFrameHeader` / `P28LicenseAttestation`) at compile time, per
//! compile-time pin law: any mirror drift is a compile error, loud, never
//! a silent wire ABI divergence. The Wave-1 "off-limits Cargo.toml"
//! deferral is discharged with this court (dev-dependency only; the gate
//! kernel's runtime dep surface stays zero-external).
//!
//! Chicago: the collaborators are the real structs; the court asserts on
//! compiled-layout state (size/align/offset), the strongest form of final
//! state there is.

use cgap_core::frame::{CgapFrameHeader, P28LicenseAttestation};
use cgap_gate::penstock::{CgapFrameHeaderAbi, P28LicenseAttestationAbi};

// Size + align equality: the mirrors are layout-identical to the
// canonical types.
const _: () = assert!(
    core::mem::size_of::<CgapFrameHeaderAbi>() == core::mem::size_of::<CgapFrameHeader>()
);
const _: () = assert!(
    core::mem::align_of::<CgapFrameHeaderAbi>() == core::mem::align_of::<CgapFrameHeader>()
);
const _: () = assert!(
    core::mem::size_of::<P28LicenseAttestationAbi>()
        == core::mem::size_of::<P28LicenseAttestation>()
);
const _: () = assert!(
    core::mem::align_of::<P28LicenseAttestationAbi>()
        == core::mem::align_of::<P28LicenseAttestation>()
);

// Per-field offset equality (A04 §6.1 ladder, all 7 fields).
const _: () = assert!(
    core::mem::offset_of!(CgapFrameHeaderAbi, magic)
        == core::mem::offset_of!(CgapFrameHeader, magic)
);
const _: () = assert!(
    core::mem::offset_of!(CgapFrameHeaderAbi, version)
        == core::mem::offset_of!(CgapFrameHeader, version)
);
const _: () = assert!(
    core::mem::offset_of!(CgapFrameHeaderAbi, flags)
        == core::mem::offset_of!(CgapFrameHeader, flags)
);
const _: () = assert!(
    core::mem::offset_of!(CgapFrameHeaderAbi, reserved)
        == core::mem::offset_of!(CgapFrameHeader, reserved)
);
const _: () = assert!(
    core::mem::offset_of!(CgapFrameHeaderAbi, payload_byte_len)
        == core::mem::offset_of!(CgapFrameHeader, payload_byte_len)
);
const _: () = assert!(
    core::mem::offset_of!(CgapFrameHeaderAbi, oabi_digest)
        == core::mem::offset_of!(CgapFrameHeader, oabi_digest)
);
const _: () = assert!(
    core::mem::offset_of!(CgapFrameHeaderAbi, attestation)
        == core::mem::offset_of!(CgapFrameHeader, attestation)
);

// P28 mirror pair: all 6 field offsets.
const _: () = assert!(
    core::mem::offset_of!(P28LicenseAttestationAbi, license_magic)
        == core::mem::offset_of!(P28LicenseAttestation, license_magic)
);
const _: () = assert!(
    core::mem::offset_of!(P28LicenseAttestationAbi, version)
        == core::mem::offset_of!(P28LicenseAttestation, version)
);
const _: () = assert!(
    core::mem::offset_of!(P28LicenseAttestationAbi, attestation_flags)
        == core::mem::offset_of!(P28LicenseAttestation, attestation_flags)
);
const _: () = assert!(
    core::mem::offset_of!(P28LicenseAttestationAbi, safe_harbor_ceiling_cents)
        == core::mem::offset_of!(P28LicenseAttestation, safe_harbor_ceiling_cents)
);
const _: () = assert!(
    core::mem::offset_of!(P28LicenseAttestationAbi, origin_authority_digest)
        == core::mem::offset_of!(P28LicenseAttestation, origin_authority_digest)
);
const _: () = assert!(
    core::mem::offset_of!(P28LicenseAttestationAbi, upstream_receipt)
        == core::mem::offset_of!(P28LicenseAttestation, upstream_receipt)
);

// Runtime witness (so `cargo test` reports the court, not just a silent
// const pass): a green compile of this file IS the verdict, and the test
// below re-asserts the size facts as runtime truth.
#[test]
fn frame_abi_mirror_is_layout_identical() {
    assert_eq!(
        core::mem::size_of::<CgapFrameHeaderAbi>(),
        core::mem::size_of::<CgapFrameHeader>()
    );
    assert_eq!(
        core::mem::align_of::<CgapFrameHeaderAbi>(),
        core::mem::align_of::<CgapFrameHeader>()
    );
    assert_eq!(core::mem::size_of::<CgapFrameHeaderAbi>(), 128);
    assert_eq!(core::mem::align_of::<CgapFrameHeaderAbi>(), 64);
    assert_eq!(core::mem::size_of::<P28LicenseAttestationAbi>(), 80);
    assert_eq!(core::mem::align_of::<P28LicenseAttestationAbi>(), 8);
}
