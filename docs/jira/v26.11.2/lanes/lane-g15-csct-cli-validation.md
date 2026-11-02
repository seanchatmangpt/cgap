# Lane G15 — csct_runner CLI validation (fresh re-witness)

- date: 2026-11-02
- branch: main (no git writes; coordinator owns transitions)
- subject: `crates/csct/src/bin/csct_runner.rs` (real path; work order's
  `csct.rs` was wrong — no such file exists)
- build root lease: `/tmp/g15-target` (deleted at lane end)

## Modes re-witnessed on the integrated tree

All runs via
`CARGO_TARGET_DIR=/tmp/g15-target cargo run [--release] --quiet
--manifest-path /Users/sac/cgap/crates/csct/Cargo.toml --bin csct_runner -- <args>`.
Build: `cargo build --release --bin csct_runner` → exit 0 (5.7 s,
cgap-blake3 + csct).

| mode | args | profile | exit | stdout |
|---|---|---| typed witness |
|---|---|---|---|---|
| 1 | `--suite=cloud-cartel-1360shard-vs-cgap` | release | **0** | empty (Run mode prints only on Receipt/Mutate) |
| 2 | `--suite=nope` | release | **2** | `[SKIP] csct_runner: unknown subject/flag '--suite=nope' (accepted: ...)` — loud, no fake pass |
| 3 | `--receipt` | debug | **3** | full markdown receipt; stage 6 THE_SOLUTION is **LoudSkip**, timing assertion `ingress_filter_per_frame_ns_lt_10=75.24/10 ns/frame[FAIL]` explicitly labeled LoudSkip — no fake pass |
| 4 | `--mutate=zero-tax` | release | **0** | controls ran first (no `[VACUOUS]` line), full receipt, `final_commercial_verdict = PASS`, roi 352x |

Mode 4 is the post-ROI-fix behavior: previously exit 1 (pre-fix ROI);
now controls pass, suite passes, exit 0.

## Test census (fresh, this session)

`cargo test` (debug) → **14/14 pass, 0 failed**:
- 7 unit `classify()` tests (bin)
- 7 court tests (`tests/csct_pipeline_court.rs`), incl.
  `andon_refusal_wire_value_is_101` and `court_flips_on_stage_mutation`
  (anti-vacuity mutation present and passing)

## JSON sweep

`grep -v '^//' bin | grep -c 'json\|serde'` → **0**. The only
serde/json text in the bin is the doc comment ("No Python, no JSON, no
serde — stdout is the markdown receipt"). Stdout in modes 3/4 is
markdown/structured plain as required.

## Exit-code mapping audit (code-read + court, no in-person refusal)

- Bin `exit_for()` (`crates/csct/src/bin/csct_runner.rs:68-79`) returns
  first non-Pass stage's `exit.as_u32()`, else verdict→0/1.
- `ExitCode` (`crates/csct/src/types.rs:9-17`): Pass=0, GateFail=1,
  SubjectMissing=2, LoudSkip=3, **AndonRefusal=101**
  (`types.rs:14`, asserted `types.rs:196`).
- Unknown-arg path exits via `ExitCode::SubjectMissing` (witnessed live,
  mode 2).
- AndonRefusal: in-person trigger not fired this session (would need a
  real court refusal); covered by (a) code-read of `andon.rs:17`
  `pub const ANDON_REFUSAL: i32 = 101` and (b) the passing court
  `tests/csct_pipeline_court.rs:123 andon_refusal_wire_value_is_101`.
  Cited, not round-tripped in-person: the 101 path is UNVERIFIED
  in-person, ALIVE as code+court.

## Transient observation (disclosure)

One `cargo test` invocation mid-lane failed with E0425 (compile abort,
0 tests run); immediate re-run compiled clean and passed 14/14. Read as
a concurrent lane's in-flight write, resolved by the time of re-run. No
fix applied by this lane; compile-abort is not a failure count.

## Fences (must-not-regress surfaces adjacent to this lane)

- `crates/csct` is a standalone crate with its own `[workspace]` —
  root workspace builds are untouched by this lane. Read-only session:
  zero source edits, receipt file only.

## Standing

- Modes 1-4: **ALIVE** (command + exit, this session, integrated tree).
- AndonRefusal 101 in-person: **UNVERIFIED** (code-read + passing court
  cited; never rounds up).
- Zero-JSON claim: **ALIVE** (grep run this session).

## Cleanup

- `/tmp/g15-target` deleted at lane end per lease law.
