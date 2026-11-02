# W1-11 Bench Protocol Receipt

Lane: W1-11 (lane-runner), repo `/Users/sac/cgap` (canonical checkout), branch main.
Sole writes honored: `06-bench-protocol.md` + this receipt. No git commands run.
`/Users/sac/stardf` never touched.

## Files written

- `docs/jira/v26.11.2/06-bench-protocol.md` (149 lines, all ≤100 chars, H1
  `06-bench-protocol`, JSON footer validated with `json.loads` this session)
- `docs/jira/v26.11.2/waves/w1-11-bench-protocol.md` (this file)

## Method

Authorship was evidence-grounded: every file:line citation in the protocol was read from
disk this session via grep/sed before writing, not recalled from context.

## Commands (exits observed)

- `ls crates/bench/benches/` — exit 0; confirmed the five chains-* files exist.
- `grep -n` seed constants across `crates/bench/benches/chains-*.rs` — exit 0; CHAIN_SEED
  `0xC001_0000_5EED_0001` at chains-ingress.rs:54, CANONICAL_SEED
  `0xBF58_476D_1CE4_E5B9` at chains-bn3.rs:33, spread constant never a seed
  (bitslice.rs:687-699, read via sed).
- `grep -n divan crates/bench/Cargo.toml` — exit 0; `divan = "=0.1.21"` at :25, `harness =
  false` at :30-97.
- `grep -n "P3-16|P3-17|P3-18|P3-19" docs/jira/v26.11.1/waves/bench-chains-*.md` — exit 0;
  P3-16 ingress, P3-17 parquet, P3-18 bn3, P3-19 seal+sovereign confirmed.
- `awk length>100` sweep — exit 0 with zero over-length lines after two fix rounds.
- `json.loads` on the fenced footer — exit 0 ("JSON OK").

## Standing

Document standing: **PARTIAL_ALIVE** — the method-of-record document exists, is on disk,
verified (line lengths, JSON footer, citation spot-checks against real files). The method
itself is UNVERIFIED as a measured discipline until a v26.11.2 lever receipt is produced
under it; per the protocol's own law, all perf figures remain [TARGET-v26.11.2].
No courts were run — this lane is a documentation lane; the anti-vacuity requirement is
discharged by the protocol's §4 mutation law, which binds future receipts, and no court
could flip on a prose artifact without being decorative. Record: **no courts_run**.

## Divergences

- The dispatch named five chains-* files; disk also carries non-chains bench files
  (ingress.rs, kernels.rs, etc.) — protocol scopes to the five chains-* files per §1, as
  dispatched.
- Dispatch said "cite real file:line"; all citations verified against disk this session.
  One mid-write token-sweep fix: a `bbench` typo in the g07 filename was caught and fixed
  via Edit before finishing.
