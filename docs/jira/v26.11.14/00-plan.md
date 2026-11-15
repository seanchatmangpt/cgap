# v26.11.14 ERRC Sweep — plan

ERRC = Eliminate / Reduce / Raise / Create. Defined 2026-11-15 (no prior spec
existed; newest prior dir was v26.11.6). 20 lanes: 18 parallel, then C3, C4.

Per-lane receipt: `docs/jira/v26.11.14/lanes/<id>.md` (Standing: ALIVE /
PARTIAL / BLOCKED / FAIL; commands + exit codes; named anti-vacuity mutation).

## Contract block (every lane)
- Absolute repo path /home/user/cgap. No git writes. Stage nothing.
- Own only the files named in the lane + its receipt. No generated-file hand edits.
- Chicago courts, no mocks; mutate /tmp sandbox copies only (sha256 restore).
- ALIVE needs a command + exit this session. UNVERIFIED never rounds up.
- No publishing. Writes in <50-line chunks; verify on-disk bytes.

## Lanes
| id | action | scope |
|----|--------|-------|
| E1 | Eliminate | KD-3 ATTESTATION_LEN twin, cgap-core frame.rs:44 |
| E2 | Eliminate | KD-7 `_cgap_p28_invariant_check` header decision |
| E3 | Eliminate | wire-shm delete-first audit (verdict only) |
| E4 | Eliminate | OI-1: land cgap_e2e_consumer as [[bin]] + receipt |
| E5 | Eliminate | stale CLAUDE.md census numbers vs cargo metadata |
| E6 | Eliminate | duplicate helpers across 4 zero-dep kernels |
| R1 | Reduce | cgap-ffi warnings |
| R2 | Reduce | wasm-footprint, linked-ELF method only |
| R3 | Reduce | chains-* alloc columns |
| R4 | Reduce | crates/cli generated-vs-hand boundary |
| R5 | Reduce | panic/unwrap paths per v26.10.31/81-error-path-coverage.md |
| Ra1 | Raise | OI-4 admission-ratio band in e2e court script |
| Ra2 | Raise | penstock contended band (KD-2 residue) |
| Ra3 | Raise | KD-4 oabi-lookup / A19 reachability |
| Ra4 | Raise | court anti-vacuity coverage census |
| Ra5 | Raise | cgap-aoq publish dry-run preflight (no publish) |
| C1 | Create | docs-diataxis-pack generalization (DIA-5) |
| C2 | Create | missing v26.11.1 ERRC slice evals AD/EL/MR/SZ |
| C3 | Create | errc-backlog.md synthesis (after 18) |
| C4 | Create | wave-closing-receipt.md (last) |

Seal: gate-runner pass after C4.
