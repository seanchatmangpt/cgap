# Lane XC-01 — truth-based FFI export census (EVAL-04 defect 5)

Identity: /Users/sac/cgap @ main, branch main, session 2026-11-02.
Owned files ONLY: `include/cgap.h`, `scripts/ffi_surface_gate.py`.

## Diff

- include/cgap.h: replaced the count-pinned "11 symbols" comment with the
  truth-based law; added `typedef struct CgasStatsBlock` (repr(C) mirror of
  `crates/cgap-ffi/src/stats.rs:53-88` — 120 B, align 8, explicit pads,
  FNV-1a-32 CRC over the first 112 bytes) and the
  `CgapStatus cgap_stats_snapshot(CgasStatsBlock* out)` prototype.
- scripts/ffi_surface_gate.py: default path now BUILDS a fresh release
  cdylib into `/tmp/xc01-target` (`cargo build --release --manifest-path
  crates/cgap-ffi/Cargo.toml`, CARGO_TARGET_DIR=/tmp/xc01-target) and
  derives the expected surface via `nm -gU` from that dylib; `--dylib PATH`
  overrides; `--build` flag retained; self-check now audits the fresh dylib
  (it previously baselined the stale repo-target dylib — which, pre-fix,
  made the baseline read "prototyped but not exported: cgap_stats_snapshot",
  a real flip the stale-baseline bug masked).
- NO hardcoded 11 remains anywhere in the gate or header.

## Commands + exits (this session)

- `python3 scripts/ffi_surface_gate.py --build` → exit 0,
  "PASS: 12 exported cgap_* symbols == 12 header prototypes (1:1)".
- `python3 scripts/ffi_surface_gate.py` (truth-based default) → exit 0, same.
- `python3 scripts/ffi_surface_gate.py --self-check` → exit 0; all six
  mutations flipped as wanted (fake-proto, dropped cgap_version proto,
  hostile shim dylib, corrupted-restore detection, byte-exact restores).
- `nm -gU` census of fresh dylib: 12 T symbols, including
  `_cgap_stats_snapshot` at 0x2a50 — RC-05's export IS in the built dylib.
- `cc -fsyntax-only` on a TU including cgap.h + calling cgap_stats_snapshot
  → HEADER_COMPILE_OK.
- Build lease `/tmp/xc01-target` deleted at lane end (LEASE_CLEANED).

## Standing

ALIVE — every claim above carries a command+exit run this session against
the exact subject. Symbol count witnessed: 12/12, exit 0.

## Fences (must-not-regress, run)

- FIX-07 gate (scripts/ffi_surface_gate.py) — run, exit 0 (above); its
  self-check non-vacuity battery re-run post-edit, exit 0.
- Header ABI freeze static_asserts unaffected (no cgap_frame_view edit);
  cc -fsyntax-only compile fence run, OK.
- Note: the gate is nm-derived, so the repo-target dylib being stale is
  now IMMUNE by construction — the gate can no longer pass on a stale
  artifact unless --dylib is passed explicitly.

## Divergences / disclosures

- Gate script was already 1:1 nm-vs-header before this lane; the
  count-pinning lived in the header comment + the stale-DEFAULT_DYLIB
  baseline. Fixed both, not a rewrite.
- stats.rs was on-disk complete (with `cgap_stats_snapshot` extern) when
  read; no receipt-pending fallback needed. stats.rs itself untouched
  (RC-05's file).
- write-integrity hook flagged paren imbalance from `[0,112)` literals in
  C comments; reworded to "first 112 bytes". Balance now 0/0 vs HEAD 0/0.
