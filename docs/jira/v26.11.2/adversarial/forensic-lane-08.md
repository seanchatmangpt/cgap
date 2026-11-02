# FA8 — Hostile Forensic Lane 08: `rg_prunes` fail-open court battery

Standing: **CONFIRMED-IN-PART** (adversarial presumption partially upheld, mechanically).
Probe: `/tmp/fa8-probe` (path-dep on the real crate, `CARGO_TARGET_DIR=/tmp/fa8-target`).
Command+exit: `cargo run -q` (exit 0, runs below); `cargo run -q --bin classify` (exit 0).
Shared tree: untouched except this report file.

## Interrogation 1 — malformed/edge-case bounds (verdict per suspicion)

Refs: `crates/cgap-parquet/src/scan.rs:320-325` (`rg_prunes`:
`pred.0 > b.max || pred.1 < b.min`; `None => false`), `:620-647` (`read_statistics`:
min/max must both be present AND exactly 8 bytes, else `None`).

| suspicion | verdict | evidence (this session, exit 0) |
|---|---|
| min>max inverted → silently drops matching rows | **CONFIRMED — WRONG RESULTS, not fail-open** | C9: bounds (min 40, max 10), pred (20,30) → rows=`[]`, pruned=true, control=[(0,1,20),(0,2,30)]. `rg_prunes` returns true for any pred not covering the full inverted interval, so matching rows 20,30 are skipped and the scan returns Ok with an empty result. Docstring at scan.rs:317-319 claims fail-open; that claim is FALSE for present-but-inverted bounds. |
| min==max | safe | C9: (25,25),(20,20),(30,30),(21,29) all match control; prune only when genuinely disjoint. |
| missing one bound / wrong width | fail-open CONFIRMED safe | scan.rs:626/632: `<[u8;8]>::try_from(raw)` failure leaves `None`; scan.rs:640-646: partial → `None`; rg_prunes `None => false`. C9 partial-wire variants match control; C3 no-stats full scan matches control. |
| u64::MIN/MAX boundary bounds | safe | C5/C9 (MIN,MAX) full scan, rows=control. |
| min==max=0 or u64::MAX (disjoint-correct) | correct prune, not a defect | C9: prune is semantically right (bounds genuinely disjoint from pred). |
| float bit-pattern ordering | safe-by-type-restriction | `PhysType::from_id` (scan.rs:255-270) admits only ids 1 (INT32), 2 (INT64), 7 (FLBA); FLOAT(4)/DOUBLE(5) → `UnsupportedSchema(id)`. Writer seam `LevelFreeType` (level_free.rs:145-152) = U32/U64/Bytes32 only. No float-typed column can reach rg_prunes; the bit-pattern Ord at scan.rs:322 is type-consistent with `value_u64`'s bit-pattern accessor, so prune and scan agree. C4: bounds [1, 2^63] with sign-bit-set data never wrongly prunes. |
| flip field ids / swap min/max on wire | becomes inverted-bounds case | C9 inverted row; same defect class as row 1. |

## Interrogation 2 — the float trap analogue

`rg_prunes` compares bit patterns (u64 Ord). For u64 data that is EXACTLY what
`scan_u64_pred` also compares, so prune predicate and value predicate are the same
total order — no divergence is constructible for admitted types. The only way the
prune orders differently than the data semantics is to feed float-ordered bounds
into a u64 column: C4b demonstrated that float-ordered (inverted, bit-pattern-wise)
bounds also fail into the inverted-bounds defect class of row 1, not a new one.

## Interrogation 3 — silent-degrade masquerade

- Missing/partial stats degrade to full scan with NO scan-time status: `scan_u64_pred`
  (scan.rs:793-812) returns `Ok` identically in pruned and degraded runs. The caller
  can only detect degradation structurally, post hoc, via public `meta().row_groups[i].bounds`
  (scan.rs:310) or the `pages_parsed` counter (scan.rs:783) — neither is surfaced by
  a scan-status channel. Verdict: **blind-by-default; degradation is detectable only
  structurally, not reported.**
- C6: degraded (no-stats) scan parses the page (pages_parsed 0→1) exactly like a
  kept group; `rg_prunes`'s `None => false` (scan.rs:323) is indistinguishable from
  an overlap-keep in the scan result.

## Interrogation 4 — offset-class footer corruption

260-mutation byte-flip sweep (0x00/0xFF over every footer byte, single-row-group
u64 fixture, real crate, exit 0):
refused=123 (typed `ScanError` at open/scan — MB8 offset-class holds: no crash,
no panic, exit 0 of the court itself, zero `wrong-via-read-path` from offset fields),
full_scan_ok=53 (degraded-but-correct full scan), PRUNE_CAUSED_WRONG=10
(statistics min/max value-byte flips produce bounds that wrongly prune matching
rows, Ok exit), READ_PATH_WRONG=18 (page-payload byte flips, pos 34-43 = data
corruption — general parquet no-checksum property, not rg_prunes-specific).
One apparent third class (pos=127, bounds unchanged yet pruned) traced to
row-group num_rows corruption → 0-row scan (pages_parsed==0 is an artifact of
`for row in 0..rows` never reaching resolve; not a pruning defect).

## Overall verdict

1. **CONFIRMED DEFECT** — present-but-inverted or otherwise pred-disjoint-corrupt
   bounds wrongly prune matching rows and return `Ok(empty)`. The docstring
   fail-open claim at scan.rs:317-319 ("missing statistics must not change
   results") is true for MISSING stats but the code never validates min<=max;
   a corrupt/attacker-controlled footer silently drops rows. Fix shape: refuse
   (typed error) or fail-open (treat as None) when `bounds.min > bounds.max`.
   Fix must land with a /tmp sandbox mutation court per kernel-crate law.
2. Partial/wrong-width/missing stats: fail-open is real and control-verified.
3. Float trap: safe-by-type-restriction (from_id refuses FLOAT/DOUBLE; writer
   seam is u64/u32/bytes32-only). Recorded as safe-by-construction, not by
   validation.
4. Degradation is detectable structurally (`bounds` public, `pages_parsed`) but
   never reported by the scan itself — caller-blind by default.
5. Offset-class footer corruption refuses typed (MB8 confirmed for this class).

## Anti-vacuity

The battery flips: C9 inverted-bounds row returns `[]` vs control `[(0,1,20),(0,2,30)]`,
and classify's PRUNE_CAUSED_WRONG=10 — the court distinguishes present-corrupt from
missing-bounds outcomes; a fail-open validator would flip it. Zero Process.sleep;
no mocks; every result from the real crate on real files in /tmp.
