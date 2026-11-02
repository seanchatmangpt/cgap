# TPS-2 — Station 200 cull execution plan

Lane: TPS-2 (PLAN). Date: 2026-11-02. Read-only reconciliation of
the TPS manual's Station 200 against the CULL verdict receipts.
Cull receipts read fresh this session: cull-01, cull-02, cull-03,
cull-04, cull-05, cull-08, cull-09 (LANDED);
cull-06, cull-10 (PENDING — not on disk at read time).
Lanes read fresh: lane-dx-06-fixture-races.md,
lane-c06-scripts-gate-coverage.md.
Station 200 sequencing law: AFTER the cull-10 wave plan lands;
never parallel with any in-flight cull lane.

## 0. Reconciliation: TPS manual vs cull-01 (the contradiction)

TPS manual Station 200 says: "remove zero_datafusion_gate.py".
CULL-01 + CULL-05 both rule KEEP (KEEP+fix / KEEP-FROZEN):
- the gate's subject is the live whole-graph NEGATIVE invariant
  post-removal (removal receipt cited at :16; needle is the bare
  string "datafusion" over cargo metadata/tree/manifests — it
  never names cgap-datafusion). It is the regression sealer that
  keeps the removal removed — a reintroduction vaccine, not a
  ghost.
- its `--self-check` exit 1 is NOT a gate defect: it is the
  pre-existing vendor/nanoarrow self-recursive symlink loop
  (lane-c06 ERRATA 1). Fix = one-line ignore-guard in
  copy_subject's ignore set (~zero_datafusion_gate.py:194).
RESOLUTION: in favor of the receipt. Station 200's "remove" row
is struck and replaced by lane S200-1 (symlink-guard fix). No
deletion of zero_datafusion_gate.py is planned, now or later;
its only re-open trigger is gate-subject death, defined below.

## 1. Cull ledger (verdicts incorporated)

| Cull | Verdict incorporated | Item |
|---|---|---|
| cull-01 | zdf KEEP+fix; gate3_cdata->gate3_schema MERGE | S200-1, S200-2 |
| cull-02 | doctor consolidation; c10 dies; stub | S200-3 |
| cull-02 open q | c07/c08/c09 WARN-degrade decision | S200-4 |
| cull-03 | posh_wire composition-test merge | S200-5 |
| cull-03 | octopus 101-exit witness split | S200-6 |
| cull-03 | pack_demux c7 harden-or-cull | S200-7 |
| cull-04 | csct pipeline mutation_controls_pass cull | S200-8 |
| cull-05 | zdf KEEP-FROZEN; retargets (z_string keys, | S200-9 |
| cull-05 | dead_library prefix_max_u8x8 REMOVE-ENTRY | S200-9 |
| cull-05 | workstream F10/F5 RE-TARGET | S200-9 |
| cull-08 | doc errata rewrites/tombstones | S200-10 |
| cull-09 | dod dead branch + doctor-agents alias | S200-11 |
| cull-06 | PENDING — no verdict to incorporate | gated |
| cull-10 | PENDING — wave plan is the sequencing gate | gated |

## 2. Sequencing

1. GATE: cull-10-cull-wave-plan.md must be LANDED on disk before
   any S200 item starts. If it contradicts any S200 row below,
   cull-10 wins; re-reconcile that row only.
2. Order below is strict: one item at a time, no parallel lanes,
   each item's fence must be green before the next starts.
3. S200-1 (symlink guard) precedes S200-2 (gate3 merge): both
   touch scripts/ gate files and share the C06/coverage fences.
4. S200-3 and S200-4 are one doctor work-package: stub first,
   then the c07/c08/c09 decision, never interleaved with S200-9.
5. Doc-only items (S200-10) may run only when no code lane is
   in flight (receipt_lint --strict watches docs/jira).

## 3. Ordered execution items

### S200-1 — zero_datafusion_gate symlink ignore-guard fix
- Origin: cull-01 (KEEP+fix), cull-05 (KEEP-FROZEN), lane-c06
  ERRATA 1. Strikes TPS Station 200 "remove zdf" row.
- Owning lane: new lane-c06b-zdf-symlink-guard (C06 scope heir).
- Files: scripts/zero_datafusion_gate.py (~:194, copy_subject
  ignore set: add the nanoarrow self-nesting symlink loop so
  shutil copy skips it); tests/gates/test_gate_exits.py (flip
  the `--self-check == 1` pin to `== 0` once guard lands; keep
  the exit-2 subject-missing pin unchanged).
- Regression fence: tests/gates/test_gate_exits.py stays green
  (17 passed -> 18); enforce_coverage_floor.py must not newly
  regress (currently honest-red exit 1; zdf branch coverage must
  rise, not fall); nominal `zero_datafusion_gate.py` exit 0.
- Re-open trigger: any reintroduction of a datafusion dep edge
  (gate flips red = sealer working) OR self-check re-exits 1 on
  a NEW unignored symlink loop (guard too narrow; widen set).
- Scope: S.

### S200-2 — gate3_cdata -> gate3_schema merge
- Origin: cull-01 (sole VERIFIED-REDUNDANT at assert level).
- Owning lane: lane-c06c-gate3-merge.
- Files: scripts/gate3_schema.py (fold cdata's wire-parity set
  as schema config 0 in the per-config loop); delete
  scripts/gate3_cdata.py (subsumer: gate3_schema.py check 4,
  bitwise buffers per config, same cdylib).
- Regression fence: `python3 scripts/gate3_schema.py` exit 0
  pre/post with the folded config 0 wire-parity set asserted;
  tests/gates/test_gate_exits.py (gate3 pins) stays green.
- Re-open trigger: any C-Data export ABI change, OR gate3_schema
  drops its check 4 (bitwise buffer pin).
- Scope: M.

### S200-3 — doctor consolidation (canonical Python doctor)
- Origin: cull-02; DXD-02 decision stands (extend
  scripts/doctor_agent_lanes.py; doctor.rs reference-only,
  ggen-regen hazard).
- Owning lane: lane-dx-01-doctor-agent (heir work-package).
- Files: scripts/doctor_agent_lanes.py (port c01 toolchain,
  c02 wasm-targets, c03 git, c04 b3sum, c05 pyshacl,
  c06 workspace-check, c07 scripts-list, c08 receipts,
  c09 lsl-ratified; each ported check gets env-override +
  flip arm per DXD-02; missing-but-expected = WARN never RED).
  crates/cli/src/bin/doctor.rs -> 5-line redirect stub exec'ing
  the Python script, header naming DXD-02 + CULL-02. doctor.rs
  c10 (dirty-tree) DIES — subsumer: doctor_agent_lanes.py
  dirty-tree census (same `git status --porcelain` input,
  strictly richer per-dir output). Confirm whether doctor.rs is
  ggen-generated; if yes the stub dies at next regen (accepted).
- Regression fence: exit ABI 0/1/2 max unchanged, both call
  paths; `cargo test -p cli` stays green; doctor-agents recipe
  (justfile) exit 0; stub exec returns identical top-level
  exit codes for a clean tree.
- Re-open trigger: any doctor.rs check is deleted from the
  Python port without a named subsumer, OR the stub stops
  matching the Python doctor's exit code on the same tree.
- Scope: L.

### S200-3b — c07/c08/c09 WARN-degrade decision (parent call)
- Origin: cull-02 open questions 1 (not blockers).
- Owning lane: parent decision recorded inside lane-dx-01.
- Decision to record: c07 (v26.10.31-frozen 10-name script
  list), c08 (receipts), c09 (lsl-ratified) migrate as
  WARN-degrade (never RED on version-frozen path drift) vs
  park. Recommended: WARN-degrade with the frozen list carried
  as a data file, so drift is visible, not silent.
- Regression fence: none beyond S200-3's (same work package).
- Re-open trigger: any of c07/c08/c09 silently passing on a
  drifted tree after landing (WARN was swallowed).
- Scope: S.

### S200-4 — c06 workspace-check runtime budget
- Origin: cull-02 open question 2 (~5s DXD-02 budget).
- Owning lane: lane-dx-01-doctor-agent (same package).
- Decision: flag-gate the workspace-check port (e.g. --deep)
  or split it out of the default doctor pass; record which in
  the lane receipt with a measured runtime.
- Regression fence: default doctor pass wall-time <= ~5s on a
  warm tree (DXD-02 budget), measured in the receipt.
- Re-open trigger: default pass exceeds the budget post-merge.
- Scope: S.

### S200-5 — posh_wire composition-test merge
- Origin: cull-03 (OVERLAP-PARTIAL verdict).
- Owning lane: lane-git-03-ingress-ring (heir) + lane-posh-01
  (wire owner consulted).
- Files: crates/cgap-git/tests/posh_wire_court.rs — move the
  scatter-gather composition test (:143-197) into
  crates/cgap-git/tests/ingress_ring_court.rs; KEEP in
  posh_wire_court.rs the magic/stride flip courts, and KEEP the
  magic-flip demux cross-check (:225-239) — it is the only
  wire-side assert of the SubjectMissing law (ghost subject
  named: SubjectMissing via demux).
- Regression fence: `cargo test -p cgap-git --test
  ingress_ring_court --test posh_wire_court` exit 0, test
  counts reconciled (+1 moved, 0 lost); full `cargo test -p
  cgap-git` green (unblocked once GIT-16 lands — see DX-6
  LOUD-PENDING).
- Re-open trigger: any demux/wire SubjectMissing law regression
  OR the cross-check is deleted without a named subsumer.
- Scope: M.

### S200-6 — octopus 101-exit witness split
- Origin: cull-03 Flag 1 (~50% flake; failure mode = self-re-exec
  child run, not merge logic).
- Owning lane: lane-git-12-octopus-merge (heir).
- Files: crates/cgap-git/tests/octopus_merge_court.rs — split
  the 101-exit andon witness (mutation_conflict_injection_flip,
  :296-373; child exit arm ~:369) into its own #[ignore]-gated
  test (`#[ignore = "self-re-exec child; run explicitly"]`),
  run via `cargo test -p cgap-git --test octopus_merge_court
  -- --ignored`; KEEP the in-process DomainClash arm
  (:214/:326/:329-337; asserted twice in-file already).
- Regression fence: `cargo test -p cgap-git --test
  octopus_merge_court` exit 0 non-flaky (3 consecutive clean
  runs witnessed in the lane receipt); the --ignored witness
  exits 0 when run explicitly.
- Re-open trigger: the DomainClash in-process arm loses either
  in-file assertion, or the ignored witness flakes when run
  explicitly.
- Scope: S.

### S200-7 — pack_demux c7 zero-alloc harden-or-cull
- Origin: cull-03 Flag 2 (SERIAL mutex guards this binary only;
  false NON-zero from foreign allocations is the failure mode;
  DX-6 already applied the SERIAL fix — the residual risk is
  cross-binary).
- Owning lane: lane-git-01-pack-demux (heir).
- Files: crates/cgap-git/tests/pack_demux_court.rs
  (c7_parse_path_is_zero_alloc, :346-368).
- Hardening (preferred): after the SERIAL hold, re-read the
  counter and if non-zero, re-arm and re-probe once, and
  attribute the residue via allocation backtrace or c8's static
  source scan verdict; cull (fallback) only if a witnessed
  foreign-allocation false-POSITIVE persists after hardening —
  subsumer named: c8 static source scan + soc_full_chip_court
  whole-pipeline zero-alloc window.
- Regression fence: `cargo test -p cgap-git --test
  pack_demux_court` exit 0, 10 tests, 3 consecutive clean runs;
  concurrent-witness rerun after GIT-16 lands (DX-6 PENDING).
- Re-open trigger: any false NON-zero on a clean tree, or the
  SERIAL hold removed without a replacement.
- HARDEN first, cull only on witnessed persistent false-positive.
- Scope: S (harden) / M (cull path).

### S200-7b — egress_ring SERIAL pattern census
- Origin: lane-dx-06 (same SERIAL Mutex pattern lives in
  egress_ring_court.rs; same cross-binary residual risk).
- Owning lane: lane-dx-06-fixture-races (heir work package).
- Files: crates/cgap-git/tests/egress_ring_court.rs (SERIAL
  Mutex hold across all 5 tests) — audit once GIT-16 lands and
  the DX-6 concurrent witness rerun is clean.
- Regression fence: DX-6's 2-target concurrent witness 0/0 x3
  rounds, post-GIT-16.
- Re-open trigger: any false NON-zero from foreign allocations.
- Scope: S.

### S200-8 — csct pipeline mutation_controls_pass cull
- Origin: cull-04 (duplicates src/mutation.rs:66 verbatim in
  intent).
- Owning lane: lane-b08-cscb-runner (csct court owner).
- Files: crates/csct/tests/cscb_pipeline_court.rs (or the
  csct_pipeline_court.rs at :73-78) — remove the
  mutation_controls_pass assert block.
- Subsumer (falsifier satisfied): src/mutation.rs:66.
- Regression fence: csct pipeline court exits 0 with count
  -1; `cargo test -p csct` green; the mutation flip coverage
  remains via src/mutation.rs:66's own test surface.
- Re-open trigger: src/mutation.rs:66 assert weakens or moves
  without a named heir.
- Scope: S.

### S200-9 — cull-05 retarget bundle
- Owning lane: split per subject (see sub-items).
- Sub-item S200-9a — zero_string DOC_EXECUTION_FAMILY retarget
  (S): drop the two ghost keys "cli"/"integration" from
  scripts/zero_string_gate.py:447-456 (subsumers: real package
  names cgap-cli, cgap-integration). Fence:
  zero_string_gate.py nominal exit 0; --self-check exit 0.
  Re-open: a new package name resolves to a directory name.
- Sub-item S200-9b — dead_library prefix_max_u8x8 REMOVE-ENTRY
  (S): delete the entry from
  scripts/dead_library_exceptions.json — subsumer: the live
  consumption at cscb_c1_c2_ingress.rs:82. Re-open: the symbol
  loses its consumer (re-add entry + must-consume).
- Sub-item S200-9c — workstream map RE-TARGET (S): F10 ->
  point at open R7 (or remove if R7 retired); F5 -> refresh
  against the 90+ test expansion (docs/lane-f5-workspace-
  integration.md). Fence: receipt_lint census mode exit 0.
  Re-open: a row restates a stale subject after this refresh.
- No code behavior change; fences are gate exits + lint.
- Re-open trigger (bundle): any retargeted row/gate/key reverts
  to its pre-cull subject without a named subsumer.

### S200-10 — cull-08 doc errata bundle
- Owning lane: lane-mv-08-doc-refresh (heir work package).
- Files: exactly the cull-08 action table (10 rows):
  REVENUE_GENERATION_COMPENDIUM.md:103, 12-ard-prd-cgap.md:19,
  paper/position-paper-v3.md:231,233,
  v26.10.31/07-chain-economy-8020.md:36 (tombstone),
  LICENSE-P28-SIL.md:69 (errata rider), lane-b05-cscb-c6.md
  :21,47, lane-git-11-retract.md:90 (low),
  lane-mv-05-sla-reconcile.md:44 + audit/release-blockers.md
  :237 (verify/rewrite), audit/dod-csct-cscb.md:35.
- Regression fence: receipt_lint.py census exit 0;
  `--strict` may go red only on NEW findings (pre-existing
  UNRECEIPTED-CLAIMs are lane-c06 ERRATA 3, out of scope).
- Re-open trigger: any tabled claim restated as fact elsewhere
  without citing the errata.
- Scope: M.

### S200-11 — cull-09 justfile follow-through
- Owning lane: lane-dx-07-lane-scaffold / DXD-10 wave F heir.
- Files: justfile — delete the dead dod placeholder branch
  (else exit-1 branch, now unreachable), fix stale header :8-9;
  doctor-merged becomes canonical; doctor-agents retained one
  wave as alias, then removed (the eventual cull).
- Regression fence: `just --list` exit 0; `just doctor`,
  `just doctor-agents` (alias wave), `just doctor-merged`,
  `just dod`, `just status` exit 0 post-edit.
- Re-open trigger: doctor-agents removed before doctor-merged
  is canonical (order inversion), or a recipe name collides
  (DXD-07 Anti-pattern-1).
- Scope: S.

## 4. PENDING culls (no plan rows; gate only)

- cull-06: not on disk. When it lands, reconcile its verdicts
  into this plan as S200-12+ rows (same template: owning lane,
  files, fence, re-open trigger, scope) — do not guess its
  subject.
- cull-10: not on disk. THE sequencing gate. S200 order stands
  only if cull-10 does not contradict any row; cull-10 wins on
  contradiction; re-reconcile the contradicting row only.

## 5. Falsifier law (every deletion names its subject)

Every planned deletion and its subsumer/ghost subject:
- gate3_cdata.py DELETED, subsumer gate3_schema.py check 4
  (bitwise buffers per config, same cdylib).
- doctor.rs c10 check DELETED, subsumer doctor_agent_lanes.py
  dirty-tree census.
- doctor.rs binary -> redirect stub (not deleted; survives
  ggen regen or accepted loss).
- posh_wire composition test MOVED, subsumer
  ingress_ring_court.rs (same assert, new home); posh_wire
  magic-flip demux cross-check KEPT (only SubjectMissing
  wire-side assert).
- octopus 101-exit arm SPLIT not deleted (#[ignore] witness);
  DomainClash arm kept.
- pack_demux c7: harden first; cull fallback names c8 static
  scan + soc whole-pipeline window as subsumer.
- csct mutation_controls_pass DELETED, subsumer
  src/mutation.rs:66.
- zero_string "cli"/"integration" keys DELETED, subsumers
  cgap-cli / cgap-integration package names.
- dead_library prefix_max_u8x8 entry DELETED, subsumer: live
  consumption at cscb_c1_c2_ingress.rs:82.
- zero_datafusion_gate.py NOT deleted — struck from Station
  200. Its deletion trigger would require a named subject:
  only if the whole-graph negative invariant is retired AND a
  named sealer subsumes it. None exists today.
- cull-06/cull-10 rows: falsifier applies when they land.
