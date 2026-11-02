# W2-7 OST Evidence Greps Receipt

Lane: W2-7 (lane-runner). Repo: /Users/sac/cgap, branch main.
Sole writes: 03-ost-design-around-register.md (row updates) +
this receipt. No git writes. /Users/sac/stardf untouched
(read-only cites only). 2026-11-01.

## Per-row evidence

### Row1 — US 10,817,467 (shared-memory CAS over snapshot state)

- Command: `grep -rnE 'CAS|compare_exchange|AtomicU64'
  /Users/sac/cgap/crates/cgap-storage/
  /Users/sac/cgap/crates/cgap-parquet/`
- Exit: 0; output: 13 lines (/tmp/w27-row1.txt).
- All 13 hits in tests/ harness code: `AtomicU64` unique
  counters (cgap-storage/tests/common/mod.rs:14,18;
  cgap-parquet/tests/offset_fuzz.rs:317,318) plus `CAS`
  substring matches inside `CASE_TIMEOUT`/`LCG_CASES`
  (offset_fuzz.rs:66,69,374,524,805,876;
  robustness.rs:59,133,185). Zero `compare_exchange`.
- Src-scoped re-run: same pattern over both crates' src/
  → exit 1, 0 hits.
- Verdict: DISTINCT-MECHANISM (negative-surface evidence
  landed: no CAS surface in src; test hits are harness
  counters/timeouts, not a snapshot-CAS mechanism).

### Row2 — US 11,216,456 (runtime tuple-binding propagation)

- Landed cite re-verified on disk: ls -la shows
  crates/cgap-parquet/src/bitslice.rs, 36648 bytes, 930
  lines; seed cite lines 687-699 confirmed by sed -n.
- Command: `ls -la .../bitslice.rs; wc -l; sed -n '687,699p'`
- Exit: 0. PLNF compile-time cardinality: still PENDING —
  no cgap implementation evidence; recorded honestly.
- Verdict: bit-slice half LANDED; PLNF half PENDING (row
  stays PENDING overall for PLNF; not upgraded).

### Row3 — US 11,409,698 (runtime subquery continuation frames)

- Command: `grep -rniE 'continuation|Fiber|Generator|resume'
  /Users/sac/cgap/crates/cgap-bn3/ /Users/sac/cgap/crates/cgap-bont/`
- Exit: 0; output: 5 lines (/tmp/w27-row3.txt). Case-folded
  sweep: `continuation`, `Fiber`, `resume` contribute ZERO
  hits (verified: subset grep exit 1, 0 hits); all 5 hits
  are doc-comment "corpus/row generator" prose
  (unify.rs:133,1250,continuation-irrelevant;
  shacl_compiler.rs:2888) — no runtime Generator/Fiber type.
- Verdict: continuation surface ZERO → DISTINCT-MECHANISM
  negative-surface evidence landed; semi-naive cascade
  implementation: still PENDING.

### Row4 — US 11,645,278 (runtime topology switching)

- Cite: crates/cgap-bn3/src/unify.rs (lane A12, v26.11.1
  waves 9-10): R0..=R255 ABI (unify.rs:100,169),
  UnifyError::RegisterOverflow (unify.rs:191); Wave 2
  branchless-selection kernel.
- Command: `grep -rnE 'dyn |dyn\b|Box<dyn|topology|Topology|
  switch|dispatch' /Users/sac/cgap/crates/cgap-bn3/src/`
- Exit: 0. Zero `topology` hits. Sole `dyn` hit: sigil.rs:757
  `&dyn core::error::Error` — error-trait upcast in a test,
  not dynamic topology dispatch. builtins.rs:497: "No `dyn`,
  no `fn` values". Remaining hits are the compile-time
  `dispatch` match in builtins.rs — the design-around
  itself, not a topology switch.
- Verdict: DISTINCT-MECHANISM (A12 landed cite + zero
  topology/dyn-dispatch surface).

## Cross-check vs patent/claims.txt

claims.txt read this session (13 lines). Claim elements and
mechanisms match the register rows 1-4 verbatim. Status
columns in claims.txt (rows 1,3 PENDING; row 2 LANDED-with-
PENDING-PLNF; row 4 LANDED) predate this run; register rows
are the authoritative updated statuses. No contradiction in
claim-element text found.

## Standing

- Row1 DISTINCT-MECHANISM: ALIVE (commands + exits above).
- Row2: half-landed, PLNF PENDING — UNVERIFIED for PLNF.
- Row3: negative-surface DISTINCT-MECHANISM landed; cascade
  impl PENDING.
- Row4 DISTINCT-MECHANISM: ALIVE.
- No mutation court required for this lane: greps are
  negative-surface evidence, not gates; verdicts are
  recorded per-row with commands+exits, PENDING rows kept
  PENDING.

<!-- JSON
{"lane":"W2-7","files_written":["docs/jira/v26.11.2/03-ost-design-around-register.md","docs/jira/v26.11.2/waves/w2-07-ost-greps.md"],"commands":["grep -rnE 'CAS|compare_exchange|AtomicU64' crates/cgap-storage/ crates/cgap-parquet/ -> exit 0, 13 lines","grep -rnE 'CAS|compare_exchange|AtomicU64' crates/cgap-storage/src/ crates/cgap-parquet/src/ -> exit 1, 0 hits","grep -rniE 'continuation|Fiber|Generator|resume' crates/cgap-bn3/ crates/cgap-bont/ -> exit 0, 5 lines (subset continuation|Fiber|resume exit 1, 0 hits)","grep -rnE 'dyn |dyn\\b|Box<dyn|topology|Topology|switch|dispatch' crates/cgap-bn3/src/ -> exit 0, 0 topology hits, 1 dyn hit (sigil.rs:757 test upcast)","ls -la crates/cgap-parquet/src/bitslice.rs; wc -l; sed -n 687,699p -> exit 0"],"courts_run":[],"standing":"row1 DISTINCT-MECHANISM ALIVE; row2 half-landed PLNF PENDING; row3 negative-surface landed cascade PENDING; row4 DISTINCT-MECHANISM ALIVE","divergences":["dispatch said expect zero for row1 full-crate grep; got 13 test-harness hits — resolved by src-scoped grep (0) + disclosed test hits, all non-mechanism"]}
-->
