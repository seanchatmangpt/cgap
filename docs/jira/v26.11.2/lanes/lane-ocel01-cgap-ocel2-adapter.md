# Lane OCEL-01 receipt — cgap-ocel2 adapter

- **Subject**: `crates/cgap-ocel2/` — standalone membrane crate
  (own `[workspace]`, `publish = false`, NOT a root member), new
  capability: cgap receipt chain -> OCEL 2.0 projection.
- **Standing**: ALIVE (all commands + exits below run this session).
- **Hard boundary held**: zero existing files modified. Tracked M set
  re-read at drain matches the known wave-modified set exactly
  (39 paths); `cgap-ocel2` tracked-mod count = 0; new files are
  untracked `??` only.

## Files written (all untracked)

- `crates/cgap-ocel2/Cargo.toml` — standalone manifest; path deps only
  (cgap-ffi, cgap-arrow, cgap-blake3, cgap-parquet[std], cgap-p28); no
  external crate deps.
- `crates/cgap-ocel2/src/lib.rs` — projection: events/objects/E2O,
  typed refusals, verify-gated projector.
- `crates/cgap-ocel2/src/view.rs` — SQLite/DuckDB DDL constants +
  `full_ddl()` + DDL court.
- `crates/cgap-ocel2/tests/tamper_court.rs` — the falsifier court.
- `crates/cgap-ocel2/tests/roundtrip_court.rs` — columns re-derive
  chain linkage court.
- `docs/jira/v26.11.2/lanes/lane-ocel01-cgap-ocel2-adapter.md` — this
  receipt.

## Mapping (approved sketch)

- **Events**: one row per chain link. `event_id` = chain ordinal
  (receipt zero-gap `index`); documented as ordinal ≠ wall clock, no
  `ocel:timestamp` column, no timestamp substitution. `activity` =
  transition type `receipt.append`. Per-event `receipt_digest` =
  `BLAKE3(prev_hash || block bytes || license bytes)` hex.
- **Objects**: `(dev, ino)` keying shape (`FileIdentity`); object ids
  `file:<dev>:<ino>`, plus block-derived fallback object
  `block:<hex8>` keeping E2O total. `lease_id` per object row.
- **E2O**: `(event_id, object_id, qualifier)`; qualifiers
  `committed_block` / `witness`.
- **Output**: real `cgap_arrow::batch::Batch` columnar tables +
  documented SQL view DDL (`view::OCEL_FLAT_VIEW_DDL`) so pm4py can
  consume.

## Courts (Chicago: real chain / blocks / license / BLAKE3; no mocks; no sleeps)

- `tests/tamper_court.rs` — 5 tests: pristine baseline (non-vacuity
  negative control), tamper-one-block-byte -> `Spoliation`, stored
  receipt-field tamper -> `Spoliation`, license tamper ->
  `Spoliation`, shape verdicts (EmptyChain / WitnessCountMismatch)
  typed, projected block digests match live bytes.
- `tests/roundtrip_court.rs` — from projected columns ALONE: zero-gap
  ordinals, prev-hash linkage (`hash_{i-1} == prev_i`), genesis
  anchor; from columns + live bytes: full link law
  `BLAKE3(prev || block || license)` re-derived per event and matched
  to the projected digest (raw block bytes are not a projected column
  — disclosed boundary); `(dev, ino)` object and E2O edge round-trip.
- `view.rs` court — DDL names all three tables + flat view; the
  "NOT a wall clock" note is asserted load-bearing.
- **Total**: 7 passed / 0 failed, exit 0.

## Anti-vacuity witness (SHIM-COURT, /tmp sandbox — shared tree untouched)

- Sandbox copy `/tmp/ocel01-sandbox` (path deps absolutized to
  /Users/sac/cgap/crates — the relative path deps break in a /tmp
  copy; disclosed).
- MUTATION: verify gate in `Ocel2Projector::project` removed
  (replaced with `let _ = (chain, blocks, license)`).
- AFTER: `cargo test --test tamper_court` FAILED (3 passed / 2
  failed, exit nonzero): `license_tamper_refuses_projection` and
  `shape_verdicts_are_typed` flip; `tamper_one_link_is_typed_refusal`
  still passes because the belt-and-braces per-block digest
  cross-check in `emit` is an independent second layer — defense in
  depth, disclosed. The court FLIPS (a gate whose removal is
  outcome-preserving would be VACUOUS).
- RESTORE: shared lib.rs sha256
  `41ca54d6...41b4208a` byte-identical pre/post (sandbox-only
  mutation; shared tree never mutated, so no mtime bump needed).
- Sandbox deleted at lane end.

## Commands (this session)

- `cd crates/cgap-ocel2 && CARGO_TARGET_DIR=/tmp/ocel01-target cargo test`
  -> 7 passed / 0 failed, exit 0 (twice).
- `CARGO_TARGET_DIR=/tmp/ocel01-target cargo test --test tamper_court`
  in the shimmed sandbox -> FAILED (exit nonzero) — the flip leg.
- `cd /Users/sac/cgap && cargo metadata --no-deps` -> **23 members**,
  `cgap-ocel2` absent (root metadata unperturbed).
- `git status --porcelain` M-set re-read at drain = known wave set
  exactly; no unexpected M; zero cgap-ocel2 tracked modifications.

## Honest scoping (verbatim-bound, mirrored in crate docs)

1. PROJECTION RATE IS A TARGET. No bench exists for this adapter. The
   receipted 82.24 GB/s is BN3-internal (a different instrument, a
   different subject); it says nothing about this projection's rate.
   Any rate claim here is UNVERIFIED until a bench lands.
2. COMMIT-LEG RECEIPT WIRING IS UNSUPPORTED UPSTREAM (FIX-08
   declined). "Every event cryptographically backed" holds ONLY for
   chain-covered events (minted via `RingReceiptChain::append`,
   verified via `verify`). The commit leg itself has no receipt
   surface; events projected from it would be unbacked.
3. No private API reached: all consumed surfaces are `pub`
   (`ChainReceipt` fields, `RingReceiptChain::{verify,receipts,clone}`,
   `BatchBuilder`/`ColumnBuilder`, `SealedBlock::bytes`,
   `P28LicenseAttestation::to_bytes`/`validate_intrinsic`). No decode
   re-derived locally.

## Fences (must-not-regress surfaces this diff could break)

- Root `cargo metadata` member count (23) — run this session, pass.
- No existing file touched, so no existing gate surface is in the
  blast radius; nearest surfaces (`cgap-ffi::receipt_chain` courts)
  are read-only consumers of the public API.
