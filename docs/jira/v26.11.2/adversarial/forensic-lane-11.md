# Forensic Lane FA11 — Hostile Forensics on the G7-PATENT Gate

Repo /Users/sac/cgap, branch main, 2026-11-02. Only write = this file;
mutations ran against /tmp copies (sha256-exact originals untouched —
claims.txt was read, never modified).

## 1. Deconstruction of the verdict logic

`scripts/verify_ost_design_around.py` grants DISTINCT-MECHANISM per row on
three inputs, none of which is structural analysis of cgap code:

1. **Zero src-scoped grep hits** for a one-line regex per patent
   (ROW_GREPS, verify_ost_design_around.py:29-50). One regex per patent
   (one-line alternatives) — a mechanism *named* in a variable name or
   comment trips it; a mechanism implemented under other names does not.
2. **Cite-file-exists + marker greps** (ROW_EVIDENCE, :53-70): the positive
   surface is `os.path.isfile(cite)` plus substring greps for 2-3 markers
   in ONE file per row.
3. **Status-column text** in claims.txt: a row whose status contains
   "PENDING" (and not "LANDED") is PENDING; "LANDED" text plus missing
   markers yields PARTIAL. The verdict is a function of the row's own
   status string.

Run this session: `python3 scripts/verify_ost_gate` — full command and exit
below in §5. Verdict: this is an **evidence-REGISTER consistency gate**, not
an infringement analyzer. It verifies that the register is internally
consistent with the corpus at the granularity of filenames, substrings, and
a self-declared status string. It never parses a patent claim, never reads
an actual claim element, and never analyzes code structure (no CFG, no
dataflow, no call-graph over the cited mechanism). Grade: accurate at its
own ambition — the register's header says "explicitly NOT legal advice —
engineering register only", and `01-release-gates.md:220-235` asks only for
"4/4 claims DISTINCT-MECHANISM verdict" + auditor sign-off. **No receipt or
charter document overclaims it as an infringement analysis.** The hostile
presumption fails here: the framing is honest at its stated ambition.

Anti-vacuity: the gate does flip. Mutation M1 (below) and the recorded
w6-g7-patent-gate-flip.md receipt both show exit 0 -> 1. Not vacuous.

## 2. Row 1 deep-dive (US10,817,467): re-scope = claim laundering?

### 2a. The element column is self-authored — and moves with the answer

`docs/jira/v26.20` claims.txt contains NO verbatim patent claim text. The
"element" field is the register's own paraphrase of what the patent claims
("shared-memory CAS over snapshot state"). The G7 gate never reads any
external patent text. Consequence: when the operator ordered the re-scope,
BOTH columns moved together — the design-around text AND the element it is
"distinct from". The comparison target was redefined in the same edit as
the answer. Proven mechanically: **mutation M2** replaced row 1's element
text with the arbitrary string "completely fake element text ZZQQ" and the
gate still returned exit 0 (command+exit in §5). The gate has no patent
text to compare against, so element wording is unfalsifiable inside the
gate. The re-scope is therefore a register-internal redefinition: legitimate
as an honest refinement note (the G7R KNOWN-DIVERGENCE note is retained and
discloses the single-ring limitation), but the distinctness verdict is
relative to a comparison target the repo itself authored. It cannot be
elevated to "valid design-around vs the patent's actual claim elements"
from anything on disk.

### 2b. Single-writer vs the '467 element set

Question posed: does "single-writer append-only ring" answer each element
of a claim reading roughly "parallel materialization with shared mutable
tries + CAS publication"? Answer from code:

- The landed mechanism (mmap_writer.rs:721-842, RingSink; drain_to_spill
  :781-802; RunOverrun :829-842; sovereign.rs:729-773) practices ZERO of
  the three structural features: no shared mutable structure (immutable
  append), no CAS/atomic, no parallel materialization. Verified this
  session: `grep -rnE 'CAS|compare_exchange|AtomicU64|shared-memory'
  crates/cgap-storage/src crates/cgap-parquet/src` -> exit 1, 0 hits.
- Does it "answer each element"? It answers them by NOT PRACTICING them,
  not by an alternative that delivers the same capability. If the actual
  '467 claims require concurrency (plural actors on shared mutable
  tries with CAS publication), a single-writer design is outside the
  claim; if any element reads generically ("updating a shared data
  structure"), element-by-element infringement analysis is impossible
  from this repo because no claim text exists on disk. The honest
  classification is **avoidance-by-omission**: cgap avoids the patent by
  declining the capability the patent covers (concurrent snapshot
  materialization) rather than by a distinct mechanism that provides
  that capability. That is legitimate engineering and the register's
  retained G7R divergence note half-discloses it ("SINGLE ring;
  partition-sealed delta pages are a further refinement"), but neither
  claims.txt nor the register states the dependency: distinctness holds
  ONLY under the single-writer premise, which nothing enforces.
- **What breaks with two writers (probed in code)**: `COMMIT_LEASE` is
  `thread_local!` (sovereign.rs:786-790). A second thread entering the
  commit entrypoint finds `slot.is_none()` (:734) and takes the grant
  branch: `std::fs::File::create(path)` (:735) on the SAME path —
  POSIX `O_TRUNC`. The second writer silently truncates the first
  writer's spill file mid-stream. There is no lease table, no owner
  check, no typed refusal for concurrent grant (the CGAP_ERR_LEASE_
  EXHAUSTED paths :742/:759/:764 all fire on IO/shape errors, not on
  double-grant). So the answer to "second thread gets its own lease?" is
  YES — an own lease AND an own truncate of the shared path; the
  single-writer assumption the row-1 distinctness rests on is an
  UNENFORCED, undocumented precondition, and a concurrent caller
  converts it into silent data loss, the exact failure class
  RunOverrun/CGAP_ERR_LEASE_EXHAUSTED exist to prevent. This is the
  strongest finding of the lane: the re-scoped row-1 mechanism is
  single-writer by accident of `thread_local!`, not by typed design.

## 3. Row 2 (US11,216,456): bit-slice pre-filter vs SIP

SIP (sideways information passing) per the '456 family propagates
tuple-level bindings between join operators AT RUNTIME: one operator's
output tuples are pushed down to restrict the other side as execution
proceeds, tuple by tuple.

The cgap mechanism (unify.rs): at `unify_batch_probe` entry the candidate
cache is invalidated (:760-770); per single-bound-slot join atom, the
current bindings column is harvested into `probe.key_set` (:1305-1315:
copy from `sc.cur_vars`, sort, dedup); if `key_set.len() <=
KEY_PREFILTER_MAX` the mask `probe.mask_s` is built via
`emit_join_masks_view` (:1318) BEFORE the radix-bucketed candidate scan,
and the inner probe still does full-word equality (superset prefilter,
disclosed :921-925).

Mechanical delta from SIP: cgap passes a SET-level prefilter (distinct
key set -> per-row-group survival mask) computed once per batch before
traversal, not tuple-at-a-time propagation between operators. That is a
real mechanical distinction — semi-join mask pushdown, not sideways
tuple shipping.

BUT the register wording overclaims: row 2's DISTINCT claim says
"compile-time PLNF cardinality ... bounds fixed before execution".
Ground truth: `bound_keys` = `probe.key_set` is derived from the CURRENT
BINDINGS at probe time — a runtime product of earlier atoms in the same
rule evaluation, refreshed per batch (cache invalidated every
`unify_batch_probe` entry). Bounds are fixed before the candidate SCAN,
not before execution; they are runtime-derived. The register itself
concedes the compile-time PLNF cardinality component "is a NAMED OPEN
ITEM, NOT part of the DISTINCT claim" (register:72-73) — so the
"compile-time" half of the wording is declared-while-unlanded and the
landed distinctness rests entirely on the batch-level mask pushdown.
Verdict: mechanically distinct from SIP (real), register wording
("bounds fixed before execution") is an overclaim by one scope level —
honest wording is "bounds fixed before candidate traversal, derived
per-batch from runtime bindings".

## 4. The grep families re-run this session (command+exit in §5)

- Row 1 (`CAS|compare_exchange|AtomicU64|shared-memory` over
  cgap-storage/src + cgap-parquet/src): exit 1, 0 hits. Negative
  surface CONFIRMED — no CAS/atomic anywhere in the scoped crates.
- Row 2 (`tuple[_ ]?binding|TupleBinding|propagate_binding` over
  crates/): 0 hits. CONFIRMED zero.
- Row 3 (`continuation|Fiber|Generator|resume` over cgap-bn3/src +
  cgap-bont/src): exactly 1 src hit — unify.rs:1823, verified BY LINE
  READ to be a `///` doc-comment line ("Generator with the 'every rule
  fires' witness"). Classification "doc-comment prose" is CORRECT.
  Note: the register cites this hit at :1688 and :1817 in different
  places; the live line is :1823 — line drift from edits, same content,
  no substance change.
- Row 4 (`topology|Topology|Box<dyn|&dyn ` over cgap-bn3/src): exactly
  1 src hit — sigil.rs:773, verified BY LINE READ:
  `let _: &dyn core::error::Error = &BN3Error::PayloadOverflow;`. This
  is a trait-object upcast for error plumbing in a test-adjacent file,
  not runtime dispatch. The NON_MECHANISM_CODE regex
  (verify_ost_design_around.py:128) whitelists exactly this shape.
  Classification CORRECT, but note the whitelist is a point exception:
  a future `&dyn SomeDispatchTrait` would be classified mechanism
  (good), while a second error-plumbing pattern spelled differently
  would trip a false NOT-DISTINCT (fail-safe direction). Register cites
  sigil.rs:757; live line is :773 — same line drift.
- Verdict per suspicion: no mechanism-classified hit is waved away.
  Both non-mechanism classifications check out as genuine comments /
  error plumbing on direct inspection.

## 5. Commands and exits (this session)

- `python3 scripts/verify_ost_design_around.py` -> exit 0,
  "G7-PATENT: PASS 4/4 DISTINCT-MECHANISM".
- Mutation M1 (row-1 status LANDED->PENDING, /tmp copy):
  `python3 scripts/verify_ost_design_around.py --claims
  /tmp/fa11/claims_mut.txt --report /tmp/fa11/report_m1.json` -> exit 1
  ("PENDING (status column PENDING ...)"). Proves the status STRING is
  load-bearing.
- Mutation M2 (row-1 element text -> "completely fake element text
  ZZQQ", status restored): -> exit 0. Proves the element text is never
  compared to anything — no patent-text channel exists in the gate.
- `grep -rnE 'RingSink|drain_to_spill|RunOverrun'
  crates/cgap-parquet/src/mmap_writer.rs | wc -l` -> 41 (markers real,
  not decorative strings; RingSink is a real 64 KiB ring with typed
  refusal paths).
- Per-row greps in §4, each with exit observed.

## 6. Verdict per suspicion

1. "G7 1/4->4/4 achieved by rewriting regexes": NOT SUSTAINED as stated —
   the gate flips on real mutations (M1 here; w6 flip receipt), markers
   are real code, and the two waved src hits are genuine comment/plumbing.
   But SUSTAINED in the weaker form: 4/4 was achieved by RE-SCOPING row
   text (rows 1 and 2 both "re-scoped per operator order" on 2026-11-02),
   and the gate cannot challenge a re-scope because it has no patent text.
2. "Decorative ROW_EVIDENCE strings": REFUTED — 41 real marker hits; the
   ring, spill drain, typed refusals, and BLAKE3 chain are landed code.
3. "Row 1 re-scope launders the gap": PARTIALLY SUSTAINED — the element
   column was rewritten with the answer (self-referential comparison
   target; M2 proves no external check), and distinctness rests on an
   UNENFORCED single-writer precondition that silently truncates under a
   second writer (sovereign.rs:729-744). Avoidance-by-omission is
   legitimate engineering but is nowhere stated as a dependency.
4. "Row 2 is rebranded SIP": REFUTED at mechanism level (set-level mask
   pushdown vs tuple-level runtime propagation); SUSTAINED at wording
   level ("bounds fixed before execution" overstates — key set is
   runtime-derived per batch).

Standing: ALIVE for every claim above (command+exit in §5 run this
session against /Users/sac/cgap, branch main). Shared tree untouched
except this file.
