//go:build linux
// +build linux

// linux_cgo.go — Lane AX-4 linux-arm64 adaptation of the IPC-2 ffi
// alloc-purity courts. This file is additive to the darwin court: it
// carries ONLY the container-side cgo link flags so the same darwin
// test files run unchanged inside the container against the dylib
// served at /opt/cgap/lib (AX-3 dylib path convention).
//
// Every line disclosed in lane-ax-04-alloc.md. The darwin #cgo lines in
// ffi.go (lines 17-18) still apply on darwin only — they are inert on
// linux because these #cgo linux constraints scope to linux; the
// missing -I/-L directories they name resolve to no-ops at gcc/link
// time on linux.
package ffi

/*
#cgo linux CFLAGS: -I/opt/cgap/include
#cgo linux LDFLAGS: -L/opt/cgap/lib -lcgap_ffi -Wl,-rpath,/opt/cgap/lib
*/
import "C"
