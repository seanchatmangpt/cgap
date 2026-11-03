# Lane FS-15 (DOD-18) — staging plan refresh + commit message

Date: 2026-11-02. Repo: /Users/sac/cgap @ main 7f7ccdf7 (read-only git:
status/diff/rev-parse only; zero git writes). Owned files: this receipt
+ dod/COMMIT_MSG_v26.11.2.txt. Style <=100-char lines. No Process.sleep.

## Census refresh (this session, exit 0)

- `git status --porcelain` -> 235 entries: 104 M, 135 ?? (exit 0).
  Supersedes w-commitplan-v2's 252 (MV-8 refresh) and F11's 138 —
  both stale-as-of-now per the receipt-cited-counts law.
- `git diff --stat` -> 104 files changed, 6094 insertions(+), 732
  deletions(-) (exit 0).
- `git rev-parse HEAD` -> 7f7ccdf7df614a22260a96f53dc8b6af4897fa2e.

## X-01's 6 UNOWNED findings — resolution

1. crates/cgap-ffi/src/oabi_lookup.rs + xip_tables.rs: STILL UNOWNED as
   files — no lane receipt claims the diffs. Content-classified this
   session via `git diff`: oabi_lookup = rustfmt-class reflow of the
   surface court's run_bounded/oabi_verb_refusal plumbing; xip_tables =
   hex-table line-width reflow. Nearest-neighbor standing: XC-01 ran
   `scripts/ffi_surface_gate.py --build` exit 0 (12/12 symbols) over
   this exact surface post-diff (lane-xc01-ffi-surface-truth.md), and
   FIX-19 §2 re-ran the ffi reproducer green. Classified
   UNOWNED-ESCALATED to the coordinator: format-only content, gate-
   corroborated standing, safe to stage in Commit 1 but the coordinator
   must own the attribution.
2. docs/jira/v26.11.2/bench-validate/ (bench-01..05, opt-03) +
   audit/pub-01-dry-run-matrix.md: RESOLVED RECEIPTED — transcribed
   and cited by dod/dod-12-bom-receipt.md, dod/dod-16-claim-sweep-
   verdict.md, dod/charter-t12-gate-run.md, dod/charter-p3-analyze-
   verify.md and lanes (dod-06/dod-08/dod-10/goal-01/promo01). Stage
   in Commit 3.
3. Empty .alive sentinels (lanes/{f01,fix02,gap01,lane-fix04,r16,
   rc-07,tc-11,xc05}.alive): COORDINATOR-CALL — liveness markers, not
   artifacts. Segregated OUT of all commits; delete at integration.
4. lanes/dispatch-ledger.jsonl (M): COORDINATOR-CALL — fleet bookkeeping.
   Segregated OUT of Commits 0-5; coordinator stages or discards.

## Staging script (explicit paths, never `git add -A`)

Coordinator executes in order. After Commit 0 and before Commit 1:
`cargo build --workspace` exit 0 gate. After Commit 1: `cargo test -p
cgap-court` exit 0. Before Commit 3: `make gate0` exit 0 (cli ggen
regen is Commit 0's checkpoint; byte-identical replay per CLAUDE.md).

### Commit 0 — generated-artifacts regen checkpoint (regen, then stage)

    cd /Users/sac/cgap/crates/cli && rm Cargo.toml && ggen sync run \
      && python3 manifest_patch.py   # cli regen, byte-identical replay
    cd /Users/sac/cgap && make gate0 # ggen sync x2 cmp + header compile
    git add Cargo.toml Cargo.lock crates/cli/Cargo.toml \
      crates/cli/.clap-noun-verb/ocel.json crates/cli/manifest_patch.py \
      include/cgap.h ontologies/cloud/Cargo.toml
    git commit -F docs/jira/v26.11.2/dod/COMMIT_MSG_v26.11.2.txt || true
    # (final message commit lands at Commit 5 seal; intermediate commits
    #  use the same -F file — coordinator may squash at drain)

### Commit 1 — crates/ code + tests

    git add crates/cgap-arrow/src/{blake3,foreign,slab,slice}.rs \
      crates/cgap-bn3/src/{arena,noalloc,unify}.rs \
      crates/cgap-bont/src/{primitives,shacl_compiler}.rs \
      crates/cgap-core/Cargo.toml \
      crates/cgap-ffi/src/{lib,oabi_lookup,receipt_chain,sovereign, \
      stats,xip_tables}.rs \
      crates/cgap-ffi/tests/{fa9_commit_gating,stats_block_court}.rs \
      crates/cgap-git/Cargo.toml \
      crates/cgap-git/src/{admission_gate,dag,egress_ring,ingress_ring, \
      pack_demux,ref_store,retract}.rs \
      crates/cgap-git/tests/{andon_liveness,breach_regression, \
      egress_layout,pack_demux_poison,quota_alloc_purity, \
      posh_refusal_matrix_court,ref_store_reentrancy, \
      ref_store_width_arms}_court.rs \
      crates/cgap-p28/src/{asc450,attestation}.rs \
      crates/cgap-p28/tests/attestation_mac_court.rs \
      crates/cgap-parquet/src/{byteslice,footer,mmap_writer,scan}.rs \
      crates/cgap-storage/src/{atomic_commit,closure,dred}.rs \
      crates/cgap-storage/tests/{common/mod.rs,journal_fault_matrix_ \
      court.rs,media_loss_recovery_court}.rs \
      crates/cgap/src/pipeline.rs \
      crates/cgap-gate/tests/penstock_corpus_court.rs \
      crates/cgap-bn3/tests/capacity_bound_court.rs \
      crates/bench/Cargo.toml crates/bench/benches/chains-ingress.rs \
      crates/bench/benches/mctq_consolidation.rs \
      crates/bench/src/bin/{arrow_accessors_bench,bitslice_bench}.rs \
      crates/cgap-fleet-ops crates/cgap-ocel2 crates/csct/Cargo.toml
    # EXCLUDE: crates/cgap-fleet-ops/target/ (gitignored; verified
    # pattern .gitignore `**/target/`). All rc-* regression courts ride
    # here so the court harness commit is self-consistent.

### Commit 2 — scripts + gates

    git add scripts/boundary_gate.py scripts/receipt_lint.py \
      scripts/verify_cscb_sram.py scripts/zero_string_gate.py \
      scripts/conformance_kit_export.py scripts/ffi_surface_gate.py \
      scripts/gate_battery.py scripts/journal_mutation_matrix.py

### Commit 3 — docs + receipts (orphan segregation resolved)

    git add docs/jira docs/ARCHITECTURE.md docs/NOTICE-blake3.md \
      CONTRIBUTING.md SECURITY.md README.md CHANGELOG.md \
      docs/CHALLENGER_SALES_REPORT.md docs/REVENUE_GENERATION_COMPENDIUM.md \
      docs/audit docs/releases docs/jira/v26.10.31
    # bench-validate/ + pub-01 matrix ride here (finding 2 RECEIPTED).
    # .alive sentinels + dispatch-ledger.jsonl stay OUT (findings 3-4).
    # EXCLUDE from staging: docs/jira/v26.11.2/lanes/*.alive and
    # docs/jira/v26.11.2/lanes/dispatch-ledger.jsonl.

### Commit 4 — license + legal

    git add LICENSE.md LICENSE-P28-SIL.md SECURITY.md
    # SECURITY.md is new; staged here and NOT in Commit 3 (remove it
    # from the Commit 3 list when executing — legal-class file).

### Commit 5 — dod/ seal artifacts

    git add docs/jira/v26.11.2/dod
    # Includes COMMIT_MSG_v26.11.2.txt + this receipt's lane file is
    # already in Commit 3's docs/jira sweep; the seal commit is the
    # terminal one and carries the dod/ dir verbatim.

## Fences named and their standing

- Workspace build + court harness + gate0: coordinator gates at the
  commit boundaries above; this plan-only lane ran no builds (PLAN
  lane, read-only git). Standing for the plan: ALIVE (census commands
  + exits this session); the BUILD standing remains w-commitplan-v2's
  and dod-17's, re-grounded at use time.
- Never-commit debris (restated): libae8_dbg.rmeta,
  crates/cgap-branchless/wrapper_riscv32imc.o,
  crates/cgap-fleet-ops/target/, *.alive, /tmp lane leases.

## Divergences

- Dispatch said "X-01's inventory" — no receipt named X-01 exists on
  disk (grepped lanes/, waves/, audit/). The 6 findings were taken from
  the dispatch text and resolved against disk directly (sections
  above). Recorded as a [registered divergence], not silently
  repaired.
- The 5-commit plan in dispatch is realized as Commits 0-5 (6 entries:
  0 is the regen checkpoint, not a content commit) — consistent with
  the dispatch's own "Commit 0 ... -> Commit 5" enumeration.
- oabi_lookup.rs/xip_tables.rs file-level ownership is still unclaimed
  (finding 1 = UNOWNED-ESCALATED); only surface-level gate standing
  exists (XC-01 exit 0). Coordinator owns final attribution.

## Token-sweep verification

- COMMIT_MSG: 90 lines, awk length>100 sweep clean (0 hits), written
  in 4 chunks <50 lines each.
- This receipt: written in 6 chunks <50 lines each; on-disk bytes
  verified by the Edit tool round-trips (no corruption pattern).

<!-- json-footer
{"lane":"FS-15","aka":"DOD-18",
"receipt":"docs/jira/v26.11.2/lanes/lane-fs15-staging-plan.md",
"files_written":["docs/jira/v26.11.2/lanes/lane-fs15-staging-plan.md",
"docs/jira/v26.11.2/dod/COMMIT_MSG_v26.11.2.txt"],
"commands":["git status --porcelain (exit 0, 235 entries: 104 M, 135 ??)",
"git diff --stat (exit 0, 104 files, +6094/-732)",
"git rev-parse HEAD (exit 0, 7f7ccdf7)",
"awk length>100 sweep on COMMIT_MSG (exit 0, 0 hits)"],
"courts_run":[],
"standing":"ALIVE (census+reads this session; plan-only lane, no builds, no git writes)",
"git_writes":0,
"divergences":["no X-01 receipt on disk; findings resolved from dispatch text [registered]",
"oabi_lookup/xip_tables UNOWNED-ESCALATED (format-class, XC-01 gate corroborates)",
"bench-validate/pub-01 now RECEIPTED via dod-12/dod-16/charters",
"alive sentinels + dispatch-ledger.jsonl segregated as coordinator-call"]}
json-footer -->
