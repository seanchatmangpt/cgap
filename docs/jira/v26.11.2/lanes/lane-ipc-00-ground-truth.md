# Lane IPC-0 — Go FFI ground truth (GATE lane)

Standing: ALIVE (every claim below backed by a command+exit run this session).

## W1 Commands + exits

| cmd | exit |
|---|---|
| `go version` | 0 → go1.23.4 darwin/arm64 |
| `CARGO_TARGET_DIR=/tmp/ipc0-target cargo build --release -p cgap-ffi` | 0 (Finished release in 2.75s) |
| `nm -gU /tmp/ipc0-target/release/libcgap_ffi.dylib \| awk …` | 0 |

## W2 Substrate

- Go: **go1.23.4 darwin/arm64** — present, no install step needed. GO on toolchain.
- Artifact: `cargo build --release -p cgap-ffi` produces **libcgap_ffi.dylib**
  (cdylib, 615,552 B) per `crates/cgap-ffi/Cargo.toml` `crate-type = ["cdylib", "rlib"]`
  (no staticlib). Build root lease `/tmp/ipc0-target` deleted at lane end.
- Header: `include/cgap.h` freezes four sovereign entrypoints; §2 owner table
  landed (crates/cgap-ffi/src/sovereign.rs:293/413/534/698).

## W3 Real export table (nm -gU on the built dylib, 11 symbols)

| symbol | source | notes |
|---|---|---|
| cgap_admit_frame | sovereign.rs:413 | frame → CgapFrameView, zero-copy |
| cgap_eval_bn3 | sovereign.rs:534 | .bn3 eval → int32 lanes |
| cgap_commit_parquet | sovereign.rs:698 | lanes → Parquet file (host) |
| cgap_verify_p28 | sovereign.rs:293 | P28 spoliation verify, read-only |
| cgap_i32_batch | lib.rs:45 | ArrowArray slot batch (frozen slot surface) |
| cgap_schema_addr / cgap_array_addr | lib.rs:66/73 | slot address probes |
| cgap_batch_rows | lib.rs:80 | row count probe |
| cgap_version | lib.rs:87 | version string |
| cgap_digest_of | lib.rs:132 | BLAKE3 digest of exported slot |
| cgap_p28_invariant_check | cgap-p28 trap.rs:82 (rides the link) | P28 invariant check export |

## W4 Charter-call inventory: real vs missing (C-ABI surface)

FINDINGS (loud, not stubbed — each missing symbol is a cgap-ffi roadmap item):

| charter call | status | evidence |
|---|---|---|
| batch ingress entry point | **MISSING at C-ABI** | Rust `pub fn ingress` exists (cgap-gate/src/penstock.rs:513) + `prefilter8` (penstock.rs:674); NO `#[no_mangle]` wrapper, NO `cgap_ingress*` symbol in the dylib |
| radix join | **MISSING everywhere** | zero `radix` hits in cgap-gate/cgap-bn3 FFI paths; only `from_str_radix` parses in penstock.rs:1639 and unrelated cgap-git uses; no join primitive in any lane crate |
| penstock filter (C-ABI) | **MISSING at C-ABI** | `prefilter8` is Rust-only; no exported symbol |
| zero-alloc query | **PARTIAL** | `cgap_eval_bn3` is the zero-copy eval path (header lines 130-137: callee never allocates in admit/eval/verify); but no generic query/join/filter query entrypoint exists |
| four sovereign entrypoints | REAL, all exported | nm confirms admit/eval/commit/verify all present |

## W5 GO/NO-GO per charter suite

| suite | verdict | needed symbols | missing |
|---|---|---|---|
| alloc CTQ-01 | **GO** | zero-alloc assert over admit/eval/verify | none — real alloc courts already exist (tests/alloc_courts.rs, cgap-audit CountingAlloc) |
| ingress latency CTQ-02 | **NO-GO** | `cgap_ingress_bytes(bytes,len,…) → view/lanes` wrapping penstock::ingress + prefilter8 | `cgap_ingress_bytes` (wrapper absent; the Rust path is real and fast, only the membrane is missing) |
| mutation matrix CTQ-04 | **NO-GO** | mutation driver needs an ingress/join surface to mutate; today only admit/eval/commit/verify | same as CTQ-02 plus `cgap_radix_join` (no radix join exists at ANY layer) |
| shm transport | **NO-GO** | shared-memory ring/zero-copy slot transport over the Arrow slot surface | no shm symbol set at all (`cgap_shm_*` absent); frozen Arrow slot surface (cgap_i32_batch/addresses) is the nearest substrate |

## W6 Fences

Diff is docs-only (one new receipt file); no gates/courts touched. Standing
fences cross-referenced: zero_string_gate.py, boundary_gate.py untouched.

## W7 Standing

ALIVE for every claim above (commands + exits this session). Verdict:
**GO overall with 3 suite-level NO-GOs** — toolchain present, real 11-symbol
C-ABI built and measured, but the Go-side charter suites CTQ-02/CTQ-04/shm
have NO entrypoints to bind; the missing symbols above are the cgap-ffi
roadmap, not stubs.
