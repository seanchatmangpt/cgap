# Lane C13 receipt — G7 patent gate final consistency + FA11 remediation

Date: 2026-11-02. Repo /Users/sac/cgap, branch main. No git writes.
Shared tree untouched by any mutation; all mutations in /tmp/c13.

## 1. Marker greps (grep-verified before adding)

- `grep -n "0x4347_0006" crates/cgap-ffi/src/sovereign.rs` -> hits at
  sovereign.rs:87 and :1785; `CGAP_ERR_CONCURRENT_WRITER_DENIED` 8 hits
  in sovereign.rs, 2 in fa9_commit_gating.rs (exit 0 both).
- `grep -n "CGAP_ERR_CONCURRENT_WRITER_DENIED\|0x4347_0006"
  crates/cgap-ffi/tests/fa9_commit_gating.rs` -> :32, :34, :66, :214
  (exit 0). Markers real, added to ROW_EVIDENCE row 1 as `extra_cites`
  (sovereign.rs: CGAP_ERR_CONCURRENT_WRITER_DENIED, 0x4347_0006;
  fa9_commit_gating.rs: both markers), each keyed by cite path in the
  report JSON.
- Register doc row-1 landed-cite list + Standing paragraph updated
  (sovereign.rs:87 0x4347_0006, spill_gate pre-IO denial
  sovereign.rs:758-764, race court fa9_commit_gating.rs 226 lines).

## 2. Gate runs x2 (stable)

- `python3 scripts/verify_ost_design_around.py --report /tmp/c13/report_run1.json`
  -> G7-PATENT: PASS 4/4 DISTINCT-MECHANISM, exit 0.
- Same command with `report_run2.json` -> 4/4, exit 0. Stable.
- Report JSON row-1 `evidence_markers` now includes 7 keys:
  mmap_writer.rs:{RingSink, drain_to_spill, RunOverrun},
  sovereign.rs:{CGAP_ERR_CONCURRENT_WRITER_DENIED, 0x4347_0006},
  fa9_commit_gating.rs:{CGAP_ERR_CONCURRENT_WRITER_DENIED, 0x4347_0006}.
  A missing marker or cite now reads PENDING (exit 1).

## 3. Anti-vacuity flip (merged duplicate, /tmp)

- Mutation: full row-2 (US11,216,456) mechanism field := row-1's full
  mechanism text, /tmp/c13/flip/claims.txt.
- First two attempts did NOT flip (exit 0) — first-line-only mechanism
  swap; the full normalized multi-line mechanism text still differed,
  so those runs are recorded as vacuous harness runs, not verdicts.
  Court requires full-field duplicate.
- `python3 scripts/verify_ost_design_around.py --claims
  /tmp/c13/flip/claims.txt --report /tmp/c13/flip/report.json`
  -> rows 1+2 NOT-DISTINCT (duplicate/near-duplicate mechanism shared
  across rows ['US10,817,467','US11,216,456']), 2/4, FAIL, exit 1.
  Flip confirmed.

## 4. FA11 M2 residual (ZZQQ element-text corruption)

- Mutation: row-1 element -> "completely fake element text ZZQQ",
  /tmp/c13/m2/claims.txt (single-row replacement; an earlier buggy
  harness emitted row 1 twice and exit 1 — harness defect, not a
  gate verdict, disclosed here).
- `python3 scripts/verify_ost_design_around.py --claims
  /tmp/c13/m2/claims.txt --report /tmp/c13/m2/report.json`
  -> 4/4 DISTINCT, exit 0.
- Recorded residual (OPEN limitation, not fixed by this lane): the
  gate is a register-consistency gate. The element column remains
  self-authored — no patent text exists on disk to compare against,
  so element wording is unfalsifiable inside the gate and ZZQQ-class
  corruption still passes. The FA9FIX enforcement (0x4347_0006,
  pre-IO denial, race court) is now mechanically enforced and
  gate-marked, but the patent-text grounding gap stays OPEN.
- First M2 run in this session (before the single-row fix) returned
  exit 1 for the wrong reason (duplicate-row harness artifact); the
  clean run above is the M2 witness.

## Standing

ALIVE. Gate stable 4/4 exit 0 x2 with the new row-1 markers active;
flip court exits 1; M2 residual re-confirmed and disclosed as a named
limitation.
