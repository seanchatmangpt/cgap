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
