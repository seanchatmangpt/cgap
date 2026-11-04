//! Wire-side translation: MCP tool payloads to/from wire-shm binary frames.
//!
//! Divergence note (recorded in RECEIPT.md): wire-shm's public
//! `stamp_frame`/`read_frame` allocate internally (`Vec` for bookkeeping and
//! payload buffers), and cgap-mcp owns neither file. The zero-alloc window
//! court therefore measures cgap-mcp's own stack-buffer frame encoder
//! `encode_frame_into`, byte-identity-checked against wire-shm's stamped
//! frame bytes in the round-trip court (drift guard: any wire-shm framing
//! change fails that court).

use wire_shm::{crc32_ieee, WireError, WireRing, HEADER_LEN, MAGIC, MAX_PAYLOAD};

/// Encode a WIR0 frame (header + payload) into `buf` with ZERO heap
/// allocations. Returns bytes written.
pub fn encode_frame_into(buf: &mut [u8], payload: &[u8], ordinal: u32) -> Result<usize, WireError> {
    let n = payload.len();
    if n > MAX_PAYLOAD {
        return Err(WireError::FrameTooLarge {
            len: n,
            max: MAX_PAYLOAD,
        });
    }
    if buf.len() < HEADER_LEN + n {
        return Err(WireError::Io("frame buffer too small".into()));
    }
    buf[0..4].copy_from_slice(&MAGIC.to_be_bytes());
    buf[4..8].copy_from_slice(&crc32_ieee(payload).to_be_bytes());
    buf[8..12].copy_from_slice(&(n as u32).to_be_bytes());
    buf[12..16].copy_from_slice(&ordinal.to_be_bytes());
    buf[16..HEADER_LEN + n].copy_from_slice(payload);
    Ok(HEADER_LEN + n)
}

/// Stamp `message` into the ring at the ring's next ordinal and read it back,
/// verifying the read-back payload matches exactly. Returns the ordinal used.
pub fn echo_wire_roundtrip(ring: &mut WireRing, message: &[u8]) -> Result<u32, WireError> {
    let next = ring.next_ordinal()?;
    if next > u32::MAX as u64 {
        return Err(WireError::Io("ordinal space exhausted".into()));
    }
    let ordinal = next as u32;
    ring.stamp_frame(message, ordinal)?;
    let read_back = ring.read_frame(ordinal)?;
    if read_back != message {
        return Err(WireError::Io(format!(
            "echo mismatch: stamped {} bytes, read back {} bytes",
            message.len(),
            read_back.len()
        )));
    }
    Ok(ordinal)
}
