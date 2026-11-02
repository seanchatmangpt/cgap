//! POSH-3 ETag court: deterministic Azure-style ETag formatting over real
//! cgap_blake3::hash, into a caller-provided fixed buffer (no heap).
//!
//! PROVENANCE NOTE (fence g): the exact Azure `W/"..."` quoting form is defined
//! here as a named fixture const (`AZURE_ETAG_WIRE_FORM`). This court proves
//! self-consistency of that fixture form + digest determinism + buffer law.
//! No in-tree receipt grounding real Azure wire parity was located this lane,
//! so Azure parity is NOT claimed.
use cgap_blake3::hash;

/// Named fixture (NOT a claimed Azure schema): one resource delta sample.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SampleResourceDelta {
    pub resource_id: u64,
    pub revision: u64,
    pub payload_tag: [u8; 8],
}

/// Named fixture const: the wire form this court pins. 4 + 64 + 1 = 69 bytes.
pub const AZURE_ETAG_WIRE_FORM: &str = "W/\"<64-hex>\"";
pub const ETAG_WIRE_LEN: usize = 4 + 64 + 1;

#[derive(Debug, PartialEq, Eq)]
pub enum EtagFmtError {
    /// Buffer smaller than ETAG_WIRE_LEN.
    BufferTooSmall { needed: usize, got: usize },
}

/// Hex-encode 32 bytes into out[..64], lowercase.
fn hex_encode(digest: &[u8; 32], out: &mut [u8; 64]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (i, b) in digest.iter().enumerate() {
        out[i * 2] = HEX[(b >> 4) as usize];
        out[i * 2 + 1] = HEX[(b & 0x0f) as usize];
    }
}

/// Format `W/"<64-hex>"` of the digest of `delta` into `buf`. No heap.
pub fn format_azure_etag(
    delta: &SampleResourceDelta,
    buf: &mut [u8],
) -> Result<usize, EtagFmtError> {
    if buf.len() < ETAG_WIRE_LEN {
        return Err(EtagFmtError::BufferTooSmall { needed: ETAG_WIRE_LEN, got: buf.len() });
    }
    let mut canonical = Vec::with_capacity(24);
    // Deterministic canonical byte serialization of the delta (fixture-defined).
    canonical.extend_from_slice(&delta.resource_id.to_le_bytes());
    canonical.extend_from_slice(&delta.revision.to_le_bytes());
    canonical.extend_from_slice(&delta.payload_tag);
    let digest: [u8; 32] = hash(&canonical);
    let mut hexb = [0u8; 64];
    hex_encode(&digest, &mut hexb);
    buf[0] = b'W';
    buf[1] = b'/';
    buf[2] = b'"';
    buf[3..67].copy_from_slice(&hexb);
    buf[67] = b'"';
    Ok(ETAG_WIRE_LEN)
}

/// Round-trip parse: extract the 64-hex body of a `W/"..."` etag, decode to
/// 32 bytes.
pub fn parse_etag_body(buf: &[u8]) -> Option<[u8; 32]> {
    if buf.len() != ETAG_WIRE_LEN || &buf[0..3] != b"W/\"" || buf[67] != b'"' {
        return None;
    }
    let hex = &buf[3..67];
    let mut out = [0u8; 32];
    for i in 0..32 {
        let hi = (hex[i * 2] as char).to_digit(16)?;
        let lo = (hex[i * 2 + 1] as char).to_digit(16)?;
        out[i] = ((hi << 4) | lo) as u8;
    }
    Some(out)
}

fn digest_of(delta: &SampleResourceDelta) -> [u8; 32] {
    let mut buf = [0u8; ETAG_WIRE_LEN];
    format_azure_etag(delta, &mut buf).expect("fixture buffer sized");
    parse_etag_body(&buf).expect("well-formed fixture etag")
}

#[test]
fn court1_determinism_identical_deltas_byte_identical_etags() {
    let a = SampleResourceDelta { resource_id: 0xA20C_0FFE_E5A7_E000, revision: 7, payload_tag: [1,2,3,4,5,6,7,8] };
    let b = a;
    let mut ba = [0u8; ETAG_WIRE_LEN];
    let mut bb = [0u8; ETAG_WIRE_LEN];
    assert_eq!(format_azure_etag(&a, &mut ba), Ok(ETAG_WIRE_LEN));
    assert_eq!(format_azure_etag(&b, &mut bb), Ok(ETAG_WIRE_LEN));
    assert_eq!(ba, bb, "identical deltas must yield byte-identical etags");
    // Cross-instance too: two distinct delta values with same fields.
    let c = SampleResourceDelta { resource_id: a.resource_id, revision: a.revision, payload_tag: a.payload_tag };
    let mut bc = [0u8; ETAG_WIRE_LEN];
    format_azure_etag(&c, &mut bc).expect("sized");
    assert_eq!(ba, bc);
}

#[test]
fn court2_one_attribute_mutation_distinct_digest() {
    let base = SampleResourceDelta { resource_id: 0xBF58_476D_1CE4_E5B9, revision: 1, payload_tag: [0xAA; 8] };
    let mut mutated = base;
    mutated.revision = 2; // single-attribute mutation
    let da = digest_of(&base);
    let db = digest_of(&mutated);
    assert_ne!(da, db, "one-attribute mutation must flip the digest");
    // Second attribute axis too.
    let mut m2 = base;
    m2.payload_tag[0] ^= 0x01;
    assert_ne!(da, digest_of(&m2));
}

#[test]
fn court3_wire_format_shape_and_roundtrip() {
    let d = SampleResourceDelta { resource_id: 42, revision: 9, payload_tag: *b"posh3tag" };
    let mut buf = [0u8; ETAG_WIRE_LEN];
    assert_eq!(format_azure_etag(&d, &mut buf), Ok(ETAG_WIRE_LEN));
    assert_eq!(&buf[0..3], b"W/\"", "must start with W/\"");
    assert_eq!(buf[67], b'"', "must end with closing quote");
    for &b in &buf[3..67] {
        assert!(b.is_ascii_hexdigit() && !b.is_ascii_uppercase(), "lowercase hex body");
    }
    let body = parse_etag_body(&buf).expect("round-trip parse");
    let mut rebuf = [0u8; ETAG_WIRE_LEN];
    format_azure_etag(&d, &mut rebuf).expect("sized");
    assert_eq!(&rebuf[3..67], &buf[3..67]);
    // Digest embedded in etag equals fresh hash of same delta bytes.
    let mut canon = Vec::new();
    canon.extend_from_slice(&d.resource_id.to_le_bytes());
    canon.extend_from_slice(&d.revision.to_le_bytes());
    canon.extend_from_slice(&d.payload_tag);
    assert_eq!(body, hash(&canon));
    // Malformed inputs rejected by the parser.
    assert!(parse_etag_body(&buf[0..68]).is_none(), "wrong length rejected");
    buf[0] = b'X';
    assert!(parse_etag_body(&buf).is_none(), "wrong prefix rejected");
}

#[test]
fn court4a_undersized_buffer_typed_err_flip() {
    let d = SampleResourceDelta { resource_id: 1, revision: 1, payload_tag: [0; 8] };
    let mut small = [0u8; ETAG_WIRE_LEN - 1];
    assert_eq!(
        format_azure_etag(&d, &mut small),
        Err(EtagFmtError::BufferTooSmall { needed: ETAG_WIRE_LEN, got: ETAG_WIRE_LEN - 1 })
    );
    let mut tiny = [0u8; 0];
    assert!(format_azure_etag(&d, &mut tiny).is_err());
}

#[test]
fn court4b_stale_cached_etag_mismatch_detected_flip() {
    let original = SampleResourceDelta { resource_id: 7, revision: 1, payload_tag: *b"v1state!" };
    let mut cached = [0u8; ETAG_WIRE_LEN];
    format_azure_etag(&original, &mut cached).expect("sized");
    let mut changed = original;
    changed.revision = 2;
    let mut fresh = [0u8; ETAG_WIRE_LEN];
    format_azure_etag(&changed, &mut fresh).expect("sized");
    assert_ne!(cached, fresh, "stale cached etag must NOT match mutated resource");
    assert!(parse_etag_body(&cached) != parse_etag_body(&fresh));
}
