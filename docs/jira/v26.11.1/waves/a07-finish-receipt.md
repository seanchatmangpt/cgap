# A07 Finish Receipt

Lane: A07 (cgap-bosym slab + zero-string gate finish), repo `/Users/sac/cgap`,
canonical checkout, branch main, 2026-11-01. Lease honored:
`crates/cgap-bosym/src/`, `scripts/zero_string_gate.py`, this file. No git
writes; other lanes' in-flight diffs (bn3/bont/ffi/p28/core/git/boundary_gate)
observed in `git diff --stat` and untouched.

## Files Written

- `crates/cgap-bosym/src/slab.rs` — ingress limiter + 4 ingress courts
  (this session's delta, +282 lines per `git diff --stat`)
- `scripts/zero_string_gate.py` — Pyright fix-forward, GATE_VERSION
  1.4.0 -> 1.4.1 (+30/-14)
- `docs/jira/v26.11.1/waves/a07-finish-receipt.md` — this receipt

## Mission Item 1: bosym ingress limiter (slab.rs)

Constant-work structural/schema admission at the DMA boundary, wired as the
parse ingress:

- `INGRESS_REGISTER_LEN = HEADER_SIZE` (16B fixed register), `MIN_FRAME_SIZE
  = HEADER_SIZE + TRAILER_SIZE` (48B).
- `ingress_register(&[u8]) -> [u8; 16]`: zero-padded stack array, no heap.
- `ingress_limit(have, &reg) -> IngressVerdict`: four structural checks
  (min length, magic, version, v0-forbidden flags) ALL evaluate
  unconditionally; the Admit verdict is an OR-reduction of the check bits
  (no early exit, no data-dependent loop bounds). Drop class is a
  fixed-priority diagnostic picked after the branch-free verdict.
- `From<IngressDropClass> for BosymError`: one refusal taxonomy — the
  limiter maps onto the parser's typed errors (Truncated/BadMagic/
  UnsupportedVersion/UnsupportedFlags), no second list.
- `parse_archive` now runs the limiter FIRST; `Vec::with_capacity` and all
  parser allocations are reached only for Admit payloads (pre-allocation
  drop).

Courts (slab.rs `mod ingress_courts`, real collaborators, no mocks):

- `court_ingress_drops_nonconforming_prealloc` — 4 singly-mutated real
  frames (too-short / bad-magic / bad-version / forbidden-flags) each Drop
  with the expected class; allocation counter UNCHANGED across every drop.
- `court_ingress_admission_is_allocation_free` — conforming control arm
  Admits with zero allocations; parser behind the limiter parses it.
- `court_ingress_wired_into_parse` — every drop class surfaces through
  `parse_archive` as the identical typed error (limiter is the real
  ingress, not a dead API).
- `court_ingress_register_construction` — zero-padding + prefix capture,
  allocation-free.

Defects the courts caught during this session (courts killed, not decorated):

1. Unclosed `mod ingress_courts` brace — compile abort, fixed before any
   test ran (not counted as a test failure).
2. `court_ingress_register_construction` failed first run: the fixture
   `Vec::concat` allocated INSIDE the measurement window. Fixed by moving
   fixture construction before `alloc_delta()` baseline.
3. Process-global `AtomicUsize` counter was polluted by parallel test
   threads (deterministic +1 in a court window). Fixed structurally:
   thread-local per-thread counting in the counting allocator (the hook
   runs on the calling thread), so a court's window is immune to sibling
   test threads.

## Mission Item 2: gate trio re-run first-hand (post Pyright edits)

- `python3 scripts/zero_string_gate.py` — EXIT 0;
  `SUMMARY scanned=71 crates=21 hits=0 emitter_hits=2 adv_hits=0
  unknown=480759 allowed=12231 skipped_artifacts=0 vocab=9
  source=graph-derivation:ontologies/*/src/lib.rs gate=1.4.1`
- `python3 scripts/zero_string_gate.py --self-check` — EXIT 0;
  `SELF-CHECK poison artifacts=1 hits=2 fired=True` (non-vacuous: the
  poison arm fired), `clean artifacts=1 scanned=1 hits=0`,
  `scan-mutation-witness artifacts=2 mutated=none`,
  `SELF-CHECK RESULT PASS`.
- `python3 scripts/zero_string_gate.py --register` — EXIT 0.
- Due diligence (edits touched court code): `--courts` — EXIT 0,
  `COURTS-SUMMARY total=13 failed=0` (T4 poison fired=True; T5 loud
  `[SKIP] .text placement not achievable on this target` with the
  cross-check asserted on `__TEXT,__const`).

## Mission Item 3: Pyright fix-forward

`pyright 1.1.409` (`/opt/homebrew/bin/pyright`): before = 6 errors at
exactly the known lines (237 possibly-unbound `name`; 672 Optional
`regex.search`; 797/1668/1680 Optional `regex.pattern`; 1559 return-type
tuple-vs-dict). Fixes, all behavior-preserving or fail-closed-tightening:

- `bosym_parse.fail` annotated `-> NoReturn` (+ `typing` import) — the
  raise is now part of the type, `name` provably bound at line 237.
- `AllowEntry.matches` raises typed `VocabError` when constructed with
  neither arm (was an untyped AttributeError crash path).
- New `AllowEntry.pattern_or_exact()` used at the register render sites
  and courts T6/T7 — same values, Optional access eliminated.
- `build_emitter_fixture_workspace` return annotation corrected to
  `"tuple[dict, Path]"` (annotation was wrong; caller always unpacked).

After: `pyright scripts/zero_string_gate.py` -> `0 errors, 0 warnings,
0 informations`. Exit contract (0/1/2/3) untouched: no predicate, refusal
class, or exit path changed; `--courts` 13/13 re-proves it.

## Mission Item 4: host cargo test -p cgap-bosym

- `CARGO_TARGET_DIR=/tmp/cgap-lane-A07-target cargo test -p cgap-bosym`
  — EXIT 0, `test result: ok. 14 passed; 0 failed` (10 pre-existing slab
  tests + 4 new ingress courts), doc-tests 0. Never `--workspace`.

## Mission Item 5: sandbox flip (adversarial mutation, /tmp only)

Shim-court cycle over a real copy of the crate at `/tmp/cgap-a07-flip`
(standalone manifest, edition 2021; canonical tree never touched):

- BASELINE: sandbox src sha256
  `df7c4d57311fd7e4b3de71e139747059843e1c4750a2b8f8512d47f5f8bf4bf1`;
  `cargo test court_ingress` EXIT 0, 4 passed.
- SHIM (marked SHIM-COURT in-source): `let flags_ok =
  flags & FLAGS_V0_MASK == 0;` -> `let flags_ok = true;`. Rerun EXIT 101,
  `court_ingress_drops_nonconforming_prealloc` FAILED with
  `limiter ADMITTED forbidden-flags — must drop`. The court flips.
- AFTER/RESTORE: byte-exact restore from the canonical file, sha256
  re-verified equal to BASELINE (fail-closed comparison in the shell);
  rerun EXIT 0, 4 passed.

## Standing

- ALIVE: every claim above carries this session's command + exit.
- `cargo test -p cgap-bosym`: 14/14 with the ingress limiter wired into
  `parse_archive`; gate trio green at gate=1.4.1; Pyright 0/0/0.

## Divergences

1. Lane doc `A07-bosym-slab.md` line 449 says the slab is PROPOSED /
   UNVERIFIED — stale relative to the landed wave-9 state (gate 1.4.x
   executing with receipts); doc update is lane-doc-owner scope, noted
   here not edited (lane doc is authoritative input, not my write target).
2. `lib.rs` is off-limits per contract, so the limiter lives in
   `slab.rs` (module `slab::ingress_*`) rather than a new module file;
   API surface is `cgap_bosym::slab::ingress_limit` etc.
3. "Constant-time" is implemented and claimed as constant-WORK (fixed
   16-byte register, fixed 4-check budget, OR-reduced verdict, no
   data-dependent loops, zero heap). It is not a cryptographic-constant-
   time claim (no secret-dependent branches to hide).
4. Pre-existing known state confirmed on re-check: register mode carries
   the `a07_register_deferral` divergence for boundary_gate.py (A20-owned
   file; untouched by this lane).

## Cleanup

- `/tmp/cgap-lane-A07-target` DELETED at lane end (lease debris law).
- `/tmp/cgap-a07-flip` + `/tmp/cgap-a07-flip-target` DELETED at lane end.
