# f3-04-courts-green

Wave F3 item 4 gate-runner receipt. Repo `/Users/sac/cgap`. Precondition: bounded
poll (20s interval, 600s deadline) for `f3-01-version-lock.md` — present at t=380s
(2026-11-01 16:24), so all legs ran on the post-version-bump tree. Exits captured
directly (`cmd > out 2>&1; echo $?`) — an initial `${PIPESTATUS[0]}` capture was
invalid under zsh (unbound, fell back to tail's rc) and was re-run; cited exits are
from the re-runs. Build root `CARGO_TARGET_DIR=/tmp/f3-4-target`, deleted post-run.

## Commands and exits

| leg | command | exit | observed |
|---|---|---|---|
| poll | poll `waves/f3-01-version-lock.md` every 20s, deadline 600s | n/a | present t=380s |
| 1 | `CARGO_TARGET_DIR=/tmp/f3-4-target cargo test -p cgap-court --test court_verdict` | 0 | 13 passed; 0 failed (13.63s) |
| 2 | `python3 scripts/boundary_gate.py --mutate --report /tmp/f3-4-ledger.json` | 0 | 19/19 killed, kill_rate 1.0000 >= 0.98, 5 equivalent, 0 survivors |
| 3 | `python3 scripts/boundary_gate.py --shim-court-only` | 0 | killed by a04_identity_court (non-vacuous, §6.3) |
| 4 | `python3 scripts/boundary_gate.py --shim-court-only --shim-inert` | 1 | VACUOUS GATE fired — negative control behaves as required |
| 5 | `python3 scripts/zero_string_gate.py --self-check` | 0 | poison fired=True hits=2; clean hits=0; RESULT PASS |

## Receipts

- Ledger re-read from disk: `/tmp/f3-4-ledger.json` -> `kill_rate: 1.0`,
  `abi_version: 1` (keys: config/rows/skips/non_vacuity/kill_rate/summary).
- Leg-4 exit 1 is the REQUIRED negative-control refusal, not a failure: the inert
  shim not being killed voids every pass the court could produce (§6.3).
- No leg red; no repairs made; no git writes; lease scope respected.
