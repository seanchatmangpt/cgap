# AHDCLT: Adversarial Hyperdimensional Cognitive Load Theory

Status: REVISED doctrine (post-redteam, 2026-11-02). The compute-kernel
half of Pillar 1 is RETIRED-REFUTED (see below); the statutory-vice
half (P28-SIL mapping) is intact. The surviving moat is
state-chaining + adversarial courts + C-ABI integration cost.

## Retired-Refuted: compute-kernel "epistemic unconstructability"

The original doctrine claimed compute kernels (SWAR sieve, prefilter
mix, 24 B frame scan) were "technically unconstructable without the
original substrate" because predicates were "load-bearing in the
type/shape system". This half is RETIRED-REFUTED as of 2026-11-02.

Fired falsifier: RED-01 (`docs/jira/v26.11.2/redteam/
red-01-cleanroom-bypass.md:159-164`) — a hostile C99 clean-room
rebuild constructed from the repo's own doc comments alone (every
constant and mask printed in prose) passed differential and exhaustive
courts (524,288-lane swar_lt sweep, 0 fails) at kernel-level perf
parity-or-better. The substrate's documentation was itself the
complete construction recipe. RED-02 (`red-02-llm-extraction-exploit.md:104-124`)
additionally found zero anti-LLM tripwire mechanism in code: the
chunked map-reduce extraction pipeline ran undetected, so the Pillar-2
"tripwire" claim is decoration as written.

Per the doc's own register rule (every claim fails closed to UNKNOWN
until falsified adversarially), these halves are retired, not
weakened. No "unconstructable" or "anti-LLM tripwire" claim may be
cited from this document going forward.

## Surviving moat (what actually resisted, measured)

The redteam confirmed the real extraction cost sits in three layers:

1. State-chaining: the admit/commit frame digest chain (BLAKE3
   substrate) and BN3 golden vectors pin exact lane outputs over real
   bytes; return-constant stubs cannot reproduce them
   (red-03-ffi-shim-decoupling.md:99-111).
2. Adversarial courts: byte-identity, spoliation-flip
   (CGAP_ERR_P28_SPOLIATION 0x5032_38FF), determinism, and digest
   courts crack every sovereign stub — "an enterprise can rewire the
   WIRE in <48h; it cannot rewire the PROOF"
   (red-03-ffi-shim-decoupling.md:113-121).
3. C-ABI integration cost: full semantic decoupling of the dlopen
   surface at the measured ~41h stub-only budget still fails every
   byte-identity/spoliation/golden-vector court; true parity requires
   porting the court set, which is the integration cost the doctrine
   actually buys (red-03 §7 SURVIVING verdict).

## Extraction-deterrence boundary (explicit)

All gates and courts (boundary_gate.py, zero_string_gate.py,
cfg_homology.sh, symbol_gate.py, et al.) court the ORIGINAL tree's
properties. A foreign rewrite in another repo never enters their
subject universe — no on-disk mechanism sees it (RED-02 finding,
red-02-llm-extraction-exploit.md:119-120). The moat therefore binds a
DOWNSTREAM consumer running cgap's courts against a provider, not a
hostile rewriter outside the court perimeter. Deterrence beyond that
boundary is legal (statutory vice), not mechanical.

## Pillar 1 — Statutory & legal vice (INTACT, unaffected by RED-01)

Low-level compute operations are annotated so that they map directly
onto statutory predicates: IEC 61508 SIL ratings (P28-SIL), Class-I
safe-harbor conditions, export-control classifications. Consequences:

- For an auditor or court, conformance is a mechanical check over the
  mapping, not a litigation over intent.
- For an unauthorized fork, the same mapping is toxic: stripping or
  bypassing the predicate annotations breaks the legal envelope, so
  the fork inherits statutory exposure it cannot reconstruct.

This layer was NOT under test in RED-01 (see red-01 §5 scope limit:
the P28-SIL mapping at LICENSE-P28-SIL.md / crates/cgap-ffi/src/
sovereign.rs is a different layer and is not voided by the kernel
breach). RED-01 breach does not affect this section.

## Retired pillars (historical, do not cite)

- Pillar 2 (Anti-LLM cognitive tripwires): RETIRED-REFUTED 2026-11-02
  per RED-02 W5 (zero tripwire mechanism in code; W4 chunked pipeline
  undetected).
- Pillar 1 compute-kernel half ("technically unconstructable"):
  RETIRED-REFUTED 2026-11-02 per RED-01.

## Criticism Discipline

The doctrine remains a thesis at design-conversation register; every
surviving claim fails closed to UNKNOWN until falsified
adversarially. The pre-registered falsifiers #1 and #2 have now fired
(RED-01, RED-02). Remaining falsifier:

3. If verification cost grows with N dimensions (the turnstile
   becomes a linear scan over the constraint matrix), the asymmetry
   claim fails and the doctrine reduces to ordinary hardening.

Pending further adversarial testing, AHDCLT should be used as a design
lens — "does this structure make honest use cheap and circumvention
expensive?" — with the moat read as the measured three-layer form
above, not as unconstructability.

## See Also

- docs/jira/v26.11.2/redteam/red-01-cleanroom-bypass.md (fired
  falsifier, compute-kernel refutation)
- docs/jira/v26.11.2/redteam/red-02-llm-extraction-exploit.md
  (extraction-deterrence boundary)
- docs/jira/v26.11.2/redteam/red-03-ffi-shim-decoupling.md (surviving
  moat: courts, digest chains, ~41h integration budget)
- 15-arrow-microarch-probe.md
- 18-dflss-charter.md
