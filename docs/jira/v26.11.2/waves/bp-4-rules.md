# BP-4 Rules Receipt

Lane: BP-4 SCOPED-RULES (lane-runner). Repo `/Users/sac/cgap`, branch
main, 2026-11-02.

## Lease honored
Four NEW files under `.claude/rules/` + this receipt. No git writes, no
shared-tree code edits.

## Files written
- `/Users/sac/cgap/.claude/rules/cli-generated.md` (paths:
  `crates/cli/**`) — never-hand-edit law, custom_handlers.rs seam,
  regen command, standalone-workspace note. Receipt cited:
  `waves/cli-2-handlers.md`.
- `/Users/sac/cgap/.claude/rules/kernels-no-std.md` (paths: the four
  kernel crates) — zero-dep/no_std, nm purity (0 alloc/panic), seed
  discipline, sandbox court. Receipt: `waves/r2-xip-noalloc.md`.
- `/Users/sac/cgap/.claude/rules/bench.md` (paths: `crates/bench/**`) —
  Divan `=0.1.21` membrane, black_box + checksum asserts, retained-alloc
  flip channel, mandatory 80/20 PARETO split. Receipt:
  `waves/f2-15-bench-flips.md` (FLIPPED-4/4).
- `/Users/sac/cgap/.claude/rules/publish.md` (paths: `**/Cargo.toml`) —
  one attempt per crate, Retry-After / stop-on-slide, description +
  license-file house style, publish=false until receipted. Receipt:
  `waves/f3-publish-ledger.md`.

## Verification
- On-disk sweep (awk >100-char line scan): zero long lines.
- Frontmatter shape matches the best-practices form (`---` /
  `paths:` / quoted globs).
- All four receipts cited above exist on disk (read this session).

## Standing
ALIVE — files written and swept this session. Rule loading is exercised
by Claude Code on the next session touching a matching path (UNVERIFIED
until then; no court exists for frontmatter parsing itself).
