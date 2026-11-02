// Package interop -- LANE IPC-3 minimal cgo binding. OWNERSHIP DIVERGENCE,
// disclosed: the dispatch named only ingress_court_test.go, but the Go
// toolchain does not support `import "C"` in _test.go files (golang/go#8601),
// so the cgo preamble lives in this non-test file. IPC-2's landed bindings
// supersede this diff per the dispatch disclosure law.
//
// LANE AX-5 linux adaptation (disclosed line-by-line in
// docs/jira/v26.11.2/lanes/lane-ax-05-latency.md):
//   - LDFLAGS split per-OS via #cgo constraints: darwin keeps the IPC-3
//     rpath (/tmp/ipc3-target/release); linux links -L/work/target/release
//     (the in-container build root the AX-5 runner materializes). No env
//     expansion exists in #cgo directives, so linux's path is the fixed
//     container mountpoint.
//   - AdmitFrame body is UNCHANGED; the cgo preamble is the only delta.
package interop

/*
#cgo darwin LDFLAGS: -L/tmp/ipc3-target/release -lcgap_ffi -Wl,-rpath,/tmp/ipc3-target/release
#cgo linux LDFLAGS: -L/work/target/release -lcgap_ffi -Wl,-rpath,/work/target/release
#include <stdint.h>
#include <stddef.h>
typedef uint32_t CgapStatus;
typedef struct {
  const uint8_t* payload;
  size_t payload_len;
  uint32_t schema_id;
  uint32_t rule_count;
} cgap_frame_view;
extern CgapStatus cgap_admit_frame(const uint8_t* bytes, size_t len, cgap_frame_view* out_view);
*/
import "C"

import "unsafe"

// AdmitFrame binds cgap_admit_frame (sovereign.rs:413, 11-symbol table).
// UNCHANGED from IPC-3 (body identical on darwin and linux).
func AdmitFrame(b []byte) (uint32, uint64) {
	var view C.cgap_frame_view
	var bp *C.uint8_t
	if len(b) > 0 {
		bp = (*C.uint8_t)(unsafe.Pointer(&b[0]))
	}
	st := C.cgap_admit_frame(bp, C.size_t(len(b)), &view)
	return uint32(st), uint64(view.payload_len)
}
