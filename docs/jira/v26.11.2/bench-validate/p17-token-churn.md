# p17-token-churn

Lane P-17 token-churn baseline leg, 2026-11-03. Repo `/Users/sac/cgap`,
branch main. MCP-style JSON-RPC 2.0 over stdio (newline-delimited) vs a
compact binary frame encoding of the SAME payloads, measured this
session with a purpose-built Python harness (`/tmp/p17-harness/
harness.py`; stdlib + tiktoken 0.12.0 for token figures — no bytes/4
heuristic anywhere). N=1000 exchanges (2000 envelopes), zero
round-trip failures. Deliverable: per-envelope wire sizes (bytes,
chars), per-exchange totals Min/Med/Max over N=1000 legs, and real
tokenizer counts for both encodings.

## 1. Layer boundary (fence F2 - read first)

The ns/frame figures in this campaign live on a DIFFERENT LAYER than
this report. The 10.20 ns/frame admission-kernel row (11-measured-
claims row 5) and its registered contention divergence (row 28) are
quiet-host ingress kernel admission figures. NONE of the numbers below
is an admission-kernel figure, and no kernel figure appears here. This
report's figures are wire-protocol layer: bytes-per-envelope,
chars-per-envelope, tokens-per-envelope for two encodings of the same
payloads. There are no time figures in this report at all (the harness
measures size, not latency). Any table that places a kernel ns/frame
row beside these numbers MUST label the boundary: admission kernel vs
wire protocol encoding, ns/frame vs bytes-and-tokens, different
instruments, different workloads - not like-for-like, never summed and
never ratioed.

## 2. Harness

- Location: `/tmp/p17-harness/harness.py` (scratch, not repo capital;
  214 lines; results tee'd to `/tmp/p17-harness/results.txt`).
- Command: `python3 /tmp/p17-harness/harness.py` - exit 0, this
  session, 2026-11-03T06:46Z (leg window 06:45:33Z-06:47:25Z UTC).
- Zero new repo dependencies: Python 3 stdlib (`json`, `struct`,
  `statistics`) plus `tiktoken` 0.12.0 from the ambient environment
  (not installed into the repo; no Cargo.toml / package.json / lockfile
  touched). rustc 1.101.0-nightly (12433c307 2026-10-24) present but
  not exercised - no Rust build in this leg.
- JSON leg: `json.dumps(obj, separators=(",",":"), ensure_ascii=True)`
  + `\n` delimiter (MCP stdio newline-delimited framing); delimiter
  byte counted in `json_bytes`.
- Binary leg: deterministic compact frame `MAGIC(2B) | type(1B) |
  body`, body = recursive value codec (zigzag varint ints, length-
  prefixed UTF-8 strings, tagged bool/null/float64/list/dict). Header
  is 3 bytes total; no padding, no alignment.
- Round-trip correctness (asserted final state, per envelope):
  decode(binary) == original object field-for-field; re-encode ==
  original bytes; JSON parse -> re-dump == original line. 2000/2000
  envelopes passed, `ROUNDTRIP_FAILURES 0` (harness exit 0).
- Replay: `python3 /tmp/p17-harness/harness.py` - fully deterministic
  corpus (fixed 5-method cycle, id = 1000+i), no RNG, byte-identical
  output expected on replay.

## 3. Measured legs (2026-11-03, this session)

Ambient: loadavg 2.96 3.15 2.97 at leg start (sysctl vm.loadavg,
2026-11-03T06:45:33Z) and 2.53 2.94 2.90 at leg end (06:47:25Z).
Size metrics are load-independent in the limit but the ambient is
recorded per F2 discipline; figures are not quiet-host-scoped.

## 4. Wire sizes (measured, exact bytes on the wire)

Per-envelope, over n=1000 of each kind (receipt: harness stdout,
`results.txt` LEG req / LEG resp blocks):

| Envelope kind | Metric | Min | Median | Max | Sum (n=1000) |
|---|---|---|---|---|---|
| request | JSON bytes | 79 | 126 | 189 | 127,626 |
| request | binary bytes | 67 | 106 | 161 | 108,635 |
| request | delta bytes (J-B) | - | - | - | 18,991 (14.88%) |
| request | chars (J = B here) | 79 | 126 | 189 | 127,626 |
| response | JSON bytes | 113 | 165 | 324 | 179,534 |
| response | binary bytes | 92 | 143 | 280 | 155,334 |
| response | delta bytes (J-B) | - | - | - | 24,200 (13.48%) |
| response | chars (J = B here) | 113 | 165 | 324 | 179,534 |

Chars vs bytes: identical here because the JSON leg is
`ensure_ascii=True` (pure ASCII; 1 char = 1 byte) and the binary
frame's token leg is measured on a byte-per-char latin1 rendering
(section 5). This is a property of the corpus, not a law - non-ASCII
payloads would diverge.

Per-exchange totals (request + response summed, over N=1000
exchanges):

| Exchange metric | Min | Median | Max |
|---|---|---|---|
| JSON bytes | 259 | 291 | 403 |
| Binary bytes | 216 | 249 | 347 |
| JSON tokens | 86 | 92 | 109 |
| Binary tokens | 112 | 115 | 147 |

Grand totals over 2000 envelopes: JSON 307,160 B vs binary 263,969 B
- the binary frame saves 43,191 B (14.06%) on the wire.

## 5. Token figures (MEASURED with a real tokenizer)

Tokenizer: `tiktoken` 0.12.0, encoding `o200k_base` (a real GPT-style
BPE implementation - not a bytes/4 heuristic). JSON strings are
tokenized as-is. Binary frames have no text form, so the stated
deterministic rendering is latin1 byte-per-char (`b.decode("latin1")`)
fed to the same tokenizer; the rendering is 1:1 invertible, so this
counts the actual frame bytes under the tokenizer's byte-level merges.

| Metric | JSON (2000 env) | Binary (2000 env) | Delta |
|---|---|---|---|
| Tokens, sum | 96,200 | 123,042 | binary +26,842 (+27.90%) |
| Tokens, mean per envelope | 48.10 | 61.52 | +13.42 |
| Mean per exchange | 48.10 (req+resp 2 env) | 61.52 | +13.42 |

Headline finding (MEASURED, and it cuts against the obvious story):
the compact binary frame is 14.06% SMALLER in bytes but tokenizes
27.90% LARGER under o200k_base. High-entropy binary byte sequences
break BPE merges that compress repetitive ASCII JSON keys
("jsonrpc", "method", "content"); a downstream LLM agent consuming
the binary encoding pays MORE in tokens despite the wire saving.
Zero-copy wire savings do not transfer to the token layer for this
corpus - the paper's "Zero-Copy Agency" claim must scope zero-copy to
bytes, not tokens, or add a text-affine binary encoding.

## 6. Self-audit

- Line length: `awk length>99` over this file returns 0 offending
  lines (re-checked after every appended chunk, this session).
- Every number in sections 3-5 carries a receipt: harness stdout
  (`/tmp/p17-harness/results.txt`, exit 0), figures re-read from that
  file at write time - none asserted from memory.
- Token figures are tokenizer-derived (tiktoken o200k_base), not
  estimated; the binary-frame token method (latin1 byte-per-char
  rendering) is stated above and is deterministic.
- Round-trip correctness asserted on final state (decoded object
  equality, re-encode byte equality), 2000/2000, not call-count.
- No mocks: the tokenizer is real, the codec is a real
  encoder/decoder pair exercised both directions per envelope.
- No Process.sleep: the leg is a single bounded computation, no
  waiting anywhere.

## 7. Divergences and honest limits

- Dispatch goal line contained a self-correction ("produce ... README?
  No. Produce p17-token-churn.md"); the authoritative target is this
  file. No README was written.
- Corpus is a fixed 5-method MCP-style cycle with compact payloads
  (mean 153.58 B/envelope JSON); token-churn ratios on much larger
  payloads (KB-scale tool results) are UNVERIFIED - this baseline does
  not extrapolate to them.
- Binary frame is this harness's own minimal codec, not a shipped cgap
  wire format; it bounds the class "compact length-prefixed frame of
  the same fields", nothing more.
- tiktoken o200k_base is OpenAI-family; other tokenizer families
  (Claude, Llama) will produce different absolute token counts. The
  bytes figures are tokenizer-independent; the +27.90% direction is
  expected to hold for byte-level BPE generally but is only measured
  here for o200k_base.
