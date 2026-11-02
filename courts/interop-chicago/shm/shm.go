// Package shm implements the macOS transport court for lane IPC-4.
//
// Mechanism (named per dispatch): /dev/shm is ABSENT on macOS (the dispatch
// fence), so the real mechanism is an mmap'd temp file — os.CreateTemp +
// syscall.Mmap (pure stdlib, zero cgo) — mapped MAP_SHARED so producer and
// consumer share one physical backing. This is the same kernel primitive
// POSIX shm_open wraps; the delta is naming, not semantics.
//
// SCOPE (loud, per IPC-0 lane-ipc-00-ground-truth.md W5): transport-only.
// No cgap kernel involvement — cgap_shm_* symbols do not exist in
// libcgap_ffi.dylib (nm, IPC-0 W3). The frozen Arrow slot surface
// (cgap_i32_batch / cgap_schema_addr / cgap_array_addr) is RECORDED here as
// the nearest substrate but NOT bound: binding an absent kernel path would
// fake the integration this court reserves for the FFI roadmap.
//
// Wire layout (little-endian, 32-byte header then facts):
//
//	off  0: magic      u64 = 0x4347415053484D31 ("CGAPSHM1")
//	off  8: tx_id      u64 (monotone transaction id)
//	off 16: fact_count u32
//	off 20: stride     u32 (= corpus.FactBytes = 24)
//	off 24: (4 bytes pad to 8-align) + 4 reserved = header total 32
//	off 32: facts[fact_count], 24 B each (corpus.LE wire format)
package shm

import (
	"encoding/binary"
	"errors"
	"fmt"
	"os"
	"syscall"

	"avec.chatman.dev/cgap/courts/interop-chicago/corpus"
)

// Magic is the typed wire magic ("CGAPSHM1").
const Magic uint64 = 0x4347415053484D31

// HeaderBytes is the fixed header size.
const HeaderBytes = 32

// Header is the typed 32-byte transport header.
type Header struct {
	Magic     uint64
	TxID      uint64
	FactCount uint32
	Stride    uint32
}

// EncodeHeader writes h into buf[0:HeaderBytes] little-endian.
func EncodeHeader(buf []byte, h Header) {
	binary.LittleEndian.PutUint64(buf[0:], h.Magic)
	binary.LittleEndian.PutUint64(buf[8:], h.TxID)
	binary.LittleEndian.PutUint32(buf[16:], h.FactCount)
	binary.LittleEndian.PutUint32(buf[20:], h.Stride)
	// bytes 24..32 reserved, left zero.
}

// DecodeHeader parses buf[0:HeaderBytes].
func DecodeHeader(buf []byte) (Header, error) {
	if len(buf) < HeaderBytes {
		return Header{}, fmt.Errorf("shm: short region %d < %d", len(buf), HeaderBytes)
	}
	return Header{
		Magic:     binary.LittleEndian.Uint64(buf[0:]),
		TxID:      binary.LittleEndian.Uint64(buf[8:]),
		FactCount: binary.LittleEndian.Uint32(buf[16:]),
		Stride:    binary.LittleEndian.Uint32(buf[20:]),
	}, nil
}

// ErrMagic is the typed anti-vacuity error: the mapped header magic did not
// match Magic. Reader must halt, never reassemble from a foreign region.
var ErrMagic = errors.New("shm: header magic mismatch in mapped region")

// ErrStride is the typed stride-mismatch error.
var ErrStride = errors.New("shm: stride mismatch in mapped region")

// ErrCount is the typed count-vs-capacity error.
var ErrCount = errors.New("shm: fact_count exceeds mapped capacity")

// Segment is one MAP_SHARED mmap'd temp-file segment.
type Segment struct {
	f   *os.File
	b   []byte // mapped region, HeaderBytes + facts
	path string
}

// Create makes a new segment sized for n facts and maps it read/write.
func Create(n int) (*Segment, error) {
	f, err := os.CreateTemp("", "cgap-shm-court-*.seg")
	if err != nil {
		return nil, err
	}
	size := int64(HeaderBytes + n*corpus.FactBytes)
	if err := f.Truncate(size); err != nil {
		f.Close()
		return nil, err
	}
	b, err := syscall.Mmap(int(f.Fd()), 0, int(size), syscall.PROT_READ|syscall.PROT_WRITE, syscall.MAP_SHARED)
	if err != nil {
		f.Close()
		return nil, fmt.Errorf("shm: mmap: %w", err)
	}
	return &Segment{f: f, b: b, path: f.Name()}, nil
}

// Open maps an existing segment read-only (the consumer side).
func Open(path string, size int64) (*Segment, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	b, err := syscall.Mmap(int(f.Fd()), 0, int(size), syscall.PROT_READ, syscall.MAP_SHARED)
	if err != nil {
		f.Close()
		return nil, fmt.Errorf("shm: mmap(ro): %w", err)
	}
	return &Segment{f: f, b: b}, nil
}

// Bytes exposes the mapped region (no copy; read path slices this).
func (s *Segment) Bytes() []byte { return s.b }

// Path is the backing file path (consumer opens by path in a second mapping).
func (s *Segment) Path() string { return s.path }

// Close unmaps and removes the backing temp file.
func (s *Segment) Close() error {
	err := syscall.Munmap(s.b)
	if ferr := s.f.Close(); ferr != nil && err == nil {
		err = ferr
	}
	if s.path != "" {
		os.Remove(s.path)
	}
	return err
}

// WriteBatch writes the canonical corpus batch under the given tx_id into
// the mapped region: header then facts, straight into shared memory.
func (s *Segment) WriteBatch(txID uint64, facts []corpus.Fact) error {
	h := Header{Magic: Magic, TxID: txID, FactCount: uint32(len(facts)), Stride: corpus.FactBytes}
	if HeaderBytes+len(facts)*corpus.FactBytes > len(s.b) {
		return ErrCount
	}
	EncodeHeader(s.b, h)
	wire := corpus.Encode(facts)
	copy(s.b[HeaderBytes:], wire)
	return nil
}

// ReadFacts validates the header (typed halt on bad magic/stride) and
// returns facts COPIED OUT of the region. It is the correctness arm.
func ReadFacts(b []byte) ([]corpus.Fact, error) {
	h, err := DecodeHeader(b)
	if err != nil {
		return nil, err
	}
	if err := validate(h, len(b)); err != nil {
		return nil, err
	}
	facts := make([]corpus.Fact, h.FactCount)
	for i := range facts {
		off := HeaderBytes + i*corpus.FactBytes
		facts[i] = corpus.Fact{
			X: binary.LittleEndian.Uint64(b[off:]),
			Y: binary.LittleEndian.Uint64(b[off+8:]),
			Z: binary.LittleEndian.Uint64(b[off+16:]),
		}
	}
	return facts, nil
}

// ReadZeroCopy validates the header and returns a slice VIEW of the mapped
// fact region (no copy, no heap reassembly) plus the validated header. The
// caller must not retain the view past Unmap.
func ReadZeroCopy(b []byte) ([]corpus.Fact, Header, error) {
	h, err := DecodeHeader(b)
	if err != nil {
		return nil, Header{}, err
	}
	if err := validate(h, len(b)); err != nil {
		return nil, Header{}, err
	}
	view := unsafeSliceFacts(b[HeaderBytes : HeaderBytes+int(h.FactCount)*corpus.FactBytes])
	return view, h, nil
}

func validate(h Header, cap int) error {
	if h.Magic != Magic {
		return ErrMagic
	}
	if h.Stride != corpus.FactBytes {
		return ErrStride
	}
	if HeaderBytes+int(h.FactCount)*corpus.FactBytes > cap {
		return ErrCount
	}
	return nil
}
