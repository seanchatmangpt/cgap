//! Lane GIT-08: canonical-format lease commit synthesis in a fixed
//! 1,024-byte scratch (no heap, no dynamic strings, no clock reads).
//!
//! Wire grammar (named consts below, byte-for-byte):
//!
//! ```text
//! tree <64hex>\n
//! parent <64hex>\n
//! author <ts> <tz>\n
//! committer <ts> <tz>\n
//! \n
//! <message>\n
//! lease-phi <64hex>\n
//! lease-delta <decimal>\n
//! lease-receipt <160hex>\n
//! ```
//!
//! Divergence from git canonical commit form: the 64-hex object names are
//! BLAKE3 digests (32 B, cgap-blake3 `hash`), NOT SHA-1. The module takes
//! the 32 B digests as caller inputs and renders them as 64 hex; it does
//! not hash the commit body itself (no cgap-blake3 dependency in this
//! zero-dep kernel). The receipt binding (payload digest of the commit
//! prefix) is the caller's policy; the court computes it with the real
//! cgap-blake3 register.
//!
//! Receipt layout (80 B): C7 framing (`crates/csct/src/bin/
//! cscb_c7_receipt_sealing.rs`) exists in the tree but ships no cgap-git
//! consumable module, so per dispatch this lane defines the layout: nonce
//! u64 LE (0..8) + parent 32 B (8..40) + payload 32 B (40..72) + tag 8 B
//! (72..80). Tag = first 8 bytes of SHA-1 over
//! `DOMAIN_LEASE_RECEIPT || nonce_le || parent || payload`, using an
//! in-module SHA-1 copy pinned byte-equal to the crate's public
//! `pack_write::sha1` by the court (lib.rs is off-limits this lane, so the
//! module must be self-contained under `#[path]` include).

// court-consumed surface (CEN2 ALLOW-WITH-REASON): commit_synth_court / e2e_wire_substrate_court / soc_full_chip_court; wire consumers pending — module-level allow (uniform surface, not scattered attrs).
#![allow(dead_code)]
#![deny(unsafe_code)]

use core::fmt;

/// Fixed scratch size. Every commit must render inside it.
pub const SCRATCH_LEN: usize = 1024;

/// Caller message bound (bytes).
pub const MAX_MESSAGE_LEN: usize = 256;

/// Fixed timezone literal rendered after both timestamps.
pub const TZ_UTC: &str = "+0000";

/// 64-hex rendering of a 32-byte digest.
pub const DIGEST_HEX_LEN: usize = 64;

/// Receipt hex rendering of the 80-byte receipt.
pub const RECEIPT_HEX_LEN: usize = 160;

/// Wire keys. These consts ARE the grammar.
pub const KEY_TREE: &str = "tree ";
pub const KEY_PARENT: &str = "parent ";
pub const KEY_AUTHOR: &str = "author ";
pub const KEY_COMMITTER: &str = "committer ";
pub const KEY_LEASE_PHI: &str = "lease-phi ";
pub const KEY_LEASE_DELTA: &str = "lease-delta ";
pub const KEY_LEASE_RECEIPT: &str = "lease-receipt ";

/// Domain separator folded into the receipt tag.
pub const DOMAIN_LEASE_RECEIPT: &str = "cgap-lease-receipt-v1";

/// Typed failures. `LeaseTrailerMissing` is the AndonRefusal class: a
/// commit that omits the P28 lease trailers is refused admission, mapped
/// by callers to the exit-101 Andon family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitSynthError {
    /// Render would exceed the fixed 1,024-byte scratch.
    ScratchExhausted,
    /// Message exceeds [`MAX_MESSAGE_LEN`].
    MessageTooLong,
    /// Message or digest hex contains bytes outside the grammar alphabet.
    IllegalByte(u8),
    /// Header lines (tree/parent/author/committer) malformed.
    MalformedHeader,
    /// Body before the trailer block is empty.
    EmptyMessage,
    /// A `lease-*` trailer is absent. AndonRefusal class.
    LeaseTrailerMissing(&'static str),
    /// Trailer present but its value does not match the grammar.
    MalformedTrailer(&'static str),
    /// Receipt tag over the decoded 72-byte prefix does not match the
    /// wire tag — the receipt was tampered or is foreign.
    ReceiptTagMismatch,
    /// Trailing bytes after the final trailer newline.
    TrailingGarbage,
}

impl fmt::Display for CommitSynthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitSynthError::ScratchExhausted => f.write_str("scratch exhausted (1024B cap)"),
            CommitSynthError::MessageTooLong => f.write_str("message exceeds 256B bound"),
            CommitSynthError::IllegalByte(b) => write!(f, "illegal byte 0x{b:02x}"),
            CommitSynthError::MalformedHeader => f.write_str("malformed header line"),
            CommitSynthError::EmptyMessage => f.write_str("empty commit message"),
            CommitSynthError::LeaseTrailerMissing(k) => {
                write!(f, "andon: lease trailer missing: {k}")
            }
            CommitSynthError::MalformedTrailer(k) => write!(f, "malformed trailer: {k}"),
            CommitSynthError::ReceiptTagMismatch => f.write_str("receipt tag mismatch"),
            CommitSynthError::TrailingGarbage => f.write_str("trailing bytes after trailers"),
        }
    }
}

/// 80-byte P28 lease receipt: nonce u64 LE + parent 32B + payload 32B +
/// tag 8B (SHA-1-derived, domain-separated). 80 bytes, no padding holes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseReceipt {
    pub nonce: u64,
    pub parent: [u8; 32],
    pub payload: [u8; 32],
    pub tag: [u8; 8],
}

const RECEIPT_LEN: usize = 80;

impl LeaseReceipt {
    /// Domain-separated tag over the 72-byte prefix.
    pub fn expected_tag(nonce: u64, parent: &[u8; 32], payload: &[u8; 32]) -> [u8; 8] {
        let mut buf = [0u8; 21 + 8 + 32 + 32];
        buf[..21].copy_from_slice(DOMAIN_LEASE_RECEIPT.as_bytes());
        buf[21..29].copy_from_slice(&nonce.to_le_bytes());
        buf[29..61].copy_from_slice(parent);
        buf[61..93].copy_from_slice(payload);
        let d = sha1(&buf);
        let mut tag = [0u8; 8];
        tag.copy_from_slice(&d[..8]);
        tag
    }

    /// Seal a new receipt (tag computed here, not caller-supplied).
    pub fn seal(nonce: u64, parent: [u8; 32], payload: [u8; 32]) -> Self {
        LeaseReceipt {
            nonce,
            parent,
            payload,
            tag: Self::expected_tag(nonce, &parent, &payload),
        }
    }

    pub fn to_bytes(&self) -> [u8; RECEIPT_LEN] {
        let mut b = [0u8; RECEIPT_LEN];
        b[0..8].copy_from_slice(&self.nonce.to_le_bytes());
        b[8..40].copy_from_slice(&self.parent);
        b[40..72].copy_from_slice(&self.payload);
        b[72..80].copy_from_slice(&self.tag);
        b
    }

    pub fn from_bytes(b: &[u8; RECEIPT_LEN]) -> Result<Self, CommitSynthError> {
        let mut nonce_b = [0u8; 8];
        nonce_b.copy_from_slice(&b[0..8]);
        let mut parent = [0u8; 32];
        parent.copy_from_slice(&b[8..40]);
        let mut payload = [0u8; 32];
        payload.copy_from_slice(&b[40..72]);
        let mut tag = [0u8; 8];
        tag.copy_from_slice(&b[72..80]);
        Ok(LeaseReceipt {
            nonce: u64::from_le_bytes(nonce_b),
            parent,
            payload,
            tag,
        })
    }
}

/// The full commit lease spec. Timestamps are caller-provided u64
/// (no clock reads in no_std — determinism is the court's pin).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseCommitSpec<'a> {
    /// BLAKE3 digest of the tree (caller computes with cgap-blake3).
    pub tree: [u8; 32],
    /// BLAKE3 digest of the parent commit (zeroes on root).
    pub parent: [u8; 32],
    pub author_ts: u64,
    pub committer_ts: u64,
    /// Message body, at most [`MAX_MESSAGE_LEN`] bytes, no `\n` required.
    pub message: &'a str,
    /// Φ_lease — 32-byte lease digest.
    pub lease_phi: [u8; 32],
    /// ΔS_admitted — admitted state delta.
    pub lease_delta: u64,
    /// Sealed 80-byte P28 lease receipt.
    pub receipt: LeaseReceipt,
}

/// The fixed 1,024-byte scratch. `render` writes the canonical commit
/// bytes into `buf[..len]`; nothing else is allocated anywhere.
#[derive(Clone)]
pub struct CommitScratch {
    buf: [u8; SCRATCH_LEN],
    len: usize,
}

impl CommitScratch {
    pub const fn new() -> Self {
        CommitScratch {
            buf: [0u8; SCRATCH_LEN],
            len: 0,
        }
    }

    pub fn reset(&mut self) {
        self.len = 0;
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const CAPACITY: usize = SCRATCH_LEN;

    /// Render the canonical commit into the scratch. Returns the written
    /// slice. Fail-closed on any bound violation.
    pub fn render(&mut self, spec: &LeaseCommitSpec<'_>) -> Result<&[u8], CommitSynthError> {
        self.len = 0;
        for b in spec.message.bytes() {
            if b == b'\n' {
                return Err(CommitSynthError::IllegalByte(b));
            }
        }
        for k in [KEY_LEASE_PHI, KEY_LEASE_DELTA, KEY_LEASE_RECEIPT] {
            if spec.message.starts_with(k) {
                return Err(CommitSynthError::IllegalByte(b' '));
            }
        }
        if spec.message.len() > MAX_MESSAGE_LEN {
            return Err(CommitSynthError::MessageTooLong);
        }
        self.push(KEY_TREE.as_bytes())?;
        self.push_hex32(&spec.tree)?;
        self.push(b"\n")?;
        self.push(KEY_PARENT.as_bytes())?;
        self.push_hex32(&spec.parent)?;
        self.push(b"\n")?;
        self.push(KEY_AUTHOR.as_bytes())?;
        self.push_u64(spec.author_ts)?;
        self.push(b" ")?;
        self.push(TZ_UTC.as_bytes())?;
        self.push(b"\n")?;
        self.push(KEY_COMMITTER.as_bytes())?;
        self.push_u64(spec.committer_ts)?;
        self.push(b" ")?;
        self.push(TZ_UTC.as_bytes())?;
        self.push(b"\n\n")?;
        self.push(spec.message.as_bytes())?;
        self.push(b"\n")?;
        self.push(KEY_LEASE_PHI.as_bytes())?;
        self.push_hex32(&spec.lease_phi)?;
        self.push(b"\n")?;
        self.push(KEY_LEASE_DELTA.as_bytes())?;
        self.push_u64(spec.lease_delta)?;
        self.push(b"\n")?;
        self.push(KEY_LEASE_RECEIPT.as_bytes())?;
        let rb = spec.receipt.to_bytes();
        self.push_hex(&rb)?;
        self.push(b"\n")?;
        Ok(self.as_bytes())
    }
}

impl Default for CommitScratch {
    fn default() -> Self {
        Self::new()
    }
}

use core::ops::Deref;
impl Deref for CommitScratch {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// ------------------------------------------------------------ emit fns --
impl CommitScratch {
    fn push(&mut self, s: &[u8]) -> Result<(), CommitSynthError> {
        if self.len + s.len() > SCRATCH_LEN {
            return Err(CommitSynthError::ScratchExhausted);
        }
        self.buf[self.len..self.len + s.len()].copy_from_slice(s);
        self.len += s.len();
        Ok(())
    }

    fn push_hex(&mut self, bytes: &[u8]) -> Result<(), CommitSynthError> {
        for &b in bytes {
            let hi = b >> 4;
            let lo = b & 0x0F;
            self.push(&[
                if hi < 10 { b'0' + hi } else { b'a' + hi - 10 },
                if lo < 10 { b'0' + lo } else { b'a' + lo - 10 },
            ])?;
        }
        Ok(())
    }

    fn push_hex32(&mut self, d: &[u8; 32]) -> Result<(), CommitSynthError> {
        self.push_hex(d)
    }

    fn push_u64(&mut self, v: u64) -> Result<(), CommitSynthError> {
        let mut tmp = [0u8; 20];
        let mut n = 20usize;
        let mut v = v;
        if v == 0 {
            tmp[19] = b'0';
            n = 19;
        }
        while v > 0 {
            tmp[n - 1] = b'0' + (v % 10) as u8;
            n -= 1;
            v /= 10;
        }
        self.push(&tmp[n..])
    }
}

// ------------------------------------------------------------ validate --
/// Parsed + receipt-verified commit fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseCommitFields {
    pub tree: [u8; 32],
    pub parent: [u8; 32],
    pub author_ts: u64,
    pub committer_ts: u64,
    pub message: [u8; MAX_MESSAGE_LEN],
    pub message_len: usize,
    pub lease_phi: [u8; 32],
    pub lease_delta: u64,
    pub receipt: LeaseReceipt,
}

fn decode_hex(src: &[u8], out: &mut [u8]) -> Result<(), CommitSynthError> {
    if src.len() != out.len() * 2 {
        return Err(CommitSynthError::MalformedTrailer("hex length"));
    }
    for (i, out_b) in out.iter_mut().enumerate() {
        let hi = hex_val(src[2 * i])?;
        let lo = hex_val(src[2 * i + 1])?;
        *out_b = (hi << 4) | lo;
    }
    Ok(())
}

fn hex_val(b: u8) -> Result<u8, CommitSynthError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err(CommitSynthError::IllegalByte(b)),
    }
}

fn parse_u64(src: &[u8]) -> Result<u64, CommitSynthError> {
    if src.is_empty() {
        return Err(CommitSynthError::MalformedHeader);
    }
    let mut v: u64 = 0;
    for &b in src {
        if !b.is_ascii_digit() {
            return Err(CommitSynthError::MalformedHeader);
        }
        v = v
            .checked_mul(10)
            .and_then(|x| x.checked_add((b - b'0') as u64))
            .ok_or(CommitSynthError::MalformedHeader)?;
    }
    Ok(v)
}

/// Validate canonical commit bytes: full grammar check plus P28 lease
/// receipt tag verification. A commit missing any `lease-*` trailer is
/// refused with [`CommitSynthError::LeaseTrailerMissing`] (AndonRefusal
/// class). Fail-closed on every violation.
pub fn validate(buf: &[u8]) -> Result<LeaseCommitFields, CommitSynthError> {
    let mut cur = Cursor { buf, pos: 0 };
    let tree_hex = cur.line_prefix(KEY_TREE.as_bytes())?;
    let parent_hex = cur.line_prefix(KEY_PARENT.as_bytes())?;
    let author = cur.line_prefix(KEY_AUTHOR.as_bytes())?;
    let committer = cur.line_prefix(KEY_COMMITTER.as_bytes())?;
    cur.blank_line()?;
    let msg = cur.message_and_trailers()?;
    let phi_line = cur
        .line_prefix(KEY_LEASE_PHI.as_bytes())
        .map_err(|_| CommitSynthError::LeaseTrailerMissing(KEY_LEASE_PHI))?;
    let delta_line = cur
        .line_prefix(KEY_LEASE_DELTA.as_bytes())
        .map_err(|_| CommitSynthError::LeaseTrailerMissing(KEY_LEASE_DELTA))?;
    let receipt_line = cur
        .line_prefix(KEY_LEASE_RECEIPT.as_bytes())
        .map_err(|_| CommitSynthError::LeaseTrailerMissing(KEY_LEASE_RECEIPT))?;
    if cur.pos != buf.len() {
        return Err(CommitSynthError::TrailingGarbage);
    }
    let mut tree = [0u8; 32];
    decode_hex(tree_hex, &mut tree)?;
    let mut parent = [0u8; 32];
    decode_hex(parent_hex, &mut parent)?;
    let author_ts = parse_ts(author)?;
    let committer_ts = parse_ts(committer)?;
    let mut lease_phi = [0u8; 32];
    decode_hex(phi_line, &mut lease_phi)?;
    let lease_delta = parse_u64(delta_line)?;
    let mut rb = [0u8; 80];
    decode_hex(receipt_line, &mut rb)?;
    let receipt = LeaseReceipt::from_bytes(&rb)?;
    if receipt.tag != LeaseReceipt::expected_tag(receipt.nonce, &receipt.parent, &receipt.payload) {
        return Err(CommitSynthError::ReceiptTagMismatch);
    }
    Ok(LeaseCommitFields {
        tree,
        parent,
        author_ts,
        committer_ts,
        message: msg.0,
        message_len: msg.1,
        lease_phi,
        lease_delta,
        receipt,
    })
}

/// Sequential cursor over the canonical byte form. No heap: every field
/// is a borrowed slice or a fixed array.
struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Consume one full line; the line must start with `prefix`.
    fn line_prefix(&mut self, prefix: &[u8]) -> Result<&'a [u8], CommitSynthError> {
        let line = self.next_line()?;
        if !line.starts_with(prefix) {
            return Err(CommitSynthError::MalformedHeader);
        }
        Ok(&line[prefix.len()..])
    }

    fn next_line(&mut self) -> Result<&'a [u8], CommitSynthError> {
        let rest = &self.buf[self.pos..];
        let nl = rest
            .iter()
            .position(|&b| b == b'\n')
            .ok_or(CommitSynthError::TrailingGarbage)?;
        self.pos += nl + 1;
        Ok(&rest[..nl])
    }

    fn blank_line(&mut self) -> Result<(), CommitSynthError> {
        if self.pos >= self.buf.len() || self.buf[self.pos] != b'\n' {
            return Err(CommitSynthError::MalformedHeader);
        }
        self.pos += 1;
        Ok(())
    }

    /// Consume everything up to (not including) the next trailer line.
    /// Returns the message copied into a fixed 256-byte field. A message
    /// line beginning with a `lease-*` trailer key is illegal (it would
    /// make the trailer boundary ambiguous), so the caller must keep the
    /// message free of such lines; the scratch render enforces this.
    fn message_and_trailers(&mut self) -> Result<([u8; MAX_MESSAGE_LEN], usize), CommitSynthError> {
        let mut msg = [0u8; MAX_MESSAGE_LEN];
        let mut mlen = 0usize;
        let mut consumed = 0usize;
        loop {
            let rest = &self.buf[self.pos + consumed..];
            if starts_with_any(rest) {
                break;
            }
            let nl = rest
                .iter()
                .position(|&b| b == b'\n')
                .ok_or(CommitSynthError::TrailingGarbage)?;
            let line = &rest[..nl];
            if mlen + line.len() + 1 > MAX_MESSAGE_LEN {
                return Err(CommitSynthError::MessageTooLong);
            }
            if mlen > 0 {
                msg[mlen] = b'\n';
                mlen += 1;
            }
            msg[mlen..mlen + line.len()].copy_from_slice(line);
            mlen += line.len();
            consumed += nl + 1;
        }
        if mlen == 0 {
            return Err(CommitSynthError::EmptyMessage);
        }
        self.pos += consumed;
        Ok((msg, mlen))
    }
}

fn starts_with_any(rest: &[u8]) -> bool {
    for k in [KEY_LEASE_PHI, KEY_LEASE_DELTA, KEY_LEASE_RECEIPT].iter() {
        if rest.starts_with(k.as_bytes()) {
            return true;
        }
    }
    false
}

/// `<ts> +0000`, nothing else.
fn parse_ts(src: &[u8]) -> Result<u64, CommitSynthError> {
    let mut it = src.split(|&b| b == b' ');
    let ts = parse_u64(it.next().ok_or(CommitSynthError::MalformedHeader)?)?;
    if it.next() != Some(TZ_UTC.as_bytes()) || it.next().is_some() {
        return Err(CommitSynthError::MalformedHeader);
    }
    Ok(ts)
}

// -------------------------------------------------------- in-module SHA-1 --
// lib.rs is off-limits this lane, so `crate::pack_write::sha1` is unusable
// under the court's `#[path]` include; this is a verbatim private copy of
// the crate's public implementation, pinned byte-equal to it by the court
// (`private_sha1_matches_pack_write_sha1`) so the two cannot drift.

const fn sha1_init() -> [u32; 5] {
    [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ]
}

fn sha1(data: &[u8]) -> [u8; 20] {
    let mut state = sha1_init();
    let mut len_bits: u64 = 0;
    let mut buf = [0u8; 64];
    let mut buf_len = 0usize;
    let mut rest = data;
    len_bits = len_bits.wrapping_add((rest.len() as u64) * 8);
    while !rest.is_empty() {
        let n = core::cmp::min(64 - buf_len, rest.len());
        buf[buf_len..buf_len + n].copy_from_slice(&rest[..n]);
        buf_len += n;
        rest = &rest[n..];
        if buf_len == 64 {
            sha1_compress(&mut state, &buf);
            buf_len = 0;
        }
    }
    // Padding: 0x80, zeroes, then the big-endian bit count, total a
    // multiple of 64. Two explicit cases; mirrors pack_write's byte-fed
    // padding byte-for-byte.
    let bits = len_bits;
    let mut block = [0u8; 64];
    if buf_len < 56 {
        block[..buf_len].copy_from_slice(&buf[..buf_len]);
        block[buf_len] = 0x80;
        block[56..64].copy_from_slice(&bits.to_be_bytes());
        sha1_compress(&mut state, &block);
    } else {
        block[..buf_len].copy_from_slice(&buf[..buf_len]);
        block[buf_len] = 0x80;
        sha1_compress(&mut state, &block);
        let mut tail = [0u8; 64];
        tail[56..64].copy_from_slice(&bits.to_be_bytes());
        sha1_compress(&mut state, &tail);
    }
    let mut out = [0u8; 20];
    for (i, w) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&w.to_be_bytes());
    }
    out
}

fn sha1_compress(state: &mut [u32; 5], block: &[u8; 64]) {
    let mut w = [0u32; 80];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }
    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = *state;
    for (i, &wi) in w.iter().enumerate() {
        let (f, k) = match i {
            0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999),
            20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
            _ => (b ^ c ^ d, 0xCA62_C1D6),
        };
        let tmp = a
            .rotate_left(5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(k)
            .wrapping_add(wi);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = tmp;
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
}
