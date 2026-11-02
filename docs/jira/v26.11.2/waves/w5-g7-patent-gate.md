# W5-G7 Patent Gate Receipt

Lane: D9 (design). Repo: /Users/sac/cgap, branch main. 2026-11-02.

## Deliverables

- `scripts/verify_ost_design_around.py` (NEW, 280 lines) — G7-PATENT gate
  per `01-release-gates.md:226-244`: parses
  `docs/jira/v26.11.2/patent/claims.txt` (pipe-format, 2-space indent
  continuation), runs each row's DISTINCT-MECHANISM grep family src-scoped
  against `crates/` via real `grep -rnE` subprocess, classifies src vs
  tests hits (tests-only != practicing), emits
  `docs/jira/v26.11.2/patent/ost_report.json`.
- Exit contract: 0 all DISTINCT; 1 any PENDING/PARTIAL/NOT-DISTINCT
  (failing rows named on stdout); 3 `[SKIP] claims.txt-absent` /
  `[SKIP] corpus-absent`.

## Baseline run (real, this session)

    python3 scripts/verify_ost_design_around.py \
      --claims docs/jira/v26.11.2/patent/claims.txt \
      --corpus /Users/sac/purrdf/corpora \
      --report docs/jira/v26.11.2/patent/ost_report.json
    -> exit 1

- US10,817,467 -> PENDING (status column PENDING; no landed evidence)
- US11,216,456 -> PARTIAL (LANDED bitslice cite + markers verified;
  PLNF component PENDING)
- US11,409,698 -> PENDING (status column PENDING; the single grep hit
  unify.rs:1688 is a `///` doc-comment mention of "Generator",
  classified non-mechanism and disclosed in ost_report.json)
- US11,645,278 -> DISTINCT-MECHANISM (zero mechanism src hits — the
  sigil.rs:757 `&dyn core::error::Error` upcast is classified
  non-mechanism, disclosed; R0/RegisterOverflow markers verified in
  crates/cgap-bn3/src/unify.rs, 32 lines match)
- Report on disk: 4493 bytes, claims sha256 0ce2db232c00...
- Gate verdict FAIL (1/4 DISTINCT), failing rows named: this is a
  CORRECT fail (rows 1-3 genuinely lack landed evidence), not a lane
  failure.

## Skip paths (exit 3, loud)

- claims.txt-absent probe (jiva path): exit 3 `[SKIP] claims.txt-absent`.
- corpus-absent probe (/Users/sac/purrdf/critique): exit 3
  `[SKIP] corpus-absent`.

## FLIP court (anti-vacuity, /tmp sandbox, sha256-exact restore)

- Mutation: merged row1's mechanism field with row4's
  ("branchless register unification + static instruction schedule
  (R0..=R255 ABI)") -> duplicate-mechanism register.
- Command: script run with `--claims /tmp/claims_flipped.txt
  --report /tmp/ost_report_flip.json` -> exit 1.
- Flip witnessed: US11,645,278 dropped DISTINCT-MECHANISM ->
  NOT-DISTINCT (duplicate/near-duplicate mechanism shared across rows
  [US10,817,467, US11,645,278]); US10,817,467 also NOT-DISTINCT.
  A gate that could not flip would be VACUOUS; this one flips.
- Shared tree untouched: claims.txt sha256 before/after identical
  (0ce2db232c00..., exact restore verified True).

## Exactly what would close each PENDING row

- Row1 US10,817,467: land a negative-surface src grep receipt for
  cgap-storage/cgap-parquet src (already 0 mechanism hits) PLUS change
  the claims.txt status column from PENDING to LANDED-<cite> pointing
  at an existing cite file whose markers verify. The snapshot-delta
  mechanism cite itself (partition-sealed delta pages) does not exist
  in the tree — that implementation or an explicit registered
  divergence row is the missing evidence.
- Row2 US11,216,456: PLNF compile-time cardinality implementation
  evidence (no cgap implementation exists today); the bitslice half
  is already verified LANDED.
- Row3 US11,409,698: semi-naive cascade implementation evidence in
  caller-leased scratch — a real cite file + markers, and status
  PENDING -> LANDED in claims.txt.

## Standing

- Gate machinery: ALIVE (baseline exit 1, skip probes exit 3, FLIP
  court flip witnessed — all commands+exits above, this session).
- G7-PATENT gate itself: FAIL 1/4 DISTINCT (correct fail; rows 1-3
  lack landed evidence — closure paths named above). The prior BLOCKED
  status (script-absent) is resolved; the gate now fails honestly
  rather than skipping.

<!-- JSON
{"lane":"D9","files_written":["scripts/verify_ost_design_around.py","docs/jira/v26.11.2/patent/ost_report.json","docs/jira/v26.11.2/waves/w5-g7-patent-gate.md"],"commands":["python3 scripts/verify_ost_design_around.py --claims docs/jira/v26.11.2/patent/claims.txt --corpus /Users/sac/purrdf/corpora --report docs/jira/v26.11.2/patent/ost_report.json -> exit 1 (1/4 DISTINCT)","skip probes: claims-absent + corpus-absent -> exit 3 [SKIP]","FLIP court: mutated /tmp/claims_flipped.txt (merged duplicate mechanism) -> exit 1, US11,645,278 DISTINCT->NOT-DISTINCT witnessed, sha256-exact restore True (0ce2db232c00)"],"courts_run":["FLIP merged-near-duplicate-claims (flipped: row4 NOT-DISTINCT)"],"standing":"gate machinery ALIVE (baseline exit 1 + skip exit 3 + FLIP flip witnessed); G7-PATENT verdict FAIL 1/4 DISTINCT — correct fail, rows 1-3 PENDING with named closure paths","divergences":["row1 grep family (CAS/compare_exchange/AtomicU64) src-scoped returned 0 hits as expected; dispatch expected rows 2-3 PENDING — actual: rows 1-3 not DISTINCT, row4 DISTINCT (verified on disk)"]}
-->
