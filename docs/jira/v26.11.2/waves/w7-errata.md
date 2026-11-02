# W7 drift errata — consolidated receipt

Lane: F4 (W7 drift errata wave). Repo /Users/sac/cgap, branch main,
2026-11-02. Method: metagate sweep over wave receipts against the
p4-terminus Table 4 standard (zero witnessed command+exit = VACUOUS);
each correction is an APPENDED dated line in the source receipt, never
a rewrite of original claims.

## Errata items

| # | receipt | drift | action |
|---|---|---|---|
| 1 | `waves/sm-4-hooks.md` | 3 non-reproducible claims: (a) default marker path `~/.claude/hooks/publish-authority.token` — script default is repo-local `.claude/hooks/publish-authority.token` (`publish-guard.sh:7`); (b) "parse error/absent command exits 2" — script exits 3 (loud skip) on empty CMD (`publish-guard.sh:16`); exit 2 is the block path (`:30`); (c) Court-evidence exit codes derived from (b). Consistent with the W2-19/D7 addendum already in that receipt. | ERRATA line appended 2026-11-02 |
| 2 | `waves/pub-w3-witness.md` | VACUOUS-class: claims standing ALIVE with ZERO witnessed command+exit blocks (outcome tables only). | marked per p4 metagate standard; ALIVE reclassified UNVERIFIED |
| 3 | `waves/pub-w5-witness.md` | VACUOUS-class: no command/exit blocks; poll terminated PENDING with no observed terminal state. | marked per p4 metagate standard; rows UNVERIFIED |
| 4 | `waves/bp-4-rules.md` | Rule-loading self-marked UNVERIFIED (next-session only); census confirms the mark and annotates STILL-UNVERIFIED — no frontmatter-parsing court has run. | annotated in place (appended line) |
| 5 | `waves/g1k-05-machinery-share-audit.md` | In-place correction verified PRESENT (lines 23-25: root `benches/` is EMPTY / 0 tracked; live bench surface is `crates/bench/benches/`, 15 files) — recorded, no edit needed. | verified |
| 6 | `docs/jira/v26.11.2/repairs/repair-05-production-wire.md` | Coordinator-assembled receipt (lane 429-terminated; coordinator ran `cargo test -p cgap-gate` / `-p cgap-branchless`, exits 0, 2026-11-02). Verification present but executor is coordinator, not the lane. | recorded as coordinator-witness class |
| 7 | `docs/jira/v26.11.2/convergence/lane-03-closure-arena-hardening.md` | Coordinator-assembled; fix W4-verified on disk STATICALLY only — closure_ladder bench leg NOT re-executed by any witness (typed OPEN owed). | recorded as coordinator-witness class, verification owed |

Items 6-7 form one class: coordinator-written receipts where the owning
lane died pre-witness; both disclose their executor honestly, but
lane-03's perf claim remains UNVERIFIED until the bench leg reruns.

## Standing

ALIVE — all appended lines written and verified on disk this session
(diff below); no source edits, no git writes. Items 2-3 stand
UNVERIFIED until successor witnesses embed real command+exit blocks.

## JSON footer

```json
{
  "lane": "F4-w7-errata",
  "date": "2026-11-02",
  "items": 7,
  "appended_lines": ["sm-4-hooks.md", "pub-w3-witness.md", "pub-w5-witness.md", "bp-4-rules.md"],
  "verified_only": ["g1k-05-machinery-share-audit.md"],
  "coordinator_class": ["repairs/repair-05-production-wire.md", "convergence/lane-03-closure-arena-hardening.md"],
  "standard_cited": "waves/p4-terminus.md:65-82 (Table 4 metagate)",
  "standing": "ALIVE"
}
```
