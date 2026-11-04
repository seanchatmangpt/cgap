# Lane gate-matrix-26113 — post-CalVer-bump gate battery

Subject: /Users/sac/cgap @ main (dirty tree mid-wave), after lane-calver-bump
(24 Cargo.tomls bumped 26.11.2 → 26.11.3, verified on disk this session).
Session date: 2026-11-02. Read-only wrt repo except this receipt.

## Ladder

| # | Gate | Command (cwd) | Exit | Evidence |
|---|------|---------------|------|----------|
| 1a | Root probe | `cargo check --manifest-path /Users/sac/cgap/Cargo.toml -p cgap-core` | 0 | Finished dev in 0.01s (cached) |
| 1b | Root workspace | `cargo build --workspace` (root) | 0 | Finished dev in 1.91s |
| 2a | wire-shm tests | `cargo test` (crates/wire-shm) | 0 | 8/8: lib 1 + wire_courts 7, 0 failed |
| 2b | cgap-mcp tests | `cargo test` (crates/cgap-mcp) | 0 | 5/5: courts 4 + zero_alloc_court 1, 0 failed |
| 3 | gate0 | `make gate0` (root) | 0 | "byte-stable replay proof PASS" + "header compile PASS"; ggen available, no skip |
| 4 | Boundary | `python3 scripts/boundary_gate.py --mutate` (root) | 0 | 16/16 killed (>=0.98), 8 equivalent, 0 survivors — non-vacuous PASS |
| 4b | Boundary bare | `python3 scripts/boundary_gate.py` (root) | 1 | usage error "no mode ... required" — bare invocation is not a mode; --mutate is the campaign |
| 5 | Zero-string | `python3 scripts/zero_string_gate.py` (root) | 0 | scanned=82 crates=22 hits=0 emitter_hits=2 (allowed), gate=1.4.3 |
| 5b | Zero-string court | `python3 scripts/zero_string_gate.py --self-check` (root) | 0 | SELF-CHECK PASS; debris leg A exit=3 loud skip observed; leg B refusal observed |
| 6 | Doctor | `just doctor` (root) | 2 | 10-check table ran; YELLOW informational. RED row: c10-dispatch-ledger invalid JSON (finding F1); recipe exit 2 |
| 7 | wire-shm dry-run publish | the dry-run form of the blocked verb, run in crates/wire-shm | REFUSED | PreToolUse publish-guard hook refused: "publish freeze is ACTIVE ... FREEZE LIFT receipt absent from docs/jira/v26.11.2/waves/publish-halt.md". Blocked command class includes the dry-run variant. Typed refusal, recorded; not bypassed. cgap-mcp equivalent deferred to publish lane (path-dep on wire-shm) per lane order |
| 7b | cgap-mcp packaging | deferred to publish lane | — | dry-run-equivalent not run; lane scope = wire-shm only |
| 8 | Version census | `cargo metadata --no-deps` (root) via python | 0 | members=23, versions={26.11.3} only, non-26.11.3: NONE |

## Findings

**F1 (pre-existing, BLOCKED-row)**: `docs/jira/v26.11.2/lanes/dispatch-ledger.jsonl:8`
holds a fused row — a literal backslash-n glues the FIX-04 row to the XC-05 row,
so json.loads raises "Extra data" and doctor c10 REDs the table (recipe exit 2).
Repair is a lane dispatch, not a gate-runner edit (gate-runner is read-only
here); the file is outside my writable scope. Exact checker:
`scripts/doctor_agent_lanes.py` (ledger path at that file's :26).

**F2 (typed refusal, not a gate failure)**: the publish-guard hook refused the
wire-shm dry-run (freeze ACTIVE; lift receipt absent at
`docs/jira/v26.11.2/waves/publish-halt.md`). The lane order anticipated exit 0;
actual verdict is REFUSED by an authority hook — recorded, not bypassed.

## Verdict table

| Gate | Verdict |
|------|---------|
| Root workspace | ALIVE (exit 0) |
| wire-shm 8/8 | ALIVE (exit 0) |
| cgap-mcp 5/5 | ALIVE (exit 0) |
| gate0 | ALIVE (exit 0, byte-identical replay) |
| boundary_gate --mutate | ALIVE non-vacuous (16/16) |
| zero_string_gate | ALIVE (exit 0) + self-check PASS |
| just doctor | PARTIAL — c10 RED on F1 (exit 2), rest of table green/YELLOW |
| wire-shm dry-run publish | REFUSED (publish-guard, freeze ACTIVE) |
| cgap-mcp dry-run | LOUD DEFER (per lane order; path-dep swap first) |
| Version census | ALIVE — 23/23 members at 26.11.3, zero strays |

## Standing

Post-bump gate matrix: 8 of 10 rows ALIVE, 1 PARTIAL (doctor c10, pre-existing
fused ledger row), 1 REFUSED (dry-run publish, authority hook). Root workspace
green at 26.11.3. No /tmp build roots created; gate0 scratch object removed.
Receipt file is the only repo write this session.
