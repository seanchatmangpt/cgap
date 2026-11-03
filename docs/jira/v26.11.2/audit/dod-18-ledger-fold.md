# dod-18-ledger-fold

DOD-18 final audit-ledger fold input (2026-11-02, read-only lane;
landed by coordinator). Census: ls lanes/ -> 143 entries (141
lane-*.md receipts + dispatch-ledger.jsonl + tc-11.alive). Zero
silent-death families. Double-registration check: new fleet ids
(gap/ax/posh/tc/tps/ipc/lspa/dx/fix/mv/dfx/dflss/dod/cgap-prd)
have 0 ledger hits — safe appends; six drained PENDING rows
(git-01, git-07, git-08, git-16, B02, C01) must REPLACE their
ledger PENDING rows, not duplicate.

## ADD-ROWS (one per landed lane; standing as receipted)

git: git-01 ALIVE (pack_demux_court 10/0); git-03 retry ALIVE
(ingress_ring 6/6 debug, 7/7 release); git-07 ALIVE (egress_ring
5/0); git-08 ALIVE (commit_synth_court 10/10, lib 61/61);
git-16 ALIVE (soc court 1/0, court 5/5). B: B02 ALIVE-courts
(court-flip exit 1; SLA honest FAIL stands: 42.4 s vs 25 ms).
C: C01 ALIVE-snapshot (llvm-cov baseline, no crate verdict).
AX: AX-0 gate ALIVE (docker GO local / cluster NO-GO / ax 127
NOT APPROVED); AX-1 image ALIVE; AX-2 manifests DRAFT-UNRUN;
AX-3 dylib ALIVE (in-image canonical path); AX-4 alloc ALIVE
(host legs, cluster DRAFT-UNRUN); AX-5 latency ALIVE (delta
table 1.7-1.8x); AX-6 shm ALIVE (transport-only); AX-7 mutation
ALIVE-host (container LOUD-BLOCKED); AX-8 seal ALIVE (SL-001..004,
SL-004 landed); AX-9 probe ALIVE (NO-GO by design, nothing
simulated); AX-10 index ALIVE. GAP: 01-12 all ALIVE (details per
receipts; GAP-06 PARTIAL: 90% floor honest red). IPC: 0/1/2/3/4/6/8
ALIVE (IPC-5/7 not dispatched; IPC-7 Cpk now landed via DOD-14).
LSPA: 1-6 ALIVE (LSPA-1 seal wiring + hang fix; LSPA-3 arms with
(i) red -> closed by DOD-15). POSH: 1-5 ALIVE. TC: 01-09,11,12,14
ALIVE. TPS: 3-6 ALIVE. DX: 1-8 ALIVE. FIX: 01-07 ALIVE. MV:
04/05/08 ALIVE. DFX: 1-6 ALIVE. DFLSS: DF-1..6 ALIVE. CGAP-PRD:
05/06/10 ALIVE. DOD: 01 (SPDX fix, 79/79), 02 (branchless flip,
NO-GOs->GO), 03 (arrow nested-types default, 199/0), 04
(ref_store width-gated atomics), 05 (core no_std migration,
95->0), 06 (61 pins unified + lock regen; package verify BLOCKED
pre-push by registry resolution — expected), 07 (authors/desc
manifests), 08 (README rewrites), 09 (docsrs blocks + gpu
doc-cfg), 10 (claims rewording), 11 (R10 all children flip),
12 (G2 decision package), 13 (AX runner/Dockerfile fixes),
14 (N=30 Cpk: decided Cpu 17.252 PASS, aspirational -45.762
loud, jitter flip witnessed), 16 (compendium verify, DOD-10 owns
landing), 17 (Station 300 PARTIAL), 19 (ship-gate final spec),
20 (register refresh).

## Known-blockers staleness (coordinator updates)

1. B05 C6 honest FAIL — re-scoped by MV-5 (150M decided).
2. C05 parquet default-features 101 — fixed by FIX-02.
3. C06 90% coverage floor honest RED — persists (by design).
4. B08 zero-tax cannot-flip — made deterministic (GAP-04 + DOD-11).
5. G2-PARETO OPEN — measured + decision package landed (DOD-12).

## See Also

audit/audit-ledger.md · audit/tc-10-open-recensus.md ·
audit/dod-19-shipgate-final.md
