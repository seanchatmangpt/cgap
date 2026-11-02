# receipt-interop-chicago-v1

sealed-by: seal-receipt/0.1.0 (IPC-6)

## Environment

- host: <host>
- go: <go version>
- cgo: <CGO settings>
- thermal caveat: <thermal data disclosure; "none collected" is a valid, named entry>

## Per-CTQ measured table

| CTQ | measured | method | command | exit | receipt |
|---|---|---|---|---|---|
| <ctq-name> | <value> | <batch-amortized \| machine-dependent \| load-sensitive ...> | `<exact command>` | <exit> | <receipt path> |

## Sales bullets (self-linted)

- <bullet text — quantitative claims MUST inherit method + receipt + exit from their CTQ row> [receipt: <path>]

## Transcript root

transcript-root (sha256 over the receipt bytes above, trailing whitespace trimmed): <hex>
DIVERGENCE-NAMED: transcript root is sha256, not BLAKE3 — Go stdlib has no BLAKE3; named per dispatch, not faked.

<!--
Honesty gates (fail-closed, emit-time refusal, exit 1 with named SL lines):
  SL-001 quantitative bullet without method disclosure
  SL-002 bullet citing no receipt path
  SL-003 bullet number lacking a this-session command exit in its input
Anti-vacuity flip: feed a fixture with an uncited claim -> exit 1; fix -> exit 0.
-->
