# Workstream Receipt Map (R7 closure artifact)

Date: 2026-11-02. Author: lane GAP-10. Subject: mapping the four
workstreams named in release-blockers R7
(`/Users/sac/cgap/docs/jira/v26.11.2/audit/release-blockers.md:106-118`)
and doctrine-ship-plan A12
(`/Users/sac/cgap/docs/jira/v26.11.2/audit/doctrine-ship-plan.md:79-81`)
to whatever covering receipts exist on disk. UNVERIFIED never rounds up.

## Method

Word-boundary grep `\b(F5|F10|WG2|WG3)\b` across
`/Users/sac/cgap/docs/jira/v26.11.2/{lanes,waves,audit,dflss}` and
`/Users/sac/lsp-max/docs/` (run this session), plus targeted file reads.
A row is covered only by a receipt file on disk whose commands+exits
apply to the workstream's landed code; each cited receipt re-read this
session.

## Rows

| Workstream | Covering receipt(s) | Standing |
|---|---|---|
| F5 | `/Users/sac/lsp-max/docs/lane-f5-workspace-integration.md:11-17` — ALIVE on its own session: `cargo check -p branchless-lsp` exit 0, `cargo test -p branchless-lsp` exit 0 (21 passed / 0 failed), `cargo build --release` exit 0. Coverage note: predates the G04-G07/LSPA expansion of branchless-lsp; covers integration only. | PARTIAL |
| F10 | NO COVERING RECEIPT FOUND. `\bF10\b` matches only the *absence* references (release-blockers.md:106-118; dflss/00-dmadv-rollup.md:186; doctrine-ship-plan.md:79). No receipt file in either repo names F10. | UNVERIFIED |
| WG2 | (a) `/Users/sac/lsp-max/docs/lane-wg2-tier2-wiring.md:52-68` — ALIVE (63 lib + 5 + 6 + 3 = 77 tests, exit 0; `CARGO_TARGET_DIR=/tmp/wg2-target cargo test -p branchless-lsp`); (b) `/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-g05-lsp-wire-rebuild.md` (wire rebuild, ANDON_REFUSED witness count 3); (c) `/Users/sac/cgap/docs/jira/v26.11.2/audit/dod-lsp-max-wire.md:44,124` — WG2 live-wire branchy/clean flip re-verified VERIFIED-DONE on the fresh release binary, 2026-11-02. | ALIVE |
| WG3 | (a) `/Users/sac/lsp-max/docs/lane-wg3-parameterized-attrs.md:56-60` — ALIVE (gate exit 0 twice, 63 lib + 5 + 6 + 3 all green); (b) `/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-g04-lspr-attributes.md` (lspr attributes surface). | ALIVE |

## Honest limits

- WG2/WG3 rows map to receipts of the lsp-max lane family (dispatch
  named lane-g04/g05/g06/g07-lsp-*); G06 (`lane-g06-lsp-didsave-court.md`)
  and G07 (`lane-lspa-01`'s parent `lane-g07-lsp-conformance-seal.md`)
  cover adjacent surfaces (did_save court, conformance seal), cited in
  `audit/dod-lsp-max-wire.md:18-22` as receipts re-read at use time.
- F5's receipt covers only the original integration (21 tests); the
  suite is now 90+ tests per `audit/dod-lsp-max-wire.md:8-21` — so F5
  stands PARTIAL, not ALIVE-for-current-surface.
- F10 remains genuinely uncovered; R7 stays OPEN for F10 until a
  receipt lands or the coordinator retires the token.
