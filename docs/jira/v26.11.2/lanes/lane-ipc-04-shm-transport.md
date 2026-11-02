# Lane IPC-4 — Go-side SHM transport court (RE-SCOPED per IPC-0)

Standing: ALIVE (every claim backed by a command+exit run this session,
2026-11-02).

## W0 Re-scope (loud)

IPC-0 (`lane-ipc-00-ground-truth.md` W5) returned NO-GO on the kernel-bind
arm: no `cgap_shm_*` symbols exist in libcgap_ffi.dylib. Per dispatch, this
lane is TRANSPORT-ONLY on the Go side: no cgap kernel involvement. The
frozen Arrow slot surface (`cgap_i32_batch` / `cgap_schema_addr` /
`cgap_array_addr`) is RECORDED as the nearest substrate and NOT bound — the
kernel-call path is the cgap-ffi FFI roadmap, not this court.

## W1 Commands + exits

| cmd | exit |
|---|---|
| `go version` | 0 → go1.23.4 darwin/arm64 |
| `go vet ./shm/` | 0 |
| `go test ./shm/ -v` | 0 (4/4 PASS) |
| `CGO_ENABLED=0 go test ./shm/ -count=1` | 0 (pure-Go, zero cgo proven) |
| `go test ./shm/ -bench . -benchmem -count=3 -run xxx` | 0 |

## W2 Mechanism (named)

**mmap'd temp file**: `os.CreateTemp` + `syscall.Mmap` MAP_SHARED — pure
stdlib, **zero cgo** (proven by the `CGO_ENABLED=0` run, exit 0). POSIX
`shm_open` via cgo was rejected: /dev/shm is absent on macOS (dispatch
fence) and cgo would add toolchain + alloc cost for the same kernel
primitive (mmap of a shared backing is what shm_open wraps; delta is
naming, not semantics).

Wire: 32-byte LE header (magic `0x4347415053484D31` "CGAPSHM1", tx_id u64,
fact_count u32, stride u32) + facts at stride 24 B (corpus.LE format).
Producer maps RW, consumer maps the SAME file via a second independent
PROT_READ mapping.

## W3 Courts (4, all PASS)

1. `TestTransportRoundtrip` — producer writes the canonical 1,365-fact batch
   (FACT_WINDOW provenance: HONEST NONE — named coprime stride,
   pack_demux.rs:43-51, per corpus/gen.go); consumer reads via an independent
   second mapping; byte-exact vs `corpus.Encode(batch)` + per-fact equality.
2. `TestMagicCorruptionFlip` — **ANTI-VACUITY**: corrupted header magic in
   the mapped region → reader halts with typed `ErrMagic`; flip proven
   both directions (baseline pass → corrupt → ErrMagic → restore → pass).
3. `TestZeroAllocReadPath` — `testing.AllocsPerRun` over the validated
   zero-copy read: **0 allocs/run** (no heap reassembly; cgo absent).
4. `TestFileReadBaselineRoundtrip` — honest fallback arm: pread (no mmap)
   reads byte-exact too; delta is cost, not correctness.

## W4 Cost table (Apple M3 Max, darwin/arm64, go1.23.4, -count=3)

| arm | ns/op | B/op | allocs/op |
|---|---|---|---|
| mmap zero-copy read (mapped region view) | 4.52–4.54 | 0 | 0 |
| pread fallback (one syscall per read) | 1377–1405 | 0 | 0 |

Measured method: `go test -bench -benchmem -count=3`, min-of-3 reported as a
range. mmap read is ~310x cheaper per read than the pread arm (one memcpy
into a kernel buffer per pread vs none through the mapping). Perf numbers
above are measured this session with the stated method on this host; not
generalized beyond darwin/arm64.

## W5 Fences

Diff is additive in `courts/interop-chicago/shm/` (new package + tests only;
`go.mod`/`corpus` untouched). Fences that could regress: none touched —
zero_string_gate.py, boundary_gate.py, cargo courts all outside this diff.
The interop module stays stdlib-only (publish-frozen law) — verified by the
CGO_ENABLED=0 run.

## W6 Divergences

- RE-SCOPED per IPC-0 as instructed: kernel-bind arm absent, recorded not
  stubbed.
- The zero-copy view uses `unsafe.Slice` reinterpretation (pure Go unsafe,
  no cgo) — documented in unsafe_slice.go; retention past Munmap is the
  caller's contract (godoc'd on ReadZeroCopy).
- No [SKIP] arms: mmap proved practical bounded-time; fallback arm run as a
  measured comparison, not a skip.

## W7 Standing

ALIVE for all claims above. IPC-0's shm NO-GO stands for the KERNEL arm;
the transport arm it gated is now ALIVE on the Go side and awaits the
`cgap_shm_*` membrane to bind.
