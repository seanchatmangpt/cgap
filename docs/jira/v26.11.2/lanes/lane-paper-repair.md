# Lane receipt: paper repair (P-18 red team S1+S2, N1-N4)

- Subject: docs/jira/v26.11.2/paper/zero-copy-agency.md + parts/ (fix-forward)
- Date: 2026-11-03 (system date on disk context; paper re-stamped 2026-11-03)
- Standing: ALIVE (all commands+exits run this session, un-piped)
- Files written: zero-copy-agency.md, parts/s0..s7 (re-sliced byte-identical),
  this receipt. Nothing else touched; no git writes.

## Per-finding fixes

- S1 (§1 + §4.3 + Z7): "planned, unbuilt" passages replaced with LANDED,
  ALIVE: crates/cgap-mcp v26.11.3, bin cgap-mcpd, 5/5 tests exit 0,
  round-trip / real-stdio protocol / zero-alloc window / mutation FLIPPED /
  persistence courts, cited (receipt:
  ../../../../crates/cgap-mcp/RECEIPT.md:12-16,20-42) and subset scope
  (RECEIPT.md:24-29: minimal JSON-RPC MCP-shape subset, NOT full MCP
  conformance). Z7 row flipped to BUILT/ALIVE with receipt cell.
- S2 (§1 + Z5): 14.06% re-attributed to its true subject — the p17
  harness's own minimal codec (3-B header; receipt:
  ../bench-validate/p17-token-churn.md:43-46), explicitly NOT a shipped
  cgap wire format (p17:148-150); WIR0's 16-B header cited
  (crates/wire-shm/src/lib.rs:19-23) with WIR0-specific saving declared
  unmeasured [TARGET-v26.11.2]. Z5 row carries the same attribution.
- N1 (§1 + Z1): max corrected 1639.58 -> 1741.12 µs (3-run max = run 1;
  receipt: ../bench-validate/p01-stdio-baseline.md:38,40); p95 13.62
  scoped as run-3-only with per-run 11.96/12.79/13.62 (p01:38-40).
- N2: compact-payload / KB-scale-UNVERIFIED caveat added at the §1
  byte-saving site (receipt: ../bench-validate/p17-token-churn.md:144-150).
- N3 (§1): "removes the byte tax" -> "removes the byte-SIZE tax"; frame-
  path latency declared unmeasured (p17:20-24, no time figures); latency
  removal marked [TARGET-v26.11.2], never claimed. §4.3 keeps bridge
  wire round-trip latency as [TARGET-v26.11.2] too.
- N4: header date corrected 2026-11-02 -> 2026-11-03 with dated errata
  note citing p17:34 (2026-11-03T06:46Z leg).
- N5: NOT repaired here — coordinator addendum errata per dispatch.

## Post-repair audits (commands + exits, this session)

1. Line length: `awk 'length>100 && $0 !~ /^\|/'` -> 0 non-table long
   lines (exit 0; 23 long lines total, all claims-table rows — named
   exemption per the v2 finalize precedent, Z25).
2. A2 residual sweep (python classifier, exit 0): 43 residual digit
   lines, all pre-existing lawful classes (tag-on-continuation-line,
   headings, paths/identifiers/version strings); no NEW unlawfully
   untagged quantitative claim introduced by this diff.
3. Receipt existence+content: 61 cited path:line spans checked (python,
   exit 0): all resolve; `crates/...` cites resolve repo-root-relative
   (existing convention); content spot-checks confirm each NEW cite's
   line content matches its claim (RECEIPT.md:12/13/16/20/24/26/34/38,
   p17:20/34/43/45/94-95/144/148-150, wire-shm lib.rs:19/20/23,
   p01:38/40).
4. Parts identity: `cat parts/s*.md | cmp - zero-copy-agency.md` -> exit 0
   (byte-identical; parts re-sliced mechanically from the repaired file).
5. Regressions: `grep -n 'planned, unbuilt\|7,000x'` over paper+parts ->
   0 matches (rc 1 = pass); `grep 'unbuilt'` -> 0 matches anywhere.

## Tag census (re-read at use time, supersedes any recalled count)

- `(receipt:` occurrences: 54; `[TARGET-v26.11.2]`: 7;
  `[CITE-TBD`: 6 (reserved, no external citations used).

## Fences (could this diff break existing gates?)

- A1-A4 audits re-run above, all pass on the repaired file.
- Tag-count drift is the audited surface: receipt-tag count moved 45 ->
  54 (+9: new cites in §1, §4.3, Z5, Z7). The assembly receipt's "50"
  addendum (N5) is now further stale; coordinator errata should cite the
  NEW census from this receipt (54), not re-derive.
- Shared tree: zero edits outside paper/ + parts/ + this receipt; no
  cargo runs, no build roots created.

## Divergences

- Dispatch said §4.3 at ~lines 191-202 and §1 at ~52-53; post-repair the
  file grew (351 lines), line numbers shifted — content-level targets
  matched exactly.
- Dispatch regression item said "RECIPT" (typo for RECEIPT); treated as
  RECEIPT.
