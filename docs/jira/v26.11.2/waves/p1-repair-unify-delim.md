# p1-repair: unify.rs delimiter repair

## Finding (on-disk truth vs task snapshot)

The task snapshot reported `crates/cgap-bn3/src/unify.rs` as not compiling
(unclosed `(` at line 990, file ends 2663, +548/-124). On-disk reality at
lane start: the file is 2687 lines, `fn eval_chunk_into` sits at line 982,
the paren is closed, and the diff vs HEAD is +548/-100. The killed lane's
edit was evidently complete past the snapshot point before this lane ran.

## Verification ladder (all run this session, exact subject /Users/sac/cgap)

| Gate | Command | Exit | Result |
|---|---|---|---|
| build | `CARGO_TARGET_DIR=/tmp/p1-repair-target cargo build -p cgap-bn3` | 0 | Finished dev |
| tests | `CARGO_TARGET_DIR=/tmp/p1-repair-target cargo test -p cgap-bn3` | 0 | 119 unit + 7 lever courts, 0 failed |
| workspace | `CARGO_TARGET_DIR=/tmp/p1-repair-target cargo build --workspace` | 0 | Finished dev (pre-existing unused-dep warning, cgap-integration) |

lever_courts.rs anti-vacuity probe `l3_antivacuity_probe_drop_flips` passed.

## Contract checks

- Naming law: zero naming-law-violation matches in crates/cgap-bn3
  (grep -ric sweep, count 0).
- No git writes performed; read-only `git diff --numstat` only.
- Frozen pins untouched; no sleeps; no new crates; no DataFusion.

## Diff summary

`crates/cgap-bn3/src/unify.rs`: +548/-100 vs HEAD. This diff is the prior
lane's work, verified by this lane — not repaired by it.

```json
{"lane":"p1-repair","files_written":["docs/jira/v26.11.2/waves/p1-repair-unify-delim.md"],"commands":["cargo build -p cgap-bn3 (exit 0)","cargo test -p cgap-bn3 (exit 0; 119 unit + 7 lever courts passed)","cargo build --workspace (exit 0)"],"courts_run":["tests/lever_courts.rs x7 incl. l3_antivacuity_probe_drop_flips"],"standing":"ALIVE","divergences":["task claimed unify.rs non-compiling at 2663 lines / unclosed paren at 990; on-disk state already compiles (2687 lines, +548/-100 vs HEAD) — gate satisfied by verification, no edit needed"]}
```
