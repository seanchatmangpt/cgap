# PP-6 Comparison-Honesty Audit Receipt (paper-comparison-audit.md)

- Lane: PP-6-comparison (court-auditor, read-only except this receipt)
- Date: 2026-11-01
- Repo: /Users/sac/cgap

## Command ledger (auditor's own runs)

| # | Command | Exit |
|---|---|---|
| C1 | `grep -rl "RDFox" /Users/sac/cgap --include="*.md" --include="*.ttl"` | 1 (zero matches) |
| C2 | `grep -rn "RDFox" /Users/sac/cgap/docs/jira/v26.11.1` | 1 (zero matches) |
| C3 | `grep -ril "rdfox" /Users/sac/cgap` (full repo) | 1 (zero matches) |
| C4 | `grep -rl "position paper" /Users/sac --include="*.md"` (home sweep) | 0; no cgap position paper in results |
| C5 | `grep -rn "82.24\|GB/s" .../docs` | 0 (hits listed below) |

## Finding 0 — audit subject MISSING

The cgap position paper (PP-1 output) HAS NOT LANDED. Zero RDFox mentions
exist anywhere in /Users/sac/cgap (C1-C3). The paper-comparison-audit lease
was correct to anticipate this; every RDFox-specific row below is therefore
audited as SUBJECT-MISSING, not as a claim verdict on absent text.

## Per-row verdict table

| Row | Claim text (source) | Verdict | Evidence |
|---|---|---|---|
| R1 | "82 GB/s contiguous scan vs RDFox 2-6.1M facts/sec materialization" | NOT PRESENT in any cgap doc (subject missing). Had it been present: CATEGORY-ERROR — scan bandwidth (B/s over a contiguous array) vs materialization throughput (facts/s over index construction) are different physical quantities; no lawful ratio exists between them. | C1-C3 |
| R2 | RDFox internal characterization as vtables / pointer-thrash naive tries | NOT PRESENT. Had it been present: UNVERIFIED and CONTRADICTED by public materials — RDFox uses dictionary encoding + compressed integer indexes (columnar), not naive pointer tries. A diagram must compare against RDFox's documented design. | C1-C3 |
| R3 | cgap materialization throughput vs RDFox | NOT PRESENT. cgap's own materialization throughput is UNMEASURED (RDF-1 wave lanes own that measurement). Any performance-delta phrasing vs RDFox is unlawful until then; only architectural-difference phrasing is admitted. | C2; waves/ ledger has no materialization-throughput receipt |
| R4 | CHALLENGER_SALES_REPORT.md:37 — "82.24 GB/s scan ... Standard graph engines (<100 MB/s) ... 82x margin" | CATEGORY-ERROR + UNVERIFIABLE baseline. cgap side is receipted (bench-bn3-receipt.md:69). The "<100 MB/s" baseline cites no source and no engine; an unnamed-baseline ratio ("82x") must be deleted or sourced. Also internally inconsistent: same row's margin column says "82x over 1.0 GB/s wire rate", conflating two different baselines in one row. | Read of file |
| R5 | REVENUE_GENERATION_COMPENDIUM.md:246 — "Replaces slow RDF/SPARQL/OWL/Prolog engines with line-rate deterministic reasoning" | UNVERIFIABLE. No RDF/SPARQL/OWL/Prolog engine was executed in any cgap receipt; "slow" is an unsourced comparative adjective. Restate as architectural difference. | Read of file |
| R6 | 82.24 GB/s scan figure itself | LIKE-FOR-LIKE vs its own >1.0 GB/s charter target: receipted. bench-bn3-receipt.md:45,69,76 (82.24 GB/s Divan fastest, 19.12 us, arithmetic shown); bench-chains-bn3.md:37 shows 80.32 GB/s isolated / 12.3 GB/s chain — the 82.24 figure is the isolated-scan number and docs must not cite it as end-to-end chain throughput. | bench-bn3-receipt.md, bench-chains-bn3.md |

## Required restatements (for the corrected paper, when PP-1 lands)

1. "cgap-bn3 scans its 24-byte cache-aligned statement arena at 82.24 GB/s
   (isolated Divan bench, receipt bench-bn3-receipt.md). This is memory-bus
   scan bandwidth, not rule-materialization throughput."
2. "cgap's own materialization (fixpoint project) throughput is not yet
   measured; the RDF-1 wave's lanes measure it. A lawful RDFox comparison
   awaits that measurement; until then any RDFox statement is limited to
   architectural difference (dictionary-encoded compressed integer indexes
   per RDFox public materials vs fixed-width 24B slot arena), never
   performance delta."
3. Delete the "Standard graph engines (<100 MB/s)" baseline and the "82x"
   ratio in CHALLENGER_SALES_REPORT.md:37, or source the baseline.
4. Delete "slow" from REVENUE_GENERATION_COMPENDIUM.md:246 or run an
   executed RDF/SPARQL engine baseline.

## Standing

PARTIAL — the audit ran with real commands (non-vacuous), but its primary
subject (the position paper) does not exist yet; verdicts R1-R3 are
forward-binding constraints on PP-1's text, and R4-R6 bind existing docs.

## Divergences

- The lane lease named a position paper via "PP-1's grep result"; PP-1 has
  not landed, so the expected subject was absent. This is reported, not
  repaired (auditor is read-only).
