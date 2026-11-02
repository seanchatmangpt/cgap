# dxd-10-dx-wave-plan

DXD-10 DX implementation wave plan (2026-11-02). Consumes DXD-01..05,
07..09; refuses re-proposals of the landed DX fleet
(lanes/lane-dx-01..08). PLAN only — no code lands here.

## 0. Landed-fence register (REFUSED to re-propose)

| landed tool | receipt | fence |
|---|---|---|
| run_checked.py | lane-dx-05 | additive, no callers |
| lease_janitor.py v1 (dirs) | lane-dx-03 | just leases wired |
| doctor_agent_lanes.py 4-check base | lane-dx-01 | just doctor-agents |
| write-integrity hook | lane-dx-02 | flip-tests 9/9 |
| test_census.py + just census | lane-dx-08 | CENSUS_ARGS export form |
| receipt_lint strict/STRUCT/--fix | lane-dx-04 | census 445/190 |
| lane-runner scaffold + template | lane-dx-07 | flip-tests 9/9 |
| shared-fixture sweep + court | lane-dx-06 | cgap-court 14+3 exit 0 |

Ledger cross-check: audit-ledger.md has no DXD/DX rows — nothing
double-registered. Pending: DXD-06 coverage plan (doc absent; the
DX-6 fixture-race RECEIPT is landed — wave lanes must not re-touch
the six fixed race sites or shared_fixtures_court.rs).

## 1. Wave lanes (disjoint ownership)

| lane | owns | scope | source |
|---|---|---|---|
| W10-DX-A | scripts/doctor_agent_lanes.py (grow in place) | M | DXD-02+09 |
| W10-DX-B | scripts/ffi_drift_gate.py (NEW) + ffi/ffi_symbol_registry.toml (NEW) + crates/cgap-ffi/tests/ffi_drift_pins.rs (NEW) | L | DXD-04 |
| W10-DX-C | scripts/lease_janitor.py (FILE_GRAMMAR/GUARDS/--interop/--roots/--selftest) | M | DXD-08 |
| W10-DX-D | courts/interop-chicago scripts+docs (build_dylib.sh, fixtures regen doc, seal example JSON, go.mod pin); NOT ffi/ffi.go (IPC-2 owes the #include) | M | DXD-03 |
| W10-DX-E | scripts/receipt_lint.py fixture arm + scripts/receipt_lint_fixtures/ (NEW) | S | DXD-05 |
| W10-DX-F | justfile ONLY (single writer for the wave) | S | DXD-01+07+08 |

Justfile: ONE writer (F) so the DX-1 duplicate-recipe abort class
cannot recur; A-E file wiring specs to F, never edit it.

## 2. Courts-with-flips (per lane)

- A: DXD-02's 6 checks + DXD-09's adds (nightly-channel probe,
  homebrew-llc riscv-backend probe, LC_RPATH via otool, ipc3-target
  WARN row, orphan census from C's grammar). Each check's DXD_* env
  flip (stripped list/PATH -> WARN; GO_VERSION 1.21.5/1.22.0/garbage
  -> YELLOW/GREEN/RED; utime dylib -> WARN stale; forced-platform
  shm -> YELLOW; PMSET fixture -> YELLOW). Fence: DX-1's 4 checks
  byte-identical in behavior.
- B: nm ground truth (per lane-ipc-00 W3 — source grep lied once);
  wave3_courts header scanner; registry TOML [symbols]/[roadmap]/
  [structures]; roadmap absence = PENDING (pass), unregistered
  presence = FAIL. Mutations: deleted cgap_eval_bn3 decl -> exit 1 +
  greppable FAIL row; retyped payload_len -> cc pin compile-fail;
  deleted Go extern -> Go column flips. Negative control: in-tree
  header -> exit 0.
- C: FILE_GRAMMAR + GUARDS (repo target*, ACTIVE corpus,
  resolved-path, symlinks) + --roots dual + --interop; default
  behavior byte-identical. Courts: idempotence, negative-control
  survivors, guard court. Fence: DX-3 grammar + planted court pass.
- D: DXD-03 gaps 1/2/5/7 (toolchain pin, build_dylib.sh canonical
  build, fixtures regen doc, seal example JSON). Gap 3 (ffi.go
  #include) is IPC-2's inbound dependency — vet-1 stays ATTRIBUTED.
  Gap 8: record do-NOT-document DYLD_LIBRARY_PATH.
- E: DXD-05 GAPs A-D as fixtures + minimal rule deltas; each flips
  both directions. Fences: --strict typed exit 1; --self-check rc 0;
  GAP-05 crash fix holds; GAP E informational (sha256 divergence
  accepted, never BLAKE3-demanded).
- F: order (1) cosmetic dup comment :81-82, (2) single `interop:`
  recipe at EOF (DXD-01 verdicts — no interop-build/vet/test/bench),
  (3) clean-interop after C, (4) doctor-merged after A. {{NAME}}
  resolution law; no unprefixed vars.

## 3. Dependency order

C first (A's orphan census sources its grammar); B and D independent
parallel; A after C; E fully independent; F LAST (wraps A+C outputs;
parse probe after every insertion).

## 4. Wave Verify

Per lane: courts with flips witnessed, /tmp-only fixtures; shared
fence `just --evaluate` exit 0 + `just --list` diff. Close: one
audit-ledger row per lane; receipts lanes/lane-w10-dx-{a..f}-<slug>.md.

## 5. Wave falsifier (verbatim)

the DX wave Verify passes iff every new recipe/doctor check has a
witnessed poison-arm flip and zero justfile parse regressions
(just --evaluate exit 0 after every lane).

## See Also

audit/dxd-01..09 · lanes/lane-dx-01..08 · audit/audit-ledger.md
