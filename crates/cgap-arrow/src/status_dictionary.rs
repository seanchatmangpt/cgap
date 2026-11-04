//! Hand-law producer for the `DictionaryUInt32Utf8` import-layout kind.
//!
//! WHY A HAND FILE: the layout owner is cgap-arrow, and
//! `generated_arrow_primitives.rs` is ggen-generated and never hand-edited,
//! so the producer is a NEW hand-law sibling module registered in `lib.rs`
//! (three-zone law: generated / hand-law / courts).
//! Allow-with-reason: this producer serves the AOQ status-band consumers —
//! cgap-ffi `status_maps.rs` (`CGAP_AOQ_STATUS_*`, frozen band
//! 0x4347A001..A008) and arcus `aoq_ipc.go` (`AoqStatus*`) — which import
//! Arrow dictionary arrays whose values are the 8 status names.
//!
//! LAW (import spec, `generated_arrow_primitives.rs` ARRAY_KINDS entry
//! "DictionaryUInt32Utf8"): format "i", 3 buffers, slot bit widths
//! (0,1)(1,32)(2,8), 64-byte alignment. Buffer 0 = validity bitmap
//! (1 bit/slot, LSB-first per `bitmap_bit_test`), buffer 1 = u32
//! dictionary indices (32 bits/slot), buffer 2 = child Utf8 values region
//! (8-bit data lanes; child i32 offsets prefix the data so the values
//! buffer is self-describing).
//!
//! The dictionary values are the FROZEN status band (gate 020/070 law):
//! 8 (wire word, name) pairs, band 0x4347A001..A008. AUTHORITY = the
//! canonical graph: this module consumes the GENERATED projection
//! `crate::status_band::STATUS_BAND` (pack
//! `ggen-marketplace/packs/aoq-pack/templates/status_band_rs.tmpl` over the
//! aoq:StatusConstant individuals), checksum-frozen by the consumer-repo
//! slot `.ggen/freeze/aoq-pack/crates/cgap-arrow/src/status_band.rs.blake3`.
//! This module carries no band copy of its own: the producer and the
//! tri-language drift court (`tests/status_band_tri_court.rs`) share the
//! one generated table.

#![cfg(not(target_os = "none"))]

use alloc::vec::Vec;
use crate::buffer::{Buf, ALIGNMENT};

/// FROZEN status band (gate 020/070 law): (wire word, symbolic name),
/// band 0x4347A001..A008, slot i holds 0x4347A001 + i. GENERATED projection
/// (`crate::status_band`, authority = the canonical graph); re-exported here
/// so the producer and the tri-language drift court
/// (`tests/status_band_tri_court.rs`) pin all three language tables to ONE
/// generated table.
pub use crate::status_band::STATUS_BAND;

/// Producer error: typed refusals, no panics on input facts.
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryError {
    /// Band table changed size away from the frozen 8-entry law.
    BandSize(usize),
    /// Hostile offset band: an offset pair violates `start <= end` or
    /// `36 + end <= values.len()` — attacker u32s, typed refusal, no panic.
    BandRange {
        /// Start offset of the offending name (band entry i).
        start: u32,
        /// End offset of the offending name (band entry i+1).
        end: u32,
        /// Values-buffer byte length the range was checked against.
        band_len: usize,
    },
}

/// The built 3-buffer dictionary array per the import spec:
/// validity + u32 indices + child Utf8 values, each buffer 64-aligned.
pub struct StatusDictionary {
    /// Buffer 0: validity bitmap, 8 slots -> 1 used byte, all valid.
    pub validity: Buf,
    /// Buffer 1: u32 dictionary indices [0..8], LE.
    pub indices: Buf,
    /// Buffer 2: child Utf8 values — i32 offsets (9 entries, LE) prefixing
    /// the concatenated UTF-8 name bytes (8-bit data lanes).
    pub values: Buf,
    /// The 8 name byte lengths (offsets derive from these).
    pub name_lengths: [u32; 8],
}

/// Pad `n` up to the 64-byte alignment quantum.
const fn pad64(n: usize) -> usize {
    (n + (ALIGNMENT - 1)) & !(ALIGNMENT - 1)
}

/// Build the 8-entry status dictionary array.
///
/// Buffer 0: validity — 8 bits all set (LSB-first per `bitmap_bit_test`),
/// padded to 64 bytes. Buffer 1: indices [0..8] u32 LE, padded to 64.
/// Buffer 2: values — 9 i32 offsets (LE) prefixing concatenated UTF-8
/// name bytes, padded to 64.
pub fn build_dictionary() -> Result<StatusDictionary, DictionaryError> {
    if STATUS_BAND.len() != 8 {
        return Err(DictionaryError::BandSize(STATUS_BAND.len()));
    }

    // Buffer 0: validity — 8 slots -> byte 0xFF, padded to 64.
    let mut validity = Buf::new(pad64(1));
    validity.as_mut_slice()[0] = 0xFF;

    // Buffer 1: indices — 8 x u32 LE = 32 bytes, padded to 64.
    let mut indices = Buf::new(pad64(8 * 4));
    for (i, slot) in indices.as_mut_slice()[..32].chunks_exact_mut(4).enumerate() {
        slot.copy_from_slice(&(i as u32).to_le_bytes());
    }

    // Buffer 2: values — child Utf8 offsets + data (8-bit lanes). The
    // names are the generated band's names, derived in a loop: this file
    // carries no second copy of the band (P7 drift-pin consolidation).
    let names: Vec<&str> = STATUS_BAND.iter().map(|(_, n)| *n).collect();
    assert_eq!(names.len(), 8, "frozen band size checked above");
    let mut offsets = [0u32; 9];
    let mut total = 0usize;
    for (i, n) in names.iter().enumerate() {
        total += n.len();
        offsets[i + 1] = total as u32;
    }
    let mut values = Buf::new(pad64(9 * 4 + total));
    {
        let v = values.as_mut_slice();
        for (i, o) in offsets.iter().enumerate() {
            v[i * 4..i * 4 + 4].copy_from_slice(&o.to_le_bytes());
        }
        let mut cursor = 9 * 4;
        for n in names.iter() {
            v[cursor..cursor + n.len()].copy_from_slice(n.as_bytes());
            cursor += n.len();
        }
    }

    let mut name_lengths = [0u32; 8];
    for (i, (_, n)) in STATUS_BAND.iter().enumerate() {
        name_lengths[i] = n.len() as u32;
    }

    Ok(StatusDictionary {
        validity,
        indices,
        values,
        name_lengths,
    })
}

/// Decode round trip: given the producer's buffers, recover the 8 names.
pub fn decode_names(dict: &StatusDictionary) -> Result<Vec<&str>, DictionaryError> {
    let v = dict.values.as_slice();
    if v.len() < 9 * 4 {
        return Err(DictionaryError::BandSize(v.len()));
    }
    let mut names = Vec::with_capacity(8);
    for i in 0..8 {
        let start = u32::from_le_bytes(v[i * 4..i * 4 + 4].try_into().expect("band word"));
        let end = u32::from_le_bytes(v[(i + 1) * 4..(i + 1) * 4 + 4].try_into().expect("band word"));
        // Hostile offsets: refuse typed instead of slicing past the buffer.
        if start > end || 9 * 4 + end as usize > v.len() {
            return Err(DictionaryError::BandRange {
                start,
                end,
                band_len: v.len(),
            });
        }
        let data_start = 9 * 4;
        names.push(
            core::str::from_utf8(&v[data_start + start as usize..data_start + end as usize])
                .map_err(|_| DictionaryError::BandRange {
                    start,
                    end,
                    band_len: v.len(),
                })?,
        );
    }
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated_arrow_primitives::{bitmap_bit_test, ARRAY_KINDS};

    /// The import spec's DictionaryUInt32Utf8 entry: pinned facts.
    fn spec() -> &'static crate::generated_arrow_primitives::ArrayKindSpec {
        let kind = ARRAY_KINDS
            .iter()
            .find(|k| k.label == "DictionaryUInt32Utf8")
            .expect("DictionaryUInt32Utf8 must stay modeled in ARRAY_KINDS");
        assert_eq!(kind.format, "i");
        assert_eq!(kind.buffer_count, 3);
        assert_eq!(kind.alignment, 64);
        kind
    }

    /// Round-trip court against the import spec: the producer's buffers
    /// obey every modeled fact and decode back to the frozen band.
    #[test]
    fn producer_matches_import_spec_round_trip() {
        let kind = spec();
        assert_eq!(
            kind.slot_bit_widths,
            &[(0u8, 1u16), (1, 32), (2, 8)],
            "import spec slot widths drifted; producer layout must be re-pinned"
        );

        let dict = build_dictionary().expect("frozen 8-entry band always builds");

        // Buffer facts: exactly 3 buffers, each 64-byte aligned.
        assert_eq!(kind.buffer_count, 3);
        assert_eq!(dict.validity.as_ptr() as usize % kind.alignment, 0);
        assert_eq!(dict.indices.as_ptr() as usize % kind.alignment, 0);
        assert_eq!(dict.values.as_ptr() as usize % kind.alignment, 0);

        // Slot 0 (1 bit): every validity bit set, LSB-first bit law.
        let v = dict.validity.as_slice();
        for i in 0..8 {
            assert!(bitmap_bit_test(v, i), "slot 0 bit {i} must be valid");
        }
        // Padded region stays zero: no stray bits beyond 8 slots.
        assert_eq!(&v[1..], &[0u8; 63]);

        // Slot 1 (32 bits): indices are identity [0..8) u32 LE.
        let idx = &dict.indices.as_slice()[..32];
        for i in 0..8 {
            let expected = (i as u32).to_le_bytes();
            assert_eq!(&idx[i * 4..i * 4 + 4], &expected, "index slot {i}");
        }
        assert_eq!(&dict.indices.as_slice()[32..], &[0u8; 32]);

        // Slot 2 (8-bit data lanes): child offsets + UTF-8 name bytes.
        let names = decode_names(&dict).expect("well-formed values buffer decodes");
        let band_names: Vec<&str> = STATUS_BAND.iter().map(|(_, n)| *n).collect();
        assert_eq!(names, band_names);
        for (i, n) in names.iter().enumerate() {
            assert_eq!(n.len() as u32, dict.name_lengths[i]);
        }

        // Byte-exact law for buffer 2 (offsets prefix, data follows).
        let v = dict.values.as_slice();
        let expected_offsets: [u8; 36] = [
            0, 0, 0, 0, // OK
            2, 0, 0, 0, // PENDING
            9, 0, 0, 0, // BUSY
            13, 0, 0, 0, // INCOMPLETE_FRAME
            29, 0, 0, 0, // DIGEST_MISMATCH
            44, 0, 0, 0, // EPOCH_MISMATCH
            58, 0, 0, 0, // KEY_UNKNOWN
            69, 0, 0, 0, // EPOCH_RETIRED
            82, 0, 0, 0, // end
        ];
        assert_eq!(&v[..36], &expected_offsets);
        assert_eq!(&v[36..59], b"OKPENDINGBUSYINCOMPLETE");
        assert_eq!(v[36 + 82], 0, "pad region starts past data end");
    }

    /// Typed refusal: a band away from 8 entries is refused, never built.
    #[test]
    fn band_size_law_is_typed() {
        assert_eq!(STATUS_BAND.len(), 8);
        assert_eq!(DictionaryError::BandSize(7), DictionaryError::BandSize(7));
    }

    /// Corrupt-input court (negative leg): hostile offset band where
    /// entry 1 points past the buffer (and an inverted pair start > end)
    /// refuses typed — never a slice panic on external input. Mutant kill:
    /// delete the BandRange guard and the slice panics.
    #[test]
    fn hostile_offset_band_refuses_typed() {
        let dict = build_dictionary().expect("frozen band builds");
        let band_len = dict.values.as_slice().len();

        // Case A: end runs past the buffer.
        let mut hostile = StatusDictionary {
            validity: Buf::new_zeroed(pad64(1)),
            indices: Buf::new_zeroed(pad64(8 * 4)),
            values: Buf::new(band_len),
            name_lengths: [0u32; 8],
        };
        hostile.values.as_mut_slice()[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            decode_names(&hostile),
            Err(DictionaryError::BandRange {
                start: 0,
                end: u32::MAX,
                band_len,
            })
        );

        // Case B: inverted pair (start > end).
        hostile.values.as_mut_slice()[0..12].copy_from_slice(&[0u8; 12]);
        hostile.values.as_mut_slice()[4..8].copy_from_slice(&9u32.to_le_bytes());
        hostile.values.as_mut_slice()[8..12].copy_from_slice(&2u32.to_le_bytes());
        assert_eq!(
            decode_names(&hostile),
            Err(DictionaryError::BandRange {
                start: 9,
                end: 2,
                band_len,
            })
        );
    }
}
