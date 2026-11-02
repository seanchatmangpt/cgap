# repair-10: include/cgap.h status-enum parity (lane F8)

Defect (W14/W9, from w6-fa9-commit-gating.md:23 "got.len() == 6 unchanged"):
`CGAP_ERR_CONCURRENT_WRITER_DENIED = 0x4347_0006` existed Rust-side only
(sovereign.rs:87); C consumers of `include/cgap.h` could not match it by
name.

## Diff

- `include/cgap.h`: added `CGAP_ERR_CONCURRENT_WRITER_DENIED = 0x43470006u`
  to the status enum, matching the existing entry style (comment-stripped
  parser shape unchanged; comment cites repair-10).
- `crates/cgap-ffi/src/sovereign.rs`:
  - `MIRROR_ENUM` gains the name/value pair (parity map now 7 entries).
  - `status_court_register_hygiene_and_header_value_parity` pin:
    `got.len() == 6` -> `== 7` with repair-10 in the message.
  - Mutation corpus grows M13 (drop `CGAP_ERR_CONCURRENT_WRITER_DENIED =`,
    named leg "status enum drift"); corpus now 13 mutations.

## Anti-vacuity witnesses (this session)

1. Flip 1 (real, live): first run after strengthening MIRROR_ENUM but with
   the `== 6` pin in place — status court panicked
   `left: 7, right: 6, header enum pins exactly the six register values`
   (sovereign.rs:1828), test exit nonzero. The pin was load-bearing.
2. Flip 2 (in-court mutation): `header_court_mutation_corpus_all_detected`
   with M13 —
   `DETECTED  M13 enum member drop (CONCURRENT_WRITER_DENIED) -> status enum drift (isolated flip)`;
   corpus: 13 mutations, every anchor unique, every flip on its named leg.
3. Post-fix green: `cargo test -p cgap-ffi` (full, exit 0) — lib 50 passed;
   integration 4+3+2+5+0.

## Commands (exits this session)

- `CARGO_TARGET_DIR=/tmp/f8-target cargo test -p cgap-ffi --lib` — first run:
  48 passed / 2 failed (pin flip 1 above + wave9_a20 stress court; the
  wave9 failure did not reproduce on rerun and is unrelated to the header:
  concurrent-wave resource contention, UNVERIFIED root cause, not retried
  to green-chase).
- rerun: 50 passed, 0 failed (exit 0).
- `cargo test -p cgap-ffi` (full suite): all targets green, exit 0.
- `cargo test -p cgap-ffi --lib header_court_mutation -- --nocapture` —
  M13 DETECTED line, exit 0.
- `grep -c CGAP_ERR_CONCURRENT_WRITER_DENIED include/cgap.h` -> 2
  (definition + Rust-twin comment).

## Standing

ALIVE for the parity fix (command+exit above, this session). The wave9_a20
stress-court non-reproducing failure is recorded UNVERIFIED, not rounded.
Build root lease `/tmp/f8-target` deleted at lane end.

## Divergences

- Dispatch said "the ffi test asserting got.len()==6": the pin lives at
  sovereign.rs:1828 inside `status_court_register_hygiene_and_header_value_parity`
  (in `src/`, not tests/). No file under crates/cgap-ffi/tests/ pins the
  count; w6-fa9 receipt's "unchanged" note refers to this src pin.
- Dispatch's "temporarily revert to witness the flip" was replaced by the
  in-court mutation (M13) + the live pin-flip already witnessed in witness 1 —
  no shared-tree revert performed.
