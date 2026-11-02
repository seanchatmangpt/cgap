// Package ffi — cgo bindings over the cgap-ffi cdylib membrane
// (crates/cgap-ffi), plus the IPC-2 alloc-purity membrane courts.
//
// blake3.go: minimal standard BLAKE3 (32-byte digest mode, no key),
// ported 1:1 from the repo canon crates/cgap-blake3/src/lib.rs
// (compress_scalar + hash_chunk + stack Merkle, ROOT folded into the
// last-block compress — byte-identical to cgap-blake3 and to the
// official BLAKE3 vectors that crate passes). The admit positive
// control in alloc_court_test.go is the end-to-end witness.
package ffi

import "encoding/binary"

const (
	chunkLen = 1024
	blockLen = 64
)

var blake3IV = [8]uint32{
	0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
	0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
}

const (
	flagChunkStart = 1 << 0
	flagChunkEnd   = 1 << 1
	flagParent     = 1 << 2
	flagRoot       = 1 << 3
)

// msgPerm is the standard BLAKE3 message-word permutation (lib.rs:20).
var msgPerm = [16]uint{2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8}

func g(s *[16]uint32, a, b, c, d int, mx, my uint32) {
	s[a] = s[a] + s[b] + mx
	s[d] = rotr(s[d]^s[a], 16)
	s[c] = s[c] + s[d]
	s[b] = rotr(s[b]^s[c], 12)
	s[a] = s[a] + s[b] + my
	s[d] = rotr(s[d]^s[a], 8)
	s[c] = s[c] + s[d]
	s[b] = rotr(s[b]^s[c], 7)
}

func rotr(x uint32, n uint) uint32 {
	return x>>n | x<<(32-n)
}

// compress is compress_scalar (lib.rs:77) verbatim: state =
// [cv, IV[0..4], counter_lo, counter_hi, block_len, flags]; 7 rounds
// with full message permutation between rounds; final fold.
func compress(cv *[8]uint32, block []byte, counter uint64, nBlock, flags uint32) [16]uint32 {
	var ms [16]uint32
	for i := 0; i < 16; i++ {
		ms[i] = binary.LittleEndian.Uint32(block[i*4 : i*4+4])
	}
	var s [16]uint32
	copy(s[0:8], cv[:])
	copy(s[8:12], blake3IV[0:4])
	s[12] = uint32(counter)
	s[13] = uint32(counter >> 32)
	s[14] = nBlock
	s[15] = flags

	for r := 0; r < 7; r++ {
		for i := 0; i < 4; i++ {
			g(&s, i, i+4, i+8, i+12, ms[i*2], ms[i*2+1])
		}
		diag := [4][4]int{{0, 5, 10, 15}, {1, 6, 11, 12}, {2, 7, 8, 13}, {3, 4, 9, 14}}
		for i := 0; i < 4; i++ {
			q := diag[i]
			g(&s, q[0], q[1], q[2], q[3], ms[8+i*2], ms[8+i*2+1])
		}
		if r < 6 {
			var next [16]uint32
			for i := 0; i < 16; i++ {
				next[i] = ms[msgPerm[i]]
			}
			ms = next
		}
	}
	for i := 0; i < 8; i++ {
		s[i] ^= s[i+8]
		s[i+8] ^= cv[i]
	}
	return s
}

// chunkCV folds `data` (0..=1024 B) as one chunk at `counter`,
// 64-byte blocks; ROOT goes into the last block when isRoot
// (lib.rs:154 hash_chunk twin).
func chunkCV(data []byte, counter uint64, isRoot bool) [8]uint32 {
	var cv [8]uint32 = blake3IV
	nBlocks := (len(data) + blockLen - 1) / blockLen
	if nBlocks == 0 {
		nBlocks = 1 // empty input: one zero block, START|END
	}
	for b := 0; b < nBlocks; b++ {
		var block [64]byte
		start := b * blockLen
		end := start + blockLen
		if end > len(data) {
			end = len(data)
		}
		copy(block[:], data[start:end])
		flags := uint32(0)
		if b == 0 {
			flags |= flagChunkStart
		}
		if b == nBlocks-1 {
			flags |= flagChunkEnd
			if isRoot {
				flags |= flagRoot
			}
		}
		n := uint32(end - start)
		full := compress(&cv, block[:], counter, n, flags)
		copy(cv[:], full[0:8])
	}
	return cv
}

// parentCV merges two chaining values (PARENT flag; ROOT folded in on
// the final/root merge, cgap-blake3 hash() Merkle twin).
func parentCV(left, right [8]uint32, isRoot bool) [8]uint32 {
	var block [64]byte
	for i := 0; i < 8; i++ {
		binary.LittleEndian.PutUint32(block[i*4:i*4+4], right[i])
	}
	cv := left
	flags := uint32(flagParent)
	if isRoot {
		flags |= flagRoot
	}
	full := compress(&cv, block[:], 0, blockLen, flags)
	var out [8]uint32
	copy(out[:], full[0:8])
	return out
}

// Hash returns the 32-byte BLAKE3 digest (no key), stack-Merkle over
// 1 KiB chunks, root merge carrying ROOT (lib.rs hash() twin).
func Hash(data []byte) [32]byte {
	nChunks := (len(data) + chunkLen - 1) / chunkLen
	if nChunks == 0 {
		nChunks = 1
	}
	if nChunks == 1 {
		// Single chunk IS the root node (lib.rs fast path twin).
		return serveDigest(chunkCV(data, 0, true))
	}
	var stack [54][8]uint32
	top := 0
	for i := 0; i < nChunks; i++ {
		start := i * chunkLen
		end := start + chunkLen
		if end > len(data) {
			end = len(data)
		}
		cv := chunkCV(data[start:end], uint64(i), false)
		total := i + 1
		for total%2 == 0 && top > 0 {
			cv = parentCV(stack[top-1], cv, false)
			top--
			total /= 2
		}
		stack[top] = cv
		top++
	}
	cv := stack[top-1]
	top--
	for top > 0 {
		cv = parentCV(stack[top-1], cv, top == 1)
		top--
	}
	var out [32]byte
	for i := 0; i < 8; i++ {
		binary.LittleEndian.PutUint32(out[i*4:i*4+4], cv[i])
	}
	return out
}

func serveDigest(cv [8]uint32) [32]byte {
	var out [32]byte
	for i := 0; i < 8; i++ {
		binary.LittleEndian.PutUint32(out[i*4:i*4+4], cv[i])
	}
	return out
}
