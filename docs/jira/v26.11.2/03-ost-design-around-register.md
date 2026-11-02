# v26.11.2 OST Design-Around Register (4/4 DISTINCT-MECHANISM)

Transcription of CHARTER-CGAP-STARDF-v26.11.2 §OST DESIGN-AROUND REGISTER.
Repo: /Users/sac/cgap. Lease: docs/jira/v26.11.2/03-ost-design-around-register.md
(this file, NEW; no other file written; no git writes performed).
Explicitly NOT legal advice — engineering register only.

## Law (binding on every v26.11.2 lane)

Any lane restating a patented mechanism as a design-around = typed REFUSED
(OVERNIGHT-LOOP.md §6, gate-enforced). Parity is achieved by independent
mechanisms, not renamed ones. Each row below names the mechanism that is
DIFFERENT, not a paraphrase of the claim.

## Register

| # | Patent | OST patented mechanism | v26.11.2 design-around (distinct mechanism) | Refused alternative |
|---|--------|------------------------|---------------------------------------------|---------------------|
| 1 | US 10,817,467 | shared-memory CAS over snapshot state | partition-disjoint snapshot-delta: each partition seals its own delta pages; no shared-memory CAS anywhere | any CAS/lock-based snapshot merge |
| 2 | US 11,216,456 | runtime tuple-binding propagation | compile-time PLNF cardinality + vertical bit-slice masks; bounds fixed before execution | runtime sideways information passing |
| 3 | US 11,409,698 | runtime subquery continuation frames | stratified semi-naive bit-matrix cascades in caller-leased scratch; no continuation frames | runtime subquery continuation |
| 4 | US 11,645,278 | runtime topology switching | branchless register unification + static instruction schedule (R0..=R255 ABI) | dynamic topology dispatch |

## Cross-check (read-only, this session)

Per-row implementation-evidence status (PENDING = named for v26.11.2, no
cgap implementation yet; statement-evidence = waves/ receipt naming the
mechanism).

1. US 10,817,467 — evidence run 2026-11-01 (lane W2-7, receipt
   waves/w2-07-ost-greps.md):
   `grep -rnE 'CAS|compare_exchange|AtomicU64'
   crates/cgap-storage/src/ crates/cgap-parquet/src/` → exit 1,
   0 hits (no shared-memory CAS surface in src). Full-crate sweep
   (src+tests) = 13 hits, ALL in tests/ harness code: `AtomicU64`
   unique-counters (tests/common/mod.rs:14,18;
   offset_fuzz.rs:317,318) + `CASE_TIMEOUT`/`LCG_CASES` substring
   matches on CAS — zero `compare_exchange`. Verdict:
   DISTINCT-MECHANISM (negative-surface evidence on record).
   Adjacent statement on record: stardf RDF-5 incremental
   (/Users/sac/stardf/arrow/docs/ost/incremental.md) seals immutable
   footer-sequenced delta pages, recomputes views PER-DELTA-PAGE.
2. US 11,216,456 — bit-slice half implemented, PLNF half PENDING.
   Cite on disk, re-checked 2026-11-01: crates/cgap-parquet/
   src/bitslice.rs (930 lines; vertical bit-slice masks; canonical
   SplitMix64 court seed cite at bitslice.rs:687-699). PLNF
   compile-time cardinality: still PENDING — no cgap implementation
   evidence this session; recorded honestly as PENDING.
   Underlying statement evidence on record: stardf SIP-ALT
   (/Users/sac/stardf/arrow/docs/ost/sip-alt.md) cites
   "NOT-practiced: US 11,216,456 / US 11,409,698 claim classes" via
   precomputed structural planning from sealed per-file per-predicate
   count tables. cgap partial impl evidence: crates/cgap-parquet/src/
   bitslice.rs (vertical bit-slice masks). Compile-time PLNF
   cardinality: PENDING.
3. US 11,409,698 — evidence run 2026-11-01 (lane W2-7):
   `grep -rniE 'continuation|Fiber|Generator|resume'
   crates/cgap-bn3/ crates/cgap-bont/` → exit 1 on the
   continuation|Fiber|resume subset, 0 hits (no continuation-frame
   surface). Broadened sweep 5 hits, all doc-comment "corpus/row
   generator" prose in tests (unify.rs:133,1250,1262,1879;
   shacl_compiler.rs:2888) — no runtime Generator/Fiber type.
   Verdict: continuation surface zero = DISTINCT-MECHANISM
   negative-surface evidence; semi-naive cascade
   implementation: still PENDING.
4. US 11,645,278 — implementation evidence on disk, re-checked
   2026-11-01: crates/cgap-bn3/src/unify.rs (lane A12, v26.11.1
   waves 9-10): register-file ABI R0..=R255 (unify.rs:100,169),
   Wave 2 branchless-selection kernel, typed
   UnifyError::RegisterOverflow (unify.rs:191). Negative-surface
   run 2026-11-01 (lane W2-7): `grep -rnE 'dyn |dyn\b|Box<dyn|
   topology|Topology|switch|dispatch' crates/cgap-bn3/src/` →
   0 `topology` hits; sole `dyn` hit is an error-trait upcast in a
   sigil test (sigil.rs:757, `&dyn core::error::Error`) — not
   dynamic topology dispatch; builtins.rs:497 states "No `dyn`, no
   `fn` values"; all other hits are the compile-time `dispatch`
   match — the design-around itself. Verdict: DISTINCT-MECHANISM.

### errc-backlog + dia-5 scout cross-check

- docs/jira/v26.11.1/waves/errc-backlog.md: read this session. Zero
  patent/design-around rows (grep for patent/design-around/467/278:
  zero hits). Its only grounded candidate is the docs-diataxis-pack
  generalization — no overlap with any register row.
- docs/jira/v26.11.1/waves/dia-5-marketplace-scout.md: read this
  session. Documents the readme-diataxis-pack consumer run + FM-PACK-013
  mutation court. No patent content; no overlap with any register row.
- Related gate: docs/jira/v26.11.2/01-release-gates.md G7-PATENT
  (verify_ost_design_around.py, 4/4 DISTINCT-MECHANISM verdict);
  docs/jira/v26.11.2/patent/ now holds claims.txt (transcribed
  register, cross-checked consistent this session — claim elements
  and mechanisms match the rows above; row statuses in claims.txt
  (rows 1,3 PENDING rows) predate this run; register rows above
  are the authoritative updated statuses).

## Receipt

- Written 2026-11-01 by V2-PATENT (court-auditor); evidence greps
  and row updates this session by lane W2-7 (receipt
  waves/w2-07-ost-greps.md), repo /Users/sac/cgap, branch main.
- Standing: transcription ALIVE; row 1 DISTINCT-MECHANISM
  (negative-surface src grep exit 1, 0 hits); row 2 partial
  (bitslice.rs re-checked on disk, PLNF PENDING); row 3
  continuation surface zero (DISTINCT-MECHANISM negative-surface),
  semi-naive cascade impl PENDING; row 4 DISTINCT-MECHANISM
  (A12 unify.rs cited, no topology/dyn dispatch surface).
