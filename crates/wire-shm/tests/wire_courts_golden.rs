//! WIR0 golden cross-language court: wire-shm's stamp_frame/read_frame_into
//! against the shared self-pinned fixture
//! `tests/fixtures/wir0_golden.txt` (identical copy of arcus
//! `internal/ringbuf/testdata/wir0_golden.txt`).
//!
//! SELF-PIN LAW (first assertion): sha256 over every case line (non-'#',
//! non-blank) concatenated verbatim with trailing newline must equal the
//! hardcoded pin below — any un-mirrored fixture edit reds this court before
//! any case runs. sha256 is hand-rolled (wire-shm zero-dependency law: zero
//! crates, even in tests); its correctness is fenced by the known vectors
//! `test_sha256_vectors` and by the pin itself matching the Go court.
//!
//! Case grammar is documented in the fixture header. Positives stamp and
//! require a byte-exact pinned header + payload read-back; negatives write a
//! raw header/payload into a slot and require the STAGE-named law (wire-shm
//! validation order MAGIC -> ORDINAL -> LEN -> BUDGET -> CRC) to be the
//! FIRST failure with the exact `WireError` variant fields; the `seg` case
//! opens a full segment image from bytes.

use std::path::PathBuf;
use wire_shm::{WireError, WireRing};

// Hardcoded fixture self-pin (must match the fixture's `# pin:` line and the
// Go court's goldenPin in internal/ringbuf/framed_test.go).
const GOLDEN_PIN: &str = "f56989dcfbcc7ba8135073b162cb24d8c05961aacca4e3e1606d249770eb0da3";

// --- Minimal pure-std sha256 (FIPS 180-4) ----------------------------------

struct Sha256 {
    state: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total: u64,
}

impl Sha256 {
    fn new() -> Self {
        Sha256 {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buf: [0u8; 64],
            buf_len: 0,
            total: 0,
        }
    }
    fn write(&mut self, mut data: &[u8]) {
        self.total = self.total.wrapping_add(data.len() as u64);
        if self.buf_len > 0 {
            let take = 64 - self.buf_len;
            let take = take.min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buf_len = 0;
            }
        }
        while data.len() >= 64 {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[..64]);
            self.compress(&block);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }
    fn finish(mut self) -> [u8; 32] {
        // Message length must be captured BEFORE padding (write() updates
        // self.total for padding bytes too).
        let msg_len = self.total;
        let bits = msg_len.wrapping_mul(8);
        self.write(&[0x80]);
        while self.buf_len != 56 {
            self.write(&[0]);
        }
        self.write(&bits.to_be_bytes());
        let mut out = [0u8; 32];
        for (i, w) in self.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_be_bytes());
        }
        out
    }
    fn compress(&mut self, block: &[u8; 64]) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut h = self.state;
        for i in 0..64 {
            let s1 = h[4].rotate_right(6) ^ h[4].rotate_right(11) ^ h[4].rotate_right(25);
            let ch = (h[4] & h[5]) ^ ((!h[4]) & h[6]);
            let t1 = h[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = h[0].rotate_right(2) ^ h[0].rotate_right(13) ^ h[0].rotate_right(22);
            let maj = (h[0] & h[1]) ^ (h[0] & h[2]) ^ (h[1] & h[2]);
            let t2 = s0.wrapping_add(maj);
            h[7] = h[6];
            h[6] = h[5];
            h[5] = h[4];
            h[4] = h[3].wrapping_add(t1);
            h[3] = h[2];
            h[2] = h[1];
            h[1] = h[0];
            h[0] = t1.wrapping_add(t2);
        }
        for i in 0..8 {
            self.state[i] = self.state[i].wrapping_add(h[i]);
        }
    }
}

fn sha256_hex(data: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    for d in data {
        h.write(d);
    }
    h.finish().iter().map(|b| format!("{b:02x}")).collect()
}

/// Known vectors fence the hand-rolled sha256 (a wrong implementation would
/// fail the pin assertion from the other side — this makes the failure mode
/// nameable instead of mysterious).
#[test]
fn test_sha256_vectors() {
    assert_eq!(
        sha256_hex(&[b""]),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(&[b"abc"]),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // Multi-part write must equal single-shot (the pin path writes line+LF).
    assert_eq!(sha256_hex(&[b"1234", b"5678"]), sha256_hex(&[b"12345678"]));
}

// --- Golden court -----------------------------------------------------------

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wir0_golden.txt")
}
use std::path::Path;

fn parse_cases() -> Vec<String> {
    let raw = std::fs::read_to_string(fixture_path()).expect("fixture present");
    raw.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

fn hex_bytes(s: &str) -> Vec<u8> {
    if s == "-" {
        return Vec::new();
    }
    assert!(s.len() % 2 == 0, "odd hex length");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex byte"))
        .collect()
}

fn tmp_segment(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("wir0-golden-{}-{}.seg", tag, std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

#[test]
fn golden_fixture_court() {
    let cases = parse_cases();
    assert!(!cases.is_empty(), "fixture must carry cases");
    // Pin input: each case line WITH its trailing LF (matches the fixture's
    // pin law and the Go court's hashing).
    let mut pin_input: Vec<u8> = Vec::new();
    for c in &cases {
        pin_input.extend_from_slice(c.as_bytes());
        pin_input.push(b'\n');
    }
    let refs: Vec<&[u8]> = vec![&pin_input];
    // SELF-PIN LAW: first assertion, before any case runs.
    let pin = sha256_hex(&refs);
    assert_eq!(
        pin, GOLDEN_PIN,
        "fixture self-pin mismatch — the fixture was edited without re-deriving its mirrors"
    );
    for c in &cases {
        let f: Vec<&str> = c.split_whitespace().collect();
        match f[0] {
            "pos" => golden_pos(
                f[1],
                f[2].parse::<u32>().unwrap(),
                f[3].parse::<u32>().unwrap(),
                &hex_bytes(f[4]),
                &hex_bytes(f[5]),
            ),
            "neg" => golden_neg(
                f[1],
                f[2].parse::<u32>().unwrap(),
                f[3].parse::<u32>().unwrap(),
                &hex_bytes(f[4]),
                &hex_bytes(f[5]),
                f[6],
            ),
            "seg" => golden_seg(f[1], f[2].parse::<u32>().unwrap(), &hex_bytes(f[3])),
            other => panic!("unknown case kind {other}"),
        }
    }
}

fn slot_offset(ordinal: u32, cap: u32) -> u64 {
    wire_shm::BOOK_LEN as u64 + (ordinal % cap) as u64 * wire_shm::SLOT_LEN as u64
}

/// Read raw bytes at a file offset (std only).
fn read_raw(path: &PathBuf, off: u64, len: usize) -> Vec<u8> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path).expect("open segment");
    f.seek(SeekFrom::Start(off)).expect("seek");
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf).expect("read raw");
    buf
}

/// Overwrite bytes at a file offset (std only).
fn write_raw(path: &PathBuf, off: u64, bytes: &[u8]) {
    use std::io::{Seek, SeekFrom, Write};
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open segment for write");
    f.seek(SeekFrom::Start(off)).expect("seek");
    f.write_all(bytes).expect("write raw");
}

fn golden_pos(name: &str, cap: u32, ordinal: u32, payload: &[u8], header: &[u8]) {
    let path = tmp_segment(name);
    {
        let mut ring = WireRing::create(&path, cap).expect("create");
        ring.stamp_frame(payload, ordinal).expect("stamp");
    }
    // Stamped header must equal the pinned header byte-exact.
    let got = read_raw(&path, slot_offset(ordinal, cap), wire_shm::HEADER_LEN);
    assert_eq!(got, header, "pos {name}: header not byte-exact");
    // Read must return the payload.
    let ring = WireRing::open(&path).expect("open");
    let mut out = vec![0u8; wire_shm::SLOT_LEN];
    let n = ring.read_frame_into(ordinal, &mut out).expect("read");
    assert_eq!(&out[..n], payload, "pos {name}: payload mismatch");
    std::fs::remove_file(&path).ok();
}

fn golden_neg(name: &str, cap: u32, ordinal: u32, header: &[u8], payload: &[u8], stage: &str) {
    let path = tmp_segment(name);
    {
        // Create a zeroed segment of the right shape; the frame bytes are
        // then written RAW (header + payload) — not via stamp_frame, whose
        // ordinal bookkeeping the tamper cases deliberately bypass.
        let mut ring = WireRing::create(&path, cap).expect("create");
        let _ = &mut ring;
    }
    let off = slot_offset(ordinal, cap);
    write_raw(&path, off, header);
    write_raw(&path, off + wire_shm::HEADER_LEN as u64, payload);
    let ring = WireRing::open(&path).expect("open");
    let plen = u32::from_be_bytes([header[8], header[9], header[10], header[11]]) as usize;
    let mut out = vec![0u8; wire_shm::SLOT_LEN];
    if stage == "BUDGET" {
        // Fixture BUDGET law: budget = HEADER_LEN + plen - 1.
        out = vec![0u8; wire_shm::HEADER_LEN + plen - 1];
    }
    let budget_have = out.len();
    let err = ring
        .read_frame_into(ordinal, &mut out)
        .expect_err("neg case must refuse");
    match (stage, err) {
        ("MAGIC", WireError::BadMagic { found }) => {
            let want = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
            assert_eq!(found, want, "neg {name}: BadMagic field");
        }
        ("ORDINAL", WireError::StaleOrdinal { expected: e, found }) => {
            assert_eq!(
                e, ordinal,
                "neg {name}: StaleOrdinal.expected = read ordinal"
            );
            let stored = u32::from_be_bytes([header[12], header[13], header[14], header[15]]);
            assert_eq!(
                found, stored,
                "neg {name}: StaleOrdinal.found = stored ordinal"
            );
        }
        ("LEN", WireError::FrameTooLarge { len, max }) => {
            assert_eq!(len, plen, "neg {name}: FrameTooLarge.len");
            assert_eq!(max, wire_shm::MAX_PAYLOAD, "neg {name}: FrameTooLarge.max");
        }
        ("BUDGET", WireError::BudgetTooSmall { need, have }) => {
            assert_eq!(
                need,
                wire_shm::HEADER_LEN + plen,
                "neg {name}: BudgetTooSmall.need"
            );
            assert_eq!(have, budget_have, "neg {name}: BudgetTooSmall.have");
        }
        ("CRC", WireError::CrcMismatch { expected: e, found }) => {
            let stored = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
            assert_eq!(e, stored, "neg {name}: CrcMismatch.expected = stored crc");
            assert_eq!(
                found,
                wire_shm::crc32_ieee(payload),
                "neg {name}: CrcMismatch.found = crc over corrupted payload"
            );
        }
        (s, e) => panic!("neg {name}: stage {s}: wrong refusal {e:?}"),
    }
    std::fs::remove_file(&path).ok();
}

fn golden_seg(name: &str, cap: u32, seg: &[u8]) {
    assert_eq!(
        seg.len(),
        wire_shm::BOOK_LEN + cap as usize * wire_shm::SLOT_LEN,
        "seg {name}: image size"
    );
    // Bookkeeping asserts over the raw image before open.
    let next = u64::from_be_bytes(seg[0..8].try_into().unwrap());
    let seg_cap = u32::from_be_bytes(seg[8..12].try_into().unwrap());
    assert_eq!(next, 12, "seg {name}: next_ordinal");
    assert_eq!(seg_cap, cap, "seg {name}: capacity");
    let path = tmp_segment(name);
    std::fs::write(&path, seg).expect("write segment image");
    // Open-from-bytes: WireRing::open validates capacity-vs-size fail-closed.
    let ring = WireRing::open(&path).expect("open segment image");
    for (i, want) in [b"seg-slot-zero".as_slice(), b"seg-slot-one".as_slice()]
        .into_iter()
        .enumerate()
    {
        let ordinal = 10u32 + i as u32;
        let mut out = vec![0u8; wire_shm::SLOT_LEN];
        let n = ring
            .read_frame_into(ordinal, &mut out)
            .expect("read stamped ordinal");
        assert_eq!(&out[..n], want, "seg {name}: ordinal {ordinal} payload");
    }
    assert_eq!(
        ring.next_ordinal().expect("next"),
        12,
        "seg {name}: next_ordinal"
    );
    std::fs::remove_file(&path).ok();
}
