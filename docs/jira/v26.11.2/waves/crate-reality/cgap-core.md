# Lane REAL-cgap-core — reality receipt (2026-11-01)

## Verdict: REAL with one pre-existing failing court (PARTIAL)

cgap-core is real software, not a shell. One test fails and it is a
pre-existing, session-observed defect (matches W4-P3), not a compile abort.

## Commands + exits (this session)

| command (CARGO_TARGET_DIR=/tmp/real-core-target) | result |
|---|---|
| `cargo test -p cgap-core` (run 1, full) | 6 binaries ok + manifest_court FAILED; cargo aborted remaining targets after failure; shell exit masked by pipe (EXIT=0 is last-command rc) |
| `cargo test -p cgap-core --test {rv32imc_court,source_scan_gate,static_assert_court,unit_dma_ring}` | 2+5+4+13 = 24 passed, 0 failed |
| rdflib query over ontology/cgap.ttl | 9 triples; NO `p:CrateManifest` term exists |
| rmtree /tmp/real-core-target | exit 0, removed |

## Test census (63 collected)

| target | passed | failed |
|---|---|---|
| unittests (lib) | 17 | 0 |
| cross_lane_gate_court | 8 | 0 |
| doc_offset_table_court | 2 | 0 |
| guard_flip_court | 11 | 0 |
| identity_court | 4 | 0 |
| ingress_layout_court | 6 | 0 |
| manifest_court | 2 | **1** |
| rv32imc_court | 2 | 0 |
| source_scan_gate | 5 | 0 |
| static_assert_court | 4 | 0 |
| unit_dma_ring | 13 | 0 |

62 passed / 1 failed / 0 compile-abort.

## The failing court (W4-P3 confirmed, fires this session)

`crates/cgap-core/tests/manifest_court.rs:56`
`dod10_no_spdx_license_identifiers_anywhere` — asserts Cargo.toml contains
no SPDX-shaped license reference. Manifest line 7 is
`license-file = "../../LICENSE.md"`, which the court's own spdx_shape
detector flags. The court contradicts the manifest it patrols.

Owner disposition required: either the court learns `license-file` to a
frozen instrument is lawful (and checks for `license = "<SPDX>"` instead),
or the manifest moves the reference out of SPDX shape. Not fixed by this
lane — manifest/court are outside my file lease.

## Drift check (rdflib p:CrateManifest)

`p:CrateManifest` does not exist anywhere in `ontology/cgap.ttl`
(9 triples, only cgap:Project/CTQ terms under https://cgap.chatmangpt.com/).
Drift check is therefore [SKIP] — no-manifest-in-graph, loud, not vacuous.
Divergence: the dispatch assumed a p:CrateManifest term; the graph predates it.

## Substance

- src: 1,123 LOC across 4 files (frame.rs 692, dma_ring.rs 263,
  isomorphism.rs 164, lib.rs 4); 26 pub items (12+10+4).
- tests: 2,343 LOC, 11 court/unit files, 75 `#[test]` annotations
  (63 collected via cargo this session).
- Real courts present: mutation flip at named offsets, guard flip,
  ingress layout, cross-lane gate, rv32imc — all passing, none vacuous.
