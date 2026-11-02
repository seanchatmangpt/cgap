# doctrine-swarm-and-isel

DOCTRINE-1 audit (2026-11-02). Narrative = hypothesis; VERIFIED =
re-derived from a session command or receipt bytes checked.

## Pillar 1 — swarm law

(a) Disclosure-in-both: G04 (:58-68) + G07 (:13,20) both name their
counterparty — VERIFIED. C05/FIX-07: FIX-07 (:48-54) attributes two
excised used bindings + unused_mut to C05 and demands an amendment;
C05's receipt has ZERO FIX-07 mentions — **DEFECT-OPEN, amendment not
landed**. SLA events: 9 lane receipts mention the SLA; 8 record
applied/received unblocks; C05 explicitly applied none.

(b) Anti-vacuity: AE4 fresh run exit 0 with three mutant arms held
(M-AE4-1 census flip, branchy self-check, M-G03-E binary patch).
Sweep of 65 lane receipts: 40 hit flip/mutation terms. Honestly named
assert-only courts: **B08 zero-tax cannot flip** (self-reported FAIL);
B02-C3's VACUOUS-COURT arm implemented but not witnessed. ~24 receipts
cite runnable commands without inline "witnessed" language.

(c) Round-up spot-check (7 receipts): no round-up found — ALIVE claims
scoped or caveated; UNVERIFIED lines held honestly (B02, B04, G05, G07,
FIX-03, C02). Two DEFECT-OPEN provenance items: B08's flip-run
baseline lives at /tmp (ephemeral, not re-derivable).

## Pillar 2 — instruction courts

Fresh AE4 run: **exit 0**, verdict CLEAN (census + linked witness +
mutant + self-check + binary mutant all held); 23 PASS rows +
3 ALLOWED-RESIDUAL. All three formerly-FORBIDDEN fns branchless
(count_leading_eq_u8 beqz; prefix_max_u8x8 bgeu+bltu;
transpose_64x64 bne — G01:38-56, G02:43-45). ISA-ceiling residuals
documented: cmov_u64 1x bnez (RV32 has no cmov), eytzinger_restore
1x beqz, select_slice 2.

aarch64: AE4 remains riscv-only in-tree. scripts/verify_lsp_branchless.py
EXISTS (LSPA-6) and runs CLEAN this session (exit 0): clean select.rs ->
n_refused=0 PASS; M-AE4-1 mutant -> n_refused=0 (aarch64 -O lowers to
csel — the AE4 mutant does NOT flip on aarch64, honest nuance);
portable mutant (black_box forward branch) -> n_refused=3 PASS.
**GAP: no LSPA-6 receipt exists** (grep across docs/ = empty) — the
aarch64 court is landed tooling with no paper trail:
ASPIRATIONAL-UNVERIFIED at the documentation layer.

## Weakest links (pillars 1-2)

1. C05 receipt amendment owed, not landed (disclose-in-both violated
   in the C05 direction).
2. LSPA-6 aarch64 court: script VERIFIED, receipt MISSING.
3. B08 flip court assert-only by its own law + ephemeral /tmp receipt
   path.

## See Also

audit/doctrine-ship-plan.md · lanes/lane-g01-scan-branchless.md ·
lanes/lane-g02-transpose-branchless.md · lanes/lane-fix07-bench-build.md
