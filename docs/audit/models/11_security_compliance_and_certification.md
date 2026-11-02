# AU11 — Security, Compliance & Certification Timeline Model (Phase 2)

**Status:** ANALYSIS ONLY — no certification is held, applied-for, or claimed.
Every mapping below is an *analysis class* of what an evaluator would examine,
not an attestation of conformance. Baseline: v26.11.2, 2026-11-02.

## 1. Property → criteria-class mapping (all ANALYSIS)

| cgap property (grounded) | Certification criteria class | Grounding |
|---|---|---|
| AU3 memory discipline: linker-ASSERTed SRAM budgets (enclave_128k.ld, xip.ld), saturating-disarm allocator courts, spill-on-rotation durability (STAGED != DURABLE residual declared) | Memory-safety review class — evaluator walks boundary ASSERTs, allocator-reentrancy playbook, and the declared durability residual | docs/audit/metrics/03_sram_and_allocator_discipline.md |
| cgap-blake3 custom digest + BLAKE3 receipt chain (`RingReceiptChain`, GENESIS→rN, `hash = BLAKE3(prev_hash || block || license wire bytes)`; tamper court flips on a single prev_hash bit) | Crypto-module review class (FIPS 140-3 / CC crypto guidance) — ANALYSIS: a vendored/custom digest is reviewed as a cryptographic module, never assumed compliant | crates/cgap-ffi/src/receipt_chain.rs:44,102-105,179,309 |
| Constant-time/branch-free primitives with honest ISel residual: exact residual branch counts asserted (1/1, 2/2), rv32imc no-Zicond ceiling cited, machine-readable residuals.toml sourced from REP1 | Side-channel / timing-attack review class — evaluator checks constant-time claims against disassembly; the residual grid (`isel_residual`/`repaired_hot`/`skip_reference`) is exactly the artifact such a review demands, with counts asserted EXACTLY, not faked PASS | docs/audit/metrics/02_branchless_algorithmic_complexity.md:54-78; countermeasures/residuals.toml:20 (REP1 authoritative) |
| Fail-closed gate fleet (zero_datafusion, receipt_lint --strict, registry_sweep, boundary, release_check; exit nonzero on anomaly; "fail-closes: exit 0 iff every cell covered AND...") | Safety-instrumented-function (SIF) review class — gate failure = halt, not warn-and-continue | docs/jira/v26.11.2/waves/w2-20-terminus.md:8-20; scripts/registry_sweep.py:4-6 |
| Typed-refusal law: 100% `CgapStatus`, single opaque `ReceiptChainRefusal::Spoliation` class, "no failure fingerprinting" | Predictable-failure / opaque-refusal review (evaluator checks the refusal taxonomy is total and branchable) | docs/jira/v26.11.2/15-ctq-tree.md:67; receipt_chain.rs Refusal enum |
| No-Process.sleep gate law, bounded gates, explicit 0/1/2/3 exit contract | Deterministic-harness review (no timing nondeterminism in evidence generation) | docs/jira/v26.11.2/01-release-gates.md:14-16 |
| Publish-guard freeze: registry_sweep read-only, "Never publishes", publish records point to frozen coordination receipts | Release-management / configuration-management control class | scripts/registry_sweep.py:7,11,25-30 |

## 2. Calendar timeline model (ASSUMPTION ranges, standard public figures)

All ranges are STATED ASSUMPTIONS from publicly discussed norms, not quotes from any cgap doc.

| Certification class | ASSUMPTION calendar range | Notes |
|---|---|---|
| SOC 2 Type I → Type II | Type I: 3–6 mo; Type II: 6–12 mo incl. 3–12 mo observation window | Observation window is calendar-bound, not evidence-bound |
| Common Criteria (CC) evaluation | 12–24 mo for EAL-ish assurance packages (ASSUMPTION, widely cited norm) | Includes developer-evidence prep + lab queue |
| FIPS 140-3 crypto module | 12–30 mo incl. CMVP lab queue (ASSUMPTION) | Custom algorithm (BLAKE3) review adds tail risk |
| FedRAMP-ish SaaS authorization | 6–18 mo (ASSUMPTION) | Dominated by org process, not code evidence |
| ISO/IEC 27001 stage 1+2 | 3–9 mo (ASSUMPTION) | Policy/process-driven |

## 3. The honest asymmetry: what the ~198-receipt fleet compresses and what it does not

Evidence-readiness is unusually high: an evaluator can be flip-witnessed on courts
+ receipt fleet (~198 receipts across v26.10.31/v26.11.1/v26.11.2 wave/receipt
dirs; 48 receipt files in docs/jira/v26.10.31/receipts; 07-court-corpus.md:155
"The receipt is the flip witness; no receipt, no flip"; W7 metagate sweep,
waves/w7-errata.md — 7 drift items, 2 VACUOUS-class reclassified UNVERIFIED,
append-only errata law, metagate standard p4-terminus.md Table 4).

**Compresses (developer-side evidence production):**
- CC/FIPS *developer evidence production*: claim→implementation correspondence is
  already receipted and flip-witnessed, cutting the "produce + defend functional
  evidence" phase materially.
- *Re-demonstration on request*: courts re-run (w3-r3-court-reruns.md) mean a
  re-demonstration is a script run, not a rebuild.
- SOC 2 *control description* drafting: the gate fleet IS a control description.
- Side-channel review prep: the residuals.toml grid + exact-count census gives the
  evaluator mechanical court output instead of adversarial Q&A on each symbol.

**Does NOT compress (institution-side, calendar-bound):**
- **Org/legal process is not compressible by evidence quality.** SOC 2 observation
  window, CC lab queue, CMVP queue, auditor scheduling, committee sign-off chains,
  legal review: calendar- and institution-bound. A perfect evidence package arrives
  early to the same queue and waits the same time.
- Custom-digest tail risk: BLAKE3 chaining is a *shape* match to FIPS Merkle
  chaining, not a validated module; that review is additive, not reduced, by
  receipt quality.
- W7 errata class 6-7: coordinator-witnessed receipts disclose their executor
  honestly but stay UNVERIFIED until the owning leg re-runs — evaluator due
  diligence re-verification effort remains, and is correct to remain.

## 4. Soak / staging cycle assumption table

ASSUMPTION table (stated ranges, not measurements; cgap gates today are
single-run deterministic, not soak-tested).

| Cycle type | ASSUMPTION range | Applicable class |
|---|---|---|
| Deterministic re-run battery (current state) | hours–days | All classes — already exceeds typical evidence norms |
| Staging soak, representative workload | 2–6 weeks | SOC 2 Type II observation input |
| Soak with fault injection (spoliation flips) | 1–3 months | SIF / fail-closed review |
| Long-horizon availability soak | 3–12 months | SOC 2 Type II full window |
| CC re-evaluation / delta cycle | 6–12 months per major version | CC maintenance |

**Honest limit (repeated):** evidence quality compresses *developer-side*
effort only. Reviewer-side and institution-side timelines dominate the
calendar and are indifferent to receipt fleets.
