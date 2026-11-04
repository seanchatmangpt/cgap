# AF-25 — G2 Methodology Convergence (v26.11.3)

- **Lane**: AF-25 IMPLEMENTATION, G2 methodology convergence (coordinator
  ruling executed; AF-17's two-axis audit incorporated).
- **Deliverable**: `crates/cgap-aoq/tests/cbmqs_gates.rs` — G2 gate
  function rewrite only.
- **Date**: 2026-11-03.

## Standing: ALIVE (release) / loud-SKIP (debug, by design)

## The ruling, executed

AF-10's G2 FAIL (717.0/719.7 ns vs the 100 ns ceiling) was a
methodology+profile divergence, not a codec regression. Two axes:

1. **Methodology (axis 1)**: the 100 ns/header ceiling is lawful only
   under the amortized-min method (AB-6: min-of-7×100k amortized
   per-header). AF-10's gate measured per-call wall and honestly
   FAILed. The amortized path is now implemented INSIDE the G2 gate
   function: warm loop of 100k encode+decode, then min-of-7×100k
   tight-loop batches per operation, that number fenced against the
   100 ns ceiling.
2. **Profile (axis 2, AF-17)**: the ceiling is RELEASE-only (the
   standard's gate law; AB-6's 86 ns was measured under
   `cargo test --release`). The G2 gate now pins profile explicitly:
   a debug-profile run measures both methodologies, prints them as
   context, and loud-SKIPs the fence (typed reason names AF-17 axis 2;
   never a fake pass). The court runs are `cargo test -p cgap-aoq
   --test cbmqs_gates --release`.

## Gate output (methodology column, both methodologies disclosed)

`amortized-min <x> ns [FENCED vs 100 ns] | per-call-median <y> ns
[RECORDED, unfenced] | profile <release|debug> | encode/decode breakdown`.

## Court: AB-6's number reproduces through this implementation

| Run | Profile | Amortized-min (fenced) | Per-call (recorded) | Verdict |
|---|---|---|---|---|
| pass 1 | release | enc 44.5 + dec 41.3 = **85.8 ns** — PASS | enc 42.0 + dec 42.0 = 84.0 ns | 7 PASS / 0 FAIL / 1 SKIP (G6, pre-existing typed skip) |
| pass 2 | release | enc 44.5 + dec 41.4 = **85.9 ns** — PASS | enc 42.0 + dec 42.0 = 84.0 ns | 7 PASS / 0 FAIL / 1 SKIP |
| context run | debug | enc 422.2 + dec 321.3 = 743.5 ns — loud SKIP | enc 458.0 + dec 333.0 = 791.0 ns | 6 PASS / 0 FAIL / 2 SKIP |

AB-6's receipted 86 ns (release) reproduces through my independent
implementation to within 0.3% (85.8/85.9 vs 86.0). The fence holds.
Per-call (84.0 ns release) is recorded unfenced alongside.

## Root cause (recorded per addendum (d))

AF-10's fail-honest G2 FAIL was caused by fencing a per-call, DEBUG-profile
measurement against a release-profile amortized ceiling: a
methodology+profile divergence, not a codec regression. Corroborated:
my debug context run measured 743.5 ns amortized — same magnitude as
AF-10's 717.0/719.7 ns through an identical method/profile; release
measures 85.8 ns on the identical code.

## Commands + exits (this session)

- `cargo test -p cgap-aoq --test cbmqs_gates --release -- --nocapture`
  — pass 1: `test result: ok`, verdict `7 PASS / 0 FAIL / 1 SKIP`,
  G2 PASS amortized 85.8 ns.
- Same command pass 2: `test result: ok`, verdict `7 PASS / 0 FAIL /
  1 SKIP`, G2 PASS amortized 85.9 ns.
- `cargo test -p cgap-aoq --test cbmqs_gates -- --nocapture` (debug,
  context run): `test result: ok`, verdict `6 PASS / 0 FAIL / 2 SKIP`,
  G2 loud-SKIP with typed reason; context 743.5 ns amortized /
  791.0 ns per-call.
- Release compile check: 1 warning in the test target — pre-existing
  (`drop(slab)` at cbmqs_gates.rs:239, G1b region, not AF-25-owned;
  untouched, disclosed).

## Other gates unregressed

All other gates green ×2 under release with identical measured values to
AF-10's landed run (G1 0 B/op, G1b >12M pushes/s, G3 flip witnessed,
G4 KATs+flips, G5 storm+FrameTooLarge, G7 measured-not-fenced 1270.1
ns/link). G6 remains the pre-existing loud typed SKIP (AB-17 not landed).

## Never ran git commands.
