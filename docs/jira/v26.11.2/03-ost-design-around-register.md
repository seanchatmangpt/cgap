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
| 1 | US 10,817,467 | shared-memory CAS over snapshot state | immutable columnar append-only ring commits — a pre-leased 64 KiB ring sink (CAP = 65,536 B, mmap_writer.rs:775; capacity counts BYTES here, never lanes) with rotation drain to a pre-opened spill handle, typed RunOverrun refusal, plus a fail-closed BLAKE3 hash-chain receipt log binding every commit | any CAS/lock-based snapshot merge |
| 2 | US 11,216,456 | runtime tuple-binding propagation | compile-time PLNF cardinality + vertical bit-slice masks; bounds fixed before execution | runtime sideways information passing |
| 3 | US 11,409,698 | runtime subquery continuation frames | stratified semi-naive bit-matrix cascades in caller-leased scratch; no continuation frames | runtime subquery continuation |
| 4 | US 11,645,278 | runtime topology switching | branchless register unification + static instruction schedule (R0..=R255 ABI) | dynamic topology dispatch |

## Cross-check (read-only, this session)

Per-row implementation-evidence status (PENDING = named for v26.11.2, no
cgap implementation yet; statement-evidence = waves/ receipt naming the
mechanism).

1. US 10,817,467 — LANDED 2026-11-02 (lane G7F, operator order to
   re-scope the row-1 design-around to the LANDED mechanism).
   Prior KNOWN-DIVERGENCE (2026-11-02 lane G7R) is retained below as
   the honest refinement note: the re-scoped claim is explicitly a
   SINGLE ring; partition-sealed delta pages are a further
   refinement, not claimed.
   New design-around wording (register row 1): immutable columnar
   append-only ring commits — a pre-leased 64 KiB ring sink (CAP =
   65,536 B, mmap_writer.rs:775) with rotation drain to a pre-opened spill handle, typed RunOverrun
   refusal, plus a fail-closed BLAKE3 hash-chain receipt log binding
   every commit.
   Landed cites (marker greps verified this session; line cites
   re-verified 2026-11-02 lane RW-03): RingSink
   (9 hits), drain_to_spill (2 hits), RunOverrun (30 hits) at
   crates/cgap-parquet/src/mmap_writer.rs:721-860 (RingSink :721;
   drain_to_spill :828; typed RunOverrun refusals in the drain body
   at :843, :851); BLAKE3 hash-chain receipt leg prev_hash field +
   BLAKE3 link fn
   (crates/cgap-ffi/src/receipt_chain.rs:51-57, :117-120).
   Single-writer enforcement (FA9FIX, added 2026-11-02 lane C13):
   typed pre-IO concurrent-writer denial CGAP_ERR_CONCURRENT_WRITER_DENIED
   = 0x4347_0006 (crates/cgap-ffi/src/sovereign.rs:87, spill_gate
   Claim::Denied before any spill IO, sovereign.rs:758-764); race
   court crates/cgap-ffi/tests/fa9_commit_gating.rs (606 lines,
   register pin + spill_gate race; the formerly unenforced
   single-writer premise of this row is now mechanically enforced).
   Courts re-witnessed:
   waves/w5-ring-rewitness.md. Gate: DISTINCT-MECHANISM (exit 0 on
   this row, run 2026-11-02, lane G7F).
   Prior refinement note (G7R, 2026-11-02): the earlier design-around
   wording ("partition-disjoint snapshot-delta; each partition seals
   its own delta pages") has NO landed cgap artifact — zero
   `partition`-sealing surface in crates/cgap-storage/src +
   crates/cgap-parquet/src (sole `partition_point` hits are
   binary-search calls, unrelated). The W3-L4 RingSink pre-leased
   ring commit leg is a SINGLE ring with a hash-chain receipt leg,
   NOT per-partition sealed delta pages. Negative-surface evidence on
   record (lane W2-7, 2026-11-01): `grep -rnE
   'CAS|compare_exchange|AtomicU64'
   crates/cgap-storage/src/ crates/cgap-parquet/src/` → exit 1,
   0 hits (no shared-memory CAS surface). Adjacent statement on
   record: stardf RDF-5 incremental
   (/Users/sac/stardf/arrow/docs/ost/incremental.md) seals immutable
   footer-sequenced delta pages, recomputes views PER-DELTA-PAGE.
2. US 11,216,456 — RE-SCOPED 2026-11-02 (lane G7R, operator order):
   the DISTINCT claim covers the vertical bit-slice masks with bounds
   fixed before execution — the component that answers the patent
   element ("runtime tuple-binding propagation": cgap computes
   qualifying-row masks via vertical bit-slices + semi-join, bounds
   pre-execution, no runtime propagation). Cite on disk:
   crates/cgap-parquet/src/bitslice.rs (930 lines; canonical
   SplitMix64 court seed cite at bitslice.rs:687-699). The
   compile-time PLNF cardinality component is a NAMED OPEN ITEM, not
   part of the DISTINCT claim. Gate: DISTINCT-MECHANISM (exit 0 on
   this row, run 2026-11-02).
   Doctrine note (lane RW-03, 2026-11-02): no register row rests on
   claimed unconstructability of any mechanism — the moat is landed
   distinct mechanism, never "cannot be built". The AHDCLT
   compute-kernel half ("technically unconstructable") is
   RETIRED-REFUTED per docs/jira/v26.10.31/16-ahdclt.md:3-4,13
   (REVISED status, 2026-11-02; RED-01 clean-room rebuild) and must
   not be cited as a design-around assumption.
   Underlying statement evidence on record: stardf SIP-ALT
   (/Users/sac/stardf/arrow/docs/ost/sip-alt.md) cites
   "NOT-practiced: US 11,216,456 / US 11,409,698 claim classes" via
   precomputed structural planning from sealed per-file per-predicate
   count tables.
3. US 11,409,698 — LANDED 2026-11-02 (lane G7R). Cite:
   crates/cgap-bn3/src/unify.rs. Mechanism evidence: static
   RuleCompiler rule-to-register compilation (unify.rs:553-563),
   semi-join bit-matrix cascades (`semi_join_masks`, unify.rs:914;
   per-row survival mask, unify.rs:744) over caller-leased probe
   scratch (unify.rs:740, JoinProbeScratch). No continuation frames
   (negative-surface run 2026-11-01 lane W2-7: 0 mechanism hits; the
   sole src hit unify.rs:1887 is a `///` doc-comment "Generator"
   mention, classified non-mechanism and disclosed in
   ost_report.json). Note: the register's design-around wording says
   "stratified"; the landed cascade is semi-naive delta-round strata
   (crates/cgap-storage/src/closure.rs:114 semi_naive_closure) — the
   strata mapping is per-delta-round, disclosed here. Gate verdict
   DISTINCT-MECHANISM (exit 0 on this row, run 2026-11-02).
4. US 11,645,278 — implementation evidence on disk, re-checked
   2026-11-01: crates/cgap-bn3/src/unify.rs (lane A12, v26.11.1
   waves 9-10): register-file ABI R0..=R255 (unify.rs:155, :486),
   Wave 2 branchless-selection kernel, typed
   UnifyError::RegisterOverflow (unify.rs:246). Negative-surface
   run 2026-11-01 (lane W2-7): `grep -rnE 'dyn |dyn\b|Box<dyn|
   topology|Topology|switch|dispatch' crates/cgap-bn3/src/` →
   0 `topology` hits; sole `dyn` hit is an error-trait upcast in a
   sigil test (sigil.rs:773, `&dyn core::error::Error`) — not
   dynamic topology dispatch; builtins.rs:497-498 states "No `dyn`,
   no `fn` values"; all other hits are the compile-time `dispatch`
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
- Standing (updated 2026-11-02, lane RW-03): quantitative sweep of
  this file + 04-stardf-closure.md against the exclusive numeric
  registry (11-measured-claims.md). Registry-scoped perf figures
  (10.20 ns/frame quiet-host row 5 / row 28 contention divergence;
  36,048 B subject-scoped + bn3 6,520,984 B REFUSED footprint rows;
  row 29 "constant-time" UNSUPPORTED) do NOT appear in either file —
  nothing to rewrite on those; only stale line cites + the bytes-vs-
  lanes scope qualifier needed repair. All file:line cites in rows
  1-4 re-verified on disk this session (see diffs above); AHDCLT
  doctrine note added (row 2 block). No numbers invented; UNVERIFIED
  never rounded up.
- Standing (updated 2026-11-02, lane G7F): row 1 LANDED (re-scoped
  to the single-ring design-around per operator order — RingSink +
  drain_to_spill + RunOverrun in mmap_writer.rs:721-842, BLAKE3
  prev_hash chain in receipt_chain.rs:34-76, single-writer
  enforcement CGAP_ERR_CONCURRENT_WRITER_DENIED 0x4347_0006
  (sovereign.rs:87; fa9_commit_gating.rs court); partition-sealed delta
  pages remain a declared refinement, not claimed); rows 2, 3, 4
  DISTINCT-MECHANISM. Gate 4/4 DISTINCT exit 0 run 2026-11-02 lane
  G7F; flip court (merged-duplicate mutation) drops to exit 1,
  receipt waves/w6-g7-patent-gate-flip.md.
