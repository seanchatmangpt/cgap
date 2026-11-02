# repair-05-production-wire — branchless production wiring receipt

Lane REP5. Repo /Users/sac/cgap, branch main. 429-terminated after
landing code; this receipt assembled by the coordinator from on-disk
verification (EX1 cross-checked independently).

## Landed state (verified on disk)

- cgap-branchless/src/penstock.rs: NEW production primitives —
  `penstock_prefilter8` / `penstock_prefilter8_mix` (exact shipped
  multiply-hash formula) and `swar_all_zero_vl` (variable-length
  zero-byte detect matching is_zero_at semantics), differential courts
  vs the shipped gate implementations (cargo test -p cgap-branchless:
  32 lib passed, coordinator witness 2026-11-02).
- cgap-gate/src/penstock.rs: S2/S5 delegate to
  `cgap_branchless::penstock::swar_all_zero_vl` (:651-660); S6/S7
  delegate to `penstock_prefilter8` (:676-683). Dep edge
  cgap-gate/Cargo.toml:16 (landed B7/W1).
- p28: NO dep, NO swap — B8 verdict stands (zero saturating surface;
  forbid-comment court). Vacuous-debt law honored.

## Verification (coordinator-run this session)

| cmd | exit |
|---|---|
| cargo test -p cgap-gate | 0 (12 unit + 3 l1 courts) |
| cargo test -p cgap-branchless | 0 (32 lib; census test file was REP1-mid-flight at check time) |

## Outstanding

- Bench re-run (chains-ingress ns/frame vs 10.65/11.73 baseline) is
  covered by H4 (verification lane, in flight).
- Census cells for the two new primitives: REP1's 16/16 grid predates
  them — census extension owed to the BR12 verification standard.

## Defect closure

FA6 S3 (dead library): CLOSED for swar_all_zero_vl +
penstock_prefilter8 (real production consumers, gate hot path).
Remaining unconsumed: saturated_subs_p28, poison_mask, swar_lt,
swar_splat, eytzinger machinery (BR12 dead-library law applies:
consume-then-census).
