# interop-chicago court module

Standalone Go module (own go.mod), stdlib only, zero external/web deps
(publish-frozen law). Scaffolds the cgap interop courts.

## Layout

- `go.mod` — module `avec.chatman.dev/cgap/courts/interop-chicago`, go 1.23.
- `corpus/gen.go` — deterministic corpus generator. SplitMix64 stepped from
  the frozen corpus seed `0xA20C_0FFE_E5A7_E000` (multiplier
  `0xBF58_476D_1CE4_E5B9` is the PRNG internals, never a state).
  1,365-fact batches (`pack_demux::FACT_WINDOW`, a named coprime stride,
  HONEST NONE provenance, `crates/cgap-git/src/pack_demux.rs:43-51`).
- `cmd/seal_receipt/main.go` — LOUD-STUB, real emitter owned by IPC-6.

## Fact layout (recorded choice)

24-byte records, three little-endian u64 coordinates (x, y, z) per fact —
NOT the charter's 64-byte assumption. This mirrors the format the cgap
kernels actually consume: `scripts/generate_cscb_corpus.py` (24 B triples)
and `crates/csct/src/bin/cscb_c1_c2_ingress.rs` (`FRAME_BYTES: usize = 24`).

## SplitMix64 variant (measured, IPC-1)

Two SplitMix64 variants coexist in-tree: the canonical Python corpus
generator (multiplier twice, final shift 43) vs the csct ingress self-gen
(textbook second multiplier `0x94D0_49BB_1331_11EB`, shift 31). They yield
DIFFERENT streams from the same seed (first u64 `3f4bf12704f198f5` vs
`159fd140385b5c61`, witnessed this session). This module mirrors the Python
generator. The two seeds never conflate: corpus seed `0xA20C...E000` is the
STATE; multiplier `0xBF58...E5B9` is PRNG internals.

## Determinism court

```
cd /Users/sac/cgap/courts/interop-chicago
go build ./... && go vet ./...
go run ./corpus/court  (see court below)  # or: go test ./corpus
```

The court (see lane receipt
`docs/jira/v26.11.2/lanes/lane-ipc-01-scaffold.md` for exits) asserts:

1. Two generations from the frozen seed are byte-identical.
2. A single-bit seed tweak flips the digest (anti-vacuity). A court that
   cannot flip is FAIL.
