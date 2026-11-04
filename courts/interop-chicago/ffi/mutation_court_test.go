// mutation_court_test.go — LANE AX-7 mutation court, Linux-container
// adaptation of the four POSH-5 arms
// (crates/cgap-git/tests/posh_refusal_matrix_court.rs).
//
// SOURCE DISCLOSURE (dispatch law): lane IPC-5 did NOT land — no
// lane-ipc-05* receipt exists on disk at dispatch time (witnessed:
// docs/jira/v26.11.2/lanes/ listing, 2026-11-02) and no
// mutation_court_test.go existed anywhere under courts/. This file is
// therefore ported from POSH-5's landed refusal matrix, per dispatch,
// and adapted from the cgap-git Rust surface (Demux / RefTable /
// IngressRing) to the C-ABI surface that actually crosses the
// membrane: cgap_admit_frame + cgap_eval_bn3 (IPC-0 W3 11-symbol
// table, lane-ipc-00-ground-truth.md).
//
// ARM MAP (POSH-5 arm -> AX-7 in-container arm):
//  1. bad magic    -> magic flip CGAP->XGAP -> typed INVALID_MAGIC
//     (0x43470001), the FFI analog of Demux
//     SubjectMissing.
//  2. cyclic inject-> duplicate/out-of-order fact coordinates through
//     the REAL eval path -> result differs (both
//     results witnessed).
//  3. stride overf -> overflow beyond the derived
//     INGRESS_RING_BYTES/FACT_STRIDE bound -> typed
//     Overflow. THE C-ABI HAS NO INGRESS EXPORT
//     (IPC-0 W5: batch ingress MISSING at C-ABI), so
//     this arm is [SKIP-MISSING-EXPORT]
//     cgap_ingress_bytes — loud, never a fake pass.
//  4. alloc control-> counting counter observes >0 (Go MemStats
//     counting control — the membrane-bounded analog
//     of POSH-5's CountingAlloc).
//
// Anti-vacuity: every arm carries its flip AS the test; an
// always-admit membrane makes arms 1/2 FAIL, not skip.
package ffi

import (
	"encoding/binary"
	"runtime"
	"testing"
)

// mutateStore builds a columnar s|p|o LE-u64 store (72 B, the
// sovereign.rs store_columns twin layout) from explicit rows, so the
// court controls row order and duplication — the coordinate mutations
// of arm 2 are real data, not bit flips.
func mutateStore(rows [][3]uint64) []byte {
	b := make([]byte, 0, len(rows)*24)
	var tmp [8]byte
	// SLA UNBLOCK (disclose-in-both, lane IPC-2 -> AX-7): the pinned eval
	// payload layout is COLUMNAR s|p|o (sovereign.rs store_columns; the
	// eval body splits payload.len()/24 into s/p/o column slices,
	// sovereign.rs:611+). Row-major emission below yields zero matches —
	// transposed to columnar, byte layout unchanged otherwise.
	for col := 0; col < 3; col++ {
		for _, r := range rows {
			binary.LittleEndian.PutUint64(tmp[:], r[col])
			b = append(b, tmp[:]...)
		}
	}
	return b
}

// SELF-CONTAINED FRAME BUILDER (disclosed divergence, lane AX-7): the
// landed IPC-2 helper wireFrame (alloc_court_test.go:62-81) builds ONLY
// the 128-byte header — it never copies `payload` into frame[128:],
// so the BLAKE3 digest (over the real payload) mismatches the
// all-zero payload bytes and cgap_admit_frame refuses every
// wireFrame-built frame with 0x43470002 (witnessed this session:
// stage-1 penstock::ingress ADMITs the frame; stage-2
// verify_payload returns DigestMismatch on the dumped bytes).
// mutation_court_test.go therefore carries its OWN payload-correct
// builder below and does not call wireFrame. The defect in the landed
// court is disclosed to the owning lane in lane-ax-07-mutation.md.
func ax7WireFrame(payload []byte) []byte {
	frame := make([]byte, 128+len(payload))
	copy(frame[0:4], "CGAP")
	frame[4] = 1    // FRAME_VERSION (A04 §6)
	frame[5] = 0x04 // FLAG_BN3
	binary.LittleEndian.PutUint64(frame[8:16], uint64(len(payload)))
	d := Hash(payload)
	copy(frame[16:48], d[:])
	// A17 §2.1 attestation offsets 48..128: ROOT, max ceiling.
	copy(frame[48:52], "P28S")
	binary.LittleEndian.PutUint16(frame[52:54], 1)
	binary.LittleEndian.PutUint16(frame[54:56], 0x01) // LICENSE_FLAG_ROOT
	binary.LittleEndian.PutUint64(frame[56:64], 5_000_000_000)
	o := Hash([]byte(urnLicenseP28))
	copy(frame[64:96], o[:])
	copy(frame[128:], payload) // THE FIX: payload bytes on the wire
	return frame
}

// evalStore admits payload and evals the pinned one-rule network
// (alloc_court_test.go twins); returns status + projected lanes.
func evalStore(t *testing.T, payload []byte, net []byte) (uint32, []int32) {
	t.Helper()
	frame := ax7WireFrame(payload)
	var view View
	st := AdmitFrame(frame, &view)
	if st != StatusOK {
		return st, nil
	}
	lanes := make([]int32, 8)
	var count SizeT
	if st := EvalBn3(&view, net, lanes, &count); st != StatusOK {
		return st, nil
	}
	return StatusOK, lanes[:int(count)]
}

// TestArm1MagicFlipRefusesTyped: BEFORE a clean frame admits; AFTER
// the magic flip CGAP->XGAP refuses with the pinned typed status
// 0x43470001 INVALID_MAGIC (include/cgap.h status table). An
// always-admit membrane fails this court.
func TestArm1MagicFlipRefusesTyped(t *testing.T) {
	frame := ax7WireFrame(storePayload())
	var view View
	if st := AdmitFrame(frame, &view); st != StatusOK {
		t.Fatalf("BEFORE failed: clean frame refused, status %#x", st)
	}
	m := make([]byte, len(frame))
	copy(m, frame)
	copy(m[0:4], "XGAP")
	st := AdmitFrame(m, nil)
	if st != StatusInvalidMagic {
		t.Fatalf("CANNOT FLIP: XGAP magic status=%#x, want 0x43470001 INVALID_MAGIC -- court FAIL", st)
	}
}

// TestArm2CoordinateMutationResultDiffers: duplicate / out-of-order
// fact coordinates through the REAL eval path change the projected
// result. BEFORE: pinned three-row store evals lanes [1 5]. AFTER
// (duplicate): (1,2,100) twice evals lanes [1 1]. AFTER
// (out-of-order): rows 0/1 swapped evals lanes [5 1]. ALL THREE
// results are witnessed and pairwise different — a mutation-blind eval
// (same lanes for every store) fails this court.
func TestArm2CoordinateMutationResultDiffers(t *testing.T) {
	net := oneRuleNetwork()

	clean := mutateStore([][3]uint64{{1, 2, 100}, {5, 2, 100}, {9, 3, 250}})
	st, lanes := evalStore(t, clean, net)
	if st != StatusOK {
		t.Fatalf("BEFORE failed: clean store refused, status %#x", st)
	}
	t.Logf("witness clean store lanes = %v", lanes)
	if len(lanes) != 2 || lanes[0] != 1 || lanes[1] != 5 {
		t.Fatalf("BEFORE failed: clean lanes %v, want [1 5]", lanes)
	}

	dup := mutateStore([][3]uint64{{1, 2, 100}, {1, 2, 100}, {9, 3, 250}})
	stD, lanesD := evalStore(t, dup, net)
	if stD != StatusOK {
		t.Fatalf("AFTER duplicate refused, status %#x", stD)
	}
	t.Logf("witness duplicate-coordinate store lanes = %v", lanesD)
	if len(lanesD) != 2 || lanesD[0] != 1 || lanesD[1] != 1 {
		t.Fatalf("CANNOT FLIP: duplicate store lanes %v, want [1 1] -- court FAIL", lanesD)
	}

	ooo := mutateStore([][3]uint64{{5, 2, 100}, {1, 2, 100}, {9, 3, 250}})
	stO, lanesO := evalStore(t, ooo, net)
	if stO != StatusOK {
		t.Fatalf("AFTER out-of-order refused, status %#x", stO)
	}
	t.Logf("witness out-of-order store lanes = %v", lanesO)
	if len(lanesO) != 2 || lanesO[0] != 5 || lanesO[1] != 1 {
		t.Fatalf("CANNOT FLIP: out-of-order store lanes %v, want [5 1] -- court FAIL", lanesO)
	}

	if sameLanes(lanes, lanesD) || sameLanes(lanes, lanesO) || sameLanes(lanesD, lanesO) {
		t.Fatalf("CANNOT FLIP: results not pairwise distinct: %v / %v / %v", lanes, lanesD, lanesO)
	}
}

func sameLanes(a, b []int32) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

// TestArm3OverflowTypedAtDerivedBound: POSH-5 arm 3 port. The bound is
// DERIVED, never magic: INGRESS_RING_BYTES=32768, FACT_STRIDE=24
// (3 x u64), floor = 1365 facts inline; fact 1366 (32784 > 32768)
// refuses typed IngressError::Overflow (andon exit class 101).
//
// [SKIP-MISSING-EXPORT] LAW: the C-ABI has NO ingress symbol —
// cgap_ingress_bytes is absent from IPC-0's nm-witnessed 11-symbol
// table (lane-ipc-00-ground-truth.md W5, "batch ingress entry point
// MISSING at C-ABI"). An arm that cannot flip because the FFI lacks
// the error surface SKIPS LOUD with the missing symbol named; it is
// never faked as a pass.
func TestArm3OverflowTypedAtDerivedBound(t *testing.T) {
	const (
		ingressRingBytes = 32_768 // cgap-git ingress_ring::INGRESS_RING_BYTES
		factStride       = 3 * 8  // one (u64,u64,u64) fact record
		maxInlineFacts   = ingressRingBytes / factStride
	)
	if maxInlineFacts != 1_365 {
		t.Fatalf("derived bound drift: %d, want 1365", maxInlineFacts)
	}
	// Loud skip, never a fake pass: named missing symbol, cited table.
	t.Skipf("[SKIP-MISSING-EXPORT] cgap_ingress_bytes -- no ingress symbol at C-ABI (IPC-0 W5 table); overflow arm cannot cross the membrane. Port stands at crates/cgap-git/tests/posh_refusal_matrix_court.rs arm3 (witnessed).")
}

// TestArm4AllocPositiveControl: POSH-5 arm 4 port — a real counting
// counter must observe >0 for a genuinely allocating operation. The
// membrane-bounded analog of CountingAlloc is Go's own malloc counter
// (runtime.MemStats.Mallocs, not a mock): BEFORE/AFTER around a real
// retained allocation.
//
// NOISE LAW (witnessed flake this session: a bare no-op pair moved the
// counter by 7 from GC/runtime background): single-pair deltas are
// under the noise floor, so the signal is LOOP-AMPLIFIED (N real
// allocations vs N no-op rounds) and the court asserts the ALLOC
// delta strictly exceeds the observed NO-OP floor per paired round.
// Ratio-style fence, not an absolute ms/alloc bound.
func TestArm4AllocPositiveControl(t *testing.T) {
	sample := func() uint64 {
		var ms runtime.MemStats
		runtime.ReadMemStats(&ms)
		return ms.Mallocs
	}

	const rounds = 8
	var allocTotal, noopTotal uint64
	sink := 0
	_ = sink
	var retained []uint64 // ESCAPE: the buffers must reach the heap (a
	// non-escaping make([]uint64,16) is stack-allocated and the counter
	// rightly reports 0 — witnessed this session, DEAD INSTRUMENT fail).
	for i := 0; i < rounds; i++ {
		a0 := sample()
		for j := 0; j < 1000; j++ {
			buf := make([]uint64, 16)
			buf[0] = uint64(j)
			retained = buf // escapes: heap allocation every iteration
			_ = retained
		}
		a1 := sample()
		if a1 <= a0 {
			t.Fatalf("DEAD INSTRUMENT: alloc round %d observed %d -> %d", i, a0, a1)
		}
		allocTotal += a1 - a0

		n0 := sample()
		for j := 0; j < 1000; j++ {
			sink += j
		}
		n1 := sample()
		if n1 < n0 {
			t.Fatalf("counter went backwards %d -> %d", n0, n1)
		}
		noopTotal += n1 - n0
	}
	_ = sink
	if len(retained) != 16 || retained[0] != 999 {
		t.Fatalf("retained buffer lost: len %d, retained[0] %d (final-state law)", len(retained), retained[0])
	}
	if allocTotal <= noopTotal {
		t.Fatalf("CANNOT FLIP: alloc signal %d does not exceed no-op noise floor %d over %d paired rounds", allocTotal, noopTotal, rounds)
	}
	t.Logf("alloc counter witnessed: alloc signal %d > no-op floor %d mallocs over %d x1000-op paired rounds", allocTotal, noopTotal, rounds)
}
