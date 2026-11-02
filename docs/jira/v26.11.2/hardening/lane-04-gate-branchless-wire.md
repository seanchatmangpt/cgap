# Lane H4 — penstock branchless wire verification

Standing: **ALIVE** (commands + exits run this session, 2026-11-02).
Scope: read-only verification of the REP5 branchless swaps in cgap-gate
`penstock`, bench, anti-vacuity court, and the BR1 poison_mask site check.
Only file written: this receipt.

## 1. Landed swaps verified

- `crates/cgap-gate/src/penstock.rs:651-660` (`is_zero_at`) now delegates
  to `cgap_branchless::penstock::swar_all_zero_vl` (crates/cgap-branchless/
  src/penstock.rs:64, SWAR all-zero detect, HI=0x8080..8080 / NOT_HI consts
  at :50-51).
- `crates/cgap-gate/src/penstock.rs:676-686` (`prefilter8`) now delegates to
  `cgap_branchless::penstock::penstock_prefilter8` (crates/cgap-branchless/
  src/penstock.rs:44, mix constant `PENSTOCK_PREFILTER_MIX`
  `0x2544_5491_4F6C_DD1D` at :30 — NOT a frozen seed; seed-hygiene note
  in both modules).
- Delegate lines cited in-source with a "REP5 swap" comment and a
  sha256-pinned shipped oracle (pin receipt inside
  cgap-branchless/src/penstock.rs:116-120:
  prefilter8 `0a9447ab...a82c4`, is_zero_at `6e66ff3d...5d7`).

## 2. Test receipts (CARGO_TARGET_DIR=/tmp/h4-target)

- `cargo test -p cgap-gate` — exit 0: 12 unit courts passed (incl.
  `dam_corpus_10m_zero_alloc_zero_misclassified`, 60.10s) + 3 L1 lever
  courts passed (`l1_ceiling_law_flip_is_the_named_mutant`,
  `l1_attestation_byte_flip_escalates_to_spoliation`,
  `l1_prefilter_escalation_policy_court`).
- `cargo test -p cgap-branchless --lib` — exit 0: 32 passed, incl.
  `penstock::courts::prefilter_differential_vs_shipped` (50k corpus vs
  sha256-pinned shipped oracle),
  `all_zero_differential_vs_shipped`, and both named mutation courts
  `mutation_m_mix_seed_conflation_flips` +
  `mutation_m_zero_has_zero_conflation_flips`.

## 3. Bench (chains-ingress, bench profile, this host)

`cargo bench -p bench --bench chains-ingress`, exit 0. stage_penstock_
admission, median:
- 65536 frames: 536.8 us -> **8.19 ns/frame** (baseline 10.65;
  w6-b7 target band 9.99-10.71)
- 1048576 frames: 9.916 ms -> **9.45 ns/frame** (baseline 11.73)

Verdict: **zero regression** — both sizes land under the w6-b7 target
band's lower edge, i.e. faster than target, far under the 10.65/11.73
baseline. Method: median of 100 samples, timer precision 41 ns (diva
harness), release/bench profile, /tmp/h4-target.

## 4. Anti-vacuity court (sandbox mutation, /tmp only)

- SHIM-COURT: replaced `PENSTOCK_PREFILTER_MIX`
  `0x2544_5491_4F6C_DD1D` with the seed-conflated
  `0xBF58_476D_1CE4_E5B9` in a /tmp copy of cgap-branchless
  (/tmp/h4-sandbox/cgap-branchless, lease deleted after).
- Flip observed: `prefilter_differential_vs_shipped` and
  `mutation_m_mix_seed_conflation_flips` both FAILED (prefilter
  diverges from the shipped oracle: left 11609655306693650753 vs
  right 17158879954370695877 on buf=[63,123,236,65,43,253,93,39]).
- Flip exit: **101** (cargo test failure), captured with `echo $?=$?`
  after redirect, not after a pipe (first echo after tail was a pipe
  artifact = 0, disclosed).
- Gate linkage: cgap-gate `prefilter8` delegates to this constant, so
  admission decisions diverge (l1_prefilter_escalation_policy_court
  asserts `view.prefilter == prefilter8(&frame[..8])` at
  lever_courts.rs:120-122 — corrupting the constant breaks that
  equality). Shared tree untouched: mutation was /tmp-only;
  post-check sha256(cgap-branchless/src/penstock.rs) =
  42404ce8e9334933d1af36e951acf6c91b9101272bf930d97617af5b1e1205a6.

## 5. poison_mask / len>MAX site (BR1 flag) — honest result

- `poison_mask` lives at crates/cgap-branchless/src/arithmetic.rs:39
  (all-ones iff quota==0) with its own differential oracle courts
  (arithmetic.rs:96-124); it has **zero consumers** in cgap-gate or
  cgap-ffi (grep witnessed 2026-11-02). BR1's routing stands: no
  poison_mask/len>MAX hot site exists in the gate admission path.
- The admission length checks are plain branches, not a poison_mask
  select: row-1 `len < HEADER_LEN` (penstock.rs:524, INVALID_MAGIC)
  and row-2 `len > bytes.len()` (penstock.rs:530). The `MAX_FRAME`
  const at penstock.rs:1744 is bench-subset-only, not an admission
  bound.

## Divergences

- None vs the lane doc, except: first `court-exit=0` reading was a
  shell pipe artifact (`$?` after `tail`), re-captured as 101 —
  recorded here rather than silently dropped.
