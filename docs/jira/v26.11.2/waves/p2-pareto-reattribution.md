# p2-pareto-reattribution

Lane p2-pareto: chains-parquet re-attribution + one honest lever.
Repo `/Users/sac/cgap`, canonical checkout, no git writes, publish
frozen. Subject commit `6801affadb861792c73cefb66453b6c5483fc47f`
(working tree carries W3-L2 SWAR-FNV edits on top; this lane layered
its lever on the same files).

## Stage-share table (divan, /tmp/p2-pareto-target, 100 iters)

| leg | 65,536 | 1,048,576 | share @1M |
|---|---|---|---|
| chain_ram (control) | 1.49 ms | 24.02 ms | 100% |
| chain_file (control) | 1.50 ms | 24.15 ms | — |
| seal_arm | 1.062 ms | 17.22 ms | ~72% of chain |
| verify_arm | 425.1 µs | 6.81 ms | ~28% of chain |
| encode_arm | 160.7 µs | 2.57 ms | ~11% (per lane) |
| decode_arm | 47.1 µs | 0.76 ms | ~3% |
| digest_arm (added) | 377.8 µs | 6.05 ms | fnv checksum + block_digest |
| flush_arm (added) | 1.957 µs | 42.1 µs | memcpy only |

Arithmetic: seal_arm(17.22) = writer encode 2 lanes ~10.2 ms + seal
digests ~6.0 ms + flush 0.04 ms. The legacy "86% seal/verify/flush"
attribution was stale post-W3-L2.

## Dominant stage

Writer encode path: `encode_hybrid_into` literal-loop,
crates/cgap-parquet/src/mmap_writer.rs:421-482 — ~5.1 ms/lane vs
2.57 ms for standalone `rle_encode` (encode_arm), i.e. the writer's
byte-at-a-time `push_byte`/`push_varint` body roughly doubles the
encode cost over the standalone encoder at 1M rows.

## Lever landed (ONE)

Fused `SealDigests` sweep — crates/cgap-parquet/src/footer.rs
(struct `SealDigests` region, after `block_digest`). `seal()`
previously made three passes over the body: the meta field-5 body
FNV (`build_meta_block`), the full-length `fnv1a64` over
`[0, B+M+80)`, and 4-lane `block_digest` over `[0, B+M)`. Now one
read pass feeds six FNV-1a-64 states; the meta+attestation tail
feeds the five live states. Byte-exact by construction (identical
`h ^= b; h *= PRIME` byte-step sequence and seeds per state; frozen
constants untouched).

## Before/after (chain numbers, divan mean @1,048,576)

| leg | before | after | delta |
|---|---|---|---|
| chain_ram | 24.02 ms | 23.21 ms | -3.4% |
| chain_file | 24.15 ms | 23.57 ms | -2.4% |
| seal_arm | 17.22 ms | 16.34 ms | -0.88 ms |

verify_arm/encode_arm/decode_arm unchanged within noise, as expected
(the lever touches only the seal-side digest path).

## Target status (honest)

[L2 TARGET-v26.11.2] `<10 ms @ 1M` NOT met. Chain moved 41.54 ms
(P3-17) -> 42.7 (W3-L2, no move) -> 23.21 ms measured this lane (the
24 ms level was already present before this lane's lever — the W3-L2
receipt's 42.7 ms did not reproduce; likely machine or build-profile
drift, recorded as a divergence). Remaining budget: writer encode
~10.2 ms + verify digests ~6 ms + decode 0.76 ms; the <10 ms target
needs the encode-path lever, which is out of this lane's ONE-lever
budget — recorded as the next honest lever target.

## Anti-vacuity mutation court

Mutation: one-bit xor flip inside the levered body
(`*lane ^= byte` -> `*lane ^= byte ^ 1` in `SealDigests::absorb`),
applied in a /tmp sandbox copy and swapped in; sha256 before
`3b01b989d32b9cdc6806bdcbfb7a7421b09c631b3d03b2a974fb356d6746ba7a`
recorded. `cargo test -p cgap-parquet --lib seal` under mutation:
FAILED (2 of 5 courts flipped: `hostile_sealed_block_corpus_fails_
closed_with_typed_verdicts`,
`a04_identity_court_sealed_block_frames_with_oabi_digest`). FLIPPED,
not vacuous. Restore: sha256-exact (verified above), mtime bumped
(a02-finish-receipt.md:170-184 law), `--lib seal` back to 5 passed /
0 failed.

## Equity gates

- `cargo test -p cgap-parquet`: 223 passed / 0 failed / 1 ignored
  exit 0 (180 unit incl. lever_courts byte-wise oracle,
  mutation_kills 17, rle_court 8, codec_properties 4,
  aligned_interop 4, page_zero_alloc 2, robustness 3).
- `cargo build --workspace`: Finished, exit 0 (pre-existing
  cgap-integration manifest warning only, not session-introduced).
- Naming law grep over touched artifacts: zero matches.

## Divergences

1. W3-L2 receipt recorded 42.7 ms @1M chain; this lane measures
   24.02 ms on the same pre-lever tree. Not reproduced; likely
   machine/build-profile drift. Both receipts stand as recorded.
2. The lane contract asked for `seal_verify_arm`; the existing file
   carries `seal_arm` + `verify_arm` (finer split) from P3-17 — kept
   verbatim, added `digest_arm`/`flush_arm` instead.
3. L2's "86% seal/verify/flush" share was stale: measured split is
   writer-encode-dominant (~42%), seal digests ~25% pre-lever.

{"lane":"p2-pareto","files_written":["crates/cgap-parquet/src/footer.rs","crates/bench/benches/chains-parquet.rs","docs/jira/v26.11.2/02-pareto-levers.md","docs/jira/v26.11.2/waves/p2-pareto-reattribution.md"],"commands":[["cargo bench -p bench --bench chains-parquet (before)",0],["cargo bench -- digest_arm flush_arm",0],["cargo test -p cgap-parquet",0],["cargo bench -p bench --bench chains-parquet (after)",0],["cargo test -p cgap-parquet --lib seal (MUTATED)",1],["cargo test -p cgap-parquet --lib seal (restored)",0],["cargo build --workspace",0],["sha256 footer.rs (restore)",0]],"courts_run":["lever_courts (byte-wise FNV oracle, in 223-test gate)","mutation_kills 17 (in gate)","seal-lib courts x5 (mutation FLIPPED 2, restored green)"],"standing":"PARTIAL_ALIVE (lever landed byte-exact, chain -3.4%, <10 ms target not met, encode path recorded as next lever)","divergences":["W3-L2 42.7 ms baseline not reproduced (measured 24.02 ms pre-lever)","contract named seal_verify_arm; existing seal_arm+verify_arm split kept","legacy 86% seal-share attribution was stale — encode-dominant measured"]}
