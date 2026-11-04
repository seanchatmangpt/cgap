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
