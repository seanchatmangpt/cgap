# Lane paper-assembly — "Zero-Copy Agency" paper (lane C)

Subject: `docs/jira/v26.11.2/paper/zero-copy-agency.md` (310 lines,
18,766 bytes) + `docs/jira/v26.11.2/paper/parts/` (8 section files,
byte-identical contiguous assembly). 2026-11-02, /Users/sac/cgap @ main.

## W1 Subject and files written

- paper/zero-copy-agency.md (assembled from parts, never hand-edited
  after assembly)
- paper/parts/s0-header.md, s1-thesis.md, s2-layers.md, s3-results.md,
  s4-design.md, s5-falsifiers.md, s6-self-audit.md, s7-claims.md
- lanes/lane-paper-assembly.md (this receipt)

## W2 Sources read (all on disk, receipted elsewhere)

p01/p02/p17 bench-validate baselines; position-paper-v3.md (tagging law
+ tone template, content NOT duplicated); claims-appendix.md (template);
crates/wire-shm src + tests; lanes/lane-wire-shm.md.

## W3 Commands + exits (this session, cwd paper/)

```
python3 assemble (parts -> zero-copy-agency.md)      exit 0
wc -l zero-copy-agency.md                            -> 310    exit 0
awk 'length > 100' census                            -> 22 long lines, exit 0
  (all 22 are claims-table rows 280-305, the named table-row exemption;
   parts/ files carry the same rows: 44 long lines total across parts)
tag census: receipt 45 / TARGET 6 / CITE-TBD 6       exit 0
untagged-numeral sweep: 73 residual digit lines,
  all classified lawful per v3 divergence D4 classes
  (tag-continuation in a tagged clause, identifier 0x…/HTTP/1.1,
  structural heading, section cross-ref); 0 unlawfully untagged
                                                     exit 0
A3 receipt existence: 10 cited doc paths, 2 flags both classified:
  `../...md` is the s6 template placeholder (not a citation) and
  ../lanes/lane-paper-assembly.md was unwritten at check time
  (written later in this session — now resolves)      exit 0
A3 line-number spot checks (6 files, 16 lines): 0 failures, exit 0
A4 parts identity: contiguous byte-identical assembly, cover
  18766 of 18766 bytes                               exit 0
```

## W4 Verification ladder

parts-assembly identity (A4) -> line-length fence (A1) -> untagged-
numeral classification (A2) -> receipt existence + line spots (A3) ->
tag census. Prose lane: no executable mutation court exists for the
paper itself; anti-vacuity is carried by falsifiers F1-F7 (§5) plus the
wire-shm substrate courts cited from the substrate receipt (8/8 tests,
CRC-poly mutation FLIPPED — receipt: ../lanes/lane-wire-shm.md:22-27).

## W5 Fences stated and run

- position-paper-v3.md is read-only for this lane — untouched
  (divergence class D-lane-lease).
- p01/p02/p17 baselines untouched (explicit dispatch prohibition).
- No git writes; no crate edits; /tmp only for scratch (none created —
  assembly ran in-tree on owned files only).
- Zero-string / boundary gates not in this diff's surface (no lib.rs,
  no crate touched) — not run, stated as out-of-surface, not skipped
  silently.

## W6 Standing

- Paper: ALIVE — assembled, audited, on disk, all four audit courts run
  this session with exits above.
- Substrate (wire-shm) standing inherited from its own receipt (8/8,
  FLIPPED); not re-run by this lane.
- [TARGET-v26.11.2] count 6: cgap-mcp bridge (unbuilt), frame-path
  latency figures (none exist), CITE-TBD class definition x6 lines
  carrying the tag. Receipt-cited counts re-read from disk at use time.

## W7 Divergences

- D1: dispatch said "parts/ (create)" with one file per section, each a
  byte-identical subsequence — done via mechanical concatenation, A4
  proves contiguity (stronger than subsequence).
- D2: 22 long lines in the main file, all claims-table rows; the A1
  exemption was named in §6 before the audit ran.
- D3: the 6 [CITE-TBD] occurrences are the class definition + falsifier
  prose, not unpinned external citations; the paper cites no external
  public facts, so the placeholder register is empty by construction.

## Coordinator audit addendum (post-lane, 2026-11-02)

Court-auditor PASS on all four audits + content checks. Defect D1 found
and FIXED FORWARD by coordinator: zero-copy-agency.md:69 said "~7,000x"
layer gap; arithmetic (7,400 ns / 10.20 ns) and the cited receipt
(p01-stdio-baseline.md:170, "~700x") say ~700x. Fixed in main file AND
parts/s2-layers.md; parts-identity court re-passed (cmp exit 0); residual
'7,000x' grep exit 1 (0 matches). Receipt-tag census re-derived 2026-11-03
(grep -o '(receipt:' | wc -l): 45 tags (red-team N5 confirms my earlier
"50" was a line-count, not a tag-count — corrected here, not rewritten
away; the 50 figure was wrong). FRESH ERRATA 2026-11-03: post-repair
census is 54 receipt-tags (repair lane added cites; re-verified on disk
grep -o). 45 was the pre-repair figure — cite 54 for the current file.
