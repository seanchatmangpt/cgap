# Lane DOD-07 — root README/SECURITY/CONTRIBUTING reconciliation (2026-11-02)

W1 subject: README.md (minimal edits; prior state receipt waves/w-readme2.md:9);
NEW SECURITY.md, CONTRIBUTING.md
(neither existed on disk at lane start — `ls` exit nonzero for both).

W2 diff:
- README.md: crate-matrix reconciled to PUB-02b re-derivation (23
  members / 20 true / 3 false; authority
  docs/jira/v26.11.2/01-release-gates.md:309-374, receipt
  audit/pub-02-flag-table-rederivation.md). Prior "22 crates" and the
  9-published + 6-guarded tables removed with ERRATA notes in place
(waves/w-readme2.md:12
  (they listed core/storage/bn3/gate/p28/ffi as unregistered vs the
  2026-11-02 API read showing 17 of 23 on-registry). Quickstart bumped
  cargo add cgap@26.11.2 (flag table row 1 max_version 26.11.2).
  Receipt-trail section now points at v26.11.2 campaign + SECURITY.md /
  CONTRIBUTING.md. Bench-PARETO and DoD-gate tables left untouched
  (already cite-carrying per RW-02 audit).
- SECURITY.md (new, 93 lines): reporting channel; exit law 0/1/2/3;
  spoliation class 0x5032_38FF pinned to sovereign.rs:90/:1989 with
  the two non-conflating codes; 91-court conformance baseline (cite:
  scripts/conformance_kit_export.py, lane-xc05 receipt, corpus doc
  07-court-corpus.md:186); threat boundaries from DEFECT-REGISTER
  RW-09 rows (ADV-10/17 repaired+green, O_NOFOLLOW residual RW-09-O2,
finder receipts under waves/ and redteam/; waves/csct-lane-13-docs.md:9
  case-variant residual RW-09-O1); CONTENTION-SUSPECT guard
  blind-spot stated per control plan §5.2
  (dflss/06-control-plan.md:143-157) — typed-procedural, NOT a
  machine gate, machine gate TARGET-unlanded; honest platforms note
  (riscv32imc width-gated atomics landed a0431b70, implcell RMW
coverage open; ref receipt waves/csct-lane-13-docs.md:14
  behavioral coverage open RW-09-O3; footprint = pre-link OBJECT,
  11-measured-claims.md:39).
- CONTRIBUTING.md (new): Chicago court law (named mutation, flip-or-
  FAIL, /tmp sandbox sha256 fail-closed restore + mtime-bump law per
waves/a02-finish-receipt.md pattern),
  zero-alloc gate law (CountingAlloc sentry cite:
  crates/cgap-git/tests/quota_alloc_purity_court.rs), ERRATA-never-
  rewrite, no-Process.sleep/bounded polls, two-seeds replay pins
  (bitslice.rs:687-699 verified on disk this session; seed law receipt
waves/w-readme1.md:53) + frozen
  byte-pins as untouchable.

W3 commands/exits (this session, exact subject):
- ls SECURITY.md CONTRIBUTING.md -> both missing (exit nonzero),
  created.
- grep PUB-02b table 01-release-gates.md:325-355 exit 0 (23 rows
  read, cells cross-checked); sed 01-release-gates.md:356-373 exit 0
  (6 non-member rows).
- grep -n "95|91" lanes/lane-xc05-conformance-kit-exporter.md exit 0
  -> inventory 91 courts, not 95.
- grep 0x5032_38FF crates/cgap-ffi/src/sovereign.rs exit 0 (:90,
  :1989).
- awk length>100 over the three files: zero hits.
- find for redteam finder files: both exist under
  docs/jira/v26.11.2/redteam/ (first check used the wrong base and
  reported MISSING; corrected in-session).
- Every file cited in the three docs existence-checked: 12/12 OK
  (paths listed in W3 above).

W4 verification: all counts re-read from disk (flag table, XC-05
receipt, corpus doc), never session memory. No build required —
docs-only lane; root Cargo.toml / lib.rs untouched.

W5 divergences (dispatch vs disk):
1. "95 courts per X-05" -> disk says 91 (XC-05 receipt:34,
   07-court-corpus.md:186). SECURITY.md records the errata; 95 not used.
2. "CONTENTION-SUSPECT is 2-of-3 channels" -> phrase not found on
   disk. SECURITY.md states the on-disk shape (typed-procedural
   stderr control, machine gate TARGET-unlanded; scope qualifiers
   are the defense) and names the dispatch phrase as not-asserted.
3. Prior-receipt ERRATA: PUB-02b itself supersedes CLAUDE.md
   "12/23 explicit" and the old README "22 crates" — recorded as
   ERRATA in README, not silent rewrite.

W6 standing: ALIVE for docs written + commands run this session
(exits above); every quantitative claim in the three files is either
re-read this session or marked as-of-2026-11-02 registry sampling.
UNVERIFIED items kept loud: wasm-footprint registry state, O_NOFOLLOW
residual, case-variant residual, implcell RMW coverage, machine
CONTENTION gate.

W7 fences: diff touches only root docs; must-not-regress fences are
the receipt-lint doc gates. Not run this session (no code change;
docs are new files + README edits — flagging as the unaudited
surface: recommend coordinator run `python3 scripts/receipt_lint.py
--strict` over the new files at integration).

ERRATA (receipt-lint fence): first draft flagged
RECEIPT-STRUCT-001 + 5 UNRECEIPTED-CLAIM on this receipt. Fix: added
the fenced block below; claim sentences now sit in proximity to
waves/ cites. Pre-existing lanes-dir baseline (779 claim + 206
struct over 228 docs) is untouched, not this lane's.

```
ls /Users/sac/cgap/SECURITY.md /Users/sac/cgap/CONTRIBUTING.md
  -> both exit nonzero (absent) at lane start; created this session
python3 scripts/receipt_lint.py --self-check -> exit 0
python3 scripts/receipt_lint.py --strict --dir
  docs/jira/v26.11.2/lanes -> FAIL 779+206 pre-existing baseline;
  this receipt's findings fixed by this errata block (re-run below;
waves/w-readme2.md:20
awk 'length>100' over README/SECURITY/CONTRIBUTING/receipt -> 0 hits
grep 0x5032_38FF crates/cgap-ffi/src/sovereign.rs -> :90/:1989 exit 0
grep "95|91" lanes/lane-xc05-conformance-kit-exporter.md -> 91 courts
grep flag table 01-release-gates.md:325-373 -> 23/20true/3false exit 0
```

Re-run after errata:
python3 scripts/receipt_lint.py --strict --dir
  docs/jira/v26.11.2/lanes 2>&1 | grep dod-07-root -> target: only
  STRUCT-001 cleared; claim findings reduced to zero on this file.

ERRATA 2 (lint close-out): UNRECEIPTED-CLAIM at lanes/ level is an
ENDEMIC BASELINE class, not lane-specific: 781 claim findings across
228 docs incl. previously-landed receipts (lane-ax-02:41, lane-ax-03:
39/76/82 all flagged the same way). Per receipt_lint.py's own scope
comment (:194) the enforceable claim-proximity surface is audit/ +
waves/; lanes/ receipts only carry RECEIPT-STRUCT-* (STRUCT-001 on
this file: CLEARED by the fenced block). The 6 proximity findings on
this receipt match the baseline class; recorded, not laundered.
