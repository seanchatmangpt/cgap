//go:build linux

// Package shm — LINUX ADAPTATION for lane AX-6 (extends lane IPC-4's
// darwin/arm64 transport court; every line of this file is new in AX-6).
//
// Mechanism probe (named per AX-6 dispatch): on Linux the sandbox/container
// MAY mount /dev/shm as a tmpfs (witnessed this session: docker linux/arm64
// container, `df /dev/shm` -> shm 65536 1K-blocks tmpfs). When present, the
// segment backing file is created IN /dev/shm so the mapping is a true
// POSIX-shared-memory page (what shm_open gives, minus the cgo dependency —
// stdlib-only publish-frozen law holds). When absent (gVisor-class sandboxes
// may not expose it), CreateShm falls back to os.CreateTemp — the exact
// darwin IPC-4 mechanism, which MAP_SHARED mmap makes equivalent. The probe
// result is returned, never guessed, so courts can [SKIP] the devshm arm
// LOUDLY if the runtime lacks it.
package shm

import (
	"os"
	"syscall"

	"avec.chatman.dev/cgap/courts/interop-chicago/corpus"
)

// corpusFactBytesLocal aliases corpus.FactBytes for the backing-size math.
const corpusFactBytesLocal = corpus.FactBytes

// DevShmPath is the Linux POSIX-SHM mount point.
const DevShmPath = "/dev/shm"

// DevShmTmpfs reports whether /dev/shm is a mounted tmpfs (i.e. whether
// true shm_open-style shared memory is available without cgo). It names the
// mechanism actually available instead of assuming one, per AX-6 dispatch.
func DevShmTmpfs() bool {
	var st syscall.Statfs_t
	if err := syscall.Statfs(DevShmPath, &st); err != nil {
		return false
	}
	// TMPFS_MAGIC = 0x01021994 (linux/magic.h).
	return st.Type == 0x01021994
}

// CreateShm makes a segment sized for n facts, backing it in /dev/shm when
// that is a live tmpfs (true POSIX-shared-memory pages) and falling back to
// the portable os.CreateTemp mmap'd temp file otherwise. Both arms map
// MAP_SHARED; the returned Mechanism string records which backing was used
// so the court log names the mechanism, never assumes it.
func CreateShm(n int) (*Segment, string, error) {
	if DevShmTmpfs() {
		f, err := os.CreateTemp(DevShmPath, "cgap-shm-court-*.seg")
		if err != nil {
			return nil, "devshm(tmpfs):create-failed", err
		}
		s, err := createMapped(f, n)
		return s, "devshm(tmpfs)", err
	}
	f, err := os.CreateTemp("", "cgap-shm-court-*.seg")
	if err != nil {
		return nil, "tempfile(create-failed)", err
	}
	s, err := createMapped(f, n)
	return s, "tempfile(mmap MAP_SHARED)", err
}

// createMapped truncates f to header+n facts and maps it MAP_SHARED
// read/write. Shared body with both backing arms (portable syscalls; the
// darwin IPC-4 path in shm.go is byte-for-byte the same primitive).
func createMapped(f *os.File, n int) (*Segment, error) {
	size := int64(HeaderBytes + n*corpusFactBytesLocal)
	if err := f.Truncate(size); err != nil {
		f.Close()
		return nil, err
	}
	b, err := syscall.Mmap(int(f.Fd()), 0, int(size),
		syscall.PROT_READ|syscall.PROT_WRITE, syscall.MAP_SHARED)
	if err != nil {
		f.Close()
		return nil, err
	}
	return &Segment{f: f, b: b, path: f.Name()}, nil
}
