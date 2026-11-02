# R10 — ggen CI/publish mechanics (Explore brief)

Lane: R10. Sources: `/Users/sac/ggen/.github/workflows/`, `/Users/sac/ggen/scripts/ci/`,
`/Users/sac/ggen/cicd.toml`. Read-only Explore lane; claims cited file:line. All claims are
about ggen, re-read at use time before landing (stale by default).

## Key finding for cgap v26.11.2

**No crates.io `cargo publish` automation exists anywhere in ggen's release chain.** ggen's
"publish law" is guard-then-tag + read-only post-publish detection; cgap's per-crate sequenced
`cargo publish` with backoff (CLAUDE.md publish law) is a NEW mechanism with no ggen precedent.
Citable ggen pattern: one sequenced attempt, detect-only remediation, human-authorized
re-publish.

## 1. Release chain order

semantic-release → tag → release.yml → dispatched sub-targets → partial-publish detection.

1. `semantic-release.yml` (main push): Conventional Commits → CalVer `YY.M.patch` (NOT
   SemVer; sanity gate refuses month >12: `.github/workflows/semantic-release.yml:110-115`).
   Bumps 4 version-law files in lockstep (`:210-216`: Cargo.toml,
   `.specify/repo-facts.ttl` `rf:workspaceVersion`, `ggen.toml`,
   `crates/ggen-engine/Cargo.toml`), propagates via `ggen sync run` (`:240-241`), bumps via
   force-pushed ephemeral branch + squash PR (`:243-301`), tags/releases via softprops
   (`:306-315`), then **explicitly dispatches release.yml** because GITHUB_TOKEN tag pushes
   suppress `on: push` triggers (`:317-339`).
2. `publish-candidate.yml` (tags-only; never uploads assets, `:10-12`): runs all release
   gates (`:45-90`): `just verify-tcps` (`:45-46`) → docs-sync drift gate excluding
   by-design-churn evidence paths (`:48-73`) → book gates (`:75-79`) →
   `guard-pack-proofs` (`:80-81`) → `guard-claims-schema` (`:83-84`) →
   **`just guard-publish-standing`** full/strict (`:86-87`) →
   `guard-version-consistency.sh` (`:89-90`); then creates/pushes `v$VERSION` tag from
   Cargo.toml idempotently (`:92-106`).
3. `release.yml` (tag or dispatch): release-health job (`:36-205`) runs `cargo cicd
   workspace doctor` + `status show` + `target show` + `test changed` (`:127-131`) and a wpm
   process-conformance oracle gate with TRUTHFUL/VARIANCE/DECEPTIVE verdicts; DECEPTIVE = hard
   fail (`:166-205`). Then 4-target binary matrix build+upload (`:207-287`), then explicitly
   dispatches `homebrew-release.yml` (`:300-314`) and `release-debian.yml` (`:325-338`).
4. `detect-partial-publish.yml`: `workflow_run` fan-in on all 4 legs (`:66-72`), read-only
   (`permissions: contents: read`, `:83-84`), calls `scripts/ci/check-partial-publish.sh`,
   tag resolution with fallback (`:116-129`). Detection-only — auto-rollback explicitly out
   of scope (`detect-partial-publish.yml:49-50`, `check-partial-publish.sh:18-22`); the
   script prints human runbook `gh workflow run` commands per failed leg (`:31-45`).

## 2. Crate registry checks

- `guard-publish-target.sh` invariants: `cargo publish --dry-run -p ggen` must succeed
  (`:25-28`); no non-root workspace member named `ggen` via `cargo metadata` (`:38-46`);
  vendored engine crates (`ggen-engine`, `praxis-core`, `praxis-graphlaw`) carry
  `publish = false` (`:48-54`); exactly one bin named `ggen`, owned by `ggen-cli-lib`
  (`:76-94`).
- `guard-publish-standing.sh`: gates real `cargo publish` on `docs/aps/claims.toml` — full
  mode **executes every claim's `falsifier` command** (timeout `GUARD_FALSIFIER_TIMEOUT_SECS`,
  default 900s, `:100`); a standing=ALIVE claim whose falsifier fails now is a hard error
  (`:131-136`); publish-gated BLOCKED claims need `exception_admitted_by` (`:77-83`); stale
  evidence-commit coordinates warn-only (`:86-90`). Vocabulary ALIVE/PARTIAL/BLOCKED/UNVERIFIED
  (`:21`). Schema-only mode (pre-commit) never executes falsifiers (`:5-10`).
- `guard-version-consistency.sh`: three version sources must agree (`.specify/repo-facts.ttl`,
  Cargo.toml `[workspace.package]`, `ggen.toml`) (`:1-10` doc, `:35-41` drift check).

## 3. cicd.toml

`/Users/sac/ggen/cicd.toml` (35 lines): toolchain pin `nightly-2026-06-22` (`:3`);
`[state]`/`[target]` size/prune limits (`:6-15`); `[test.changed]`/`[trybuild.changed]`
changed-only vs `origin/main` (`:17-23`); `[git.phase] require_clean_tree = true` (`:26`);
`[autonomic] mode = "suggest"` (`:29-31`). Consumed by `cargo cicd` in release.yml:127-131.

## 4. Reproducibility

release.yml `build-release-repro` + `verify-cross-host-reproducibility` (`:340-542`): every
target built twice (replica a/b) on independent VMs, deliberately no cache (`:350-356`),
hashes travel by artifact not job outputs (`:355-357`), sha256 comparison fails loudly
(`:516-542`).

## 5. GITHUB_TOKEN suppression table

| Trigger | Status | Workaround | Cite |
|---|---|---|---|
| release.yml `push: tags` | suppressed | explicit dispatch from semantic-release.yml:326-339 | release.yml:7-14 |
| release-debian.yml `push: tags` | suppressed | dispatch from release.yml:325-338 | release.yml:316-324 |
| homebrew-release.yml `release: published` | suppressed | dispatch from release.yml:300-314 | release.yml:289-299 |
| docker-build-push.yml `push: tags` | suppressed (PR #441) | workflow_run fan-in | detect-partial-publish.yml:17-28 |

## 6. Conventions to carry into cgap publish-law doc

- Explicit dispatch whenever a step creates a tag/release with GITHUB_TOKEN.
- `cancel-in-progress: false` on every publish/release workflow (semantic-release.yml:19-21,
  publish-candidate.yml:16-18, release.yml:31-33).
- One attempt, no automated retry/backoff: failures detected read-only, human remediation via
  printed runbook commands (`detect-partial-publish.yml:48-50`).
- Sequenced gates BEFORE tag creation (publish-candidate.yml:45-90), then idempotent tag push.
- Version-law lockstep across 4 files + `ggen sync run` propagation
  (semantic-release.yml:210-241) + 3-way consistency guard.
- `deploy/` is monitoring/staging only, not in the release chain
  (`deploy/monitoring/MONITORING.md`, `deploy/staging/DEPLOYMENT_CHECKLIST.md`).
