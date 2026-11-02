# Lane IPC-1: interop-chicago scaffold (v26.11.2)

Subject: `/Users/sac/cgap/courts/interop-chicago/` (standalone Go module)

## W1 Files written

- `courts/interop-chicago/go.mod` — module
  `avec.chatman.dev/cgap/courts/interop-chicago`, go 1.23, stdlib only,
  zero external deps.
- `courts/interop-chicago/corpus/gen.go` — SplitMix64 corpus generator,
  1,365-fact batches (`pack_demux::FACT_WINDOW`, named coprime stride,
  HONEST NONE, `crates/cgap-git/src/pack_demux.rs:43-51`).
- `courts/interop-chicago/README.md` — layout + variant disclosure.
- `courts/interop-chicago/cmd/seal_receipt/main.go` — NOT my content at
  lane end (see W5).

## W2 Layout choice (recorded divergence from charter)

24-byte facts (3x u64 LE), NOT the charter's 64-byte assumption. Mirrors
what the kernels consume: `scripts/generate_cscb_corpus.py` (24 B triples)
and `crates/csct/src/bin/cscb_c1_c2_ingress.rs` (`FRAME_BYTES = 24`).
Batch = 1,365 x 24 = 32,760 bytes.

## W3 Commands + exits (this session, go1.23.4 darwin/arm64)

- `go build ./...` + `go vet ./...` — exit 0 (BUILD_OK/VET_OK).
- Determinism court (`/tmp/ipc1-court/court/main.go`, sandbox): two gens
  byte-identical (sha256 `2611fc8daf521430e57c8fd21d30333baa856c94e9c0af
  a4d1b1621c7d7c7cf8`); seed^1 flip digest
  `e7dc29c1...d43d81` differs — anti-vacuity FLIPS. COURT exit=0.
- Cross-language court: `go run` dump + `cmp` vs
  `python3 scripts/generate_cscb_corpus.py` first-32,760-byte prefix —
  byte-identical (INTEROP_CMP_OK).

## W4 Seed law

Corpus seed `0xA20C_0FFE_E5A7_E000` = SplitMix64 STATE; multiplier
`0xBF58_476D_1CE4_E5B9` = internals. Never conflated.

## W5 Disclosure (both directions, compile-freeze SLA)

IPC-6 landed its real receipt emitter into my dispatch file
`cmd/seal_receipt/main.go` mid-flight (replacing my LOUD-STUB). My
interim stub compiled clean; IPC-6's landed emitter also compiles clean
(`go build ./...` exit 0 against the final state). IPC-6 is the owner of
that file's content; IPC-1 supersedes nothing there. IPC-6's receipt
must cite this scaffold lane.

## W6 Divergences named

1. Two SplitMix64 variants coexist in-tree (Python generator: multiplier
   twice + shift 43; csct ingress self-gen: 0x94D0...EB + shift 31).
   Streams differ (first u64 `3f4bf12704f198f5` vs `159fd140385b5c61`,
   witnessed). This module mirrors the PYTHON generator (canonical corpus
   authority); the ingress `--self-gen` path is itself disclosed upstream
   as an in-process divergence.
2. Transcript root in IPC-6's emitter is sha256, not BLAKE3 — Go stdlib
   has no BLAKE3 (named in the emitter itself).

## W7 Standing: ALIVE

Module builds, vets, and both courts pass this session with cited exits.
Perf numbers: none claimed. Corpus binary itself is /tmp-only (not
landed); landed artifact is the generator source.
