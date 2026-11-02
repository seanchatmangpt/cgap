# Lane G12 — csct warmer.rs independent verification

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout, no git writes).

## Subject

`crates/csct/src/warmer.rs` — CSCT Stage 1 "THE WARMER" (RFC-0042 §4 baseline model).

## Steps and commands (all run this session)

1. **sha256 + integrity.** Pre-edit hash:
   `43b3e9b9bb36cdaf2ba4f77d1a13de7c65581e940d393373413e202087d38a84`
   (178 lines). Read the full file: balanced braces, module closed at
   line 178, complete `#[cfg(test)]` block. No truncated-file signature.
   No heredoc / placeholder / TODO / `unimplemented!` tokens found.
2. **Warmer courts.**
   `cd /Users/sac/cgap/crates/csct && CARGO_TARGET_DIR=/tmp/g12-target cargo test warmer`
   → exit 0, **2/2 passed** (`warmer_real_run_passes`,
   `mutated_params_cannot_satisfy_ingest`), 35 filtered out.
3. **Full suite (twice).** `cargo test` twice in crates/csct with the same
   lane build root → exit 0 both runs; 37 lib + 7 + 7 + 0 = 51 tests, 0
   failures both runs. Confirms the dispatch's stale-premise notice.
4. **Anti-vacuity witness.** Read `mutated_params_cannot_satisfy_ingest`
   (warmer.rs:156-177). Premise correction: the dispatch said
   "1360→136 × elapsed"; the actual mutation is **shards 1360→136 AND
   elapsed 1000→1** (`ELAPSED_SECONDS / 1000`), yielding
   `136 × 1000 × 1 = 136,000 < 1,000,000` — the mutated model genuinely
   cannot satisfy `ingested == 1_000_000`, so the flip direction is real.
   Added a trivially safe **third arm** (elapsed=0):
   `baseline_ingest_capacity(136, 1000, 0) == 0` and
   `0.min(1_000_000) < 1_000_000`. Post-edit hash:
   `80f5353b3a282275f753cb690efb7d7fd917af2703bba03a7847b4fabc3ce554`.
5. **Stage-1 receipt digest determinism.** Built a throwaway probe crate
   at /tmp/g12-probe (path-dep on csct, deleted after) that prints
   `csct::warmer::warmer().receipt` hex. Three independent process runs:
   `1b6212dfd6da9c04cc7b4681f1741b1c2f002dfe2a29ef821d2194af63e162cd`
   — identical across all three. This probe ran AFTER the third-arm edit
   landed, so it is the post-edit digest; and the third arm is
   `#[cfg(test)]`-only, so it does not perturb the production receipt
   path either way.

## Files written

- `docs/jira/v26.11.2/lanes/lane-g12-csct-warmer-verify.md` (this file)
- `crates/csct/src/warmer.rs` — third arm added to
  `mutated_params_cannot_satisfy_ingest` (test-only, authorized by
  dispatch step 4). No other file touched.

## Fences (must-not-regress, run)

- Full `cargo test -p csct` surface: 37/37 lib + 7 + 7 integration, exit 0,
  both runs, AFTER the third-arm edit landed. No fence regressed.

## Standing

ALIVE for the court runs, full-suite runs, and determinism probe
(commands + exits above, this session, exact subject, post-edit build).
No UNVERIFIED residue.

## Divergences

- Dispatch premise said the mutation is "1360→136 × elapsed makes ingest
  < 1M"; actual test mutates BOTH shards (÷10) and elapsed (÷1000).
  Direction still flips (136,000 < 1,000,000); recorded here, not "fixed".
