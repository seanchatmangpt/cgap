# dxd-01-justfile-census

DXD-01 justfile census (2026-11-02). Subject: /Users/sac/cgap/justfile
(94 lines). `just --list` exit 0 (fresh).

## 1. Landed recipes

| recipe | :line | source |
|---|---|---|
| default | 15-16 | — |
| doctor | 21-34 | W2-14 chain, cgap-cli doctor bin |
| verify | 37-38 | verify_v26.10.31.sh |
| bench | 41-43 | Rust bench crate + ctq_report |
| courts | 46-52 | 3 shell courts, no Go |
| branchless | 55-56 | LSPA-6 verify_lsp_branchless.py |
| receipts | 59-66 | receipts dir tally |
| dod / status | 69-70 / 73-74 | fail-closed placeholders |
| leases | 78-79 | DX-3 lease_janitor.py |
| doctor-agents | 85-86 | DX-1 doctor_agent_lanes.py |
| census | 91-93 | DX-8 test_census.py (CENSUS_ARGS export) |

No collisions among the landed DX/LSPA recipes. Cosmetic: duplicated
doctor-agents header comment :81-82.

## 2. Per-recipe verdicts

- interop-build: EXTEND — genuinely missing (verify is cargo-only;
  zero `go` invocations anywhere in the file).
- interop-vet: EXTEND — missing; README:37 is the only canonical form.
- interop-test: SKIP as its own recipe — census covers Rust targets
  only (test_census.py:32-52 enumerates via cargo metadata; Go
  packages invisible); `go test ./...` subsumes compile checking.
- interop-bench: EXTEND — bench is Rust-only; Go FFI benches
  (ffi/alloc_court_test.go) uncovered.
- interop-seal: SKIP/EXTEND borderline — sealer exists (IPC-6), no
  recipe runs it; sealing is per-receipt emission, low loop value.

## 3. Overlap questions answered

- doctor-agents covers dylib staleness? NO — check_stale_server
  (doctor_agent_lanes.py:46-72) covers only the branchless-lsp binary
  in lsp-max; the cgap-ffi cdylib has no staleness check anywhere.
- census covers go tests? NO — cargo metadata enumeration only.

## 4. Minimal additive set (non-duplicative)

```
# interop-chicago Go court: build+vet+test in one fail-fast pass
interop:
    cd courts/interop-chicago && go build ./... && go vet ./... && go test ./...
```

Optional second only if seal-emission recurs:

```
interop-seal path:
    cd courts/interop-chicago && go run ./cmd/seal_receipt {{path}}
```

Do NOT add interop-build/interop-vet (stages of `interop`),
interop-test (folded), or interop-bench (extend `bench` if ever
needed). Net: 1 recipe now, 1 conditional.

## See Also

audit/dxd-07-justfile-collisions.md · audit/dxd-02-doctor-design.md
