# R2 Wire-Parity Receipt

Lane: W2-11 WIRE-PARITY-CLOUD (wave R2 item 11), repo `/Users/sac/cgap`,
branch `main`, one canonical checkout, no git writes. Lease:
`ontologies/cloud/` (tests per its layout) +
`docs/jira/v26.11.1/waves/r2-wire-parity.md` (this file). A10's lane is
concluded; this was the sole active writer; all A10 artifacts
(`src/lib.rs`, `src/bin/emit-pack.rs`, `tests/courts.rs`,
`tests/raw_consumer.rs`, `tosca_cimi_v1.oabi`) were treated read-only and
are byte-identical after the pass (sha256 witness below). Date
2026-11-01. Lane spec: `docs/jira/v26.11.1/lanes/A10-cloud-oabi.md`
(authoritative for vocabulary and layout).

## What the spec declared

- The event surface (the "siphon"): lane doc §5.3 pins the EventStream
  vocabulary — `cimi.Event`, `cimi.Event.content`, `cimi.Event.severity`
  (enum family 3: INFO=0 / WARNING=1 / CRITICAL=2, per the crate's
  `ENUM_FAMILIES` table), `cimi.Event.timestamp`, `cimi.Job` (alias of
  `cimi.Event` via the §5.3 equivalence row), `eventstream.channel`
  (the one sanctioned `LOCAL(A10)` row on this class).
- §6 pins the pack bytes (A06 v1 header + 31 x 48 B records); §7 pins
  five courts (T1 pack digest, T2 slot goldens, T3 loader refusals, T4
  equivalence, T5 C-ABI consumption).
- **No wire golden vector is declared.** §7 defines no event-frame
  encoding and no golden byte vector for an event→slots mapping. The
  missing spec vector's coordinates:
  `docs/jira/v26.11.1/lanes/A10-cloud-oabi.md` §7 (court list T1-T5,
  none is a wire vector) and §6 (pins pack bytes only). Per the W2-11
  dispatch, the vectors were therefore constructed deterministically
  from the committed pack and are labelled NEWLY-MINTED at every use
  site (`tests/wire_parity.rs` GOLDEN_E1/GOLDEN_E2 doc comments).

## What is witnessed (this session, this tree)

The siphon event → tosca_cimi slots → 64-byte wire frame ("CEW1",
layout documented in the court file) is ALIVE under three independent
derivations agreeing byte-for-byte:

1. PYTHON MINTER: parses the committed slot goldens out of
   `src/lib.rs` (no hand transcription) and assembles both frames; its
   output is the committed golden hex.
2. FACADE path: encodes via `cgap_cloud` public API only
   (`slot_of`, `resolve_equivalence_lexeme`, `LEXEME_TABLE`,
   `ENUM_FAMILIES`).
3. RAW path: parses the `.oabi` bytes by offset, re-derives every
   record SlotId from its stored LexHash, resolves each event lexeme
   through real BLAKE3 preimages, and cross-checks record
   ClassId/EnumId fields against hand-restated spec constants — zero
   `cgap_cloud` items.

Two events witnessed: E1 = plain CIMI event, presented root
`cimi.Event`, severity WARNING; E2 = job-completion event presented
through the `cimi.Job` alias, severity CRITICAL — the alias resolves
on the wire (canonical `cimi.Event` slot at frame offset 16, alias
slot at offset 56, asserted different).

## Courts run (name -> pass / flip evidence)

| Court | Verdict | Evidence |
|---|---|---|
| `wire_parity_facade_raw_golden` | PASS | E1+E2: facade == raw == golden, byte-identical (assert at tests/wire_parity.rs:348-349); E2 field witnesses: canonical root == `CIMI_EVENT_SLOT`, presented == `CIMI_JOB_SLOT`, root != presented |
| `golden_sensitivity` | FLIPS | variant swap (INFO/CRITICAL) and presented swap both fail to reproduce the goldens (`assert_ne` held); out-of-subset severity "FATAL" refuses typed `VariantUnknown` |
| `flip_slot_corruption_refuses_typed` | FLIPPED | one SlotId byte of record 19 (`cimi.Event.content`, named by real-BLAKE3 re-derivation BEFORE the flip) corrupted in a /tmp copy, flip byte = SEED % 8 = 1 from canonical seed 0xBF58476D1CE4E5B9 -> parity path refuses typed `SlotDerivationMismatch { index: 19 }`; restore byte-exact (BLAKE3 digest equal), mtime bumped on the copy; canonical pack sha256+mtime identical |
| `flip_class_field_corruption_refuses_typed` | FLIPPED | ClassId field of record 20 (`cimi.Event.severity`) 2 -> 3 (in range; slot derivation proven still holding on the mutated copy, so A06 T3's check is blind to this class) -> refuses typed `ClassFieldMismatch { record_class: 3, expected_class: 2 }` naming `cimi.Event.severity`; same restore protocol |
| `courts.rs` T1/T1-mut/T2/T3/T4/T4-flip/metadata/A06-witness (8) | PASS | re-run green this session (pre-existing, unmodified) |
| `raw_consumer.rs` (1) | PASS | re-run green this session (pre-existing, unmodified) |

Anti-vacuity is real: flip court B is the court's distinguisher from
the A06 loader's T3 (slot re-derivation cannot see a class-field
corruption; the parity court's field cross-check can — witnessed by the
in-court assertion that derivation still holds on the mutated copy).

## Commands / exits (this session)

`CARGO_TARGET_DIR=/tmp/w211-target` (deleted via python3
`shutil.rmtree` at pass end):

| Command | Exit | Witness |
|---|---|---|
| `python3 /tmp/w211-mint/wire_mint.py` | 0 | E1/E2 golden hex minted from committed lib.rs goldens; lib.rs class/family cross-checks green |
| `cargo test -p cgap-cloud --test wire_parity` | 0 | 4 passed; 0 failed |
| `cargo test -p cgap-cloud` | 0 | 13 passed; 0 failed (0 lib + 0 bin + 8 courts + 1 raw_consumer + 4 wire_parity) |
| `shasum -a 256 ontologies/cloud/tosca_cimi_v1.oabi` | 0 | `26a1783e...c5684469` identical before and after the run |
| `stat -f "%m"` on the pack | 0 | mtime 1793540913 identical before and after (no shared-tree mutation) |
| python3 `shutil.rmtree('/tmp/w211-target')` | 0 | lane build root deleted (lease cleanup law) |
| `ls -d /tmp/w211-wire-parity-*` | — | no matches: both flip courts removed their own sandbox dirs |

## Golden minting procedure (replayable)

`wire_mint.py` (run from /tmp, deleted after; algorithm restated here
for replay): regex-parse every
`LexemeRow { lexeme: "...", class: N, enum_family: F, ... slot: 0x... }`
from `ontologies/cloud/src/lib.rs`; map family constants to ids
(NONE=0, MACHINE_STATE=1, VOLUME_STATE=2, EVENT_SEVERITY=3,
JOB_STATUS=4, CREDENTIAL_TYPE=5); assert the six event lexemes' classes
are 2 and severity's family is 3; assemble per frame layout:

```text
CEW1: [0..4]=magic "CEW1", [4..6]=version 1 LE,
[6..8]=class 2 LE, [8..10]=family 3 LE, [10..12]=variant LE,
[12..16]=zero, [16..24]=canonical root slot LE,
[24..32]=cimi.Event.content slot LE, [32..40]=cimi.Event.severity slot LE,
[40..48]=cimi.Event.timestamp slot LE, [48..56]=eventstream.channel slot LE,
[56..64]=presented slot LE (root for E1, cimi.Job for E2)
```

Output (committed verbatim in `tests/wire_parity.rs`):

```text
E1 4345573101000200030001000000000037BD719B611F1EEF084E636A119313DF9DE358047C559350E60D17F9357AC358EF1543592580280837BD719B611F1EEF
E2 4345573101000200030002000000000037BD719B611F1EEF084E636A119313DF9DE358047C559350E60D17F9357AC358EF15435925802808E152B44C718B61ED
```

## What remains

- T5 (C-ABI consumption through the real A19 surface) stays BLOCKED on
  A19 — pre-existing, receipted at
  `a10-finish-receipt.md:82-107` (sovereign `#[no_mangle]` hits absent);
  not this lane's scope and not faked here. Lane A10 standing remains
  PROPOSED per its DoD item 11 until T5 is not-BLOCKED.
- The CEW1 frame is court-defined, not spec-defined. If A10's spec
  later declares a wire golden vector, the facade/raw derivation paths
  are reusable and the court compares directly against it.
- Wave-2 (broadened vocabulary, bidirectional equivalence) is recorded
  scope in the lane doc §10 — untouched here.

## Divergences

- The dispatch context said "9/9 courts" landed for A10; the on-disk
  count is 8 (courts.rs) + 1 (raw_consumer) = 9 test cases, which the
  crate suite confirms (8+1+4=13 with this lane's additions). Counting
  convention, not a defect.
- A10 lane doc §6's example enum_count formula ("per committed
  enum-carrying lexemes" = 5) vs the landed `ENUM_COUNT = 6` (NONE
  sentinel inside the loaded space) was already recorded as a divergence
  by A10's own lane report (lib.rs:92-100); re-observed here, not
  re-adjudicated.
