//! C1 minimal-validation-tier courts (lane NA1, v26.11.2 repair-15).
//!
//! Ladder semantics adopted from nanoarrow ArrowArrayViewValidateMinimal
//! (vendor array.c:1595-1676): the minimal tier is length>=0, offset
//! bounds, offset+length <= i64::MAX, length*elem_size buffer arithmetic,
//! validity skipped when null_count <= 0. Full nested walks stay ABSENT
//! (cgap fixed-width only, N7). Typed fail-closed errors, no panics.

use cgap_arrow::validate::{validate_minimal, ArrayView, BufferId, MinimalError};

fn view(
    length: i64,
    offset: i64,
    null_count: i64,
    validity_bytes: Option<usize>,
    data_bytes: usize,
    elem_size_bytes: usize,
) -> ArrayView {
    ArrayView {
        length,
        offset,
        null_count,
        validity_bytes,
        data_bytes,
        elem_size_bytes,
    }
}

// Court 1: minimal tier passes on well-formed views.
#[test]
fn court1_minimal_passes_wellformed() {
    let s = validate_minimal(&view(4, 0, 0, None, 32, 8)).expect("well-formed");
    assert_eq!(s.offset_plus_length, 4);
    assert!(s.validity_skipped);
    assert_eq!(s.validity_checks, 0);
    // offset window inside the buffer also passes
    let s = validate_minimal(&view(4, 8, 0, Some(8), 1024, 8)).expect("offset ok");
    assert_eq!(s.validity_checks, 1);
    assert_eq!(s.offset_plus_length, 12);
    // non-multiple-of-8 slot counts round up via bytes_for_bits
    let s = validate_minimal(&view(9, 0, 0, None, 18, 2)).expect("i16 x9");
    assert_eq!(s.offset_plus_length, 9);
}

// Court 2: undersized buffers are refused with typed errors.
#[test]
fn court2_undersized_buffers_refused() {
    // data buffer under-covers len*elem
    assert_eq!(
        validate_minimal(&view(4, 0, 0, None, 24, 8)),
        Err(MinimalError::BufferUndersized {
            which: BufferId::Data,
            need: 32,
            got: 24,
        })
    );
    // validity buffer under-covers bytes_for_bits(offset+length)
    assert_eq!(
        validate_minimal(&view(80, 0, 3, Some(4), 640, 8)),
        Err(MinimalError::BufferUndersized {
            which: BufferId::Validity,
            need: 10,
            got: 4,
        })
    );
}

// Court 3: validity work is skipped when null_count <= 0 (observable via
// the MinStats counters; pure arithmetic facts, not mocks).
#[test]
fn court3_validity_skipped_when_null_count_le_zero() {
    // null_count == 0, buffer absent: skip
    let s = validate_minimal(&view(8, 0, 0, None, 64, 8)).expect("ok");
    assert_eq!((s.validity_skipped, s.validity_checks), (true, 0));
    // null_count == 0, buffer present but empty: skip (array.c:1632-1636)
    let s = validate_minimal(&view(8, 0, 0, Some(0), 64, 8)).expect("ok");
    assert_eq!((s.validity_skipped, s.validity_checks), (true, 0));
    // null_count > 0: validity tier entered exactly once
    let s = validate_minimal(&view(8, 0, 2, Some(1), 64, 8)).expect("ok");
    assert_eq!((s.validity_skipped, s.validity_checks), (false, 1));
    // negative null_count is treated as "no nulls known" -> still skip
    let s = validate_minimal(&view(8, 0, -1, None, 64, 8)).expect("ok");
    assert!(s.validity_skipped);
}

// Court 4: negative length / offset and i64 overflow refused fail-closed.
#[test]
fn court4_bounds_refused() {
    assert_eq!(
        validate_minimal(&view(-1, 0, 0, None, 8, 8)),
        Err(MinimalError::NegativeLength { got: -1 })
    );
    assert_eq!(
        validate_minimal(&view(1, -2, 0, None, 8, 8)),
        Err(MinimalError::NegativeOffset { got: -2 })
    );
    assert_eq!(
        validate_minimal(&view(i64::MAX, 1, 0, None, 8, 8)),
        Err(MinimalError::OffsetLengthOverflow)
    );
}
