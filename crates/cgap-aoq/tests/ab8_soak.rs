//! AB-8 soak: 10k-frame seal -> writev -> UDS -> verify -> unseal.
//!
//! Deliverables (PG-A2 bench/stress table):
//! - throughput frames/sec over a REAL unix-domain socket, writev scatter
//!   of the writer's segments (body by reference)
//! - zero-alloc-per-frame court: a counting global allocator observes the
//!   hot loop; any allocation during the 10k-frame round trips fails
//!   (anti-vacuity: the counter is proven non-trivially-zero by the setup
//!   phase, which allocates and moves the count)

use std::io::{IoSlice, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicUsize, Ordering};

use cgap_aoq::verify::{decrypt_in_place, verify_window};
use cgap_aoq::writer::{AoqWriter, TAG_BYTES};
use cgap_arrow::blake3;

/// Counting allocator (court instrumentation, not production).
static ALLOCS: AtomicUsize = AtomicUsize::new(0);

struct Counting;

unsafe impl std::alloc::GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { std::alloc::System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        unsafe { std::alloc::System.dealloc(p, l) }
    }
}

#[global_allocator]
static A: Counting = Counting;

/// Keystream block (host-derived; R-K1).
fn keystream_block(seq: u32, block_no: u32) -> [u8; 32] {
    let mut inp = [0u8; 40];
    inp[0..7].copy_from_slice(b"ab8-ks ");
    inp[7..11].copy_from_slice(&seq.to_le_bytes());
    inp[11..15].copy_from_slice(&1_760_000_002u32.to_le_bytes());
    inp[15..19].copy_from_slice(&block_no.to_le_bytes());
    blake3::hash(&inp)
}

const BODY: usize = 64;
const FRAMES: usize = 10_000;
/// Body is 64 bytes, so the window law's body pad is exactly 64.
const WIN_LEN: usize = 8 + 120 + BODY + 64 + 8;

/// One socketpair frame: scatter-writev the writer segments + inline pad.
fn writev_frame(
    tx: &mut UnixStream,
    w: &mut AoqWriter,
    plain: &[u8; BODY],
    zero_pad: &[u8; 64],
) -> std::io::Result<usize> {
    let seq = w.cursor().1;
    let mut ct = *plain;
    let ks0 = keystream_block(seq, 0);
    let ks1 = keystream_block(seq, 1);
    for i in 0..BODY {
        ct[i] ^= if i < 32 { ks0[i] } else { ks1[i - 32] };
    }
    let h = blake3::hash(&ct);
    let mut tag = [0u8; TAG_BYTES];
    tag.copy_from_slice(&h[..TAG_BYTES]);
    let segs = w
        .frame_segments(0, 7, &tag, &ct)
        .expect("soak: lawful frame");
    let bufs = [
        IoSlice::new(segs.prefix),
        IoSlice::new(segs.header),
        IoSlice::new(segs.pad),
        IoSlice::new(segs.body),
        IoSlice::new(zero_pad),
        IoSlice::new(segs.tail),
    ];
    let n = tx.write_vectored(&bufs)?;
    w.commit();
    Ok(n)
}

/// Reader side: read one window, verify, decrypt, return plaintext.
fn read_unseal_frame(rx: &mut UnixStream, ordinal: u32, seq: u32) -> std::io::Result<[u8; BODY]> {
    let mut win = [0u8; WIN_LEN];
    rx.read_exact(&mut win)?;
    let mut ct = [0u8; BODY];
    // Framing pass via ipc (parses body_len from the padding).
    let (n, _) = cgap_arrow::ipc::read_with_meta(&win, None, &mut ct)
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    assert_eq!(n, BODY);
    let tag = {
        let mut t = [0u8; 16];
        t.copy_from_slice(&win[76..92]);
        t
    };
    let vw = verify_window(&win, ordinal, seq, &tag)
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    let ks0 = keystream_block(seq, 0);
    let ks1 = keystream_block(seq, 1);
    let mut ks = [0u8; 64];
    ks[..32].copy_from_slice(&ks0);
    ks[32..].copy_from_slice(&ks1);
    decrypt_in_place(&vw, &ks, &mut win).map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    let mut plain = [0u8; BODY];
    plain.copy_from_slice(&win[vw.body_at..vw.body_at + BODY]);
    Ok(plain)
}

#[test]
fn soak_10k_seal_writev_uds_verify_unseal() {
    let (mut tx, mut rx) = UnixStream::pair().expect("socketpair");
    let plain = [0x33u8; BODY];
    let zero_pad = [0u8; 64];

    // Warm-up frame OUTSIDE the counted region (proves the counter can
    // move: setup allocates).
    let before_setup = ALLOCS.load(Ordering::Relaxed);
    {
        // Explicit allocation proves the counter moves (non-vacuity):
        // a counter that cannot move cannot witness zero.
        let probe: Vec<u8> = vec![7u8; 991];
        assert_eq!(probe[990], 7);
        let mut w = AoqWriter::new();
        let _ = writev_frame(&mut tx, &mut w, &plain, &zero_pad).expect("warm-up");
        let _ = read_unseal_frame(&mut rx, 0, 0).expect("warm-up read");
    }
    let after_setup = ALLOCS.load(Ordering::Relaxed);
    assert!(
        after_setup > before_setup,
        "VACUOUS COUNTER: setup allocated nothing"
    );

    // The counted region: 10k frames, zero allocations permitted.
    let mut w = AoqWriter::new();
    let baseline = ALLOCS.load(Ordering::Relaxed);
    let t0 = std::time::Instant::now();
    let mut bytes = 0usize;
    for i in 0..FRAMES {
        let cursor = i as u32;
        bytes += writev_frame(&mut tx, &mut w, &plain, &zero_pad).expect("soak write");
        let back = read_unseal_frame(&mut rx, cursor, cursor).expect("soak read");
        assert_eq!(back, plain, "frame {i}: plaintext round-trip");
    }
    let dt = t0.elapsed();
    let allocs = ALLOCS.load(Ordering::Relaxed) - baseline;
    let fps = (FRAMES as f64) / dt.as_secs_f64();
    println!("soak: {FRAMES} frames, {bytes} bytes, {dt:?}, {fps:.0} frames/sec, {allocs} allocs");
    assert_eq!(
        allocs, 0,
        "zero-alloc-per-frame court: {allocs} allocations"
    );
}
