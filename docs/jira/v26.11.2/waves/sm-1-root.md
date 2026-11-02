# SM-1 root CLAUDE.md rewrite receipt

Lane: SM-1-ROOT (lane-runner). Repo: /Users/sac/cgap, branch main.
Lease honored: CLAUDE.md (root) + readme-archive/CLAUDE-pre.md + this
file. No git writes. No Process.sleep.

## 1. Archive

- Pre-image: CLAUDE.md (sha256
  `736de4409338e1ace5d0c27cd50c7ee31b0dc30742bd6d7a6e6dd7976571b166`)
  archived to `docs/jira/v26.11.2/readme-archive/CLAUDE-pre.md`;
  sha256 of archive byte-identical (cp + `shasum -a 256` both sides,
  exit 0).

## 2. Verification commands (this session)

- `cargo metadata --no-deps --format-version 1` -> 22 members, all
  26.11.1, `cgap-datafusion` absent, `cgap-verification` absent from
  members (exit 0).
- `ls crates/` -> 23 dirs incl. `cgap-verification` (present on disk,
  `publish = false`, standalone `[workspace]` pin in its Cargo.toml,
  root membership pending).
- crates.io census (`curl -A cgap-publish-receipt api/v1/crates/<n>`,
  23 names queried): 18 at 26.11.1 (cgap, arrow, blake3, git, parquet,
  core, storage, bont, bosym, bn3, gate, crypto, p28, ffi, gpu, audit,
  fibo, cloud); ABSENT: cgap-beam, cgap-integration, cgap-court,
  cgap-verification; `bench` resolves to an unrelated crate at 1.1.0
  (name collision).
- `ls docs/jira/v26.11.1/waves/ | wc -l` -> 152 (matches
  `docs/jira/v26.11.2/waves/v2-07-census.md:9`).
- Seed constants witnessed on disk:
  `crates/cgap-parquet/src/bitslice.rs:687-699` (multiplier
  0xBF58_476D_1CE4_E5B9; corpus spread 0xA20C_0FFE_E5A7_E000).
- mtime-bump law witnessed: a02-finish-receipt.md:170-184.

## 3. Section list (new CLAUDE.md)

1. What-this-is (honest counts: 22 members / 18 published at 26.11.1 /
   4 ABSENT + bench name collision; datafusion removed)
2. Agents (unchanged)
3. Commands (unchanged gate law + exit-code contract)
4. Publish law (NEW: one attempt per crate, sequenced post-window,
   stop-on-slide 1h backoff — OVERNIGHT-LOOP.md:128; delist set —
   01-release-gates.md:28-32)
5. CLI (unchanged)
6. Layout (cgap-verification present-not-member; cgap-datafusion
   REMOVED; v26.11.2 doc set; 152 v26.11.1 receipts)
7. Generated artifacts (unchanged)
8. Conventions (NEW: standing vocabulary; THE SEED DISTINCTION;
   mtime-bump restore law; cwd-reset sandbox hazard; frozen pins kept)
9. Agent fan-outs (unchanged)
10. Bash (unchanged, incl. cwd-reset)

## 4. Divergences from dispatch

- Dispatch predicted "curl x22 at 26.11.1": actual census is 18 at
  26.11.1; cgap-beam/cgap-integration/cgap-court/cgap-verification are
  ABSENT from crates.io; `bench` is a name collision (1.1.0, not ours).
- cgap-verification is on disk but NOT a root workspace member (cargo
  metadata 22 does not include it); documented as such.
- Three lines exceed 100 chars in CLAUDE.md: two verbatim code-block
  commands (lines 31, 36) and one table row (line ~95) — all carried
  over verbatim from the archived pre-image; wrapping would change the
  commands; table row carries an explanatory HTML comment.
- Seed literals appear in code, not in the v26.11.2 docs; the doc
  pointer cites bitslice.rs directly.
