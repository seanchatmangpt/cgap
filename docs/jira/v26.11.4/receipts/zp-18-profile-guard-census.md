# ZP-18 — Profile-Guard Census Gate Gap-Close Receipt
Standing: PARTIAL

Date: 2026-11-04. Lane: GAP-CLOSE. Status: ALIVE (census gate + self-check
re-run this session, both exit 0).

## Gate receipts (ALIVE — this session)

- `python3 scripts/profile_guard_census.py` -> exit 0:

      profile-guard census: 73 TIMING files, 64 guard-composing files,
      11 allowlist entries
      PASS: all 10 unfenced TIMING file(s) allowlisted

  followed by the 10 allowlisted-file list (mutation_kills, kill_stress,
  zero_alloc, edge_closure_courts, aoq_sidecar_court,
  aoq_sidecar_e2e_court, fa9_commit_gating, offset_fuzz, robustness,
  fibo wave9_integration_courts).

- `python3 scripts/profile_guard_census.py --self-check` -> exit 0,
  stdout `SELF-CHECK OK: poison HIT (unguarded TIMING), clean PASS
  (guarded)` — the non-vacuity witness: the gate is proven able to REFUSE.

Note on trajectory: the "56 -> final" counts (56 TIMING files at gate
birth, grew to 73 as the wave landed) are session-only — no on-disk doc
records the 56 start (verified by grep this session, disclosed as
PARTIAL). The current counts above are ALIVE.

## profile_guard compose

- `crates/cgap-aoq/src/profile_guard.rs` — `:23 struct ProfileViolation`,
  `:82 pub fn assert_release() -> Result<(), ProfileViolation>`, `:95
  pub fn fence<T>(value: T) -> T` (`core::hint::black_box` anti-DCE
  wrapper).
- Registered at `crates/cgap-aoq/src/lib.rs:27-29` ("RX-4: reusable
  release-profile detector + anti-DCE fence").
- Origin receipt (cited, PARTIAL): `/Users/sac/cgap/docs/jira/v26.11.3/
  receipts/rx-4-profile-guard.md` (AF-25 G2 + AF-10 G1b migrations).
- Compose witness in the field (cited, PARTIAL):
  `docs/jira/v26.11.4/receipts/zp-17-gate10-settling.md:21-24` —
  receipt_chain_scale composes `assert_release` (5 sites) + `fence`.

## Census gate design + fail-closed rules

`scripts/profile_guard_census.py`:

- `:2` "fail-closed timing-fence census (ZP-18 gate)".
- Mechanical subject discovery: workspace members from `cargo metadata`,
  never a hardcoded crate list (`:60+` docstring).
- Classification predicate `classify()` (`:49-56`): TIMING iff content
  contains any of `Instant::now`, `std::hint::black_box`,
  `Instant::elapsed`; guarded iff it composes
  `profile_guard::assert_release` or `profile_guard::fence`.
- Exit ABI 0 pass / 1 gate-fail / 2 subject-missing / 3 loud-skip (`:12`).
- Fail-closed rules (`:14-18` docstring): allowlist entry for a
  nonexistent path -> exit 1; entry missing `ticket` or `reason` -> exit 1;
  allowlisted file that is ALSO fenced -> exit 1 (allowlist stays
  minimal); unfenced TIMING file not allowlisted -> exit 1 (file list
  printed).

## Self-check non-vacuity (poison HIT)

- Docstring `:20-21`: `--self-check` builds poison/clean twin fixtures in
  a temp workdir and runs the production predicate on both; exits nonzero
  printing `VACUOUS GATE` unless poison HITS and clean passes.
- Witness at `:196`: `SELF-CHECK OK: poison HIT (unguarded TIMING), clean
  PASS (guarded)` — reproduced this session, exit 0.
- Wired into the gate ladder: `scripts/verify_v26.10.31.sh:183-187` —
  `run_gate profile-guard-census python3 scripts/profile_guard_census.py
  --self-check`, commented "ZP-18: timing-fence census non-vacuity
  (poison HIT + clean pass). The full tree census is
  scripts/profile_guard_census.py run bare".

## Mutation-probed fail-closed rules

The four refusal rules above are each a mutation probe by construction:
poison an allowlist path, strip a ticket, double-register a fenced file,
or add an unfenced TIMING file — each mutation must flip the gate to
exit 1, and the poison twin in `--self-check` proves the predicate
actually bites (a gate that cannot flip is FAIL, a skip is loud).
The self-check's poison HIT witness (this session) is the executed
non-vacuity proof; the four rule-by-rule flip witnesses from the lane's
own probe runs are session-only and stand PARTIAL — the rules' text is
ALIVE on disk at the line numbers above.

## Allowlist entries

`scripts/profile_guard_allowlist.toml` — 11 entries, all
`added = "2026-11-04"`, each with path/ticket/reason:

- `ZP-18-C` — `crates/bench/benches/kernels.rs` (divan-no-per-bench-skip).
- `ZP-18-D` — `crates/cgap-blake3/tests/zero_alloc.rs` (alloc-court-only;
  `black_box` is DCE-prevention, never a timing fence).
- `ffi-ZP18-D-1/2/3` — `crates/cgap-ffi/tests/fa9_commit_gating.rs`,
  `aoq_sidecar_court.rs`, `aoq_sidecar_e2e_court.rs` (every `Instant::now`
  is a bounded-poll DEADLINE per the no-sleep doctrine, never a duration
  measurement).
- `ZP18-D-residual-1..6` (GAP-CLOSE wave): cgap-arrow `mutation_kills.rs`
  (bounded-poll deadline), cgap-beam `kill_stress.rs` (subprocess
  wall-time diagnostic), cgap-bn3 `edge_closure_courts.rs` (black_box DCE
  prevention), cgap-parquet `offset_fuzz.rs` + `robustness.rs` (corpus
  wall-time diagnostics), `ontology-packs/fibo/tests/
  wave9_integration_courts.rs` (advisory elapsed print, never a gate).

## B3 one-per-line-import predicate finding

Disclosed honestly: the "B3 one-per-line-import predicate" finding is not
recoverable from the cgap repo — no script, fixture, or doc names it
(verified by grep this session). The closest on-disk artifact is the
boundary gate's per-line scanning notes
(`scripts/boundary_gate.py:386,410`, which concern multi-line string
literals, not imports). The finding, if it was a receipt-lint predicate
proposal, died with the session; it is recorded here as UNVERIFIED, not
rounded up into a landed item. Any future lane claiming it must re-derive
it against `scripts/receipt_lint.py` (which currently carries no such
rule).

## Standing summary

- Census gate PASS + self-check poison HIT: ALIVE (this session's exits).
- Current counts (73 TIMING / 64 guarded / 11 allowlist): ALIVE.
- 56-start trajectory, rule-by-rule mutation flip receipts: PARTIAL
  (session-only).
- B3 import-predicate finding: UNVERIFIED (no on-disk referent).
- Nothing BLOCKED.
