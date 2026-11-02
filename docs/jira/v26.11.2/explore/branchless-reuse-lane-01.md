# BR1 — Branchless reuse map: cgap-gate (penstock.rs)

Subject: `crates/cgap-gate/src/penstock.rs` (1871 lines; only gate src of note —
no ingress.rs/header.rs exist; hot path is `ingress` :515-618, `prefilter8`
:677-686, `is_zero_at` :651-660). Status: all gains [ANALYTIC] unless cited.
Primitives inventoried from `crates/cgap-branchless/src/` (select, swar,
arithmetic, eytzinger, radix) + the two REP5 fns named in dispatch
(`penstock_prefilter8`, variable-length zero-detect).

Divergence recorded: no REP5 doc found in the tree
(grep REP5 over docs/jira/v26.11.1+v26.11.2 = 0 hits); reconciled from the
dispatch description only. cgap-branchless is not yet consumed by cgap-gate
(grep branchless in gate src = 0 hits).

## Hot-path substitution sites

| # | Site | Current logic | Target primitive | Expected gain | Anti-vacuity court |
|---|---|---|---|---|---|
| S1 | :536 `bytes[OFF_MAGIC..OFF_MAGIC+4] != MAGIC` + :541 version | two early-exit compares | `swar_eq_u64` over bytes 0..8 (magic+version+flags in one word) folds rows 1-2 into one SWAR eq against `b"CGAP" \| version<<32` — but version sits at byte 4, so eq target must splat; keep per-row codes via post-hoc mask decode | [ANALYTIC] 2 compares+2 branches → 1 load+1 cmp; the big win is with S6's single-load fold | Mutant A: /tmp sandbox copy of penstock.rs with the swar fold's target constant corrupted — admission court must exit 101; sha256-exact restore + mtime bump |
| S2 | :550-558 digest all-zero scan (8-byte early-exit loop) | hand-rolled byte loop | `swar_has_zero_byte`-family: OR-fold 4x u64 loads, `== 0` — REP5's variable-length zero-detect (`variable_zero_detect`) exact fit; replaces `is_zero_at` :651-660 too (S5) | [ANALYTIC] early-exit loop → 5 ops fixed-time; also removes the data-dependent early exit the doc itself notes is the fast path | Mutant B: sandbox penstock.rs, invert the zero-detect fold (`== 0` → `!= 0`) — zero-digest reject court (row 3a) must flip to exit 101 |
| S3 | :566 `declared > (len - HEADER_LEN) as u64` | single compare, already branchless-cheap | none needed — keep; only `saturated_subs_p28`-adjacent note: no sub-overflow path | ~0; do NOT churn | n/a — flag as leave-alone |
| S4 | :573 attestation magic compare | slice eq, 4 bytes | `swar_eq_u64` shared with S1's load if attestation word loaded as u64 at `a..a+8` (magic+version fused) | [ANALYTIC] fuses rows 4-5's first two compares into the same fixed-time unit as S1/S6 | Mutant C: corrupt the fused attestation target word — SPOLIATION court exit 101 |
| S5 | :597 `is_zero_at(bytes, a+48, 32)` receipt-zero check | 32-iteration early-exit loop | same REP5 variable-length zero-detect as S2 | [ANALYTIC] fixed-time 32B scan; removes last data-dependent loop on the hot admit path | covered by Mutant B court (same fn); if REP5 lands two fns, one court per fn |
| S6 | :614-615 `prefilter8` + `escalate` bit | own byte-loop load + own mix | REP5 `penstock_prefilter8` (drop-in; identical semantics per dispatch) | measured-by-REP5's receipt, else [ANALYTIC] | Mutant D: corrupt the mix constant `0x2544_5491_4F6C_DD1D` — prefilter court must exit 101 (proves REP5 fn is the live callee, not the local) |
| S7 | :615 `u16::from(bytes[OFF_FLAGS]) & ESCALATE_MASK != 0` | already branchless (mask+test) | already the `poison_mask` class; keep | 0 | covered by Mutant D |
| S8 | refusal handling (`if len > MAX` REFUSE→poison_mask class) | NOT PRESENT in gate — penstock has no quota/refusal-mask path; nearest analogue is row-1 geometry `if len < 128` :524 / `len > bytes.len()` :530 | `arithmetic::poison_mask(quota)` applies to the CALLER (dispatch's "if len>MAX REFUSE" site lives in cgap-ffi/sovereign or transport, not penstock). Flag: BR1 scope is penstock.rs; the FFI refusal-mask site is out of lane but named here for the coordinator | out of lane; [ANALYTIC] cmov-class gain at FFI ingress | out of lane — court belongs to the FFI owner; loud [SKIP] from BR1: "poison-mask site not in penstock.rs" |
| S9 | unaligned-frame checks | none — penstock reads via byte loops, not unaligned loads; the "unaligned" concern dissolves once S1/S2/S4 use word loads (`u64::from_le_bytes` of a stack array is alignment-safe on all targets incl. 32-bit; this is the read_u64_le codegen note :620-626) | REP5 zero-detect + swar eq must preserve the byte-loop→word-load bounds-check elimination property or the 6.3ns-vs-4.9ns regression documented at :624-626 recurs | [ANALYTIC] neutral if byte-loop loads kept; regression risk if slice helpers return | Mutant E: swap REP5 word-load to `bytes[off..off+8].try_into().unwrap()` slice form — frame-perf court (if any) must regress; if no perf court exists, exit 101 on the row-2 court after the form change |

## MUST STAY BRANCHY (hot-path-only discipline)

- :524/:530/:536/:541/:559/:566/:573/:577/:582/:587/:591/:598 all `return
  CGAP_ERR_*` sites — typed-refusal cold paths; branches are correct here and
  predicted-not-taken on the admit fast path. Branchless-ifying them trades a
  cold branch for code-size and review risk. Keep branchy.
- :606 `if let Some(view)` — API-shape branch, once per frame, fine.
- Courts module (:700+) — test-only, out of scope.

## Reconciliation with REP5

- S2+S5 → REP5 variable-length zero-detect (direct replacement of
  `is_zero_at` and the inline digest scan).
- S6 → REP5 `penstock_prefilter8` (drop-in; Mutant D proves the swap live).
- S1/S4 swar_eq_u64 fusions are NOT in REP5's named surface — they are BR1
  proposals, gated on REP5's landing (shared word-load helper).
- Overlap risk: if REP5 swaps S6 before BR1 lands, Mutant D still applies
  unchanged (corrupt REP5's constant in the sandbox — exit 101 proves the
  callee identity). No conflict.

## Standing

UNVERIFIED — read-only lane; every gain is [ANALYTIC]; no builds run this
session. Mutant courts A-E are named, not run (BR1 writes docs only; the
implementing lane owns the flip runs). Perf numbers are TARGETs.
