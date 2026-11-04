//! Hand-law courts for the private `generated_arrow_primitives` module
//! (CG-1 gate G1, v26.11.3).
//!
//! LAW: the generated module is never hand-edited; its courts live OUTSIDE
//! it (this file), so a regeneration that drifts any modeled fact flips a
//! court instead of passing silently. PG-6 decision: the module stays
//! PRIVATE (`mod`, not `pub mod`, not deleted).
//!
//! BUFFER_TOO_SHORT unreachability (pinned law): `cgap_import_array`
//! never produces `ForeignError::BufferTooShort` - the C Data Interface
//! carries no buffer byte lengths (the spec's documented gap; see
//! `foreign.rs` module doc), so there is no caller-supplied length to be
//! "too short". The `status::BUFFER_TOO_SHORT` arm exists only because
//! `map_status` is total over `ForeignError`; from the import entrypoints'
//! perspective the arm is dead and must stay unreachable (no court below
//! observes it on any reachable branch).
//!
//! map_status access note: `map_status` is private to the generated
//! module and that file is never edited, so the 6-arm lattice is asserted
//! THROUGH the real entrypoints (`cgap_import_array` /
//! `cgap_import_schema`) - the real call path, not a re-implemented copy.

#![cfg(test)]

use crate::c_abi::{ArrowArray, ArrowSchema, CGAP_FLAG_NULLABLE};
use crate::generated_arrow_primitives::{
    bitmap_bit_test, cgap_import_array, cgap_import_schema, status, ARRAY_KINDS,
};
use crate::generated_layout::LAYOUT_SPECS;
use core::ffi::{c_char, c_void};

// ---------------------------------------------------------------------------
// Fixtures: real ArrowArray / ArrowSchema structs (Chicago: no mocks - these
// are the real C Data Interface ABI types, mutated per branch).
// ---------------------------------------------------------------------------

/// NUL-terminated static format string.
macro_rules! fmt_ptr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const c_char
    };
}

/// A real `ArrowSchema` with format `$f`, everything else zero/null.
fn schema_with_format(f: *const c_char) -> ArrowSchema {
    ArrowSchema {
        format: f,
        name: core::ptr::null(),
        metadata: core::ptr::null(),
        flags: CGAP_FLAG_NULLABLE,
        n_children: 0,
        children: core::ptr::null_mut(),
        dictionary: core::ptr::null_mut(),
        release: None,
        private_data: core::ptr::null_mut(),
    }
}

/// 64-byte-aligned backing store (the Arrow buffer alignment law the
/// import scaffold checks).
#[repr(align(64))]
struct Align64([u64; 8]);

/// Two real 64-byte-aligned static buffers. Statics: the alignment is
/// guaranteed by the type, the lifetime by the program.
static BACKING_A: Align64 = Align64([0; 8]);
static BACKING_B: Align64 = Align64([0; 8]);

/// Slot table over the aligned statics + the MISALIGNED (+8) twin.
struct AlignedBuffers;

impl AlignedBuffers {
    fn slots() -> [*const c_void; 2] {
        [
            unsafe { &raw const BACKING_A }.cast::<u8>() as *const c_void,
            unsafe { &raw const BACKING_B }.cast::<u8>() as *const c_void,
        ]
    }

    /// Slot pointers shifted +8 bytes: 64-aligned base, remainder 0x08 -
    /// the MISALIGNED fixture.
    fn misaligned() -> [*const c_void; 2] {
        let s = Self::slots();
        [
            ((s[0] as usize + 8) as *const c_void),
            ((s[1] as usize + 8) as *const c_void),
        ]
    }
}

/// A real `ArrowArray` over two buffer slots. `slots` must outlive the
/// court's use of the returned struct (caller holds the AlignedBuffers).
unsafe fn array_fixture(
    slots: &[*const c_void; 2],
    length: i64,
    n_buffers: i64,
) -> ArrowArray {
    ArrowArray {
        length,
        null_count: 0,
        offset: 0,
        n_buffers,
        n_children: 0,
        buffers: slots.as_ptr() as *mut *const c_void,
        children: core::ptr::null_mut(),
        dictionary: core::ptr::null_mut(),
        release: None,
        private_data: core::ptr::null_mut(),
    }
}

// ---------------------------------------------------------------------------
// Court A: status-lattice co-domain (pairwise distinct)
// ---------------------------------------------------------------------------

/// The C-ABI status constants are pairwise distinct: a collapsed pair would
/// map two refusal classes onto one wire code and lose a refusal class at
/// the boundary. Per-variant mapping is asserted through the real
/// entrypoints in `import_branch_courts` (map_status is private to the
/// never-edit generated module).
#[test]
fn status_lattice_codes_pairwise_distinct() {
    let codes = [
        status::OK,
        status::INVALID_FORMAT,
        status::FORMAT_MISMATCH,
        status::MISSING_BUFFER,
        status::BUFFER_TOO_SHORT,
        status::MISALIGNED,
        status::NEGATIVE_LENGTH,
    ];
    let mut seen: [i32; 7] = [0; 7];
    for (i, c) in codes.iter().enumerate() {
        for j in 0..i {
            assert_ne!(*c, seen[j], "status code {c} duplicated");
        }
        seen[i] = *c;
    }
}

// ---------------------------------------------------------------------------
// Court B: cgap_import_array per-branch courts (real ABI fixtures)
// ---------------------------------------------------------------------------

#[test]
fn import_branch_courts() {
    // -- INVALID_FORMAT via null array pointer -----------------------------
    let schema = schema_with_format(fmt_ptr!("tu"));
    let rc = unsafe { cgap_import_array(core::ptr::null_mut(), &schema) };
    assert_eq!(rc, status::INVALID_FORMAT, "null array ptr");

    // -- INVALID_FORMAT via null schema pointer ----------------------------
    let slots = AlignedBuffers::slots();
    let mut array = unsafe { array_fixture(&slots, 4, 2) };
    let rc = unsafe { cgap_import_array(&mut array, core::ptr::null()) };
    assert_eq!(rc, status::INVALID_FORMAT, "null schema ptr");

    // -- INVALID_FORMAT via null format string -----------------------------
    let null_fmt = schema_with_format(core::ptr::null());
    let slots = AlignedBuffers::slots();
    let mut array = unsafe { array_fixture(&slots, 4, 2) };
    let rc = unsafe { cgap_import_array(&mut array, &null_fmt) };
    assert_eq!(rc, status::INVALID_FORMAT, "null format string");

    // -- FORMAT_MISMATCH: format "q" is not a modeled array kind -----------
    let qschema = schema_with_format(fmt_ptr!("q"));
    let slots = AlignedBuffers::slots();
    let mut array = unsafe { array_fixture(&slots, 4, 2) };
    let rc = unsafe { cgap_import_array(&mut array, &qschema) };
    assert_eq!(rc, status::FORMAT_MISMATCH, "unmodeled format q");

    // -- NEGATIVE_LENGTH: length -1 with modeled format --------------------
    let tschema = schema_with_format(fmt_ptr!("tu"));
    let slots = AlignedBuffers::slots();
    let mut array = unsafe { array_fixture(&slots, -1, 2) };
    let rc = unsafe { cgap_import_array(&mut array, &tschema) };
    assert_eq!(rc, status::NEGATIVE_LENGTH, "length -1");

    // -- MISSING_BUFFER site 1: n_buffers short of the modeled 2 -----------
    let slots = AlignedBuffers::slots();
    let mut array = unsafe { array_fixture(&slots, 4, 1) };
    let rc = unsafe { cgap_import_array(&mut array, &tschema) };
    assert_eq!(rc, status::MISSING_BUFFER, "n_buffers=1 < modeled 2");

    // -- MISSING_BUFFER site 2: n_buffers counts, but slot 1 is null -------
    let slots_full = AlignedBuffers::slots();
    let slots_null1 = [slots_full[0], core::ptr::null()];
    let mut array = unsafe { array_fixture(&slots_null1, 4, 2) };
    let rc = unsafe { cgap_import_array(&mut array, &tschema) };
    assert_eq!(rc, status::MISSING_BUFFER, "null slot 1");

    // -- MISALIGNED: slots present, addresses +8 (0x08 mod 64) -------------
    let mis = AlignedBuffers::misaligned();
    let mut array = unsafe { array_fixture(&mis, 4, 2) };
    let rc = unsafe { cgap_import_array(&mut array, &tschema) };
    assert_eq!(rc, status::MISALIGNED, "slot addr %% 64 == 8");

    // -- OK + anti-vacuity flip: valid DenseUnionArray fixture -------------
    let slots = AlignedBuffers::slots();
    let mut array = unsafe { array_fixture(&slots, 4, 2) };
    let rc_ok = unsafe { cgap_import_array(&mut array, &tschema) };
    assert_eq!(rc_ok, status::OK, "valid tu fixture imports OK");
    // ANTI-VACUITY MUTATION: reduce n_buffers on the SAME fixture -> the
    // court input family must flip to MISSING_BUFFER. A gate that accepts
    // the reduced fixture is a VACUOUS gate and this assert fails it.
    array.n_buffers = 1;
    let rc_flip = unsafe { cgap_import_array(&mut array, &tschema) };
    assert_eq!(rc_flip, status::MISSING_BUFFER, "n_buffers-reduced twin must flip");
}

/// `cgap_import_schema`: null schema / null format -> INVALID_FORMAT,
/// modeled format -> OK.
#[test]
fn import_schema_branch_courts() {
    let rc = unsafe { cgap_import_schema(core::ptr::null()) };
    assert_eq!(rc, status::INVALID_FORMAT, "null schema ptr");
    let null_fmt = schema_with_format(core::ptr::null());
    let rc = unsafe { cgap_import_schema(&null_fmt) };
    assert_eq!(rc, status::INVALID_FORMAT, "null format string");
    let ok_schema = schema_with_format(fmt_ptr!("U"));
    let rc = unsafe { cgap_import_schema(&ok_schema) };
    assert_eq!(rc, status::OK, "modeled format U");
    let q_schema = schema_with_format(fmt_ptr!("q"));
    let rc = unsafe { cgap_import_schema(&q_schema) };
    assert_eq!(rc, status::OK, "schema import validates UTF-8 only, not kind membership");
}

// ---------------------------------------------------------------------------
// Court C: ARRAY_KINDS <-> LAYOUT_SPECS consistency (3 unambiguous kinds)
// ---------------------------------------------------------------------------

/// The 3 unambiguous ARRAY_KINDS rows have a same-family LAYOUT_SPECS row
/// with matching buffer_count. The `"i"` collision is PINNED, not fixed:
/// both `DictionaryUInt32Utf8` (ARRAY_KINDS, buffer_count 3) and `Int32`
/// (LAYOUT_SPECS, code "i", buffer_count 2) bind format `"i"` with
/// DISAGREEING shapes - an ontology-level ambiguity whose fix is a
/// follow-up ticket, NOT this lane's (PG-6). The court pins the
/// disagreement so neither side can drift silently.
#[test]
fn array_kinds_layout_specs_consistency() {
    let unambiguous: [(&str, &str, &str, usize); 3] = [
        ("tu", "DenseUnionArray", "DenseUnion", 2),
        ("w:32", "FixedSizeBinaryArray32", "FixedSizeBinary", 2),
        ("U", "LargeStringArray", "LargeUtf8", 3),
    ];
    for (fmt, kind_label, layout_label, bc) in unambiguous {
        let kind = ARRAY_KINDS
            .iter()
            .find(|k| k.format == fmt)
            .unwrap_or_else(|| panic!("ARRAY_KINDS missing {fmt}"));
        assert_eq!(kind.label, kind_label);
        assert_eq!(kind.buffer_count, bc);
        let spec = LAYOUT_SPECS
            .iter()
            .find(|s| s.label == layout_label)
            .unwrap_or_else(|| panic!("LAYOUT_SPECS missing {layout_label}"));
        assert_eq!(
            kind.buffer_count, spec.buffer_count,
            "{layout_label}: buffer_count agrees across tables"
        );
    }
    // The "i" collision, pinned as data (see fn doc).
    let i_kind = ARRAY_KINDS
        .iter()
        .find(|k| k.format == "i")
        .expect("ARRAY_KINDS \"i\" row");
    assert_eq!(i_kind.label, "DictionaryUInt32Utf8");
    assert_eq!(i_kind.buffer_count, 3);
    let i_spec = LAYOUT_SPECS
        .iter()
        .find(|s| s.code == Some("i"))
        .expect("LAYOUT_SPECS Int32 row");
    assert_eq!(i_spec.label, "Int32");
    assert_eq!(i_spec.buffer_count, 2);
    assert_ne!(
        i_kind.buffer_count, i_spec.buffer_count,
        "PINNED COLLISION: format i binds two disagreeing shapes (ontology follow-up)"
    );
}

// ---------------------------------------------------------------------------
// Court D: BitTestLaw kill + anti-vacuity table-shape assert
// ---------------------------------------------------------------------------

/// The ontology BitTestLaw (`bitmap_bit_test`): LSB-first packed-bit read.
/// KILL: flipping the law's input byte flips the observed bit - a law whose
/// output does not track its input is a dead law and this court fails.
#[test]
fn bit_test_law_tracks_input_and_kills() {
    let bitmap = [0b0101u8, 0b0000_0010u8];
    // LSB-first: bit 0 set, bit 1 clear, bit 2 set; byte 1 bit 1 set.
    assert!(bitmap_bit_test(&bitmap, 0));
    assert!(!bitmap_bit_test(&bitmap, 1));
    assert!(bitmap_bit_test(&bitmap, 2));
    assert!(bitmap_bit_test(&bitmap, 9));
    assert!(!bitmap_bit_test(&bitmap, 8));
    // Kill: mutate one byte -> the law's output flips with it.
    let mut m = bitmap;
    m[0] ^= 0b0000_0010; // bit 1 was clear -> becomes set
    assert!(bitmap_bit_test(&m, 1), "mutation flips the law output");
    m[0] ^= 0b0000_0010;
    assert!(!bitmap_bit_test(&m, 1), "restore returns the law to baseline");
}

/// Anti-vacuity table-shape assert: the modeled kind table is exactly the
/// shape the branch courts above drive; an ontology row added or removed
/// without the courts noticing is impossible.
#[test]
fn array_kinds_table_shape() {
    assert_eq!(ARRAY_KINDS.len(), 4, "4 modeled kinds (ontology individuals)");
    for k in ARRAY_KINDS {
        assert!(!k.label.is_empty());
        assert!(!k.format.is_empty());
        assert!(k.buffer_count > 0);
        assert_eq!(k.alignment, 64, "modeled alignment is the Arrow 64 law");
        assert!(!k.slot_bit_widths.is_empty());
    }
    // Formats unique within the table.
    for (i, a) in ARRAY_KINDS.iter().enumerate() {
        for b in &ARRAY_KINDS[i + 1..] {
            assert_ne!(a.format, b.format, "duplicate format: {}", a.format);
        }
    }
}
