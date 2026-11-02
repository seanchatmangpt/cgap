//! Fixed-width statement validator (A2P prune lane, v26.11.2).
//!
//! Minimal fail-closed validation for the fixed-width core surface: a
//! 3-column u64 statement view is checked for length vs buffer capacity
//! and 8-byte alignment, with no nested/offset walk. Typed errors, no
//! panics on inputs (kernel-crate law).

use core::fmt;

/// Typed validation error, fail-closed: every refusal names the column
/// and the violated law.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidateError {
    /// Column buffer too small for `len` u64 slots (`len * 8 > buf.len()`).
    LengthOverflow {
        col: usize,
        len: usize,
        bytes: usize,
    },
    /// Column buffer base not 8-byte aligned for u64 elements.
    Misaligned { col: usize },
    /// Too many columns (fixed-width statement view is exactly 3).
    ColumnCount { got: usize },
}

impl fmt::Display for ValidateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidateError::LengthOverflow { col, len, bytes } => write!(
                f,
                "column {} overflow: len*8={} > buffer {} bytes",
                col,
                len * 8,
                bytes
            ),
            ValidateError::Misaligned { col } => {
                write!(f, "column {} buffer not 8-byte aligned", col)
            }
            ValidateError::ColumnCount { got } => {
                write!(f, "expected 3 statement columns, got {}", got)
            }
        }
    }
}

/// A raw column view: base pointer + slot count + buffer bytes, no nested
/// or offset walk. The base is never dereferenced by the validator.
#[derive(Clone, Copy)]
pub struct FixedWidthColumn {
    pub base: *const u64,
    pub len: usize,
    pub bytes: usize,
}

/// Validate a 3-column u64 statement view: per column, assert the base is
/// 8-byte aligned and `len * 8 <= buffer bytes`. Fail-closed.
pub fn validate_fixed_width(columns: &[Option<FixedWidthColumn>; 3]) -> Result<(), ValidateError> {
    for (i, col) in columns.iter().enumerate() {
        let col = match col {
            Some(c) => c,
            None => return Err(ValidateError::Misaligned { col: i }),
        };
        if col.base.align_offset(8) != 0 {
            return Err(ValidateError::Misaligned { col: i });
        }
        if col.len.saturating_mul(8) > col.bytes {
            return Err(ValidateError::LengthOverflow {
                col: i,
                len: col.len,
                bytes: col.bytes,
            });
        }
    }
    Ok(())
}

/// Validation-level ladder (nanoarrow C1, synthesis §R6/C1; array.c:1595
/// `ArrowArrayViewValidateMinimal`). cgap is fixed-width only (N7): the
/// full nested walks (DEFAULT/FULL) stay ABSENT by design.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationLevel {
    /// No validation at all (nanoarrow NANOARROW_VALIDATION_LEVEL_NONE).
    None,
    /// Length >= 0, offset bounds, length*elem_size buffer arithmetic,
    /// validity skipped when null_count <= 0. No buffer dereference.
    Minimal,
}

/// A geometry-only view (no dereference): length/offset/null_count plus
/// buffer byte-coverages, mirroring nanoarrow's buffer_views[0..2].
#[derive(Clone, Copy, Debug)]
pub struct ArrayView {
    pub length: i64,
    pub offset: i64,
    pub null_count: i64,
    /// Validity buffer size in bytes (None = absent, size 0).
    pub validity_bytes: Option<usize>,
    /// Data buffer size in bytes.
    pub data_bytes: usize,
    /// Element size in bytes for the fixed-width data buffer.
    pub elem_size_bytes: usize,
}

/// Typed minimal-tier error, fail-closed: names the violated law.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinimalError {
    /// `length < 0`.
    NegativeLength { got: i64 },
    /// `offset < 0`.
    NegativeOffset { got: i64 },
    /// `offset + length > i64::MAX`.
    OffsetLengthOverflow,
    /// Buffer under-covers `offset + length` (`need > got` bytes).
    BufferUndersized {
        which: BufferId,
        need: usize,
        got: usize,
    },
}

/// Which of the first two buffers refused (validity=0, data=1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferId {
    Validity,
    Data,
}

/// Observable work counters for the minimal tier (court observability for
/// the null_count <= 0 validity skip; not a mock — pure arithmetic facts).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinStats {
    pub offset_plus_length: i64,
    /// Number of validity-buffer checks actually performed.
    pub validity_checks: u64,
    /// True when the validity tier was skipped because null_count <= 0
    /// and the validity buffer is absent/empty.
    pub validity_skipped: bool,
}

/// Minimal validation tier (C1 first tier). Never dereferences buffers;
/// pure integer arithmetic, fail-closed, no panics.
pub fn validate_minimal(v: &ArrayView) -> Result<MinStats, MinimalError> {
    if v.length < 0 {
        return Err(MinimalError::NegativeLength { got: v.length });
    }
    if v.offset < 0 {
        return Err(MinimalError::NegativeOffset { got: v.offset });
    }
    // offset + length must fit i64 (u64 math, no overflow panic).
    let opl = v.offset as u64 + v.length as u64;
    if opl > i64::MAX as u64 {
        return Err(MinimalError::OffsetLengthOverflow);
    }
    let opl = opl as usize;
    let mut stats = MinStats {
        offset_plus_length: opl as i64,
        validity_checks: 0,
        validity_skipped: false,
    };
    // Validity: skip when null_count <= 0 and buffer absent/empty
    // (nanoarrow array.c:1632-1636); else need bytes-for-bits coverage.
    if v.null_count <= 0 && v.validity_bytes.map_or(true, |b| b == 0) {
        stats.validity_skipped = true;
    } else {
        stats.validity_checks += 1;
        let need = opl.saturating_add(7) / 8;
        let got = v.validity_bytes.unwrap_or(0);
        if need > got {
            return Err(MinimalError::BufferUndersized {
                which: BufferId::Validity,
                need,
                got,
            });
        }
    }
    // Data buffer: length * elem_size arithmetic (array.c:1637-1640).
    let need = opl.saturating_mul(v.elem_size_bytes);
    if need > v.data_bytes {
        return Err(MinimalError::BufferUndersized {
            which: BufferId::Data,
            need,
            got: v.data_bytes,
        });
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Replay: deterministic cases, no RNG needed; the None column reuses
    // the Misaligned slot law (null base is refused, not defaulted).
    #[test]
    fn valid_three_column_view_passes() {
        let a = [0u64; 4];
        let b = [1u64; 4];
        let c = [2u64; 4];
        let cols = [
            Some(FixedWidthColumn {
                base: a.as_ptr(),
                len: 4,
                bytes: 32,
            }),
            Some(FixedWidthColumn {
                base: b.as_ptr(),
                len: 4,
                bytes: 32,
            }),
            Some(FixedWidthColumn {
                base: c.as_ptr(),
                len: 4,
                bytes: 32,
            }),
        ];
        assert!(validate_fixed_width(&cols).is_ok());
    }

    #[test]
    fn length_overflow_is_typed_error() {
        let a = [0u64; 2];
        let b = [1u64; 4];
        let c = [2u64; 4];
        let cols = [
            Some(FixedWidthColumn {
                base: a.as_ptr(),
                len: 4,
                bytes: 16,
            }),
            Some(FixedWidthColumn {
                base: b.as_ptr(),
                len: 4,
                bytes: 32,
            }),
            Some(FixedWidthColumn {
                base: c.as_ptr(),
                len: 4,
                bytes: 32,
            }),
        ];
        assert_eq!(
            validate_fixed_width(&cols),
            Err(ValidateError::LengthOverflow {
                col: 0,
                len: 4,
                bytes: 16
            })
        );
    }

    #[test]
    fn unaligned_base_is_typed_error_without_dereference() {
        let buf = [0u8; 64];
        let unaligned = unsafe { buf.as_ptr().add(1) as *const u64 };
        let aligned = [1u64; 2];
        let c = [2u64; 2];
        let cols = [
            Some(FixedWidthColumn {
                base: unaligned,
                len: 2,
                bytes: 16,
            }),
            Some(FixedWidthColumn {
                base: aligned.as_ptr(),
                len: 2,
                bytes: 16,
            }),
            Some(FixedWidthColumn {
                base: c.as_ptr(),
                len: 2,
                bytes: 16,
            }),
        ];
        assert_eq!(
            validate_fixed_width(&cols),
            Err(ValidateError::Misaligned { col: 0 })
        );
    }

    #[test]
    fn none_column_is_typed_misalignment() {
        let b = [1u64; 2];
        let c = [2u64; 2];
        let cols = [
            None,
            Some(FixedWidthColumn {
                base: b.as_ptr(),
                len: 2,
                bytes: 16,
            }),
            Some(FixedWidthColumn {
                base: c.as_ptr(),
                len: 2,
                bytes: 16,
            }),
        ];
        assert_eq!(
            validate_fixed_width(&cols),
            Err(ValidateError::Misaligned { col: 0 })
        );
    }

    #[test]
    fn error_display_names_column_and_law() {
        let e = ValidateError::LengthOverflow {
            col: 2,
            len: 3,
            bytes: 8,
        };
        use core::fmt::Write;
        let mut s = alloc::string::String::new();
        let _ = s.write_fmt(format_args!("{}", e));
        assert_eq!(&s, "column 2 overflow: len*8=24 > buffer 8 bytes");
    }
}
