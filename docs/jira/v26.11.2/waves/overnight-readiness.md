# Overnight-Readiness Receipt (BP-9)

Lane: BP-9 OVERNIGHT-READINESS (gate-runner). Repo: /Users/sac/cgap.
Lease: docs/jira/v26.11.2/waves/overnight-readiness.md (this file).
No git writes. Verdict: READY-WITH-GAPS (gaps below do not block launch;
the loop's own loud-skip/BLOCKED paths absorb every one of them).

## Five unattended-run questions

### 1. DONE-CONDITION

Written down, on disk: OVERNIGHT-LOOP.md §5 Terminus law — terminal
state is exactly the complete 22-crate table (every cell pass /
registered-divergence / loudly-MISSING) PLUS an eight-gate receipt
(G1-G8, each with first-hand command+exit); "BLOCKED is a valid
terminal cell; silence is not." Gate specs: 01-release-gates.md
(G1-PUBLISH ... G8-STARDF-F1, each COMMAND/RECEIPT/FLIP/BLOCKED).
Wave order + iteration law: OVERNIGHT-LOOP.md §2 (V2-A tail -> V2-B
five levers looped -> V2-C positioning/release) + 05-wave-plan.md
(wave serialization, 7-agent peak).

### 2. SELF-CHECK

The loop checks itself via: (a) per-lever anti-vacuity flips
(OVERNIGHT-LOOP.md §2 V2-B: each lever lands only with a witnessed
kernel flip; 01-release-gates.md G2 FLIP: threshold met without a flip
= `VACUOUS`, exit 3); (b) Divan PARETO before/after on the same
subject, row admitted only if the measured delta is in the claimed
direction (OVERNIGHT-LOOP.md V2-B; 05-wave-plan.md V2-B: "never round
up"); (c) the eight gates run first-hand with exit codes recorded
(OVERNIGHT-LOOP.md §3; 01-release-gates.md dispatch blocks).
Non-vacuity is structural: exit-code contract 0/1/2/3, mutations only
in /tmp sandbox copies with sha256 fail-closed restore (both files).

### 3. TOUCH-SCOPE

BP-2 LANDED: /Users/sac/cgap/.claude/settings.json — verified exists,
parses (python3 json.load -> BOTH_VALID, exit 0, this session), and
carries permissions.allow (cargo test/check/metadata --no-deps/bench
--no-run; boundary_gate.py + zero_string_gate.py incl. --self-check;
crates.io index reads with cgap-publish-receipt UA; ls/grep/find;
read-only git status/log/diff), permissions.ask (git push), and
permissions.deny (Read .env/.env.local/secrets/**). Receipt:
waves/bp-2-settings.md (ALIVE for the settings file itself).
settings.local.json: exists, parses, 10 allow entries
(cargo build/test/metadata/publish, ggen sync run, just doctor/status,
manifest_patch.py, scripts/*, escript test/*); SM-2 receipt records
NO EDIT on both files.
BP-8: NOT LANDED. No receipt, no file, no reference to "BP-8" anywhere
under docs/jira/ (grep over v26.11.1 + v26.11.2 returned zero hits).
OPEN — named gap G-1 below.
One-canonical-checkout / no-worktree law: 05-wave-plan.md Scope
("Three waves over the one canonical checkout /Users/sac/cgap, branch
main, coordinator owns all git transitions") + OVERNIGHT-LOOP.md header
+ repo CLAUDE.md. Lanes are leases, not copies.

### 4. STOP CONDITIONS

Cited from OVERNIGHT-LOOP.md §4: no Process.sleep; bounded polls with
explicit deadlines, deadline expiry = typed BLOCKED, never spin;
handoff poll ~60s interval, <=45 min total (HANDOFF.md still absent —
loud, not silent); publish law = ONE attempt per crate, sequenced
post-window, stop-on-slide + 1hr backoff, no retry storms; peak
concurrency 7 agents; compile-freeze SLA ~10 min with disclosed
minimal unblocks; census-vs-abort rule (exit 1 with 0 tests executed
is a compile-abort, never a failure tally).
--max-turns per agent leg: NOT WRITTEN in OVERNIGHT-LOOP.md or
05-wave-plan.md. Recommendation (this lane): 50 turns per lane-runner
leg (a lever lane = read receipts, edit 1-2 crates, run its flip +
bench, write receipt — 50 bounds a runaway without truncating a legal
leg); 30 turns per court-auditor / gate-runner leg (audit + rollup is
read-heavy, narrow). Spend consideration: peak 7 agents x ~3 waves of
legs; a hard per-leg turn cap is the spend control (no dollar meter
exists in the runbook). These caps are this lane's recommendation —
recommend the operator write them into OVERNIGHT-LOOP.md §4 before
launch (gap G-2).

### 5. RESULT DESTINATION + SIGN-OFF

Destination: docs/jira/v26.11.2/waves/ receipts (v2-14-loop.md is the
V2-14 loop receipt lease; all lane receipts land in waves/ per every
lane's lease block). Terminus: OVERNIGHT-LOOP.md §5. Coordinator
sign-off items: waves/v2-13-commit-manifest.md (ordered commit plan
over 265 dirty cgap paths + 30 stardf paths, executed AFTER wave close,
zero git writes by lanes); beam decision (RESTART-NOW.md: cgap-beam
"guarded, no receipt" = lawful non-publish); integration retry
(RESTART-NOW.md + OVERNIGHT-LOOP.md §4: cgap-integration owes ONE
publish invocation, sequenced post-window, stop-on-slide + 1hr
backoff); plus waves/v2-12-stop-list.md KEEP obligations (P3-13
17-crate sweep, RDF-11 auditor sign-off, RDF-12, PP-4, PP-7).

## Exact launch invocation (operator)

    cd /Users/sac/cgap && claude -p "$(cat docs/jira/v26.11.2/OVERNIGHT-LOOP.md \
      | sed 's/^/# /')" \
      --append-system-prompt "You are the overnight loop driver for
      /Users/sac/cgap. Your governing runbook is
      docs/jira/v26.11.2/OVERNIGHT-LOOP.md — read it first and execute
      §2 wave order V2-A -> V2-B -> V2-C, then §3 gates G1-G8,
      first-hand, citing command+exit for every claim. One canonical
      checkout; no git writes by lanes; receipts to
      docs/jira/v26.11.2/waves/. Terminus law §5 is the only done.
      Cap every agent leg at 50 turns (30 for audit/gate legs). On
      publish: one attempt, stop-on-slide + 1hr backoff. Loud skips
      only: [SKIP] <reason>." \
      --permission-mode acceptEdits \
      --allowedTools "Bash(cargo test *)" "Bash(cargo check *)"
        "Bash(cargo metadata --no-deps *)" "Bash(cargo bench --no-run *)"
        "Bash(python3 scripts/boundary_gate.py)"
        "Bash(python3 scripts/zero_string_gate.py)"
        "Bash(python3 scripts/zero_string_gate.py --self-check)"
        "Bash(curl -A cgap-publish-receipt https://index.crates.io/*)"
        "Bash(ls *)" "Bash(grep *)" "Bash(find *)"
        "Bash(git status *)" "Bash(git log *)" "Bash(git diff *)" \
      --max-turns 50

Notes: --allowedTools mirrors BP-2's permissions.allow exactly (no
escalation beyond the landed rules; git push stays in ask via
settings.json, so the loop will pause for operator approval on push —
intended). cargo bench runs and gate-script args beyond the pinned
forms are NOT in the allowlist: first occurrence pauses to operator
(ask) or is refused — the operator should either pre-approve
`Bash(cargo bench *)` and `Bash(python3 scripts/release_check.py)`
or expect pauses. --permission-mode acceptEdits (not bypassPermissions)
keeps the ask/deny rails live.

## Gap list (READY-WITH-GAPS)

- G-1: BP-8 lane not landed — no receipt, no file, zero references in
  docs/jira/. If BP-8 was chartered to land a settings surface (e.g.
  settings.local.json rules or a permissions refinement), it is OPEN
  under lane id BP-8; BP-2's settings.json alone is sufficient to
  launch (the invocation above needs nothing from BP-8).
- G-2: --max-turns per leg not written in OVERNIGHT-LOOP.md §4; this
  receipt proposes 50/30. Operator should ratify before launch or pass
  --max-turns 50 on the CLI (invocation already does).
- G-3: G1 self-fails until the 26.11.2 version bump (root Cargo.toml
  [workspace.package] reads 26.11.1 on disk, verified this session);
  release_check.py:21 still pins 26.10.29 per 01-release-gates.md and
  requires the pre-run edit. Both are V2-C item-4 work, expected to be
  absorbed by the loop itself; until then G1 exits 1 by design.
- G-4: BLOCKED-path subjects absent on disk (as documented in
  01-release-gates.md): mat-closure + DRED bench targets absent in
  crates/bench; verify_ost_design_around.py + patent/claims.txt
  absent; purrdf/scripts lacks stardf_f1_census.py / stardf_golden.py
  / kd_flip.py. G3/G4/G7/G8 will exit 2/3 (typed BLOCKED) — loud, and
  BLOCKED is a valid terminal cell per §5.
- G-5: HANDOFF.md (V2-11) still missing; OVERNIGHT-LOOP.md §1 already
  prescribes the loud HANDOFF: MISSING log + bounded-poll recovery, so
  this does not block launch.

## Verdict

READY-WITH-GAPS. Every gap is either absorbed by the loop's own
loud-skip/BLOCKED paths (G-3, G-4, G-5) or is an operator pre-launch
ratification (G-1, G-2). Zero commands would be VACUOUS; this receipt
cites: settings.json + settings.local.json parse (BOTH_VALID, exit 0),
ls of .claude/hooks/ and scripts/, grep for BP-8 (zero hits), version
read of root Cargo.toml (26.11.1), reads of OVERNIGHT-LOOP.md,
01-release-gates.md, 05-wave-plan.md, RESTART-NOW.md,
waves/bp-2-settings.md, waves/sm-2-settings.md.
