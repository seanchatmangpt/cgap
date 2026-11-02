# cgap

Sovereign `#![no_std]` columnar/deductive substrate; 22-crate root
workspace at 26.11.1, 18 crates on crates.io (`bench` there is a name
collision, not ours; `cgap-beam`/`cgap-integration`/`cgap-court`/
`cgap-verification` are registry-absent). cgap-datafusion was removed
(`docs/jira/v26.11.1/waves/cgap-datafusion-removal.md:6`) — do not
re-scaffold. Campaign: v26.11.2 starDF (`docs/jira/v26.11.2/
00-ard-prd.md`; runbook `OVERNIGHT-LOOP.md`, wave plan `05-wave-plan.md`).

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

Gate exit codes: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
A skip is loud, never a fake pass. A court that cannot flip is FAIL; a
mutation-free green gate run is VACUOUS (exit nonzero).

## Publish law (crates.io)

- ONE publish attempt per crate; sequenced post-window, never
  concurrent with lever lanes (`docs/jira/v26.11.2/OVERNIGHT-LOOP.md:146-149`).
- Registry slide (index reject or timeout): back off 1 hour. No retry
  storms.
- `publish = false`: bench (registry name collision), beam, court
  (structural [[test]]-only) — 19/22 published, 3 guarded
  table at `docs/jira/v26.11.2/01-release-gates.md:28-32`.

## CLI (ggen six-pack generated)

`crates/cli` is a STANDALONE ggen crate (own `[workspace]`, NOT a root
member). Consumer seams: `crates/cli/src/custom_handlers.rs` (the
cnv:CustomBehavior hole, filled once) and `crates/cli/manifest_patch.py`
(post-sync kernel deps + `[patch.crates-io]` to `~/clap-noun-verb`
26.9.1). Side-bins `doctor`/`status`/`verify_v26` are hand-maintained.

## Layout (pointers only — read the tree for detail)

- `crates/cgap-{arrow,blake3,git,parquet}` — published zero-dep kernels
- `crates/{cgap-core,cgap-storage,cgap-bont,cgap-bosym,cgap-bn3,
  cgap-gate,cgap-crypto,cgap-p28}` — moonshot lane crates
- `crates/{cgap-ffi,cgap-beam,cgap-gpu,audit,integration,cgap-court}` — membranes, court harness
- `crates/cgap-verification` — on disk, NOT in the 22-member `cargo metadata` list
- `crates/cli` — standalone ggen CLI (above); `ontologies/{fibo,cloud}` — O-ABI packs
- `docs/jira/v26.11.1/{lanes,waves}/` — A01-A20 specs + 152 receipts
  (`docs/jira/v26.11.2/waves/v2-07-census.md:9`)
- `docs/jira/v26.11.2/` — starDF campaign (00..05, OVERNIGHT-LOOP.md, waves/)
- `scripts/boundary_gate.py`, `scripts/zero_string_gate.py` — fail-closed gates

## Generated artifacts (never hand-edit; regenerate via the graph)

| File | Regenerate |
|---|---|
| `crates/cli/src/{main,lib,generated_cli,generated_behavior,generated_boundaries}.rs`, `tests/zero_code_cli.rs`, `docs/command-reference.md`, `ggen.lock` | `cd crates/cli && rm Cargo.toml && ggen sync run && python3 manifest_patch.py` |
<!-- row above intentionally exceeds 100 chars: verbatim command, do not wrap -->
| `crates/cli/Cargo.toml` | crate-pack output, then patched by `manifest_patch.py` |
| `ontology/cgap.ttl` projections | repo-root `ggen sync run` (`gen-templates/` consumer extras) |

## Conventions

- Standing vocabulary: ALIVE requires a command + exit run this session
  against the exact subject; UNVERIFIED never rounds up; perf numbers
  are TARGETs until measured with a stated method.
- Two seeds never conflate: SplitMix64 multiplier `0xBF58_476D_1CE4_E5B9`
  vs corpus seed `0xA20C_0FFE_E5A7_E000` — both in
  `crates/cgap-parquet/src/bitslice.rs:687-699`; swapping them breaks
  byte-identical replay.
- Frozen byte-pins, never sweep: license URN
  `urn:avec:license:p28-sil:v2.1.0-sovereign`; v26.10.31 SHACL IRIs
  (`avec:` -> `https://avec.chatman.dev/`).
- `git commit -F <file>`, never inline `-m`; no rebase, no force-push.
- Chicago courts: real collaborators, assert final state, no mocks;
  every court names its anti-vacuity mutation; mutate /tmp sandbox
  copies with sha256 fail-closed restore, never the shared tree.
- mtime-bump restore law: byte-exact restore with pre-mutation mtimes
  does NOT invalidate cargo's fingerprint — the mutated binary
  survives (`docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184`);
  bump mtimes or `cargo clean` between mutation and restore.
- cwd resets to repo root between Bash calls — absolute paths only.
- No Process.sleep: bounded polls on observables; the runtime owns time.

## Agents

Only `.claude/agents/` agents (lane-runner, court-auditor, gate-runner)
plus read-only Explore/Plan — the PreToolUse hook refuses the built-in
generics. Subagent prompts carry the absolute repo path + contract block.

## Agent fan-outs

- Never `git add -A` — stage explicit paths; `target-*/` build roots are
  lease debris (`.gitignore` is the backstop, not the mechanism).
- No stop/resume mid-wave (kills in-flight work); one `resumeFromRunId`
  pass at completion. Post-run truth is `journal.jsonl`.
- Agents corrupt long-file writes (~1 in 5 over ~100 lines): write in
  <50-line chunks or assemble via python; verify on-disk bytes.

## Bash (zsh, macOS, BSD)

- `echo ===` breaks zsh ("= not found" glob); quote separators.
- BSD xargs has no `-a`; use `while read -r; do ...; done < file`.
- `$?` after a pipe is the last command's rc — check
  `git rev-parse origin/main`, don't trust `git push | tail`.
- `sed -i ''` (empty arg) on macOS.
- Slow push: check payload (`git diff-tree --stat <sha> | tail -1`),
  background with a long window, never kill mid-pack.
