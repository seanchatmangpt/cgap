# cgapprd-09-prd-rewrite-plan

Lane: CGAP-PRD-9 (PLAN). 2026-11-02. Read-only lane; this file is the
single deliverable.
Target: docs/jira/v26.11.2/12-ard-prd-cgap.md (161 lines, 10,871 B,
verified this session via wc -c / wc -l).
Consumed (all landed on disk in audit/): cgapprd-01 (claims audit),
cgapprd-02 (equation grounding), cgapprd-03 (frame layout),
cgapprd-04 (checkpoint design, DRAFT but PRESENT - no PENDING row),
cgapprd-08 (boundaries).

## Falsifier (verbatim, load-bearing)

The PRD Verify passes iff every quantitative claim carries a receipt
file:line and zero claims use a referent that fails an in-tree grep.

## Mount state

- 12-ard-prd-cgap.md :1-5 charter status; :19 Chatman Equation;
  :48-72 P1-P6; :74-82 NFR table (SS3.2); :84-119 ARD SS4; :121-138
  SS5 governance; :143-157 SS6 doctrine; :159-161 landing receipt.
- Supersession pointer: 00-ard-prd.md (per :3-5). PRD-8's five
  AshA2A-era clauses have NO on-disk referent inside 12-ard (grep:
  zero hits for AshA2A / work order / socket in 12-ard, 00-ard-prd,
  14-dflss-charter). SS4 below is therefore audited under PRD-8's
  W1 discipline + W3 11-symbol cap, not its literal clause table.

## Legend

STAYS = receipt-backed, cite the receipt. REWRITE = corrected text
spec given. DELETE = invented lore, no rewrite.

## Section-by-section plan

### Header block :1-5 - STAYS
Receipt: waves/v2-17-ard-landing.md:159-161 (quoted verbatim in the
doc's own trailing JSON, ALIVE standing).

### SS1 Executive Summary :7-17 - STAYS
Positioning prose; no falsifiable quantitative claim. The flat-
columnar/branchless/pre-leased framing is receipt-backed by AE4
disassembly invariants (PRD-1 fresh run, exit 0; lane-g01:3).

### SS1 equation :19 - REWRITE (PRD-2)
DELETE the modular framing 'Delta_S + Phi == R (mod Sigma_PQC)' as
fact. Replace verbatim with PRD-2's typed-trailer block (audit/
cgapprd-02-equation-grounding.md 'Exact rewrite'): The Chatman
Equation's landed trace is three typed commit trailers with typed
refusals - lease-phi, lease-delta, lease-receipt - and a fail-closed
render; no modular arithmetic exists. lease-phi is a caller-supplied
32-byte digest the render passes through; lease-delta is a u64
admitted-delta count; lease-receipt is an 80-byte frame
(nonce || parent || payload || tag) verified fail-closed against its
domain-separated tag. There is no Sigma_PQC term in the tree: the
digest is BLAKE3 (classical), and any post-quantum seal remains an
unbuilt Wave-3 seam.
Receipts: commit_synth.rs:60-62,111-156,187-189,260-267; zero .rs
hits for Sigma_PQC (PRD-2 grep 2); chatman.rs:9-10,351,368,422.

### SS2.1 Market Thesis :23-31 - STAYS
Competitive positioning; claims are historical/qualitative ('sub-32 KB
Flash environments' names a market tier, not a measured capability).
No receipt required; no in-tree referent asserted.

### SS2.2 Tier table :33-42 - STAYS
Boundary law (design constraint), not a measurement. Consistent with
PRD-8 W1 clause-2 mechanism reality (sovereign.rs:413 admission,
cgap-git ingress_ring + admission_gate).

### SS3.1 P1 penstock :48-52 - STAYS
Receipt: PRD-1 FR-01 VERIFIED - fresh verify_disassembly_invariants.py
exit 0 incl. binary mutant; lane-g01:3, lane-g02:118-121.

### SS3.1 P2 columnar 24 B :53-57 - STAYS + clarify note (PRD-3)
Already says '24 bytes per row' (:56) - matches landed reality
(generate_cscb_corpus.py:6,25,66; cscb_c1_c2_ingress.rs:45 [u8;24]
type-level pin; ingress_ring.rs:30; workspace-cgap.yaml:23). ADD one
clarifying sentence: the only 64-byte object is cgap-arrow's Quad64
(layout.rs:36) RDF-quad buffer layout, not the CSCB fact frame; no
code reads bytes 24..64 of any fact (PRD-3 SS5).

### SS3.1 P3 join/deduction :58-61 - STAYS with receipt
Receipt: unify.rs:840 RADIX_BUCKETS=4096 (PRD-1 FR-02 VERIFIED);
dod-cgap-bn3.md:17. Cross-boundary joins stay bounded by PRD-8 W3:
cgap_radix_join is MISSING at every layer (NO-GO CTQ-04) - the C-ABI
surface claim must not be inferred from this paragraph.

### SS3.1 P4 lease commits :62-66 - STAYS
Receipts: ingress_ring.rs:146-150,280; commit_synth.rs lease trailers
(PRD-2); posh_alloc_sentry_court.rs:8-24 (PRD-1 fresh).

### SS3.1 P5 receipt chaining :67-69 - STAYS
Receipt: cgap-blake3 31/31 incl. chain_mutation_court (PRD-1 fresh);
chain_mutation_court.rs:1; commit_synth.rs:111-156 (80-byte frame,
ReceiptTagMismatch fail-closed, PRD-2).

### SS3.1 P6 O-ABI packs :70-72 - STAYS
Receipt: pack_demux.rs:40-51 FACT_WINDOW=1,365 pack stride with
HONEST-NONE doc; corpus pack '<QQQ' 24 B (PRD-3 SS2).

### SS3.2 NFR table :74-82 - REWRITE (PRD-1 honest markings)
Row Admission <15 ns/frame: keep as TARGET, mark ASPIRATIONAL - no
receipt in PRD-1's table verifies 15 ns; wording: 'Target; unmeasured
in this charter; nearest measured ingress is the IPC-4 mmap transport
court (4.52-4.656 ns/op) per PRD-8 W2.'
Row Deductive Materialization: rewrite per NFR-01 HONEST FAIL -
'Measured median 23.35 ns/fact (lane-posh-02-batch.md:43,111), 2.3x
over the 10 ns/fact target; the 10 ns figure is marketing and is
withdrawn. Reported strictly in derived facts/s.' The <2 ms closure
gate is DELETED below - this row must not inherit it.
Row Commit <4 us: mark TARGET until a batch-amortized receipt exists
(PRD-4 SS7 claim-format law; no current receipt in PRD-1's table).
Row Footprint <32,768 B Flash: rewrite honest - 'SRAM VERIFIED:
36,048 B, headroom 95,024 B (verify_cscb_sram.py exit 0, PRD-1
NFR-03); Flash ceiling and stack bound remain UNVERIFIED (stack
bound is self-declared).'
Row Determinism: STAYS - split-PRNG seed law is court-backed
(corpus/algorithm seed split per repo CLAUDE.md; mutation courts).
Whole-table note (PRD-1 SS5 weakest links): zero-alloc claims travel
with the scoped fence (posh_alloc_sentry_court.rs:8-24; 2,662 allocs
/ 650,294,994 B measured whole-pipeline) - 'zero-allocation' without
the scope fence is VACUOUS and must not appear.

### SS4.1 Execution model :84-98 - STAYS with receipts
Items 1-4 are design law. Receipts: fresh AE4 exit 0 (items 1-2);
no_std cgap-blake3 Cargo.toml:5 with allowlisted RV32 residuals
cmov_u64/eytzinger_restore/select_slice (PRD-1 NFR-06, residuals not
hidden). Item 3 SWAR: keep, design law, no per-byte claim.

### SS4.2 Concurrency design-around :100-110 - STAYS
Prohibition clauses; consistent with PRD-8 W2 (0 allocs mmap
transport, ingress_ring SPSC law, ingress_ring.rs:20-30 module doc).

### SS4.3 Interface boundaries :112-119 - STAYS + caveat (PRD-8)
Typed-refusal sentence: VERIFIED (sovereign.rs:413 cgap_admit_frame,
typed refusals; ref_store.rs:48-75 Refusal::tag; GIT-09
admission_gate AndonRefusal). ADD caveat sentence per PRD-8 W3: all
cross-boundary Go/POSH claims are bounded by the frozen 11-symbol
admit/eval/commit/verify C-ABI surface until cgap_ingress_bytes,
cgap_radix_join, and the cgap_shm_* set land (all NO-GO today).
Do NOT import PRD-8's AshA2A clause set - zero in-tree wiring
(PRD-8 W0).

### SS5 Governance :121-138 - STAYS
Discipline clauses, not measurements. Receipts exist for the
pattern: chain_mutation_court (mutation flip witnessed), POSH-4
positive-control-first court, e5 warmup honesty pin
(edge_closure_courts.rs:422-477, PRD-4 SS0).

### SS6 Doctrine :143-157 - REWRITE one clause (:156)
'sub-15 ns admission ... sub-4 us sovereign commits' currently reads
as landed capability. Rewrite per PRD-4 SS7 honesty law: 'sub-15 ns
admission and sub-4 us sovereign commits are TARGETS, unmeasured in
this charter; the independently shippable grounds are the landed
courts: typed-refusal admission, SWAR filtering, lease-bounded
commits.' '24 B 3-column segments' at :152 STAYS (receipt as P2).

### Footer JSON :159-161 - REWRITE after landing
Replace files_written/commands with this rewrite lane's receipt;
keep standing ALIVE only if Verify passes the falsifier.

## DELETED (invented lore, no rewrite)

1. 'mod Sigma_PQC' as conservation-law fact (:19) - superseded by
   the PRD-2 typed-trailer rewrite above; the modular framing is
   deleted outright, not annotated.
2. Sub-ms checkpointing as current capability - no paragraph in
   12-ard carries it (FR-06 DEFECT-OPEN, PRD-1; nearest in-tree
   mark: dflss/02-ax-charter-ctq.md:68 NO-GO), and PRD-4 SS7 makes
   it TARGET-only until the batch-amortized receipt lands. The
   rewrite must NOT import it from 14-dflss-charter.md or the CTQ
   tree.
3. '<2 ms closure' as a release gate - profile-ambiguous per PRD-1
   NFR-02 (1.873 ms is DEBUG best-of-3, lane-git-12:60; release
   witness non-repro lane-b02:32; 36.9 s standalone lane-mv-05:66).
   The gate framing is deleted from SS3.2's materialization row;
   honest per-profile wording replaces it.

## Sequencing

1. Apply SS1 equation rewrite (PRD-2 text, verbatim).
2. Apply SS3.2 NFR row rewrites + zero-alloc scope fence note.
3. Apply SS6 :156 target-marking rewrite.
4. Add P2 Quad64 clarify sentence and SS4.3 11-symbol caveat.
5. Replace footer JSON; run PRD Verify against the falsifier.

## See Also

audit/cgapprd-01-prd-audit.md | audit/cgapprd-02-equation-grounding.md
| audit/cgapprd-03-frame-layout.md | audit/cgapprd-04-checkpoint-
design.md | audit/cgapprd-08-boundaries.md | 12-ard-prd-cgap.md
