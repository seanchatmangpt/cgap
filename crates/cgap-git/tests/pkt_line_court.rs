//! Lane GIT-02 mutation court: pkt-line codec.
//!
//! Real bytes only, no mocks. Every court names its anti-vacuity flip
//! twin — a hand-mutated reimplementation that MUST disagree with the
//! real codec; a court that cannot flip is FAIL.

#[path = "../src/pkt_line.rs"]
mod pkt_line;
use pkt_line::{decode_header, decode_pkt, encode_err, encode_pkt, Header, Pkt, PktError};

// ---- flip twin 1: decoder that accepts non-hex length digits ----

/// Mutated hex parser: folds any byte to its low nibble and never
/// validates. On `000g` it yields 0 instead of [`PktError::BadHexLength`].
fn mutated_hex_len(h: &[u8; 4]) -> u16 {
    let nib = |c: u8| (c as u32 & 0xF) as u16;
    (nib(h[0]) << 12) | (nib(h[1]) << 8) | (nib(h[2]) << 4) | nib(h[3])
}

// ---- flip twin 2: encoder that stamps a fixed wrong header ----

fn mutated_encode(payload: &[u8]) -> [u8; 64] {
    let mut out = [0u8; 64];
    out[0..4].copy_from_slice(b"0010"); // wrong: constant header
    out[4..4 + payload.len()].copy_from_slice(payload);
    out
}

// ---- court 1: invalid hex length `000g` => typed Err ----

#[test]
fn court_bad_hex_length() {
    let err = decode_header(b"000g").expect_err("000g is not valid hex");
    assert_eq!(err, PktError::BadHexLength);

    // Flip twin: the mutated parser accepts `000g` (folds 'g' to 7 and
    // yields a value); the real codec refuses it. Ok-vs-Err disagreement
    // proves the court discriminates.
    assert_eq!(
        mutated_hex_len(b"000g"),
        7,
        "flip-twin self-check: mutated parser yields a value where real codec refuses"
    );
    assert!(decode_header(b"000g").is_err());
}

#[test]
fn court_bad_hex_length_digits() {
    // Every non-hex ASCII byte must be rejected, including at every
    // position of the header.
    for pos in 0..4usize {
        let mut h = *b"0010";
        h[pos] = b'g';
        assert_eq!(
            decode_header(&h),
            Err(PktError::BadHexLength),
            "non-hex digit at position {pos} must be BadHexLength"
        );
    }
}

// ---- court 2: truncated payload (declared length > available bytes) ----

#[test]
fn court_truncated_payload() {
    // Header says 0014 (20 total) but only 10 bytes exist.
    let err = decode_pkt(b"0014abc").expect_err("declared 20, only 7 available");
    assert_eq!(
        err,
        PktError::Truncated {
            declared: 20,
            available: 7
        },
        "truncation must be typed with declared vs available"
    );

    // 3-byte input: not even the header fits.
    let err = decode_pkt(b"001").expect_err("header itself truncated");
    assert_eq!(
        err,
        PktError::Truncated {
            declared: 4,
            available: 3
        }
    );
}

// ---- court 3: oversize length (above the 65516-byte protocol cap) ----

#[test]
fn court_oversize_length() {
    // ffff = 65535 total > 65520 cap => BadLength.
    let err = decode_header(b"ffff").expect_err("65535 exceeds the protocol cap");
    assert_eq!(err, PktError::BadLength(65_535));
    // Exactly at cap + 1 is refused; cap itself would be Data.
    let err = decode_header(b"fff5").expect_err("65525 exceeds the cap");
    assert_eq!(err, PktError::BadLength(65_525));
    // The cap itself (65520 total) is a legal Data header.
    assert_eq!(decode_header(b"fff0"), Ok(Header::Data(65_520)));
}

// ---- court 4: control markers decode as kinds, not lengths ----

#[test]
fn court_control_markers() {
    assert_eq!(decode_header(b"0000"), Ok(Header::Flush));
    assert_eq!(decode_header(b"0001"), Ok(Header::Delim));
    assert_eq!(decode_header(b"0002"), Ok(Header::ResponseEnd));
    assert_eq!(decode_pkt(b"0000"), Ok((Pkt::Flush, 4)));
    assert_eq!(decode_pkt(b"0002"), Ok((Pkt::ResponseEnd, 4)));
    // 0003: legal hex, but below the 4-byte minimum for a data line.
    assert_eq!(decode_header(b"0003"), Err(PktError::BadLength(3)));
}

// ---- court 5: round-trip real payload, byte-exact vs hand-computed ----

#[test]
fn court_roundtrip_byte_exact() {
    let payload = b"refs/heads/main\x00report-status";
    let mut out = [0u8; 64];
    let n = encode_pkt(payload, &mut out).expect("29-byte payload fits the 64-byte buffer");
    // Hand-computed: 15 + 1 + 13 = 29 payload, total 33 = 0x21.
    assert_eq!(payload.len(), 29);
    let expect: [u8; 33] = *b"0021refs/heads/main\x00report-status";
    assert_eq!(n, 33);
    assert_eq!(
        &out[..33],
        &expect[..],
        "wire bytes must be byte-exact vs hand computation"
    );

    // Round-trip decode: same bytes, same payload, full consumption.
    let (pkt, consumed) = decode_pkt(&out[..n]).expect("round-trip must decode");
    assert_eq!(pkt, Pkt::Data(payload));
    assert_eq!(consumed, 33);

    // Flip twin: constant-header encoder must produce different bytes on
    // the same payload; the court discriminates.
    assert_ne!(
        &mutated_encode(payload)[..33],
        &expect[..],
        "flip-twin encoder disagrees with the real encoder"
    );
}

// ---- court 6: flush/delim/response-end encoders emit exact markers ----

#[test]
fn court_encode_control() {
    let mut out = [0u8; 64];
    assert_eq!(encode_pkt(b"", &mut out).expect("empty payload"), 4);
    assert_eq!(&out[..4], b"0004", "empty data pkt is 0004");
    pkt_line::encode_flush(&mut out).expect("flush pkt encodes");
    assert_eq!(&out[..4], b"0000");
    pkt_line::encode_delim(&mut out).expect("delim pkt encodes");
    assert_eq!(&out[..4], b"0001");
    pkt_line::encode_response_end(&mut out).expect("response-end pkt encodes");
    assert_eq!(&out[..4], b"0002");
}

// ---- court 7: ERR packets encode and round-trip with typed message ----

#[test]
fn court_err_packets() {
    let mut out = [0u8; 64];
    let n = encode_err(b"access denied", &mut out).expect("short ERR msg fits");
    // Hand-computed: 4 header + "ERR " + 13 = 21 = 0x15.
    let expect: &[u8] = b"0015ERR access denied";
    assert_eq!(n, 21);
    assert_eq!(
        &out[..n],
        expect,
        "ERR wire bytes byte-exact vs hand computation"
    );

    let (pkt, consumed) = decode_pkt(&out[..n]).expect("ERR pkt round-trips");
    assert_eq!(consumed, 21);
    assert_eq!(pkt.err_message(), Some(&b"access denied"[..]));
    assert!(!pkt.is_flush());

    // Non-ERR payload yields no message.
    let mut out2 = [0u8; 64];
    let n2 = encode_pkt(b"nacl", &mut out2).unwrap();
    let (pkt3, _) = decode_pkt(&out2[..n2]).unwrap();
    assert_eq!(pkt3.err_message(), None);
}

// ---- court 8: payload-too-large refusal on the 64-byte stack buffer ----

#[test]
fn court_payload_too_large() {
    let mut out = [0u8; 64];
    let big = [0xAAu8; 61];
    let err = encode_pkt(&big, &mut out).expect_err("61-byte payload exceeds the 60-byte cap");
    assert_eq!(
        err,
        PktError::PayloadTooLarge { len: 61, cap: 60 },
        "oversize encode must be typed, never panic"
    );
    // Boundary: 60 bytes is exactly the cap and must succeed.
    let ok = encode_pkt(&[0xAAu8; 60], &mut out).expect("60-byte payload is at the cap");
    assert_eq!(ok, 64);
    assert_eq!(&out[..4], b"0040", "60-byte payload total 64 = 0x40");
}

// ---- court 9: multi-pkt stream decode (framing over a real stream) ----

#[test]
fn court_stream_framing() {
    // Hand-assembled wire: flush + data("want x") + delim.
    // "want x" is 6 bytes => data line total 10 => header "000a".
    let mut wire: Vec<u8> = Vec::new();
    wire.extend_from_slice(b"0000");
    wire.extend_from_slice(b"000awant x");
    wire.extend_from_slice(b"0001");
    let (pkt1, c1) = decode_pkt(&wire).expect("flush decodes");
    assert_eq!(pkt1, Pkt::Flush);
    let (pkt2, c2) = decode_pkt(&wire[c1..]).expect("data decodes");
    assert_eq!(pkt2, Pkt::Data(b"want x"));
    assert_eq!(c2, 10, "data line consumes header + payload exactly");
    let (pkt3, c3) = decode_pkt(&wire[c1 + c2..]).expect("delim decodes");
    assert_eq!(pkt3, Pkt::Delim);
    assert_eq!(c3, 4);
    assert_eq!(c1 + c2 + c3, wire.len(), "stream fully consumed");
}
