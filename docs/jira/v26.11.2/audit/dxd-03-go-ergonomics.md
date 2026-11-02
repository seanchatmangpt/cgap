# dxd-03-go-ergonomics

DXD-03 Go ergonomics audit (2026-11-02). go1.23.4 darwin/arm64, M3 Max.
Module root /Users/sac/cgap/courts/interop-chicago. All exits real, this
session. Findings reflect the tree as of IPC-2's ffi/ landing (14:42).

## (a) Build the dylib

cgo directives hard-code the output path — the cargo invocation is
load-bearing and exists NOWHERE in the module (only an ffi.go comment):

    CARGO_TARGET_DIR=/tmp/ipc2-target cargo build --release -p cgap-ffi
    # /tmp/ipc2-target/release/libcgap_ffi.dylib

Second binding (ingress_court_binding.go) expects a DIFFERENT dir:
#cgo LDFLAGS: -L/tmp/ipc3-target/release -lcgap_ffi -Wl,-rpath,...
Both /tmp dirs currently hold dylibs. Source: crates/cgap-ffi
(crate-type cdylib+rlib).

## (b) go vet — RED today (real finding)

    go vet ./...   # exit 1
    ffi/ffi.go:56:36: could not determine kind of name for
    C.CGAP_ERR_CONCURRENT_WRITER_DENIED (and 6 more)

Cause: ffi.go's cgo preamble declares functions but never #include
<cgap.h>, so the C.* status constants are undefined even though
#cgo CFLAGS -I points at /Users/sac/cgap/include and cgap.h:58-68
defines the enum. One-line fix (add the include). Same class as the
golang/go#8601 workaround debt in ingress_court_binding.go.
IPC-2's file — fix owed by that lane at drain.

## (c) Tests

No DYLD_LIBRARY_PATH / CGO_ENABLED overrides needed (baked absolute
-L/-rpath; darwin default CGO_ENABLED=1). Measured: go test ./shm/ 0;
go test . 0 (links ipc3 dylib); go test ./corpus/ 0 (no test files).
BUT the ingress court silently depends on /tmp/ipc3-fixtures
(frame_24.bin — regenerated via /tmp/ipc3-fxgen, documented only in a
t.Fatalf hint at ingress_court_test.go:46). Missing => FAIL-fast
SKIP-RESCOPED, not a skip. Nothing documents regen.

## (d) Benches

go test -bench works env-free: BenchmarkMmapRead 11692 ns/op,
BenchmarkPreadRead 13838, BenchmarkAdmitBatch1365 252242,
BenchmarkAdmitSingle 304.2 (at -benchtime=10x). No bench runner
script, no benchtime policy in-module.

## (e) Seal a receipt

go run ./cmd/seal_receipt (-in r.json -out r.md): exit 0 emit /
1 self-lint refusal / 2 usage-IO. No example input JSON ships —
first-time users hit 1/2 blind.

## Ergonomics gaps

| # | gap | class |
|---|-----|-------|
| 1 | no `toolchain go1.23.4` pin in go.mod (receipts record it) | one-line |
| 2 | dylib cargo build line lives in a comment; two divergent target dirs | new script |
| 3 | ffi.go preamble missing #include <cgap.h> — vet RED | one-line (IPC-2) |
| 4 | absolute paths x3 (-L/-rpath x2 + -I) bake /tmp and /Users/sac | new script (cgo can't read env; preamble-rewrite script is the only real fix) |
| 5 | /tmp/ipc3-fixtures regen undocumented outside a t.Fatalf | new script |
| 6 | no go:generate — nothing checked in to regenerate | not-worth-it |
| 7 | no example seal_receipt input JSON | one-line |
| 8 | env coupling zero (good) — do NOT add DYLD_LIBRARY_PATH docs; it would mask a broken rpath | record |

## Verified-good

shm/ and corpus/ pure stdlib zero-cgo; benches run plain; sealer
fail-closed at emit (SL-001..003).

## See Also

audit/dxd-01-justfile-census.md · audit/dxd-02-doctor-design.md
