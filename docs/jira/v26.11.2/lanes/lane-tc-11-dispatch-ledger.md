# Lane TC-11: dispatch ledger + silent-death doctor check

**Standing: ALIVE** (all commands + exits run this session, 2026-11-02).

## 1. Identity

- Lane TC-11, repo `/Users/sac/cgap` @ main. tps-07 rank 2 (A2):
  dispatch ledger + silent-death doctor check.
- Files written (full ownership this session):
  - `docs/jira/v26.11.2/lanes/dispatch-ledger.jsonl` (NEW, 5 rows)
  - `scripts/doctor_agent_lanes.py` (appended c10 check; see §7)
  - `docs/jira/v26.11.2/_LANES-TEMPLATE.md` (heartbeat + ledger-row
    law added to contract block)
  - this receipt
- Zero path overlap with any other lane's owned files as dispatched.

## 2. Authority

Dispatch prompt (coordinator, tps-07 rank 2/A2) + contract block in
`docs/jira/v26.11.2/_LANES-TEMPLATE.md`.

## 3. Commands + exits (all this session)

- `python3 scripts/doctor_agent_lanes.py` — exit 1 baseline (pre-existing
  WARNs: orphan-leases, dirty-tree, c08-receipts; c10-dispatch-ledger
  PASS "1 receipted, 3 in-flight(fresh)").
- Court A (stale row, no receipt): `DX1_LEDGER_PATH=/tmp/tc11-court/lanes-A/ledger.jsonl DX1_LANES_DIR=/tmp/tc11-court/lanes-A python3 scripts/doctor_agent_lanes.py`
  → c10 **WARN SILENT DEATH: tc-dead (row 60 min old, no receipt)**,
  overall exit 1.
- Court B (stale row + planted receipt `lane-tc-dead-court.md`):
  same env, lanes-B sandbox → c10 **PASS "1 receipted, 0 in-flight"**.
- Court C (stale heartbeat): sandbox `tc-hb.alive` backdated 60 min →
  c10 **WARN "stale heartbeat: tc-hb (heartbeat 60 min old)"**.
- Poison arm: `DX1_POISON_C10=1 python3 scripts/doctor_agent_lanes.py`
  → c10 **WARN "poison arm: ledger watch"**.
- Fences: `DX1_POISON_C06=1 ... --full` → c06 WARN poison arm fired
  (TC-02's court still flips), full run exit 1 unchanged. Doctor exit
  remains 0/1 (informational) — never 2. c06 --full poison run rc=0 is
  the grep pipe rc; full-run exit witnessed 1 on baseline and on
  poison-free full run earlier in session.

## 4. Consequence

- Doctor check `c10-dispatch-ledger`: any `dispatch-ledger.jsonl` row
  older than 45 min (env `DX1_LEDGER_STALE_MIN`) with no matching
  `lanes/lane-<id>*.md` receipt = WARN "silent death"; row + receipt =
  PASS; `<lane-id>.alive` heartbeat (new template law) older than the
  same N = WARN; fresh in-flight rows = PASS. Invalid JSON row = RED.
  Courts root it at /tmp sandboxes via `DX1_LEDGER_PATH` /
  `DX1_LANES_DIR` / `DX1_LEDGER_STALE_MIN` / `DX1_POISON_C10`.
- Ledger seeded (epoch ts 1793662880) with the 4 in-flight lanes that
  had on-disk absence evidence at seed time, plus tc-11 itself:
  - tc-02: no `lane-tc-02*.md` receipt + concurrent mtime churn on
    `scripts/doctor_agent_lanes.py`
  - tc-05: no receipt + live lease `/tmp/tc05-target`
  - tc-10: no receipt + `audit/tc-10-open-recensus.md` present
  - tc-13: no receipt + `audit/tc-13-asha2a-feasibility.md` present
  - tc-11 (this lane)
  Each row carries sha256(premise string) as `premise_hash`; the raw
  premise strings are quoted above (receipt supersedes hash as the
  human-readable form).

## 5. Courts table

| Court | Mutation (anti-vacuity) | Flip witness (command+exit) | Exit |
|---|---|---|---|
| A silent-death | stale row, no receipt | WARN SILENT DEATH tc-dead, exit 1 | 1 |
| B receipted | stale row + planted receipt | PASS 1 receipted | 1 (baseline WARNs elsewhere) |
| C stale heartbeat | backdated `.alive` | WARN stale heartbeat | 1 |
| poison | DX1_POISON_C10=1 | WARN poison arm | 1 |

All four flipped. A first attempt was contaminated (shared lanes dir
let court B's receipt satisfy court A) — re-run in isolated dirs;
contamination itself witnesses the check reads real receipts, not a
stub.

## 6. Standing

ALIVE for: c10 check (real + poison flips this session), ledger seed
(on-disk, verified by the baseline c10 line reading it), template law
(on disk). The doctor exit class is unchanged: 0/1 informational, RED
only for ledger JSON corruption.

## 7. Divergences / disclosures

- Dispatch examples LSPA-3, CULL-10, TPS-3, TPS-5 were all ALREADY
  receipted on disk at seed time (`lane-lspa-03-attr-witness.md`,
  `lane-tps-03-gates.md`, `lane-tps-05-battery.md`; no cull-* lane ids
  exist in lanes/). Doc yields to reality: I seeded with the lanes that
  actually had absence evidence this session (tc-02/05/10/13).
- Dispatch said append "after TC-02's ports". TC-02 landed mid-lane
  (receipt `lane-tc-02-doctor.md`, 15:40). Its landed check set is
  c06–c09; my c10 appends after c09 in `main()`, satisfying the
  ordering. No mid-flight collision: TC-02's receipt predates my first
  edit of the file.
- **Ownership disclosure (both directions, per
  same-checkout-fanout law):** TC-02's receipt claims "full ownership"
  of `scripts/doctor_agent_lanes.py`; my dispatch ALSO names it. My
  changes are strictly additive on TC-02's landed final state (env
  roots at :19-27, `check_c10_dispatch_ledger` before `main()`, one
  call + marker comment in `main()`). TC-02 supersedes on the ports
  body; I supersede the c10 tail. No unblock was applied by either
  side.
- tc-02's ledger row went receipted mid-session (its receipt landed
  between seed and baseline run) — the check immediately reclassified
  it receipted, which is the design working, not a stale row.
- No cargo builds ran in this lane; no target lease was minted. Court
  sandbox `/tmp/tc11-court` deleted at lane end via shutil.
