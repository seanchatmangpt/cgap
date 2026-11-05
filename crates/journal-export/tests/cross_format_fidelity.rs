//! Cross-format fidelity court (lane 12): fixture lane -> emit_verified
//! IPC1 windows (cgap's OWN emitter) + rows carrier -> synthesized
//! journal frames -> sealed dir -> segment_source (manifest sha256
//! fail-closed) -> ipc_walk (tamper tier) -> parquet export (real
//! FileSink) -> verify_seal -> RLE/page decode -> VALUE-equal to the
//! fixture. Three mutation flips by exact variant. Darwin honesty pin:
//! async-I/O token pin (see the darwin court below).

use cgap_arrow::blake3;
use cgap_arrow::ipc::{self, MetaHeader};
use std::path::PathBuf;
use journal_export::ipc_walk;
use journal_export::parquet_sink;
use journal_export::segment_source::{self, crc32_ieee, sha256};

/// Fixture emitter: cgap emit_verified + the lane-12 rows carrier.
fn emit_window(out: &mut [u8], body: &[u8], rows: u64, schema: [u8; 32], seq: u32) -> usize {
    let meta = MetaHeader {
        magic: ipc::MAGIC,
        version: ipc::LAYOUT_VERSION,
        flags: 0,
        schema_digest: schema,
        body_len: body.len() as u32,
        alignment: 64,
        seq,
        reserved0: 0,
        header_digest: 0,
        pad_tail: [0; 4],
    };
    let sealed = ipc::seal_meta(meta);
    let n = ipc::emit_verified(out, &sealed, body).expect("emit_verified");
    out[104..112].copy_from_slice(&rows.to_le_bytes());
    n
}

/// Synthesize one sealed segment from window bodies (real files).
fn write_segment(dir: &std::path::Path, name: &str, first_seq: u64, schema: [u8; 32], bodies: &[Vec<u8>], rows_per_window: &[u64]) -> std::path::PathBuf {
    let sealed = dir.join("sealed");
    std::fs::create_dir_all(&sealed).unwrap();
    let path = sealed.join(format!("{name}.journal"));
    let mut bin: Vec<u8> = Vec::new();
    for (i, body) in bodies.iter().enumerate() {
        let mut out = vec![0u8; journal_export::window_len64(body.len())];
        emit_window(&mut out, body, rows_per_window[i], schema, (first_seq + i as u64) as u32);
        let key = b"seg";
        let mut frame_body: Vec<u8> = Vec::new();
        frame_body.extend_from_slice(&(first_seq + i as u64).to_be_bytes());
        frame_body.extend_from_slice(&1000i64.to_be_bytes());
        frame_body.extend_from_slice(&5u16.to_be_bytes());
        frame_body.extend_from_slice(&7u16.to_be_bytes());
        frame_body.extend_from_slice(&(key.len() as u32).to_be_bytes());
        frame_body.extend_from_slice(key);
        frame_body.extend_from_slice(&(out.len() as u32).to_be_bytes());
        frame_body.extend_from_slice(&out);
        let crc = crc32_ieee(&frame_body);
        bin.extend_from_slice(b"SMAL");
        bin.extend_from_slice(&((frame_body.len() + 4) as u32).to_be_bytes());
        bin.extend_from_slice(&frame_body);
        bin.extend_from_slice(&crc.to_be_bytes());
    }
    std::fs::write(&path, &bin).unwrap();
    let meta = format!("{{\"segment\": \"{name}\", \"sha256\": \"{}\"}}", hex(&sha256(&bin)));
    std::fs::write(format!("{}.meta.json", path.display()), meta).unwrap();
    std::fs::write(format!("{}.seal", path.display()), b"sealed\n").unwrap();
    path
}

#[test]
fn sha256_known_answer_vectors() {
    assert_eq!(
        hex(&sha256(b"")),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        hex(&sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let long = vec![0x61u8; 1_000_000];
    assert_eq!(
        hex(&sha256(&long)),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

#[test]
fn crc32_ieee_known_vectors() {
    assert_eq!(crc32_ieee(b""), 0x0000_0000);
    assert_eq!(crc32_ieee(b"123456789"), 0xCBF4_3926);
    assert_eq!(crc32_ieee(b"SMAL frames tile exactly"), crc32_ieee(b"SMAL frames tile exactly"));
}

/// The full fidelity chain. Windows produced by CGAP's OWN emit_verified.
#[test]
fn cross_format_fidelity_roundtrip() {
    let dir = std::env::temp_dir().join(format!(
        "journal_export_fidelity_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let schema = [0xAAu8; 32];

    // fixture: 2 windows x 16 rows (2 row groups — every span visible to
    // verify_seal), u32 lane values
    let fixture: Vec<u32> = (0..16u32)
        .map(|i| (i.wrapping_mul(7).wrapping_add(13)) & 0xFF)
        .collect();
    let make_body = |w: u32| {
        fixture
            .iter()
            .flat_map(|v| v.wrapping_add(w * 100).to_le_bytes())
            .collect::<Vec<u8>>()
    };
    let bodies1 = vec![make_body(0)];
    let bodies2 = vec![make_body(1)];
    let rows1 = vec![16u64];
    let rows2 = vec![16u64];
    let seg = write_segment(&dir, "seg-00000000000000000001", 1, schema, &bodies1, &rows1);
    let seg2 = write_segment(&dir, "seg-00000000000000000002", 10, schema, &bodies2, &rows2);
    let _ = seg2;

    // discover -> read (manifest sha256 fail-closed) -> walk verified
    let segs = segment_source::discover_sealed_segments(&dir).unwrap();
    assert_eq!(segs.len(), 2, "both sealed segments discovered");
    let mut all_windows = Vec::new();
    for s in &segs {
        let segment = segment_source::read_sealed_segment(s).unwrap();
        assert_eq!(segment.frames.len(), 1);
        for f in &segment.frames {
            let win = ipc_walk::walk_verified(&f.payload).unwrap();
            assert_eq!(win.rows, 16);
            assert_eq!(win.schema_digest, schema);
            all_windows.push(win);
        }
    }
    assert_eq!(all_windows.len(), 2);

    // export through the real FileSink, verify_seal, RLE-decode: VALUE-equal
    let out = dir.join("export.par2");
    let exported =
        parquet_sink::export_windows(&all_windows, &[ipc_walk::COLUMN_NAME], &out).unwrap();
    assert_eq!(exported.rows, 32);
    let lane = parquet_sink::read_back_lane(&out, &exported.digest, 32).unwrap();
    let mut expected: Vec<u32> = Vec::new();
    for w in 0..2u32 {
        expected.extend(fixture.iter().map(|v| v.wrapping_add(w * 100)));
    }
    assert_eq!(lane, expected, "parquet decode is VALUE-equal to the fixture");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Three corruption classes, each flipped by its EXACT typed variant:
/// manifest sha256, frame CRC, window EOS.
#[test]
fn three_corruption_classes_flip_by_exact_variant() {
    let dir = std::env::temp_dir().join(format!(
        "journal_export_flips_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let schema = [0xBBu8; 32];
    let body: Vec<u8> = (0..16u32).flat_map(|v| v.to_le_bytes()).collect();
    let seg = write_segment(&dir, "seg-00000000000000000007", 1, schema, &[body.clone()], &[16]);
    let bin = std::fs::read(&seg).unwrap();

    // 1. manifest sha256 flip (without re-signing)
    let mut tainted = bin.clone();
    tainted[10] ^= 0xEF;
    std::fs::write(&seg, &tainted).unwrap();
    match segment_source::read_sealed_segment(&seg) {
        Err(journal_export::ExportError::ManifestMismatch { .. }) => {}
        other => panic!("expected ManifestMismatch, got {other:?}"),
    }
    std::fs::write(&seg, &bin).unwrap();

    // 2. frame CRC flip (re-sign manifest so ONLY the CRC layer trips)
    let mut tainted = bin.clone();
    let n = tainted.len();
    tainted[n - 1] ^= 0xFF; // last byte of frame 1's CRC
    std::fs::write(&seg, &tainted).unwrap();
    let meta_path = format!("{}.meta.json", seg.display());
    let meta = std::fs::read_to_string(&meta_path).unwrap();
    let meta = meta.replace(&hex(&sha256(&bin)), &hex(&sha256(&tainted)));
    std::fs::write(&meta_path, meta).unwrap();
    match segment_source::read_sealed_segment(&seg) {
        Err(journal_export::ExportError::CrcMismatch { frame: 0 }) => {}
        other => panic!("expected CrcMismatch(frame 0), got {other:?}"),
    }
    std::fs::write(&seg, &bin).unwrap();
    let meta = std::fs::read_to_string(&meta_path).unwrap();
    let meta = meta.replace(&hex(&sha256(&tainted)), &hex(&sha256(&bin)));
    std::fs::write(&meta_path, meta).unwrap();

    // 3. window EOS corruption: framing layer only (manifest re-signed,
    // CRC recomputed not needed — EOS lives INSIDE the payload, so the
    // CRC stays lawful; the walker is the layer that refuses)
    let payload_at = 8 + 8 + 8 + 2 + 2 + 4 + 3 + 4;
    let win_len = journal_export::window_len64(body.len());
    let mut tainted = bin.clone();
    tainted[payload_at + win_len - 1] ^= 0xAA;
    // the payload sits INSIDE the CRC scope (SeqID..Payload) — rebuild the
    // frame with a recomputed CRC so ONLY the walker layer trips
    let frame_len_at = 4;
    let body_len = u32::from_be_bytes(tainted[4..8].try_into().unwrap()) as usize;
    let frame_body_at = 8;
    let crc_at = frame_len_at + 4 + body_len - 4;
    let new_crc = crc32_ieee(&tainted[8..crc_at]);
    tainted[crc_at..crc_at + 4].copy_from_slice(&new_crc.to_be_bytes());
    let _ = frame_body_at;
    std::fs::write(&seg, &tainted).unwrap();
    let meta = std::fs::read_to_string(&meta_path).unwrap();
    let meta = meta.replace(&hex(&sha256(&bin)), &hex(&sha256(&tainted)));
    std::fs::write(&meta_path, meta).unwrap();
    let segment = segment_source::read_sealed_segment(&seg).unwrap();
    match ipc_walk::walk_verified(&segment.frames[0].payload) {
        Err(journal_export::ExportError::Ipc(ipc::IpcError::MissingEos { .. })) => {}
        other => panic!("expected Ipc(MissingEos), got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Darwin honesty pin: source-grep court — zero async-I/O primitives
/// anywhere in this crate's source (the export path is a plain
/// std::fs host surface).
#[test]
fn darwin_honesty_pin_zero_async_io_tokens() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let src = std::fs::read_dir(format!("{manifest}/src")).unwrap();
    let mut hits = 0usize;
    for e in src {
        let path = e.unwrap().path();
        if path.extension().and_then(|x| x.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap().to_lowercase();
        for token in ["io_uring", "o_direct", "liburing", "aio"] {
            if text.contains(token) {
                println!("HIT: {} contains {}", path.display(), token);
                hits += 1;
            }
        }
    }
    assert_eq!(hits, 0, "async-IO tokens found in a std-only host crate");
}
