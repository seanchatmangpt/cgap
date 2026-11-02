//! W5 stats-skip court (lane D8, v26.11.2): whole-row-group pruning from
//! fixed-width footer statistics.
//!
//! Chicago discipline: real collaborators — the file under court is written
//! by the crate's own writer path, read back by the crate's own mmap
//! scanner, and final state is asserted (returned rows AND the
//! `pages_parsed` observable). No mocks. Files live in /tmp tempdirs.
//!
//! Anti-vacuity mutation: the SAME writer with the bounds argument removed
//! (empty slice -> no Statistics struct in the footer) must flip this
//! court — the pruned scan then reads every page (`pages_parsed > 0`).
//! A court that cannot flip is FAIL.

#![cfg(feature = "std")]

use cgap_parquet::scan::{
    write_fixture_file_with_bounds, Bounds, FixtureColumn, ParquetFile, PhysType,
};

/// One u64 column: values `lo..lo+rows` (all distinct), bounds from the
/// corpus itself. Returns the columns, the footer bounds, and the values
/// so expectations are derived, never hardcoded.
fn u64_col(lo: u64, rows: usize) -> (Vec<FixtureColumn>, Bounds, Vec<u64>) {
    let vals: Vec<u64> = (0..rows as u64).map(|i| lo + i).collect();
    let bytes: Vec<u8> = vals.iter().flat_map(|v| v.to_le_bytes()).collect();
    (
        vec![FixtureColumn {
            name: "x".into(),
            phys: PhysType::U64,
            bytes,
        }],
        Bounds {
            min: vals[0],
            max: vals[rows - 1],
        },
        vals,
    )
}

#[test]
fn prunes_out_of_bounds_and_scans_in_bounds() {
    let rows = 64usize;
    let (cols, bounds, vals) = u64_col(100, rows);
    let dir = std::env::temp_dir().join(format!("cgap_stats_skip_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmpdir");

    // File A: bounds present. Predicates entirely outside [100,163].
    let fa = dir.join("with_stats.par");
    std::fs::write(
        &fa,
        write_fixture_file_with_bounds(&cols, rows, &[Some(bounds)]),
    )
    .expect("write A");
    let pf = ParquetFile::open(&fa).expect("open A");
    assert_eq!(pf.row_groups().len(), 1);
    assert_eq!(
        pf.row_groups()[0].bounds,
        vec![Some(bounds)],
        "footer bounds round-trip"
    );

    let hits = pf.scan_u64_pred(0, (0, 99)).expect("scan out-of-bounds");
    assert!(hits.is_empty(), "out-of-bounds predicate matches nothing");
    assert_eq!(
        pf.pages_parsed(),
        0,
        "row group pruned whole: no page header parsed"
    );

    // In-bounds predicate scans and returns exactly the corpus rows in the
    // range — derived from the corpus, not a hardcoded count.
    let want: Vec<(usize, usize, u64)> = vals
        .iter()
        .enumerate()
        .filter(|(_, v)| **v >= 150 && **v <= 199)
        .map(|(i, v)| (0, i, *v))
        .collect();
    let hits = pf.scan_u64_pred(0, (150, 199)).expect("scan in-bounds");
    assert_eq!(hits, want, "in-bounds results equal a full scan");
    assert!(pf.pages_parsed() > 0, "in-bounds scan resolves pages");

    // Exact-boundary predicates are inclusive: pred == [min,max] is
    // scanned, not pruned.
    let hits_all = pf.scan_u64_pred(0, (100, 163)).expect("scan full range");
    assert_eq!(hits_all.len(), rows);

    // Disjoint ranges prune inclusively on each edge.
    assert!(
        pf.row_groups()[0].rg_prunes(0, (164, 300)),
        "above max prunes"
    );
    assert!(pf.row_groups()[0].rg_prunes(0, (0, 99)), "below min prunes");
    assert!(
        !pf.row_groups()[0].rg_prunes(0, (99, 100)),
        "touching min scans"
    );
    assert!(
        !pf.row_groups()[0].rg_prunes(0, (163, 200)),
        "touching max scans"
    );
}

#[test]
fn stripped_bounds_mutation_flips_prune() {
    // The mutation: SAME writer, bounds stripped (no Statistics struct in
    // the footer). The prune must NOT fire — pages get parsed, rows are
    // scanned — proving the court is not vacuous.
    let rows = 64usize;
    let (cols, bounds, _vals) = u64_col(100, rows);
    let dir = std::env::temp_dir().join(format!("cgap_stats_skip_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmpdir");

    let fb = dir.join("stripped.par");
    std::fs::write(&fb, write_fixture_file_with_bounds(&cols, rows, &[])).expect("write stripped");
    let pf = ParquetFile::open(&fb).expect("open stripped");
    // One column chunk -> one fail-open None slot (bounds advisory).
    assert_eq!(pf.row_groups()[0].bounds, vec![None]);

    // Same out-of-bounds predicate: without bounds the row group cannot
    // be pruned, so the full scan runs and finds nothing (results are
    // unchanged — only the work differs).
    let hits = pf.scan_u64_pred(0, (0, 99)).expect("scan stripped");
    assert!(hits.is_empty());
    assert!(
        pf.pages_parsed() > 0,
        "MUTATION FLIP FAILED: prune fired without bounds"
    );
    let _ = bounds;
}

/// W5/FA8 repair court: present-but-corrupt bounds must fail open, never
/// prune. An inverted bounds struct (`min > max`) trusted at parse time
/// prunes EVERY predicate range (`pred.1 < min` fires for any pred below
/// min), turning a scan over matching rows into `Ok(empty)` — a silent
/// wrong result. The parse-time validation in `read_statistics` rejects
/// inverted bounds as UNTRUSTED, so they surface as `None` (identical to
/// the stripped-bounds control) and the scan reads the full row set.
#[test]
fn inverted_bounds_fail_open_full_scan() {
    let rows = 64usize;
    let (cols, _real, vals) = u64_col(100, rows);
    let dir = std::env::temp_dir().join(format!("cgap_stats_skip_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmpdir");

    // Inverted bounds, also disjoint from the data [100,163]: written as
    // min=5000, max=10. If a reader trusted these, EVERY predicate would
    // prune (min > max inverted makes the prune condition fire for any
    // range below min) and even an in-bounds predicate would return
    // Ok(empty) — the FA8 silent wrong-result path.
    let inverted = Bounds { min: 5000, max: 10 };
    let fc = dir.join("inverted.par");
    std::fs::write(
        &fc,
        write_fixture_file_with_bounds(&cols, rows, &[Some(inverted)]),
    )
    .expect("write inverted");

    // Control: the same corpus with the Statistics struct stripped —
    // the canonical fail-open shape.
    let fb = dir.join("inverted_control.par");
    std::fs::write(&fb, write_fixture_file_with_bounds(&cols, rows, &[])).expect("write control");

    let pf_bad = ParquetFile::open(&fc).expect("open inverted");
    let pf_ctl = ParquetFile::open(&fb).expect("open control");

    // Parse-time rejection: the inverted bounds surface as None, exactly
    // like absent statistics.
    assert_eq!(
        pf_bad.row_groups()[0].bounds,
        vec![None],
        "inverted bounds must be rejected as UNTRUSTED at parse time"
    );
    assert_eq!(
        pf_ctl.row_groups()[0].bounds,
        vec![None],
        "control carries no bounds"
    );

    // Full row set identical to the no-bounds control: an in-bounds
    // predicate returns every matching corpus row (a trusting reader would
    // have returned Ok(empty) here — the wrong result this court kills).
    let want: Vec<(usize, usize, u64)> = vals
        .iter()
        .enumerate()
        .filter(|(_, v)| **v >= 120 && **v <= 150)
        .map(|(i, v)| (0, i, *v))
        .collect();
    assert!(!want.is_empty(), "corpus must match the predicate");
    let hits_bad = pf_bad.scan_u64_pred(0, (120, 150)).expect("scan inverted");
    let hits_ctl = pf_ctl.scan_u64_pred(0, (120, 150)).expect("scan control");
    assert_eq!(hits_bad, want, "inverted bounds must not change results");
    assert_eq!(hits_bad, hits_ctl, "inverted == stripped control");

    // No pruning happened: pages were parsed for both files (a pruned row
    // group resolves nothing and leaves pages_parsed at 0).
    assert!(
        pf_bad.pages_parsed() > 0,
        "MUTATION FLIP SURFACE: inverted bounds must NOT prune"
    );
    assert_eq!(
        pf_bad.pages_parsed(),
        pf_ctl.pages_parsed(),
        "inverted does the same work as no bounds"
    );

    // rg_prunes itself never fires on the rejected bounds, for any
    // predicate — including ranges entirely below the corrupt min.
    assert!(!pf_bad.row_groups()[0].rg_prunes(0, (0, 99)));
    assert!(!pf_bad.row_groups()[0].rg_prunes(0, (6000, 7000)));
}

/// FA8 second corrupt shape: bounds internally CONSISTENT (min <= max) but
/// disjoint from the data. Parse-time validation cannot detect this shape —
/// there is no data at bounds-parse time to compare against — so an
/// ordered-but-disjoint bounds still prunes. Recorded as a typed OPEN
/// defect in repair-07 (caller-blindness family): this court pins the
/// CURRENT behavior so any future fix must consciously change it.
#[test]
fn ordered_disjoint_bounds_prune_current_behavior_pinned() {
    let rows = 64usize;
    let (cols, _real, _vals) = u64_col(100, rows);
    let dir = std::env::temp_dir().join(format!("cgap_stats_skip_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmpdir");

    let disjoint = Bounds {
        min: 5000,
        max: 6000,
    };
    let fd = dir.join("disjoint.par");
    std::fs::write(
        &fd,
        write_fixture_file_with_bounds(&cols, rows, &[Some(disjoint)]),
    )
    .expect("write disjoint");
    let pf = ParquetFile::open(&fd).expect("open disjoint");

    // Ordered bounds pass validation and are trusted (round-trip).
    assert_eq!(pf.row_groups()[0].bounds, vec![Some(disjoint)]);
    // Pinned current behavior: an in-bounds-for-the-data predicate is
    // pruned by the corrupt-but-ordered bounds -> Ok(empty). OPEN defect,
    // not a pass condition being celebrated: see repair-07 receipt.
    let hits = pf.scan_u64_pred(0, (120, 150)).expect("scan disjoint");
    assert!(
        hits.is_empty(),
        "pinned OPEN defect: ordered-disjoint corrupt bounds still prune"
    );
    assert_eq!(pf.pages_parsed(), 0, "pruned: no pages parsed");

    // Regression: VALID bounds (min <= max, honest from the corpus) still
    // prune after the validation landed.
    let (cols_v, bounds_v, _) = u64_col(100, rows);
    let fv = dir.join("valid.par");
    std::fs::write(
        &fv,
        write_fixture_file_with_bounds(&cols_v, rows, &[Some(bounds_v)]),
    )
    .expect("write valid");
    let pf_v = ParquetFile::open(&fv).expect("open valid");
    assert_eq!(pf_v.row_groups()[0].bounds, vec![Some(bounds_v)]);
    assert!(
        pf_v.row_groups()[0].rg_prunes(0, (0, 99)),
        "valid bounds must still prune (regression guard)"
    );
    let hits = pf_v.scan_u64_pred(0, (0, 99)).expect("scan valid pruned");
    assert!(hits.is_empty());
    assert_eq!(pf_v.pages_parsed(), 0, "valid-bounds prune skips pages");
}
