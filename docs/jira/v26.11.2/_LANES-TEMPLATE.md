# Lanes Template

Copy this scaffold per lane. Receipts land at
`docs/jira/v26.11.2/lanes/lane-<id>-<slug>.md`.

## Contract block (copy verbatim into every dispatch prompt)

    LANE <id>, repo /Users/sac/cgap (branch main, ONE canonical checkout).
    NO git commands — the coordinator owns all git transitions.
    Write ONLY your owned files; everything else read-only. Absolute
    paths everywhere; cwd resets between Bash calls.
    Receipt: docs/jira/v26.11.2/lanes/lane-<id>-<slug>.md with command+exit
    evidence per claim. Honest standing vocabulary: ALIVE / PARTIAL /
    BLOCKED / UNVERIFIED / REFUSED — never round up; ALIVE requires a
    command+exit run this session against the exact subject.
    Anti-vacuity mandatory: every court names its anti-vacuity mutation
    and has a witnessed flip; a court that cannot flip is FAIL; skips are
    loud ([SKIP] <reason>). Compile-abort is not a failure count.
    Long-file law: write in <50-line chunks or python-assemble; verify
    on-disk bytes before building.
    Lease law: CARGO_TARGET_DIR=/tmp/<id>-target, unique per lane,
    deleted at lane end (python shutil.rmtree; rm -rf is denied).
    Chicago courts: real collaborators, assert final state, no mocks.
    Sandbox mutation law: mutate /tmp copies, never the shared tree;
    sha256-verified restores; bump mtime / cargo clean after restore.
    Disclosure duties: an SLA unblock to another lane's file is disclosed
    in BOTH lanes' receipts; the owner supersedes the diff on landing.
    Heartbeat law (TC-11): while in flight, touch
    docs/jira/v26.11.2/lanes/<lane-id>.alive on every meaningful
    progress step; a heartbeat older than 45 min triggers the doctor
    c10-dispatch-ledger WARN. Before the receipt lands, append ONE row
    to docs/jira/v26.11.2/lanes/dispatch-ledger.jsonl:
    {"lane": id, "ts": epoch, "fleet": fleet-prefix,
    "premise_hash": sha256-of-dispatch-premise}. The c10 check reads
    this ledger: stale row with no receipt = WARN "silent death".
    Report (JSON terminus): {"lane":..., "files_written":[],
    "commands":[], "courts_run":[], "standing":"", "divergences":[]}

## Owned-files manifest

| # | Path (absolute) | Disposition (edit/new) | Why this lane owns it |
|---|---|---|---|
| 1 | | | |
| 2 | | | |

Disjoint-ownership check: name the sibling lanes sharing the module and
confirm zero path overlap.

## Courts table

| Court | Mutation (anti-vacuity) | Flip witness (command+exit) | Exit |
|---|---|---|---|
| | | | |

A court with no witnessed flip is FAIL, not pass. A skip is loud.

## Receipt skeleton (W7 fields)

1. **Identity** — lane id, exact subject (path/branch/SHA).
2. **Authority** — dispatch prompt + owned-files manifest.
3. **Commands+exits** — every gate/court/build command and its exit code.
4. **Consequence** — the diff summary; fences that could regress and
   their run results.
5. **Courts** — the courts table above, filled.
6. **Standing** — honest vocabulary, never rounded up.
7. **Divergences/disclosures** — doc-vs-reality notes; SLA unblocks
   disclosed in both directions.
