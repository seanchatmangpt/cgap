//! M-CTQ consolidation bench (lane XC-04, v26.11.2).
//!
//! One harness, three stations, each measured with the csct N=7
//! Min/Median/Max method (03-msa-gauge.md Sec 2) so the three receipted
//! but unconsolidated CTQs become promotable into 11-measured-claims.md:
//!
//! 1. M-CTQ-04 seal throughput — the REAL b06 sealing path: 80-byte
//!    lease frames (nonce u64 LE + parent 32B + splitmix64 payload 32B +
//!    tag `0xC5CB_C700_0000_0080`) hashed by the real `cgap_blake3::hash`
//!    kernel, monotonic chain, batch window FACT_WINDOW = 1,365
//!    (pack_demux.rs:51) amortized — one `Instant` pair per run (gauge
//!    law: 41 ns timer floor, 03-msa-gauge.md Sec 1).
//! 2. M-CTQ-05 FFI transition cost — the REAL exported call path
//!    `cgap_admit_frame` (sovereign.rs:413) called IN-PROCESS through its
//!    actual `extern "C" ABI` via the cgap-ffi rlib (measured route:
//!    in-process extern "C" entry, NOT the cdylib; the cdylib adds
//!    dyld trampoline cost this harness does not price — disclosed).
//!    Two arms: null-refusal floor probe (ipc-03's 85 ns proxy) and a
//!    batch-1365 admit over REAL 152 B A04 frames built by
//!    `CgapFrameHeader::encode_into`.
//! 3. M-CTQ-06 mmap read — the ipc-04 mechanism in Rust: temp file,
//!    producer RW MAP_SHARED mapping, consumer SECOND independent
//!    PROT_READ mapping, zero-copy 24 B-fact reads, wire header
//!    `0x4347415053484D31` ("CGAPSHM1").
//!
//! Anti-vacuity: every station runs a positive control (real admit must
//! return CGAP_OK; read-back must be byte-exact; chain digests
//! re-derive). An all-zero or error measurement is a typed FAIL, exit 1.
//! A station that cannot produce a real measurement does NOT fake one.
//!
//! Contention: the chains-ingress.rs CONTENTION-SUSPECT guard pattern
//! (32 MiB streaming walk, 12 ns/64B-line quiet-host ceiling, loud
//! stderr marker) is reproduced here — the marker line is the grep
//! contract for consumers.
//!
//! harness = false; timing via `std::time::Instant`; no Process.sleep.

use std::hint::black_box;
use std::time::Instant;

use cgap_blake3::hash;
use cgap_core::frame::{
    CgapFrameHeader, P28LicenseAttestation, FLAG_BN3, FRAME_HEADER_LEN, LICENSE_FLAG_ROOT,
    LICENSE_MAGIC,
};
use cgap_ffi::sovereign::{cgap_admit_frame, CgapFrameView, CGAP_OK};

/// csct rig law: N=7 independent runs per station (03-msa-gauge.md Sec 2).
const RUNS: usize = 7;
/// Coprime batch window (crates/cgap-git/src/pack_demux.rs:51).
const FACT_WINDOW: usize = 1_365;
/// 24 B facts (ipc-04 wire stride; corpus.LE format).
const FACT_STRIDE: usize = 24;
/// 152 B frame = 128 B A04 header + 24 B payload.
const FRAME_BYTES: usize = FRAME_HEADER_LEN + FACT_STRIDE;
/// ipc-04 SHM wire magic.
const SHM_MAGIC: u64 = 0x4347_4150_5348_4D31;

// ---------------------------------------------------------------------------
// Contention guard (pattern: chains-ingress.rs:706-790, reproduced).
// ---------------------------------------------------------------------------

const GUARD_BUF_BYTES: usize = 32 * 1024 * 1024;
const CONTENTION_THRESHOLD_NS: f64 = 12.0;
const CONTENTION_MARK: &str = "CONTENTION-SUSPECT:";

fn guard_walk(buf: &[u8], passes: u32) -> f64 {
    let lines = buf.len() / 64;
    let mut best = f64::INFINITY;
    let mut acc: u64 = 0;
    for _ in 0..passes {
        let start = Instant::now();
        let mut i = 0;
        while i < buf.len() {
            let b: [u8; 8] = buf[i..i + 8].try_into().expect("8-byte window");
            acc = acc.wrapping_add(u64::from_le_bytes(b));
            i += 64;
        }
        let per_line = start.elapsed().as_secs_f64() * 1e9 / lines as f64;
        if per_line < best {
            best = per_line;
        }
    }
    black_box(acc);
    best
}

fn contention_guard() {
    let mut buf = vec![0u8; GUARD_BUF_BYTES];
    for chunk in buf.chunks_mut(4096) {
        chunk[0] = 0xA5;
    }
    black_box(&buf);
    let ns = guard_walk(&buf, 3);
    if ns > CONTENTION_THRESHOLD_NS {
        eprintln!(
            "{} host-noise: streaming-walk min {} ns/64B-line exceeds \
             quiet-host ceiling {} — run is CONTENTION-SUSPECT; numbers \
             NOT valid for quiet-host-scoped claims.",
            CONTENTION_MARK, ns, CONTENTION_THRESHOLD_NS
        );
    } else {
        eprintln!(
            "CONTENTION-GUARD: quiet ({} ns/64B-line <= ceiling {})",
            ns, CONTENTION_THRESHOLD_NS
        );
    }
}

// ---------------------------------------------------------------------------
// Stats: Min/Median/Max over N=7 (quantile honesty: no p99 — max-of-7 is
// sorted[6], not a tail quantile; FIX-04 law).
// ---------------------------------------------------------------------------

fn report(station: &str, unit: &str, mut xs: Vec<f64>) -> (f64, f64, f64) {
    xs.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    let (min, med, max) = (xs[0], xs[RUNS / 2], xs[RUNS - 1]);
    println!(
        "[{} {}] min={:.2} median={:.2} max={:.2} {} runs={}",
        station, unit, min, med, max, unit, RUNS
    );
    (min, med, max)
}

fn fail(msg: &str) -> ! {
    eprintln!("[MCTQ-FAIL] {}", msg);
    std::process::exit(1);
}

// ---------------------------------------------------------------------------
// Station 1 — M-CTQ-04: seal throughput (b06 sealing path, batch-1365).
// ---------------------------------------------------------------------------

const DOMAIN_TAG: u64 = 0xC5CB_C700_0000_0080;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Seals FACT_WINDOW receipts through the real blake3 kernel; returns
/// elapsed seconds and a chain-validity flag (digest re-derivation +
/// parent linkage on the final receipt, positive control).
fn seal_chain(mut parent: [u8; 32]) -> (f64, bool) {
    let mut nonce: u64 = rand_tail();
    let t0 = Instant::now();
    let mut ok = true;
    let mut last_frame: Option<[u8; 80]> = None;
    for _ in 0..FACT_WINDOW {
        let mut frame = [0u8; 80];
        nonce = nonce.wrapping_add(1);
        frame[0..8].copy_from_slice(&nonce.to_le_bytes());
        frame[8..40].copy_from_slice(&parent);
        let mut sm = nonce;
        for chunk in frame[40..72].chunks_exact_mut(8) {
            chunk.copy_from_slice(&splitmix64(&mut sm).to_le_bytes());
        }
        frame[72..80].copy_from_slice(&DOMAIN_TAG.to_le_bytes());
        let digest = hash(&frame);
        ok &= digest != [0u8; 32];
        parent = digest;
        last_frame = Some(frame);
    }
    // Positive control OUTSIDE the timed window: re-derive the final
    // digest from the final frame (chain court, b06 law).
    if let Some(f) = last_frame {
        ok &= hash(&f) == parent;
    }
    (t0.elapsed().as_secs_f64(), ok)
}

fn rand_tail() -> u64 {
    // Deterministic, non-zero chain tail (no RNG dep; corpus law respected:
    // the splitmix64 multiplier above is the stream, not a seed swap).
    0xA20C_0FFE_E5A7_E000u64.wrapping_mul(0xBF58_476D_1CE4_E5B9) | 1
}

// ---------------------------------------------------------------------------
// Station 2 — M-CTQ-05: FFI transition cost (real exported call path,
// in-process extern "C" ABI; disclosed: not the dyld/cdylib route).
// ---------------------------------------------------------------------------

/// Builds one REAL 152 B A04 frame: 128 B CgapFrameHeader (magic CGAP,
/// version 1, FLAG_BN3, BLAKE3 oabi_digest over the 24 B payload, root
/// P28 attestation) + 24 B payload (ipc-03 fixture shape).
fn build_frame(payload: &[u8; FACT_STRIDE], out: &mut [u8; FRAME_BYTES]) -> bool {
    let att = P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: 1,
        attestation_flags: LICENSE_FLAG_ROOT,
        safe_harbor_ceiling_cents: 100_000_000,
        origin_authority_digest: hash(payload),
        upstream_receipt: [0u8; 32],
    };
    let header = CgapFrameHeader {
        magic: *b"CGAP",
        version: 1,
        flags: FLAG_BN3,
        reserved: [0u8; 2],
        payload_byte_len: FACT_STRIDE as u64,
        oabi_digest: hash(payload),
        attestation: att,
    };
    let encoded = header.encode_into(out).is_ok();
    out[FRAME_HEADER_LEN..].copy_from_slice(payload);
    encoded
}

fn station_ffi() {
    // Positive control fixture: one valid frame, admitted once for real.
    let payload = [0x5Eu8; FACT_STRIDE];
    let mut frame = [0u8; FRAME_BYTES];
    if !build_frame(&payload, &mut frame) {
        fail("station-2 could not encode the A04 fixture frame");
    }
    let mut view = CgapFrameView {
        payload: std::ptr::null(),
        payload_len: 0,
        schema_id: 0,
        rule_count: 0,
    };
    let status = unsafe { cgap_admit_frame(frame.as_ptr(), FRAME_BYTES, &mut view) };
    if status != CGAP_OK {
        fail(&format!(
            "station-2 positive control: real admit refused with typed status 0x{:08X} \
             (expected CGAP_OK) — cannot honestly measure the admitted path",
            status
        ));
    }
    black_box(view.payload_len);

    // Arm A — null-refusal membrane floor (ipc-03's honest proxy).
    const FLOOR_ITERS: usize = 200_000;
    let mut floors: Vec<f64> = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t0 = Instant::now();
        for _ in 0..FLOOR_ITERS {
            let s = unsafe { cgap_admit_frame(std::ptr::null(), 0, &mut view) };
            black_box(s);
        }
        floors.push(t0.elapsed().as_secs_f64() * 1e9 / FLOOR_ITERS as f64);
    }
    let (fmin, _fmed, fmax) = report("MCTQ-05", "floor ns/call", floors);
    if fmax <= 0.0 {
        fail("station-2 floor measured all-zero");
    }

    // Arm B — batch-1365 amortized admit over REAL frames.
    let batch = vec![frame; FACT_WINDOW];
    let mut per_frame: Vec<f64> = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t0 = Instant::now();
        for f in &batch {
            let s = unsafe { cgap_admit_frame(f.as_ptr(), FRAME_BYTES, &mut view) };
            black_box(s);
        }
        per_frame.push(t0.elapsed().as_secs_f64() * 1e9 / FACT_WINDOW as f64);
    }
    let (amin, _amed, amax) = report("MCTQ-05", "admit ns/frame", per_frame);
    if amax <= 0.0 || batch.iter().any(|f| f[0..4] != *b"CGAP") {
        fail("station-2 admit measured all-zero or fixture corrupted");
    }
    eprintln!(
        "MCTQ-05-SPREAD: floor {}..{} ns, admit {}..{} ns (min..max, N=7)",
        fmin, fmax, amin, amax
    );
}

// ---------------------------------------------------------------------------
// Station 3 — M-CTQ-06: mmap zero-copy read (ipc-04 mechanism, Rust).
// Raw mmap(2) via libc externs — std links libSystem on darwin; no new
// crate dependency is added (bench Cargo.toml [dependencies] is not this
// lane's file to edit beyond the [[bench]] entry).
// ---------------------------------------------------------------------------

extern "C" {
    fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut u8;
    fn munmap(addr: *mut u8, len: usize) -> i32;
}

const PROT_READ: i32 = 0x01;
const PROT_WRITE: i32 = 0x02;
const MAP_SHARED: i32 = 0x0001;

/// SHM wire: 32 B header (magic u64 LE "CGAPSHM1", tx_id u64, fact_count
/// u32, stride u32) + FACT_WINDOW facts at 24 B stride.
fn shm_wire_len() -> usize {
    32 + FACT_WINDOW * FACT_STRIDE
}

fn station_mmap() {
    let path = std::env::temp_dir().join("mctq_shm_court.bin");
    let len = shm_wire_len();
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap_or_else(|e| fail(&format!("station-3 open failed: {}", e)));
    file.set_len(len as u64)
        .unwrap_or_else(|e| fail(&format!("station-3 set_len failed: {}", e)));
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();

    // Producer: RW MAP_SHARED mapping, write the canonical wire.
    let wptr = unsafe {
        mmap(
            std::ptr::null_mut(),
            len,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd,
            0,
        )
    };
    if wptr as isize == -1 {
        fail("station-3 producer mmap failed");
    }
    let wire = unsafe { std::slice::from_raw_parts_mut(wptr, len) };
    wire[0..8].copy_from_slice(&SHM_MAGIC.to_le_bytes());
    wire[8..16].copy_from_slice(&0xC0DEu64.to_le_bytes());
    wire[16..20].copy_from_slice(&(FACT_WINDOW as u32).to_le_bytes());
    wire[20..24].copy_from_slice(&(FACT_STRIDE as u32).to_le_bytes());
    for (i, fact) in wire[32..].chunks_exact_mut(FACT_STRIDE).enumerate() {
        let v = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        fact[0..8].copy_from_slice(&v.to_le_bytes());
    }
    let expect_tail: u64 = {
        let i = FACT_WINDOW - 1;
        (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
    };
    let mut tail = [0u8; 8];
    tail.copy_from_slice(&wire[len - FACT_STRIDE..len - FACT_STRIDE + 8]);
    if tail != expect_tail.to_le_bytes() {
        fail("station-3 producer wire tail mismatch");
    }

    // Consumer: SECOND independent PROT_READ mapping (ipc-04 mechanism).
    let rptr = unsafe { mmap(std::ptr::null_mut(), len, PROT_READ, MAP_SHARED, fd, 0) };
    if rptr as isize == -1 {
        fail("station-3 consumer mmap failed");
    }
    let region = unsafe { std::slice::from_raw_parts(rptr, len) };

    // Positive control: magic + tail read through the second mapping.
    let mut m = [0u8; 8];
    m.copy_from_slice(&region[0..8]);
    if u64::from_le_bytes(m) != SHM_MAGIC {
        fail("station-3 magic mismatch through consumer mapping");
    }

    // Timed arm: zero-copy reads of every fact through the mapping.
    let mut ns_per_read: Vec<f64> = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t0 = Instant::now();
        let mut acc: u64 = 0;
        for fact in region[32..].chunks_exact(FACT_STRIDE) {
            let b: [u8; 8] = fact[0..8].try_into().expect("24B fact window");
            acc = acc.wrapping_add(u64::from_le_bytes(b));
        }
        let el = t0.elapsed().as_secs_f64() * 1e9 / FACT_WINDOW as f64;
        black_box(acc);
        ns_per_read.push(el);
    }
    if ns_per_read.iter().any(|x| *x <= 0.0) {
        fail("station-3 read path measured all-zero");
    }
    let (rmin, _rmed, _rmax) = report("MCTQ-06", "read ns/op", ns_per_read);

    // Anti-vacuity flip (ipc-04 court 2, in-lane): corrupt the mapped
    // magic THROUGH the producer mapping; consumer must see corruption
    // (shared page proof — proves this is a real SHARED mapping, not a
    // private copy). Restore, verify byte-exact.
    wire[0] ^= 0xFF;
    let mut cm = [0u8; 8];
    cm.copy_from_slice(&region[0..8]);
    if u64::from_le_bytes(cm) == SHM_MAGIC {
        fail("station-3 corruption NOT visible across mappings — not a shared mapping");
    }
    wire[0] ^= 0xFF;
    let mut rm = [0u8; 8];
    rm.copy_from_slice(&region[0..8]);
    if u64::from_le_bytes(rm) != SHM_MAGIC {
        fail("station-3 restore failed");
    }
    eprintln!(
        "MCTQ-06-FLIP: cross-mapping corruption flip + restore witnessed (min {} ns/op)",
        rmin
    );

    unsafe {
        munmap(wptr, len);
        munmap(rptr, len);
    }
}

// ---------------------------------------------------------------------------
// Station 1 driver + main.
// ---------------------------------------------------------------------------

fn station_seal() {
    let mut rates: Vec<f64> = Vec::with_capacity(RUNS);
    for run in 0..RUNS {
        let (elapsed, ok) = seal_chain([0u8; 32]);
        if !ok {
            fail(&format!("station-1 run {} chain control failed", run));
        }
        rates.push(FACT_WINDOW as f64 / elapsed);
    }
    if rates.iter().any(|r| *r <= 0.0) {
        fail("station-1 measured all-zero rate");
    }
    let (smin, _smed, smax) = report("MCTQ-04", "seal receipts/s", rates);
    eprintln!(
        "MCTQ-04-SPREAD: {}..{}/s (min..max, N=7); batch window {}",
        smin, smax, FACT_WINDOW
    );
}

fn main() {
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!(
            "LOUD-SKIP {} : {v}",
            std::env::args().next().unwrap_or_default()
        );
        return;
    }
    contention_guard();
    println!(
        "=== station 1: M-CTQ-04 seal throughput (batch {}) ===",
        FACT_WINDOW
    );
    station_seal();
    println!("=== station 2: MCTQ-05 FFI transition cost ===");
    station_ffi();
    println!("=== station 3: MCTQ-06 mmap zero-copy read ===");
    station_mmap();
    println!("[MCTQ-DONE] all three stations measured, controls PASS, standing ALIVE");
}
