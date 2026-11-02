# The Receipt Economy

Why work in cgap is admitted by courts and recorded as receipts, and
why a green gate run is not automatically worth anything.

## Courts, not reviewers

A court is a mechanical admission instrument: real collaborators, real
final state asserted, no mocks (Chicago discipline, repo CLAUDE.md
"Conventions"). Its power comes from its anti-vacuity mutation: every
court names the mutation that must flip it. "A court that cannot flip
is FAIL" (CLAUDE.md exit-code contract; A19 §"Header court
(anti-vacuity: a court that cannot flip is FAIL)"); "a mutation-free
green gate run is VACUOUS (exit nonzero)."

The reasoning is adversarial: a test suite that only ever agrees with
the code verifies the test author's model, not the code. The mutation
is the falsifier — if the court still passes with the mechanism
corrupted, the court was decoration. `waves/r14-pin-final.md` is the
pattern at scale: each pin swept, verdict recorded, divergence
registered.

## The standing vocabulary

Claims use a fixed vocabulary — ALIVE / PARTIAL / BLOCKED / REFUSED /
UNVERIFIED / UNSUPPORTED — where ALIVE requires command + exit run
this session on the exact subject. UNVERIFIED is the default and is
never rounded up to ALIVE. The vocabulary exists because receipts
outlive sessions: a receipt that says "works" is unreadable later, a
receipt that says "ALIVE: cmd X, exit 0, sha Y" replays.

## The append-only trail

The `docs/jira/v26.11.1/waves/` directory is the economy's ledger:
one receipt per landed unit, append-only, each receipt carrying
subject, commands/exits, courts run, and standing. The campaign ledger
(`waves/v26111-campaign-ledger.md`) cross-indexes them with
floor readings (projection coverage, machinery share) so the frontier
reads recorded truth, not session memory.

## Why append-only

Receipts are evidence, and evidence is not revised — it is superseded
by a newer receipt. A correction lands as a re-audit receipt (the
`a17-wave2-re-audit.md` pattern), never an edit of the old one. This
is what makes the trail replayable: any later lane can re-read the
chain and reconstruct what was known, and when, without trusting
anyone's recollection.
