//go:build linux

// Lane AX-6 LINUX court additions. Chicago discipline: real mmap'd segments,
// real second mapping for the consumer, assert final state, no mocks.
// ANTI-VACUITY named per court: corrupting the mapped header magic (in either
// backing arm) must flip the reader to typed ErrMagic; a green run on
// healthy bytes alone is VACUOUS. A missing /dev/shm is a LOUD [SKIP] on the
// devshm arm only — never a fake pass — and the tempfile arm still runs.
package shm

import (
	"bytes"
	"errors"
	"testing"

	"avec.chatman.dev/cgap/courts/interop-chicago/corpus"
)

// court_devshm_roundtrip (linux): if /dev/shm is a live tmpfs, the segment
// backs onto true POSIX-shared-memory pages; producer writes the canonical
// batch, consumer opens an independent read-only mapping, byte-exact final
// state asserted. If the sandbox exposes no /dev/shm, the arm is a LOUD
// [SKIP] with the probe result logged — the tempfile court below still runs.
func TestDevShmRoundtrip(t *testing.T) {
	if !DevShmTmpfs() {
		t.Skipf("[SKIP] /dev/shm not a tmpfs in this sandbox (probe Statfs(%s) failed or wrong magic); tempfile arm still runs", DevShmPath)
	}
	seg, mech, err := CreateShm(corpus.FACT_WINDOW)
	if err != nil {
		t.Fatalf("CreateShm: %v", err)
	}
	defer seg.Close()
	if mech != "devshm(tmpfs)" {
		t.Fatalf("mechanism %q, want devshm(tmpfs) — probe/mount disagreement", mech)
	}
	if err := seg.WriteBatch(0x2A, corpus.Batch()); err != nil {
		t.Fatalf("WriteBatch: %v", err)
	}
	cons, err := Open(seg.Path(), segSize)
	if err != nil {
		t.Fatalf("Open(second mapping): %v", err)
	}
	defer cons.Close()
	view, h, err := ReadZeroCopy(cons.Bytes())
	if err != nil {
		t.Fatalf("ReadZeroCopy: %v", err)
	}
	if h.TxID != 0x2A || h.FactCount != corpus.FACT_WINDOW || h.Stride != corpus.FactBytes {
		t.Fatalf("header: %+v", h)
	}
	if !bytes.Equal(cons.Bytes()[HeaderBytes:], corpus.Encode(corpus.Batch())) {
		t.Fatal("devshm facts not byte-exact vs corpus.Encode(batch)")
	}
	if len(view) != corpus.FACT_WINDOW {
		t.Fatalf("view len %d != %d", len(view), corpus.FACT_WINDOW)
	}
}

// ANTI-VACUITY (linux): corrupt the mapped header magic in the devshm/tmpfs
// backing -> typed ErrMagic; restore -> pass again. Flip both directions.
func TestDevShmMagicCorruptionFlip(t *testing.T) {
	if !DevShmTmpfs() {
		t.Skipf("[SKIP] /dev/shm not a tmpfs; tempfile flip court covers anti-vacuity")
	}
	seg, _, err := CreateShm(corpus.FACT_WINDOW)
	if err != nil {
		t.Fatalf("CreateShm: %v", err)
	}
	defer seg.Close()
	if err := seg.WriteBatch(1, corpus.Batch()); err != nil {
		t.Fatal(err)
	}
	if _, _, err := ReadZeroCopy(seg.Bytes()); err != nil {
		t.Fatalf("baseline read must pass: %v", err)
	}
	seg.Bytes()[0] ^= 0xFF
	if _, _, err := ReadZeroCopy(seg.Bytes()); !errors.Is(err, ErrMagic) {
		t.Fatalf("flip FAILED: want ErrMagic, got %v", err)
	}
	seg.Bytes()[0] ^= 0xFF
	if _, _, err := ReadZeroCopy(seg.Bytes()); err != nil {
		t.Fatalf("post-restore read must pass: %v", err)
	}
}

// BenchmarkMmapReadLinux: linux/arm64 re-measure of the IPC-4 mmap arm via
// the CreateShm path (mechanism logged, not assumed), for the delta table.
func BenchmarkMmapReadLinux(b *testing.B) {
	seg, mech, err := CreateShm(corpus.FACT_WINDOW)
	if err != nil {
		b.Fatal(err)
	}
	defer seg.Close()
	b.Logf("backing mechanism: %s (DevShmTmpfs=%v)", mech, DevShmTmpfs())
	if err := seg.WriteBatch(9, corpus.Batch()); err != nil {
		b.Fatal(err)
	}
	cons, err := Open(seg.Path(), segSize)
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
