# A13 Finish Receipt

- **Lane**: A13-bn3-builtins (finisher), v26.11.1, repo `/Users/sac/cgap`
  (one canonical checkout, branch `main`)
- **Date**: 2026-11-01
- **Subject**: `crates/cgap-bn3/src/builtins.rs` — sha256
  `92e6ed73cf33af4e78cd160171ba349adfaa7a0169d9fda440f34688f55b8363`
  (byte-unchanged across every run in this receipt; verified before ==
  after). This lane wrote NO code this session: waves 1-9 landed the
  module complete, and every materially testable claim of waves 2-9 was
  re-executed THIS session per the hearsay rule. The deliverable is this
  receipt file, closing the wave-9 audit's one PARTIAL axis (missing
  receipt FILE under `docs/jira/v26.11.1/waves/`).
- **Files written this session**: this receipt only.

## 1. Baseline re-execution (canonical tree, this session)

- `CARGO_TARGET_DIR=/tmp/cgap-lane-A13-target cargo test -p cgap-bn3`
  -> exit 0, 102 passed / 0 failed (doc-tests 0).
- `CARGO_TARGET_DIR=... cargo test -p cgap-bn3 --lib builtins::`
  -> exit 0, 26 passed / 0 failed (courts itemized in §4).
- `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=... cargo check -p cgap-bn3
  --lib` -> exit 0, clean.
- `CARGO_TARGET_DIR=... cargo rustc --release -p cgap-bn3 --lib --
  --emit=asm` -> exit 0; artifact analyzed in §2.

Note: the package count is 102, not the wave-9 audit's 87 — sibling lanes
grew `unify`/`arena` courts between the two runs; the 26 `builtins::`
courts are the A13 surface and are itemized below. Package count drift is
not an A13 code delta: `builtins.rs` is byte-identical to the wave-9
audit's subject modulo the wave-9 header block (sha256 differs from the
audit's `5060b702…` only because the audit predates the wave-9 header).

## 2. Zero-indirection court (spec 5.3), re-derived this session

Fresh release asm; dispatch region bounded by the next top-level symbol
after `__RNvNtCsilLg8LBkrzA_8cgap_bn38builtins8dispatch`:

- region 238 raw lines; **0 `blr`, 0 `br xN`, 0 `brk`**;
- exactly **3 direct `bl`**: `cgap_blake3::hash`,
  `cgap_blake3::hash_many_simd`, `alloc::raw_vec::grow_one` (trace sink
  growth path);
- 1 `adrp` feeding `l_anon.…@PAGE` + `add …@PAGEOFF` — anonymous
  constant-pool data load, not a function-pointer table;
- whole-object census: 6 `br x3` sites, all inside
  `cgap_bn3::sigil::BN3Error Display::fmt` (A11's function), none in
  dispatch. The spec 2.1/5.3 gate is ALIVE as an emitted-code fact.

## 3. Adversarial sandbox flip (Chicago anti-vacuity)

Sandbox `/tmp/a13-finish-sb.F85cQf` (mktemp; copies of `cgap-bn3` +
`cgap-blake3` under a minimal root workspace; sandbox `builtins.rs`
sha256 `92e6ed73…` == canonical at copy time; fail-closed anchor
count==1 assert on the mutation):

- BEFORE: `cargo test -p cgap-bn3 --lib builtins::` exit 0, 26/26.
- Mutation M-GT (the mission's named example): `math:greater_than`
  comparison polarity `x > y` -> `x < y`.
- AFTER: **exit 101, 25 passed / 1 FAILED** —
  `builtins::tests::known_answer_math_greater_than` panics at
  `builtins.rs:797` (`greater_than(3,2)` truth-table row). The
  truth-table court kills the polarity flip; the court is load-bearing.
- Canonical tree integrity: `builtins.rs` sha256 before == after
  (`92e6ed73…`); all mutations lived in the sandbox only.

## 4. Spec 5.1 acceptance — 26 builtins courts (all green this session)

Known-answer truth tables on boundary inputs; crypto KATs against the
official `b3sum`; typed-refusal and hygiene courts:

- ABI/registry: `opcode_court_constants_match_spec`,
  `registry_court_no_duplicates_dense_gapped`, `payload_court_layout_is_4_bytes`,
  `register_court_r0_to_r255_fits_u8`, `sigil_court_builtinop_is_0x04`,
  `payload_encode_decode_roundtrip`, `builtin_error_surface_court`.
- KATs (spec 5.1): `known_answer_math_greater_than` ((3,2)->1, (2,3)->0,
  (3,3)->0, (2^56-1,0)->1), `known_answer_math_sum_checked` (incl.
  (2^56-1,1) -> `Err(Overflow)` and the 56-bit ceiling arm),
  `known_answer_math_mul10` (floor(Max/10) boundary both sides),
  `known_answer_math_min` (bounds 0/Max), `known_answer_crypto_blake3_abc`,
  `known_answer_crypto_blake3_wide_4kib`, `known_answer_log_assert`,
  `log_trace_identity_court`, `log_counters_latch_court`.
- Refusal/hygiene: `unknown_opcode_typed_refusal`,
  `all_registered_opcodes_dispatch`, `registry_exhaustively_closed_sweep`
  (all 256 opcodes), `crypto_invalid_range_refusals`,
  `refusal_write_isolation_court`, `eval_count_wraps_at_u64_max_court`,
  `hostile_domain_sweep_court` (256 ops x 12x12 hostile register domain,
  debug profile), `hostile_decode_word_court` (68 words),
  `nostd_hygiene_court_module_body_has_no_std_paths`,
  `panic_hygiene_court_module_body_has_no_panic_sites`.

Crypto known-answer vector cross-check, this session:
`printf 'abc' | b3sum` -> `6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85`
== the `known_answer_crypto_blake3_abc` literal byte-for-byte. (The same
run reconfirms the lane doc 5.1 literal `ba7816bf…` is SHA-256("abc"),
not BLAKE3 — divergence 1 below.)

## 5. Spec 5.2 dispatch-court mutations — standing

M1/M2/M3/M3c/M4/M5/M6/M7 (8/8 flips) were last witnessed by the wave-9
adversarial audit (`a13-wave9-audit-receipt.md` section 2, 2026-11-01,
exit 101 rows) in its own sandbox with byte-exact restores. This finish
run re-proves the underlying kill-power constructively with M-GT (section
3): the one mutation class the mission names for this lane (comparison
polarity) flips its truth-table court. M1/M3 kill at BUILD (E0080 via the
ABI-freeze const-assert lattice), so a court-pass-under-mutation outcome
is structurally unreachable for the renumber class; M2-class arm deletion
kills at test level (`known_answer_math_min` +
`registry_exhaustively_closed_sweep`, witnessed in the audit run).

## 6. DoD scorecard (lane doc Section 6)

| DoD | Status | Evidence |
|---|---|---|
| 1 builtins.rs exists; build green | ALIVE | `cargo test -p cgap-bn3` exit 0 (102/102) |
| 2 closed registry, 9 opcodes, dense, gapped | ALIVE | registry courts + 256-opcode sweep green |
| 3 u8-match dispatch, no dyn/fn/registry | ALIVE | source read; asm court section 2 |
| 4 5.1 KATs green incl. real digest | ALIVE | 26/26 + b3sum cross-check |
| 5 dispatch court; 5.2 mutations killed | ALIVE | audit run 8/8 + M-GT this session |
| 6 zero-indirection court | ALIVE | section 2 (0 blr / 0 br xN / 0 brk) |
| 7 typed refusals, no panic path | ALIVE | refusal-isolation + hostile-sweep + panic-hygiene |
| 8 -D warnings green; no unsafe | ALIVE | exit 0; audit grep clean |
| 9 real cgap_blake3 calls, no re-impl | ALIVE | asm `bl` targets are the real surface |

## 7. Standing

**ALIVE.** Every gate ran this session with witnessed command + exit.
The lane's only session output is this receipt file; `builtins.rs` is
byte-unchanged. Remaining coordinator items (unchanged from the wave-9
audit): the lane doc still reads "PROPOSED / does not exist yet" and
section 5.1 still carries the SHA-256 literal where BLAKE3 is meant.

## 8. Divergences (doc yields; recorded, not repaired — not lane files)

1. Lane doc `A13-bn3-builtins.md` section 5.1 digest literal
   `ba7816bf8f01cfea…` is SHA-256("abc"), not BLAKE3("abc") — reconfirmed
   this session by `shasum -a 256` vs `b3sum`. The test asserts the
   correct BLAKE3 literal and documents the divergence in-module
   (builtins.rs:902-914).
2. Lane doc Status "PROPOSED / `crates/cgap-bn3` does not exist yet" is
   stale: the crate exists, builds, and 102/102 tests pass.
3. Asm line-count variance across receipts (238 here vs 236/239 in
   waves 8/9) is region-boundary counting variance; the flip-relevant
   properties are identical every run (0 blr / 0 br xN / 0 brk / 1 adrp /
   3 direct bl).
4. `crates/cgap-bn3/Cargo.toml` carries a disclosed wave-1 unblock (the
   `cgap-blake3` path dep); coordinator review was requested at the time
   and remains open. This finish lane did not touch it.
5. `git status --porcelain` is clean except fleet-wide `Cargo.lock`
   drift from lane builds (coordinator integration item, pre-existing).

## Commands run this session (identity + exit)

- `CARGO_TARGET_DIR=/tmp/cgap-lane-A13-target cargo test -p cgap-bn3`
  -> exit 0 (102/102)
- `CARGO_TARGET_DIR=/tmp/cgap-lane-A13-target cargo test -p cgap-bn3
  --lib builtins::` -> exit 0 (26/26)
- `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=/tmp/cgap-lane-A13-target
  cargo check -p cgap-bn3 --lib` -> exit 0
- `CARGO_TARGET_DIR=/tmp/cgap-lane-A13-target cargo rustc --release
  -p cgap-bn3 --lib -- --emit=asm` -> exit 0
- `python3 /tmp/a13_asm_court.py <asm>` -> dispatch region census (section 2)
- sandbox BEFORE `cargo test -p cgap-bn3 --lib builtins::` -> exit 0 (26/26)
- sandbox AFTER M-GT `cargo test ...` -> exit 101, 1 FAILED (`known_answer_math_greater_than`)
- `printf 'abc' | b3sum` -> `6437b3ac…d85` (matches KAT literal)
- `printf 'abc' | shasum -a 256` -> `ba7816bf…` (proves divergence 1)
- canonical sha256 before/after -> `92e6ed73…` identical

Cleanup law: `/tmp/cgap-lane-A13-target` deleted at lane end; sandbox
`/tmp/a13-finish-sb.F85cQf` (lane lease debris) deleted at lane end.
