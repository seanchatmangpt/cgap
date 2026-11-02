# Lane IPC-2 — Go cgo membrane + alloc-purity courts

Standing: **ALIVE** (every claim below backed by a command + exit run this session).
Receipt: `docs/jira/v26.11.2/lanes/lane-ipc-02-alloc-purity.md`.

## W1 — Commands + exits

| cmd | exit | witness |
|---|---|---|
| `go version` | 0 | go1.23.4 darwin/arm64 |
| `CARGO_TARGET_DIR=/tmp/ipc2-target cargo build --release -p cgap-ffi` | 0 | libcgap_ffi.dylib 615,552 B, Finished 3.09s |
| `nm -gU /tmp/ipc2-target/release/libcgap_ffi.dylib` | 0 | exactly the IPC-0 11-symbol table |
| `CGO_ENABLED=1 go vet ./ffi` (cwd `courts/interop-chicago`) | 0 | clean |
| `CGO_ENABLED=1 go test -count=1 ./ffi` | 0 | ok 2.299s |
| `-v` suite census | 0 | **10 PASS / 1 loud SKIP / 0 FAIL** |
| `CGO_ENABLED=1 go build -gcflags="-m -m" ./ffi` and `go test -gcflags="-m -m"` | 0 | esc trace, see W5 |
| Rust probe `cargo run` (/tmp/ipc2-blake3-probe) | 0 | byte-identity + cross-admit witness, see W4 |

## W2 — Substrate + bound symbols

- Binds ONLY IPC-0-verified symbols (nm -gU re-witnessed this session):
  `cgap_admit_frame`, `cgap_eval_bn3`, `cgap_version`, `cgap_digest_of`
  (+ frozen-slot probes `cgap_i32_batch`/`cgap_array_addr`). Signatures
  cited from `include/cgap.h:146/153` and
  `crates/cgap-ffi/src/{lib,sovereign}.rs` — cited, not redefined.
- cgo preamble declares the LP64 `cgap_frame_view` layout twin
  (include/cgap.h:77-82, 24 B/8-aligned); the Go `View` type is the
  unsafe-cast twin, asserted via the view positive control
  (payload pointer lands at base+128 — the zero-copy law across the
  membrane, witnessed in TestAdmitPositiveControl).
- BLAKE3 in Go (`blake3.go`) is a 1:1 port of the repo canon
  `crates/cgap-blake3/src/lib.rs` (compress_scalar + hash_chunk +
  stack Merkle, ROOT folded into the last-block compress). Official
  empty-input vector witnessed passing: af1349b9f5f9a1a6...41f3262.
- End-to-end digest parity: the Go-built frame is byte-identical to the
  Rust `wire_frame` twin output (compared via /tmp probe, first diff
  NONE over all 200 bytes), and the REAL Rust admit path accepts it
  (probe: `rust admit(built) = 0x0`).

## W3 — Courts (all real collaborators, final-state asserts)

| court | asserts | anti-vacuity |
|---|---|---|
| TestBLAKE3OfficialEmptyVector | official known-answer | wrong hash fails the vector |
| TestAdmitPositiveControl | GO frame admits (0x0); view.payload == base+128 (zero-copy); corrupt-digest flips to typed 0x43470002 (witnessed refusal logged) | digest byte flip |
| TestEvalPositiveControl | admit+eval = lanes [1 5], count 2, deterministic 2nd crossing | corrupt store/payload would change lanes (AX-7 arm 2 flips jointly) |
| TestSteadyStateAllocPurity | steady admit+eval loop @ AllocsPerRun(1_000_000) = 1.0 allocs/op, lanes [1 5] | any drift flips the pinned 1.0 |
| TestMembraneTaxDecomposition | view-mint admit = 1.0; null-view admit / eval / pinner / version = 0.0 each | per-component drift flips |
| TestVersionAllocPurity | version membrane read = 0.0 allocs/op | drift flips |
| TestAntiVacuityFlip | escaping closure = 1 allocs/op; no-op = 0.0 (both directions witnessed) | the flip IS the court |
| AX-7 arms 1/2/4 (shared suite) | magic flip -> typed INVALID_MAGIC; coordinate mutations change lanes; MemStats alloc control > 0 | each arm is its own flip |

## W4 — cgo-tax measurement (HONEST FINDING, not a masked 0.0)

Measured with `testing.AllocsPerRun(1_000_000, ...)` over the steady
admit->eval loop:

- **Steady-state admit+eval: 1.0 allocs/op, 24 B/op** — the per-op Go
  `View` mint (`var view View`) escapes to the heap because its address
  crosses the cgo boundary (esc trace: `view escapes to heap` at
  `ffi/alloc_court_test.go:110/140`). This is the zero-copy view
  protocol's real Go-side membrane tax. Per the dispatch honesty clause
  it is RECORDED, not masked by hoisting the view out of the loop
  (hoisting would measure a view-reuse protocol, not the steady state).
- Components at **0.0 allocs/op** each: null-view admit, eval crossing
  (including runtime.Pinner pin/unpin), runtime.Pinner alone, version
  read, no-op closure.
- The Rust-side eval path allocates Vecs through the system allocator
  (rules/columns, sovereign.rs:585+) — a further membrane tax invisible
  to AllocsPerRun; recorded, unmeasured here (Rust courts own that
  surface: tests/alloc_courts.rs).
- Esc trace (`go build/test -gcflags="-m -m"`): `_cgo_gotypes.go` p0/p1/
  p2/p4 escape to heap (the cgo arg pins); `view escapes to heap`
  (alloc_court_test.go:110/140); PinFrame/UnpinView/deferwrap inline
  cleanly. Recorded in-receipt as the esc record.

## W5 — Anti-vacuity

- Flip witnessed: escaping closure measured **1 allocs/op** (pinned by
  fresh heap target per op; an append-only closure was DEAD INSTRUMENT
  — AllocsPerRun warms up once, absorbing the first growth; the
  make-per-op form flips reliably). No-op closure = 0.0.
- Corrupt-digest admit flips to typed `0x43470002` (witnessed in-test
  log line: `corrupt-digest typed refusal status: 0x43470002`).
- A vacuous court is FAIL by law; every court above flips or jointly
  flips with AX-7's arms.
- Arm 3 skip is AX-7's loud [SKIP-MISSING-EXPORT] (cgap_ingress_bytes
  absent at C-ABI — IPC-0 W5 NO-GO table), never a fake pass.

## W6 — Fences + disclosures

- Fences: zero_string_gate.py / boundary_gate.py untouched; shared
  compile stayed green throughout (SLA unblocks applied within minutes).
- DISCLOSED SLA UNBLOCKS (disclose-in-both law):
  1. **IPC-2 -> AX-7** in `ffi/mutation_court_test.go`: (a) mutateStore
  emitted ROW-major bytes; the pinned eval payload layout is COLUMNAR
  s|p|o (sovereign.rs store_columns; eval splits len/24 into columns,
  sovereign.rs:611+) — transposed to columnar; (b) unused `retained`
  variable broke the shared compile — `_ = retained` added. AX-7's
  receipt must cite this receipt; AX-7 supersedes on landing.
  2. **AX-7 -> IPC-2** (their disclosed notes in my files): package-level
  `sink` var note (alloc_court_test.go:232 comment, lane AX-4 note per
  their text) and their self-contained frame builder workaround for the
  wireFrame payload-copy bug (now fixed at source in W4/W3 — their
  builder remains, harmless).
  3. AX-7 is concurrently editing mutation_court_test.go /
  alloc_linux_test.go in the same directory; my edit touches only the
  columnar fix + `_ = retained`; owner supersedes on landing.
- Divergences: (a) view.Payload in the Go `View` type is
  `*C.uint8_t`; cgo pointer law required runtime.Pinner pinning per
  crossing (AdmitFrame/EvalBn3 pin, unpin on defer) — added cgo-law
  surface the header does not name. (b) Build-root lease
  `/tmp/ipc2-target` is NOT deleted at lane end: the test binary links
  -L/tmp/ipc2-target/release with rpath — deletion breaks replay.
  Coordinator deletes at integration; replay command documented in W1.
- No mocks anywhere: real dylib, real cgo, real Rust admit path.

## W7 — Standing

ALIVE for every claim above (commands + exits this session). 10 PASS /
1 loud SKIP / 0 FAIL; go vet clean; esc trace recorded; the honest
steady-state membrane tax is 1.0 allocs/op (view mint), components 0.0.
