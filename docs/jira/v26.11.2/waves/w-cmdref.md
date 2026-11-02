# W-CMDREF — CLI docs projection regen + byte-identity + --help court

Lane: W-CMDREF (docs projection regen). Repo `/Users/sac/cgap`. Date 2026-11-02.

## Scope

Files written this session: this receipt only. No git commands. Shared tree
sources untouched.

## Method

1. Baseline sha256 of the three sync outputs
   (`crates/cli/Cargo.toml`, `crates/cli/docs/command-reference.md`,
   `crates/cli/ggen.lock`) -> `/tmp/wcmdref-before.sha`.
2. `cd crates/cli && rm Cargo.toml && ggen sync run &&
   python3 manifest_patch.py` (cycle 1).
3. sha256 -c of baseline: all three OK (exit 0).
4. Second full cycle; `cmp` on both `Cargo.toml` and
   `docs/command-reference.md` against cycle-1 copies: identical.
5. sha256 -c again: all three OK (exit 0).
6. `cargo build` in crates/cli (exit 0), then `cgap --help` and per-noun
   `--help` for all 15 nouns; extracted the full verb surface (36 rows,
   incl. clap built-in `noun help` rows) and diffed against the 21 rows
   parsed from `docs/command-reference.md` headings.

## Results (exits)

| Step | Exit |
|---|---|
| `ggen sync run` cycle 1 | 0 |
| `python3 manifest_patch.py` cycle 1 | 0 |
| sha256 -c baseline (3 files) after cycle 1 | 0 |
| `ggen sync run` cycle 2 | 0 |
| `python3 manifest_patch.py` cycle 2 | 0 |
| `cmp` Cargo.toml / command-reference.md (cycle 1 vs 2) | 0, identical |
| sha256 -c baseline after cycle 2 | 0 |
| `cargo build` (crates/cli) | 0 |
| `cgap --help` + 15 noun helps | 0 |
| surface diff (36 binary rows vs 21 reference rows) | 0 after filtering clap built-in `help` subcommand rows |

## Byte-identity

All three outputs byte-identical across two full regen cycles:
sha256 of `crates/cli/docs/command-reference.md` =
`22f773b6b4fd5893d9e06f1882924c0f9ed42cdca379487ce66dda1663207d82`.
sync1 decisions: `docs/command-reference.md` — "unchanged: content
identical" (projected by `clap-noun-verb-verification-pack`
`command-reference.md.tmpl`; pack pinned at
`crates/cli/ggen.toml:13`).

## --help court

Binary surface (36 rows incl. clap's `noun help` built-ins) vs reference
(21 `## cgap <noun> <verb>` rows): zero drift after excluding the 15
clap-generated `noun help` rows (reference projects admitted commands
only; clap auto-adds `help` subcommands). No drift defect found;
nothing hand-edited.

## gate0

`make gate0` at repo root: **exit 2 — FAIL**, pre-existing, NOT
introduced by this lane. Root cause outside the repo:
`/Users/sac/ggen-marketplace/packs/cgap-pack/templates/receipts-index-tbl.tmpl`
line 2 uses Tera-unsupported slice syntax `row.sha[:12]` → ggen
FM-TPL-017 parse abort during the root `ggen sync run`. Template file
mtime 2026-11-01 23:05 (commit 2664bcaf5 "add force:true to the three
output templates" in ggen-marketplace). The CLI-side sync is unaffected;
only the repo-root sync (and thus gate0) aborts.

## Divergences

- gate0 BLOCKED by a foreign pack template defect in
  `~/ggen-marketplace` (outside this lane's writable set — not fixed
  here, recorded per contract).
- The task named `docs/command-reference.md` ambiguously; real path is
  `crates/cli/docs/command-reference.md`.
- --help court is a surface-set diff, not a per-option court; option
  flags/args are not compared row-by-row.
