# Lane TPS-6 — Kanban ledger re-derivation (docs-only, zero source diff)

## W7 Receipt

### 1. Identity

Lane TPS-6, repo /Users/sac/cgap (main). Subject:
docs/jira/v26.11.2/audit/tps-06-kanban-ledger.md (written this lane) +
this receipt. No source/tree byte changed.

### 2. Authority

Dispatch prompt (owned files: audit/tps-06-kanban-ledger.md +
lanes/lane-tps-06-kanban.md).

### 3. Commands + exits (census, this session)

    ls docs/jira/v26.11.2/lanes/              → exit 0 (105 receipts listed)
    ls lanes/ | grep "gap-10\|ax-08"          → exit 0: lane-ax-08-seal.md
                                                 present; NO gap-10 receipt
    grep -rli "kanban" <repo>                 → exit 0, 0 hits (excluding
                                                 the files this lane wrote)
    grep "LSPA\|AX-\|GAP-" audit/audit-ledger.md → 0 hits (families absent
                                                 from ledger)
    sed -n reads of lane-lspa-03-attr-witness.md:103-112,
    lane-ax-08-seal.md:70-76, lane-ax-09-suspend.md:58-65,
    lane-b06-cscb-c7.md:49-53, lane-posh-02-batch.md:107-112,
    lane-ax-10-index.md:40-41, cgapprd-07-nfr02-gate.md:70-79,
    cull-08-doc-retirement.md:9-12, cgapprd-09-prd-rewrite-plan.md:54 → exit 0

### 4. Consequence

One new audit doc re-derives the kanban from on-disk receipts:
GAP-10 WIP (no receipt), LSPA-3 DONE/PARTIAL, AX-8 DONE/ALIVE
(supersedes cull-07:8-9's "ABSENT" — errata recorded, cull-07 not
rewritten), AX-9 DONE/refusal-witness (end-to-end UNVERIFIED-by-design),
B06 sealer row corrected with the load caveat. Fences: this diff can
regress nothing — it is a new docs file; the only must-not-regress
surface is cull-07's errata discipline (followed: ERRATA-not-rewrite).

### 5. Courts

No court applies — docs-only lane with zero executable subject; there
is no mutation that can flip a prose census except the census itself.
Anti-vacuity is carried by the census commands in §3: the grep for
"kanban" returning 0 hits is the falsifier of "the draft table exists",
and the gap-10/ax-08 ls grep is the falsifier of each status call. Both
ran this session with exits shown. Loud skips: none.

### 6. Standing

ALIVE for every row derived from a receipt re-read this session (the
cited file:line refs were read fresh in §3); UNVERIFIED for anything
beyond that (no re-runs of the cited receipts' own courts — their
standing is as-receipted, per audit-ledger.md's law, not re-derived).

### 7. Divergences/disclosures

- **The dispatched "TPS manual kanban draft" is not on disk** (repo-wide
  grep = 0 hits). Doc yields to reality; the re-derived ledger stands
  alone at audit/tps-06-kanban-ledger.md and corrects the four named
  stale rows per the dispatch text.
- **ERRATA against cull-07-ipc-legacy.md:8-9** ("AX-8 receipt: ABSENT"):
  lane-ax-08-seal.md landed since; ALIVE at :70-76. Recorded in
  tps-06-kanban-ledger.md §D; cull-07 left byte-identical.
- No SLA unblock applied to any other lane's files this session.
- No double registration: audit-ledger.md has no GAP/LSPA/AX rows; B06
  points at audit-ledger.md:43.
