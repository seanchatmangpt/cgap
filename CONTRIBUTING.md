# CONTRIBUTING.md

Contribution law for cgap (v26.11.2). Source of truth is the repo
CLAUDE.md + the campaign docs; this file encodes the non-negotiables
with their on-disk authority.

## Chicago court law

- Real collaborators, assert final state, no mocks (mocking banned
  by default; a double only where genuinely infeasible, stated,
  simple fake not a mock).
- Every court names its anti-vacuity mutation; a court that cannot
  flip is FAIL.
- A mutation-free green gate run is VACUOUS (exit nonzero).
  Gate/court-law witness: waves/overnight-readiness.md
  (docs/jira/v26.11.2).
- Courts mutate /tmp sandbox copies, never the shared tree.
- Restore is sha256 fail-closed (mtime-bump restore law,
  `docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184`).
- Byte-exact restore does NOT trigger rebuild — bump mtimes or
  `cargo clean -p <crate>` before rebuild.
- Skips are loud (`[SKIP] <reason>`), never silent.
- No Process.sleep: bounded polls on real observables with stated
  deadlines; the runtime owns time (production sleeps only as
  SCHEDULING with an adjacent `# SLEEP-CLASS: SCHEDULING`
  annotation).
- Every claim cites its receipt; receipt-cited counts are re-read
  from disk at use time, never from session memory.

## Zero-alloc ingress gate law

- Positive-control-first: prove the gate refuses before trusting it
  to pass. CountingAlloc sentry witness:
  `crates/cgap-git/tests/quota_alloc_purity_court.rs` (window-gated
  alloc/byte ledger, asserts allocs==0 AND bytes==0).
- Gate exit codes 0/1/2/3: pass / gate fail / subject missing /
  loud skip. Compile-abort is not a failure count (census tooling
  must report the broken file, W672 finding).
- Perf numbers are TARGETs until measured with a stated method;
  quiet-host scope qualifiers on every bench figure.

## Doc law

- ERRATA, never rewrite: never silently edit a landed receipt; a
  defect in a prior receipt is corrected by a new errata note citing
  the original file+line (errata witness: waves/w-readme2.md,
  docs/jira/v26.11.2).
- Git: commit with `-F <file>` (never inline `-m`), no rebase, no
  force-push, never `git add -A` — stage explicit paths.
- Markdown discipline: <=100 char lines, kebab-case, language-tagged
  code, H1 matches filename.
- Two seeds never conflate, and are untouchable: SplitMix64
  multiplier `0xBF58_476D_1CE4_E5B9` and corpus seed
  "0xA20C_0FFE_E5A7_E000" — both pinned in
  `crates/cgap-parquet/src/bitslice.rs:687-699`; swapping or
  "fixing" them breaks byte-identical replay.
- Frozen byte-pins never swept: license URN
  `urn:avec:license:p28-sil:v2.1.0-sovereign`; v26.10.31 SHACL IRIs.
