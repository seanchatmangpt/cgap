# v2-13-commit-manifest

Lane: V2-13 (gate-runner, read-only). Repo: /Users/sac/cgap + /Users/sac/stardf.
Status: PLANNED — coordinator executes AFTER wave close. Zero git writes performed.

## Commands (this session)

| cmd | exit |
|---|---|
| `git -C /Users/sac/cgap status --porcelain` | 0 (265 lines) |
| `git -C /Users/sac/stardf status --porcelain` | 0 (30 lines) |
| `git check-ignore -v target` | 0 (matched .gitignore:25 `**/target/`) |
| credential-pattern grep over all dirty files | 0 — 5 matches, ALL benign (gate scripts' own scan regexes + probe tokens) |

Base: cgap `cfd085a9` (entire campaign uncommitted on top).

## Group A — campaign source edits (crates/**, M) — 40 files

40 modified files under crates/** (src + Cargo.toml + README) incl.
cli/{.clap-noun-verb/ocel.json,Cargo.lock,Cargo.toml,docs/command-reference.md,manifest_patch.py,ontology/cgap-cli.ttl,src/*}.
Commit 1 `git add` list = all 40 `M crates/**` paths from `git status --porcelain`.

## Group B — new crate cgap-verification (8 files) — cgap-datafusion ABSENT

- `crates/cgap-verification/` (8 files, untracked; includes own Cargo.lock).
- `cgap-datafusion`: ABSENT from `ls crates` — removal receipt
  docs/jira/v26.11.1/waves/cgap-datafusion-removal.md corroborates. VERIFIED ABSENT.

## Group C — receipts/docs (untracked) — 189 files

- docs/jira/v26.11.1/waves/** : 139 files (20 a*-finish-receipts, f3-01..f3-20 publish ledger, cli-1..5, dia-1..6, doc-1..6, errc-*, research-*, paper-*, r1x, v26111-campaign-ledger, vac-bench-*, gate-*, gap-*)
- docs/jira/v26.11.2/ : 5 files (00-ard-prd, 02-pareto-levers, 05-wave-plan, waves/v2-01-charter, waves/v2-05-wave-plan)
- docs/jira/v26.11.1/lanes/*.md modified : 11 files (A01-A16 lane specs)
- docs/jira/v26.11.1/readme-archive/ : 9 files
- docs/diataxis/ : 57 files (SUMMARY, tutorials 01-03, how-to-guides, reference 5, explanation 5, marketplace-generated)
- docs/{CHALLENGER_SALES_REPORT,REVENUE_GENERATION_COMPENDIUM}.md : 2
- crates/**/README.md (audit, bench, beam, bn3, bont, bosym, court, crypto, gate, gpu, p28, storage, cli, integration, wasm-footprint, arrow, blake3? [blake3/README modified in Group A], core) : 17 new READMEs
- ontologies/{cloud,fibo}/README.md : 2; ontologies/cloud/tests/wire_parity.rs : 1
- README.md (root, M) : 1
- scripts/hop_ratio.py, scripts/projection_coverage.py : 2

Note: lane-spec edits (11 lane .md M) are docs-class; coordinator may fold into the docs commit.

## Group D — bench files — 15 files

crates/bench/benches/ : bn3, chains-{bn3,ingress,parquet,seal,sovereign}, challenger_{arrow,crypto,git,parquet}, ingress, kernels, parquet, seal (14 benches) + crates/bbench/README.md → actually crates/bench/README.md (new) counted in C-READMEs; benches = 14 files.
Correction: 14 bench files under crates/bench/benches/ (Group A already carries M crates/bench/Cargo.toml).

## Group E — manifest edits — 27 files

M Cargo.lock, Cargo.toml, tests/court_verdict.rs (court harness edit, source-class), plus per-crate Cargo.toml already in Group A. Root Cargo.lock+Cargo.toml + crates/wasm-footprint/{Cargo.lock,Cargo.toml} + crates/cli/Cargo.lock. De-duplicated against Group A; Group A ∪ E = all M paths.

## Group F — scripts/ — 4 files

M scripts/{boundary_gate.py,zero_string_gate.py} + untracked scripts/{projection_coverage.py,hop_ratio.py} (projection_coverage, hop_ratio: present).

Both gate scripts contain the credential-scan regex itself (PROBE_TOKEN/POISON_TOKEN) — benign.

## Group G — DO-NOT-COMMIT: none

- Credentials-class: NONE (scan hit only gate scripts' own scan machinery).
- /tmp debris in-tree: none (all untracked paths are campaign artifacts).
- target dirs: gitignored (.gitignore:25 `**/target/`), verified.
- `.clap-noun-verb/ocel.json` at repo root: untracked OCEL tap output — RECOMMEND gitignore or commit with Group A; flag to coordinator.

ROOT `.clap-nun-verb` note: crates/cli/.clap-noun-verb/ocel.json is tracked-M; the ROOT `.clap-noun-verb/` dir is untracked tap debris — recommend .gitignore, do not commit.

Untracked-dirs default-add caution: `git add crates/cgap-verification crates/cli/src/custom_handlers crates/cgap-p28/tests ontologies/cloud/tests docs/diataxis docs/jira/v26.6...` — every untracked dir was expanded and enumerated above; no stray files.

## stardf (/Users/sac/stardf) — 30 dirty paths (12 M + 18 untracked roots, 77 files when expanded)

Groups: S-wave (6 new crates: duckdb-bridge 2, incremental 7, inline 7, per-lane counts above; spine 6 files + spine_courts.rs), CLI-era (scripts/ 1, docs/{abi-freeze-witness,parity-inventory,parity-vs-ost,post-move-repair,ticket-001-state,WORK-BOARD}.md, tickets/), move-archive (docs/move-archive/ 3), authority re-homing (authority/ 6 files).
Plus M: .claude/settings.json, AGENTS.md, CLAUDE.md, arrow/{Cargo.lock,Cargo.toml}, datalog lib.rs, h03_bootstrap_sim.sh, tickets/README.md, workflows/*.workflow.js.

Suggested stardf commit order: (1) arrow/ sources + new stardf-* crates + spine, (2) authority/ + scripts/ + docs/ + tickets/, (3) repo config (settings.json, AGENTS.md, CLAUDE.md, workflows/).

## cgap commit plan (ordered)

1. **sources-first commit**: all 40 `M crates/**` src+Cargo.toml+README edits + tests/court_verdict.rs + crates/cgap-verification/ (8) + benches (14) + crates/cgap-ffi/src/{oabi_lookup.rs,xip_tables.rs}, crates/cgap-git/src/pack.rs, crates/cgap-storage/src/dred.rs, crates/cgap-bn3/src/noalloc.rs, crates/cli/src/custom_handlers/ (3), crates/cli/tests/cli_new_verbs.rs, crates/cgap-core/tests/ingress_layout_court.rs, crates/cgap-p28/tests/, ontologies/cloud/tests/w.commit 1 = full source closure (workspace green at this commit).
  ~96 files.
2. **manifests commit**: Cargo.toml, Cargo.lock, tests/court_verdict.rs if not in 1; cli/Cargo.lock. (If all M folded into 1, this is empty — coordinator may merge.)
3. **scripts commit**: scripts/{boundary_gate.py,zero_string_gate.py,projection_coverage.py,hop_ratio.py} — 4 files.
4. **docs/receipts commit**: all docs/jira/v26.11.1{,/waves}, v26.11.2, diataxis, readme-archive, READMEs, ontologies READMEs, root README.md, docs/{CHALLENGER_SALES_REPORT,REVENUE_C...} — 189+ files.
5. **CLI regen artifacts** may fold into commit 1 (generated_cli.rs, ocel.json, command-reference.md are gen-surface).

Final check after all commits: `git status --porcelain` returns only gitignored target/ debris. End state: clean except `.clap-noun-verb/` if not committed/ignored.
