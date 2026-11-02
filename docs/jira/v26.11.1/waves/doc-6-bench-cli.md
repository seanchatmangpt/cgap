# DOC-6 Receipt: bench + cli READMEs and archive INDEX

Lane DOC-6-bench-cli · 2026-11-01 · repo /Users/sac/cgap (canonical
checkout, branch main) · no git writes.

## Files written (lease)

- `crates/bench/README.md` (NEW)
- `crates/cli/README.md` (NEW — see Divergences)
- `docs/jira/v26.11.1/readme-archive/INDEX.md` (NEW)
- `docs/jira/v26.11.1/waves/doc-6-bench-cli.md` (this receipt)

## Commands + exits (this session)

- `cd crates/cli && cargo test` — exit 0; 19 passed / 0 failed
  (12 lib + 6 mutation_kills + 1 zero_code_cli real-CLI court).
- `./target/debug/cgap self version` → `"26.11.1"`, exit 0.
- `grep -l '[[bench]]' crates/cgap-*/Cargo.toml` — no matches (kernel
  no-contamination sweep, exit 1 from grep = zero hits).
- `shasum -a 256` over all six archived files — exit 0 (INDEX rows).
- Token-sweep of all four written files — on-disk bytes verified, no
  truncation or duplication.

## Numbers cited (all re-read from receipts this session)

- ingress: digest stage 93.38 ms ≈ 85% segment / ~91% of 102.7 ms chain.
- sovereign: commit 164.4/187.6 ms = 87.6%; LEASE_EXHAUSTED at 65376
  lanes, typed 0x43470004.
- bn3: unify 81.2% @ 65,536 → 99.9% @ 1,048,576.
- seal: three-way (45/35/19%), flat 2.37 µs/row tail.
- parquet (F2-12): zero-alloc decode MEASURED (system alloc profiler).
- CLI: 19/0 tests, `self version` 26.11.1 — run this session.

## Courts run

No court owned by this lane; verification is receipt re-read + live
witness runs above. Non-vacuity: every number in both READMEs traces to
a waves/ receipt or a this-session command; no number invented.

## Divergences

1. `crates/cli/README.md` did not exist pre-lane — nothing to archive;
   created fresh. The archive dir held the six files INDEXed above
   (root + four kernel READMEs + cgap-core), landed by other DOC lanes.
2. `chains-parquet` has no landed receipt yet (`waves/` has
   bench-parquet-receipt.md for the isolated F2-12 bench only); the
   bench README marks the chains-parquet row "receipt pending" rather
   than inventing numbers.
3. INDEX `original path` for `cgap-arrow.md` vs `root-README-pre-26.11.1.md`
   both reconstruct to `README.md` — arrow's archive is
   arrow-specific content; provenance per its landing lane's receipt.
