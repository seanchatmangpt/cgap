// alloc_court_test.go — IPC-2 alloc-purity courts (Go testing.AllocsPerRun)
// over the cgap-ffi cdylib membrane. Lane receipt:
// docs/jira/v26.11.2/lanes/lane-ipc-02-alloc-purity.md.
package ffi

import (
	"encoding/binary"
	"encoding/hex"
	"runtime"
	"testing"
	"unsafe"
)

const urnLicenseP28 = "urn:avec:license:p28-sil:v2.1.0-sovereign"

// k is SlotConst (sigil 0x00): the term word IS the value
// (sovereign.rs:3749 k() twin, cited not redefined).
func k(v uint64) uint64 { return v }

// uni is VarUni (sigil 0x01 << 56 | reg).
func uni(reg uint64) uint64 { return 0x01<<56 | reg }

// storePayload is the pinned three-row store (sovereign.rs store_columns
// twin): rows (1,2,100), (5,2,100), (9,3,250), columnar s|p|o LE u64
// words — 72 bytes.
func storePayload() []byte {
	cols := []uint64{
		k(1), k(5), k(9), // s column
		k(2), k(2), k(3), // p column
		k(100), k(100), k(250), // o column
	}
	b := make([]byte, 0, len(cols)*8)
	var tmp [8]byte
	for _, w := range cols {
		binary.LittleEndian.PutUint64(tmp[:], w)
		b = append(b, tmp[:]...)
	}
	return b
}

// oneRuleNetwork is the pinned one-rule network (sovereign.rs
// one_rule_network twin): (s ?x, p 2, o 100) => (7, 8, ?x); matches
// rows 0 and 1, lanes = projected ?0 = [1, 5].
func oneRuleNetwork() []byte {
	words := []uint64{
		1,                  // rule_count
		42,                 // rule_id
		k(7), k(8), uni(0), // head (7, 8, ?0)
		1,                    // body_len
		uni(0), k(2), k(100), // body (?0, 2, 100)
	}
	b := make([]byte, 0, len(words)*8)
	var tmp [8]byte
	for _, w := range words {
		binary.LittleEndian.PutUint64(tmp[:], w)
		b = append(b, tmp[:]...)
	}
	return b
}

// wireFrame builds the 128-byte A04 header + payload frame with the
// BLAKE3 digest embedded and FLAG_BN3 (sovereign.rs wire_frame twin).
func wireFrame(payload []byte) []byte {
	frame := make([]byte, 128+len(payload))
	copy(frame[0:4], "CGAP")
	frame[4] = 1    // FRAME_VERSION
	frame[5] = 0x04 // FLAG_BN3
	// [6:8] reserved zero-on-write (A04 §6)
	binary.LittleEndian.PutUint64(frame[8:16], uint64(len(payload)))
	d := Hash(payload)
	copy(frame[16:48], d[:])
	copy(frame[128:], payload) // AX-4 unblock: payload must be IN the frame buffer (see lane-ax-04 receipt §unblock)
	// attestation (A17 §2.1 offsets 48..128): P28 magic, ver 1, ROOT
	// flag, max safe-harbor ceiling, origin digest, zero upstream.
	copy(frame[48:52], "P28S")
	binary.LittleEndian.PutUint16(frame[52:54], 1)
	binary.LittleEndian.PutUint16(frame[54:56], 0x01) // LICENSE_FLAG_ROOT
	binary.LittleEndian.PutUint64(frame[56:64], 5_000_000_000)
	origin := Hash([]byte(urnLicenseP28))
	copy(frame[64:96], origin[:])
	// [96:128] upstream_receipt zeros (Root)
	copy(frame[128:], payload)
	return frame
}

// wireFrameCorruptDigest flips one oabi_digest byte — the admit gate
// MUST refuse typed (anti-vacuity for the admit positive control).
func wireFrameCorruptDigest(payload []byte) []byte {
	f := wireFrame(payload)
	f[20] ^= 0xFF
	return f
}

func TestBLAKE3OfficialEmptyVector(t *testing.T) {
	d := Hash(nil)
	got := hex.EncodeToString(d[:])
	const want = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
	if got != want {
		t.Fatalf("BLAKE3 empty-input known-answer FAILED: got %s", got)
	}
}

// TestAdmitPositiveControl: a Go-built frame with a Go-BLAKE3 digest
// must ADMIT through the real sovereign body — the end-to-end Go/Rust
// digest parity witness (sovereign.rs:443 verify_payload re-derives
// with the real cgap-blake3). A corrupted digest must REFUSE typed.
func TestAdmitPositiveControl(t *testing.T) {
	payload := storePayload()
	frame := wireFrame(payload)
	var view View
	st := AdmitFrame(frame, &view)
	if st != StatusOK {
		t.Fatalf("valid frame must admit; status %#x (Go BLAKE3 vs Rust canon drift?)", st)
	}
	if uint64(view.PayloadLen) != uint64(len(payload)) {
		t.Fatalf("view.payload_len = %d, want %d", uint64(view.PayloadLen), len(payload))
	}
	// Zero-copy: view payload must sit inside the Go frame buffer, past
	// the 128-byte header (include/cgap.h §"cgap_frame_view").
	base := uintptr(unsafe.Pointer(&frame[0]))
	got := uintptr(unsafe.Pointer(view.Payload))
	if got != base+128 {
		t.Fatalf("view.payload = %#x, want base+128 (%#x) — zero-copy law", got, base+128)
	}
	// Corruption leg: flipped digest byte -> typed refusal, never OK.
	stc := AdmitFrame(wireFrameCorruptDigest(payload), nil)
	if stc == StatusOK {
		t.Fatalf("corrupted digest admitted — admit gate is vacuous")
	}
	t.Logf("corrupt-digest typed refusal status: %#x", stc)
}

// TestEvalPositiveControl: the real one-rule network over the real
// three-row store yields exactly 2 matches, lanes [1 5] (projected ?0),
// deterministic across a second membrane crossing
// (sovereign.rs:4016-4042 twin).
func TestEvalPositiveControl(t *testing.T) {
	payload := storePayload()
	frame := wireFrame(payload)
	var view View
	if st := AdmitFrame(frame, &view); st != StatusOK {
		t.Fatalf("admit failed: %#x", st)
	}
	net := oneRuleNetwork()
	lanes := make([]int32, 8)
	var count SizeT
	if st := EvalBn3(&view, net, lanes, &count); st != StatusOK {
		t.Fatalf("eval failed: %#x", st)
	}
	if int(count) != 2 {
		t.Fatalf("eval count = %d, want 2", int(count))
	}
	if lanes[0] != 1 || lanes[1] != 5 {
		t.Fatalf("lanes = %v, want [1 5]", lanes[:int(count)])
	}
	lanes2 := make([]int32, 8)
	var count2 SizeT
	if st := EvalBn3(&view, net, lanes2, &count2); st != StatusOK || count2 != count {
		t.Fatalf("eval not deterministic: st %#x count %d", st, int(count2))
	}
	for i := 0; i < int(count); i++ {
		if lanes2[i] != lanes[i] {
			t.Fatalf("lane %d differs across calls: %d vs %d", i, lanes2[i], lanes[i])
		}
	}
}

// TestSteadyStateAllocPurity is the CTQ-01 court: the steady-state
// membrane loop (admit -> eval, one full membrane crossing each) measured
// with testing.AllocsPerRun(1_000_000, ...).
//
// HONEST MEASUREMENT (not a 0.0 claim): the measured steady-state tax is
// 1.0 allocs/op, decomposed in TestMembraneTaxDecomposition as the
// per-op view mint (`var view View` escapes to the Go heap because its
// address crosses the cgo boundary — the zero-copy view protocol's real
// Go-side cost). The cgo transition mechanism, the runtime.Pinner
// crossings, the null-view admit, the eval, and the version read all
// measure 0.0 individually (component courts below). Per the dispatch
// honesty clause this tax is recorded as the finding, NOT masked by
// hoisting the view out of the measured loop — that would measure a
// different (view-reuse) protocol, not the real steady state. The
// Rust-side eval Vec allocations (sovereign.rs:585+) are a further
// system-allocator tax invisible to AllocsPerRun, recorded in the
// lane receipt.
func TestSteadyStateAllocPurity(t *testing.T) {
	payload := storePayload()
	frame := wireFrame(payload)
	net := oneRuleNetwork()
	lanes := make([]int32, 8)
	var count SizeT

	one := func() {
		var view View
		if st := AdmitFrame(frame, &view); st != StatusOK {
			t.Fatalf("admit %#x", st)
		}
		if st := EvalBn3(&view, net, lanes, &count); st != StatusOK {
			t.Fatalf("eval %#x", st)
		}
	}

	// Warm-up: anything lazy (dylib rebase, cgo mechanism init) must land
	// outside the measured window — but nothing the loop needs per-op is
	// pre-built here beyond the fixed working set above.
	one()
	one()

	avg := testing.AllocsPerRun(1_000_000, one)
	const measuredTax = 1.0 // view-mint alloc/op, see decomposition court
	if avg != measuredTax {
		t.Fatalf("steady-state membrane tax DRIFTED: measured %v allocs/op, pinned %v (view-mint decomposition in TestMembraneTaxDecomposition)", avg, measuredTax)
	}
	if int(count) != 2 || lanes[0] != 1 || lanes[1] != 5 {
		t.Fatalf("steady-state eval wrong: count %d lanes %v", int(count), lanes[:int(count)])
	}
}

// TestMembraneTaxDecomposition pins WHICH op carries the 1.0 allocs/op:
// only the admit crossing that mints a Go View (24-byte struct escaping
// to the heap via the cgo boundary). Null-view admit, eval, pinner
// crossings, and the version read are individually 0.0. Anti-vacuity:
// any of these components drifting (e.g. cgo growing a per-call alloc)
// flips the respective number.
func TestMembraneTaxDecomposition(t *testing.T) {
	payload := storePayload()
	frame := wireFrame(payload)
	net := oneRuleNetwork()
	lanes := make([]int32, 8)
	var count SizeT
	var view View
	if st := AdmitFrame(frame, &view); st != StatusOK {
		t.Fatalf("admit %#x", st)
	}

	admitView := testing.AllocsPerRun(100_000, func() {
		var v View
		AdmitFrame(frame, &v)
	})
	admitNull := testing.AllocsPerRun(100_000, func() {
		AdmitFrame(frame, nil)
	})
	evalOnly := testing.AllocsPerRun(100_000, func() {
		EvalBn3(&view, net, lanes, &count)
	})
	pinOnly := testing.AllocsPerRun(100_000, func() {
		var pin runtime.Pinner
		pin.Pin(&frame[0])
		pin.Unpin()
	})
	versionRead := testing.AllocsPerRun(100_000, func() {
		sinkVersionPtr = VersionPtr()
	})

	if admitView != 1.0 {
		t.Fatalf("view-mint admit = %v allocs/op, pinned 1.0", admitView)
	}
	for name, v := range map[string]float64{
		"null-view admit": admitNull, "eval": evalOnly,
		"pinner": pinOnly, "version read": versionRead,
	} {
		if v != 0.0 {
			t.Fatalf("MEMBRANE TAX DRIFT: %s = %v allocs/op (was 0.0)", name, v)
		}
	}
}

// TestVersionAllocPurity: the frozen-slot version string crosses the
// membrane without Go-heap allocation when served as a raw C pointer
// read (no GoString copy in the measured op).
func TestVersionAllocPurity(t *testing.T) {
	if Version() == "" {
		t.Fatalf("cgap_version returned empty")
	}
	avg := testing.AllocsPerRun(1_000_000, func() {
		sinkVersionPtr = VersionPtr()
	})
	if avg != 0.0 {
		t.Fatalf("MEMBRANE TAX FINDING: version membrane crossing = %v allocs/op", avg)
	}
}

var sinkVersionPtr uintptr

// TestAntiVacuityFlip: a deliberately escaping closure (append in loop)
// MUST drive AllocsPerRun above zero, and a no-op closure MUST measure
// 0.0 — both directions witnessed. A court that cannot flip is FAIL.
var sink []int // AX-4 SLA unblock: package-level so the assignment is a real escape (see lane-ax-04 receipt)

func TestAntiVacuityFlip(t *testing.T) {
	flip := func() {
		// fresh heap target per op: the warm-up pass must not absorb the
		// allocation (AllocsPerRun runs f once before the measured N)
		s := make([]int, 4)
		for i := 0; i < 4; i++ {
			s[i] = i
		}
		sink = s
		_ = sink
	}
	avgFlip := testing.AllocsPerRun(1_000, flip)
	if avgFlip <= 0.0 {
		t.Fatalf("ANTI-VACUITY: escaping closure measured %v allocs/op — court cannot flip", avgFlip)
	}
	avgNoop := testing.AllocsPerRun(1_000, func() {})
	if avgNoop != 0.0 {
		t.Fatalf("no-op closure measured %v allocs/op (harness drift)", avgNoop)
	}
	t.Logf("flip witnessed: escaping closure = %v allocs/op, no-op = %v", avgFlip, avgNoop)
}
