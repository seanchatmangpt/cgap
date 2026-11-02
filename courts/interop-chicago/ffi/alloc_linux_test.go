//go:build linux
// +build linux

// alloc_linux_test.go — Lane AX-4 linux-arm64 flip witness. The darwin
// TestAntiVacuityFlip (alloc_court_test.go:230-246) measured 0 allocs/op
// inside the golang:1.23-bookworm container: Go stack-allocates the
// `sink` backing array because the closure never lets it escape after
// the append loop (dead-store-eliminable). This court is the linux
// adaptation: the closure writes through a PACKAGE-LEVEL sink pointer,
// which escape analysis cannot demote, so AllocsPerRun must count real
// heap allocations. A court that cannot flip is FAIL.
package ffi

import "testing"

// sinkLinux is written only through the pointer captured below — a
// package-level variable, so the backing store escapes.
var sinkLinux []int

func TestAntiVacuityFlipLinux(t *testing.T) {
	flip := func() {
		for i := 0; i < 4; i++ {
			sinkLinux = append(sinkLinux[:0], i)
			sinkLinux = append(sinkLinux, i)
		}
	}
	avgFlip := testing.AllocsPerRun(1_000, flip)
	if avgFlip <= 0.0 {
		t.Fatalf("ANTI-VACUITY: escaping closure measured %v allocs/op — court cannot flip", avgFlip)
	}
	avgNoop := testing.AllocsPerRun(1_000, func() {})
	if avgNoop != 0.0 {
		t.Fatalf("no-op closure measured %v allocs/op (harness drift)", avgNoop)
	}
	t.Logf("linux flip witnessed: escaping closure = %v allocs/op, no-op = %v", avgFlip, avgNoop)
}
