# W1-02: CLI six-pack regen byte-identical replay

- Date: 2026-11-01, lane W1-02, repo /Users/sac/cgap (main), branch untouched.
- Subject: `/Users/sac/cgap/crates/cli` ggen six-pack regen contract.

## Command ladder (all exits witnessed this session)

| # | Command (cwd=crates/cli) | Exit |
|---|---|---|
| 0 | `sha256sum` 5-surface baseline | 0 |
| 1 | `rm Cargo.toml && ggen sync run` | 0 |
| 2 | `python3 manifest_patch.py` | 0 |
| 3 | sha256 run-1 digests | 0 |
| 4 | `rm Cargo.toml && ggen sync run` (2nd) | 0 |
| 5 | `python3 manifest_patch.py` (2nd) | 0 |
| 6 | `diff run1 run2 digest sets` | 0 (`DIGEST_SETS_IDENTICAL`) |
| 7 | `CARGO_TARGET_DIR=/tmp/w1-02-target cargo build` | 0, 13.62s |

## Digests (identical across baseline, run 1, run 2)

```
01ac7eae...a3597d  Cargo.toml
32f417ed...e3fb    src/generated_cli.rs
0518fc90...56fa    src/generated_behavior.rs
9f859e87...4d76b   src/generated_boundaries.rs
b247b2db...3dd571  tests/zero_code_cli.rs
```

Full hashes in /tmp/w1-02-digests-run1.txt and run2.txt (byte-equal).

## Anti-vacuity

Mutation = delete `Cargo.toml` before each run. A regen that failed to
reproduce the manifest would surface as a diff/patch failure; run-1
`decisions` shows `Cargo.toml`, `generated_cli.rs`, `zero_code_cli.rs`
written, others `unchanged: content identical` — the writer actually
wrote, and output equals the committed bytes.

## Notes

- ggen 26.10.11, graph_hash_hex 881c22a9e00fff...029a5c, closure pinned
  to `~/ggen-marketplace` packs (see run 1 stdout receipt above).
- Build proven with a lane build root, deleted at lane end.
- Standing: ALIVE (regen replay + patched-manifest compile, exit 0 each).

## JSON

```json
{"lane":"W1-02","files_written":["docs/jira/v26.11.2/waves/w1-02-cli-regen-replay.md"],"commands":["ggen sync run x2 (exit 0)","manifest_patch.py x2 (exit 0)","cargo build (exit 0, 13.62s)","sha256/diff digest sets (identical)"],"courts_run":["regen-replay digest court (delete-manifest mutation, byte-identical restore)"],"standing":"ALIVE","divergences":[]}
```

## ADDENDUM (W2-19, 2026-11-02) — D5

D5 (from the W1-17 audit receipt): the digest evidence above was
/tmp-parked, not embedded. Re-run this session, contract executed x2:
`rm Cargo.toml && ggen sync run` (exit 0, run 1) +
`python3 manifest_patch.py` (exit 0, run 1); identical for run 2
(sync exit 0, patch exit 0). `cmp` of the two 5-surface digest sets:
exit 0. Embedded digest table (sha256, all three states identical —
baseline == run 1 == run 2):

| surface | sha256 (first 16) |
|---|---|
| Cargo.toml | 01ac7eae dab05c8c |
| src/generated_cli.rs | 32f417ed f028c1a5 |
| src/generated_behavior.rs | 0518fc90 19adc03e |
| src/generated_boundaries.rs | 9f859e87 53611ff6 |
| tests/zero_code_cli.rs | b247b2db 7131e1bc |

The original receipt's standing (ALIVE) is confirmed by this re-run.
Evidence source: W1-17 audit receipt (D5).
