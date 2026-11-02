# Receipt: V2-03 LEVERS (Pareto lever engineering specs)

Lane: V2-LEVERS · 2026-11-01 · repo /Users/sac/cgap (canonical checkout)
Charter: CHARTER-CGAP-STARDF-v26.11.2

## Subject

- `docs/jira/v26.11.2/02-pareto-levers.md` (new) — five lever specs
  (L1 ingress, L2 parquet, L3 bn3, L4 sovereign, L5 seal), each with
  measured receipt coordinate, bottleneck share, root cause at
  file:line, fix, target, and named anti-vacuity mutation.
- `docs/jira/v26.11.2/waves/v2-03-levers.md` (this receipt).

## Grounding (reads this session)

- All five v26.11.1 receipts read in full:
  `bench-chains-{ingress,parquet,bn3,sovereign,seal}.md`
  (docs/jira/v26.11.1/waves/), each ALIVE (exit 0 on 2026-11-01).
- Root causes verified on disk:
  - `crates/cgap-parquet/src/footer.rs:119-127` — byte-wise
    `fnv1a64_seeded` loop (read this session).
  - `crates/cgap-ffi/src/sovereign.rs:706-730` — `File::create` at
    line 722, boxed FileSink scratch, `Err(_) ->
    CGAP_ERR_LEASE_EXHAUSTED` paths (read this session).
  - `crates/cgap-bn3/src/unify.rs:11,35-39,274,556,577,609` —
    nested-loop join documented as the v1 spec-3.2 strategy (read
    this session).
- Penstock: `crates/cgap-gate/src/penstock.rs` exists; measured
  penstock admission 108.5 Mitem/s from the P3-16 receipt. The
  5.70 ns/frame gate figure is dispatch-sourced (TARGET, not
  measured here).

## Divergences

1. Sovereign line numbers: dispatch named 706-724; on-disk the
   `File::create` call site is at line 722 inside a block spanning
   ~706-730. Spec cites 706-730 with the call site pinned to 722.
2. No file named CHARTER-CGAP-STARDF-v26.11.2 exists under
   /Users/sac/cgap/docs (searched: DFLSS_CHARTER.md,
   phase-change-charter.md, bench-campaign-charter.md,
   18-dflss-charter.md matched; no STARDF charter). Cited as the
   dispatch authority per lease; charter file itself UNVERIFIED.
3. The 65,376-lane lease constant is not spelled in
   sovereign.rs source; it is the P3-19 measured
   LEASE_EXHAUSTED boundary (receipt-sourced), so the spec cites
   the receipt, not a source line.
4. M2b finding carried into L2: the truncation mutation SURVIVED in
   P3-17 (decoder is page-header span-bounded); the L2 court keeps
   the byte-flip mutation as its flip class.

## Standing

PARTIAL — both leased files written and verified on disk this
session; specs are grounded in receipts read this session. The
targets remain TARGETs: no v26.11.2 bench has run (this lane's
lease is spec-writing only, no code edits, no benches executed for
the fixes). Next lanes: L1-L5 fix implementation + re-measure with
the stated divan method.
