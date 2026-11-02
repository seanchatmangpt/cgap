package shm

// Lane IPC-4 transport court. Chicago discipline: real mmap'd temp file, real
// second mapping for the consumer, assert final state. ANTI-VACUITY named:
// corrupting the mapped header magic must flip the reader to ErrMagic.

import (
	"bytes"
	"errors"
	"syscall"
	"testing"

	"avec.chatman.dev/cgap/courts/interop-chicago/corpus"
)

const segSize = int64(HeaderBytes + corpus.FACT_WINDOW*corpus.FactBytes)

// court_transport_roundtrip: producer writes 1,365 facts into the mapped
// region; consumer opens a SECOND independent read-only mapping of the same
// backing file and reads byte-exact via the zero-copy view.
func TestTransportRoundtrip(t *testing.T) {
	batch := corpus.Batch()
	prod, err := Create(corpus.FACT_WINDOW)
	if err != nil {
		t.Fatalf("Create: %v", err)
	}
	defer prod.Close()

	if err := prod.WriteBatch(0x2A, batch); err != nil {
		t.Fatalf("WriteBatch: %v", err)
	}

	// Consumer: independent PROT_READ mapping of the same file.
	cons, err := Open(prod.Path(), segSize)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer cons.Close()

	view, h, err := ReadZeroCopy(cons.Bytes())
	if err != nil {
		t.Fatalf("ReadZeroCopy: %v", err)
	}
	if h.Magic != Magic || h.TxID != 0x2A {
		t.Fatalf("header: %+v", h)
	}
	if h.FactCount != corpus.FACT_WINDOW || h.Stride != corpus.FactBytes {
		t.Fatalf("header counts: %+v", h)
	}
	if len(view) != corpus.FACT_WINDOW {
		t.Fatalf("view len %d != %d", len(view), corpus.FACT_WINDOW)
	}
	// Byte-exact final-state assertion against the generator's LE wire form.
	if !bytes.Equal(cons.Bytes()[HeaderBytes:], corpus.Encode(batch)) {
		t.Fatal("mapped facts not byte-exact vs corpus.Encode(batch)")
	}
	for i, f := range batch {
		if view[i] != f {
			t.Fatalf("fact %d: mapped %+v != generated %+v", i, view[i], f)
		}
	}
}

// ANTI-VACUITY: corrupt the mapped header magic -> reader halts with typed
// ErrMagic. Flip: healthy region passes, corrupted region fails.
func TestMagicCorruptionFlip(t *testing.T) {
	prod, err := Create(corpus.FACT_WINDOW)
	if err != nil {
		t.Fatalf("Create: %v", err)
	}
	defer prod.Close()
	if err := prod.WriteBatch(1, corpus.Batch()); err != nil {
		t.Fatalf("WriteBatch: %v", err)
	}
	if _, _, err := ReadZeroCopy(prod.Bytes()); err != nil {
		t.Fatalf("baseline read must pass: %v", err)
	}

	// Mutate the mapped bytes in place (sandbox object we own; not a repo tree).
	prod.Bytes()[0] ^= 0xFF
	_, _, err = ReadZeroCopy(prod.Bytes())
	if !errors.Is(err, ErrMagic) {
		t.Fatalf("flip FAILED: want ErrMagic, got %v", err)
	}

	// Restore + re-prove the reader still passes on healthy bytes.
	prod.Bytes()[0] ^= 0xFF
	if _, _, err := ReadZeroCopy(prod.Bytes()); err != nil {
		t.Fatalf("post-restore read must pass: %v", err)
	}
}

// Zero-heap-reassembly read path: AllocsPerRun over the validated zero-copy
// read. cgo is absent (stdlib Mmap), so the read loop itself should be 0.
func TestZeroAllocReadPath(t *testing.T) {
	prod, err := Create(corpus.FACT_WINDOW)
	if err != nil {
		t.Fatalf("Create: %v", err)
	}
	defer prod.Close()
	if err := prod.WriteBatch(7, corpus.Batch()); err != nil {
		t.Fatalf("WriteBatch: %v", err)
	}
	reg := prod.Bytes()

	var hdr Header
	allocs := testing.AllocsPerRun(100, func() {
		view, h, err := ReadZeroCopy(reg)
		if err != nil {
			t.Fatalf("read: %v", err)
		}
		if len(view) != corpus.FACT_WINDOW {
			t.Fatal("short view")
		}
		hdr = h
	})
	t.Logf("read-path allocs/run = %v (Header tx_id=%d)", allocs, hdr.TxID)
	if allocs != 0 {
		t.Fatalf("read path allocates: %v allocs/run (want 0; no heap reassembly)", allocs)
	}
}

// FileReadBaseline: the honest [SKIP]-avoidance comparison arm — a plain
// os.File read loop over the same bytes, cost measured in the receipt table.
func BenchmarkMmapRead(b *testing.B) {
	prod, err := Create(corpus.FACT_WINDOW)
	if err != nil {
		b.Fatal(err)
	}
	defer prod.Close()
	prod.WriteBatch(9, corpus.Batch())
	cons, err := Open(prod.Path(), segSize)
	if err != nil {
		b.Fatal(err)
	}
	defer cons.Close()
	reg := cons.Bytes()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if _, _, err := ReadZeroCopy(reg); err != nil {
			b.Fatal(err)
		}
	}
}

// One real syscall delta recorded for the cost table: mmap vs pread of the
// segment (measured, not asserted on wall-clock ratios).
func TestFileReadBaselineRoundtrip(t *testing.T) {
	prod, err := Create(corpus.FACT_WINDOW)
	if err != nil {
		t.Fatalf("Create: %v", err)
	}
	defer prod.Close()
	if err := prod.WriteBatch(3, corpus.Batch()); err != nil {
		t.Fatal(err)
	}
	// Consumer arm WITHOUT mmap: pread the whole segment into one buffer.
	f, err := syscall.Open(prod.Path(), syscall.O_RDONLY, 0)
	if err != nil {
		t.Fatal(err)
	}
	defer syscall.Close(f)
	buf := make([]byte, segSize)
	if n, err := syscall.Pread(f, buf, 0); err != nil || int64(n) != segSize {
		t.Fatalf("pread: n=%d err=%v", n, err)
	}
	got, err := ReadFacts(buf)
	if err != nil {
		t.Fatal(err)
	}
	want := corpus.Batch()
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("file arm fact %d mismatch", i)
		}
	}
}

// BenchmarkPreadRead: the fallback-arm cost (one pread per read), for the
// receipt cost table. Honest delta vs the mmap arm.
func BenchmarkPreadRead(b *testing.B) {
	prod, err := Create(corpus.FACT_WINDOW)
	if err != nil {
		b.Fatal(err)
	}
	defer prod.Close()
	prod.WriteBatch(9, corpus.Batch())
	fd, err := syscall.Open(prod.Path(), syscall.O_RDONLY, 0)
	if err != nil {
		b.Fatal(err)
	}
	defer syscall.Close(fd)
	buf := make([]byte, segSize)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if _, err := syscall.Pread(fd, buf, 0); err != nil {
			b.Fatal(err)
		}
	}
}
