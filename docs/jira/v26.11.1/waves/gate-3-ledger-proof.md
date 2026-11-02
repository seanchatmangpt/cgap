# gate-3-ledger-proof

Status: **ALIVE** — composition 4-of-4 witnessed 2026-11-01 (R10; completion
section below). Prior PARTIAL record (3-of-4, pending the then-missing
`cgap_eval_bn3` seam) follows unchanged; GAP-1 landed the bodies and R10
landed the eval-hop composition court + its flip witness the same day.

## Witnessed this session (all exit 0, isolated target)

- `cargo test -p cgap-audit` → 37/0; 1M stress prints: powl verify=0.624s;
  ocel2 append=0.272s seal=0.613s open=0.783s, rows=1_000_000.
- `cargo test -p cgap-gate` → 11/0 (dam corpus 10M frames, 55.62s).
- `cargo test -p cgap-court a19_seal_1m_ledger_court` → green, 8.64s:
  real penstock ingress over 1M frames (zero-alloc asserted) → lane fold →
  commit via frozen CgapCommitParquetFn (StaticPageWriter/FooterSealer/
  Tier-H FileSink) → disk read-back verify_seal, twin-commit determinism,
  flip refusals, RLE lane identity from disk → cgap-crypto keyed receipt
  under a real Lease (tamper flips DigestMismatch) → p28 verify via frozen
  CgapVerifyP28Fn (8 intrinsic spoliation refusals).
- `cargo test -p cgap-bn3` → 111/0 (stage ALIVE incl. own 1M stress court).
- Zero-alloc: CountingAlloc self-courts 5/5, string-leak + alloc-free path
  courts, dam (0,0) per-frame across 10M, seal-court with_alloc_tracking
  is_zero asserted, bn3 hot-path zero-alloc court.

## The missing hop

`CgapEvalBn3Fn` (crates/cgap-ffi/src/sovereign.rs:174) is declared,
header-pinned (:555/:602-611/:865/:941), and UNIMPLEMENTED — no
`fn cgap_eval_bn3` in crates/ or tests/. The a19_seal composition imports no
cgap_bn3; bn3's 1M stress runs in a separate test module. admit → eval_bn3 →
commit is therefore not executable end-to-end today. Close condition: GAP-1
lands the cgap_eval_bn3 body, then a composition court gains the eval hop —
4-of-4 re-witness required before Gate 3 reads ALIVE.

## Corrections carried

- Dispatch path `crates/cgap-audit` wrong: package cgap-audit lives at
  crates/audit (inline src tests).
- `cgap_commit_parquet` also has no sovereign body; the seal court composes
  its own behind the frozen alias (tests/court_verdict.rs:2263) — GAP-1's
  brief covers this body too.

---

## GATE-3 COMPLETION (R10, 2026-11-01) — composition 4-of-4 WITNESSED

Standing: **ALIVE**. The missing bn3 hop is in the chain and courted.

- New named court `a19_seal_1m_eval_court` (tests/court_verdict.rs, mod
  a19_seal): sovereign admit (A14+A04, `CgapAdmitFrameFn`) over 1M lane-triple
  frames (zero-alloc sweep asserted, zero-copy view pinned) → sovereign eval
  (`CgapEvalBn3Fn`) closure over the ADMITTED views, 1M lanes out, per-row
  differential pin eval == harness fold → commit through the same frozen
  `CgapCommitParquetFn` 8-row-group body the existing court composes →
  twin-determinism + `verify_seal` from disk (footer rows == 1M) → body/
  footer flip refusals → lane identity from DISK vs the EVAL output → keyed
  receipt + tamper flip → `CgapVerifyP28Fn` green + 8 intrinsic spoliation
  refusals + digest pin + 79/81 B.
- `CARGO_TARGET_DIR=/tmp/r10-target CARGO_PROFILE_DEV_OPT_LEVEL=2 cargo test
  -p cgap-court --test court_verdict a19_seal_1m_eval_court -- --nocapture`
  → exit 0, 1 passed, 2.47s (witness line "SEAL+EVAL 1M LEDGER (4-of-4)").
- Full suite: same env, `cargo test -p cgap-court --test court_verdict` →
  exit 0, **14 passed / 0 failed**, 2.46s — the EXISTING
  `a19_seal_1m_ledger_court` un-weakened and green in the same run.
- Anti-vacuity (the flip the court cannot survive): /tmp sandbox crate
  (/tmp/r10-flipcrate) with the eval hop NEUTERED (sovereign eval call
  replaced by a stub returning CGAP_OK/count 0) → `cargo test --test
  gate3_flip a19_seal_1m_eval_court` → **exit 101** at the closure-count
  leg ("the lane rule matches exactly the one triple": left 0, right 1);
  shared-tree tests/court_verdict.rs sha256 8184d1e9…c9cb055c byte-identical
  before/after (mutation lived only in /tmp; nothing to restore in-tree).
- nm -gU over a fresh cdylib, this session first-hand (`cargo build -p
  cgap-ffi --lib` exit 0; `nm -gU /tmp/r10-target/debug/libcgap_ffi.dylib`
  exit 0): `_cgap_admit_frame` T @0x1e1c, `_cgap_eval_bn3` T @0x2630,
  `_cgap_commit_parquet` T @0x21b0, `_cgap_verify_p28` T @0x32e8.
- GAP-1 marker: docs/jira/v26.11.1/waves/gap-1-sovereign.md standing ALIVE
  (bounded poll, ~6.5 min, hit iteration 13 of the ≤30-min budget).
- In-leg mutations (each executed every run): M1 predicate flip → count 0;
  M2 o-word flip → projected lane tracks the admitted bytes; M3 reserved
  sigil → OABI; M4 cap 0 → LEASE; M5 digest flip → admit-refused OABI;
  M7 lane word > i32::MAX → SHACL; plus the disk body/footer flips and the
  receipt tamper.
