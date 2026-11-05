# cgap Blake3 Pin Re-Mint Receipt (status_maps + cscb-targets)

Standing: UNVERIFIED

## Subject

FIX lane re-mint of 2 stale blake3 freeze pins (E-s findings), mirroring D62's
arcus re-mint. Repo: /Users/sac/cgap @ 264b9926 (main; HEAD moved 5 commits
during the lane — tree shared with concurrent lanes).

## Digests (old pin -> new pin; disk hash = new pin)

| File | Old pin (stale) | New pin = disk |
|---|---|---|
| crates/cgap-ffi/src/aoq/status_maps.rs | 400f05b6066f5360f2afd85a9896b03fd71600b7c70a9bad137c9dffa63bedb0 | d3e6cb98df7b2e6122c412a1a5c22f600a2e335891c7e5fdaf0187bed65c899f |
| docs/cscb-targets-open.md | 80ae9e682ce9a38a9782a073a22ffe28670719896d209fbb54c4cb7d1a788c6f | 3af920ec5aa1904df6ca20d74bce1526336984b782d96a5795b954ff3e7415bf |

Pin slots re-minted (64-hex, no trailing newline, format matched):
- .ggen/freeze/aoq-pack/crates/cgap-ffi/src/aoq/status_maps.rs.blake3
- .ggen/freeze/cscb-pack/docs/cscb-targets-open.md.blake3

## Lawfulness evidence

- status_maps.rs: disk content = HEAD content, last touched by b6d68517
  (repo-wide `cargo fmt --all`, 2026-11-04) on top of 7eef0938 (v26.11.3
  closure, which minted the file + freeze slots). The drift is the mechanical
  rustfmt convergence, i.e. the v3-era lawful surface. `git log --oneline -3
  -- crates/cgap-ffi/src/aoq/status_maps.rs` -> b6d68517, 7eef0938. Lawful,
  re-mint.
- docs/cscb-targets-open.md: drift = 3 errata lines (2026-11-04, "row 1
  superseded — v4 corrected-claims ledger"), added by eb841d71 "docs+law: AOQ
  status defines (graph law)..." — a documented D14/E-l-class prose edit.
  Lawful, re-mint.

## Gate verdicts (as-run, honest)

- `ggen sync run --skip-ea-admission` x2: both runs `written: []`. No file
  written or regressed by sync.
- Freeze-skips: 2 per run (status_maps.rs + cscb-targets-open.md, "frozen:
  freeze_policy=checksum ... manual edit detected"). **The "zero
  frozen-skips" gate condition is UNSATISFIABLE as specified and the
  expectation was premised on an inverted model of ggen's checksum freeze
  semantics** (read from ggen `crates/ggen-engine/src/write.rs:247-273`):
  pin == disk means "untouched since last generation — SAFE TO REGENERATE",
  i.e. sync OVERWRITES with template output; pin != disk is what PROTECTS a
  hand-completed file (the frozen-skip). During this lane, an intermediate
  sync (pin == disk at that instant) actually REGENERATED both files to
  pre-fmt/pre-errata template content; restored to lawful HEAD content via
  `git checkout --`. Steady state: pin == disk (lawful content, matching the
  arcus mirror: /Users/sac/arcus/.ggen/freeze/arcus-pack/.../adapter.go.blake3
  pin == disk verified) and sync reports the 2 frozen-skips every run — that
  IS the protection working. Zero frozen-skips would require generated ==
  lawful, i.e. carrying the fmt/errata into the marketplace templates
  (ggen-marketplace packs/aoq-pack/templates/status_maps.rs.tmpl — mtime
  Nov 3, predates the Nov 4 fmt — and packs/cscb-pack/templates/
  targets_open.md.tmpl lacks the ERRATA block): another repo, outside this
  lane's file scope.
- status_band tri court: `cargo test -p cgap-arrow --test
  status_band_tri_court` -> 6 passed; 0 failed (incl. anti-vacuity flips).
  Green.

## Re-mint ordering hazard (discovered this lane)

ggen's regressive write path calls `record_freeze_checksum`
(ggen-engine/src/write.rs) and re-mints the slot pin to the content it just
wrote — so a pin re-mint performed BEFORE any further sync is clobbered: the
first sync in this lane overwrote both pins back to the template-content
digests while regressing the files. Lawful landing order is therefore:
(1) settle disk to lawful content, (2) run all sync gate runs, (3) re-mint
pins LAST, (4) commit without another sync. Final landed state verified:
pin == disk for both slots (digests re-read from disk at commit time).

## Tooling gap (same gap D62 named on arcus)

No verifier consumes the .ggen/freeze pins (no pin-vs-disk gate exists in
cgap Makefile/scripts/courts; pins are evidence only). ggen's own freeze
check uses the same slot files but with regenerate-on-match semantics —
so a re-mint without a template fix silently converts a protected hand edit
into a regressive overwrite on the next sync. Candidate next wave: a
pin-check gate that asserts pin == disk for every tracked slot, plus a
template-currency rule (generated must equal the pinned lawful content
before a pin may be re-minted).

## Receipt fields

- Commands/exits: b3sum (exit 0), ggen sync x2 (exit 0, written:[]),
  cargo test tri court (exit 0, 6/6), git operations exit 0.
- Verification ladder: digest re-read from disk at use time; dry-run +
  full-run sync; anti-vacuity legs inside the tri court.
- Changed surface: 2 pin files + this receipt. ggen.lock untouched (sync
  demanded nothing).
