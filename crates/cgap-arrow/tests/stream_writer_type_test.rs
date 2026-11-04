//! StreamWriter type courts (lane e80-06, v26.11.3) — the cl-l witness's
//! writev handoff raised into `cgap_arrow::stream_writer::StreamWriter`.
//! Real collaborators (live UDS, real kernel writev, sealed slab): no
//! mocks. Anti-vacuity: each refusal court (zero-progress, scratch
//! truncation, seq override) pins a typed outcome that flips against a
//! looping/ignoring implementation, and T4's partial-accept court must be
//! byte-identical to a raw `ipc::emit` reference window.

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use cgap_arrow::blake3;
use cgap_arrow::ipc::{
    emit as raw_emit, read, seal_meta, window_len, IpcError, MetaHeader,
};
use cgap_arrow::slab::{ArrowRecordSlab, SlabError};
use cgap_arrow::stream_writer::{StreamWriter, StreamError, VectoredSink};

/// Real sealed 4-row slab (same shape as the witness fixture).
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

fn meta_for(body_len: usize, alignment: u32, seq: u32, digest: [u8; 32]) -> MetaHeader {
    seal_meta(MetaHeader {
        magic: cgap_arrow::ipc::MAGIC,
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

fn schema_digest_of(body: &[u8]) -> [u8; 32] {
    blake3::hash(body)
}

/// Test-only std adapter: kernel writev over a live `UnixStream`.
struct UdsSink<'a> {
    stream: &'a mut UnixStream,
}

impl<'a> VectoredSink for UdsSink<'a> {
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

/// In-memory sink: appends every accepted byte (Chicago: real buffer).
#[derive(Default)]
struct MemSink(Vec<u8>);

impl VectoredSink for MemSink {
    type Error = core::convert::Infallible;
    fn write_vectored(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error> {
        let mut n = 0usize;
        for b in bufs {
            self.0.extend_from_slice(b);
            n += b.len();
        }
        Ok(n)
    }
}

/// Sink that accepts exactly one segment per call (partial accept).
#[derive(Default)]
struct FirstSegSink {
    accepted: Vec<u8>,
    calls: usize,
}

impl VectoredSink for FirstSegSink {
    type Error = core::convert::Infallible;
    fn write_vectored(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error> {
        self.calls += 1;
        let b = bufs[0];
        self.accepted.extend_from_slice(b);
        Ok(b.len())
    }
}

/// Sink that always accepts zero bytes (the spin hazard).
struct StalledSink;

impl VectoredSink for StalledSink {
    type Error = core::convert::Infallible;
    fn write_vectored(&mut self, _bufs: &[&[u8]]) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Unique UDS path under the host tempdir; removed at scope end.
fn socket_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before epoch")
        .subsec_nanos();
    let mut p: PathBuf = std::env::temp_dir();
    p.push(format!(
        "cgap-stream-writer-{}-{}-{}.sock",
        std::process::id(),
        nanos,
        tag
    ));
    p
}

/// Emit + writev the sealed 4-row slab across a live UDS THROUGH the
/// StreamWriter type. Returns (socket-visible bytes, body_len).
fn uds_handoff(seq: u32, alignment: u32, tag: &str) -> (Vec<u8>, usize) {
    let slab = sealed_4row().expect("slab ok");
    let body = slab.as_bytes().to_vec();
    let meta = meta_for(body.len(), alignment, seq, schema_digest_of(&body));
    let mut scratch = vec![0u8; window_len(body.len(), alignment as usize)];
    let path = socket_path(tag);
    let listener = UnixListener::bind(&path).expect("bind live UDS");
    let mut stream = UnixStream::connect(&path).expect("connect live UDS");
    let (mut peer, _addr) = listener.accept().expect("accept");
    let mut writer = StreamWriter::with_seq(UdsSink { stream: &mut stream }, seq);
    let n = writer
        .emit(&mut scratch, meta, &body)
        .expect("type emit ok");
    let mut got = vec![0u8; n];
    peer.read_exact(&mut got).expect("read_exact live UDS");
    drop(writer);
    drop(stream);
    drop(peer);
    let _ = std::fs::remove_file(&path);
    (got, body.len())
}

/// T1 (type W1): emit -> writev across a live UDS through the TYPE, then
/// byte-exact IPC1 framing pins on the RECEIVED bytes (64-alignment).
#[test]
fn t1_writev_handoff_through_type_uds() {
    let (got, body_len) = uds_handoff(5, 64, "t1");
    assert_eq!(&got[0..4], &cgap_arrow::ipc::CONTINUATION.to_le_bytes());
    let meta_span = u32::from_le_bytes([got[4], got[5], got[6], got[7]]);
    assert_eq!(meta_span, 120u32, "64-alignment meta span");
    assert_eq!(&got[8..12], &cgap_arrow::ipc::MAGIC.to_le_bytes());
    assert_eq!(u16::from_le_bytes([got[12], got[13]]), 1, "layout version");
    let body_at = 8 + 120usize;
    assert_eq!(body_at % 64, 0, "body starts 64-aligned");
    let got_body_len = u32::from_le_bytes([
        got[0x28 + 8],
        got[0x29 + 8],
        got[0x2A + 8],
        got[0x2B + 8],
    ]);
    assert_eq!(got_body_len as usize, body_len);
    let got_align = u32::from_le_bytes([
        got[0x2C + 8],
        got[0x2D + 8],
        got[0x2E + 8],
        got[0x2F + 8],
    ]);
    assert_eq!(got_align, 64);
    let slab = sealed_4row().expect("slab ok");
    assert_eq!(
        &got[body_at..body_at + body_len],
        &slab.as_bytes()[..],
        "body bytes intact over the socket through the type"
    );
    assert!(
        got[body_at + body_len..got.len() - 8]
            .iter()
            .all(|&b| b == 0),
        "body padding zero-filled"
    );
    assert_eq!(
        &got[got.len() - 8..got.len() - 4],
        &cgap_arrow::ipc::CONTINUATION.to_le_bytes()
    );
    assert_eq!(&got[got.len() - 4..], &cgap_arrow::ipc::EOS.to_le_bytes());
}

/// T1b: the socket-received window decodes through the real `read` back to
/// the exact slab body — the type feeds a lawful consumer.
#[test]
fn t1b_received_window_decodes_via_read() {
    let slab = sealed_4row().expect("slab ok");
    let (got, body_len) = uds_handoff(3, 64, "t1b");
    let digest = schema_digest_of(&slab.as_bytes().to_vec());
    let mut out = vec![0u8; body_len];
    let n = read(&got, Some(&digest), &mut out).expect("read ok");
    assert_eq!(n, body_len);
    assert_eq!(&out[..n], slab.as_bytes(), "socket bytes decode to the exact slab body");
}

/// T1c: 8-alignment scatter through the type across a live UDS (the
/// witness covered 64 only).
#[test]
fn t1c_alignment8_handoff_through_type() {
    let (got, _body_len) = uds_handoff(7, 8, "t1c");
    let meta_span = u32::from_le_bytes([got[4], got[5], got[6], got[7]]);
    assert_eq!(meta_span, 64u32, "8-alignment meta span");
    let body_at = 8 + 64usize;
    assert_eq!(body_at % 8, 0, "body starts 8-aligned");
}

/// T2: the writer stamps ITS sequence over the caller's `meta.seq` and
/// advances monotonically across emits — the seq law raised with the type.
#[test]
fn t2_seq_stamped_monotonic() {
    let slab = sealed_4row().expect("slab ok");
    let body = slab.as_bytes().to_vec();
    let digest = schema_digest_of(&body);
    let mut scratch = vec![0u8; window_len(body.len(), 64)];
    let mut writer = StreamWriter::with_seq(MemSink::default(), 41);
    // Caller-supplied seq (999) is overridden by the writer's (41).
    let meta = meta_for(body.len(), 64, 999, digest);
    writer.emit(&mut scratch, meta, &body).expect("emit 1 ok");
    let w1 = scratch.to_vec();
    let seq1 = u32::from_le_bytes([w1[0x30 + 8], w1[0x31 + 8], w1[0x32 + 8], w1[0x33 + 8]]);
    assert_eq!(seq1, 41, "writer seq stamped over caller's 999");
    assert_eq!(writer.seq(), 42, "seq advanced after full write");
    writer.emit(&mut scratch, meta, &body).expect("emit 2 ok");
    let seq2 = u32::from_le_bytes([
        scratch[0x30 + 8],
        scratch[0x31 + 8],
        scratch[0x32 + 8],
        scratch[0x33 + 8],
    ]);
    assert_eq!(seq2, 42, "second emit carries the advanced seq");
}

/// T3 (anti-vacuity, refusal): a sink accepting zero bytes is a typed
/// `ZeroProgress`, not an infinite loop — flips against a looping impl.
#[test]
fn t3_zero_progress_typed_refusal() {
    let slab = sealed_4row().expect("slab ok");
    let body = slab.as_bytes().to_vec();
    let meta = meta_for(body.len(), 64, 0, schema_digest_of(&body));
    let mut scratch = vec![0u8; window_len(body.len(), 64)];
    let mut writer = StreamWriter::new(StalledSink);
    let err = writer
        .emit(&mut scratch, meta, &body)
        .expect_err("zero-progress sink must refuse");
    assert_eq!(err, StreamError::ZeroProgress);
    // Seq did NOT advance on the refused emit.
    assert_eq!(writer.seq(), 0, "refused emit does not advance the seq");
}

/// T4 (anti-vacuity, fidelity): a sink accepting only the first segment
/// per call still yields a byte-identical window — the advance loop is
/// pinned against a raw `ipc::emit` reference.
#[test]
fn t4_partial_accept_segments_byte_exact() {
    let slab = sealed_4row().expect("slab ok");
    let body = slab.as_bytes().to_vec();
    let meta = meta_for(body.len(), 64, 11, schema_digest_of(&body));
    let mut scratch = vec![0u8; window_len(body.len(), 64)];
    let mut writer = StreamWriter::with_seq(FirstSegSink::default(), 11);
    let n = writer.emit(&mut scratch, meta, &body).expect("emit ok");
    // Reference window straight from ipc::emit (no writer).
    let mut reference = vec![0u8; n];
    let rn = raw_emit(&mut reference, &meta, &body).expect("raw emit ok");
    assert_eq!(rn, n);
    assert_eq!(
        &writer.into_inner().accepted[..n],
        &reference[..rn],
        "partial-accept scatter reassembles byte-identical to ipc::emit"
    );
}

/// T5 (refusal): scratch smaller than `window_len` is the ipc framing
/// `Truncated` class, surfaced through the writer's `Framing` variant.
#[test]
fn t5_short_scratch_framing_truncated() {
    let slab = sealed_4row().expect("slab ok");
    let body = slab.as_bytes().to_vec();
    let meta = meta_for(body.len(), 64, 0, schema_digest_of(&body));
    let need = window_len(body.len(), 64);
    let mut scratch = vec![0u8; need - 1];
    let mut writer = StreamWriter::new(MemSink::default());
    let err = writer.emit(&mut scratch, meta, &body).expect_err("must refuse");
    assert!(matches!(
        err,
        StreamError::Framing(IpcError::Truncated { needed, .. }) if needed == need
    ));
    assert_eq!(writer.seq(), 0, "refused emit does not advance the seq");
}
