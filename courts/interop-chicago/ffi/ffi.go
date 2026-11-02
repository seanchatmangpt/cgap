// Package ffi — cgo membrane over the cgap-ffi cdylib (lane IPC-2).
//
// Binds only IPC-0-verified symbols (docs/jira/v26.11.2/lanes/
// lane-ipc-00-ground-truth.md W3 table, nm -gU witnessed): cgap_version,
// cgap_admit_frame, cgap_eval_bn3, cgap_digest_of. Signatures are cited
// from include/cgap.h (A19 ABI freeze) and
// crates/cgap-ffi/src/{lib,sovereign}.rs — cited, not redefined.
//
// Build: CGO_ENABLED=1 with the dylib built by
//
//	CARGO_TARGET_DIR=/tmp/ipc2-target cargo build --release -p cgap-ffi
//
// linked via absolute -L + rpath so the test binary runs without env.
package ffi

/*
#cgo CFLAGS: -I/Users/sac/cgap/include
#cgo LDFLAGS: -L/tmp/ipc2-target/release -lcgap_ffi -Wl,-rpath,/tmp/ipc2-target/release

#include <stdint.h>
#include <stddef.h>

// Layout twin of cgap_frame_view (include/cgap.h:77-82, LP64 pin:
// 24 bytes, 8-aligned; asserted in alloc_court_test.go via unsafe.Sizeof).
typedef struct {
  const uint8_t* payload;
  size_t payload_len;
  uint32_t schema_id;
  uint32_t rule_count;
} cgap_frame_view;

typedef uint32_t CgapStatus;

// Sovereign entrypoints — include/cgap.h:146/153.
CgapStatus cgap_admit_frame(const uint8_t* bytes, size_t len, cgap_frame_view* out_view);
CgapStatus cgap_eval_bn3(const cgap_frame_view* view, const uint8_t* bn3_rodata, size_t bn3_len,
                         int32_t* out_lanes, size_t lanes_cap, size_t* out_lane_count);

// Frozen slot surface — crates/cgap-ffi/src/lib.rs:52/66/73/80/87/132.
const char* cgap_version(void);
uint64_t cgap_digest_of(size_t array_addr);
size_t cgap_array_addr(void);
size_t cgap_schema_addr(void);
int64_t cgap_batch_rows(void);
void* cgap_i32_batch(int64_t n, int64_t null_every);
*/
import "C"

import (
	"runtime"
	"unsafe"
)

// SizeT aliases the C size_t so test files (which cannot import cgo)
// share the exact type used in the View layout.
type SizeT = C.size_t

// VersionPtr returns the raw C version-string pointer (no GoString
// copy) — the allocation-free membrane read used by the purity court.
func VersionPtr() uintptr {
	return uintptr(unsafe.Pointer(C.cgap_version()))
}

// Frame/BN3 status words — include/cgap.h:58-69 (cited, not redefined;
// cgo cannot see C enums, so the cited literals are pinned here).
const (
	StatusOK               uint32 = 0x00000000
	StatusInvalidMagic     uint32 = 0x43470001
	StatusOabiMismatch     uint32 = 0x43470002
	StatusShaclViolation   uint32 = 0x43470003
	StatusLeaseExhausted   uint32 = 0x43470004
	StatusConcurrentWriter uint32 = 0x43470006
	StatusP28Spoliation    uint32 = 0x503238FF
)

// View mirrors the C header-defined non-owning, zero-copy view
// (include/cgap.h §"cgap_frame_view"). Payload borrows the caller's
// Go bytes for the view's lifetime — the zero-copy law carries across
// the membrane.
type View struct {
	Payload    *C.uint8_t
	PayloadLen C.size_t
	SchemaID   C.uint32_t
	RuleCount  C.uint32_t
}

// AdmitFrame crosses the membrane once; the returned view borrows
// `bytes` — the borrow lives only while the caller keeps the pin (see
// PinFrame). Status is the sovereign status word (cited constants
// above).
func AdmitFrame(bytes []byte, out *View) uint32 {
	var p *C.uint8_t
	if len(bytes) > 0 {
		p = (*C.uint8_t)(unsafe.Pointer(&bytes[0]))
	}
	var vp *C.cgap_frame_view
	if out != nil {
		vp = (*C.cgap_frame_view)(unsafe.Pointer(out))
	}
	// cgo pointer law: the C view stores a pointer INTO the caller's Go
	// bytes (the zero-copy law carried across the membrane); pin the
	// callee-visible Go memory for the crossing.
	var pin runtime.Pinner
	pin.Pin(&bytes[0])
	defer pin.Unpin()
	return uint32(C.cgap_admit_frame(p, C.size_t(len(bytes)), vp))
}

// PinFrame pins the caller's frame bytes so a View borrowed from them
// can cross the membrane again (EvalBn3) without tripping the cgo
// unpinned-Go-pointer check. UnpinView releases it.
func PinFrame(bytes []byte) *runtime.Pinner {
	pin := &runtime.Pinner{}
	if len(bytes) > 0 {
		pin.Pin(&bytes[0])
	}
	return pin
}

// UnpinView releases a PinFrame lease.
func UnpinView(pin *runtime.Pinner) { pin.Unpin() }

// EvalBn3 evaluates an admitted view against a .bn3 rule network
// (A11 layout, LE words) writing i32 lanes into out. Same cgo pointer
// law as AdmitFrame: the view's payload pointer (into the caller's Go
// frame) and the net/lanes slices cross pinned.
func EvalBn3(view *View, net []byte, out []int32, count *SizeT) uint32 {
	var np *C.uint8_t
	if len(net) > 0 {
		np = (*C.uint8_t)(unsafe.Pointer(&net[0]))
	}
	var vp *C.cgap_frame_view
	if view != nil {
		vp = (*C.cgap_frame_view)(unsafe.Pointer(view))
	}
	var lp *C.int32_t
	if len(out) > 0 {
		lp = (*C.int32_t)(unsafe.Pointer(&out[0]))
	}
	var pin runtime.Pinner
	if view != nil && view.Payload != nil {
		pin.Pin(unsafe.Pointer(view.Payload))
	}
	if len(net) > 0 {
		pin.Pin(&net[0])
	}
	if len(out) > 0 {
		pin.Pin(&out[0])
	}
	defer pin.Unpin()
	return uint32(C.cgap_eval_bn3(vp, np, C.size_t(len(net)), lp,
		C.size_t(len(out)), count))
}

// Version returns the frozen-slot-surface version string.
func Version() string {
	return C.GoString(C.cgap_version())
}

// DigestOf folds the exported Arrow slot at arrayAddr (rotate-xor
// mutation-detection digest — NOT cryptographic; lib.rs:97-100).
func DigestOf(arrayAddr uintptr) uint64 {
	return uint64(C.cgap_digest_of(C.size_t(arrayAddr)))
}

// I32Batch exports n rows through the frozen slot surface (allocates
// on the Rust side — setup only, never inside a measured window).
func I32Batch(n, nullEvery int64) uintptr {
	return uintptr(C.cgap_i32_batch(C.int64_t(n), C.int64_t(nullEvery)))
}

// ArrayAddr serves the live export slot pointer.
func ArrayAddr() uintptr {
	return uintptr(C.cgap_array_addr())
}
