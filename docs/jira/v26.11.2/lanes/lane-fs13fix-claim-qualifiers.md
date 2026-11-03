# Lane FS-13FIX — claim qualifiers over living sales docs

Date: 2026-11-02. Closes DOD-16/FS-13's 9 typed violations
(`dod/dod-16-claim-sweep-verdict.md`) in the two owned files.
Owned: `docs/REVENUE_GENERATION_COMPENDIUM.md`,
`docs/CHALLENGER_SALES_REPORT.md` only (+ this receipt). No git
commands. Prior lanes' scope banners (X-04FIX, RW-02) extended, not
duplicated. Qualifiers only — every number kept verbatim.

## Per-site fixes (before -> after)

1. CHALLENGER_SALES_REPORT :21 — "deterministic, constant-time
   execution" -> "deterministic, branch-free execution ("constant-time"
   is UNSUPPORTED — register row 29, `11-measured-claims.md`)".
2. CHALLENGER_SALES_REPORT :23 — "Constant-time ladder searches" ->
   "Branch-free ladder searches (mechanism descriptor — row 29
   UNSUPPORTED)" AND the unscoped ">10 GB/s" gains an explicit
   UNSOURCED marker (no register row; nearest receipted figure named:
   challenger_crypto 461.8 MB/s, compendium §11).
3. CHALLENGER_SALES_REPORT :48 — "side-channel immune" -> "~27%
   latency reduction; side-channel immunity claim UNSUPPORTED (row
   29; mechanism is branch-free code shape)".
4. COMPENDIUM :240 — "`oblivious` constant-time gather" ->
   "`oblivious` branch-free gather ("constant-time" is UNSUPPORTED —
   register row 29)".
5. COMPENDIUM :241 — "constant-time `ladder_search`" -> "branch-free
   `ladder_search` ("constant-time" is UNSUPPORTED — register row
   29)".
6. COMPENDIUM :260 — "**82.24 GB/s** scan" -> "**82.24 GB/s**
   isolated-scan, quiet-host divan — BN3-internal only, not chain
   throughput; UNVERIFIED-WITHOUT-METHOD for v26.11.2, receipted
   `bench-bn3-receipt.md:45,69`; see §6 scope banner" (wording style
   matches the sibling X-04FIX banner in CHALLENGER_SALES_REPORT).
7. COMPENDIUM :313 — F2-13 row now carries the same quiet-host /
   isolated-scan / UNVERIFIED-WITHOUT-METHOD scope inline (table
   made internally consistent with the F2-11 row-cite pattern).
8. COMPENDIUM :321 — "100% Constant-Time Side-Channel Immunity" ->
   "Side-channel immunity UNSUPPORTED (row 29; demonstrated
   mechanism: `0` data-dependent branches)"; sweep figure reworded
   "branch-free full sweep".
9. COMPENDIUM :486 — "309.2 ns constant-time OID ladder search" ->
   "309.2 ns branch-free OID ladder search; "constant-time" is
   UNSUPPORTED — register row 29".

## COURT (flip)

FS-13 patterns re-run this session over both files (python window
court: ±3-line window; bare-82.24-without-scope and
constant-time/side-channel-immune-without-row-29):

```
violations: 0
```

9/9 typed violations closed. Exit 0. Anti-vacuity: the court's window
test would flag any of the pre-fix lines (verified by construction —
the pre-fix lines carried neither scope terms nor row-29 cites).

## Fences

- FS-10 constant-time census (NEGATING/MECHANISM classes): not
  contradicted; wording aligned to its mechanism-descriptor rule.
- Row 29 UNSUPPORTED standing (11-measured-claims.md:53): intact,
  re-read this session via the verdict doc; no claims-table file
  edited.
- Prior lanes' banners (RW-02 §6 scope, X-04FIX §2 scope): untouched.
- Frozen bytes (license URN, seeds, status codes): untouched.

## Divergences

- ≤100-char line rule not fully achievable inside GFM table rows and
  the §11 fenced command block — wrapping would break table/block
  structure; those rows remain long (pre-existing style in both
  files). Prose lines wrapped where possible.
- The git diff on both files also carries prior lanes' uncommitted
  banner edits (RW-02, X-04FIX, TC-08 etc.) — not this lane's diff;
  attribution is per the disjoint-ownership law.
