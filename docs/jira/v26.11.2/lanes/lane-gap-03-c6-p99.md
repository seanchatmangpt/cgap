# lane-gap-03-c6-p99 — Receipt (2026-11-02)

Finding: same defect class as FIX-04 — in
`crates/csct/src/bin/cscb_c6_retraction.rs`, `secs[6]` over N=7 sorted
run-level samples labeled "p99" is the MAX
(ceil(0.99*7)=7), mislabeled as a tail quantile in both the timing and
throughput output lines and the doc comment.

## Fix (owned file only)

`crates/csct/src/bin/cscb_c6_retraction.rs`

- `p99` -> `max` in both output lines; labels carry `runs=7`; both lines
  plus the doc comment state explicitly that no p99 is computed
  (run-level population of 7 is too small for a tail quantile).
- Doc comment (`//!` unit law) and the MV-5 gate-const provenance comment
  updated to match (old "p99 129,554,655.9" figure relabeled max).
- New honest variance report line (task 2), citing as-of-receipt figures
  from prior lanes: B05 min 248.1M facts/s
  (`lane-b05-cscb-c6.md:42`), GIT-11 min 323.8M
  (`lane-git-11-retract.md:86`), loaded-host dip 87.8M
  (`lane-b05-cscb-c6.md:45-46`); spread ~3.7x, framed as a measurement
  property, not a hidden defect.
- Gate section untouched (task 3 constraint): `DECIDED_GATE_FPS`
  (150M, min-of-N) unchanged — gate behavior unchanged, verified by the
  runs below.

## Commands + exits (this session, exact subject)

- `cd crates/csct && CARGO_TARGET_DIR=/tmp/gap03-target cargo build
  --release --bin cscb_c6_retraction` -> exit 0.
- `CARGO_TARGET_DIR=/tmp/gap03-target cargo run -q --release --bin
  cscb_c6_retraction` -> exit 0. Output: `C6 timing runs=7
  min=501.916µs median=613.042µs max=721.25µs (no p99: run-level
  population of 7 is too small for a tail quantile)`; throughput
  min=199236525.6 median=163120960.7 max=138648180.2 facts/s runs=7 (no
  p99 computed); `C6 GATE PASS: min 199236525.6 >= 150000000 facts/s
  (decided gate, lane MV-5 re-scope)`. Labels honest; gate unchanged.
- `CSCB_MUTATE=skip-sweep ... cargo run -q --release --bin
  cscb_c6_retraction` -> exit 1; `C6 MUTANT skip-sweep + 10 injected
  phantoms` + `C6 integrity FAIL: 10 phantom survivors detected` —
  phantom-flip court flips (task 4).
- Build-root lease `/tmp/gap03-target` deleted at lane end (witnessed:
  ls -> No such file or directory).

## Divergences

- `audit/dod-csct-cscb.md` does not exist on disk (no `audit/` dir at
  repo root). Dispatch named it as context; recorded here, not worked
  around.
- Gate consts on disk are MV-5's re-scope (`DECIDED_GATE_FPS = 150M`
  decided gate + 800M aspirational, reported loudly not gated), NOT
  B05's receipted 800M gate. Worked with what's on disk per dispatch;
  MV-5's re-scope comment block was left byte-identical except the one
  provenance figure relabel ("p99" -> "max") required by the repair.

## Fences

Could not regress: C6 decided gate (min-of-N vs 150M, untouched),
oracle-equality sweep court, phantom-flip mutant court (re-run, flips),
cscb_c1_c2_ingress.rs honest p99 (NOT touched — its p99 is genuine over
~15,625 batch samples per DF-1, per FIX-04's sibling audit and this
lane's dispatch).

Standing: ALIVE (all commands + exits run this session).
