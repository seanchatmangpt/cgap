# Add a lane

Add a work order to the v26.11.1 campaign: write a lane spec, dispatch it
with the contract block, close it with a receipt. Shape taken from
`docs/jira/v26.11.1/lanes/A06-bont-scalars.md` and the campaign's own
dispatches.

## Write the lane spec

`docs/jira/v26.11.1/lanes/<ID>-<slug>.md`:

```markdown
# <ID>: <title> — crate `<crate>` (PROPOSED)

- **Lane**: <ID> of 20, Squadron N, v26.11.1
- **Status**: PROPOSED
- **Files in scope**: exact paths, e.g. crates/cgap-x/src/thing.rs
- **Coordinator owns**: workspace membership, edits outside the lane
- **Read-only receipt basis**: files the lane may read but not write
```

Files in scope must be disjoint from every other live lane — the lease
partition is the concurrency mechanism.

## Dispatch with the contract block

Every subagent prompt carries: repo absolute path (`/Users/sac/cgap`), the
lease (exact file list, everything else read-only), no git writes
(coordinator owns all transitions), Chicago courts with a named
anti-vacuity mutation, and the status vocabulary
(ALIVE/PARTIAL/BLOCKED/REFUSED/UNSUPPORTED/UNVERIFIED).

## Close with a receipt

`docs/jira/v26.11.1/waves/<lane>-receipt.md`: lane id, files written,
commands with exits (`cmd; echo $?` captured directly — a piped `$?`
reports the pipe's tail, not the command), courts run with flip evidence,
standing, divergences. ALIVE requires a command run this session; UNVERIFIED
stays UNVERIFIED.

## Rules that recur

- One canonical checkout, no worktrees; lane build roots under `/tmp`,
  deleted post-run.
- Compile-freeze SLA ~10 min: fix or revert your own breakage; a minimal
  disclosed unblock fix to another lane's file is allowed.
- Never `git add -A`; explicit paths only.
- Compile-abort is not a failure count — census tooling must report the
  broken file, not a failed-test tally.
- Skips are loud: `[SKIP] <reason>`, exit 3 class; a silent skip is a
  defect.
