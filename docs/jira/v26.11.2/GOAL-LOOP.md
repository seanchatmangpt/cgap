# GOAL-LOOP — ultracode publish loop until all cgap crates at v26.11.2

Operator-approved 2026-11-01. Folds the W6 publish wave into a repeatable goal loop with hard
stop conditions. Executes after W5's registry sweep (OVERNIGHT-PROMPT.md sequencing), or as a
standalone loop whenever the entry gates hold.

## Terminal condition (the loop stops ONLY here, with a receipt)

Every workspace crate carries a final per-crate standing ∈ {PUBLISHED-26.11.2 (live crates.io
num_versions verified this session), LAWFUL-NON-PUBLISH (bench/beam/court/cli — delisted with
court pointer), BLOCKED(coords)} — 22 targets, not 22 forced successes. Silence is not a
terminus. Loop until scripts/registry_sweep.py exits 0 with every cell PASS or
REGISTERED-DIVERGENCE, or three full waves return zero NEW state changes — then stop with the
typed open set.

## Entry gates (each wave re-checks before any publish attempt)

1. Workspace version = 26.11.2 in root Cargo.toml; release_check.py pin updated to match.
2. cargo build --workspace exit 0; cargo test -p cgap-court exit 0 (G8).
3. Dependency-ordered publish matrix via cargo metadata topological sort: for each
   publishable crate, `cargo publish --dry-run -p <crate> --allow-dirty` exit 0 = resolution
   gate passed. Crates.io metadata pre-check per crate (description/license — the 400 class).
4. Gate board green: zero_datafusion_gate, registry_sweep, boundary/zero-string gates, doctor.

## Publish law (non-negotiable)

- ONE cargo publish invocation per crate per attempt-budget; version created = budget
  consumed.
- dry-run exit 0 (deps live) is the resolution gate before every attempt — never attempt
  upstream of an unpublished dependency.
- 429 → read Retry-After, honor it, record BLOCKED with command+exit+timestamp.
  STOP-ON-SLIDE law: a 429 slides the rolling new-crate window — wait the FULL window, never
  re-arm early. One re-sequenced attempt after the window; second slide → BLOCKED, next wave.
- Never concurrent with lever/bench lanes; publishes are the only actuation lane while
  active.
- Subagent refusal on a publish = REFUSED leg; coordinator runs it
  (coord-publish-record.md precedent). Immutability: num_versions > expected on any crate =
  CRITICAL finding, record, never mutate.

## Wave shape (repeat until terminal condition)

20 agents per wave, ONE message:

- 1 sequencer (the single publish actuator)
- 8 crates.io watchers (bounded polls ~60s, 90-min deadline, append-only witness file,
  disjoint line ranges, num_versions exactly-once per crate)
- 2 sweep/witness agents (post-publish sweep re-run + 08-registry-sweep.md cell updates)
- 1 consumer-proof agent (/tmp consumer crate, registry-only build + one kernel call per
  published crate + flip courts)
- 1 README reconciliation agent (archive-before-overwrite + sha256)
- 1 ledger auditor (one-attempt-law compliance; pre-mint refusals disclosed ≠ violations)
- 1 CLI regen agent (byte-identity receipt if any manifest changed)
- 1 hygiene agent (HEAD census, credential sweep, /tmp lease cleanup via shutil.rmtree)
- 4 repair agents (dry-run failures from the 400/metadata class fixed in manifests)
- 1 terminus gate-runner.

Terminus each wave: full 22-row registry table (crate | registry state | versions | witness),
changed-set vs prior wave, BLOCKED list with coordinates →
waves/terminal-<wave#>-terminus.md, JSON footer.

## Shared contract

/Users/sac/cgap one canonical checkout, coordinator owns ALL git transitions (agents never
git commit/push/publish without the sequencer lane); /Users/sac/stardf ABSOLUTELY BANNED per
operator order 2026-11-01; zero DataFusion; frozen pins untouchable (license URN, v26.10.31
SHACL IRIs, both seeds); Chicago courts (flip-or-FAIL, /tmp sandboxes, mtime-bump restore);
exit contract 0/1/2/3; standing vocabulary ALIVE/PARTIAL/BLOCKED/REFUSED/UNSUPPORTED/
UNVERIFIED (UNVERIFIED never rounds up); no Process.sleep (bounded polls); <50-line writes
with token-sweep; receipts in docs/jira/v26.11.2/waves/; no worktrees; 429/timeout on an
agent = resume in place via SendMessage, never relaunch a lane with receipts on disk.

## Standing (re-read 2026-11-02, lane RW-04 sweep)

Registry state is FROZEN-BY-OPERATOR (no publish attempts authorized;
`waves/publish-halt.md:1-5`). Cell status per crate, live-verified
via crates.io API (UA cgap-halt-check) in `waves/publish-halt.md`:

- PUBLISHED-26.11.2 (8): cgap-blake3, cgap-arrow, cgap-git,
  cgap-parquet, cgap (umbrella) — 3 versions each; cgap-core,
  cgap-storage, cgap-bont — 2 versions each (`publish-halt.md:6-17`).
- READY on lift, dry-run exit 0 (7): cgap-crypto, cgap-p28, cgap-gpu,
  cgap-audit, cgap-fibo, cgap-cloud, cgap-integration
  (`publish-halt.md:31-36`).
- BLOCKED (2): cgap-ffi — registry cgap-parquet 26.11.2 predates
  RingSink, needs parquet re-publish first; wasm-footprint —
  host-link failure in tarball verify, diagnosis owed
  (`publish-halt.md:37-40`).
- Repairs owed on lift, already-live (3): cgap-blake3 26.11.3 (NEON
  opt>=1 divergence), cgap-arrow 26.11.3 (stale embedded
  generated_version), cgap-parquet 26.11.3 (RingSink)
  (`publish-halt.md:39-42`).

Terminal condition NOT reached: the 22-row table still contains
BLOCKED and repair cells; waves remain pending the operator lift.

## Standing refresh (2026-11-02, lane GOAL-01)

RW-04's block above stands unmodified (ERRATA-never-rewrite on its
cites). This block refreshes the terminal-condition state; counts
are re-read from disk this session and remain stale by default
until re-verified at use time (prepublish-state §6.9).

Terminal condition is CLOSE but not reached. The publish window
preconditions are now receipt-complete EXCEPT two operator/coordinator
legs:

1. FREEZE LIFT: still FROZEN-BY-OPERATOR
   (`waves/publish-halt.md:1-5`; re-read this session — the halt
   doc is unchanged on disk). Publish-mode commands stay REFUSED
   until the operator edits the guard hook
   (`audit/prepublish-state.md:109-131`).
2. COMMIT LANDING: wave diffs still uncommitted in the shared
   tree; a window runs clean-tree before any registry act
   (`audit/prepublish-state.md` §2; runbook precondition 1-2,
   `dod/PUBLISH_RUNBOOK_v26.11.2.md` §1).

Receipt-complete preconditions now on disk:

- Publish matrix: PUB-01, 16/20 package-OK, 4 BLOCKED-DEP-ORDER
  (cgap, cgap-gate, cgap-ffi, cgap-integration), 3
  SKIP-PUBLISH-FALSE (`audit/pub-01-dry-run-matrix.md:44-74`).
- Flag truth: 23/23 explicit, 20 true / 3 false
  (`audit/pub-02-flag-table-rederivation.md:27-51`; supersedes
  the CLAUDE.md 12/23 line).
- Sequence + laws + defects G-1/U-1/I-1:
  `audit/pub-03-publish-sequence.md`.
- Operator command sequence, tier-ordered:
  `dod/PUBLISH_RUNBOOK_v26.11.2.md` (FROZEN-BY-OPERATOR; a plan,
  not an authorization).
- Full-repo reconciliation GREEN: workspace build + per-crate
  tests + gate battery 4/4 and self-check 8/8 + riscv 101/0
  designed-flip pair + CLI 44 tests (`audit/
  prepublish-reconciliation.md` — C-10, ALIVE, zero repairs).

Last remaining gate before a window can be declared terminal-ready:
the DoD certificate fold. The dispatch named this "FS-18"; no
on-disk referent carries that ID anywhere under
`docs/jira/v26.11.2/` (grep'd this session, zero hits). The
nearest landed carriers are `audit/dod-18-ledger-fold.md` (143
lane-dir entries folded, standing-as-receipted) and the DOD-19
ship-gate table spec (`audit/dod-19-shipgate-final.md` — a SPEC
whose exact cell texts await the coordinator's table-update
commit). Divergence recorded; the coordinator should name the
certificate artifact explicitly in the next dispatch.

Non-vacuity: this refresh block carries zero new command exits —
it is a merge-only state doc grounded in receipts re-read this
session (full reads: GOAL-LOOP.md, 05-wave-plan.md Standing,
prepublish-state.md, pub-01 matrix, prepublish-reconciliation.md,
PUBLISH_RUNBOOK §1-2). Per the metagate above, a state refresh is
not a wave; the wave terminus still owes the live 22-row registry
table, which remains BLOCKED behind the freeze.

PROMO-01 pointer (appended on coordinator follow-up, same lane):
the microbenchmark promotions this block did NOT cover have now
LANDED — claims §6.1 rows 39/40 (82.24 GB/s BN3 statement_scan_24b,
~460 ns P28 attestation verify; consolidation-addendum, doubly
witnessed F-06+FS-02) are registered in `11-measured-claims.md`
and receipted at `lanes/lane-promo01-claims-promotion.md`. This
supersedes any PROMO-01 absence recorded in earlier GOAL-01
blocks.

## Non-vacuity metagate

A wave with zero witnessed command+exit citations is VACUOUS — terminus fails it (exit
nonzero) and names the lanes. The loop is done when the registry table stops changing toward
the terminal condition, not when agents run out of things to do.
