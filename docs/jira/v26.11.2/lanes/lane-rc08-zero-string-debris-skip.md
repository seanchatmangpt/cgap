# Lane RC-08 — zero-string gate debris-skip repair (v1.4.3)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Sole owned file:
`scripts/zero_string_gate.py`. No git commands run.

## W1 Subject

`/Users/sac/cgap/scripts/zero_string_gate.py` (only file written).
Version bump 1.4.2 -> 1.4.3. `git diff --stat`: 1 file changed.

## W2 Context / defect (reproduced this session)

FIX-07's witness is confirmed and reproduced here BEFORE the fix
(pre-existing, foreign to the fix wave):

- `python3 /Users/sac/cgap/scripts/zero_string_gate.py` -> exit 1,
  `GATE-REFUSED: sections-unreadable /tmp/a07_ar.../lib.rmeta_
  (objdump -h yielded no sections)`.
- Artifact sweep (module-level, this session): 14 rlibs under
  `target/debug/build/{cgap-arrow,cgap-blake3,cgap-parquet,cgap-ffi,
  cgap-audit,cgap-core,cgap-bn3,cgap-gate,cgap-crypto,cgap-p28,
  cgap-branchless}/...` each carry a nightly-rustc metadata-link STUB
  member (`lib.rmeta_` / `lib.rmeta-link`) — object-format magic bytes
  with zero readable sections. One such member hard-refused the ENTIRE
  22-crate scan.

## W3 Fix (explicit two-case law)

1. AR-member debris (`sections-unreadable`/`section-unreadable` raised
   while scanning a member INSIDE an ar archive): loud `[SKIP] ... —
   sectionless metadata-link debris (unreadable rlib member; not a
   zero-string violation)` on stderr, member skipped, scan continues;
   a DEBRIS-ONLY rlib sets `skip` truthy so the run can reach the
   existing exit-3 loud-skip path. Applied in both `scan_artifact` and
   `dep_dir_scan_artifact`.
2. Unreadable PRIMARY subject (the discovered artifact itself, outside
   any ar member): typed refusal preserved, `sections-unreadable`/
   `section-unreadable` added to SUBJECT_MISSING_CLASSES -> exit 2
   (subject-missing), witnessed end-to-end exit 2 (fixture this
   session).

Docstring carries the DEBRIS-SKIP REPAIR (v1.4.3) block stating both
cases; exit-ABI comment updated.

## W4 Commands + exits (all this session)

| cmd | exit | outcome |
|---|---|---|
| `python3 scripts/zero_string_gate.py` (BEFORE fix) | 1 | GATE-REFUSED sections-unreadable lib.rmeta_ (reproduction) |
| module-level sweep of all discovered artifacts | 0 | 14 sections-unreadable refusals enumerated (debris census) |
| `python3 scripts/zero_string_gate.py` (AFTER fix) | 0 | SUMMARY scanned=109 crates=22 hits=0 skipped_artifacts=0 gate=1.4.3; 907 loud debris [SKIP] lines on stderr |
| `--self-check` | 0 | poison fires / clean silent + DEBRIS leg A exit=3 LOUD SKIP; leg B crate-no-artifacts baseline |
| `--courts` | 0 | COURTS-SUMMARY total=14 failed=0 (T17_debris_skip PASS added) |
| mutant `/tmp/rc08_mutant_copy.py` (debris try/except -> `if False`, 2 sites) | legs flip | mutant leg A: cmd_scan raised sections-unreadable (rc=-1, NOT loud skip); leg B unchanged — T17/debris legs FLIP, non-vacuous |
| primary-subject fixture (non-ar Mach-O stub as liba07debris.rlib) | 2 | GATE-REFUSED sections-unreadable -> exit 2 subject-missing (exit-2 law witnessed) |
| `CARGO_TARGET_DIR=/tmp/rc08-target cargo test -p cgap-ffi wave9_a07_zero_string_gate_over_all_built_crates` | see below | wave9_a07 invocation |

## W5 wave9_a07 result

- Gate leg 1 (scan): `python3 /Users/sac/cgap/scripts/zero_string_gate.py`
  -> exit 0, SUMMARY scanned=109 crates=22 hits=0 (the test asserts
  exactly this + floors 10/20 — satisfied).
- Gate leg 2 (anti-vacuity): `--self-check` -> exit 0.
- The Rust harness `wave9_a07_zero_string_gate_over_all_built_crates`
  itself is compile-BLOCKED by a FOREIGN in-flight file:
  `CARGO_TARGET_DIR=/tmp/rc08-target cargo test -p cgap-ffi --lib
  wave9_a07_zero_string_gate_over_all_built_crates` -> exit 101,
  error[E0277] `&Cell64: CoerceUnsized<&dyn Debug>` in
  crates/cgap-ffi/src/sovereign.rs (FIX-07-owned, modified in git
  status this session; not this lane's file). Compile-abort is not a
  failure count; both legs the test drives ran green standalone above.
  Re-run of the harness is left to the owner lane / integration once
  sovereign.rs compiles.

## W6 Fences (must-not-regress) — run this session

- `--self-check` poison/clean anti-vacuity: exit 0 (W4 row 4).
- `--courts` T4-T17 suite: 14/14 PASS exit 0 (W4 row 5).
- wave9_a07 `parse_gate_summary` floors (scanned>=10, crates>=20,
  hits=0): satisfied by the fixed scan (109/22/0).
- Exit-ABI stability: 0/1/2/3 meanings unchanged; only the
  sections-unreadable family re-mapped (1 -> 2 primary / skip-member),
  disclosed here.

## W7 Divergences / notes

- write-integrity hook reports STRAY_TOKENS on EVERY edit to this file:
  it flags the intentional U+FFFD literal in the T15 court string
  (present in HEAD too — pre-existing, not introduced by this lane;
  byte-verified this session). The hook's balance check (the real
  integrity assertion) is quiet.
- New behavior coupling: a debris-only rlib now sets `skip` truthy;
  mixed-debris rlibs (real objects + debris members) keep legacy
  scanned semantics, so the shared-target scan counts are unchanged
  except through the previously-refusing 14 rlibs (now scanned).
- Anti-vacuity placement note: the debris self-check leg scopes
  EMITTER_BINARIES to empty for its fixture (restored try/finally);
  the no-dead-entries law itself remains pinned by T16 arm 5 leg B,
  which ran PASS in the same session.
