//! segment_source — sealed journal segment reader (Rust mirror of
//! smalt's Smalt.Storage.JournalReader: classification names identical,
//! fail-closed order identical — .seal -> .meta.json -> manifest sha256
//! -> frames: magic, frame_len, CRC32 over SeqID..Payload, strictly
//! advancing SeqID.

use crate::{ExportError, ipc_walk};

/// One decoded journal frame.
#[derive(Clone, Debug)]
pub struct SegmentFrame {
    pub seq_id: u64,
    pub ts: i64,
    pub event_type: u16,
    pub table_id: u16,
    pub key: Vec<u8>,
    /// IPC1 window (alignment-64, verified at the export tier).
    pub payload: Vec<u8>,
}

/// One sealed segment's contents.
#[derive(Clone, Debug)]
pub struct Segment {
    pub frames: Vec<SegmentFrame>,
    pub sha256: [u8; 32],
}

/// Discover sealed segments under `<dir>/sealed`: every `*.journal`
/// whose `.seal` AND `.meta.json` both exist, sorted by name.
pub fn discover_sealed_segments(
    dir: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>, ExportError> {
    let sealed = dir.join("sealed");
    let mut out = Vec::new();
    let entries = std::fs::read_dir(&sealed).map_err(|_| ExportError::Io)?;
    for entry in entries {
        let path = entry.map_err(|_| ExportError::Io)?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("journal") {
            continue;
        }
        let p = path.to_str().ok_or(ExportError::Io)?;
        if std::fs::metadata(format!("{p}.seal")).is_ok()
            && std::fs::metadata(format!("{p}.meta.json")).is_ok()
        {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// Read a sealed segment fail-closed (journal_reader.ex classification
/// order). IPC windows are NOT walked here — call [`crate::ipc_walk`]
/// per payload at the tamper tier you need.
pub fn read_sealed_segment(path: &std::path::Path) -> Result<Segment, ExportError> {
    let p = path.to_str().ok_or(ExportError::Io)?;
    if std::fs::metadata(format!("{p}.seal")).is_err() {
        return Err(ExportError::NotSealed);
    }
    if std::fs::metadata(format!("{p}.meta.json")).is_err() {
        return Err(ExportError::MissingMeta);
    }
    let bin = std::fs::read(p).map_err(|_| ExportError::Io)?;
    let manifest_sha = manifest_sha256(&format!("{p}.meta.json"))?;
    let actual_sha = sha256(&bin);
    if manifest_sha != actual_sha {
        return Err(ExportError::ManifestMismatch {
            got: manifest_sha,
            expected: actual_sha,
        });
    }
    let frames = parse_frames(&bin)?;
    Ok(Segment { frames, sha256: actual_sha })
}

/// Extract `"sha256": "<64 hex>"` from the manifest JSON (no serde —
/// the manifest shape is pinned by JournalSink's writer).
fn manifest_sha256(meta_path: &str) -> Result<[u8; 32], ExportError> {
    let body = std::fs::read(meta_path).map_err(|_| ExportError::Io)?;
    let needle = b"\"sha256\": \"";
    let at = body
        .windows(needle.len())
        .position(|w| w == needle)
        .ok_or(ExportError::Io)?;
    let start = at + needle.len();
    let hex = body.get(start..start + 64).ok_or(ExportError::Io)?;
    let mut out = [0u8; 32];
    for i in 0..32 {
        let hi = hex_digit(hex[2 * i])?;
        let lo = hex_digit(hex[2 * i + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

fn hex_digit(c: u8) -> Result<u8, ExportError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(ExportError::Io),
    }
}

fn parse_frames(bin: &[u8]) -> Result<Vec<SegmentFrame>, ExportError> {
    let mut frames = Vec::new();
    let mut offset = 0usize;
    let mut prev_seq: Option<u64> = None;
    while offset < bin.len() {
        let have = bin.len() - offset;
        if have < 8 {
            return Err(ExportError::TruncatedTail { offset, have });
        }
        if &bin[offset..offset + 4] != b"SMAL" {
            return Err(ExportError::BadMagic { offset });
        }
        let frame_len = u32::from_be_bytes(
            bin[offset + 4..offset + 8].try_into().map_err(|_| ExportError::Io)?,
        ) as usize;
        if frame_len < 4 {
            return Err(ExportError::BadFrameLen { offset, have });
        }
        let needed = 8 + frame_len;
        if needed > have {
            return Err(ExportError::TruncatedFrame { offset, have });
        }
        let body_size = frame_len - 4;
        let body = &bin[offset + 8..offset + 8 + body_size];
        let crc_at = offset + 8 + body_size;
        let crc = u32::from_be_bytes(
            bin[crc_at..crc_at + 4].try_into().map_err(|_| ExportError::Io)?,
        );
        if crc32_ieee(body) != crc {
            return Err(ExportError::CrcMismatch { frame: frames.len() });
        }
        let f = decode_frame(body, offset, frames.len())?;
        if let Some(prev) = prev_seq {
            if f.seq_id != prev + 1 {
                return Err(ExportError::SeqRegression {
                    expected: prev + 1,
                    got: f.seq_id,
                });
            }
        }
        prev_seq = Some(f.seq_id);
        offset += needed;
        frames.push(f);
    }
    Ok(frames)
}

/// Decode one CRC-verified frame body into a typed frame. The field
/// layout must tile the body exactly (journal_reader.ex law) — any
/// short/overhang is TruncatedFrame, never a panic.
fn decode_frame(body: &[u8], offset: usize, _idx: usize) -> Result<SegmentFrame, ExportError> {
    if body.len() < 24 + 4 {
        return Err(ExportError::TruncatedFrame { offset, have: body.len() });
    }
    let seq_id = u64::from_be_bytes(body[0..8].try_into().map_err(|_| ExportError::Io)?);
    let ts = i64::from_be_bytes(body[8..16].try_into().map_err(|_| ExportError::Io)?);
    let event_type = u16::from_be_bytes(body[16..18].try_into().map_err(|_| ExportError::Io)?);
    let table_id = u16::from_be_bytes(body[18..20].try_into().map_err(|_| ExportError::Io)?);
    let key_len = u32::from_be_bytes(body[20..24].try_into().map_err(|_| ExportError::Io)?)
        as usize;
    let payload_at = 24 + key_len + 4;
    if body.len() < payload_at {
        return Err(ExportError::TruncatedFrame { offset, have: body.len() });
    }
    let payload_len = u32::from_be_bytes(
        body[24 + key_len..24 + key_len + 4].try_into().map_err(|_| ExportError::Io)?,
    ) as usize;
    if body.len() != payload_at + payload_len {
        return Err(ExportError::TruncatedFrame { offset, have: body.len() });
    }
    Ok(SegmentFrame {
        seq_id,
        ts,
        event_type,
        table_id,
        key: body[24..24 + key_len].to_vec(),
        payload: body[payload_at..payload_at + payload_len].to_vec(),
    })
}

/// CRC32 (IEEE, reflected, poly 0xEDB88320) — the journal frame CRC,
/// byte-law-identical to `:erlang.crc32`.
#[must_use]
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

const SHA_K: [u32; 64] = [
    0x428a2f98,
    0x71374491,
    0xb5c0fbcf,
    0xe9b5dba5,
    0x3956c25b,
    0x59f111f1,
    0x923f82a4,
    0xab1c5ed5,
    0xd807aa98,
    0x12835b01,
    0x243185be,
    0x550c7dc3,
    0x72be5d74,
    0x80deb1fe,
    0x9bdc06a7,
    0xc19bf174,
    0xe49b69c1,
    0xefbe4786,
    0x0fc19dc6,
    0x240ca1cc,
    0x2de92c6f,
    0x4a7484aa,
    0x5cb0a9dc,
    0x76f988da,
    0x983e5152,
    0xa831c66d,
    0xb00327c8,
    0xbf597fc7,
    0xc6e00bf3,
    0xd5a79147,
    0x06ca6351,
    0x14292967,
    0x27b70a85,
    0x2e1b2138,
    0x4d2c6dfc,
    0x53380d13,
    0x650a7354,
    0x766a0abb,
    0x81c2c92e,
    0x92722c85,
    0xa2bfe8a1,
    0xa81a664b,
    0xc24b8b70,
    0xc76c51a3,
    0xd192e819,
    0xd6990624,
    0xf40e3585,
    0x106aa070,
    0x19a4c116,
    0x1e376c08,
    0x2748774c,
    0x34b0bcb5,
    0x391c0cb3,
    0x4ed8aa4a,
    0x5b9cca4f,
    0x682e6ff3,
    0x748f82ee,
    0x78a5636f,
    0x84c87814,
    0x8cc70208,
    0x90befffa,
    0xa4506ceb,
    0xbef9a3f7,
    0xc67178f2,
];

/// SHA-256 (FIPS 180-4, pure, std-only) — the journal manifest digest,
/// byte-law-identical to `:crypto.hash(:sha256, _)`. Court-pinned in
/// tests/cross_format_fidelity.rs against the standard vectors.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bitlen = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        sha_compress(&mut h, chunk);
    }
    let mut out = [0u8; 32];
    for (i, w) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&w.to_be_bytes());
    }
    out
}

fn sha_compress(h: &mut [u32; 8], chunk: &[u8]) {
    let mut w = [0u32; 64];
    for i in 0..16 {
        w[i] = u32::from_be_bytes(chunk[4 * i..4 * i + 4].try_into().unwrap());
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }
    let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
    let (mut e, mut f, mut g, mut hh) = (h[4], h[5], h[6], h[7]);
    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let t1 = hh.wrapping_add(s1).wrapping_add(ch)
            .wrapping_add(SHA_K[i]).wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let t2 = s0.wrapping_add(maj);
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
    h[5] = h[5].wrapping_add(f);
    h[6] = h[6].wrapping_add(g);
    h[7] = h[7].wrapping_add(hh);
}
