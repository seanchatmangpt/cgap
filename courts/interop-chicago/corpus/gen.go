// Package corpus generates the deterministic interop corpus.
//
// Frozen seeds (never conflate, cgap CLAUDE.md):
//   - Corpus seed 0xA20C_0FFE_E5A7_E000 — SplitMix64 STATE (stepped by the
//     golden gamma).
//   - Multiplier 0xBF58_476D_1CE4_E5B9 — SplitMix64 INTERNALS, never a state.
//
// Fact layout (recorded divergence from the charter's 64-byte assumption):
// 24-byte records, three little-endian u64 coordinates (x, y, z) per fact —
// the exact format consumed by crates/csct ingress (cscb_c1_c2_ingress.rs,
// FRAME_BYTES=24) and minted by scripts/generate_cscb_corpus.py. 1,365 facts
// per batch = pack_demux::FACT_WINDOW (a named coprime stride,
// 1,365 = 3 x 5 x 7 x 13; HONEST NONE provenance, pack_demux.rs:43-51).
package corpus

import "encoding/binary"

// Seeds and layout constants (frozen; do not vary).
const (
	Seed        uint64 = 0xA20C0FFEE5A7E000 // SplitMix64 initial state
	Multiplier  uint64 = 0xBF58476D1CE4E5B9 // SplitMix64 multiplier (internals)
	Gamma       uint64 = 0x9E3779B97F4A7C15 // SplitMix64 golden gamma
	FactBytes          = 24                 // 3 x u64 LE per fact
	FACT_WINDOW        = 1365               // pack_demux::FACT_WINDOW, named coprime stride
)

// SplitMix64 is the canonical PRNG (single u64 state, stepped by Gamma).
type SplitMix64 struct{ state uint64 }

// NewSplitMix64 returns a stream seeded with the given state.
func NewSplitMix64(seed uint64) *SplitMix64 { return &SplitMix64{state: seed} }

// Next steps the stream and returns the next u64.
//
// Variant note (measured, IPC-1): TWO SplitMix64 variants coexist in-tree.
// The canonical corpus generator (scripts/generate_cscb_corpus.py, B08)
// applies the multiplier twice with a final shift of 43; the csct ingress
// self-gen (cscb_c1_c2_ingress.rs:52-58) uses the textbook second
// multiplier 0x94D049BB133111EB with shift 31. They produce DIFFERENT
// streams from the same seed (witnessed: first u64 3f4bf12704f198f5 vs
// 159fd140385b5c61). This package mirrors the Python generator (variantA),
// the canonical corpus authority; the ingress self-gen is itself disclosed
// as an in-process divergence.
func (s *SplitMix64) Next() uint64 {
	s.state += Gamma
	z := s.state
	z = (z ^ (z >> 30)) * Multiplier
	z = (z ^ (z >> 27)) * Multiplier
	return z ^ (z >> 43)
}

// Fact is one 24-byte coordinate fact (x, y, z), little-endian on the wire.
type Fact struct{ X, Y, Z uint64 }

// FactWindow returns the batch window (1,365 facts).
func FactWindow() int { return FACT_WINDOW }

// Batch generates the canonical 1,365-fact batch from the frozen corpus seed.
func Batch() []Fact {
	return BatchFromSeed(Seed)
}

// BatchFromSeed generates a 1,365-fact batch from an explicit seed (courts
// use this for the anti-vacuity seed-tweak flip).
func BatchFromSeed(seed uint64) []Fact {
	rng := NewSplitMix64(seed)
	batch := make([]Fact, FACT_WINDOW)
	for i := range batch {
		batch[i] = Fact{X: rng.Next(), Y: rng.Next(), Z: rng.Next()}
	}
	return batch
}

// Encode serializes facts into the 24-bytes-per-fact LE wire format.
func Encode(facts []Fact) []byte {
	buf := make([]byte, len(facts)*FactBytes)
	for i, f := range facts {
		off := i * FactBytes
		binary.LittleEndian.PutUint64(buf[off:], f.X)
		binary.LittleEndian.PutUint64(buf[off+8:], f.Y)
		binary.LittleEndian.PutUint64(buf[off+16:], f.Z)
	}
	return buf
}
