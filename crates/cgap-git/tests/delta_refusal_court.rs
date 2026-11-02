//! GS4 §4 court: pack v2 parser must refuse delta object headers (type 6
//! OFS_DELTA / type 7 REF_DELTA) with a typed error, fail-closed, instead of
//! fail-open consuming the delta payload as a zlib body.
//!
//! Real collaborators only: synthetic packs are built from the crate's own
//! `encode_object_header` + `zlib_compress` + `sha1` in a real tempdir; the
//! parser runs over the on-disk bytes. Anti-vacuity: the pre-fix parser
//! parsed the same fixtures `Ok` (type-6/7 recorded as `ParsedObject`) —
//! witness recorded in
//! `docs/jira/v26.11.2/waves/w6-git-typed-refusals.md`.

use std::fs;
use std::path::PathBuf;

use cgap_git::deflate::zlib_compress;
use cgap_git::pack_write::{encode_object_header, parse_pack_v2, sha1};
use cgap_git::GitError;

/// Build a syntactically valid pack v2 whose single entry uses `obj_type`,
/// with a real zlib body of `size` uncompressed bytes and a correct SHA-1
/// trailer (so checksum verification passes and the header parse is reached).
fn synthetic_pack(obj_type: u8, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"PACK");
    out.extend_from_slice(&2u32.to_be_bytes());
    out.extend_from_slice(&1u32.to_be_bytes());
    out.extend_from_slice(&encode_object_header(obj_type, body.len() as u64));
    out.extend_from_slice(&zlib_compress(body));
    let sum = sha1(&out);
    out.extend_from_slice(&sum);
    out
}

/// Write bytes to a real tempdir file and read them back (final-state I/O).
fn write_temp(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join("cgap-git-delta-refusal-court");
    fs::create_dir_all(&dir).expect("tempdir");
    let path = dir.join(name);
    fs::write(&path, bytes).expect("write pack");
    path
}

fn read_temp(name: &str) -> Vec<u8> {
    fs::read(
        std::env::temp_dir()
            .join("cgap-git-delta-refusal-court")
            .join(name),
    )
    .expect("read pack")
}

#[test]
fn type6_ofs_delta_header_is_refused_typed() {
    let pack = synthetic_pack(6, b"\x01\x9b\x01delta-base-payload");
    let path = write_temp("ofs_delta.pack", &pack);
    let bytes = read_temp(path.to_str().expect("utf8"));
    let err = parse_pack_v2(&bytes).expect_err("type-6 must be refused, not parsed");
    assert_eq!(
        err,
        GitError::UnsupportedDeltaEncoding,
        "expected the typed delta refusal, got {err:?}"
    );
}

#[test]
fn type7_ref_delta_header_is_refused_typed() {
    let pack = synthetic_pack(7, b"\x01\x9b\x01ref-delta-payload");
    let path = write_temp("ref_delta.pack", &pack);
    let bytes = read_temp(path.to_str().expect("utf8"));
    let err = parse_pack_v2(&bytes).expect_err("type-7 must be refused, not parsed");
    assert_eq!(
        err,
        GitError::UnsupportedDeltaEncoding,
        "expected the typed delta refusal, got {err:?}"
    );
}

#[test]
fn full_object_types_still_parse_no_over_refusal() {
    // Types 1 (commit), 2 (tree), 3 (blob) remain accepted.
    for obj_type in 1u8..=3 {
        let pack = synthetic_pack(obj_type, b"legitimate full object payload");
        let name = format!("full_type_{obj_type}.pack");
        let path = write_temp(&name, &pack);
        let bytes = read_temp(path.to_str().expect("utf8"));
        let objs = parse_pack_v2(&bytes)
            .unwrap_or_else(|e| panic!("type {obj_type} must still parse, got {e:?}"));
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].obj_type, obj_type);
    }
}

#[test]
fn invalid_types_zero_and_five_still_bad_varint() {
    for obj_type in [0u8, 5u8] {
        // encode_object_header asserts (1..=4 | 6 | 7), so reserved types
        // 0/5 are hand-encoded here: bits 6..4 carry the type code.
        let mut pack = Vec::new();
        pack.extend_from_slice(b"PACK");
        pack.extend_from_slice(&2u32.to_be_bytes());
        pack.extend_from_slice(&1u32.to_be_bytes());
        pack.push((obj_type << 4) | 0x07);
        pack.extend_from_slice(&zlib_compress(b"payload"));
        let sum = sha1(&pack);
        pack.extend_from_slice(&sum);
        let err = parse_pack_v2(&pack).expect_err("reserved type must be refused");
        assert_eq!(err, GitError::BadVarint, "type {obj_type} keeps BadVarint");
    }
}
