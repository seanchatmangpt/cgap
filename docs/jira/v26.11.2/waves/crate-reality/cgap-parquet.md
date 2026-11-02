# Lane REAL-cgap-parquet Receipt (crate reality wave)

Date: 2026-11-01. Lane: REAL-cgap-parquet. Canonical checkout /Users/sac/cgap; no git writes; no pack edits.

## Verdict: REAL — ALIVE

The largest crate is a real, tested, graph-attributed substrate crate, not a shell.

## 1. Test census (command run this session, exit 0)

`CARGO_TARGET_DIR=/tmp/real-parquet-target cargo test -p cgap-parquet` — exit 0.

| suite | passed | failed | ignored |
|---|---|---|---|
| unittests (src/lib.rs) | 180 | 0 | 1 |
| accessor_perf_court | 0 | 0 | 0 |
| aligned_interop | 4 | 0 | 0 |
| codec_properties | 4 | 0 | 0 |
| level_free_multi_court | 0 | 0 | 0 |
| lever_courts | 3 | 0 | 0 |
| mutation_kills | 17 | 0 | 0 |
| offset_fuzz | 0 | 0 | 0 |
| page_zero_alloc | 2 | 0 | 0 |
| rle_court | 8 | 0 | 0 |
| robustness | 3 | 0 | 0 |
| std_feature_court | 1 | 0 | 0 |
| doc-tests | 0 | 0 | 0 |
| TOTAL | 222 | 0 | 1 |

Total 222 passed / 0 failed across 12 binaries, exit 0. The 1 ignored is
`bitslice::tests::bench_bitslice_perf_and_pareto` — a bench-style perf
court, ignored by design (runs under `--ignored`, not part of the gate).
Compile time dominates (~59s unittest wall); all integration courts are
sub-2s.

## 2. rdflib p:CrateManifest drift-check — PASS

`p:` in packs/cgap-pack/ontology.ttl binds to
`https://seanchatmangpt.github.io/packs/cgap-pack#` (not avec:). Query:
`?m a p:CrateManifest ; p:crateName "cgap-parquet" ; p:crateVersion ?ver ;
p:cratePublish ?pub` -> `("26.11.2", true)`.
On-disk: `crates/cgap-parquet/Cargo.toml` package version is
`version.workspace = true` -> root workspace version "26.11.2";
`publish = true`. Graph row == manifest fact. Exit 0.

Note: a naive query with the avec: namespace returns 0 rows — the
receipt at waves/g1k-01-manifest-facts.md does not name the namespace;
first two probe queries here returned [] on that basis (probes, not a
defect: the facts exist and match).

## 3. Substance counts

- 23 src files, 13,756 LOC src + 3,466 LOC tests = 17,222 LOC total (wc -l).
- 132 `pub fn` in src/; 243 `#[test]` fns (src + tests).
- Largest modules: mmap_writer.rs 1,742; footer.rs 1,612; scan.rs 1,303;
  thrift.rs 921; bitslice.rs 930 (carries the frozen splitmix64 /
  corpus seed pair at bitslice.rs:687-699).
- 11 integration test files, incl. mutation_kills.rs (17 mutation
  courts) and offset_fuzz.rs (883 LOC fuzz corpus).

## 4. Commands and exits

| command | exit |
|---|---|
| CARGO_TARGET_DIR=/tmp/real-parquet-target cargo test -p cgap-parquet | 0 |
| (rerun, full-suite grep) same, PIPE_EXIT=${pipestatus[1]} | 0 |
| rdflib CrateManifest drift-check (python3 heredoc) | 0 |
| wc -l / grep pub fn / grep '#[test]' census | 0 |

Build-root lease deleted at lane end: /tmp/real-parquet-target.

## JSON footer

{"lane":"REAL-cgap-parquet","files_written":["docs/jira/v26.11.2/waves/crate-reality/cgap-parquet.md"],"commands":["cargo test -p cgap-parquet (exit 0, 222/0/1)","rdflib CrateManifest drift-check (exit 0, PASS)","wc/grep census (exit 0)"],"courts_run":["cargo test -p cgap-parquet full suite (incl. mutation_kills 17 courts, std_feature_court, rle_court, lever_courts, aligned_interop, codec_properties, robustness, page_zero_alloc)"],"standing":"ALIVE","divergences":["p: namespace is seanchatmangpt.github.io/packs/cgap-pack#, g1k-01 receipt omits it; naive avec: query returns 0 rows","Cargo.toml version is workspace-inherited; drift-check resolves via root workspace version","accessor_perf_court, level_free_multi_court, offset_fuzz each report 0 passed — they gate via compile/build-script or are conditionally filtered; not failures, but their pass mechanism is UNVERIFIED in this lane"]}
