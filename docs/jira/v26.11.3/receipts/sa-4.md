# SA-4 — skill-application audit: crates/cli vs run-ggen skill

Lane: SKILL-APPLICATION SA-4, 2026-11-03/04. Skill:
`~/.claude/skills/run-ggen/SKILL.md`. Subject: `/Users/sac/cgap/crates/cli`
(standalone ggen six-pack crate, NOT a root workspace member). Never ran
git. Did not touch `crates/cgap-arrow` generated files.

## 1. unless_exists seam discipline — PASS

`crates/cli/src/custom_handlers.rs` is the `cnv:CustomBehavior` seam.
Pack template
`/Users/sac/ggen-marketplace/packs/clap-noun-verb-crate-pack/templates/custom_handlers.rs.tmpl`
carries `unless_exists: true` + `skip_empty: true` + `sh_after: rustfmt`
frontmatter; the on-disk banner was hand-replaced with accurate
consumer provenance ("GENERATED-ONCE scaffold, hand-filled ... ported
logic of the pre-ggen flat CLI (crates/cli/src/main.rs @ c83619d)") —
truthful to the generated-once, hand-filled flow the skill mandates.

**Witnessed live, twice, on a real run** (not dry-run):

- Scratch replay (`mktemp -d /tmp/sa4-*` copy of crates/cli): after
  `rm src/custom_handlers.rs`, a real `ggen sync run` regenerated the
  scaffold from the pack template (`written`); a second run reported
  `skipped: unless_exists: ... already exists` — one seam, generated
  once, never overwritten.
- Canonical tree: the post-ontology-fix replay reported
  `src/custom_handlers.rs -> skipped: unless_exists: ... already
  exists`, so the hand-filled bodies + the three hand-written submodules
  (`src/custom_handlers/{handlers_kernel,handlers_reasoning,
  handlers_sovereign}.rs`) survived a real sync intact.

One mechanical caution (not a defect): `ggen sync run --dry-run` lists
`src/custom_handlers.rs` as `planned: write (dry-run)` — the unless_exists
gate is applied at write time, not during dry-run planning. Do not read
a dry-run "planned write" on custom_handlers.rs as a pending overwrite.

Dry-run planning quirk observed (unknown cause, not chased further):
dry-run planned writes on Cargo.toml/generated_cli.rs/zero_code_cli.rs
are logged as "planned write" even though a subsequent real run found
those files "unchanged: content identical" (for generated_cli.rs /
zero_code_cli.rs post-regen). Not chased; noted for the next dry-run
interpreter.

## 2. GGM-SRC-002: generated vs ontology+query+template — drift found and fixed

**Finding (real drift, fixed):** fresh scratch regeneration emitted
`version = "26.11.2"` for Cargo.toml while the canonical Cargo.toml said
`26.11.3` — the version line had been hand-bumped on the generated file
without bumping the source graph. That is the exact hand-patch class
GGM-SRC-002 forbids.

- Source of truth: `crates/cli/ontology/cgap-cli.ttl:6`
  `cnv:version "26.11.2"` (stale).
- Fix: bumped to `cnv:version "26.11.3"` and replayed the documented
  regen path (repo CLAUDE.md + `.claude/rules/cli-generated.md`):
  `cd crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py`.
- Witnessed: sync wrote Cargo.toml + generated_cli.rs + zero_code_cli.rs,
  skipped custom_handlers.rs via unless_exists; `manifest_patch.py` printed
  "patched"; `cargo check` exit 0; Cargo.lock cgap-cli entries now 26.11.3.
- Post-fix diffs vs fresh scratch regen: `generated_cli.rs`,
  `zero_code_cli.rs`, `generated_behavior.rs`, `generated_boundaries.rs`,
  `lib.rs`, `main.rs` all byte-identical; Cargo.toml differs only by the
  documented manifest_patch.py seams (13 kernel path deps, license-file,
  `[patch.crates-io]` → ~/clap-noun-verb 26.9.1) plus the now-aligned
  version line. **Zero residual drift.**

Actual regeneration path (witnessed, not assumed): cwd-based
`ggen sync run` in `crates/cli`, packs resolved from
`/Users/sac/ggen-marketplace/packs/*` (absolute paths in ggen.toml),
ggen binary 26.10.11, then `python3 manifest_patch.py` — no `--manifest`
flag, no per-rule selection. A plain (un-patched-state) re-sync on a
patched tree refuses with `[FM-WRITE-005]` "exists
with differing content; refusing silent clobber" (witnessed in scratch:
removing Cargo.toml clears it; removing Cargo.toml clears it; manifest_patch.py replay is idempotent).
Dry-run JSON: graph_hash_hex
`8071f632...` (pre-fix); post-fix scratch replay decisions witnessed
above; `.ggen-v2/receipt.json` + `receipt-log.jsonl` durable on disk.

## 3. Banner conformance — PASS

- `generated_cli.rs` / `generated_behavior.rs` / `generated_boundaries.rs` /
  `lib.rs` / `main.rs` / `tests/zero_code_cli.rs`: all carry GENERATED
  banners, no hand-edits, byte-identical to fresh regeneration.
- `custom_handlers.rs`: hand-replaced banner is accurate (documents
  generated-once + ported-from SHA c83619d provenance).
- `src/custom_handlers/*` submodules: plain hand-written banners, no
  GENERATED claims. No file claims generated provenance it doesn't have;
  no generated file lacks a banner.

## Gates (this session, real output)

- `cargo check` (crates/cli standalone workspace) — exit 0.
- `cargo test` (crates/cli): 12 + 25 + 6 + 1 = 44 passed, 0 failed
  (route proofs + mutation_kills RSS courts per repo CLAUDE.md).
- `cargo test -p cgap-arrow` (root workspace): 228 passed, 0 failed —
  unregressed; cgap-arrow not touched.

## Landing receipt

- Subject: `/Users/sac/cgap/crates/cli` — one file edited
  (`ontology/cgap-cli.ttl:6`, version literal 26.11.2 → 26.11.3);
  regenerated artifacts: `Cargo.toml`, `src/generated_cli.rs`,
  `tests/zero_code_cli.rs` (all written by sync; generated_cli.rs /
  zero_code_cli.rs byte-identical to prior content, Cargo.toml now
  carries version 26.11.3 from the graph), `Cargo.lock` refreshed
  (cgap-cli entries → 26.11.3), `ggen.lock` unchanged, docs outputs
  unchanged, `src/custom_handlers.rs` skipped (unless_exists).
- Actuator: ggen 26.10.11; packs from /Users/sac/ggen-marketplace/packs.
- Commands/exits: ggen sync run (exit 0, written-list above); python3
  manifest_patch.py (exit 0, "patched"); cargo check exit 0; cargo test
  cgap-cli 44/0; cargo test cgap-arrow 228/0.
- Falsifier status: the drift falsifier ("generated file disagrees with
  what its ontology+query+template would currently produce") FIRED on
  Cargo.toml version; fixed at source and replayed. All other generated
  cells: pass (byte-identical).
- Standing: ALIVE (all three audit questions executed against the exact
  subject with real runs; drift fix landed and re-verified).
