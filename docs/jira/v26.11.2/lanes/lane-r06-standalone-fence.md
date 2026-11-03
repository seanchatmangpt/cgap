# Lane R-06 — standalone crates fence (read-only tollgate)

- **Lane**: R-06, read-only fence. No source edits; no git commands.
- **Repo**: /Users/sac/cgap, branch main.
- **Files written**: this receipt only.
- **Subject**: RC-13's `crates/cgap-fleet-ops` + standalone `crates/cli`;
  prior receipt `docs/jira/v26.11.2/lanes/lane-rc13-fleet-ops.md`.
- **Commands + exits (this session)**:
  1. `cd /Users/sac/cgap/crates/cli && cargo build` → exit **0**
     (Finished dev profile in 19.52s). [CARGO_TARGET_DIR=/tmp/r06-target]
  2. `cd /Users/sac/cgap/crates/cli && cargo test` → exit **0**;
     44 tests total, 0 failed (12 lib + 25 + 6 + 1 doc-derived suites,
     5 empty suites).
  3. `cd /Users/sac/cgap/crates/cgap-fleet-ops && cargo test` → exit
     **0**; 4 courts green, 0 failed (court1 legal lifecycle,
     court2 illegal refusals, court3 SLO round-trip, court4 predicate +
     bounded poll). Matches RC-13's receipted counts.
  4. `cargo metadata --no-deps` at root → exit 0; exactly **23
     members**, `cgap-fleet-ops` ABSENT (standalone-by-design holds;
     the crates/cli pattern is not violated).
  5. `cargo build -p cgap-git --manifest-path
     /Users/sac/cgap/crates/cgap-git/Cargo.toml --target
     riscv32imc-unknown-none-elf --no-default-features` → exit **0**
     (1.05s, 11 warnings, no errors) — re-witnesses RC-02's riscv
     no_std witness on this session.
- **Courts run**: fleet-ops court1–court4 green on shared tree.
  Mutation flips were witnessed by RC-13 in its own sandbox (cited
  from its receipt; NOT re-run by R-06 — fence lane, read-only; those
  flips stand as RC-13's evidence, re-read from its on-disk receipt at
  use time).
- **Standing**: ALIVE (all five commands + exits above, run this
  session against the exact subjects).
- **Fences**: could-have-broken surfaces checked = root workspace
  membership (23, fleet-ops absent) and cgap-git riscv32imc no_std
  build. Both green. CLI test suite (44/0) and fleet courts (4/0)
  green. Zero regressions observed.
- **Cleanup**: `/tmp/r06-target` deleted via python rmtree at lane end.
- **Divergences**: none. RC-13 receipt's counts match on-disk behavior.
