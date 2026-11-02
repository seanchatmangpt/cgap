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

## Non-vacuity metagate

A wave with zero witnessed command+exit citations is VACUOUS — terminus fails it (exit
nonzero) and names the lanes. The loop is done when the registry table stops changing toward
the terminal condition, not when agents run out of things to do.
