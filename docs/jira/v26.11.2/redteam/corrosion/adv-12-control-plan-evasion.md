# Lane ADV-12 — Control-plan latch evasion & unthrottled-refusal exhaustion

Date: 2026-11-02 · Standing: **ALIVE** (all commands + exits below, this session)
Contract: READ-ONLY over `crates/`; /tmp probe consumer; NO git commands.

## 1. Identity
- Lane ADV-12, repo `/Users/sac/cgap` (branch main).
- Subject (read-only): `crates/cgap-git/src/ingress_ring.rs`,
  `crates/cgap-git/src/egress_ring.rs`,
  `docs/jira/v26.11.2/dflss/06-control-plan.md`,
  `docs/jira/v26.11.2/lanes/lane-fix04-andon-quota.md`,
  `docs/jira/v26.11.2/redteam/red-14-andon-dos-exploit.md`.
- Written: only this receipt.

## 2. Attack 1 — quota seam is observational (load-shedding gap)

Confirmed on the current tree with a /tmp probe crate (`/tmp/adv12`,
path-dep on `crates/cgap-git`, build root `/tmp/adv12-target`):

```
cd /tmp/adv12 && CARGO_TARGET_DIR=/tmp/adv12-target cargo run --release
# exit 0, all asserts passed, output:
flood: 2M refusals elapsed_ns=5923834 ns_per_refusal=2 tripped_at_flood_end=true \
  window=2000000 total=2000000
after ONE admitted frame: tripped=false window_refusals=0 total_refusals=2000000 \
  (escalated state self-cleared: true)
post-flood: 100k admit+drain cycles in 1277250 ns — ring admits at full rate
```

- After 2,000,000 class-101 refusals the quota is tripped, yet the ring
  still admits-and-refuses at full rate. 100,000 full admit+drain cycles
  completed in 1.277 ms (~12.8 ns/cycle). No mechanism in either ring
  gates admission on `tripped` — grep over both files: `tripped()` is
  never consulted by `push`/`pop`/`push_sieved`/`pop_record`. The only
  non-test consumers of the quota are the two rings themselves; all
  other references are court files.
- **Residual obligation honesty check**: the kernel does NOT shed load,
  and the docs SAY so. `ingress_ring.rs:93-100` states "OBSERVATIONAL
  ONLY — it never gates admission"; the FIX-04 receipt §3 states "The
  quota NEVER gates admission — no process-exit behavior added"; red-14
  §4 and control-plan §5.2 row 141 state the trip-gate is a standing
  recommendation. Implication-without-mechanism: **NOT FOUND** — the
  obligation transfer to the host is explicit in all three documents.
- **SPOILER-CONFIRMED (new, adjacent defect)**: `tripped` self-clears on
  the very next successful admission (`note_admission()` sets
  `tripped=false`). A host that polls `refusal_quota().tripped()` can
  MISS the escalation entirely — the reportable flood state survives at
  most one admitted frame. An edge-triggered report with a one-frame
  observation window is not a usable rate-limit signal for the host
  obligation the docs transfer. Edge→level conversion (sticky
  escalation, or a `since_tripped` generation counter) is the missing
  mechanism.

## 3. Attack 2 — counter bounds arithmetic

- `total_refusals: u64` never reset. Arithmetic (honest, two rates):
  - red-14 measured 0.54 ns/refusal → 1.85e9 refusals/s →
    2^64 / 1.85e9 = 9.97e9 s ≈ **316 years** of continuous max-rate
    flood to wrap.
  - This session's probe measured ~2.96 ns/refusal (black_box-guarded
    2M-refusal loop; the probe also `black_box`es each refused frame,
    which red-14's did not — not a controlled comparison, both orders
    of magnitude unthrottled) → 3.38e8/s → ≈ **1,729 years**.
  - Verdict: u64 lifetime wrap is UNREACHABLE at any realistic rate;
    NOT a defect.
- **SPOILER-CONFIRMED (new defect)**: `window_refusals: u32` with
  `window_refusals += 1` (`ingress_ring.rs:135`, `egress_ring.rs:165`)
  overflows at 4,294,967,295+1. Zero-admission flood rate: at
  1.85e9/s ≈ **2.3 s**; at 3.38e8/s ≈ **12.7 s** of unauthenticated
  caller-driven refusals. In debug builds (every `cargo test` run) the
  `+=` panics — an attacker-reachable kernel panic in the very seam
  FIX-04 added as a hardening. In release it wraps: the window count
  silently resets toward zero while `tripped` stays sticky, corrupting
  the reportable window. Fix classes: `wrapping_add` (typed wrap
  record) or saturating add.

## 4. Attack 3 — scoped latch claim (R-12)

Re-grep this session over both rings: zero `process::exit` hits, zero
breaker/halt/latch state on any refusal path. The refusal paths touch
only `quota.note_refusal()` + typed `Err` returns; ring state untouched
by construction. Control-plan §5.2 row (06-control-plan.md:141) scoping
claim — non-refusal-path latches at `crates/cgap-bn3/src/builtins.rs:199`
and `:431-443`, `crates/bench/benches/chains-seal.rs:293-295`, none
gating refusal — **holds on the current tree**. Fence re-run:

```
cd /Users/sac/cgap/crates/cgap-git && \
CARGO_TARGET_DIR=/tmp/adv12-target cargo test --test andon_liveness_court
# exit 0 — 3 passed; 0 failed
```

## 5. Verdicts

| Claim | Verdict |
|---|---|
| Quota never gates admission (full-rate refusal post-trip) | CONFIRMED (probe exit 0) |
| Docs imply kernel sheds load | NOT CONFIRMED — transfer explicit
  (ingress_ring.rs:93-100, fix-04 §3, red-14 §4, plan :141) |
| `tripped` usable as host rate-limit signal | SPOILER-CONFIRMED DEFECT
  — self-clears on one admitted frame (probe exit 0) |
| u64 lifetime wrap | UNREACHABLE (~316 y at 1.85e9/s; ~1729 y at
  3.38e8/s) |
| u32 window overflow | SPOILER-CONFIRMED DEFECT — panic in debug
  (~2.3-12.7 s of flood), wrap in release |
| R-12 latch scoping holds | CONFIRMED — fence exit 0, zero latch/exit hits |

## 6. Fences (must-not-regress surfaces this diff could break)

This lane wrote no code; fences re-run, not assumed: andon_liveness_court
exit 0 (3 passed) on the current shared tree. Full `cargo test -p
cgap-git` NOT run (other lanes' in-flight files; compile-abort is not a
failure count) — targeted fence stands in for it, same as the FIX-04
receipt's own scope.

## 7. Standing

ALIVE for the ADV-12 surface: three attacks executed this session
against the exact on-disk subjects, two new defects (tripped
self-clear; u32 window overflow) Spoiler-Confirmed with command+exit
evidence, u64 arithmetic recorded, R-12 scoping re-verified. The /tmp
probe crate `/tmp/adv12` and build root `/tmp/adv12-target` are lane
leases, deleted at lane end.
