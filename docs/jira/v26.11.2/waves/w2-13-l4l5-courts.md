# W2-13 — L4/L5 court shells (pre-lever controls)

Lane: W2-13, wave v26.11.2, date 2026-11-01. Subject files (sole writes):

- `crates/cgap-ffi/tests/lever_courts.rs`
- `crates/cgap-p28/tests/lever_courts.rs`

Charter: `docs/jira/v26.11.2/07-court-corpus.md` §2 (L4-COURT, L5-COURT);
lever engineering: `docs/jira/v26.11.2/02-pareto-levers.md` §L4/§L5.
Court discipline: sandbox-only, no kernel edits, exit contract 0/1/2/3.

## Standing

ALIVE — both suites ran and passed this session:

- `CARGO_TARGET_DIR=/tmp/w2-13-target cargo test -p cgap-ffi --test lever_courts` -> exit 0 (2 passed).
- `CARGO_TARGET_DIR=/tmp/w2-13-target cargo test -p cgap-p28 --test lever_courts` -> exit 0 (2 passed).
- Build root `/tmp/w2-13-target` deleted at lane end (lease law).

## L4 — sovereign commit sink (cgap-ffi/tests/lever_courts.rs)

1. `l4_committed_header_flip_is_exactly_spoliation` — REAL commit leg
   (`cgap_commit_parquet`, sovereign.rs:706-730) commits 1024 alternating
   lanes (CGAP_OK, non-empty sealed file); the verify leg
   (`cgap_verify_p28`) admits the unmutated 80 B root attestation
   (CGAP_OK); flipping ONE byte of the origin_authority_digest (offset 16,
   A17 §2.1) yields EXACTLY `CGAP_ERR_P28_SPOLIATION` (0x5032_38FF);
   byte-exact restore re-admits with an identical fold digest. Flip is the
   verdict signal (anti-vacuity, flips both ways).
2. `l4_status_flip_witness_at_lease_edge` — the reachable typed
   CgapStatus-flip witness: 1024 alternating lanes commit (CGAP_OK); 70,000
   alternating (incompressible at 8 bits) refuse typed
   `CGAP_ERR_LEASE_EXHAUSTED` (0x43470004).

### L4 named bench-level court (recorded, not faked)

The skipped-slab-write mutation is a SOURCE mutation of
`sovereign.rs:706-730` (drop one `writer.write_slab` leg) — not injectable
from a unit test without touching the shared tree. Procedure (named court
`L4-BENCH-SKIPPED-SLAB`): sandbox-copy `crates/cgap-parquet/src/` +
`Cargo.toml` to /tmp, comment the `write_slab` call, run the commit corpus
there with `CARGO_TARGET_DIR=/tmp/l4-skipped-slab-target`; kill signal =
per-stage `CgapStatus` assert fires (CGAP_OK no longer lands on a
committed file whose slab bytes are absent); restore sha256-exact + mtime
bump per the mtime-bump restore law. Exit contract: 0 green / mutated
nonzero = FLIPPED.

## L5 — seal chain (cgap-p28/tests/lever_courts.rs)

1. `l5_mixed_corpus_three_way_invariants` — mixed corpus (N=4: 2 licensed,
   2 unlicensed; first unlicensed crosses) through the CURRENT unfused
   governance path (`Asc450Meter` + `LeaseEpochSealer`, real A16 lease,
   real pinned instrument):
   - crossing discipline: exactly 1 crossing proof emitted (one-shot
     latch), event_count == N, latched set, sealed state minted on
     emission; the emitted proof count equals the crossing-capable N/2
     partition at the latch-saturating corpus size.
   - prev_root chaining: genesis sealed state chains
     `GENESIS_PREV_PROOF_DIGEST`; post-emission sealed state chains
     BLAKE3(proof bytes); post-latch seals chain the same last proof.
   - checksum: proof verifies Ok; one flipped covered byte (offset 100,
     shape-clean) in the proof or (offset 10) in the sealed state FAILS
     verify/verify_state (anti-vacuity: flips both ways).
   - Sealer: two `LeaseEpochSealer`s over the same deterministic lease
     epoch key (tag 0xA20C_0FFE_E5A7_E000) — verify is pure, consumes no
     ticks; the corpus seed uses the frozen corpus constant, not the
     SplitMix64 multiplier (pin separation law).
2. `l5_upstream_receipt_flip_is_refused` — source-mutation twin (reachable):
   derivative bound to parent wire bytes validates under
   `validate(span, Some(&parent))`; one flipped upstream_receipt byte
   (offset 48, A17 §2.1) is REFUSED.

### L5 named bench-level courts (recorded, not faked)

Source-level mutations of `crates/cgap-p28/src/{asc450,attestation}.rs`
are kernel mutations — recorded as procedures, not faked as unit tests:

- `L5-BENCH-DEMOTE-CROSSING`: sandbox-copy cgap-p28 src, gate the `if
  crossing` emission arm; kill signal = emitted-proof-count assert
  (proofs == 1) flips to 0; restore sha256-exact + mtime bump.
- `L5-BENCH-KERNEL-RECEIPT-FLIP`: sandbox-copy, corrupt the digest fold in
  the attestation validate path; kill signal = validate refuses the honest
  record (positive twin fails). Exit contract per court: 0 green baseline,
  mutated run nonzero = FLIPPED.

## Anti-vacuity summary

Every court carries a real one-byte sandbox mutation with a both-ways
verdict flip (admit -> EXACTLY spoliation -> re-admit; Ok -> typed LEASE
refusal; verify Ok -> FAIL; validate Ok -> refuse). No Process.sleep; no
mocks; real collaborators (real commit path, real lease epoch, real pinned
instrument read from disk). Restores are byte-exact in-memory wire flips
(fold-digest equality asserted); no shared-tree file was mutated, no git
commands run.

## Divergences

- 07-court-corpus cites `crates/cgap-ffi/src/sovereign.rs:706-730` and
  refusal consts at :83 — verified accurate on disk this session.
- The corpus doc's L4 "corrupt one committed header byte" is realized as
  the attestation-header flip through the verify leg (the committed
  parquet block itself is consumed by the L2 seal court; no A19 entrypoint
  reads a parquet file back).
- `instrument_header_span(LICENSE-P28-SIL.md)` is 413 bytes including the
  trailing newline after `---` (matches sovereign.rs's frozen
  PINNED_INSTRUMENT_HEADER; both lanes' courts validate against the same
  span).
