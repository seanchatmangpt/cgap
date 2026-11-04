# Red-team verdict — P-18 on "Zero-Copy Agency" (zero-copy-agency.md)

- Subject: docs/jira/v26.11.2/paper/zero-copy-agency.md (+ parts/), on-disk state
  2026-11-02, /Users/sac/cgap @ main.
- Role: adversarial reviewer-2. Verdicts only; no repairs.
- Receipt (auditor, this session): every command+exit below was run by me;
  /tmp sandbox for the mutation court; shared tree untouched (read-only).

## Standing

Paper: ALIVE (310 lines, 8 parts, on disk). Audits reproduce: A1 line-length
`awk 'length > 100'` -> 22 long lines, all claims-table rows 280-305 (the
named exemption; exit 0). A3 receipt existence: 8/8 cited doc paths exist
(exit 0). A4 parts identity: 8/8 parts are byte-identical subsequences
(python byte-span check, exit 0). Tag census on disk: 45 receipt-tag
occurrences / 6 [TARGET-v26.11.2] / 6 [CITE-TBD] (grep -o | wc -l).

Mutation court F3 WITNESSED by me in /tmp/rt-wire-shm-$$ sandbox (never the
shared tree): BASELINE cargo test -> 1 passed + 7 passed, 0 failed (rc 0);
MUTATION poly 0xEDB8_8320 -> 0xEDB8_8321 -> `test crc32::tests::known_vectors
... FAILED` (panic src/crc32.rs:46). Flip confirmed; court has teeth.

## SPOILERS (2)

- S1 — bridge described as "planned, unbuilt" but shipped ALIVE on disk.
  zero-copy-agency.md:52-53 and :191-202 ([TARGET-v26.11.2]: "designed,
  unbuilt, no measuring receipt") state the cgap-mcp bridge does not exist.
  crates/cgap-mcp exists: Cargo.toml version 26.11.3, bin cgap-mcpd, 5/5
  tests, real-stdio protocol court, mutation court FLIPPED
  (crates/cgap-mcp/RECEIPT.md, "Standing: ALIVE"). The paper misdescribes
  the shipped substrate — reviewer 2 would call the paper stale on its own
  §4.3. Fix: rewrite §4.3 from "[TARGET]" to MEASURED against
  crates/cgap-mcp/RECEIPT.md, or stamp an errata note; also update §1:52.
- S2 — the 14.06% byte saving is attributed to the wrong codec. §1:43-48
  says "The zero-copy byte path (wire-shm, §4) removes the byte tax: a
  compact binary framing ... measured 14.06% smaller". The 14.06% was
  measured on the p17 harness's own minimal codec (3-byte header;
  p17:43-46), explicitly "not a shipped cgap wire format" (p17:148-150);
  wire-shm WIR0 carries a 16-B header per envelope
  (crates/wire-shm/src/lib.rs:19-23). On the p17 corpus (~153.58 B mean
  envelope, p17:145), swapping in WIR0's 16-B header adds ~13 B/envelope
  (~4.4 pp of the 291 B median) — WIR0's saving on this corpus would sit
  materially below 14.06% and was never measured. Z5 (paper:284) inherits
  the misattribution. Fix: attribute the 14.06% to the p17 codec class
  explicitly ("a compact binary framing of the p17 harness's design, not
  the WIR0 frame format"), or measure WIR0-framed sizes on the same corpus.

## NITs (5)

- N1 — §1:29 (and Z1 paper:280): "max 1639.58 µs over N=100 x 3 runs" —
  the 3-run max is 1741.12 µs (run 1, p01:38); 1639.58 is run 3 only. The
  receipt itself (p01:177) also quotes 1.64 ms, so the drift is upstream;
  paper should quote 1741.12 (or "~1.7 ms") as the 3-run max. Same for
  p95 13.62 (run-3 only, p01:40).
- N2 — §1:43-48 states the byte-saving claim with no compact-payload /
  KB-scale-UNVERIFIED caveat; the caveat exists only at §3.3:140-142 and
  Z17 (paper:296). Every payload-scope claim site needs the limit (§1 is
  the skimming surface).
- N3 — §1:20-24 measures the "serialization tax" as TIME (7.40 µs / 53.08
  µs round-trips), then §1:43 says the byte path "removes the byte tax"
  with only byte-SIZE evidence; the frame path has no latency measurement
  (p17:20-24: no time figures; §4.3 concedes no latency claim). "Removes
  the serialization tax" is not supported; "shrinks the byte weight 14.06%
  (size, not latency)" is.
- N4 — temporal: paper header "Assembled 2026-11-02" (paper:3) cites a
  receipt whose leg ran 2026-11-03T06:46Z (p17:34). Either the assembly
  date or the p17 leg date is wrong; fix the date, note in errata.
- N5 — assembly receipt addendum says "Receipt-tag count now 50"
  (lanes/lane-paper-assembly.md, coordinator addendum); disk says 45
  receipt-tag occurrences. Re-census and correct the receipt note (errata
  policy permits dated errata, not receipt rewrite).

## OBSERVATIONS (3)

- O1 — token-refutation spin check: PASSES. +27.90% token growth is
  stated three times (§1:49-50, §2:72-79, §3.3:133-142) and enforced by
  F7 (paper:234-240); the refutation is the headline, not buried. Title
  "zero-copy-agency" alone could still mislead a title-only reader;
  a subtitle ("bytes yes, tokens no") would close it (optional).
- O2 — MCP naming: the paper never claims MCP-spec conformance and
  cgap-mcp implements a minimal JSON-RPC 2.0 subset (initialize,
  tools/list, tools/call; protocolVersion 2024-11-05) —
  crates/cgap-mcp/README.md. No overclaim found. When §4.3 is rewritten
  per S1, add an explicit "JSON-RPC subset, not full MCP spec" scoping
  line.
- O3 — falsifier teeth: F1-F7 all name observables; F3's flip is real
  (witnessed above). F2/F6 ride on the §6 audit courts, which reproduce.
  A2's residual-line classification is judgment-scoped (73 residual digit
  lines classified lawful by the assembly lane) — inherently the weakest
  falsifier, but classified and receipted, not silent.

## Verdict

SPOILER-CONFIRMED count: 2 (S1, S2). Seal-blocking: YES (>= 1 SPOILER).

Standing: paper ALIVE as a document, audits reproduce, substrate court
witnessed FLIPPED — but the two SPOILERs block sealing until S1/S2 are
repaired (rewrite §4.3 + fix the 14.06% attribution) and re-audited.
