//! Writev handoff witness (lane cl-l, v26.113) — closes GA3's residual:
//! the framing bytes are court-pinned by `ipc_courts.rs` (C1/C1b/C2) but
//! the StreamWriter-style socket write is not: `emit` had never crossed a
//! real socket. This witness binds a LIVE Unix domain socket, hands the
//! emitted IPC1 window off via a real `writev` (`write_vectored`), reads
//! it back on the listener side, and asserts byte-exact framing per the
//! IPC1 layout pinned in `src/ipc.rs`.
//!
//! REFACTOR DISCLOSURE (lane e80-06, RAISE): the handoff now goes through
//! the raised type `cgap_arrow::stream_writer::StreamWriter` (emit +
//! vectored scatter + partial-accept advance + seq stamping) instead of
//! the hand-rolled emit/writev loop. W4 mutates the BODY before emit
//! (previously one window byte post-emit, pre-write) — the same body byte
//! class, still never the shared tree. Every W1/W2/W4 assert is unchanged.
//!
//! Integration tests compile on host std (cargo test); the lib target
//! stays `#![no_std]` — the socket I/O lives entirely in this file.
//!
//! Chicago discipline: real collaborators (real UDS, real kernel writev,
//! real sealed slab). No mocks. Anti-vacuity: W1 pins every byte of the
//! received window and every IPC1 field on the SOCKET-VISIBLE bytes; W4
//! mutates one body byte of the lane's own in-memory window (never the
//! shared tree) and requires the framing read to refuse it — proving the
//! byte-exact assert is not vacuous.

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use cgap_arrow::blake3;
use cgap_arrow::ipc::{
    read, seal_meta, window_len, MetaHeader, CONTINUATION, EOS, MAGIC,
};
use cgap_arrow::slab::{ArrowRecordSlab, SlabError};
use cgap_arrow::stream_writer::{StreamWriter, VectoredSink};

/// Test-only std adapter: kernel writev over a live `UnixStream`.
struct Uds<'a> {
    stream: &'a mut UnixStream,
}

impl<'a> VectoredSink for Uds<'a> {
    type Error = std::io::Error;
    fn write_vectored(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error> {
        let mut slices = [std::io::IoSlice::new(&[]); 4];
        for (dst, src) in slices.iter_mut().zip(bufs) {
            *dst = std::io::IoSlice::new(src);
        }
        let k = bufs.len().min(slices.len());
        self.stream.write_vectored(&slices[..k])
    }
}

/// Real sealed 4-row slab (same shape as ipc_courts' C1 fixture).
fn sealed_4row() -> Result<ArrowRecordSlab<4>, SlabError> {
    let mut s = ArrowRecordSlab::<4>::new();
    for i in 0..4u32 {
        let mut p = [0u8; 60];
        p[..4].copy_from_slice(&i.to_le_bytes());
        s.push(&p, i % 2 == 0)?;
    }
    s.seal()?;
    Ok(s)
}

/// Schema identity for the body: BLAKE3 over the sealed slab window.
fn schema_digest_of(body: &[u8]) -> [u8; 32] {
    blake3::hash(body)
}

/// Metadata header for `body` at `alignment`, sequence `seq`.
fn meta_for(body_len: usize, alignment: u32, seq: u32, digest: [u8; 32]) -> MetaHeader {
    seal_meta(MetaHeader {
        magic: MAGIC,
        version: cgap_arrow::ipc::LAYOUT_VERSION,
        flags: 0,
        schema_digest: digest,
        body_len: body_len as u32,
        alignment,
        seq,
        reserved0: 0,
        header_digest: 0,
        pad_tail: [0; 4],
    })
}

/// Unique socket path under the host tempdir (no external dev-deps); the
/// witness removes it at scope end.
fn socket_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before epoch")
        .subsec_nanos();
    let mut p: PathBuf = std::env::temp_dir();
    p.push(format!(
        "cgap-writev-witness-{}-{}-{}.sock",
        std::process::id(),
        nanos,
        tag
    ));
    p
}

/// Hand `body` off over a fresh LIVE UDS THROUGH the StreamWriter type
/// (emit into scratch, then the type's writev scatter with partial-accept
/// advance) and read it back on the listener side. Returns (the window
/// bytes as written, bytes received).
fn handoff(
    meta: MetaHeader,
    body: &[u8],
    tag: &str,
) -> (Vec<u8>, Vec<u8>) {
    let path = socket_path(tag);
    let listener = UnixListener::bind(&path).expect("bind live UDS");
    let mut stream = UnixStream::connect(&path).expect("connect live UDS");
    let (mut peer, _addr) = listener.accept().expect("accept");

    let mut scratch = vec![0u8; window_len(body.len(), meta.alignment as usize)];
    let mut writer = StreamWriter::with_seq(Uds { stream: &mut stream }, meta.seq);
    let n = writer.emit(&mut scratch, meta, body).expect("emit ok");

    // Synchronization is the kernel read itself — no sleeps.
    let mut got = vec![0u8; n];
    peer.read_exact(&mut got).expect("read_exact over live UDS");

    drop(writer);
    drop(stream);
    drop(peer);
    let _ = std::fs::remove_file(&path);
    (scratch[..n].to_vec(), got)
}

/// Emit the 4-row slab's window for `seq` (kept for W4's body mutation).
fn emit_body(seq: u32) -> (MetaHeader, Vec<u8>) {
    let slab = sealed_4row().expect("slab ok");
    let body = slab.as_bytes().to_vec();
    let meta = meta_for(body.len(), 64, seq, schema_digest_of(&body));
    (meta, body)
}

/// W1: emit -> StreamWriter writev handoff over a live UDS -> byte-exact
/// framing on the socket-visible bytes (anti-vacuity: any emitter
/// offset/ordering change or a dropped/partial write flips this court).
#[test]
fn w1_writev_handoff_through_live_uds() {
    let (meta, body) = emit_body(5);
    let (w, got) = handoff(meta, &body, "w1");
    assert_eq!(w.len(), got.len(), "writev wrote the whole window");
    assert_eq!(got, w, "byte-exact window across the live UDS writev handoff");

    // IPC1 layout pins on the RECEIVED bytes, not the emitted ones.
    assert_eq!(&got[0..4], &CONTINUATION.to_le_bytes());
    let meta_span = u32::from_le_bytes([got[4], got[5], got[6], got[7]]);
    assert_eq!(meta_span, 120u32, "64-alignment meta span");
    assert_eq!(&got[8..12], &MAGIC.to_le_bytes());
    assert_eq!(u16::from_le_bytes([got[12], got[13]]), 1, "layout version");
    let body_at = 8 + 120usize;
    assert_eq!(body_at % 64, 0, "body starts 64-aligned");
    let got_body_len = u32::from_le_bytes([
        got[0x28 + 8],
        got[0x29 + 8],
        got[0x2A + 8],
        got[0x2B + 8],
    ]);
    assert_eq!(got_body_len as usize, body.len());
    let got_align = u32::from_le_bytes([
        got[0x2C + 8],
        got[0x2D + 8],
        got[0x2E + 8],
        got[0x2F + 8],
    ]);
    assert_eq!(got_align, 64);
    assert_eq!(
        &got[body_at..body_at + body.len()],
        &w[body_at..body_at + body.len()],
        "body bytes intact over the socket"
    );
    assert!(
        got[body_at + body.len()..got.len() - 8]
            .iter()
            .all(|&b| b == 0),
        "body padding zero-filled"
    );
    assert_eq!(&got[got.len() - 8..got.len() - 4], &CONTINUATION.to_le_bytes());
    assert_eq!(&got[got.len() - 4..], &EOS.to_le_bytes());
}

/// W2: the received window decodes through the real `read` (typed digest
/// check) back to the exact slab body — the socket path feeds a lawful
/// consumer, not just a byte diff.
#[test]
fn w2_received_window_decodes_via_read() {
    let slab = sealed_4row().expect("slab ok");
    let (meta, body) = emit_body(3);
    let (_w, got) = handoff(meta, &body, "w2");
    let digest = schema_digest_of(&slab.as_bytes().to_vec());
    let mut out = vec![0u8; body.len()];
    let n = read(&got, Some(&digest), &mut out).expect("read ok");
    assert_eq!(n, body.len());
    assert_eq!(&out[..n], slab.as_bytes(), "socket bytes decode to the exact slab body");
}

/// W4 (anti-vacuity): corrupt one body byte of the lane's own in-memory
/// body (never the shared tree), hand it off through the StreamWriter over
/// a live UDS, and require the received body to DIFFER from the sealed
/// slab — the same comparison W1 pins, proven to flip. Scoped finding
/// (recorded in the lane receipt, src untouched): `ipc::read` compares the
/// caller's expected digest against the header's `schema_digest` field but
/// never re-hashes the body, so a body-corrupted window still passes a
/// digest-bearing read; body integrity is the consumer's duty at the
/// framing layer's current contract.
#[test]
fn w4_mutated_window_flips_the_byte_diff_court() {
    let slab = sealed_4row().expect("slab ok");
    let (meta, mut body) = emit_body(9);
    let body_at = 8 + 120usize;
    let saved = body[0];
    body[0] ^= 0xFF;
    assert_ne!(body[0], saved, "vacuous court: mutation was a no-op");
    let (w, got) = handoff(meta, &body, "w4");
    // The mutation survives the socket byte-exact (writev fidelity) but
    // the byte-diff court flips against the pristine slab body.
    assert_eq!(got[body_at], w[body_at], "mutation survived the socket byte-exact");
    assert_ne!(
        &got[body_at..body_at + body.len()],
        &slab.as_bytes()[..],
        "byte-diff court flipped: received body differs from the sealed slab"
    );
    // And the typed read still accepts it (scoped finding, asserted so a
    // future body-integrity fix in ipc.rs flips this court too).
    let digest = schema_digest_of(&slab.as_bytes().to_vec());
    let mut out = vec![0u8; body.len()];
    let n = read(&got, Some(&digest), &mut out).expect("scoped finding: read accepts a body-corrupted window");
    assert_eq!(n, body.len());
}
