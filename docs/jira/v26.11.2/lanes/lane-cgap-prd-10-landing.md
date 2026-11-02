# Lane CGAP-PRD-10 (final PRD lane) — landing receipt

2026-11-02. Repo /Users/sac/cgap. No git commands run.

## W1 Identity & owned files

- docs/jira/v26.11.2/audit/cgapprd-prd-v2.md (272 lines — the
  receipt-true PRD/ARD v2, superseding 12-ard-prd-cgap.md per the
  landed cgapprd-09 plan)
- docs/jira/v26.11.2/lanes/lane-cgap-prd-10-landing.md (this receipt)
- docs/jira/v26.11.2/audit/doctrine-ship-plan.md — APPEND-ONLY P9 row
  inserted into the ship-gate table directly after the P7 row; zero
  pre-existing lines modified (relocated from a first-pass EOF append
  within this session, before any other lane could read it).

## W2 What landed (per cgapprd-09 sequencing)

1. SS1.1 equation rewrite: R2's exact typed-trailer block verbatim;
   'mod Σ_PQC' deleted outright (zero .rs referent, R2 grep 2).
2. SS3.2 NFR table rewritten per R1: <15 ns and <4 us rows marked
   ASPIRATIONAL-UNVERIFIED (unmeasured); materialization row carries
   the HONEST FAIL (23.35 ns/fact median, 2.3x, withdrawn 10 ns
   figure); SRAM 36,048 B VERIFIED / Flash+stack ASPIRATIONAL;
   scoped zero-alloc row with the 2,662-alloc fence; uncontended
   caveat on rate claims (lane-b06:24 vs :52-54).
3. SS6 :156 rewrite: sub-15 ns / sub-4 us are TARGETS; the landed
   courts are the shippable grounds.
4. P2 Quad64 clarify sentence (24 B fact frame vs layout.rs:75
   QUAD_SIZE=64) and SS4.3 11-symbol C-ABI caveat + explicit
   no-AshA2A-wiring note.
5. Deletions register (SS7): Σ_PQC-as-fact, sub-ms checkpointing,
   <2 ms closure gate — each with receipts.
6. Aspirational register (SS8): every aspirational row carries
   ASPIRATIONAL-UNVERIFIED + named unblock prerequisites (incl. the
   AshA2A edge: bidirectionally absent per cgapprd-08b W2/W4).

## W3 Commands + exits (this session)

- PRD Verify (the falsifier, run as a referent-grep battery):
  in-tree greps for KEY_LEASE_RECEIPT / lease-receipt trailer,
  RADIX_BUCKETS=4096, FRAME_BYTES=24, QUAD_SIZE=64,
  INGRESS_RING_BYTES=32_768, FACT_WINDOW, cgap_admit_frame, blake3
  in chatman.rs; zero .rs hits for Sigma_PQC; line-exact checks
  lane-ax-06-shm.md:70, lane-posh-02-batch.md:43,111, lane-b06:24,
  lane-posh-04-sentry.md:37, lane-git-12:60, lane-b02:32,
  lane-mv-05:66; verify_cscb_sram.py + verify_disassembly_invariants.py
  present; Hilt.WorkOrder identity_digest in ~/ash_a2a.
  Result: PRD_VERIFY_FAIL=0 (exit 0). The PRD Verify passes iff
  every quantitative claim carries a receipt file:line and zero
  claims use a referent that fails an in-tree grep — passed.
- wc -l checks after each chunk append (272-line final file,
  on-disk bytes verified).

## W4 Falsifier

Verbatim: 'The PRD Verify passes iff every quantitative claim
carries a receipt file:line and zero claims use a referent that
fails an in-tree grep.' Ran this session; exit 0. The load-bearing
fact: the corrected PRD exists on disk and passes.

## W5 Fence law (must-not-regress surfaces)

- 12-ard-prd-cgap.md left untouched (no regress surface; v2
  supersedes by header, not by edit).
- doctrine-ship-plan.md table rows P1-P7 unmodified (append-only
  honored; verified by the relocation being insert-only).
- No shared compile surface touched — no cargo gate applies; the
  lane's gate IS the PRD Verify, exit 0 above.

## W6 Standing

ALIVE — commands + exits run this session (W3). Perf numbers
carried into the PRD are receipt-cited measurements or explicitly
ASPIRATIONAL-UNVERIFIED TARGETs; nothing rounds up.

## W7 Divergences

1. R9's citation paths name lanes like lane-g01:3, lane-posh-02:43
   without the v26.11.2/ prefix and cite lane-g02/b02/git-12/mv-05
   receipts that live in v26.11.2/lanes (not v26.11.1 as R9's
   sibling docs assumed) — re-pointed at the verified on-disk paths;
   every re-pointed line checked line-exact this session.
2. P9 row first landed at EOF (after See Also) then relocated
   insert-only to its correct table position; no window existed in
   which another lane read the misplaced row (same-session fix).
