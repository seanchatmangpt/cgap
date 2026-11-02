# A09 Finish Receipt — lane finisher pass, v26.11.1

- **Lane**: A09 — FIBO debt/bond/AR O-ABI pack (`ontologies/fibo`,
  `fibo_sec_dbt_v1.oabi`).
- **Subject**: `/Users/sac/cgap` @ `main` @ `cfd085a9` (wave content
  `62215326` ancestor), canonical checkout, no worktree/clone; mutations
  only in `/tmp` sandboxes with sha256-verified restores.
- **Pass type**: lane finisher — assess the landed wave, then
  independently re-witness the mission's named courts this session.
- **Session**: 2026-11-01.
- **Files written this pass**: this receipt only. No source, test, or
  manifest change under `ontologies/fibo/`; `git status --short --
  ontologies/` returned empty at pass end, and the committed pack's
  sha256 is byte-stable across the whole pass (below).

## 1. Assessed state

`62215326` landed waves 1-9 of the lane: `src/lib.rs` facade (no_std,
zero-dep, const identities + compile asserts), `src/bin/emit-pack.rs`
deterministic fail-closed emitter (bin target `emit-fibo-pack`), the
committed 496-byte pack, and court files wave1/wave4/wave8/wave9
(21 tests). A committed adversarial audit
(`a09-wave9-backfill-audit-receipt.md`) already stands at ALIVE; this
pass re-witnessed rather than trusted it.

## 2. Commands this session (canonical tree, cwd `/Users/sac/cgap`)

Commands verbatim (cwd `/Users/sac/cgap`), indexed C1-C7:

```text
C1: CARGO_TARGET_DIR=$PWD/target-a09 cargo build -p cgap-fibo --bins
C2: CARGO_TARGET_DIR=$PWD/target-a09 cargo test -p cgap-fibo --no-fail-fast
C3: target-a09/debug/emit-fibo-pack --check
C4: target-a09/debug/emit-fibo-pack
C5: RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=$PWD/target-a09 cargo check -p cgap-fibo --all-targets
C6: python3 scripts/zero_string_gate.py
C7: shasum -a 256 ontologies/fibo/fibo_sec_dbt_v1.oabi
```

| # | Exit | Result |
|---|---|---|
| C1 | 0 | lib + `emit-fibo-pack` built |
| C2 | 0 | 21/21 pass (8 binaries: 0+0+1+6+9+1+4) |
| C3 | 0 | `OK 496 bytes, digest d7495fba8022ee19…` == `PACK_DIGEST` |
| C4 | 0 | `OK 496 bytes (unchanged)` |
| C5 | 0 | clean |
| C6 | 0 | scanned=70 crates=21 hits=0 emitter_hits=2 vocab=9 gate=1.4.1 |
| C7 | 0 | sha256 `31f75454…e61c18` (full value in §4) |

## 3. Consumer court (mission item: consumer test over the .oabi)

- Spec-declared consumer (A09 §1/§6): A06's `.oabi` reader
  `crates/cgap-bont/src/oabi.rs` (`OabiPackV1` / `OabiRecordV1`, 64 B
  header + 48 B records). The spec declares no single verification
  command; per dispatch fallback the consumer crate's tests are cited:
  `tests/wave4_courts.rs::t10_loader_court` drives the real reader
  (C2's suite), and the lane's verification command is C2 (DoD 12).
- Rule evaluated (A06 §5 / A09 §6 producer obligation):
  `SlotId == truncate64(BLAKE3(b"bont\x01" ++ lexeme))` re-derived per
  record by the reader (`verify_slot_derivation`), plus header checks
  (magic/header_len 64/flags 0b11/class_count 0x22/enum_count 0x34B/
  record_size 48/reserved zero) and the frozen ClassId set.
- Expected slot verdict (t9 golden join, wave4_courts.rs:113-139):
  record-for-record in harvest order — e.g. `fibo:SecDbt:CorporateBond`
  -> `CORP_BOND_SLOT` `0x0021B3425CD01316`, ClassId `0x00000001` — each
  re-hashed live through the real `cgap_blake3::hash`.

## 4. Byte-stable re-derivation (mission item: compile per spec -> identical bytes)

- Canonical: C3 re-derives the full pack from the frozen corpus at every
  `--check` and compares byte-for-byte with the committed file — exit 0.
- Independent cross-tree witness: a flat 4-crate `/tmp` sandbox
  (`/tmp/a09-finish-sb`: cgap-blake3, cgap-bont, cgap-core, fibo; own
  root manifest; disclosed deltas below) with the committed pack REMOVED
  and the emitter rebuilt from the copied sources emitted a fresh pack
  via the crash-consistent temp+sync+rename path: exit 0, 496 bytes,
  sha256 `31f754549f562cc9…` byte-identical to canonical, no `*.w9tmp`
  orphan left. Byte-stability is therefore witnessed across a rebuild,
  not just a re-run of one binary.

## 5. Flip court (mission item: corrupt one compiled rule entry in a /tmp copy)

Baseline: sandbox suite green, exit 0 (20/20 over wave1_derivation,
wave1_witness, wave4_courts, wave9_integration_courts — the wave-8
citation court is canonical-path-pinned and cannot run in a flat
sandbox; recorded limitation of the sandbox layout, not a court defect).

- **M-FLIP** — mutation: byte @208 (record 3 `fibo:SecDbt:CorporateBond`
  SlotId low byte) `0x16`->`0x17` — one compiled rule entry, corrupted
  on-disk in the sandbox copy.
- Flip witnessed: `t7_pack_digest_court` + `t10_loader_court` both
  FAILED, exit 101 — t7: BLAKE3 of the mutated bytes != `PACK_DIGEST`;
  t10: the reader refused the corrupted record (panic at
  wave4_courts.rs:105). Producer half on the same mutation:
  `emit-fibo-pack --check` exit 4, typed
  `PackDrift … first differing byte @208`.
- Restore: canonical pack copied over the sandbox pack, sha256
  `31f754549f562cc9…` byte-exact; post-restore re-prove 20/20, exit 0.

Disclosed process defect: the first flip invocation passed two
positional test-name filters; cargo rejected the arguments (exit 1) —
an invocation error, not a flip; caught by reading the run log before
claiming, and re-run with the `--` filter form to the witnessed flip
above. Canonical tree untouched throughout (C7 sha256 stable).

Sandbox disclosed deltas (auditor/lane-authored, coordinator role):
sandbox root `Cargo.toml` (workspace + `[workspace.package]`), fibo
manifest path rewiring `../../crates/*` -> `../*`, cgap-core's
cgap-gate dev-dep elided (gate crate not in closure), `LICENSE.md`
placed at `/tmp/LICENSE.md` for the crates' `license-file = "../../LICENSE.md"`
references.

## 6. Standing

**ALIVE** for the landed wave 1-9 slice, re-witnessed this session:
compile per spec -> byte-stable re-derivation (C3 + cross-tree fresh
emit), consumer court over the spec-declared reader with expected slot
verdicts (t10/t9 inside C2), the mission-mandated single-entry
corruption flip with byte-exact restore (M-FLIP), warnings-deny check
(C5), and the zero-string gate (C6) — every claim above carries its
command + exit from this session. Residual residue carried honestly:
DoD 3's pinned-distribution SHA-256 stays
`UNVERIFIED:no-distribution-on-disk`, recorded in place at
`ontologies/fibo/src/lib.rs:158` — the FBC-2024Q4 distribution zip is
absent on this host; admission of that artifact is coordinator scope.

## 7. Divergences and notes

1. Lane doc §7 sketches 48 B facade records; the shipped facade asserts
   56 B (repr(C) field list cannot reach 48 B padded; the 48 B figure is
   A06's `OabiRecordV1` container record, a different type). Doc yields;
   recorded in `src/lib.rs:371-379` since wave 5. Carried, unchanged.
2. `a06_citation_freshness_court` (wave8) reads canonical-tree paths and
   therefore fails inside the flat `/tmp` sandbox (exit 101 at sandbox
   baseline); excluded from the sandbox baseline scope as a disclosed
   layout limitation — it runs green in the canonical suite (C2).
3. Zero-string gate counts drift session-to-session with concurrent
   lanes (audit: scanned=66 crates=22; this pass: scanned=70 crates=21,
   hits=0 both) — the census is dynamic; A09's obligation (hits=0 for
   the facade/pack surface) held under the current tree.
4. Concurrent sibling-lane dirt was present in the shared tree
   (`crates/cgap-bont/src/primitives.rs`, `crates/cgap-core/src/frame.rs`,
   `scripts/zero_string_gate.py`, etc.); none under `ontologies/`. C2 ran
   green against the live sibling state, which strengthens the cross-lane
   claim; none of the dirt is attributable to A09.
5. Receipt naming: this pass writes `a09-finish-receipt.md` per its
   dispatch; earlier lane receipts used in-module docs and
   `a09-wave9-backfill-audit-receipt.md` (committed, untouched here).
6. The dispatch's consumer-crate hint ("likely cgap-bont or cgap-gate")
   resolves to cgap-bont: A09 §1/§6 declare the A06 `oabi.rs` reader as
   the pack consumer; cgap-gate has no declared role on this pack (the
   A14 dam is typed UNSUPPORTED for this lane, src/lib.rs:71-74).
