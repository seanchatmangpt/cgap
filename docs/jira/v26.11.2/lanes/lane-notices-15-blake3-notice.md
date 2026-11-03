# Lane NOTICES-15 — cgap-blake3 crate NOTICE

## W1 Subject

DOD-09 divergence: no NOTICE file under `crates/cgap-blake3`; license
conveyed only via `license-file = "../../LICENSE.md"` (Cargo.toml:11).

## W2 Diff

- NEW `crates/cgap-blake3/NOTICE.md` (34 lines, 1433 bytes): crate-level
  notice pointing at `docs/NOTICE-blake3.md` via relative path
  `../../docs/NOTICE-blake3.md` (resolution verified on disk, exit 0);
  carries spec-reimplementation disclosure (NOT vendored upstream), the
  FIX-15 upstream-license **UNVERIFIED** marker (no upstream license
  facts invented), the sha256-pin-before-vendoring rule, and the P28-SIL
  license-file pointer.
- `crates/cgap-blake3/Cargo.toml`: NO change needed — license-file
  already present; cargo packaging auto-includes crate-dir files, so the
  NOTICE ships without a manifest edit.

## W3 Commands

- Read `docs/NOTICE-blake3.md`, `crates/cgap-blake3/Cargo.toml`,
  `vendor/nanoarrow/NOTICE.txt` (style precedent: terse attribution).
- `python3` on-disk verification of NOTICE.md bytes + relative-path
  resolution + 100-char line fence (max 72) — exit 0.

## W4 Verification ladder

- Path resolution checked with `os.path.normpath` from the crate dir:
  resolves to the existing FIX-15 doc.
- No build affected: NOTICE.md is a doc-side file; Cargo.toml untouched,
  so no compile surface changed. Fence: `cargo build --workspace`
  unchanged by this diff (no .rs / manifest edits).

## W5 Anti-vacuity

Not a court lane; the anti-vacuity property is the on-disk resolution
check above — the pointer would fail a packaging conformance check if
`docs/NOTICE-blake3.md` moved (relative path, not repo-absolute prose).

## W6 Standing

ALIVE for the file creation (write + exit-0 verification this session).
UNVERIFIED: upstream BLAKE3 license bytes — deliberately carried as
UNVERIFIED per FIX-15; no fabrication.

## W7 Divergences

None. Dispatch matched reality: FIX-15 doc present, no NOTICE existed,
nanoarrow NOTICE style consulted (short-form attribution; ours is longer
because FIX-15 requires the vendoring rule + UNVERIFIED marker inline).
