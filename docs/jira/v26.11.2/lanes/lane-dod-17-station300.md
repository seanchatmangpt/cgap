# lane-dod-17-station300

Witnessed: 2026-11-02 session. Lane DOD-17. Goal: Station 300 GREEN
attempt over the post-fix tree (TPS-5 landed the fail-closed battery
with `--dry-run` flip).

## W1 Files written

- `docs/jira/v26.11.2/lanes/lane-dod-17-station300.md` (this receipt,
  NEW). No other files written; no fixes applied — every halt below
  resolves to an IN-FLIGHT lane's file (LOUD-PENDING) or an
  environment-class row, so the minimal-fix branch of the dispatch was
  never lawful to take.

## W2 Commands + exits (this session, exact)

| stage | command | exit | log |
|---|---|---|---|
| wait-free check | `cargo test -p cgap-core --test manifest_court` | 0 (3 passed, dod10_no_spdx GREEN) | scoped run |
| full battery | `bash scripts/run_station300.sh` | halt @2b | /tmp/tps5-dod17-battery-stdout.log |
| dry-run flip | `run_station300.sh --dry-run` | 1 (DRYRUN-FLIP-CONFIRMED) | /tmp/tps5-dod17-dryrun.log |
| 1a | `go vet ./...` | 0 | /tmp/tps5-stage-1a.log |
| 1b | `go test -count=1 ./...` | 0 | /tmp/tps5-stage-1b.log |
| 2a | `cargo build -p cgap-ffi --lib` | 0 | /tmp/tps5-stage-2a.log |
| 2b | `cargo test -p cgap-core -p cgap-git -p cgap-ffi` | 101 (FAIL, LOUD-PENDING 1) | /tmp/tps5-stage-2b.log |
| 2c | `cargo test --manifest-path crates/csct/Cargo.toml` | 0 first → 0 on rerun (69 passed) | /tmp/tps5-stage-2c.log |
| 3 | `verify_disassembly_invariants.py` | 3, 3 on retry (loud skip, env-class) | /tmp/tps5-stage-3.log |
| 4 | `verify_cscb_sram.py` | 0 (36048 B <= 131072 B) | /tmp/tps5-stage-4.log |
| 5 | docker rust:1-bookworm arm64 `cargo test -p cgap-ffi` | 101 (43 passed, 7 FAILED) | /tmp/tps5-stage-5.log |
| 5 probe | docker `cargo metadata --no-deps` | 0 (container cargo parses the manifest fine) | /tmp/dod17-docker-meta.err (stderr notes) |

Full battery: stages 1a/1b/2a OK, STATION-300-HALT stage=2b rc=101
(failing command named, fail-closed as designed). Stages 2c/3/4/5 were
additionally run individually via the same `scripts/run_checked.py`
harness to gather per-stage evidence; each cites its own
/tmp/tps5-stage-N.log.

## W3 Courts run

- dod manifest court (cgap-core manifest_court.rs) — ALIVE, exit 0,
  3/3 including the previously-HALTed
  `dod10_no_spdx_license_identifiers_anywhere` (DOD-01's fix is landed;
  `license-file` is gone from `crates/cgap-core/Cargo.toml`, only the
  frozen URN on line 12 remains). TPS-5 HALT #1 is CLOSED.
- Anti-vacuity: `--dry-run` re-run THIS session — injected
  `sh -c 'exit 1'` halts rc=1 with
  STATION-300-DRYRUN-FLIP-CONFIRMED. The flip is not vacuous.
- interop-chicago go courts (1a/1b), CSCB SRAM envelope gate (4),
  csct 69-test battery (2c) — ALIVE, exits 0.
- AE4 disassembly-invariants court (3) — loud-skip (exit 3): mutant
  build fails with `linked-witness <unmapped>` for core/alloc symbols;
  baseline was exit 0 at TPS-5 hours earlier. Deterministic on retry.
  Cause UNVERIFIED (suspect: nightly toolchain auto-update witnessed
  2026-11-02, see stage 3 log). Anti-vacuity M-G03-E within the court
  still PASSes (court can flip at byte level), so the skip is typed and
  loud, not a silent pass.

## W4 Standing: PARTIAL — LOUD-PENDING items, no PASS emitted

No STATION-300-PASS. Halts, in priority order:

1. **LOUD-PENDING stage 2b (in-flight lane)**:
   `crates/cgap-git/src/ref_store.rs` (mtime 16:10, AFTER the newest
   lane receipt 15:43; git-status modified) fails to compile its lib
   test — 6 errors: internal `Counter` methods (`load`/`store`/
   `fetch_update` reimplemented at ref_store.rs:148) shadow the
   `AtomicU64` API; callers at ref_store.rs:201/206 etc. still pass
   `core::sync::atomic::Ordering` args and one call passes 3 args to a
   1-arg method (`expected FnMut(u64)` at :84 class). The editing lane
   (no receipt landed naming the change yet; tc fleet lanes in-flight
   per dispatch-ledger.jsonl) owns the fix. NOT fixed by this lane per
   the in-flight rule. Log: /tmp/tps5-stage-2b.log.
2. **Stage 5 docker row (environment-class, typed — not SKIP'd
   silently)**: 43 passed, 7 FAILED inside rust:1-bookworm arm64. The
   nm-class failures persist from TPS-5 and are ENVIRONMENT-class:
   `nm -gU` is a macOS flag baked into cgap-ffi test SOURCE
   (oabi_lookup.rs:253 `nm_gu`, sovereign.rs:3647 `Command::new("nm")
   .args(["-gU"])`), with no env knob — the dispatch's script-side
   normalization is infeasible without editing another lane's crate
   source, so I record the row typed instead. Failures:
   oabi_lookup surface_court x2 + wave_gap1 + wave_f2 (`nm -gU failed
   ... nm: 'a.out': No such file`, or `libcgap_ffi.dylib ... (macOS
   court)` — tests hardcode macOS dylib expectations), PLUS wave9_a07/
   a14/a20 failed this run (TPS-5 saw only 5 failures; the +2 are
   consistent with compiling the tree mid-churn of the in-flight
   manifest edits; attribution UNVERIFIED). Linux-nm normalization
   belongs in cgap-ffi test code (`cfg(target_os)` on the nm flag),
   owned by whoever holds cgap-ffi — LOUD-PENDING, not fixed here.
3. **Stage 3 loud skip (exit 3)**: AE4 mutant build fails on
   `linked-witness <unmapped>` (see W3). Environment/toolchain-class
   suspect, cause UNVERIFIED — recorded typed, not a silent pass.

Stages 1a/1b/2a/2c/4: green this session (exits 0).

## W5 Divergences from the lane doc

- Dispatch anticipated the docker BSD-nm fix could be made "INSIDE the
  script": the nm flag is not script-invoked — it is baked into
  cgap-ffi test source (oabi_lookup.rs:253, sovereign.rs:3647), so
  script-side normalization is impossible without touching another
  crate's source. Took the dispatch's typed-record alternative.
- Transient in-flight defect witnessed then self-healed: at 16:10 the
  in-flight publish fleet's edit set
  `categories = "no-std, cryptography"` (string) in
  crates/cgap-blake3/Cargo.toml:9, failing manifest parse on host
  nightly cargo (stages 2c/3/4 blocked, `invalid type: string ...
  expected a vector`); minutes later the same file carries
  `categories = ['no-std', 'cryptography']` and 2c/4 pass. Recorded as
  evidence of concurrent-churn attribution limits on this battery.
- DOD-01's fix found LANDED at wait-free check (doc said "if landed";
  it is).

## W6 Fence law

Must-not-regress surfaces this diff could break: NONE — the only file
written is this receipt. No source, script, or manifest touched; the
battery itself is read-over-tree. Fences I exercised but did not edit:
run_station300.sh fail-closed halt + dry-run flip (both witnessed this
session, exit-carrying), manifest_court (green), AE4/SRAM gates (run,
per-stage exits above).

## W7 Evidence (kept, not cleaned)

/tmp/tps5-dod17-battery-stdout.log,
/tmp/tps5-dod17-dryrun.log,
/tmp/tps5-stage-{1a,1b,2a,2b,2c,3,4,5}.log (all rewritten THIS session
except 99; stage-99 dry-run log from 15:22 TPS-5, superseded by
/tmp/tps5-dod17-dryrun.log),
/tmp/tps5-{2c,3,4,5}-wrap.log, /tmp/dod17-docker-meta.err.
Lane build lease /tmp/dod17-target deleted at lane end.

