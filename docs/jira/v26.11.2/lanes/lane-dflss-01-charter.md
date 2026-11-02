# Lane DF-1 charter receipt (2026-11-02)

## W1 Subject

- `docs/jira/v26.11.2/dflss/01-project-charter.md` (new, DMADV charter,
  lane scope: cgap-core/bn3/branchless/git/blake3/storage/csct)
- `docs/jira/v26.11.2/dflss/02-voc-ctq-matrix.md` (new, CTQ-01..07
  per-cell register; canonical for these seven cells, cross-linked to
  `14-dflss-charter.md` + `15-ctq-tree.md` which stay canonical for
  their existing scope)
- `docs/jira/v26.11.2/lanes/lane-dflss-01-charter.md` (this receipt)

## W2 Evidence (fresh commands + exits, this session)

- `python3 scripts/verify_disassembly_invariants.py` -> exit 0
  (28-fn census, 3 ALLOWED-RESIDUAL RV32 ISA-ceiling rows)
- `python3 scripts/verify_cscb_sram.py` -> exit 0 (36,048 B)
- `cd crates/csct && CARGO_TARGET_DIR=/tmp/dflss1-target cargo build
  --release --bins` -> exit 0
- `cscb_c4_alloc` -> exit 0 clean ("0 allocs / 0 bytes over 1000000
  frames"); `CSCB_MUTATE=alloc` -> exit 1 breach flip
- `cscb_c1_c2_ingress` -> exit 0 (C1 205,407,348 f/s; C2 p99 5.00 ns,
  batch64-amortized method disclosed in [META] line)
- `cscb_c7_receipt_sealing` x2 -> exit 0 (median 3,894,347/s and
  3,891,452/s, min/median/max labels, runs=7)
- `cscb_c3_closure` -> exit 0 (DECIDED GATE PASS min=787.512 ms,
  batches=1, quiet host; ASPIRATIONAL 25 ms loudly NOT met, 32x)
- Line-length + H1 gates on both written files -> pass (0 over-100)

## W3 Per-CTQ standing

| CTQ | Status | Cell cite (02-voc-ctq-matrix.md) |
|-----|--------|----------------------------------|
| 01 zero-alloc | VERIFIED | CTQ-01 section |
| 02 <=10ns | VERIFIED-COND (batch64 amortized) | CTQ-02 section |
| 03 SRAM | VERIFIED (36,048 B; 39,184 report-only) | CTQ-03 section |
| 04 seal rate | VERIFIED (median ~3.89M/s) | CTQ-04 section |
| 05 closure | GATE VERIFIED / 25 ms ASPIRATIONAL-UNVERIFIED | CTQ-05 section |
| 06 branches | VERIFIED (3 documented residuals) | CTQ-06 section |
| 07 >=350x | VERIFIED by receipt; e2e re-run UNVERIFIED-this-session | CTQ-07 section |

## W4 Divergences from dispatch

1. Dispatch named `14-dflss.md` / `15-ctq.md`; on disk they are
   `14-dflss-charter.md` / `15-ctq-tree.md`. Extended by cross-link
   only (those files not owned, untouched).
2. CTQ-02 dispatch said "cscb_c1_c2_ingress.rs may still mislabel max
   as p99" - FIX-04's sibling audit (lane-fix04-cscb-p99.md:31-35)
   found the file HONEST (genuine p99 over thousands of batch-amortized
   samples); no mislabel. Residual recorded instead: stale doc comment
   "batch of 8" vs BATCH=64 const (cscb_c1_c2_ingress.rs:23 vs :50).
   Real remaining p99 defect: cscb_c6_retraction.rs:87-100 (FIX-04
   sibling audit :36-38) - owner should rename to min/median/max.
3. CTQ-05 dispatch expected ~22-25 ms-class evidence; the DECIDED gate
   on disk is 60,000 ms (MV-5 re-scope) and fresh run gave min 787.5 ms
   on a quiet host. 25 ms aspiration unmet and reported loudly in the
   gate itself. No lane-mv-5 receipt exists - provenance gap recorded.
4. CTQ-07 REQUIRED_ROI_X is at lib.rs:35 (dispatch said :30-35; ruling
   comment spans :30-34 - close enough, cited precisely).

## W5 Fences

Could not regress: 14-dflss-charter.md and 15-ctq-tree.md are untouched
(read-only for this lane; canonical pointers added from dflss/ files
instead). Gates re-run above are read-only scripts + /tmp-target bins;
shared tree built once via csct bins only (no lib.rs/Cargo.toml edits).
No mtime-sensitive restore performed (no shared-tree mutation).

## W6 Skip ledger

- csct full 6-stage suite (CTQ-07 end-to-end re-run): [SKIP] long-suite
  cost; VERIFIED by receipt instead (lane-b07/lane-g11/lane-g15), e2e
  re-run tagged UNVERIFIED-this-session in the matrix cell. Loud, not
  silent.
- cscb_c6 retraction bench (sibling p99 defect): not owned, not run;
  recorded as owner-action in divergence 2.

## W7 Standing

ALIVE. Every VERIFIED cell above has a fresh command + exit this
session against the exact subject; receipt-only cells are labeled.
Build root /tmp/dflss1-target deleted at lane end.
