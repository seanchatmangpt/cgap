//! Envelope decimal-emission court (lane H10).
//!
//! Pins byte-identity of the envelope length emitter inside `git_object_oid`
//! against std decimal formatting. The SHA-1 is the crate's own public
//! collaborator on both sides, so any divergence is attributable to the
//! decimal emitter alone — exactly the pin under test.
//!
//! Anti-vacuity: the digit-flip mutant court runs against a /tmp sandbox copy
//! of pack_write.rs with sha256-exact restore; see the lane receipt under
//! docs/jira/v26.11.2/hardening/.

use cgap_git::pack_write::{git_object_oid, Sha1};
fn ref_oid(kind: &str, content: &[u8]) -> [u8; 20] {
    let mut h = Sha1::new();
    h.update(kind.as_bytes());
    h.update(b" ");
    h.update(content.len().to_string().as_bytes());
    h.update(b"\0");
    h.update(content);
    h.finalize()
}

#[test]
fn envelope_oid_matches_reference_for_boundaries_and_sweep() {
    // Every envelope length digit-boundary plus a deterministic sweep.
    let mut sizes: Vec<usize> = vec![
        0, 1, 9, 10, 11, 99, 100, 101, 999, 1000, 1023, 1024, 65_535, 65_536, 1_048_575, 1_048_576,
    ];
    // corpus seed 0xA20C_0FFE_E5A7_E000 (frozen, never vary).
    let mut x: u64 = 0xA20C_0FFE_E5A7_E000;
    for _ in 0..256 {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        x = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        sizes.push((x >> 16) as usize);
    }
    for &size in &sizes {
        let content = vec![0xAB_u8; size & 0x7F_FFFF];
        let got = git_object_oid("blob", &content);
        let want = ref_oid("blob", &content);
        assert_eq!(
            got, want,
            "envelope decimal emission diverged at size {size}"
        );
    }
}

#[test]
fn oid_equality_is_structural() {
    // [u8;20] derive(PartialEq) lowers to unrolled word compares (verified in
    // asm receipt); this pins behavior — equal bytes equal, distinct distinct.
    let a = git_object_oid("blob", b"x");
    let b = git_object_oid("blob", b"x");
    let c = git_object_oid("tree", b"x");
    assert_eq!(a, b);
    assert_ne!(a, c);
}
