# Lane RC-12 — penstock malformed-corpus differential court

Date: 2026-11-02 · Standing: **ALIVE** · Branch main · read-only over
`crates/cgap-gate/src/penstock.rs` (sha256 `bb4ed64d…f9c1e3e8` at lane end,
never touched).

## 1. Subject / diff

One NEW file, exactly the owned surface:

- `crates/cgap-gate/tests/penstock_corpus_court.rs` (507 lines)

## 2. What the court covers

Deterministic corpus (no rng) mapped to the documented six-row fail-closed
table (`penstock.rs:505-511`):

- Row 0 valid/zero-payload/oversized-slack (admit), row 1 truncation at
  EVERY offset 0..127 + lying-caller declared len > slice, row 2 magic
  families + version, row 3a zero digest, row 3b declared-length lies
  (9 / 264 / 2^20 / u64::MAX), row 4 attestation magic families, row 5
  attestation field laws incl. receipt law broken in BOTH directions,
  row 6 reserved-header-flags admitted-with-escalate (recorded divergence).
- Bit-flip differential: bit-0 flip of every header byte 0..128 of a valid
  frame, expected class from a documented per-offset mapping
  (`flip_exp`); 46 of the 128 flips are spoliation-class.
- Cross-product legs (red-09 D3 class): byte-at-a-time prefix feeding
  (k = 1..len with declared k) and split-frame chunked reassembly + 64-B
  trailing slack converge to the one-shot class, with the table-order law
  encoded (InvalidMagic cases stay Invalid at every prefix; later-group
  cases degrade to OABI until the declared geometry fits).
- Admit-path view law on every Ok case (zero-copy pointer, declared
  geometry, escalate bit only for the row-6 family).
- Coverage pin (anti-vacuity at the table level): every taxonomy family
  and every expected class present; flip-mapping spoliation count pinned
  at 46.

## 3. Commands / exits (this session)

- `cargo test -p cgap-gate --test penstock_corpus_court` — exit 0,
  5/5 passed.
- `cargo test -p cgap-gate` — exit 0 (12 + 3 + 5 + 0 across targets).
- `cargo test -p cgap-gate --release` — exit 0 (12 + 3 + 5 + 0).
- Anti-vacuity shim (`SHIM-COURT`, /tmp copy only): flipped ONE mapping
  (`16..=47 => Ok` -> `InvalidMagic`) in a /tmp sandbox crate
  (`/tmp/rc12-sb`, path-dep on the shared tree, deleted at lane end) —
  court FAILED with 2 courts flipping
  (`corpus_differential_one_shot_…`, `corpus_prefix_feeding_…`),
  exit nonzero. Non-vacuous.
- Shared-tree cleanliness: `git status --porcelain crates/cgap-gate/`
  shows only the new owned file; penstock.rs byte-identical.

## 4. Findings

- ZERO gate-vs-doc mismatches: every corpus input's observed class
  matched the documented class on the clean run — the six-row table is
  systematically conformant, not just row-by-row courted. The two
  failures during bring-up were mapping-authoring bugs in THIS lane's
  expected-table (prefix-order law + a 47-vs-46 count pin), fixed in the
  court, not the gate.
- Flip-mapping subtleties now pinned as documentation-in-court: bytes
  5..8 (flags/reserved) are NOT refusal conditions (flip -> Ok); ceiling
  bytes 58/60 flips REDUCE the value back under ceiling (Ok), bytes
  56/57/59/61-63 increase it over (spoliation).

## 5. Fences / must-not-regress

- `cargo test -p cgap-gate` (dev + release) — ran this session, exit 0.
- `cargo build --workspace` not run (lane-owned file only; the cgap-gate
  target suite is the named fence for this surface). Recorded as UNRUN,
  not passed.

## 6. Standing

**ALIVE** for the corpus court: commands + exits above, this session.
`cargo build --workspace` standing remains UNVERIFIED this session (not
this lane's named fence).

## 7. Cleanup

`/tmp/rc12-sb` sandbox and `/tmp/rc12-target` lease deleted at lane end;
`tests/corpus_tmp` debris from a mis-directed scratch append removed.
