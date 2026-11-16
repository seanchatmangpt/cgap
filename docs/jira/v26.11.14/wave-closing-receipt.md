# v26.11.14 ERRC Sweep — closing receipt (C4)

Standing: PARTIAL. 20 sweep lanes (18 parallel + C3 + C4), 9 fix lanes
F1-F9 and 23 gap lanes (H1-H14, G1-G9) ran. KD-7 closed by F2; nothing else flipped to CLOSED. See the final
seal table below, which supersedes the first-pass table.

Seal pass (gate-runner, code identical at e3aeec5 and b08dd82; the only
commit between them is the docs-only backlog, verified by `git diff --stat`):

| Check | Exit | Verdict |
|---|---|---|
| `cargo build --workspace` | 0 | PASS (warnings only, cgap-beam) |
| `boundary_gate.py --deps` | 0 | PASS (E=0 across 15 crates) |
| `boundary_gate.py --self-check-deps` | 0 | PASS (both arms flip) |
| `zero_string_gate.py --self-check` | 1 | ENV (objdump format on Linux; other arms PASS) |
| `cargo test -p cgap-ffi --lib` | 101 | 66 pass, 3 fail, all 3 ENV |

The 3 cgap-ffi failures reproduce identically on pre-sweep `ede726f`, so
none is a regression. Two are macOS `.dylib`-path courts that cannot pass on
Linux. The third (`wave9_a07_zero_string_gate...`) exited 2 "subject
missing"; ENV is INFERRED from the objdump failure, not directly shown, and
a code defect is not ruled out.

## Lane standings

ALIVE: E4, E6, Ra4. Audit-only ALIVE: E3, E5.
PARTIAL: C1, C2, E2, R1, R2, R3, R4, R5, Ra1, Ra2.
BLOCKED: E1 (ggen absent), Ra3 (A19 slot resolution), Ra5 (registry chain).

## Not verified here

- Registry state (no probe beyond Ra5's dry-run index fetch).
- The e2e pipeline run behind Ra1's band; R2's wasm32 footprint; C1's `.rq`
  gates; all macOS-hosted gates (`ffi_surface_gate.py`, `crates/cli` build).
- The full storage suite is not green (pre-existing `closure.rs:230`).
- Perf numbers are TARGETs: R3 timings were contention-suspect (sibling
  lanes), Ra2 is host-scoped to 4 cores.

## Process notes

- A sandbox sharing a cargo target dir with the shared tree gave a false
  FAIL (Ra4); the E2 kd7 link error seen by R1 did not reproduce on a clean
  target dir (orchestrator re-check, in R1.md).
- Nothing was published. CLAUDE.md was edited once, by F1 (stale-claim fixes only).
- Ranked follow-ups: `errc-backlog.md`.

## Final seal (after F1-F9, gate-runner at HEAD 5004ddc + orchestrator re-checks)

| Check | Exit | Verdict |
|---|---|---|
| `cargo build --workspace` | 0 | PASS |
| `boundary_gate.py --deps` / `--self-check-deps` | 0 / 0 | PASS |
| `zero_string_gate.py --self-check` (post-F8) | 0 | PASS on Linux, leg A exercised; macOS path UNVERIFIED |
| `cargo test -p cgap-ffi --lib` (artifacts in target/) | - | wave9_a07 2/2 PASS after F8 (orchestrator run); full lib run not repeated after F8 |
| `closure_court` | 0 | PASS 3/3 (orchestrator re-run) |
| cgap-core / cgap-parquet suites | 0 | PASS |
| cgap-git egress_ring_court | 0 | PASS, 0/40 fails (orchestrator, after F9) |
| rdflib battery (C1) | 0 | PASS |

Caveats: the seal agent's cgap-git run saw one flaky failure before F9;
cgap-git's full suite was not re-run end to end after F9 (only the egress
binary, 40 runs). Courts-mode (T17) and boundary_gate full modes were not
run. Open items: see "Still open" in errc-backlog.md.

## Round 3 seal (HEAD after G3; orchestrator checks, all this session)

| Check | Exit | Verdict |
|---|---|---|
| `env -u CARGO_TARGET_DIR cargo build --workspace` | 0 | PASS |
| `boundary_gate.py --deps` / `--self-check-deps` | 0 / 0 | PASS |
| `zero_string_gate.py` / `--self-check` | 0 / 0 | PASS (debris leg exercised via ELF stub; macOS path UNVERIFIED) |
| G3: cgap-git, cgap-ffi --lib (69/69), cgap-storage, cgap-core | 0 each | PASS (one run each; flake rate unknown) |
| G4: --strings, --shim-court-only, zero_string --courts | 0 each | PASS |
| `boundary_gate.py --mutate` (G9, /tmp copy only) | 0 | PASS in sandbox, 16/16 killed; shared tree not run; re-key inferential |
| cgap-mcp (G8, own manifest) | 0 | PASS, compile break fixed |

Not verified: full e2e (arcus/smalt absent), liboqs interop, OTP>=26 kill_stress,
b3sum KAT, macOS paths, ggen replay, registry publish. Details and open items:
`errc-backlog.md` (Round 3 section). CLAUDE.md edits: F1 and G2 (G2 is
policy-adjacent; owner to confirm).
