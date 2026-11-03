# Lane I-14 — flag-figure adjacent-surface refresh (2026-11-02)

Scope: repo-root/doc index surfaces EXCEPT CLAUDE.md (coordinator-owned).
Truth source (read + verified this session):
`audit/pub-02-flag-table-rederivation.md` (23/23 explicit, 20 true / 3 false,
19/20 registry-exist, branchless 404-pending, bench permanent collision) and
`01-release-gates.md:325-351` refreshed member table (PUB-02b landed).

## Edits (6 sites, 4 files)

1. `00-ard-prd.md:70-77` — G1-PUBLISH "19/22 publishable set" -> "20/23
   publishable set"; scope note extended with PUB-02b supersession cite.
2. `OVERNIGHT-LOOP.md:44-49` — §2-A publish state rewritten to PUB-02b figures
   (19/20 registry-exist, branchless + integration 404-pending) with cites.
3. `OVERNIGHT-LOOP.md:124-126` — G1-PUBLISH step -> "20/23 publishable set
   (bench/beam/court publish = false per PUB-02b)".
4. `OVERNIGHT-LOOP.md:196-198` — RESTART-NOW pointer note -> PUB-02b-current.
5. `HANDOFF.md:27-35` (§1 F3) — "19/22 is current" -> superseded-by-PUB-02b note.
6. `HANDOFF.md:56-63` (D-2) — divergence register row updated with PUB-02b cite.

All edits verified on disk via grep (lines cited above).

## Checked, unedited

- `README.md` — carries NO publish-count figure ("12/23"/"19/22"/"9 true"/
  "publishable": zero matches). Lists harness crates under "never published";
  includes cgap-audit, which PUB-02b shows publish=true/registry-exists —
  flagged as an observation for the coordinator; README verified unedited
  by RW-02, so left untouched (not a count figure).
- `01-release-gates.md` — already refreshed by PUB-02b; untouched.
- `08-registry-sweep.md:68-76` — already carries the PUB-02b overlay; untouched.
- `OVERNIGHT-PROMPT.md:56` — "(OVERNIGHT-LOOP refresh (19/22))" is a historical
  action description of executed wave W1, not a current-state claim; left per
  ERRATA-never-rewrite; recorded as divergence.

## Fences

- Must-not-regress: the retained-verbatim stale text at
  `01-release-gates.md:315-323` (ERRATA law) was NOT touched; the guarded table
  cite `01-release-gates.md:28-32` retained in the PRD scope note.
- Docs-only diff; no build surface touched; zero git writes.

## Standing: ALIVE (docs-only; verification = grep on disk, exits 0)
