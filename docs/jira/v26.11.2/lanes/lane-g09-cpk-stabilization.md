# Lane G09: cpk_study stabilization (sliding-window gates, exit contract)

Repo /Users/sac/cgap @ main, 2026-11-02. Write scope honored:
`crates/bench/src/bin/cpk_study.rs` + this receipt. `scripts/verify_cpk.py`
does NOT exist (scripts/ listing witnessed) — no adapt needed; ownership
stayed cpk_study.rs, nothing else touched.

## Drift audit (task 1): 0.855 vs today's run

Repair-13 (docs/jira/v26.11.2/repairs/repair-13-cpk-study.md:36) recorded
CTQ-06 Cpk(overall) 0.855, mean 1.66 GiB/s, sigma_overall 0.26, n=100,
256 MiB hash_many_simd samples. Today's rig run measured mean 1.84 GiB/s,
sigma_overall 0.0097, n=30 — same leg code, same input size, same seeds.

Root cause (one paragraph): the 0.855 was a measurement-drift artifact of
concurrent wave load, not a capability change. The repair-13 receipt itself
localizes the miss to between-sample drift: within-sigma Cpk was 2.131
(clears 1.67) while overall-sigma was 0.855, i.e. slow drift between
samples — the signature of a contended M3 Max during the 15-lane wave
(lift register: "Cpk CTQ-06 MEASURED-FAIL (drift): re-run after load < 2
or accept", docs/jira/v26.11.2/waves/w14-lift-register.md:43). The measured
hash kernel (cgap-blake3 hash_many_simd) and the input (256 MiB, same
xorshift/LCG fill, byte-identity asserted) are unchanged since
0539fc7f — no code cause exists. Today's quiet-window run under load ~10
on idle P-cores: sigma 0.01, mean 1.84 GiB/s, Cpl 28.97. The dispatch's
claimed intermediate re-run (Cpk 1.572 / 1.72 GiB/s / CTQ-04 4.947) has NO
receipt on disk in this repo (docs-wide grep: only repair-13's 0.855;
the v26.11.1 bench-bn3-receipt "1572" hit is unrelated bytes math) — that
claim stays UNVERIFIED and is NOT used as evidence here.

## Changes to cpk_study.rs (tasks 2-4)

- Sliding-window trials: all three CTQ gates now run n=30 windows
  (WINDOW=30; was n=200/100/100). Per-run throughput/time, Cpk over the
  window, mean/sigma/Cpk reported per gate in JSON v2 (new fields
  cpk_exit_floor, window_n, anti_vacuity_injection).
- Exit contract pinned: any gate Cpk(overall) < CPK_EXIT_FLOOR=1.33 ->
  exit 1; all >= 1.33 -> exit 0, fail-closed. Charter target 1.67 kept as
  reported `cpk_target` context only. Per-gate verdicts now judged at the
  1.33 floor (consistent with exit), disclosed as a deliberate dispatch-
  ordered divergence from repair-13's 1.67 verdict rule.
- lsl=8.5 display quirk fixed: cpk_study.rs emit for CTQ-06 now uses the
  ratified gate constant LSL 1.0 GiB/s (repair-13-cpk-study.md:16-18 named
  the stale literal; :341 pre-edit). JSON now shows lsl=1.0 (or 10.0 under
  injection).
- Anti-vacuity: env CPK_INJECT_FAIL=1 forces CTQ-06 LSL to an unreachable
  10 GiB/s -> Cpk negative -> exit 1. Gate flips witnessed below.
- Runtime: ~35 s total for the full 30-window run (CTQ-06 dominates at
  ~150 ms/sample x 30). No Process.sleep anywhere.

## Runs (commands + exits)

| # | Command (cwd /Users/sac/cgap) | Exit | Result |
|---|---|---|---|
| 1 | CARGO_TARGET_DIR=/tmp/g09-target cargo build --release -p bench --bin cpk_study | 0 | release build clean (one pre-existing cgap-branchless warning, unrelated) |
| 2 | /tmp/g09-target/release/cpk_study (healthy) | 0 | verdict PASS, 3/3 gates: CTQ-05 Cpu 144.22, CTQ-06 Cpl 28.97 (mean 1.842 GiB/s, sigma 0.0097, lsl 1.0), CTQ-04 Cpu 4.176 (CV 1.20%); full /tmp/g09-normal.json |
| 3 | CPK_INJECT_FAIL=1 /tmp/g09-target/release/cpk_study | 1 | FLIPPED: CTQ-06 FAIL, Cpl -84.69 at injected lsl 10.0, failed_gates [CTQ-06]; full /tmp/g09-inject.json |
| 4 | git diff HEAD --stat (read-only) | 0 | cpk_study.rs +91/-22, sole file touched |

Court semantics: run 2 proves the gate admits a healthy process; run 3
proves it refuses under synthetic lower-bound injection. A one-sided
rig (either run alone) would be vacuous.

## Fences and standing

- Must-not-regress fences: this diff touches only the bench bin; nearest
  shared fences are `cargo test -p bench` compile surface and the bench
  membrane rules (.claude/rules/bench.md: side-bin status unchanged, no
  divan benches touched). Kernel crates untouched; no gate depends on
  cpk_study exit 0 previously (repair-13 recorded exit 1 as honest).
- Standing: ALIVE for every claim above — each cites a command + exit run
  this session. The dispatch-cited 1.572/4.947 re-run: UNVERIFIED (no
  receipt on disk). CTQ-06 charter 6-sigma target 1.67: the gate now
  reports PASS at the 1.33 floor with Cpl 28.97 under quiet-window n=30;
  the 1.67 charter target is context, not the exit gate.
- Divergences: (1) lanes/ dir did not exist — created by this receipt;
  (2) verify_cpk.py absent — scope stayed on the bin; (3) exit floor 1.33
  supersedes the 1.67 verdict rule per dispatch, disclosed; (4) n=30 on
  ALL gates (dispatch fallback n=5 not needed; full run ~35 s).
- Build lease /tmp/g09-target deleted at lane end (shutil.rmtree).
