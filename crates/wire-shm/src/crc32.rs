//! Table-driven CRC-32 (IEEE 802.3, poly 0xEDB8_8320), in-crate, zero-dep.

const POLY: u32 = 0xEDB8_8320;

/// Lazily-free static table built once via OnceLock.
use std::sync::OnceLock;

fn table() -> &'static [u32; 256] {
    static TABLE: OnceLock<[u32; 256]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        let mut i = 0usize;
        while i < 256 {
            let mut c = i as u32;
            let mut k = 0;
            while k < 8 {
                c = if c & 1 != 0 { (c >> 1) ^ POLY } else { c >> 1 };
                k += 1;
            }
            t[i] = c;
            i += 1;
        }
        t
    })
}

/// CRC-32/IEEE of `data`, big-endian framing convention (init 0xFFFF_FFFF,
/// final xor 0xFFFF_FFFF — standard reflected-algorithm value).
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let t = table();
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        let idx = ((crc ^ b as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ t[idx];
    }
    crc ^ 0xFFFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        assert_eq!(crc32_ieee(b""), 0x0000_0000);
        assert_eq!(crc32_ieee(b"123456789"), 0xCBF4_3926);
        assert_eq!(
            crc32_ieee(b"hello cgap wire"),
            crc32_ieee(b"hello cgap wire")
        );
    }
}
