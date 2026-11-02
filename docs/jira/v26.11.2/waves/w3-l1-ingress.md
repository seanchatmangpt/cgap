# W3-L1 — two-stage admission escalation (ingress prefilter/escalate)

Lane W3-L1 · repo /Users/sac/cgap · branch main · 2026-11-01

## Subject + diff

- `crates/cgap-gate/src/penstock.rs` — `FrameView` gains `prefilter: u64`
  + `escalate: bool`; `pub const ESCALATE_MASK: u16 = 0x00F0` (row-6
  reserved-flag family, A04 §6.3 header flags bits 4-7 widened); `pub
  fn prefilter8` (one unaligned LE u64 load of header bytes 0..8,
  XOR-shift + one wrapping multiply, mix constant
  `0x2544_5491_4F6C_DD1D` — NOT a frozen seed, hygiene note in-source);
  `ingress` signature unchanged; on ADMIT only: `view.prefilter =
  prefilter8(bytes)`, `view.escalate = flags & ESCALATE_MASK != 0`;
  row-6 divergence doc-commented on `ESCALATE_MASK` (row-6 frames stay
  CGAP_OK per A04 "ignored on read", now MARKED for escalation — policy
  signal, never a status change; A19 four-code register untouched,
  status-purity court still green).
- `crates/cgap-gate/tests/lever_courts.rs` — field inits + new court
  `l1_prefilter_escalation_policy_court` (clean admit → escalate false
  + prefilter == prefilter8; row-6 → CGAP_OK + escalate true;
  OFF_ATTESTATION+6 flip → still SPOLIATION). Named anti-vacuity
  mutation: escalate-always-true injection.
- `crates/bench/benches/chains-ingress.rs` — `stage3` →
  `stage3_escalated` (escalate → BLAKE3 fold; clean → view.prefilter);
  chain fold = a04 + st + (escalate ? digest : prefilter); preflight
  records `digest_fold` (escalated subset) + `prefilter_fold`; new leg
  `stage_prefilter_fold`; new preflight law: reject views carry
  prefilter == 0 && escalate == false, admit views prefilter ==
  recomputed prefilter8. `ingress_chain` consts/ItemsCount identical
  (65_536 / 1_048_576). Pool gains `escalated`/`clean` window vecs.
- `docs/jira/v26.11.2/02-pareto-levers.md` — L1 row re-measured.

## Disclosed cross-lane unblock fixes (SLA, disclose-in-both)

Additive FrameView fields break every `FrameView { payload: &[] }`
literal. Mechanical field additions (`prefilter: 0, escalate: false`)
at:
- `crates/cgap-core/tests/cross_lane_gate_court.rs` (3 sites)
- `crates/cgap-arrow/src/slab.rs` (3 sites)
- `crates/bench/bens/ingress.rs` — typo here, actual path
  `crates/bench/benches/ingress.rs` (2 sites)
- `crates/cgap-ffi/src/sovereign.rs` (1 site) — ALREADY PATCHED by the
  concurrent W3-R2 lane (their disclose note present in-file); no edit
  from this lane.

## Commands + exits (this session, CARGO_TARGET_DIR=/tmp/w3-l1-target)

- `cargo test -p cgap-gate` → exit 0 (11 lib + 3 lever_courts green).
- `cargo test -p cgap-gate --release` → exit 0 (11 + 3 green; §8.4
  fence court held; output captured, nocapture rows in prior run).
- `cargo build --workspace` → exit 0.
- `cargo bench -p bench --bench chains-ingress` → exit 0, all
  per-pass checksum asserts green (final generation re-run).

## Bench — before/after ns/frame (divan 0.1.21, bench profile, 100 samples)

Fastest column, 1,048,576-frame const (same method as the P3-16
baseline receipt; per-frame = ms·1000/1,048,576):

| stage | baseline (P3-16) | W3-L1 measured | ns/frame now |
|---|---|---|---|
| ingress_chain | 102.7 ms = 102.7 ns/frame | 10.69 ms | **10.20 ns/frame** |
| stage_penstock_admission | 9.66 ms | 8.087 ms | 7.71 ns/frame |
| stage_a04_project_validate | 6.64 ms | 4.75 ms | 4.53 ns/frame |
| stage_digest_fold (escalated subset) | 93.38 ms (all-admitted) | — (subset EMPTY) | 0.89 ns vacuous-zero leg |
| stage_prefilter_fold (new) | — | 0.5906 ms | 0.563 ns/frame |

- TARGET <15 ns/frame: **MET** — fastest 10.20, median 10.8 ms =
  10.30 ns/frame; both < 15. 102.7/10.20 = 10.1x vs the >=6.8x floor.
- Dispatch's 5.70 ns penstock-stage row: measured 7.71 ns/frame
  (divan fastest, this host) — recorded, not rounded to the dispatch
  number.
- Vacuity disclosure: the escalated subset is EMPTY on this corpus by
  construction — stage 1 refuses reserved-flag frames, so no frame is
  chain-admitted AND escalated; the flag-only policy therefore pays
  zero BLAKE3 on the chain's admitted path. The 0.89 ns digest leg is
  a vacuous-zero leg, named here (its 100-sample "409600 iters" table
  row is the tell). This is the policy's actual claim (digest bounded
  to the suspicious subset = none, on this corpus), not a hidden skip.
- Column note: the 102.7 baseline's divan column is unstated in
  02-pareto-levers.md; this receipt states its column (fastest) and
  also records median 10.30 ns/frame — target met on both.

## Flip court (named mutation: escalate-always-true)

/tmp sandbox /tmp/w3-l1-sb/cgap-gate (standalone manifest patch:
workspace-inherited fields inlined, sandbox-only):
- tree sha256 before/during/after:
  `ffee1f53…c12` (full: ffee1f5343d1170405e91c5a5d4ce11abd639b8ecf8815b76be90899fa594c12),
  sandbox copy sha-identical before mutation.
- BASELINE: `cargo test --test lever_courts` → 3/3 ok.
- MUTANT (`view.escalate = true;` injected): `l1_prefilter_escalation_policy_court`
  FAILED (2 passed / 1 failed) — FLIPPED.
- RESTORE: copied back, `touch` mtime bump, sha256-exact verified
  (restore_ok=sha256-exact printed in-session).
- Shared tree never mutated.

## Divergences

1. ESCALATE_MASK is the row-6 HEADER reserved-flag family (0x00F0),
   not an attestation-flags bit: every attestation reserved bit is a
   row-5 spoliation law and cannot legally mark an ADMITTED frame; the
   lever spec's "att_flags bit" reading is unsatisfiable without
   breaking the taxonomy courts. Divergence doc-commented in-source.
2. Escalated∧chain-admitted subset is empty on the frozen corpus
   (stage 1 refuses reserved flags) → digest leg vacuous-zero, named.
3. Dispatch named `waves/w3-l1-ingress.md`; the L1 plan doc named
   `waves/l1-ingress-escalation.md` — dispatch wins, this file.
4. Bench `stage3` renamed `stage3_escalated` per spec; `admitted`
   Pool field retained (cross-check basis) with allow(dead_code).
5. The mid-lane compile freeze (stardf-datalog / cgap-ffi, another
   lane's in-flight edits) blocked the first two bench attempts ~15
   min total; resolved by the owning lane, no edit by this lane.
6. One prior-session-session artifact: the dispatch's "keep
   ingress_chain consts/ItemsCount identical" held — 65_536 and
   1_048_576 unchanged.

## Standing

ALIVE — every gate test, bench, flip court, and workspace build ran
this session with the exits recorded above.
