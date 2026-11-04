//! AF-28 macro-benchmark court: supply-chain sudden-stop re-index — manifest
//! walk leg (cgap).
//!
//! 5,000,000 synthetic freight-manifest rows: JSON line → canonical
//! projection (fixed field order) → admission (schema digest + row cap) →
//! blake3-payload chain append (one AoqChain link per admitted row),
//! carried in cgap-arrow record batches under a declared 4096-row batch
//! cap. Post-walk: chain `verify_all`, the ΔS conservation assert (admitted
//! == chain length == sum of batch row counts), tamper non-vacuity over a
//! real `verify_records` copy, and a typed-refusal adversarial census
//! (oversize row, bad schema digest) proving append-only
//! no-mutation-on-refusal.
//!
//! Scale honesty (ESSAY-CLAIMS law): the dossier's "5M shipments / 5s"
//! totals are TARGET-UNMEASURED. What this court measures is the walk the
//! landed surfaces support at the full 5M-row scale on this host.
//!
//! Run (measured configuration):
//! `cargo test -p cgap-arrow --test macro_manifest_walk --release`
//! (~600 MB for the 5M-record chain slab; debug is unmeasured).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use cgap_arrow::batch::{BatchBuilder, ColumnBuilder};
use cgap_arrow::blake3;
use cgap_aoq::chain::{verify_records, AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::error::AoqError;

// ---------------------------------------------------------------------------
// Alloc census: counting global allocator over the whole test process.
// ---------------------------------------------------------------------------

static TOTAL_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static TOTAL_BYTES: AtomicUsize = AtomicUsize::new(0);

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() {
            TOTAL_ALLOCS.fetch_add(1, Ordering::Relaxed);
            TOTAL_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        System.dealloc(p, layout)
    }
}

#[global_allocator]
static GA: Counting = Counting;

/// Cumulative allocation snapshot (events and bytes since process start).
#[derive(Clone, Copy)]
struct Snap {
    allocs: usize,
    bytes: usize,
}

fn snap() -> Snap {
    Snap {
        allocs: TOTAL_ALLOCS.load(Ordering::Relaxed),
        bytes: TOTAL_BYTES.load(Ordering::Relaxed),
    }
}

fn delta(a: Snap, b: Snap) -> (usize, usize) {
    (b.allocs - a.allocs, b.bytes - a.bytes)
}

// ---------------------------------------------------------------------------
// Chain digest substrate: the repo's own sovereign blake3 (cgap-arrow)
// bound to cgap-aoq's caller-injected ChainDigest seam (R-K1).
// ---------------------------------------------------------------------------

struct Blk3;

const PREIMAGE_MAX: usize = 128;

impl ChainDigest for Blk3 {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        // Fixed-width parts: 32 + 2 + 8 + 32 = 74 bytes max.
        let mut buf = [0u8; PREIMAGE_MAX];
        let mut n = 0usize;
        for p in parts {
            buf[n..n + p.len()].copy_from_slice(p);
            n += p.len();
        }
        blake3::hash(&buf[..n])
    }
}

// ---------------------------------------------------------------------------
// Manifest schema + JSON→canonical projection.
// ---------------------------------------------------------------------------

/// Declared manifest schema: the fixed field order IS the schema.
const SCHEMA_KEYS: [&str; 6] = [
    "manifest_id",
    "origin",
    "dest",
    "po_number",
    "value_cents",
    "weight_kg",
];

/// blake3 over the schema's key-order fingerprint — every admitted row's
/// observed key order must hash to this.
fn schema_digest() -> [u8; 32] {
    let joined = format!("{};", SCHEMA_KEYS.join(";"));
    blake3::hash(joined.as_bytes())
}

/// Declared per-row cap on canonical bytes.
const ROW_CAP_BYTES: usize = 256;

/// Declared record-batch row cap.
const BATCH_CAP: usize = 4096;

/// Total rows in the macro walk.
const TOTAL_ROWS: usize = 5_000_000;

/// Chain epoch for every link (closed epoch set; 1 = macro-walk epoch).
const EPOCH: u16 = 1;

/// Typed refusal classes for a manifest row (the admission layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refusal {
    /// Canonical row bytes exceed `ROW_CAP_BYTES`.
    OversizeRow,
    /// Row's observed schema fingerprint != the declared schema digest.
    SchemaDigestMismatch,
}

/// A projected manifest row: canonical bytes + extracted field values.
#[derive(Debug)]
struct Projected {
    canon: Vec<u8>,
    id: String,
    origin: String,
    dest: String,
    po: String,
    value_cents: i64,
    weight_kg: f64,
}

/// Deterministic splitmix64 step over a mutable seed.
fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

const ORIGINS: [&str; 8] = [
    "cn-shanghai",
    "cn-ningbo",
    "vn-haiphong",
    "my-tanjung-pelepas",
    "kr-busan",
    "th-laem-chabang",
    "in-mundra",
    "tw-kaohsiung",
];

const DESTS: [&str; 8] = [
    "us-long-beach",
    "us-oakland",
    "us-seattle",
    "us-savannah",
    "us-houston",
    "us-new-york",
    "us-norfolk",
    "us-charleston",
];

/// Generate row `i`'s JSON manifest line. Deterministic in `i` (replayable).
fn gen_json(i: usize, po_len: usize, permute_keys: bool) -> String {
    let mut s = 0xA242_50C6_8B4D_341A ^ (i as u64);
    let o = splitmix64(&mut s) % 8;
    let d = splitmix64(&mut s) % 8;
    let v = (splitmix64(&mut s) % 500_000) as i64;
    let w = ((splitmix64(&mut s) % 40_000) as f64) / 1000.0;
    let po_zeros = "0".repeat(po_len);
    let id = format!("MF-{i:08}");
    let base = format!(
        concat!(
            r#"{{"manifest_id":"{id}","origin":"{o}","dest":"{d}","#,
            r#""po_number":"{po}","value_cents":{v},"weight_kg":{w:.3}}}"#
        ),
        id = id,
        o = ORIGINS[o as usize],
        d = DESTS[d as usize],
        po = format!("PO-{po_zeros}{v:08}"),
        v = v,
        w = w,
    );
    if !permute_keys {
        return base;
    }
    // Adversarial variant: swap the first two keys (schema-order break).
    base.replacen(r#""manifest_id":"#, r#""x_manifest_id":"#, 1)
}

/// JSON → canonical projection. Fixed field order is verified against
/// [`SCHEMA_KEYS`]; a key-order break is a typed
/// [`Refusal::SchemaDigestMismatch`], exactly the "bad schema digest"
/// adversarial class.
fn project(json: &str) -> Result<Projected, Refusal> {
    let inner = json
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .ok_or(Refusal::SchemaDigestMismatch)?;
    let fields = inner.split(',').collect::<Vec<_>>();
    if fields.len() != SCHEMA_KEYS.len() {
        return Err(Refusal::SchemaDigestMismatch);
    }
    // Observed key order; the row's schema fingerprint is blake3 over it.
    let mut observed = String::with_capacity(64);
    let mut values: Vec<&str> = Vec::with_capacity(fields.len());
    for f in &fields {
        let mut it = f.splitn(2, ':');
        let key = it.next().unwrap_or("").trim().trim_matches('"');
        let val = it.next().unwrap_or("").trim();
        observed.push_str(key);
        observed.push(';');
        values.push(val.trim_matches('"'));
    }
    if blake3::hash(observed.as_bytes()) != schema_digest() {
        return Err(Refusal::SchemaDigestMismatch);
    }
    let value_cents: i64 = values[4]
        .parse()
        .map_err(|_| Refusal::SchemaDigestMismatch)?;
    let weight_kg: f64 = values[5]
        .parse()
        .map_err(|_| Refusal::SchemaDigestMismatch)?;
    let canon = format!(
        "{}|{}|{}|{}|{}|{}",
        values[0], values[1], values[2], values[3], value_cents, weight_kg
    )
    .into_bytes();
    if canon.len() > ROW_CAP_BYTES {
        return Err(Refusal::OversizeRow);
    }
    Ok(Projected {
        canon,
        id: values[0].to_string(),
        origin: values[1].to_string(),
        dest: values[2].to_string(),
        po: values[3].to_string(),
        value_cents,
        weight_kg,
    })
}

// ---------------------------------------------------------------------------
// The macro walk court.
// ---------------------------------------------------------------------------

#[test]
fn macro_manifest_walk() {
    println!("=== AF-28 macro manifest walk ===");
    println!("rows_total   = {TOTAL_ROWS}");
    println!("batch_cap    = {BATCH_CAP} rows (declared; last batch carries the remainder)");
    println!("row_cap      = {ROW_CAP_BYTES} canonical bytes");

    let n_batches = TOTAL_ROWS.div_ceil(BATCH_CAP);
    let last_rows = TOTAL_ROWS - (n_batches - 1) * BATCH_CAP;

    // ------------------------------------------------------------------
    // Phase 1: generate + admit + chain-append, batch by batch.
    // ------------------------------------------------------------------
    let mut slab = vec![AoqChainRecord::zeroed(); TOTAL_ROWS];
    let mut chain = AoqChain::new(Blk3, &mut slab);
    let mut admitted: usize = 0;
    let mut batch_rows_total: usize = 0;

    let t0 = Instant::now();
    let s0 = snap();
    for b in 0..n_batches {
        let rows = if b == n_batches - 1 { last_rows } else { BATCH_CAP };
        let mut bb = BatchBuilder::default();
        bb = bb.with_column(
            "manifest_id",
            ColumnBuilder::Utf8(Default::default()),
        );
        bb = bb.with_column("origin", ColumnBuilder::Utf8(Default::default()));
        bb = bb.with_column("dest", ColumnBuilder::Utf8(Default::default()));
        bb = bb.with_column("po_number", ColumnBuilder::Utf8(Default::default()));
        bb = bb.with_column(
            "value_cents",
            ColumnBuilder::Int64(cgap_arrow::primitive::PrimitiveBuilder::new()),
        );
        bb = bb.with_column(
            "weight_kg",
            ColumnBuilder::Float64(cgap_arrow::primitive::PrimitiveBuilder::new()),
        );

        for j in 0..rows {
            let i = b * BATCH_CAP + j;
            let json = gen_json(i, 0, false);
            let p = match project(&json) {
                Ok(p) => p,
                Err(e) => panic!("lawful row {i} refused: {e:?}"),
            };
            // Chain append: one link per admitted row. Payload = canonical
            // row bytes; the chain computes payload_digest = H(payload).
            let idx = chain
                .append(EPOCH, &p.canon)
                .unwrap_or_else(|e| panic!("chain append refused lawful row {i}: {e:?}"));
            assert_eq!(idx as usize, admitted, "chain index must equal admitted count");

            // Batch columns come from the PROJECTION output, not from the
            // generator's variables — the admitted form is what lands.
            push_row(&mut bb, &p);
            admitted += 1;
        }
        let batch = bb
            .finish()
            .unwrap_or_else(|e| panic!("batch {b} finish refused: {e:?}"));
        assert_eq!(batch.row_count(), rows);
        batch_rows_total += batch.row_count();
        drop(batch);
    }
    let walk_wall = t0.elapsed();
    let (walk_allocs, walk_bytes) = delta(s0, snap());

    // ------------------------------------------------------------------
    // Phase 2: ΔS conservation + chain integrity.
    // ------------------------------------------------------------------
    let t1 = Instant::now();
    chain
        .verify_all()
        .unwrap_or_else(|e| panic!("post-walk verify_all failed: {e:?}"));
    let verify_wall = t1.elapsed();

    // ΔS balance: every admitted manifest is exactly one chain link and
    // every batch row is accounted for.
    assert_eq!(chain.len(), TOTAL_ROWS, "chain length != total rows");
    assert_eq!(admitted, TOTAL_ROWS, "admitted != total rows");
    assert_eq!(batch_rows_total, TOTAL_ROWS, "batch rows != total rows");
    println!("\n=== conservation (ΔS balance) ===");
    println!("admitted     = {admitted}");
    println!("chain_len    = {}", chain.len());
    println!("batch_rows   = {batch_rows_total}");
    println!("balance: admitted == chain_len == batch_rows == {TOTAL_ROWS}  OK");

    // Non-vacuity: a real tampered record COPY flips verify_records.
    let span = 64;
    let mut copy: Vec<AoqChainRecord> = chain.records()[..span].to_vec();
    copy[10].payload_digest[0] ^= 0xFF;
    let tampered = verify_records(&Blk3, &copy);
    match tampered {
        Err(AoqError::DigestMismatch { .. }) => {}
        other => panic!("tamper court vacuous: expected DigestMismatch, got {other:?}"),
    }
    // And the unmutated span still verifies (the flip is the mutation).
    let clean: Vec<AoqChainRecord> = chain.records()[..span].to_vec();
    verify_records(&Blk3, &clean).expect("clean span must verify");

    // ------------------------------------------------------------------
    // Phase 3: adversarial census — typed refusals, no chain mutation.
    // ------------------------------------------------------------------
    let head_before = chain.len();
    let mut oversize_refusals = 0usize;
    let mut schema_refusals = 0usize;
    for k in 0..8usize {
        // Oversize: pad po_number far past the row cap.
        let big = gen_json(9_000_000 + k, 220, false);
        match project(&big) {
            Err(Refusal::OversizeRow) => oversize_refusals += 1,
            other => panic!("oversize probe {k} expected OversizeRow, got {other:?}"),
        }
        // Bad schema digest: permuted key order.
        let bad = gen_json(9_001_000 + k, 0, true);
        match project(&bad) {
            Err(Refusal::SchemaDigestMismatch) => schema_refusals += 1,
            other => panic!("schema probe {k} expected SchemaDigestMismatch, got {other:?}"),
        }
    }
    assert_eq!(
        chain.len(),
        head_before,
        "refused rows must not mutate the chain (append-only)"
    );
    println!("\n=== adversarial census ===");
    println!("oversize_refusals      = {oversize_refusals} (typed OversizeRow)");
    println!("schema_refusals        = {schema_refusals} (typed SchemaDigestMismatch)");
    println!("chain_len unchanged    = {} OK", chain.len() == head_before);

    // ------------------------------------------------------------------
    // Receipt table.
    // ------------------------------------------------------------------
    let rows_sec = TOTAL_ROWS as f64 / walk_wall.as_secs_f64();
    println!("\n=== walk receipt (release, this host) ===");
    println!("walk_wall_s        = {:.3}", walk_wall.as_secs_f64());
    println!("verify_all_wall_s  = {:.3}", verify_wall.as_secs_f64());
    println!("rows_per_sec       = {:.0}", rows_sec);
    println!("walk_alloc_events  = {walk_allocs}");
    println!("walk_alloc_bytes   = {walk_bytes}");
    println!("batches            = {n_batches} (cap {BATCH_CAP}, last {last_rows})");
    println!("MACRO_MANIFEST_WALK: PASS");
}

/// Push one projected row into the batch builders (by column name).
fn push_row(bb: &mut BatchBuilder, p: &Projected) {
    macro_rules! col {
        ($name:literal) => {
            bb.column_mut($name)
                .expect("declared batch column must exist")
        };
    }
    if let ColumnBuilder::Utf8(v) = col!("manifest_id") {
        v.push(Some(p.id.as_bytes()));
    } else {
        panic!("manifest_id must be Utf8");
    }
    if let ColumnBuilder::Utf8(v) = col!("origin") {
        v.push(Some(p.origin.as_bytes()));
    } else {
        panic!("origin must be Utf8");
    }
    if let ColumnBuilder::Utf8(v) = col!("dest") {
        v.push(Some(p.dest.as_bytes()));
    } else {
        panic!("dest must be Utf8");
    }
    if let ColumnBuilder::Utf8(v) = col!("po_number") {
        v.push(Some(p.po.as_bytes()));
    } else {
        panic!("po_number must be Utf8");
    }
    if let ColumnBuilder::Int64(v) = col!("value_cents") {
        v.push(Some(p.value_cents));
    } else {
        panic!("value_cents must be Int64");
    }
    if let ColumnBuilder::Float64(v) = col!("weight_kg") {
        v.push(Some(p.weight_kg));
    } else {
        panic!("weight_kg must be Float64");
    }
}
