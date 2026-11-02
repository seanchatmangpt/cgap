# W2-8: Gate reruns (gate-runner lane)

Repo: /Users/sac/cgap @ main (6801affa). Read-only except this receipt.
Session manifest changes disclosed by dispatch: workspace 26.11.2,
license/repository metadata, integration/wasm-footprint version.workspace,
cgap-gpu publish=true.

## Ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| boundary (bare, no mode) | `python3 scripts/boundary_gate.py` (/Users/sac/cgap) | 1 | usage error: mode now required (--mutate/--shim-court-only/--deps/--strings); bare invocation is a typed refusal, not a gate verdict |
| boundary deps+strings | `python3 scripts/boundary_gate.py --deps --strings --target-dir /tmp/w2-08-target` (/Users/sac/cgap) | 1 | DEPS GATE FAILED: 9 runtime externals across covered crates (§6.4: E=0); A07 CANONICAL SCAN NOT GREEN: no parsable SUMMARY verdict line, 7 crates scanned=0 (cgap-bosym, cgap-cloud, cgap-fibo, cgap-git, cgap-gpu, cgap-integration, cgap-storage) |
| boundary shim-court (non-vacuity) | `python3 scripts/boundary_gate.py --shim-court-only --target-dir /tmp/w2-08-target` (/Users/sac/cgap) | 0 | "killed by a04_identity_court - gate is non-vacuous (§6.3)" |
| zero-string self-check | `python3 scripts/zero_string_gate.py --self-check` (/Users/sac/cgap) | 0 | SELF-CHECK RESULT PASS vocab=graph-derivation; poison hits=2 fired=True; clean hits=0; 2 rmeta members loud-SKIPped |
| just doctor | `just doctor` (/Users/sac/cgap) | 1 (by design) | 8 GREEN / 2 YELLOW / 0 RED of 10 — matches w1-05 baseline |

## Doctor detail

- GREEN: c01 toolchain, c02 wasm-targets, c03 git, c04 b3sum, c05 pyshacl, c07 scripts, c09 lsl-ratified, c10 dirty-tree (67 uncommitted entries, informational)
- YELLOW: c06 workspace-check (exit 0, 5 warnings), c08 receipts (48 receipts; newest crates/ mtime newer than newest receipt by 129005s)

## Failure classification

- deps+strings exit 1: **likely pre-existing** — the 9 cgap-blake3/cgap-crypto/cgap-arrow/cgap-parquet runtime externals are the standing kernel topology, unchanged by this session; no prior-session green deps-gate receipt found (searched v26.11.2/waves, v2-02-gates.md). The A07 scan-not-green arm (scanned=0 crates) MAY be manifest-sensitive (session version.workspace bumps) — **UNVERIFIED which arm is session-introduced**; no repair performed per lane contract.
- doctor exit 1: by design (YELLOW informational); identical to w1-05 baseline.
- zero-string self-check and shim-court: PASS, non-vacuous.

## Cleanup

- Build-root lease /tmp/w2-08-target deleted via shutil.rmtree (exit 0).
- Gate ledger retained: /tmp/w2-08-deps-strings.json

```json
{"lane":"W2-8","repo":"/Users/sac/cgap","commit":"6801affa","standing":"PARTIAL_ALIVE","gates":{"boundary_bare":{"exit":1,"note":"mode required"},"boundary_deps_strings":{"exit":1,"class":"likely pre-existing, UNVERIFIED split"},"boundary_shim_court":{"exit":0,"pass":true},"zero_string_self_check":{"exit":0,"pass":true},"doctor":{"exit":1,"summary":"8 GREEN 2 YELLOW 0 RED","matches_w1_05_baseline":true}},"lease_cleaned":true,"ledger":"/tmp/w2-08-deps-strings.json","date":"2026-11-01"}
```
