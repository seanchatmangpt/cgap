# Lane PROMO-01 — microbenchmark register promotion (2026-11-02)

## W1 Subject
`docs/jira/v26.11.2/11-measured-claims.md` — appended §6.1 (header +
prose, 22 lines) and rows 39-40 to the consolidation addendum. Sole
file written. No git commands. Existing rows 1-38 untouched (append-
only; verified: my session delta contains no `-` lines).

## W2 Diff
- §6.1 "Microbenchmark consolidation addendum (lane PROMO-01,
  2026-11-02)" after row 38 in §6.
- Row 39: BN3 statement_scan_24b @65536, 82.24 GB/s dossier figure
  CONFIRMED-IN-BAND twice independently (F-06 80.49 GB/s -2.1%;
  FS-02 19.12 µs fastest = exact 82.24 reproduction; @1M 78.3 -0.8%).
- Row 40: P28 attestation verify @1024, ~460 ns dossier (459.96)
  CONFIRMED-IN-BAND twice (F-06 462.3 +0.51%; FS-02 458.0 -0.4%).
- Both rows carry exact-subject commands (`--bench bn3 --
  statement_scan_24b` / `--bench seal -- p28_attestation_verify`),
  both receipt cites, hot-ambient scope, class
  consolidation-addendum, and the NOT-chains-bn3-legs note (FS-02
  correction, bench-03-drift-check.md:198).

## W3 Commands + exits (this session)
All commands read-only inspection of receipt files (grep/sed) against
cited receipts; exits 0 each:
- grep cite verification: bench-02-seal-bn3.md (82.24/80.49/19.54
  µs/EXIT=0 legs :36, :83-85, :103, :118-119, :147) -> exit 0
- grep cite verification: bench-03-drift-check.md (19.12 µs exact /
  82.24 @65536 :107,129-135,159; 458.0 ns :109,138-141,160; FS-02
  subject correction :198) -> exit 0
- grep cite verification: bench-02-seal-bn3.md P28 section (:90-92,
  :105, :121-123, :148) -> exit 0
- awk line-width token sweep on final file -> exit 0
- git diff --stat + `grep '^-[^-]'` diff-audit -> exit 0 (append-only
  confirmed; the 4 `-` lines in the full HEAD diff pre-date this
  session — rows 5/8 quiet-host rescoping already on disk at lane
  start)

## W4 Verification ladder
Cite-level only (this lane registers measurements made by F-06 and
FS-02; it did not re-run benches). Every number and cite in rows 39-40
was re-read from the cited receipts on disk this session. Standing of
the promoted figures: MEASURED per the F-06/FS-02 receipts (both
witnessed `cargo bench` exit 0 in their own receipts).

## W5 Replay
Re-read `bench-validate/bench-02-seal-bn3.md` §3.4/§3.5 and
`bench-validate/bench-03-drift-check.md` §4.2/§5, then diff
`11-measured-claims.md` rows 39-40 against them.

## W6 Standing
ALIVE (register append witnessed on disk this session; every promoted
number cite-verified from its source receipt on disk this session).

## W7 Divergences
1. Dispatch named FS-02's second-witness receipt as
   `bench-02-drift-check.md`; that file is FS-01's chains-seal drift
   check and contains no statement_scan/attestation content (grep exit
   1 on all target strings). The actual FS-02 second-witness receipt
   is `bench-validate/bench-03-drift-check.md` — cited there instead.
2. Edit-process note: the first Edit replaced row 38's line prefix
   instead of inserting before it (merged row 38 into the row-40
   line); repaired in-session via a python reorder — row 38 is
   byte-identical to its pre-lane form (verified by eye against the
   session-start read) and now precedes §6.1. No other row touched.
3. Fence statement: existing rows 1-38 (incl. A-02's row 38 and the
   pre-session rows 5/8 rescoping) are must-not-regress surfaces; the
   append-only diff audit (W3) is the fence run. No gate consumes this
   doc programmatically in this lane's scope.
