//! Second consumer court (F4 pattern-5 residual, optional lane): seeds the
//! cgap-parquet dictionary kernel with the AOQ frozen status band as a
//! pre-seeded status page, via cgap-arrow's producer
//! (`cgap_arrow::status_dictionary`).
//!
//! Band law re-pin: 8 distinct entries need ceil(log2(8)) = 3 bits, and
//! the SWAR kernel width set is {1,2,4,8}, so `required_bits(8) = 4`
//! (clamped up), NOT width-1 as the lane brief assumed. The page is
//! width-4; the 8-entry band is under the 256-entry cap.

#![cfg(test)]
use cgap_arrow::status_dictionary::STATUS_BAND;
use cgap_parquet::dictionary::{decode_page, encode_page, required_bits};

#[test]
fn status_band_page_round_trips_through_parquet_dictionary_kernel() {
    // Width law: 9 entries -> kernel clamp to 4 bits (ceil(log2(9)) = 4 <= 4).
    assert_eq!(required_bits(9), Ok(4));

    let names: Vec<&str> = STATUS_BAND.iter().map(|(_, n)| *n).collect();
    assert_eq!(names.len(), 9);
    assert_eq!(names[0], "OK");
    assert_eq!(names[7], "EPOCH_RETIRED");
    assert_eq!(names[8], "UNSUPPORTED_VERSION");

    // The dictionary of distinct slot ids 0..9, one width-4 index page:
    // every slot exactly once, plus a repeat, through the real kernel.
    let dict: Vec<u32> = (0..9u32).collect();
    let indices = vec![7u32, 0, 3, 3, 5, 1, 6, 2, 4, 8];
    let bits = 4u8;

    let page = encode_page(&dict, &indices, bits).expect("in-domain page encodes");
    let mut out = vec![0u32; indices.len()];
    decode_page(&page, &dict, &mut out).expect("page decodes");
    assert_eq!(out, indices);
}
