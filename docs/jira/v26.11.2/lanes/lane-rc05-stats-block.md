# Lane RC-05 — Runtime stats block (FIX-14 A1 / RED-16 A1)

Date: 2026-11-02. Repo: /Users/sac/cgap (canonical checkout, branch main).
Remediates: `redteam/red-16-headcount-paradox-audit.md` finding A1
(BREACH-CONFIRMED: zero runtime telemetry) per
`redteam/fix-14-ops-requirements.md` §A1.

## Files written (ownership honored)

- `crates/cgap-ffi/src/stats.rs` (NEW, sha256
  c09624cd6a6cb2da866874204bc86ef7e54127aebb399ead0fabf5f2e5f17f4b) —
  `CgasStatsBlock` (repr(C), 120 B, CRC'd per fix-14 R-A1.2),
  `StatsRecorder` (opt-in writer), `Record` trait hooks, `RefusalClass`
  (six-row taxonomy mirror + Class101), `QuotaState`, `health()`
  typed poll verdict, `reset()`, `extern "C" cgap_stats_snapshot`
  (R-A1.2/R-A1.3 pull-based getter, CgapStatus error channel).
- `crates/cgap-ffi/src/lib.rs` — one additive block: `pub mod stats;`
  (3 doc lines + 1 decl, end of mod list).
- `crates/cgap-ffi/tests/stats_block_court.rs` (NEW) — 5 courts.

## Court receipts (W5/W6)

- Layout court: size_of==120, align_of==8, all 15 field offsets pinned
  (0..116) via `core::mem::offset_of!` const asserts.
- `counters_advance_on_recorded_events` — final-state counters across
  admitted / each taxonomy bucket / quota publish / receipt digest.
- `health_verdict_degrades_on_tripped_quota_and_crc_corruption`
  (R-A1.4 verdict law) — tripped quota -> `Health::QuotaTripped`;
  single counter-byte flip -> `Health::CrcInvalid`; forged CRC refuses.
- `reset_returns_all_counters_to_zero` — counters, quota, digest all
  zero, header intact, counters count again from zero.
- `c_abi_getter_seals_and_typed_refuses_null` — getter overwrites a
  poisoned buffer, returns CGAP_OK(0); null out -> nonzero typed refusal.
- `global_recorder_is_live_and_snapshot_backed` — global recorder
  advances receipts_minted and serves digest through the snapshot.

## Anti-vacuity mutation court (R-A1.4, sandbox law)

- Sandbox: /tmp/rc05-sandbox-ffi (crate-only copy; path deps
  re-pointed at the canonical checkout; manifest de-workspaced).
  Shared tree NOT copied (topology guard refused full-repo rsync —
  recorded as a divergence; the crate-level sandbox is the compliant
  narrowing).
- BASELINE: 5/5 pass. MUTANT (swap admitted <-> invalid_magic
  increments, 2 marked `// MUTANT` lines): `counters_advance...`
  AND `reset_returns_all_counters_to_zero` FAIL (2 flipped).
  RESTORE: sha256-exact verified (True) + mtime touch; court green
  again (5/5). Flip witnessed, gate is non-vacuous.
- Shared-tree sha256 verified unchanged by the mutation (mutate ran
  only in /tmp).

## Commands this session (standing)

- `cargo test -p cgap-ffi --test stats_block_court` -> 5 passed/0
  failed (exit 0).
- `cargo test -p cgap-ffi` -> exit 0: lib 52 passed/0 failed (incl.
  the xip T1/r2 rv32imc courts, which COMPILE cgap-ffi for
  riscv32imc — the new module is rv32imc-clean, no AtomicU64: the
  target has no atomic RMW, so the recorder uses a documented
  single-writer Cell64, not atomics), plus 7 integration targets,
  all 0 failed. Total 80 passed / 0 failed. The dispatch's stated
  "49/50 baseline with one known foreign wave9_a07 failure" did not
  reproduce — wave9_a07 passed (52/0 lib); recorded as divergence.
- Mutation runs: mutated sandbox -> exit 1 (2 failed); restored
  sandbox -> exit 0 (5 passed).

## Divergences / fences

- Divergence 1: dispatch promised ring-record integration "if a
  zero-cost default recorder compiles without ring edits" — delivered
  as `stats::global()` + `Record` hooks + the one-line integration
  doc in stats.rs; ingress_ring.rs/egress_ring.rs NOT touched
  (another lane's files per contract).
- Divergence 2: full-repo /tmp sandbox rsync REFUSED by the topology
  guard; narrowed to a crate-level sandbox with path deps re-pointed
  absolute — same flip semantics, shared tree untouched.
- Divergence 3: riscv32imc has NO atomic RMW (AtomicU64 doesn't even
  exist there; probe command run this session). The recorder therefore
  uses `Cell64` under an explicit kernel single-writer law (module
  doc), not atomics. This is the only design that compiles on both
  host and XIP without a new dependency.
- Fences audited: xip T1/r2 rv32imc compile+size courts (pass — my
  module had to be rv32imc-clean and was caught failing exactly there
  mid-lane, then fixed); c_header_abi_court (pass); fa9_commit_gating
  (pass); boundary/zero-string surfaces untouched (no edits outside
  owned files; lib.rs delta is one additive mod decl).
- No git commands. Build-root leases /tmp/rc05-target,
  /tmp/rc05-mut-target, /tmp/rc05-sandbox-ffi deleted at lane end.

## Standing

- ALIVE — every claim above carries a command + exit run this session.
- Integration note for the ring owner (fix-14 owner recommendation
  honored: stats live in cgap-ffi, ring wiring is a 1-line call):
  `cgap_ffi::stats::global().record_admitted()` /
  `record_refused_class(RefusalClass::Class101)` /
  `record_quota(...)` / `record_receipt(&digest)` at the ring's
  admission/refusal/quota/receipt sites.
