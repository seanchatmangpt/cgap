package shm

import (
	"unsafe"

	"avec.chatman.dev/cgap/courts/interop-chicago/corpus"
)

// unsafeSliceFacts reinterprets a mapped wire buffer as a []corpus.Fact view
// WITHOUT copying. Pure Go `unsafe`, zero cgo: the mapping is plain memory.
func unsafeSliceFacts(b []byte) []corpus.Fact {
	return unsafe.Slice((*corpus.Fact)(unsafe.Pointer(&b[0])), len(b)/corpus.FactBytes)
}
