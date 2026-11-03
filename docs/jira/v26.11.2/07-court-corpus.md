# 07-court-corpus

Campaign charter: v26.11.2 starDF (`docs/jira/v26.11.2/00-ard-prd.md`). This file is the
court corpus: the shared court discipline plus one named flip court per Pareto lever
(L1-L5) and per release gate (G1-G8). Subjects are cited file:line from the on-disk
tree at the time of writing; if a subject moves, the court re-derives its cite before
running. Pattern catalog for the ggen side:
`docs/jira/v26.11.2/research/g08-court-mutation-patterns.md`.

## 1. Shared court discipline

Every court in this corpus obeys the same five laws. A court that cannot flip is FAIL;
a skip is loud (`[SKIP] <reason>`, exit 3), never silent.

1. **Sandbox only.** Mutations apply to `/tmp` copies of the subject tree, never to the
   shared checkout at `/Users/sac/cgap`. The court script copies the minimal subtree it
   needs (crate `src/` + `Cargo.toml`), mutates the copy, and runs the gate/bench there.
2. **sha256-exact restore.** Before mutating, the court records a sha256 manifest
   of every file it will touch. After the flip leg, it restores and re-hashes; a single
   byte mismatch aborts the run with a typed restore failure (exit 1, `RESTORE-MISMATCH`).
   The court's self-check must corrupt a restore on a real temp copy once to prove
   restore-detection is not vacuous (pattern: `scripts/zero_string_gate.py --self-check`).
3. **mtime-bump restore law** (witnessed, verbatim from
   `docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184`): a sha256-byte-exact restore
   that preserves pre-mutation mtimes does NOT make cargo rebuild — the fingerprint sees
   the artifact as fresh and re-runs the MUTATED binary (witnessed: FAILED verdict on
   restored source; touch -> rebuild -> clean re-run). Shim-court "restore byte-exact + clean
   re-run" legs must bump mtimes or clean the target between mutation and restore legs,
   or the clean re-run is not evidence of the restored source.
4. **Exit contract** (`docs/jira/v26.11.2/01-release-gates.md`, runner wiring): 0 pass /
   1 gate fail / 2 subject missing / 3 loud skip. A mutation-free gate run with no
   mutation leg is VACUOUS and exits nonzero. Compile-abort (exit 1, 0 tests
   executed) is recorded as
   compile-abort, never counted as a test failure (census law, waves/v2-07-census.md:9).
5. **No Process.sleep anywhere.** Courts poll observables (exit codes, file digests,
   bench estimates.json, verdict counts) with bounded, stated deadlines; the runtime
   owns time. Ratio fences replace absolute-ms duration bounds.

Build roots are leases: each court sets `CARGO_TARGET_DIR=/tmp/<court-id>-target` and
deletes it at court end. cwd resets between shell calls; courts use absolute paths.

## 2. Per-lever courts (L1-L5)

Mutations and kill signals come from `docs/jira/v26.11.2/02-pareto-levers.md` (the
engineering spec of record); this section binds each to a file:line subject and an exit.

### L1-COURT — ingress digest escalation (lever L1)

- Subject: `crates/cgap-gate/src/penstock.rs:491` `pub fn ingress(bytes, len, out)`
  — the admission check chain (magic/version/geometry rows at :502-524).
- Mutation: corrupt one slot byte or stage code in a sandbox copy of the frame corpus
  generator so the per-stage checksum assert diverges from the untimed preflight fold.
- Kill signal: the per-stage checksum assert fires (escalation still refuses a
  spoliated frame with `CGAP_ERR_INVALID_MAGIC` class).
- Exit: 0 baseline green; mutated run must fail (panic/exit nonzero) = FLIPPED.

### L2-COURT — parquet SWAR FNV (lever L2)

- Subject: `crates/cgap-parquet/src/footer.rs:119-127` `fnv1a64_seeded` (byte-wise
  FNV-1a; default-basis wrapper `fnv1a64` at :114, content address `block_digest`).
- Mutation: flip one committed window byte in the sealed block before `verify_seal`.
- Kill signal: `ChecksumMismatch { region: Body }` -> exit 101 (flip class witnessed in
  P3-17; M2 truncation law: decoder is span-bounded by the page header).
- Exit: 0 green baseline on the sandbox; 101 after the byte flip = FLIPPED.

### L3-COURT — BN3 unify join (lever L3)

- Subject: `crates/cgap-bn3/src/unify.rs` nested-loop join probe — module docs :11,35-39
  (v1 nested-loop with mask AND) and the per-body-atom probe loop at :453-455.
- Mutation: binding-alias comparator swap (doe-4 SHIM-DOE4 class) in a sandbox copy.
- Kill signal: differential oracle assert — match count must equal the raw-word O(N)
  hash join (no unify code in the oracle path); M12-7 source_mask popcount assert rides.
- Exit: 0 green; swapped comparator flips the oracle assert = FLIPPED.

### L4-COURT — sovereign commit sink (lever L4)

- Subject: `crates/cgap-ffi/src/sovereign.rs:706-730` `cgap_commit_parquet` — per-call
  `std::fs::File::create(path)` at :722 plus fresh boxed `FileSink` scratch alloc.
  Refusal const `CGAP_ERR_P28_SPOLIATION` = 0x5032_38FF at :83 (refusal sites :299,
  :307, :315, :384).
- Mutation: corrupt one committed header byte (refusal class) or drop one slab write.
- Kill signal: per-stage `CgapStatus` assert fires; the P28 spoliation negative control
  (one flipped digest byte -> exactly `CGAP_ERR_P28_SPOLIATION`) fires.
- Exit: 0 green (`CGAP_OK`); mutated run returns `CGAP_ERR_P28_SPOLIATION` = FLIPPED.

### L5-COURT — seal chain ASC450 fuse (lever L5)

- Subject: `crates/cgap-p28/src/asc450.rs` (ASC450 metering) fused with the P28 §5
  span validate in the seal chain (bench subject of `bench-chains-seal.md`, P3-19).
- Mutations (any one): bit-flip an upstream receipt (validate must refuse); demote a
  crossing (no proof emission — `proven == N/2` assert flips); flip a receipt digest
  (verify DigestMismatch).
- Kill signal: typed refusal / assert / DigestMismatch per mutation above.
- Exit: 0 green; mutated run refuses = FLIPPED.

## 3. Gate courts (G1-G8)

One flip court per release gate, per `docs/jira/v26.11.2/01-release-gates.md` (the
dispatch block of record; names cited, not redefined).

Per-gate FLIP courts (mutation / kill signal / exit / subject):

- **G1-PUBLISH** — subject: scripts/release_check.py.
  Mutation: strip crate `description` from one Cargo.toml.
  Kill: release_check FAILs on the copy. Exit on flip: 1.
- **G2-PARETO** — subject: L1-L5 bench kernels.
  Mutation: remove `#[inline]` / inject bounds check / black_box.
  Kill: level threshold flips to FAIL. Exit on flip: 1.
- **G3-MAT-BENCH** — subject: mat closure bench (absent today).
  Mutation: skip one derivation rule / de-dup one join arm.
  Kill: derived-fact count changes. Exit on flip: 2 (loud skip).
- **G4-DRED** — subject: DRED bench (absent today).
  Mutation: skip entropy recompute.
  Kill: entropy-variance number changes. Exit on flip: 2 (loud skip).
- **G5-SILICON** — subject: riscv32imc footprint + symbol_gate.py.
  Mutation: add `Vec::new()` push or `panic!` site.
  Kill: symbol_gate FAILs (alloc/panic symbol). Exit on flip: 1 (2 if riscv absent).
- **G6-NO-DF** — subject: `grep -ri datafusion` both repos.
  Mutation: append `datafusion = "42"` to Cargo.toml.
  Kill: gate FAILs on the copy. Exit on flip: 1.
- **G7-PATENT** — subject: verify_ost_design_around.py (absent).
  Mutation: merge one claim with prior-art mechanism.
  Kill: verdict drops to 3/4. Exit on flip: 3 (loud skip).
- **G8-STARDF-F1** — subject: purrdf stardf scripts (absent).
  Mutation: remove one of the 11 mappings.
  Kill: census exits nonzero, digest breaks. Exit on flip: 2 (loud skip).

All gate courts mutate `/tmp` copies only, sha256-exact restore, mtime bump
before the clean re-run (law 3, section 1).
Witnessed restore legs: waves/w2-13-l4l5-courts.md:44.

## 4. Uniform flip procedure

Every court (lever or gate) runs the same eight steps. No step is optional; a court
that cannot complete step 4 is FAIL, not SKIP.

1. `cp -R` the minimal subject subtree (crate `src/` + `Cargo.toml`, or gate script +
   its inputs) into `/tmp/<court-id>/`; record the copy exists.
2. `shasum -a 256` a manifest of every file the court will touch; save it inside
   `/tmp/<court-id>/manifest.before`.
3. Apply the named mutation (section 2 or 3) to the sandbox copy only. Record the
   mutation diff line(s).
4. Run the gate/bench against the sandbox; REQUIRE the kill signal (nonzero exit or
   typed assert) and record command + exit. No kill = court FAIL.
5. Restore the mutated file(s) byte-exact from the canonical tree; verify with
   `shasum -a 256 -c` against the step-2 manifest. A mismatch aborts with
   `RESTORE-MISMATCH` (exit 1).
6. Bump mtimes on the restored sandbox sources (`touch`) or `cargo clean -p <crate>`
   the sandbox target — law 3; without this the clean re-run is vacuous.
7. Re-run the gate/bench on the restored sandbox; REQUIRE exit 0 clean. Record
   command + exit. This leg proves the kill came from the mutation, not the copy.
   Witnessed legs: waves/w2-13-l4l5-courts.md:44.
8. Write the court receipt (waves/ slot, section 5 table) with: court ID, subject
   file:line, mutation, all three legs' commands and exits, sha256 restore proof,
   mtime-bump note. The receipt is the flip witness; no receipt, no flip.

Self-check (once per court corpus revision): corrupt a restore on a real temp copy and
prove step 5 detects it (restore-mismatch abort). A corpus whose self-check
reports VACUOUS is itself VACUOUS.

## 5. Flip summary

Every cell filled; no silent-missing. Receipt slots are
`docs/jira/v26.11.2/waves/w<N>-<court-id>.md` (or the named wave receipt).

| Court | Subject | Mutation | Kill | Receipt |
|---|---|---|---|---|
| L1-COURT | cgap-gate/src/penstock.rs:491 | slot byte corrupt | assert | w2-l1-court |
| L2-COURT | cgap-parquet/src/footer.rs:119-127 | committed window byte flip | 101 | w2-l2-court |
| L3-COURT | cgap-bn3/src/unify.rs:453-455 | comparator swap | oracle assert | w2-l3-court |
| L4-COURT | cgap-ffi/src/sovereign.rs:722 | header byte / slab drop | SPOLIATION | w2-l4-court |
| L5-COURT | cgap-p28/src/asc450.rs | receipt bit-flip / crossing demote | refuse | w2-l5-court |
| G1-FLIP | scripts/release_check.py | strip crate `description` | 1 | w3-g1-court |
| G2-FLIP | bench kernels L1-L5 | inline removal / bounds check | 1 | w3-g2-court |
| G3-FLIP | mat closure bench | skip derivation rule | 2 loud | w3-g3-court |
| G4-FLIP | DRED bench | skip entropy recompute | 2 loud | w3-g4-court |
| G5-FLIP | scripts/symbol_gate.py | alloc / panic symbol injection | 1/2 | w3-g5-court |
| G6-FLIP | datafusion grep gate | `datafusion = "42"` append | 1 | w3-g6-court |
| G7-FLIP | verify_ost_design_around.py | claim / prior-art merge | 3 loud | w3-g7-court |
| G8-FLIP | purrdf stardf scripts | remove one mapping | 2 loud | w3-g8-court |

## 6. Wave-added courts (reconciled 2026-11-02, lane RW-05)

The wave landed test-binary courts beyond the §2/§3 L/G roster. §5's
13-row flip summary is the L1-L5 + G1-G8 roster ONLY — it is not the
campaign court count. XC-05's conformance-kit exporter inventoried **91
courts** at its run (81 cargo-test targets, 5 corpus-doc entries, 5
gate-mode entries; `lanes/lane-xc05-conformance-kit-exporter.md:30,34`) —
superseded (historical). X-05's audit regenerated the exporter
(conformance_kit_export byte-identical across two runs) and counted
**95 courts** (85 cargo-test targets + 5 corpus-doc + 5 gate-mode); the
+4 delta vs XC-05 is the wave's post-XC-05 courts. All 15 §6 wave
courts verified present and present in the manifest
(`lanes/lane-x05-court-corpus-audit.md`; receipt pending on disk at
this edit — lane X-05FIX, 2026-11-02).
The §6 entries below carry one-line descriptions with the owning
receipt; subjects are cited file:line from the on-disk tree at
reconciliation time.

- `cgap-git/tests/breach_regression_court.rs` — breach-of-the-week
  regression tollgate; each red-NN confirmed breach pins a named arm.
  Receipt: `lanes/lane-r14-breach-regression-court.md`.
- `cgap-gate/tests/penstock_corpus_court.rs` — malformed-corpus
  differential court over penstock ingress (5 courts, exit 0).
  Receipt: `lanes/lane-rc12-penstock-corpus-court.md`.
- `cgap-git/tests/quota_alloc_purity_court.rs` — alloc purity across
  quota refusal paths (red-09/red-14 hardening). Receipt:
  `lanes/lane-rc14-quota-alloc-purity.md`.
- `cgap-storage/tests/media_loss_recovery_court.rs` — whole-media-loss
  DR over real `std::fs` media (6 courts). Receipt:
  `lanes/lane-rc06-media-loss-dr-court.md`.
- `cgap-storage/tests/journal_fault_matrix_court.rs` — derived
  fault-class matrix over the journal (3 courts). Receipt:
  `lanes/lane-rc07-journal-fault-matrix.md`.
- `cgap-bn3/tests/capacity_bound_court.rs` — red-12 SRAM capacity pins
  over bn3 state tables (3 courts). Receipt:
  `lanes/lane-rc03-bn3-capacity-bound.md`.
- `cgap-p28/tests/attestation_mac_court.rs` — keyed-MAC attestation
  courts over the 80 B P28 attestation (4 tests). Receipt:
  `lanes/lane-rc04-attestation-mac.md`.
- `cgap-ffi/tests/stats_block_court.rs` — stats-block layout (size
  120 / align 8, 15 pinned offsets) + sandbox mutation court (5
  tests). Receipt: `lanes/lane-rc05-stats-block.md`.
- `cgap-git/tests/ref_store_reentrancy_court.rs` — reentrancy courts
  over the real reserve path (5 courts; mutation battery exit 101).
  Receipt: `lanes/lane-rc01-ref-store-reentrancy.md`.
- `cgap-git/tests/ref_store_width_arms_court.rs` — width-gated atomic
  arm courts (5 legs, exit 0). Receipt:
  `lanes/lane-rc02-ref-store-width-arms.md`.
- `cgap-git/tests/andon_liveness_court.rs` — andon liveness over
  ingress/egress escalation (5 tests post-ADV-12; 2 witnessed mutant
  flips). Receipts: `lanes/lane-fix04-andon-quota.md` (owner) +
  `lanes/lane-r-adv12fix-quota-repair.md` (3→5 expansion).
- `cgap-git/tests/pack_demux_poison_court.rs` — F1/F2 poison closure
  over pack_demux surplus/trailing arms (3 tests). Receipt:
  `lanes/lane-fix05-pack-demux-poison.md`.
- `cgap-ocel2/tests/tamper_court.rs` — ocel2 tamper falsifier court
  (5 tests). Receipt: `lanes/lane-ocel01-cgap-ocel2-adapter.md`.
- `cgap-ocel2/tests/roundtrip_court.rs` — column-projection roundtrip
  chain-linkage court. Receipt:
  `lanes/lane-ocel01-cgap-ocel2-adapter.md`.
- `cgap-fleet-ops/tests/courts.rs` — fleet-ops lifecycle / refusal /
  SLO / predicate courts (4 tests; mutation flips). Receipt:
  `lanes/lane-rc13-fleet-ops.md`.

Standing of §6 entries: each carries its owning receipt's command+exit
run at that lane's session; this file does not re-run them — cells are
as receipted, UNVERIFIED here until re-read at use time per repo law.
Line cites are as-of the 2026-11-02 on-disk tree.

## 7. Standing (supersedes the §5 tail note)

This file remains a corpus definition, not a run receipt. §5 courts are
UNVERIFIED until their eight-step receipts land in the named wave
slots. §6 wave courts are receipt-backed per the table above. Line
cites as-of 2026-11-01 (§2/§3/§5) and 2026-11-02 (§6) on-disk tree.
