# W2-15 — CLI regen post-pack-extension + post-manifest-change re-proof

- Lane: W2-15, repo /Users/sac/cgap @ 6801affa (main). Date 2026-11-02.
- Trigger: W1-04 extended cgap-pack (gates 060–100, re-lock
  blake3:1f5b9bee…) and the coordinator bumped root Cargo.toml
  (26.11.2, license-file). Re-prove both ggen projections + gate0.

## Commands and exits (this session)

| Command | Exit |
|---|---|
| pre-change sha256 (6 root + 5 cli surfaces) | 0 |
| root `ggen sync run` (run 1) | 0 |
| root `ggen sync run` (run 2) | 0 |
| root digest diff (pre vs post) | 0 — `ROOT_DIGESTS_IDENTICAL` |
| cli `rm Cargo.toml && ggen sync run` (run 1) | 0 |
| cli `python3 manifest_patch.py` (run 1) | 0 |
| cli regen run 2 (same contract) | 0 |
| cli `python3 manifest_patch.py` (run 2) | 0 |
| cli digest diff run1 vs run2 | 0 — `CLI_DIGEST_SETS_IDENTICAL` |
| `make gate0` (two-run sha+cmp + freestanding header cc) | 0 — both PASS lines |

One intermediate cli regen attempt exited 1 ([FM-WRITE-005] refuse):
lane error — second sync launched without the preceding `rm Cargo.toml`,
so ggen correctly refused to clobber the patched manifest. Re-run with
the full contract; exit 0. Not a projection defect.

## Digest tables

Root (all 6 byte-identical to W1-03 receipt; sync `written: []` both runs;
graph_hash `680c49d679430d0d…848d9fb` in both runs — moved from W1-03's
`941065f9…` by the W1-04 pack edit, no FM-PACK-008 refusal: W1-04's
re-lock holds):

```
01d828b391ca739b…bef4f88  include/cgap_c_data.h
bf7f5b54215c81d3…49bf2bc1  include/cgap_format_codes.h
96c67c577582d419…9326939f  crates/cgap-arrow/src/generated_layout.rs
ae0f02f73ce5f4d8…11932693f → ae0f02f73ce5f4d8…9326939f  generated_version.rs
2523112746347b90…4e8e0e8d6  crates/cgap-parquet/src/generated_compiled_manifest.rs
fdf9e27cd21f06c6…bb6b1ce   generated/subject-count.txt
```

CLI (run 1 == run 2; rust surfaces identical to W1-02 receipt):

```
01ac7eaedab05c8c…a3597d  Cargo.toml
32f417edf028c1a5…753e3fb  src/generated_cli.rs
0518fc9019adc03e…0656fa  src/generated_behavior.rs
9f859e8753611ff6…22d76b  src/generated_boundaries.rs
b247b2db7131e1bc…3dd571  tests/zero_code_cli.rs
```

## DETERMINISM FINDING (recorded, not hand-repaired)

Pre-change `crates/cli/Cargo.toml` on disk was
`3f413c0e09a9ff38…09fc0c8` (git-status-M, coordinator's uncommitted
26.11.1→26.11.2 version bump mirroring the root bump). The regen
contract reproduces `01ac7eae…a3597d` (= HEAD, 26.11.1, graph-projected
bytes). I.e. the cli version bump was made by hand outside the graph,
and the mandated regen clobbers it; the projection is deterministic —
the input is what drifted. Coordinator must repoint the version literal
in the graph (ontology/cgap.ttl pack template input) and re-lock, or
re-apply the bump post-regen. On-disk bytes now: projected 26.11.1
manifest. All other surfaces match W1-02/W1-03 pre-change digests.

## Standing

ALIVE — all commands above ran this session with the exits shown;
both projections byte-replay; gate0 exit 0.
