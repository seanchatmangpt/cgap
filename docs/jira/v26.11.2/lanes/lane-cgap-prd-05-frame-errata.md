# lane-cgap-prd-05-frame-errata (2026-11-02)

Docs-only errata lane implementing PRD-3 path (a); no code moved. Register
of truth: `audit/cgapprd-03-frame-layout.md`.

## Diff summary per file

1. `docs/DFLSS_CHARTER.md:30-33` — scoped the 64-byte claim: "64-byte
   hardware cache alignment verification" now parenthesizes Quad64
   (`layout.rs:36`) and states the CSCB fact frame is a separate 24-byte =
   3x u64 LE frame, FRAME_BYTES=24. Three-line rewrite, all lines <=100.
2. `docs/jira/v26.11.2/lanes/lane-b03-cscb-c4.md:65-68` — appended-only
   errata section: line 41 "(64-byte frame chunks)" is stale (landed frame
   is 24 B); original text kept verbatim per append-only law.
3. `docs/jira/v26.11.2/00-ard-prd.md` (tail) — 3-line clarifying note: CSCB
   fact frame = 24 B (3x u64 LE), ring bound 32,768/24 = 1,365 facts; 64 B
   is cgap-arrow Quad64 only. No per-byte claim existed there before.
4. `docs/jira/v26.11.2/15-ctq-tree.md` (tail) — 2-line layout note, same
   content, cites the audit doc.
5. `docs/jira/v26.11.2/14-dflss-charter.md` (tail) — 2-line layout note,
   same content, cites the audit doc.

## Commands + exits (this session)

- `awk 'length > 100 ...'` over the 3 jira docs edited by append: 0 over-
  100 lines (exit 1 from awk = no matches, gate pass).
- `awk` over DFLSS_CHARTER.md: many PRE-EXISTING >100 lines (46); my three
  rewritten lines are 95/86/86 — within the prose law for the diff itself.
- `grep -rn '64-byte frame|64 B frame' docs/ crates/` — remaining hits:
  `v26.10.31/05-kgc4d-stardf.md:52,118` (prior-milestone KGC4D event frame,
  out of campaign scope, not CSCB); `lane-b03-cscb-c4.md:41` (append-only
  history, errata at :65-68 supersedes); audit doc :34/:55 (the register
  itself, quoting the stale claim). All qualified exceptions; no
  unqualified "64-byte frame" claim remains in v26.11.2 normative docs.

## Standing

ALIVE for the docs errata: every edit re-read from disk this session
(sed -n / tail witnesses above), grep sweep run, exit codes witnessed.
Path (b) (frame64) NOT implemented per dispatch: rejected by PRD-3 §5-§6
(nothing consumes bytes 24..64; 62.5% overhead; breaks 3 pinned courts).
