# W2-11 — L1/L2 court shells (07-court-corpus §2)

Written (sole-writes honored; no git commands; /Users/sac/stardf untouched;
frozen pins byte-identical):

- `crates/cgap-gate/tests/lever_courts.rs` (98 lines)
- `crates/cgap-parquet/tests/lever_courts.rs` (112 lines)

## Courts

- **L1** (`l1_attestation_byte_flip_escalates_to_spoliation`): hand-built
  v1 frame (ROOT lineage, zero receipt, ceiling at bound) ADMITTED
  (CGAP_OK, 32 B payload view); five attestation-slot flips (magic,
  version, lineage-zero, lineage-two-bits, ceiling-overflow) each flip
  the ingress verdict to `CGAP_ERR_P28_SPOLIATION`. Named anti-vacuity
  mutant: `l1_ceiling_law_flip_is_the_named_mutant` (u64::MAX ceiling —
  kills any W3 kernel edit that drops the ceiling law, penstock.rs:566-568).
- **L2**: reference byte-wise FNV-1a (written in the test file, zero
  kernel code in the oracle path) byte-exact equal to
  `fnv1a64_seeded` over {empty, 1..=16, 4097 bytes} x 5 seeds incl. the
  frozen corpus seed `0xA20C_0FFE_E5A7_E000`; published FNV vectors
  ("" / "a" / "foobar") asserted; committed-window (body) byte flip ->
  `SealError::ChecksumMismatch { region: CorruptRegion::Body }`.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/w2-11-target cargo test -p cgap-gate --test lever_courts`
  -> exit 0 (2 passed, 0 failed)
- `... cargo test -p cgap-parquet --test lever_courts` -> first run FAILED
  (seal RunOverrun: test buffer lacked footer room — court-author bug,
  fixed by sizing buf = body+256); rerun exit 0 (3 passed, 0 failed)
- Gate court re-confirmed after the parquet fix: exit 0 (2 passed)
- Build-root lease `/tmp/w2-11-target` deleted (rmtree; verified absent).

## Standing

ALIVE (both suites ran + passed this session against the pre-lever tree;
W3 SWAR rewrite must keep `l2_reference_equity_over_corpus` green).

## Divergences

- penstock ingress row ordering (magic/version first) means the L1 spec's
  "flip one byte at the attestation offset" court can never reach the
  attestation checks via magic/version bytes — flips are confined to the
  OFF_ATTESTATION..+80 slot, which is what the corpus spec intends.
- Corpus spec says "sandbox copy mutation" for L1; the ingress court
  mutates the frame bytes in memory (the gate kernel is pure — no tree
  mutation occurs), satisfying the sandbox law trivially.
