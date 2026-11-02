# Lane IPC-3 — membrane-latency court (RE-SCOPED per IPC-0 NO-GO)

Standing: ALIVE for every claim below (each backed by a command+exit run
this session, 2026-11-02). Perf numbers are measured THIS-session with the
stated method, on a NON-ISOLATED shared host (B06 disclosure below) —
environment-dependent figures, not portable targets.

## W1 Commands + exits

| cmd | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/ipc3-target cargo build --release -p cgap-ffi` | 0 (Finished release in 2.88s) |
| `cd /tmp/ipc3-fxgen && cargo run --release -- /tmp/ipc3-fixtures` | 0 (wrote 152 B + 207480 B) |
| `cd courts/interop-chicago && go vet ./...` | 0 (after IPC-2 unblock, W5) |
| `go test -run <5 courts> -v .` | 0 (all PASS, W4) |
| `go test -run XXX -bench ... -benchtime 200x -count 5 .` | 0 (W4 table) |

## W2 Substrate

- Fixture generation: /tmp/ipc3-fxgen (Rust, path-deps cgap-core +
  cgap-blake3) — builds REAL A04 frames: 128-byte CgapFrameHeader (magic
  CGAP, version 1, FLAG_BN3, BLAKE3 oabi_digest over the 24 B payload,
  root P28 attestation) + payload; encoded via CgapFrameHeader::encode_into.
  Fixtures: /tmp/ipc3-fixtures/frame_24.bin (152 B), batch_1365.bin
  (1365 x 152 B = 207480 B). Generation was a /tmp command, NOT a repo write.
- Binding: cgo over libcgap_ffi.dylib (cdylib, 615,552 B) —
  cgap_admit_frame only (11-symbol table, sovereign.rs:413).

## W3 Re-scope statement (loud)

The charter's ingress-kernel call does NOT exist as a C symbol
(lane-ipc-00-ground-truth.md W4/W5: NO cgap_ingress_bytes; ingress CTQ-02
NO-GO). This court therefore measures the REAL membrane over an EXPORTED
kernel: **cgap_admit_frame**, disclosed as the cheaper honest representative
(A14 gate + A04 header validation + BLAKE3 re-derivation over a 24 B
payload). cgap_eval_bn3 was NOT measured — it needs a .bn3 rodata network
fixture; deferred with this disclosure, not skipped silently.

## W4 Measured table (Apple M3 Max, shared host, concurrent lane builds)

| measurement | value | method |
|---|---|---|
| timer-resolution witness | 41 ns min positive monotonic delta | paired time.Now, 100k iters |
| empty-path proxy (membrane floor) | **85 ns/call** | 200k calls of cgap_admit_frame(NULL, 0, view) — the null-refusal short-circuit; no empty exported symbol exists in the 11-symbol table, disclosed |
| T_kernel single admit (24 B payload) | **182.7-185.6 ns/call** (min..max, 5 runs @200 iters) | go test -bench BenchmarkAdmitSingle |
| batch-1365 amortized (FACT_WINDOW) | **179-181 ns/frame** (244268-249412 ns / 1365, 5 runs) | go test -bench BenchmarkAdmitBatch1365 |
| batch-1365 independent-runs table | min=183 median=185 max=187 ns/frame (7 runs) | TestBatchStats; **no p99 reported — max-of-N is not a quantile (FIX-04/GAP-03 law)** |

Phase 3 arithmetic against the REAL numbers (not the 6.04 ns fiction): the
cgo membrane floor is 85 ns; kernel-side admit work adds ~95-100 ns on top;
a 1-second budget admits ~5.4M frames/s through this membrane (~7.4M/s of
raw kernel time at 135 ns/frame midpoint if the membrane were free). Any
charter latency budget built on the assumed 6.04 ns must be recomputed
against these figures — it is off by >20x at the membrane floor alone.

## W5 Disclosures (disclose-in-both-directions law)

1. **IPC-2 SLA unblock (BOTH receipts):** lane IPC-2's in-flight
   courts/interop-chicago/ffi/blake3.go froze the module compile
   (`end > len(data)` uint64/int mismatch at blake3.go:170). IPC-3 applied
   the minimal fix (uint64(len(data)) casts, marked with an inline comment
   naming the unblock). IPC-2's landing supersedes the diff. No lane-ipc-02
   receipt was on disk at close to cross-cite; this receipt is the
   unblocker's record per the witnessed pattern.
2. **Ownership divergence:** dispatch named ONLY ingress_court_test.go +
   this receipt, but `import "C"` is unsupported in _test.go files
   (golang/go#8601, witnessed: `use of cgo in test not supported`). A
   minimal non-test binding file was added:
   courts/interop-chicago/ingress_court_binding.go (cgo preamble +
   AdmitFrame over cgap_admit_frame). IPC-2's landed bindings supersede it.
3. **Load-sensitivity (B06 law):** the host ran concurrent lane builds
   (cargo + go); figures are NOT isolated-bench targets. Batch spread
   244-249 us across runs is the load witness.

## W6 Anti-vacuity + fences

- Named mutation: TestMutationGateBreach corrupts bytes in the Go driver —
  magic flip ("CGAP"->"XGAP") MUST return typed 0x43470001 INVALID_MAGIC,
  digest flip MUST NOT return CGAP_OK. Witnessed PASS; an admit that could
  not discriminate would FAIL the court (flip proven by the typed pin).
- TestAdmitValidFrame: real positive control — valid fixture admitted with
  view payload_len == 24 (A04 zero-copy view over the caller's bytes).
- Fences my diff could break: the interop-chicago module compile (`go vet
  ./...` exit 0 re-run after both edits); IPC-2's ffi package (untouched
  beyond the disclosed cast fix); no repo gates touched (boundary_gate.py /
  zero_string_gate.py unmodified). `go build ./...` on the module remains
  green via the same vet run.

## W7 Standing

ALIVE — courts ran this session against the exact dylib; every figure in W4
is a fresh measurement, not a citation. Lease cleanup: /tmp/ipc3-fxgen-target
deleted; /tmp/ipc3-target is KEPT deliberately — the landed binding file
links -lcgap_ffi against it; rebuild command if purged:
`CARGO_TARGET_DIR=/tmp/ipc3-target cargo build --release -p cgap-ffi`.
Fixture files /tmp/ipc3-fixtures + /tmp/ipc3-fxgen persist for court reruns
(regenerate command in W1).

## Files

- /Users/sac/cgap/courts/interop-chicago/ingress_court_test.go (dispatched)
- /Users/sac/cgap/courts/interop-chicago/ingress_court_binding.go (ownership
  divergence, disclosed W5.2)
- docs/jira/v26.11.2/lanes/lane-ipc-03-latency-court.md (this receipt)
