//! Anti-vacuity mutations (F6, v26.11.14), applied to a /tmp sandbox copy:
//! (1) `crates/cgap-aoq/src/codec.rs` encode: `fnv1a32(&out[0..60])` ->
//!     `fnv1a32(&out[0..59])`; `golden_frame_matches_committed_fixture` must FAIL.
//! (2) flip one GOLDEN_HEX byte in this file's copy (first `31` -> `32`); must FAIL.
//! Golden-byte fixture court (R1-I, S2): the genesis AOQ v3 frame bytes are
//! DERIVED from the real `codec::encode` and asserted byte-identical against
//! the committed fixture — never guessed, never hand-written.
//!
//! Golden frame law (landed ontology `packs/aoq-pack/ontology.ttl`):
//! epoch = Genesis (0), key_id = 0x0102030405060708, frame_ordinal = 7,
//! seq = 42, reserved 36B zeros, FNV-1a32 over `[0,0x3C)` at `0x3C`;
//! body = exactly 1 byte, the StreamType prefix `0x00` (stream-data).
//!
//! The same hex is committed on the smalt side as
//! `smalt/test/data/aoq_v3_golden.hex` and consumed by
//! `smalt/test/courts/aoq_cross_repo_court.exs` (decode in both directions).

use cgap_aoq::codec;
use cgap_aoq::header::{AoqHeader, Epoch};

/// Committed golden frame: 64B header + 1B body (StreamType 0x00 = DATA).
const GOLDEN_HEX: &str = "31514F41030000000807060504030201070000002A0000000000000000000000000000000000000000000000000000000000000000000000000000004DDD780B00";

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("fixture hex must be valid"))
        .collect()
}

#[test]
fn golden_frame_matches_committed_fixture() {
    let expected = hex_decode(GOLDEN_HEX);
    assert_eq!(expected.len(), 65, "golden frame is 64B header + 1B body");

    // Derive: real codec, real struct, no literals for the payload fields.
    let header = AoqHeader::new(Epoch::Genesis as u16, 0x0102_0304_0506_0708, 7, 42);
    let mut buf = [0u8; cgap_aoq::header::SIZE_BYTES];
    codec::encode(&header, &mut buf).expect("genesis header encodes");

    let mut frame = buf.to_vec();
    frame.push(0x00); // StreamType prefix 0 (stream-data), 1-byte body

    assert_eq!(
        frame, expected,
        "codec::encode output drifted from the committed fixture"
    );

    // Round-trip: the committed fixture decodes to the same fields.
    let decoded = codec::decode(&expected[..64]).expect("committed fixture decodes");
    assert_eq!(decoded.epoch, Epoch::Genesis as u16);
    assert_eq!(decoded.key_id, 0x0102_0304_0506_0708);
    assert_eq!(decoded.frame_ordinal, 7);
    assert_eq!(decoded.seq, 42);
    assert_eq!(decoded.header_digest, 0x0B78_DD4D);
}
