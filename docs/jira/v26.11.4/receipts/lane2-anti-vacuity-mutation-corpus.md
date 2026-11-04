# Lane-2 Anti-Vacuity Mutation Corpus (v26.11.4)

Status: ALIVE — every corpus leg executed this session against sandboxed
subjects; meta-court PASS (exit 0) with witness receipts; meta self-check
flips. Supersedes the unwitnessed draft corpus scripts of 39b53add
(same file family, this lane's assignment) with the witnessed versions.

## Artifacts

- `scripts/mutation_corpus/lib.sh` — sandboxed shim-court harness
  (generalizes smalt `scripts/shim_court/lib.sh`): STAGE (git archive
  HEAD) → BASELINE sha → BEFORE (gate green on pristine sandbox) → SHIM
  (caller `apply_mutation`, SHIM-COURT marker enforced) → AFTER (gate must
  flip, else COURT-FAILS vacuity receipt) → RESTORE sha256 fail-closed.
  Receipt JSON per mutation in `receipts/`.
- `scripts/mutation_corpus/m{1..5}_*.sh` — corpus legs (below).
- `scripts/mutation_corpus/registry.json` — the register (15 cells).
- `scripts/mutation_corpus/meta_court.py` — mechanical sweep (census
  allowlist tickets + scripts/ basename regex) + 100% coverage court with
  `--self-check` receipt-drop injection.
- `scripts/mutation_corpus/receipts/*.json` — 9 witnessed receipts.
- `scripts/profile_guard_allowlist.toml` — baseline repair: new
  `ffi-ZP18-D-5` ticket for `crates/cgap-ffi/tests/aoq_flip_ladder_court.rs`
  (see Baseline repair).

## Run (2026-11-04, ~21:10-21:40 UTC)

```
MC_SANDBOX=/tmp/lane2-final-sandbox MC_CARGO_SANDBOX=/tmp/lane2-final-cargo \
MC_TARGET_DIR=/tmp/lane2-final-target
for m in m1_alloc_inject m2_digest_bitflip m3_version_skew \
         m4_census_debug_profile m5_allowlist_tamper; do
  ./scripts/mutation_corpus/$m.sh; rc  # rc=0 for all five legs
done
python3 scripts/mutation_corpus/meta_court.py            # rc=0 PASS
python3 scripts/mutation_corpus/meta_court.py --self-check  # rc=0 flip OK
```

## Mutations witnessed

| mutation | court | flipped | restore sha-verified |
|---|---|---|---|
| m1 steady-state 1-byte alloc into `cgap_blake3::hash` | `crates/cgap-blake3/tests/zero_alloc.rs` (counting GlobalAlloc court) | YES (3 tests fail, exit 101) | yes |
| m2 uniform 1-bit digest corruption of `hash` output | `chain_mutation_court.rs` | **NO — COURT VACUOUS (registered defect)** | yes |
| m2 (same mutation) | `official_vectors.rs` official-vector pin | YES (exit 101) | yes |
| m3 `AOQ1_VERSION` 3→2 in the GENERATED layout (sandbox only) | `aoq1_layout_drift_court.rs` | YES (exit 101) | yes |
| m4 debug-profile substitution (assert_release + all fence markers out) of `crates/cgap-blake3/tests/percall_latency.rs` | `profile_guard_census.py` | YES (exit 1) | yes |
| m5 duplicate allowlist entry | `profile_guard_census.py` allowlist validation leg | YES (exit 1) | yes |
| built-in: census `--self-check` | poison HIT / clean PASS | YES | n/a |
| built-in: enclave budget `--self-check` | clean PASS / 32 KiB poison refuses | YES | n/a |
| built-in: gate battery cscb-sram bn3-pin mutate leg | mutant refused (exit 1) | YES | n/a |

## Findings (registered defects / gaps)

1. **chain_mutation_court vacuity (defect, unowned — owner to repair):**
   the chained-receipt court re-derives every digest with the SAME hash
   function it pins; a uniform output corruption cancels on both sides of
   `hash(&r.frame) != r.digest` and the court stays green. Receipt:
   `scripts/mutation_corpus/receipts/m2_chain_digest_bitflip.json`
   (`flipped: false`, `court_vacuous: true`). Repair direction: compose the
   official-vector pins into the chain court (cross-substrate pin), owner
   lane decision.
2. **census blind spot (registered gap):** `profile_guard_census.py` scans
   only `tests/`, `benches/`, `src/bin` — `crates/bench/src/main.rs`
   composes `Instant::now` + `assert_release` entirely outside the gate's
   subject surface. Witnessed when the first m4 draft (substituting only
   `bench/src/main.rs`) failed to flip the census. Census scope law is the
   census owner's to widen; disclosed, not repaired by this lane.
3. **Standing census RED at HEAD (repaired by this lane):**
   `crates/cgap-ffi/tests/aoq_flip_ladder_court.rs` (39b53add) landed as
   unfenced TIMING with no allowlist ticket — census RED on the canonical
   tree before this session's repair. Added ticket `ffi-ZP18-D-5`
   (bounded-poll deadline class, matching siblings D-1..4). Census now
   GREEN (exit 0, 77 TIMING files, 12 allowlist entries).

## Plane disclosures

- Alloc fence court: cgap-local (`crates/cgap-blake3/tests/zero_alloc.rs`).
  The arcus ZP-19 court does not exist in `/Users/sac/arcus` (no ZP-19
  referent found); not sandbox-runnable from cgap. Disclosed plane:
  cgap-local.
- Version-skew subject is the GENERATED projection
  `crates/cgap-aoq/src/generated_aoq_header_layout.rs`, mutated only in the
  git-archive sandbox, never in place (frozen-file law). The real-tree copy
  is untouched; sha256 restore-verified in the sandbox.
- The AOQ1 layout court reads the landed ontology at an absolute path
  (`/Users/sac/ggen-marketplace/packs/aoq-pack/ontology.ttl`); sandbox legs
  therefore depend on that cross-repo file (read-only).

## Mid-wave / pre-existing notes

- gate_battery aggregate exit 1 is PRE-EXISTING (ffi-surface
  `cgap_aoq_session_step` C-prototype gate-fail from another lane's
  in-flight surface work); the cscb-sram bn3-pin mutate leg used as this
  corpus's witness is PASS and independent of it.
- Registry EXCLUDED cells (8) are typed with tickets LANE2-EXC-1..8 and
  reasons (hardware-timing gates with no in-repo anti-vacuity mode;
  advisory canaries; reporting tools). Meta-court renders them loudly and
  drift (a swept gate missing from the registry) is exit 1.

## Untracked-evidence note

`scripts/mutation_corpus/receipts/*.json`, `registry.json`, and
`meta_court.py` were generated this session and are committed with this
receipt (evidence a fresh checkout needs). Scratch sandboxes
(`/tmp/lane2-sandbox`, `/tmp/lane2-cargo`, `/tmp/lane2-m2cargo`,
`/tmp/lane2-final-sandbox`, `/tmp/lane2-final-cargo`,
`/tmp/lane2-final-target`, `/tmp/lane2-cargo-target`, `/tmp/lane2-sb-cargo`,
`/tmp/lane2-scratch`, plus `/tmp/lane2-*.log`) could not be `rm`-deleted:
`rm` is permission-denied in this session — coordinator cleanup list.
