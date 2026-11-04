# zero-copy-agency

Assembled 2026-11-03 by lane C (paper assembly), /Users/sac/cgap @ main
(date errata, repair lane: originally stamped 2026-11-02, but the p17 leg
receipt cited below ran 2026-11-03T06:46Z; receipt:
../bench-validate/p17-token-churn.md:34).
Subject: the MCP wire tier. Companion papers: `position-paper-v3.md` (runtime/
kernel tier — this paper does NOT duplicate or restate its numbers) and
`claims-appendix.md` (its claims-mapping template, followed here in §7).
Numeric sources of record: `../bench-validate/p01-stdio-baseline.md`,
`../bench-validate/p02-http-baseline.md`,
`../bench-validate/p17-token-churn.md`, and
`../lanes/lane-wire-shm.md` (substrate receipt). Method of record per file.

Tagging law (non-negotiable, identical to `position-paper-v3.md:10-15`):
`(receipt: <file>:<line>)` for a number carried by a receipt on disk;
`[TARGET-v26.11.2]` for an aimed-for number with no measuring receipt;
`[CITE-TBD: version+date]` for a public fact cited from outside the repo.
Nothing else is lawful. UNVERIFIED never rounds up; perf numbers are
TARGETs until measured with a stated method.
## 1. Thesis

Agents pay a serialization tax on the MCP wire: every request and response
crosses the agent boundary as parsed-and-re-serialized JSON text, even when
both endpoints hold the same bytes. This paper measures that tax on the two
transports agents actually use and states what a zero-copy byte path removes
— and, honestly, what it does not.

Measured posture of the tax:

- stdio (newline-delimited JSON-RPC 2.0, the MCP stdio shape): full
  round-trip median 7.40 µs, min 7.17 µs, p95 13.62 µs (run 3; per-run
  11.96 / 12.79 / 13.62), max 1741.12 µs over N=100 x 3 runs — the 3-run
  max is run 1, not the 1639.58 seen in runs 2-3 (receipt:
  ../bench-validate/p01-stdio-baseline.md:37-40).
  Envelope ~330 B request + ~323 B response (receipt:
  ../bench-validate/p01-stdio-baseline.md:28). The prior "ms-class" paper
  label was pessimistic by ~2 orders of magnitude at the median; ms-class
  is observed only in the scheduling tail (receipt:
  ../bench-validate/p01-stdio-baseline.md:161-167).
- localhost HTTP/1.1: keep-alive round-trip median 53.084 µs (min 41.166,
  max 72.083); fresh-connect median 56.166 µs (receipt:
  ../bench-validate/p02-http-baseline.md:44-47). Full exchange wire weight
  338 B (159 B request + 179 B response, headers and body counted
  separately and together; receipt:
  ../bench-validate/p02-http-baseline.md:58-65).

The zero-copy byte path (wire-shm, §4) removes the byte-SIZE tax — no
latency removal is measured or claimed (see below). A compact binary
framing CLASS — the p17 harness's own minimal codec, 3-B header,
explicitly NOT a shipped cgap wire format (receipt:
../bench-validate/p17-token-churn.md:43-46,148-150) — measured 14.06%
smaller on the wire than JSON over 2,000 envelopes (receipt:
../bench-validate/p17-token-churn.md:94-95). Scope caveat: compact
payloads only (mean ~153.58 B/envelope); KB-scale tool results are
UNVERIFIED (receipt: ../bench-validate/p17-token-churn.md:144-150). The
shipped WIR0 frame format carries a 16-B header per envelope (receipt:
../../../../crates/wire-shm/src/lib.rs:19-23), which on the same corpus
would reduce the saving; no WIR0-framed size measurement exists on this
corpus, so any WIR0-specific saving figure is [TARGET-v26.11.2]. Frame-
path LATENCY is likewise unmeasured — p17 recorded sizes only, no time
figures (receipt: ../bench-validate/p17-token-churn.md:20-24) — so
latency removal is [TARGET-v26.11.2], never claimed. Typed integrity
(crc32, ordinals) is enforced by a court-flipped substrate (8/8 tests,
mutation court FLIPPED; receipt:
../lanes/lane-wire-shm.md:19-20,22-27,45-48). The token layer is a
different layer and the byte win does NOT transfer to it — §2 and §3
state the measured refutation.

The cgap-mcp bridge putting wire-shm frames on a live MCP server is
BUILT and ALIVE: crates/cgap-mcp v26.11.3, bin cgap-mcpd, 5/5 tests
green — round-trip field equality, real-stdio protocol shape, zero-alloc
encode window, mutation court FLIPPED, persistence (receipt:
../../../../crates/cgap-mcp/RECEIPT.md:12-16,20-42). Honest scope: a
MINIMAL JSON-RPC MCP-shape subset (initialize, tools/list, tools/call),
not full MCP spec conformance (receipt:
../../../../crates/cgap-mcp/RECEIPT.md:24-29).
## 2. Layer discipline (fence F1 and the token fence)

Two fences bind every number in this paper. Violating either falsifies it
(§5).

F1 — admission kernel vs wire protocol. The 10.20 ns/frame fastest /
10.30 ns/frame median admission-kernel figures (quiet-host ingress, divan;
registered contention divergence 124.06 ns/frame slowest) live on a
DIFFERENT layer than every µs/round-trip and B/envelope figure here
(receipt: ../paper/position-paper-v3.md:207; underlying receipts
../waves/w3-l1-ingress.md:63,69-70 and ../11-measured-claims.md rows 5/28).
ns/frame and µs/round-trip are different instruments on different
workloads: never summed, never ratioed, never placed in one table without
the boundary label (receipt:
../bench-validate/p02-http-baseline.md:11-22). The ~700x gap between
10.20 ns/frame and the 7.40 µs stdio median is a layer gap, not a win
(receipt: ../bench-validate/p01-stdio-baseline.md:168-170).

Token fence — bytes are not tokens. Byte savings and token savings are
different layers of the same wire. The p17 leg measured BOTH with a real
tokenizer (tiktoken 0.12.0, o200k_base — not a bytes/4 heuristic) and
found the binary frame 14.06% smaller in bytes yet 27.90% LARGER in tokens
(receipt: ../bench-validate/p17-token-churn.md:106-110,112-120). This
paper therefore scopes "zero-copy" to BYTES, never to tokens. Any claim,
in this file or derived from it, that binary frames save agent tokens is
a falsifier of this paper (§5, F7).
## 3. Measured results (every number receipt-tagged)

All legs ran in-session against purpose-built harnesses; receipts carry
command transcripts and exits. Ambient load recorded per leg; figures are
ambient-scoped, not quiet-host.

### 3.1 stdio baseline (p01)

Real child-process MCP-style JSON-RPC 2.0 echo server + client over
newline-delimited stdio, Rust + serde_json, zero cgap deps, opt-level 3
(receipt: ../bench-validate/p01-stdio-baseline.md:17-28). N=100
serialized envelopes x 3 runs, exit 0 x3. Per-run medians 7.38 / 7.46 /
7.40 µs; means 25.15 / 24.10 / 24.60 µs; max 1741.12 / 1639.58 / 1639.58 µs
(receipt: ../bench-validate/p01-stdio-baseline.md:37-40). 99/100 envelopes
per run land under 50 µs; the single 1.6-1.7 ms outlier per run is
first-run child-wakeup scheduling, not protocol cost (receipt:
../bench-validate/p01-stdio-baseline.md:42-49). Verdict: MEASURED —
typical-case is µs-class; the ms-class label survives only at the tail
(receipt: ../bench-validate/p01-stdio-baseline.md:176-181). F4 fence: the
shared-memory channel was NOT exercised by p01 (receipt:
../bench-validate/p01-stdio-baseline.md:181-182).

### 3.2 HTTP baseline (p02)

Zero-dependency std::net TCP harness; POST of a 71 B JSON-RPC echo body;
TCP_NODELAY; 20 unmeasured warmups per leg (receipt:
../bench-validate/p02-http-baseline.md:26-37). Hot ambient (loadavg 5.42
3.70 3.03), stated per fence discipline (receipt:
../bench-validate/p02-http-baseline.md:40-42). Keep-alive: min 41.166 /
median 53.084 / max 72.083 µs, N=100, exit 0. Fresh-connect per request:
min 47.000 / median 56.166 / max 72.083 µs, exit 0 (receipt:
../bench-validate/p02-http-baseline.md:44-47). Connection reuse saves
~11% at the median; max identical in both legs (receipt:
../bench-validate/p02-http-baseline.md:49-53). Wire weight per envelope:
88 B request header block + 71 B body = 159 B; 72 B response header block
+ 107 B body = 179 B; full exchange 338 B (receipt:
../bench-validate/p02-http-baseline.md:58-65). A real MCP server sends
more headers; this is a minimal-server floor (receipt:
../bench-validate/p02-http-baseline.md:67-69).

### 3.3 Token churn (p17) — a REFUTATION, stated as a finding

Python harness, N=1000 exchanges (2,000 envelopes), zero round-trip
failures, exit 0 (receipt: ../bench-validate/p17-token-churn.md:33-34,47-51).
Bytes: binary frame saves 43,191 B of 307,160 B — 14.06% — on the wire
(receipt: ../bench-validate/p17-token-churn.md:94-95). Per-exchange
medians: JSON 291 B vs binary 249 B (receipt:
../bench-validate/p17-token-churn.md:87-90). Tokens (tiktoken 0.12.0,
o200k_base, latin1 byte-per-char rendering of frames, deterministic and
invertible): JSON 96,200 vs binary 123,042 tokens over 2,000 envelopes —
binary +26,842 (+27.90%) (receipt:
../bench-validate/p17-token-churn.md:106-110).

HEADLINE FINDING (MEASURED, cuts against the obvious story): the compact
binary frame is 14.06% smaller in bytes but tokenizes 27.90% larger.
High-entropy byte sequences break BPE merges that compress repetitive
ASCII JSON keys; a downstream LLM agent consuming the binary encoding pays
MORE in tokens despite the wire saving (receipt:
../bench-validate/p17-token-churn.md:112-120). This is stated as a
REFUTATION of token-layer parity, not a hidden failure; scope of the
refutation: fixed 5-method corpus, compact payloads, o200k_base only —
other tokenizer families and KB-scale tool results are UNVERIFIED
(receipt: ../bench-validate/p17-token-churn.md:144-155).

### 3.4 wire-shm courts

Substrate receipt: `cargo test` exit 0, 8/8 tests passed (1 lib + 7
integration), zero warnings; `cargo tree` shows only the crate itself —
zero dependencies (receipt: ../lanes/lane-wire-shm.md:18-21). Mutation
court FLIPPED: CRC poly 0xEDB8_8320 -> 0xEDB8_8321 in a /tmp sandbox flips
the known-vector court to FAILED; shared-tree control stayed 8 passed,
0 failed (receipt: ../lanes/lane-wire-shm.md:22-27). In-test anti-vacuity:
tamper_court_flips asserts CONTROL passes and MUTATED segment fails with
typed StaleOrdinal; no [SKIP], no Process.sleep, no mocks — real file
segments throughout (receipt: ../lanes/lane-wire-shm.md:38-43; court
source crates/wire-shm/tests/wire_courts.rs:104-149).
## 4. Design: the wire-shm zero-copy byte path

### 4.1 Frame format (implemented, ALIVE)

Big-endian, fixed 16-B header: `[u32 magic 0x5749_5230 ("WIR0")] [u32
crc32(payload)] [u32 payload_len] [u32 frame_ordinal]` followed by payload
(receipt: crates/wire-shm/src/lib.rs:3-4,19-20). CRC is table-driven
CRC-32/IEEE, reflected, poly 0xEDB8_8320, init/xorout 0xFFFF_FFFF
(receipt: ../lanes/lane-wire-shm.md:10-12; known vector
crc32_ieee(b"123456789") == 0xCBF4_3926 asserted at
crates/wire-shm/tests/wire_courts.rs:21-24). Slots are 4096-B stride:
16-B header + up to 4080 payload bytes (receipt:
crates/wire-shm/src/lib.rs:21-23). Typed errors, no stringly-typed
failure: FrameTooLarge, CrcMismatch, StaleOrdinal, BadMagic, Io
(receipt: crates/wire-shm/src/lib.rs:26-32; witnessed instances
FrameTooLarge {len 5000, max 4080} and typed StaleOrdinal at
../lanes/lane-wire-shm.md:33-36).

### 4.2 Segment layout — offset-addressed file segment, NOT mmap

Honest scoping: the segment is an offset-addressed file-backed segment
(std::fs + FileExt read_all_at/write_all_at), NOT mmap — mmap requires
libc and the zero-dep law forbids it (receipt:
../lanes/lane-wire-shm.md:13-15; implementation
crates/wire-shm/src/ring.rs:7-10). Layout: 4096-B bookkeeping region
(u64 BE next_ordinal, u32 BE capacity) + fixed 4096-B slots
(receipt: crates/wire-shm/src/ring.rs:1-5). Ordinals are strictly
increasing per writer; next_ordinal is persisted AFTER the frame so
recovery never sees a torn frame (receipt:
crates/wire-shm/src/ring.rs:113-115; invariant proven at
../lanes/lane-wire-shm.md:30-36). Because the path is positioned
read/write into one file segment rather than zero-copy pointer sharing,
"zero-copy" here means zero-serialization on the byte path (binary frames
never re-encode payloads), not zero-copy in the mmap page-sharing sense.

### 4.3 cgap-mcp bridge (landed, ALIVE)

The bridge is BUILT: crates/cgap-mcp (v26.11.3, bin cgap-mcpd), mapping
wire-shm frames onto the MCP JSON-RPC surface with field equality
preserved — JSON -> wire frame -> JSON round-trip asserted
field-for-field through handle_line on a real WireRing file, with
on-disk slot byte-cross-checks against the wire-shm frame format
(receipt: ../../../../crates/cgap-mcp/RECEIPT.md:20-23). Courts, all
green (cargo test exit 0, 5 passed / 0 failed): (1) round-trip field
equality across 4 message cases; (2) protocol shape over REAL stdio via
a spawned cgap-mcpd (initialize protocolVersion 2024-11-05, tools/list,
tools/call error codes -32602/-32601/-32700); (3) zero-alloc encode
window with an allocating anti-vacuity control; (4) mutation court
FLIPPED in a /tmp sandbox (skipped ordinal -> BadMagic via -32603),
sha256-exact restore verified; (5) persistence across reopen (receipt:
../../../../crates/cgap-mcp/RECEIPT.md:12-16,20-42). Honest scope: a
MINIMAL JSON-RPC MCP-shape subset (initialize, tools/list, tools/call),
NOT full MCP spec conformance (receipt:
../../../../crates/cgap-mcp/RECEIPT.md:24-29). No latency claim is made
for the frame path itself — p17 measured sizes only, no time figures
(receipt: ../bench-validate/p17-token-churn.md:20-24); bridge wire
round-trip latency remains [TARGET-v26.11.2]. The MCP JSON-RPC shape the
bridge preserves is the p01 envelope: {"jsonrpc","id","method",
"params"} newline-delimited on stdio (receipt:
../bench-validate/p01-stdio-baseline.md:23-28).
## 5. Falsifiers (F1-F7)

Position-paper style: each falsifier names the observation that kills the
claim. A falsifier with no observable is prose.

- F1 (layer fence): falsified the moment any table or prose in this file
  sums, ratios, or unlabeled-adjacently places the 10.20/10.30 ns/frame
  admission-kernel figures beside this paper's µs/round-trip or
  B/envelope figures (receipt:
  ../bench-validate/p02-http-baseline.md:11-22; fence source
  position-paper-v3.md §5.1).
- F2 (measured-never-asserted): falsified by any figure in this paper
  without a `(receipt: ...)`, `[TARGET-v26.11.2]`, or `[CITE-TBD: ...]`
  tag on its line (§6 audit command is the check; the v3 tagging law at
  position-paper-v3.md:10-15 is the law). Zero untagged quantitative
  claims is the discipline claim.
- F3 (substrate integrity): falsified if any wire-shm court stops
  flipping — a court that cannot flip is FAIL. Currently the CRC-poly
  mutation flips the known-vector court and the in-test tamper court
  asserts both branches (receipt: ../lanes/lane-wire-shm.md:22-27,38-43).
- F4 (honest scoping): falsified if the offset-addressed file segment is
  ever restated as mmap or page-shared zero-copy, or if p01/p02/p17
  ambient figures are restated as quiet-host figures (receipt:
  ../bench-validate/p01-stdio-baseline.md:22-23; receipt:
  ../bench-validate/p02-http-baseline.md:40-42).
- F5 (citation apparatus): falsified if any [CITE-TBD] pin ships
  unpinned or an invented version/date appears; zero invented sources is
  the claim (v3 §6 F5 posture, receipt: position-paper-v3.md:260-262).
- F6 (whole-paper tagging): the audit in §6 returning any unlawfully
  untagged digit-bearing line falsifies this paper. Command + exit
  recorded in §6.
- F7 (token-layer fence): falsified by any claim — in this file or any
  derivative — that binary frames save agent tokens. The MEASURED
  refutation is +27.90% token growth under o200k_base on the p17 corpus
  (receipt: ../bench-validate/p17-token-churn.md:106-110,112-120); the
  finding is the contribution, hiding it would be the falsification.
  Close condition: a text-affine binary encoding measured token-neutral
  or better on the same corpus with the same tokenizer would retire F7.
## 6. Self-audit

Executable checks run this session against the on-disk file (commands and
exits recorded here; rerun them, they are the court):

- A1 line length: `awk 'length > 100 {c++} END{print c+0}'` over this
  file. Convention exemption named in advance, per the v2 finalize
  precedent (receipt: ../../v26.11.1/waves/paper-finalize.md:34-36,
  divergence D2): markdown table rows are exempt; every other line must
  be <= 100 chars. Observed count and line classification in the receipt
  `../lanes/lane-paper-assembly.md`.
- A2 untagged-numeral sweep: paragraph-scoped pass over every
  digit-bearing line; each residual line must be (a) tag-bearing,
  (b) an identifier or path (file paths, frame-layout tokens like
  0x5749_5230, section names), or (c) a structural heading. Zero
  unlawfully untagged quantitative claims is the pass condition.
- A3 receipt existence: every `../...md:<line>` cited in this file must
  resolve to an on-disk file at audit time; a missing receipt file is a
  FAIL, never a silent pass (method: `test -f` per cited path).
- A4 parts identity: each file under `parts/` is a byte-identical
  subsequence of this file (assembly is mechanical concatenation; the
  check is `cmp` of each part against the corresponding byte span).
- Tag census at assembly time: receipt-tag count, [TARGET-v26.11.2]
  count, and [CITE-TBD] count recorded in the receipt
  `../lanes/lane-paper-assembly.md`; a re-read at use time supersedes
  any recalled count.

ERRATA policy: fix-forward on this file only; a defect in a landed
receipt number is corrected by a dated errata note citing the original
file+line, never by rewriting the receipt. Landed receipt numbers are
immutable inputs.

## 7. Claims mapping (appendix)

Maps every quantitative claim in this paper to a receipt path:line or an
explicit placeholder class (template: `claims-appendix.md` §A).

| # | Claim (section) | Value | Receipt path:line / placeholder |
|---|---|---|---|
| Z1 (s1) | stdio round-trip | median 7.40 µs, min 7.17, p95 13.62 (run 3), max 1741.12 µs, N=100 x 3 | ../bench-validate/p01-stdio-baseline.md:37-40,:176-181 |
| Z2 (s1) | stdio envelope | ~330 B req + ~323 B resp | ../bench-validate/p01-stdio-baseline.md:28 |
| Z3 (s1) | HTTP round-trip | keep-alive 53.084 µs median / fresh 56.166 µs median | ../bench-validate/p02-http-baseline.md:44-47 |
| Z4 (s1) | HTTP wire weight | 338 B full exchange (159 req + 179 resp) | ../bench-validate/p02-http-baseline.md:58-65 |
| Z5 (s1) | byte saving | harness-local compact codec (3-B header, NOT WIR0): 14.06% smaller, 43,191 of 307,160 B; WIR0-framed saving unmeasured [TARGET-v26.11.2] | ../bench-validate/p17-token-churn.md:43-46,94-95,148-150; crates/wire-shm/src/lib.rs:19-23 |
| Z6 (s1) | substrate courts | 8/8 tests, mutation court FLIPPED | ../lanes/lane-wire-shm.md:19-20,22-27,45-48 |
| Z7 (s1) | mcp bridge | BUILT, ALIVE: v26.11.3, bin cgap-mcpd, 5/5 tests, mutation FLIPPED; minimal JSON-RPC subset, not full MCP | ../../../../crates/cgap-mcp/RECEIPT.md:12-16,20-42,24-29 |
| Z8 (s2) | kernel ns/frame fence | 10.20 fastest / 10.30 median / 124.06 contended | position-paper-v3.md:207; ../waves/w3-l1-ingress.md:63,69-70; ../11-measured-claims.md rows 5/28 |
| Z9 (s2) | token fence | binary +27.90% tokens under o200k_base | ../bench-validate/p17-token-churn.md:106-110,112-120 |
| Z10 (s3.1) | stdio per-run stats | medians 7.38/7.46/7.40; max 1741.12/1639.58/1639.58 µs | ../bench-validate/p01-stdio-baseline.md:37-40 |
| Z11 (s3.1) | stdio binning | 99/100 envelopes < 50 µs per run | ../bench-validate/p01-stdio-baseline.md:42-49 |
| Z12 (s3.2) | HTTP legs | 41.166/53.084/72.083 and 47.000/56.166/72.083 µs, exit 0 x2 | ../bench-validate/p02-http-baseline.md:44-47 |
| Z13 (s3.2) | header blocks | 88+71=159 req; 72+107=179 resp | ../bench-validate/p02-http-baseline.md:58-65 |
| Z14 (s3.3) | p17 leg | N=1000 exchanges, 2000 envelopes, 0 failures, exit 0 | ../bench-validate/p17-token-churn.md:33-34,47-51 |
| Z15 (s3.3) | exchange medians | JSON 291 B vs binary 249 B | ../bench-validate/p17-token-churn.md:87-90 |
| Z16 (s3.3) | token totals | 96,200 vs 123,042 (+26,842) | ../bench-validate/p17-token-churn.md:106-110 |
| Z17 (s3.3) | refutation scope | corpus+tokenizer limits | ../bench-validate/p17-token-churn.md:144-155 |
| Z18 (s3.4) | zero deps | cargo tree: crate only | ../lanes/lane-wire-shm.md:21 |
| Z19 (s4.1) | frame consts | MAGIC 0x5749_5230, 16-B header, slot 4096, max payload 4080 | crates/wire-shm/src/lib.rs:3-4,19-23 |
| Z20 (s4.1) | CRC params | poly 0xEDB8_8320, known vector 0xCBF4_3926 | ../lanes/lane-wire-shm.md:10-12; crates/wire-shm/tests/wire_courts.rs:21-24 |
| Z21 (s4.1) | typed errors | FrameTooLarge/CrcMismatch/StaleOrdinal/BadMagic/Io | crates/wire-shm/src/lib.rs:26-32; ../lanes/lane-wire-shm.md:33-36 |
| Z22 (s4.2) | segment layout | 4096-B bookkeeping + 4096-B slots, FileExt not mmap | crates/wire-shm/src/ring.rs:1-5,7-10; ../lanes/lane-wire-shm.md:13-15 |
| Z23 (s4.2) | ordinal law | next_ordinal persisted after frame | crates/wire-shm/src/ring.rs:113-115; ../lanes/lane-wire-shm.md:30-36 |
| Z24 (s5) | v3 tagging law | law text | position-paper-v3.md:10-15 |
| Z25 (s6) | line-length exemption | table rows exempt-by-convention | ../../v26.11.1/waves/paper-finalize.md:34-36 |
| Z26 (s2/s4) | MCP JSON-RPC shape | p01 envelope fields | ../bench-validate/p01-stdio-baseline.md:23-28 |

Citation placeholders: none used in this paper (no external public facts
cited); the [CITE-TBD] class is defined by the tagging law and reserved.
Any future external citation must land as [CITE-TBD: version+date] first
(falsifier F5).
