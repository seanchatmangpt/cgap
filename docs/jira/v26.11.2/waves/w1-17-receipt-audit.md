# w1-17 — waves/ receipt audit (court-auditor verdict, filed by coordinator)

The auditor delivered its full verdict table in its final report; the receipt file was not
written. This file lands the verdict (content preserved, classification unchanged) so the
non-vacuity metagate can cite it. Evidence = the auditor's own commands+exits (40+ spot
checks, sha256 archive reproductions, the D1 flip).

## Verdict summary

- 29 receipts audited: majority WITNESSED; PARTIAL for config/doc artifacts; stardf-tree
  claims UNVERIFIED by operator ban (not FAIL).
- Perf-claim discipline: no violations (v2-01 marks numbers [TARGET-v26.11.2]; v2-03 marks
  the 5.70 ns figure dispatch-sourced).
- D1 (real, court-confirmed): .claude/hooks/guard-generated.sh blocks only relative paths —
  flip witnessed (rel=exit 2, abs=exit 0, malformed=exit 2). Vacuous on absolute file_path
  inputs. OPEN — repair owed.
- D2: bp-10-audit G2 settings.local.json claim unreproducible (grep exit 1; mtime ordering
  recorded). ADDENDUM LANDED by W2-19.
- D3: v2-12 marker count 11 on disk, not 12. ADDENDUM LANDED.
- D4: v2-13 bench count 15, not 14 (v26_10_31.rs). ADDENDUM LANDED.
- D5: w1-02 digests /tmp-parked; W2-19 re-ran the regen ×2 and embedded the digest table.
  CLOSED.
- D7: sm-4-hooks exit-code claims (2 vs 3) unrepaired per sm-5. OPEN-defect recorded.

## Court

{"court":"D1 guard-generated.sh abs-path vacuity flip","result":"FLIPPED (rel=2 abs=0)"}
{"court":"sha256 archive reproductions (bp-1, sm-1)","result":"byte-exact"}

## Footer

{"lane":"W1-17","files_written":["docs/jira/v26.11.2/waves/w1-17-receipt-audit.md"],
"commands":["40+ spot-checks, sha256 reproductions, flips (auditor session)"],
"courts_run":["D1 flip"],"standing":"ALIVE",
"divergences":["filed coordinator-side; original lane wrote no file",
"stardf receipts UNVERIFIED by operator ban"]}
