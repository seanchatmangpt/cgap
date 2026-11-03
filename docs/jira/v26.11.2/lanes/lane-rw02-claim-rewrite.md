# Lane RW-02 — claim rewrite (README + REVENUE_COMPENDIUM), 2026-11-02

W1 subject: /Users/sac/cgap (main). Owned: README.md,
docs/REVENUE_GENERATION_COMPENDIUM.md. Read-only ground:
docs/jira/v26.11.2/11-measured-claims.md (read in full).

## W2 measure

- Dispatch ground rule: dispatch cites "FIX-12"; no FIX-12 receipt exists
  on disk (grep across docs/ = zero hits; independently recorded by GAP-03,
  dflss/00-dflss-master-dossier.md:76-79,166-167). The demotion the
  dispatch attributes to FIX-12 is embodied in claims row 29 (UNSUPPORTED
  "constant-time") + red-20 §2; §6.3's TARGET labels + byte≠dollar caveat
  were found ALREADY landed in the working tree (lines 308-311 pre-edit).
- README compendium references: NONE found — README has no
  compendium/$50M reference to align with LICENSE §2.4. Nothing to
  restore; recorded as divergence.
- Code constants verified: DEFAULT_SAFE_HARBOR_CEILING_CENTS =
  5_000_000_000 (crates/cgap-p28/src/asc450.rs:74);
  SAFE_HARBOR_CEILING_MAX_CENTS = 5_000_000_000 (attestation.rs:58).
- LICENSE.md:13 advertises "the $50M universal safe harbor" — that is the
  §2.4 gross-revenue FLOOR, distinct from the asc450 $50M accrued-liability
  CEILING. Compendium §10.2 previously read as a single concept.

## W3 diff (per-claim before/after)

Compendium (all edits merge-only; no line clobbered):
1. §1.1 "4.06x to 82.24x empirical speedups" + "branchless constant-time
   determinism" -> receipted-quiet-host framing; 82.24 GB/s named a
   byte-throughput figure (never a facts/s multiple, 06-bench-protocol §7);
   "constant-time" -> UNSUPPORTED w/ register row 29 cite.
2. §1 mermaid: "82.2 GB/s BN3" -> "+(quiet-host)".
3. §2.2 82.24 GB/s -> + quiet-host/F2-13 cite, UNVERIFIED-WITHOUT-METHOD
   for v26.11.2 register, facts/s unit-law note.
4. §3.1 Mode 2 -> Controlled Group §2.2.1-2.2.2 group aggregation added.
5. §3.1 NEW bullet "Numeric Safe-Harbor Floor (Section 2.4, v2.1.1 rider)":
   < $50M gross revenue presumed exempt; >= $50M duty (>= boundary);
   floor vs code ceiling distinction; cites LICENSE-P28-SIL.md §2.2.1-2.4
   + REVISION RIDER v2.1.1 (2026-11-02).
6. §4.1 ~459-460 ns/record, §4.3 ~835 ns round-trip -> + quiet-host +
   UNVERIFIED-WITHOUT-METHOD-for-register qualifiers.
7. §5 Tier 1 "5 Published Crates" -> + registry-truth note (9 published
   at 26.11.1, README registry table; 5 = kernel-crate subset).
8. §6 NEW scope banner: all §6.1 figures quiet-host v26.11.1 F2 receipts;
   red-10 contention divergence (row 28: slowest 124.06 ns @1M = 12.2x
   10.20 quiet-host); UNVERIFIED-WITHOUT-METHOD where no register row.
9. §6.1 penstock row 7.71 ns -> + "register row 7, MEASURED" cite.
10. §6.3 line 310 -> + quiet-host + UNVERIFIED-WITHOUT-METHOD qualifier.
    (FIX-12 TARGET labels + truncation caveat verified intact — no restore.)
11. §8.1 NEW scope block: standings as-of-date v26.11.1 dod-five-gates;
    GATE-5 30,157 B is rlib/audit-profile, NOT linked-ELF (linked ELF
    150/65,536 B flash per README/gate-5-silicon.md).
12. §10.2 $50M ceiling -> + asc450.rs:74 / attestation.rs:58 cites +
    floor-vs-ceiling non-conflation sentence.
13. §11 preamble -> + as-of-date v26.11.1 + re-read-at-use-time law.

README: NO EDITS. Every figure already carries an inline receipt cite
(dod-five-gates.md, bench receipts 2026-11-01, registry API 2026-11-02,
gate-5-silicon.md). Sweep found no unreceipted quantitative claim.

## W4 verification

- "python3 scripts/receipt_lint.py" -> exit 0; zero hits against either
  owned file (pre-existing 472 claim/1635 struct hits elsewhere,
  unchanged by this diff — fence stated + run).
- awk length>100 sweep: my authored blocks all <=100 chars; the file's
  ~200 pre-existing >100-char lines left untouched (merge, not clobber).
- Section-integrity probe: 6.3 TARGET text + byte-reduction caveat
  present; row-29 cite + floor bullet count confirmed in bytes.
- README untouched (git diff = empty for README.md).

## W5 fences (must-not-regress, run)

- receipt_lint.py (doc quantitative-claim gate) — run, exit 0, no new
  hits from owned files.
- No code/gate files touched; boundary_gate/zero_string_gate surfaces
  untouched by construction (docs-only diff).

## W6 standing

ALIVE for the rewrite itself (edits on disk, verified via git diff +
integrity probe). No benchmark was re-executed by this lane; all figures
carried over keep their receipt-cited/v26.11.1 as-of-date status and the
new UNVERIFIED-WITHOUT-METHOD labels are honest labels, not new
measurements.

## W7 divergences

1. Dispatch cited "FIX-12": no such lane/receipt id exists under docs/
   (zero grep hits; GAP-03 dossier lines 76-79 records the same). The
   demotion lives in claims row 29 + red-20 §2; §6.3's demoted state was
   already on disk — verified intact, nothing re-demoted.
2. README carries no compendium/$50M reference; the "check README
   compendium references match §2.4" instruction had no referent.
3. Compendium "5 Published Crates" vs README registry truth (9) —
   reconciled via note, not by rewriting the tier claim.
4. Receipt file is the deliverable named by the lane template; no other
   report files written.
