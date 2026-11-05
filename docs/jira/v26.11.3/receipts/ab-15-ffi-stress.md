# AB-15 — FFI Stress Receipt (Rust + Python, one receipt, both sides)

Lane: STRESS AB-15 (subject read-only; the lane owns only the files listed
under "Landed"). Date: 2026-11-03, session 21:40-22:20 PST. Repo:
/Users/sac/cgap. No git commands run.

## Landed

| File | What |
|---|---|
| `crates/cgap-ffi/tests/ffi_stress.rs` (new) | bench, slot-churn leak witness, status sweep |
| `bindings/python/avec/tests/test_ffi_stress.py` (new) | round-trip, NUL court, C-Data churn |
| `crates/cgap-ffi/Cargo.toml` (additive edit) | 2 dev-deps, disclosed below |

## Disclosed manifest touch (coordinator-owned file)

`crates/cgap-ffi/Cargo.toml` gained two **dev-dependencies only**
(`cgap-core`, `cgap-blake3`, both path deps at workspace version). Why:
integration tests cannot see regular dependencies, and the real admit-path
bench requires canon frame construction (`CgapFrameHeader::encode_into` +
`cgap_blake3::hash`, the crates/cgap-core/tests/common canon). Precedent:
the disclosed manifest cross-touch pattern in the same manifest (Wave-F2
cgap-p28 dep, GAP-1 block). Zero effect on the published dep graph (dev-deps
are publish-neutral). Revert = delete the two lines.

## Rust side (ALIVE: ran this session, outputs cited)

Gate: PARTIAL
(exit 0), re-run green at 22:16 after the last crypto freeze cleared.
Full integration sweep at 22:16 (`cargo test -p cgap-ffi` over ALL 7
--test binaries): 28 passed, 0 failed — ffi_stress 4, alloc_courts 4,
zero_copy 5, c_header_abi_court 4, arrow_offset_pins 4, lever_courts 2,
stats_block_court 5.

### Leg 1 — call-overhead bench (min-of-3, 100k calls each)

Valid frame: 128 B canon header + 64 B payload, real BLAKE3 digest, Root
attestation bound to the pinned instrument URN — the byte-identical replica
of the `wave_gap1_courts` builder, so the bench exercises the SAME admit path
the courts pin (aligned projection path, 64-byte-aligned staging). Status
assert rides every call.

- round 0: 3535.1 ns/call; round 1: 3460.6 ns/call; round 2: 3419.5 ns/call
- **ADMIT-OVERHEAD: 3419.5 ns/call min-of-3** (debug/test profile; a
  measured debug-build number, not a release claim)
- status assert rides every measured call (disclosed in-loop cost)

### Leg 2 — slot-churn + leak witness

10,000 `cgap_i32_batch` slot set/get cycles under `with_alloc_tracking`
(cgap-audit `CountingAlloc`, real global allocator of the test binary):

- AtomicPtr slot ordering holds: fresh live pointer every cycle, `array_addr`
  serves exactly the exported pointer, rows slot serves 64 — 10k/10k cycles.
- Per-call allocation is EXACTLY constant: total == single-call baseline
  × 10,000 (16 allocs / 1,184 B per call, counts and bytes both exact).
  No growth, no drift = no accumulating leak. The per-call constant includes
  the two `Box::leak` slot boxes — the documented process-lifetime design
  leak of the static slots (lib.rs), disclosed, not an accumulation.
- Digest read path: allocation-free across 10k calls (`report.is_zero()`),
  digest stable across the churn.

### Leg 3 — status-word exhaustiveness (typed-surface law at the ABI)

Hostile-input sweep witnesses ALL 6 refusal statuses, each via its native
fault class:

- INVALID_MAGIC: null bytes / garbage magic / truncated header (admit)
- OABI_MISMATCH: all-zero digest + payload-length overrun (admit);
  rule-count framing fault (eval)
- SHACL_VIOLATION: null view (eval); reserved flag bit (admit); lane > 255
  and null path (commit)
- LEASE_EXHAUSTED: zero-cap lane lease on a valid eval
- P28_SPOLIATION: null evidence, 79-byte evidence (verify_p28)
- CONCURRENT_WRITER_DENIED: live lease holder thread on the spill path denies
  a second claimer typed, BEFORE any IO (single-writer gate, FA9)

## Python side (ALIVE: ran this session)

Gate: `python3 -m pytest avec/tests/test_ffi_stress.py -v` → **4 passed**
(0.50s), plus `-s` receipts: `CGAP_VERSION: 26.11.2`, `ROUNDTRIP: 10k
version+digest calls, NUL court green every call`, `CDATA-CHURN: 1k
export->import cycles byte-stable`.

- ctypes round-trip stress: 10k `cgap_version`+`cgap_digest_of` calls, the
  avec NUL-termination court law applied on every call, argtypes/restypes
  declared (no pointer truncation), digest stable across the churn.
- pyarrow C-Data churn: 1k export→import cycles via `pa.Array._import_from_c`,
  byte-stable against recomputed ground truth every cycle (values bitwise,
  null_count). pyarrow 23.0.1 present → the skip path is unexercised.
- Threading posture (honest): single-threaded by charter. ctypes releases
  the GIL during the foreign call, so the posture rests on never driving the
  process-global single export slots from more than one thread; no threading
  module is imported. No contention measurements were made (not in charter).

## Gates

- `cargo test -p cgap-ffi --test ffi_stress`: 4/4 green (three runs).
- Full `cargo test -p cgap-ffi`: `--test` binaries green
  (ffi_stress 4, alloc_courts 4, zero_copy 5); **--lib has 4 PRE-EXISTING
  failures, not session-introduced** — root cause read from source: the
  sandbox courts (`xip_t1_riscv_elf_size_court`, `r2_production_link_court`,
  `status_court_reachability_ledger`, `wave9_a07_zero_string_gate...`) copy
  only root-level `src/*.rs` files into the /tmp fixture
  (sovereign.rs T1 court copy loop: `p.extension() == "rs"`), so today's
  `src/aoq/` DIRECTORY landing (mtimes 18:22-18:29 today) breaks their
  sandbox builds with E0583 (`file not found for module aoq`). Owner: the
  aoq landing lane / coordinator. My diff (test file + dev-deps) is
  independent of that copy loop. Repro: `cargo test -p cgap-ffi --lib`.
- Python: my file 4/4; the pre-existing avec suite
  (test_digest/test_pages/test_cdata) ERRORS AT COLLECTION because
  `libavec_ffi.dylib` / `avec_*` symbols do not exist in this repo — the
  binding targets a stale/renamed surface. Pre-existing, coordinator.
- cgap-crypto: broke twice mid-session under another lane's active edit
  (compile-freeze SLA); both repairs landed inside the SLA window by the
  owner; I never touched it. Recorded for the wave ledger.

## Findings (for the coordinator)

1. **avec/cgap symbol drift (pre-existing)**: `bindings/python/avec`
   speaks `avec_*` symbols against `libavec_ffi.dylib` — a renamed surface;
   the real cdylib is `libcgap_ffi.dylib` with `cgap_*` symbols. The binding
   is 2 versions behind the repo (26.10.29) and references
   "crates/avec-ffi" — a crate that does not exist in this 23-crate
   workspace. The existing avec tests error at collection (loud, not fake).
3. **Generated version drift witness**: `cgap_version` on the built dylib
   serves **26.11.2** (`crates/cgap-arrow/src/generated_version.rs` —
   generated file, ggen projection) while the workspace is **26.11.3**
   (root Cargo.toml). Generated artifact, never hand-edit: needs a
   `ggen sync run` regen by the coordinator, not a hand fix.
4. **Sandbox-copy defect** (pre-existing, owner = aoq landing lane): the
   T1/r2/status/zero-string sandbox courts copy only root-level `src/*.rs`
   (sovereign.rs T1 court loop) and break on the `src/aoq/` directory
   module. Fix shape: recursive copy or include `src/aoq/**` in the copy
   loop; also the zero-string court may share the same loop shape.
## Standing

- New Rust tests: ALIVE (command + exit cited above, ran this session).
- New Python tests: ALIVE (command + exit cited above, ran this session).
- Full cgap-ffi lib gate: PARTIAL — 54/58 unit tests pass; 4 failures are
  pre-existing, root-caused (E0583 via the stale-shape sandbox copy loop),
  owner assigned (aoq lane / coordinator), repro cited.
- Perf number standing: 3419.5 ns/call is a debug-profile measurement,
  UNVERIFIED as a release claim (release bench not in charter).
