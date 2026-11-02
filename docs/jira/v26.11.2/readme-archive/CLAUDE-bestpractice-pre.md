# cgap

Sovereign `#![no_std]` columnar/deductive substrate: Apache Arrow C Data
Interface kernels, BLAKE3, Parquet persistence, Git packfile v2, binary
ontology (B-ONT) / binary symbolics (BOSYM) / binary N3 deductive layers,
and P28-SIL licensing — zero-external-dep kernel crates behind a `cgap`
umbrella facade. 22-crate root workspace, every member at 26.11.1; 18
crates published to crates.io at 26.11.1 (`cgap-beam`, `cgap-integration`,
`cgap-court`, `cgap-verification` are not on the registry; `bench` on
crates.io is a name collision, not this repo's bench crate). The
cgap-datafusion spike was removed
(`docs/jira/v26.11.1/waves/cgap-datafusion-removal.md:6`). Renamed
avec -> cgap 2026-10-31. Current campaign: v26.11.2 starDF closure —
charter `docs/jira/v26.11.2/00-ard-prd.md`, governing runbook
`docs/jira/v26.11.2/OVERNIGHT-LOOP.md`, wave plan `05-wave-plan.md`.

## Agents

Never dispatch the built-in generic agents (`general-purpose`, `claude`) —
the PreToolUse hook refuses them. Only use agents defined in
`.claude/agents/` (lane-runner, court-auditor, gate-runner) plus read-only
Explore/Plan. Subagent prompts always carry the absolute repo path
(`/Users/sac/cgap`) and the contract block (file ownership, no git writes,
Chicago courts, status vocabulary).

## Commands

```bash
cargo build --workspace                 # root workspace must stay green
cargo test -p cgap-court                # court_verdict harness
cd crates/cli && cargo build && cargo test   # standalone ggen CLI (route proofs + mutation_kills RSS courts)
just doctor                             # 10 checks; YELLOW = informational (exit 1 by design)
python3 scripts/boundary_gate.py        # campaign boundary gate (fail-closed)
python3 scripts/zero_string_gate.py     # zero-string gate (--self-check = non-vacuity proof)
just doctor/status via justfile         # side-bins run with --manifest-path crates/cli/Cargo.toml
cd crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py   # CLI regen (byte-identical replay)
```

Exit-code contract for gates: 0 pass / 1 gate fail / 2 subject missing /
3 loud skip. A skip is loud, never a fake pass. A court that cannot flip
is FAIL; a mutation-free green gate run is VACUOUS (exit nonzero).

## Publish law (crates.io)

- ONE publish attempt per crate; sequenced post-window (after the
  overnight loop closes, never concurrently with lever lanes)
  — `docs/jira/v26.11.2/OVERNIGHT-LOOP.md:128`.
- On a registry slide (index reject or timeout): stop and back off
  1 hour before the next attempt. No retry storms.
- Delisted from publish (`publish = false`): audit, integration, and
  beam-optional set — see the 19/22 publishable table in
  `docs/jira/v26.11.2/01-release-gates.md:28-32`.

## CLI (ggen six-pack generated)

`crates/cli` is a STANDALONE ggen crate (own `[workspace]`, NOT a root
workspace member). Binary `cgap`, noun-verb + JSON output (`--format`
supported): `cgap self version`, `cgap bench mask`, `cgap bench parquet`,
`cgap align test`. Graph: `crates/cli/ontology/cgap-cli.ttl`; packs pinned
in `crates/cli/ggen.lock`. Consumer seams: `custom_handlers.rs`
(the cnv:CustomBehavior hole, filled once) and `manifest_patch.py`
(post-sync kernel deps + `[patch.crates-io]` to `~/clap-noun-verb` 26.9.1).
Side-bins `doctor`/`status`/`verify_v26` are hand-maintained.

## Layout

- `crates/cgap-{arrow,blake3,git,parquet}` — published kernels (zero-dep,
  no_std)
- `crates/{cgap-core,cgap-storage,cgap-bont,cgap-bosym,cgap-bn3,cgap-gate,cgap-crypto,cgap-p28}`
  — moonshot lane crates (PROPOSED until their courts land)
- `crates/{cgap-ffi,cgap-beam,cgap-gpu,audit,integration,cgap-court}` —
  membranes, audit substrate, court harness
- `crates/cgap-verification` — deterministic verification surface
  (triple ingestion, integrity rules, tamper-evident sealing); present
  on disk, standalone-build workspace pin, root membership pending —
  NOT in the 22-member `cargo metadata` list
- `crates/cli` — standalone ggen-generated CLI (see above)
- `ontologies/{fibo,cloud}` — O-ABI packs (lanes A09/A10)
- `docs/jira/v26.11.1/{lanes,waves}/` — lane specs (A01-A20) + 152
  execution receipts (`docs/jira/v26.11.2/waves/v2-07-census.md:9`)
- `docs/jira/v26.11.2/` — starDF campaign: 00-ard-prd, 01-release-gates,
  02-pareto-levers, 03-ost-design-around-register, 04-stardf-closure,
  05-wave-plan, OVERNIGHT-LOOP.md, waves/ (9 receipts)
- `scripts/` — boundary_gate.py, zero_string_gate.py (fail-closed)
- cgap-datafusion: REMOVED
  (`docs/jira/v26.11.1/waves/cgap-datafusion-removal.md:6`); do not
  re-scaffold, superseding receipts noted in
  `docs/jira/v26.11.2/waves/v2-07-census.md:77-79`

## Generated artifacts (never hand-edit; regenerate via the graph)

| File | Regenerate |
|---|---|
| `crates/cli/src/{main,lib,generated_cli,generated_behavior,generated_boundaries}.rs`, `tests/zero_code_cli.rs`, `docs/command-reference.md`, `ggen.lock` | `cd crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py` |
<!-- row above intentionally exceeds 100 chars: verbatim command, do not wrap -->
| `crates/cli/Cargo.toml` | generated by crate-pack, then patched by `manifest_patch.py` (kernel deps + `[patch.crates-io]`) |
| `ontology/cgap.ttl` projections | repo-root `ggen sync run` (`gen-templates/` consumer extras) |

## Conventions

- Standing vocabulary (ALIVE / PARTIAL / BLOCKED / REFUSED /
  UNSUPPORTED / UNVERIFIED): ALIVE requires a command + exit run this
  session against the exact subject; UNVERIFIED stays UNVERIFIED, never
  rounds up; perf numbers are TARGETs until measured with a stated
  method.
- THE SEED DISTINCTION — two 64-bit constants that must never be
  conflated: the SplitMix64 multiplier
  `0xBF58_476D_1CE4_E5B9` (RNG state advance) vs the corpus seed
  `0xA20C_0FFE_E5A7_E000` (stream spread / finalizer output). Both live
  in the canonical court RNG,
  `crates/cgap-parquet/src/bitslice.rs:687-699` — identical stream to
  the bench corpus; a court using one where the other belongs breaks
  byte-identical replay.
- Frozen byte-pins — never sweep: license URN
  `urn:avec:license:p28-sil:v2.1.0-sovereign`, v26.10.31 SHACL corpus
  IRIs (`avec:` prefix -> `https://avec.chatman.dev/`).
- `git commit -F <file>`, never inline `-m`; no rebase, no force-push.
- Chicago courts: real collaborators, assert final state, no mocks;
  every court names its anti-vacuity mutation; courts mutate /tmp
  sandbox copies with sha256 fail-closed restore, never the shared
  tree.
- mtime-bump restore law: a byte-exact restore that preserves
  pre-mutation mtimes does NOT make cargo forget the mutated artifact
  — its fingerprint keyed on mtime kept the mutated binary
  (`docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184`). Mutation
  legs must bump mtimes or clean `target/` between mutation and
  restore.
- cwd-reset sandbox hazard: Bash cwd resets to repo root between tool
  calls — a relative-path /tmp sandbox step silently lands in the repo
  or the wrong dir; use absolute paths everywhere (see Bash section).
- No Process.sleep: bounded polls on observables; the runtime owns
  time.

## Agent fan-outs / 20-lane workflows

- Never `git add -A` — stage explicit paths only; lane build roots
  (`target-*/`) are lease debris and `.gitignore` is the backstop, not
  the mechanism.
- Budget for API-429 attrition (~25% of agents at high concurrency).
  Do NOT stop/resume mid-wave (kills in-flight work); run ONE
  `resumeFromRunId` pass at completion — cached calls replay, failures
  re-run at low concurrency.
- Post-run truth is the workflow `journal.jsonl` (one result line per
  agent), not the truncated notification.
- Agents corrupt long-file writes (~1 in 5 over ~100 lines): write in
  <50-line chunks or assemble via python; token-sweep before build;
  courts verify on-disk bytes, not intent.

## Bash (zsh, macOS, BSD)

- cwd resets to repo root between Bash calls — use absolute paths.
- `echo ===` breaks zsh ("= not found" glob); quote separators.
- BSD xargs has no `-a`; use `while read -r; do ... ; done < file`.
- `$?` after a pipe is the LAST command's rc (`git push | tail` reports
  tail's 0) — check `git rev-parse origin/main` instead.
- `sed -i ''` (empty arg) on macOS.
- Slow push = check payload first (`git diff-tree --stat <sha> | tail -1`),
  then background it with a long window; do not kill mid-pack.
