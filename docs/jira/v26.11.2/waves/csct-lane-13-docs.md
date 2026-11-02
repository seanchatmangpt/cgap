# CSCT Lane 13 — Docs Receipt (RFC-0042)

Date: 2026-11-02 · Lane: CSCT-L13 · Repo: /Users/sac/cgap (canonical, no git writes)

## Files written

1. `docs/csct/README.md` — crate-level index (what CSCT is, crate map, run
   commands, exit-code table 0/1/2/3/101, dogfood law: no protobuf/JSON/serde,
   Rust structs + deterministic markdown + BLAKE3 receipts).
2. `docs/csct/csct-spec.md` — condensed RFC-0042 spec (6 stages with metric
   contracts, 4 Chicago axioms, state-transition flow).
3. This receipt.

## Commands + exits

| command | exit | result |
|---|---|---|
| `awk 'length($0) > 100 {print FILENAME":"FNR": "length($0)}' docs/csct/README.md docs/csct/csct-spec.md; echo exit=$?` | 0 | 0 over-length lines (verified: pipe to `wc -l` = 0) |

Grounding reads (no writes): `ls crates/csct` exit 0; scaffold receipt
`csct-lane-01-scaffold.md` read in full; `grep` of `types.rs`/`receipt.rs`
exit 0 confirming ExitCode variants (0/1/2/3/101) and BLAKE3 markdown
receipt rendering.

## Standing

ALIVE for the docs artifact itself (files on disk, line-length check exit 0
this session). UNVERIFIED for everything downstream: `cargo test -p csct` and
the `csct_runner` invocation are documented, NOT RUN this lane — the crate's
stage bodies are other lanes' in-flight files and the runner bin is absent.

## Divergences (observed on disk, recorded not fixed)

1. `crates/csct/src/lib.rs` declares `pub mod court;` but no `src/court.rs`
   exists in the tree — the crate cannot compile until the court lane lands.
2. `bin/csct_runner` is absent from the tree at writing time (dispatched to
   a lane in flight); the README run command names it as-contracted, with
   the absence stated rather than hidden.
3. Prior-receipt errata re-affirmed (not re-derived): lane-01's 351.15x vs
   352x ROI arithmetic flag stands (`csct-lane-01-scaffold.md:35-44`);
   `REQUIRED_ROI_X = 352.0` is verbatim in `lib.rs`. Coordinating decision
   (a)/(b) still owed before the verdict gate can pass non-vacuously.

## Fences

No shared file touched (root Cargo.toml, lib.rs, gates, courts all read-only
this lane). markdown law applied: H1 matches filename on both docs, all
lines ≤100 chars (witnessed above), kebab-case paths, See Also section in
the spec. Must-not-regress surfaces: none — docs-only diff.
