//go:build linux
// +build linux

// mmap_linux.go — Lane AX-4 linux-arm64 syscall.Mmap constants. The
// darwin shm courts (courts/interop-chicago/shm) call syscall.Mmap with
// darwin flag values; the linux flag words differ. Constants only — no
// darwin file is touched (build-tag isolated, //go:build linux).
//
// Every line disclosed in lane-ax-04-alloc.md.
package ffi

// Linux arm64 mmap protection flags (asm-generic/mman-common.h;
// values identical on arm64, cited not redefined).
const (
	mmapProtRead  = 0x1 // PROT_READ
	mmapProtWrite = 0x2 // PROT_WRITE
)

// Linux arm64 mmap mapping flags.
const (
	mmapMapShared    = 0x01 // MAP_SHARED
	mmapMapPrivate   = 0x02 // MAP_PRIVATE
	mmapMapAnonymous = 0x20 // MAP_ANONYMOUS — darwin's MAP_ANON is 0x1000; the word differs, the semantic does not
)
