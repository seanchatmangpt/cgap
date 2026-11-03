# ADV-13 — Ops-Telemetry Theater Audit (A1 / A3 / A6 wiring truth)

Date: 2026-11-02. Lane ADV-13, adversarial corrosion wave. READ-ONLY
except this file. No git commands. Repo: /Users/sac/cgap (canonical
checkout, branch main). Attacks the post-FIX-14 / RC-05 / RC-13 ops
surface for "paper tiger" risk: a skeleton that moves counters only
inside its own court, while docs drift toward implying live telemetry.

## Method

Grep-first, not receipt-first: every wiring claim below is grounded in
`grep -rn` over the current tree this session, not in any lane's
receipt text. Receipts are quoted only to test whether their wording
matches the grep truth.

- G1: `grep -rn "stats::" crates/cgap-git/src/ crates/cgap-ffi/src/{sovereign,receipt_chain,oabi_lookup,xip_tables}.rs`
  → **exit 1, zero matches**. No production module imports or calls
  `stats::` anything.
- G2: `grep -rn "impl .*SloSource" crates/cgap-fleet-ops/`
  → exactly 2 matches, both in `tests/courts.rs:185,194`
  (`LiveSource`, `StalledSource` — court fixtures holding a
  hand-constructed `SloSnapshot`).
- G3: `grep -n "record_" crates/cgap-ffi/src/stats.rs` — `record_*`
  methods exist only in `stats.rs` itself; the integration recipe is
  a doc comment at stats.rs:222-232 ("Ring-owner integration —
  documented, NOT wired by this lane").
- G4: `Record`/`record_admitted` sweep of cgap-git/src — all hits are
  ring-local names (`RecordTooLarge`, `DemuxRecord`, ring quota
  methods `Record one class-101 refusal` doc comments); none call
  into cgap-ffi stats.

## Attack 1 — A1 wiring: does any counter move in production?

**Finding: CONFIRMED THEATER (contained).** Zero call sites (G1).
`stats::global()` increments NOTHING in any production path: no ring
ingress/refusal/quota site, no FFI entrypoint, no receipt_chain site.
Every counter advance witnessed by the RC-05 courts happens because
the court itself calls `record_*` — the recorder is driven by tests,
not by the kernel. `cgap_stats_snapshot` over the C-ABI faithfully
reports a block that stays at its post-reset state forever in a real
deployment.

### Honesty grading of the A1 row (fix-14:44-72)

PARTIAL is honest here — but only because the row carries the residual
verbatim, twice:

- fix-14:57-71 "HONEST RESIDUAL: the ring-owner integration ... is
  DOCUMENTED-NOT-WIRED ... Detection at runtime is therefore available
  at the ABI seam but not yet driven by kernel events." Exact, matches
  G1.
- RC-05 receipt Divergence 1 (lanes/lane-rc05-stats-block.md:74-79)
  states ingress_ring.rs/egress_ring.rs were NOT touched and the
  integration is a doc-comment recipe. Exact, matches G1.
- stats.rs:222-223 itself carries "documented, NOT wired" in-tree.
  Matches.

**No SPOILER in A1's row text.** The earlier fix-14 A1 census framing
was already self-corrected in-file (the "CENSUS AS-OF QUALIFIED
2026-11-02" block at fix-14:4-13). What WOULD be a spoiler: any prose
elsewhere claiming "runtime telemetry exists" without the
not-wired clause — none found this session in fix-14, RC-05, or the
tree's doc comments.

## Attack 2 — fleet-ops A6: what does the host poller poll?

**Finding: CONFIRMED THEATER (contained).** `SloSource` is a trait
with exactly two impls, both test fixtures in courts.rs:185,194 (G2):
`LiveSource(SloSnapshot)` returns a snapshot the court constructed
by hand; `StalledSource` returns a snapshot plus a hardcoded
`is_stalled() -> bool`. There is no impl backed by
`cgap_ffi::stats::cgap_stats_snapshot`, by a C-ABI FFI call, by a
file, socket, or process — by any real kernel source. The `poll_until`
bounded-poll loop is real machinery polling a fixture. In production
the A6 "host poller" has nothing to poll: the seam is typed and
checksummed, but both ends of the wire are unwired (A1's kernel half
mints nothing; A6's host half reads nothing).

### Honesty grading of the A6 row (fix-14:151-175)

PARTIAL "(forensics + host poller)" is honest as written, because it
states explicitly (fix-14:170-174): "STILL MISSING: SLO counter
minting in cgap-ffi (kernel half, depends on A1 ring integration
landing), the `SloSource` -> kernel feed wiring, and any
paging/escalation surface." RC-13's receipt likewise discloses
"`SloSource` is a read-only seam with no kernel feed wired"
(fix-14 A3 row quotes it; lanes/lane-rc13-fleet-ops.md confirms the
crate is a skeleton with 4 courts). **No SPOILER.** One wording nit:
calling the host half "host poller" in the row header without the
polls-nothing qualifier invites skim-level overreading; the STILL
MISSING block is the load-bearing qualifier and it is present.

## Attack 3 — Theater verdict per row

Taxonomy: (a) documented-skeleton-honestly-labeled = immune;
(b) skeleton-labeled-as-capability = SPOILER.

| Row | Text says | Tree says | Verdict |
|---|---|---|---|
| A1 | PARTIAL, residual "DOCUMENTED-NOT-WIRED" quoted twice | G1: zero production call sites | (a) IMMUNE — honest |
| A3 | PARTIAL; R-A3.1 device identity "NOT minted in cgap-ffi yet" stated | fleet-ops is self-contained, no kernel claims | (a) IMMUNE — honest |
| A6 | PARTIAL "(forensics + host poller)", STILL-MISSING block names the unwired SloSource feed | G2: only fixture impls of SloSource | (a) IMMUNE — honest, with the skim-risk nit above |

**Aggregate verdict: NO SPOILER-CONFIRMED rows.** All three rows are
category (a): the skeleton is real, mutation-tested, and honestly
labeled as unwired at every layer (requirements register, lane
receipt, in-source doc comment). The theater risk is REAL but
LATENT: the counters move only under court stimulus, and the only
thing preventing drift into category (b) is the presence of the
residual clauses. Any future doc that drops the "DOCUMENTED-NOT-WIRED"
qualifier while citing A1/A6 as telemetry capability flips this
verdict to SPOILER-CONFIRMED.

## Fence obligations this audit closes

- FIX-14 A1/A6 rows must keep their residual clauses verbatim through
  any future edit; removal is a detectable regression (this file is
  the witness of the 2026-11-02 wording).
- RC-05 Divergence 1 and RC-13's SloSource disclosure are the
  authoritative wiring statements; a later receipt contradicting them
  without a new grep is hearsay.
- When ring integration lands, the falsifier for THIS report is:
  `grep -rn "stats::" crates/cgap-git/src/` exits 0 with call sites
  inside push/admit/refuse paths, and a production-path court (not
  court-driven record calls) advances counters. Until that grep
  flips, "telemetry is live" is UNVERIFIED and must not be claimed.

## Standing

- Commands this session: G1 grep exit 1 (zero matches); G2 grep
  exit 0 (2 fixture impls); G3/G4 greps exit 0; `sed` reads of
  fix-14, lane-rc05-stats-block.md, lane-rc13-fleet-ops.md,
  stats.rs, courts.rs — all exit 0. No builds, no tests, no git
  commands (READ-ONLY lane; no cargo evidence minted, none needed —
  the wiring question is a static grep question).
- A1/A3/A6 row verdicts: all (a) IMMUNE-honest as of this session's
  greps. Live-telemetry status: NOT WIRED (verified), not PARTIAL-by-
  inference.

