# 06 Control Plan

DFLSS Control Plan for the v26.11.2 starDF campaign. Companion to the
charter `docs/jira/v26.11.2/14-dflss-charter.md` (SMART goals G1-G6,
CTQ flowdown, tollgates) and CTQ tree `15-ctq-tree.md`. The control
plan binds each campaign gate to a frequency, a fresh this-session
exit witness, and an OCAP (out-of-control action plan) — a gate row
without a run exit is UNVERIFIED, never PASS.

All exits below were run this session (2026-11-02) against
`/Users/sac/cgap` @ main, unless marked cited. Exit code law:
0=pass / 1=gate fail / 2=subject-missing or no-data / 3=loud skip.

## GATE matrix

| Gate | Subject / command | Frequency | Exit (witness) | OCAP |
|------|-------------------|-----------|----------------|------|
| GATE-01 | `python3 scripts/verify_disassembly_invariants.py` (arms C/D/E: census + linked witness + binary mutant) | per kernel change + nightly | 0 PASS — "AE4 disassembly-invariants court: CLEAN"; anti-vacuity M-G03-E flip witnessed in-run (synthetic bnez injected into scratch copy, flagged; original archive sha256 unchanged) | on fail: halt merge of the offending kernel fn; re-census via REP1 arm; if flip arm cannot fire, gate is VACUOUS (exit nonzero required) |
| GATE-02 | `python3 scripts/zero_string_gate.py` (+ `--self-check`) | per build + nightly | 0 (scanned=92 crates=22 hits=0 emitter_hits=2 allowed) ; self-check 0 — poison artifacts=1 hits=2 fired=True, clean hits=0, RESULT PASS | on fail: refuse landing; strings route through the vocab source `graph-derivation:ontologies/*/src/lib.rs` (9 tokens) |
| GATE-03 | `python3 scripts/verify_cscb_sram.py` | per kernel change | 0 PASS — 36048 B <= 131072 B, headroom 95024 B; report-only baseline delta -3136 B vs 39184 B | on fail: freeze ingress surface; re-run freestanding witness; frame bound stays UNVERIFIED as worst-case chain bound (static analysis only) |
| GATE-04 | `cd crates/csct && cargo run --release --bin cscb_c4_alloc` (+ `CSCB_MUTATE=alloc` flip) | per csct change + nightly | base exit 0 "C4 PASS: steady-state 0 allocs / 0 bytes over 1000000 frames"; flip exit 1 "ALLOC BUDGET BREACH ... -> exit 1" — both arms run fresh | on fail: typed breach exit 1; court self-reports VACUOUS if mutate mode saw no breach |
| GATE-05 | anti-vacuity battery: `bash .claude/hooks/flip-tests.sh` ; `python3 scripts/receipt_lint.py --strict` ; `python3 scripts/boundary_gate.py --self-check-deps` | per lane receipt + nightly | flip-tests exit 0, 9 PASS / 0 FAIL; boundary --self-check-deps exit 0 (clean=0, poison=1, both arms flipped); receipt_lint --strict RESOLVED (typed MISSING-KEY guard, `lanes/lane-rc09-receipt-lint-keycrash.md`: strict exit 1 typed, --self-check exit 0, 5 courts + guard-strip anti-vacuity refused) | flip-tests: a court that cannot flip is FAIL; receipt_lint repair landed per RC-09, floor not touched |
| GATE-06 | `make gate0` (ggen sync x2 byte-identical replay + freestanding C99 header compile) | per regen + release | 0 — header compile PASS, replay held | on drift: `rm ggen.lock && ggen sync run` twice + cmp (FM-PACK-008 fix) |
| GATE-07 | `python3 scripts/test_census.py --crate cgap-parquet` | per test-wave + nightly | 0 — CENSUS: 14 targets, PASS 14, FAIL 0, COMPILE-ABORT 0, TIMEOUT 0 | compile-abort is not a failure count; census distinguishes exit-1-abort from real FAIL per W672 law |
| GATE-08 | flip-tests hook wiring: `.claude/settings.json` hook entry -> `bash .claude/hooks/flip-tests.sh` | per session (hook fires on lane events) | wired at `.claude/settings.json:65`; script run fresh this session exit 0, 9/9 PASS | if wiring is removed or script returns nonzero on a clean tree, the hook arm is VACUOUS -> restore and re-run flip-tests |
| GATE-09 | `python3 tests/gates/enforce_coverage_floor.py` | per scripts-wave + nightly | **2 (RED, no-data)** — "FLOOR: no coverage data at /tmp/c06-cov/.coverage.*"; no fresh coverage run this session | OCAP per C06: raise coverage, never lower floor; regenerate `.coverage.*` under /tmp/c06-cov, then re-run; exit 2 (no data) vs C06's exit 1 (measured red 24.4/40.8/52.6/64.9% vs floor 90) — both are honest RED, neither rounds up |

## Out-of-control log (this session)

| Trip | Evidence | Disposition |
|------|----------|-------------|
| octopus mutation court intermittent FAIL (`octopus_merge.rs:356` region) | re-run this session: `cargo test -p cgap-git --test octopus_merge_court` -> exit 0, 5 passed / 0 failed — intermittent trip did NOT reproduce | stays on watchlist as intermittent; a single green run does not clear an intermittent, it only records a non-reproducing sample |
| C3 SLA coordinate FAIL (b02) | `lanes/lane-b02-cscb-c3.md:13,19` — min 42.4 s vs 25.0 ms target; correctness + anti-vacuity courts ALIVE | honest FAIL standing recorded; MV-5 re-scope pending — no `lane-mv-5` receipt exists on disk yet (checked this session), so re-scope is UNLANDED |
| C6 throughput typed FAIL (b05) | `lanes/lane-b05-cscb-c6.md:34,49` — 248M facts/s vs 800M target, exit 1; mutant flip witnessed | honest FAIL standing recorded; re-scope same MV-5 dependency |
| receipt_lint --strict crash | RESOLVED (`lanes/lane-rc09-receipt-lint-keycrash.md`): typed `RECEIPT-LINT-MISSING-KEY` guard in `normalize_finding_keys()`; strict now exits 1 with a typed row, no traceback; `--self-check` exit 0 with 5 courts incl. guard-strip anti-vacuity arm (guard-stripped /tmp copy -> court REFUSES) | IN-CONTROL (RC-09); floor not touched |
| GATE-09 coverage floor RED | exit 2 no-data this session; C06 measured exit 1 at 24.4/40.8/52.6/64.9% vs floor 90 (`lanes/lane-c06-scripts-gate-coverage.md:46-53`) | RED is the designed state until coverage data is regenerated; floor stays at 90, never lowered |

## Escalation law

- A gate exit 1 on the shared tree freezes landing for the owning
  surface (compile-freeze SLA, ~10 min to fix or revert).
- A gate that cannot flip is FAIL; a green run with no flip arm is
  VACUOUS. Every row above names its flip witness.
- Perf numbers in the charter remain TARGETs until measured with the
  stated method; measured FAILs (C3, C6) are recorded as FAILs, never
  re-scoped by prose — re-scope lands only via a receipt (MV-5).
- Exit-2 (no-data) rows are RED, not PASS: absence of coverage data is
  itself out-of-control for the coverage floor.

---

## Section 5 (DF-05, 2026-11-02) — dflss dossier restructure

DF-05 lane addendum over the existing control plan. Every row below was
re-grounded against the receipts on disk this session; no run was
invented. Standing vocabulary per repo law; UNRUN/UNVERIFIED never
round up.

### 5.1 Anti-vacuity court narratives (landed, receipts cited)

**Path-alias race court (RED-13; fix owner pending).** RED-13
reproduced the FA9 spill-gate path-alias O_TRUNC truncation twice: FNV
of raw-path keying (`crates/cgap-ffi/src/sovereign.rs:863-892`) admits
a second thread at a `dir/./race.par2` alias; 65,508 drained bytes -> 0;
exploit binary exit 7 (red-13 §2, /tmp/red13-exploit), while
`cargo test -p cgap-ffi --test fa9_commit_gating` stays exit 0
(3 passed) — the court races only byte-identical paths
(`tests/fa9_commit_gating.rs:75,98,146`), a witnessed vacuity hole.
COURT RAN (2x, exit 7). UPDATE (this lane, 2026-11-02): the denial fix
LANDED — `lanes/lane-fix01-fa9-alias-rekey.md` (canonical_key
canonical-parent+NUL+file-name keying; new `fa9_alias_race_denied`
leg; shared tree exit 0, 4 passed; mutant raw-keying flip exit 101).
FFM-01 control row added at §5.2.

**SRAM gate flip (RED-12 + FIX-02, both landed).** RED-12 measured the
ungated bn3 surface: static NaEngine+NaMatches = 6,520,984 B (host
`size_of`, exit 0) and the riscv32imc linked-ELF charge .rodata =
6,520,976 B (+.text 56 B = 6,521,032 B) — 49.75x the 131,072 B
ceiling, silent under the 2-crate gate subject list
(red-12 W3/W5; DEPENDENCY_CRATES was cgap-blake3+cgap-branchless only).
FIX-02 LANDED (`lanes/lane-fix02-bn3-sram-gate.md`): cgap-bn3 is now a
gated subject with a parsed-const charge model cross-checked against
red-12 pins. Flip witnesses (all run in fix-02's session, exits cited
there): `--subject bn3` -> exit 1 BN3_ENCLAVE_REFUSED (6,520,984 B);
`--mutate bn3-pin` -> exit 0 with NA_MAX_MATCHES 2048->1024 mutant
tripping the pin cross-check (exit 1 DIVERGED); `--mutate sram-bloat`
-> exit 0 (136,068 B > bound); sandbox shim (default="bn3" injected in
/tmp copy) pass->exit 1; corrupted-restore probe detected by sha256
mismatch. Baseline B04 unchanged: 36,048 B <= 131,072 B, exit 0.

**Flood liveness (RED-14, landed; FIX-04 is a different subject).**
RED-14's Chicago flood probe (/tmp/red14 consumer crate, exit 0 twice):
1,000,000 FrameTooLarge + 1,000,000 Overflow class-101 refusals with
ring state untouched, then post-flood liveness — after 2,000,001 trips
the ring admitted and drained a 256 B frame byte-exact. A latched
breaker would fail step 3; it did not. Kernel brake ALIVE.
NOTE: fix-04-cscb-p99 receipt is the csct p99-labeling repair
(`cscb_c7_receipt_sealing.rs`), NOT an andon trip-gate fix — the
lane-id collision stands. The trip-gate signal itself is now
REPAIRED (2026-11-02, `lanes/lane-r-adv12fix-quota-repair.md`):
level-triggered `tripped` (survives admitted frames; clears via
`acknowledge()` or 1024 consecutive clean admissions),
`window_refusals` saturating_add at `u32::MAX` — the two ADV-12
spoilers (`redteam/corrosion/adv-12-control-plan-evasion.md`)
closed in code; F-03 re-verified (courts 5+1 exit 0, probe exit
0). The throttle-before-exit-mapping recommendation (red-14 §4)
remains a host obligation — the kernel quota stays
observational-only.

**Mutation non-vacuity law (standing).** A court that cannot flip is
FAIL; a green run with no flip arm is VACUOUS (exit nonzero). Witnessed
this wave by: GATE-01 M-G03-E in-run flip, GATE-04 CSCB_MUTATE=alloc
exit-1 flip, GATE-05 boundary --self-check-deps both arms, cpk_study
CPK_INJECT_FAIL flip (05-cpk-analysis.md Sec 2), fix-02's three SRAM
mutation courts, red-14's liveness step 3. Sandbox law: mutate /tmp
copies, sha256-exact restore, mtime bump (a01-finish-receipt.md:
170-184).

### 5.1b UNRUN surface (loud)

**"18 publishable units clean dry-run matrix": UNRUN.** No receipt in
docs/jira/v26.11.2/{lanes,waves}/ runs a clean dry-run publish matrix
over 18 crates (grep this session: dry-run mentions in lane receipts
cover receipt-lint, station300, manifests, lease-janitor — none a
publish dry-run matrix). `cargo package` evidence exists only for
cgap-core (lane-fix01-core-license.md, 1 crate). Standing: UNRUN —
never rounds up to a PASS row.

### 5.2 SPC control table (real surfaces, actual exit contracts)

Each row's exit contract was read from the script itself this session.

| Control surface | Exit contract (read from source) | Anti-vacuity arm | Status |
|---|---|---|---|
| `scripts/verify_cscb_sram.py` | Exit-ABI block at verify_cscb_sram.py:18-23 + EXIT_* consts at :74-77: 0 PASS / 1 OVERFLOW-or-unparsable-or-build-blocked-or-VACUOUS-mutant / 2 missing subject / 3 llvm-size absent (`--mutate sram-bloat`, `--mutate bn3-pin`, `--subject baseline/bn3`) | sram-bloat mutant MUST overflow else VACUOUS exit 1; bn3-pin mutant trips red-12 pin cross-check | LANDED (FIX-02 receipt) |
| `scripts/zero_string_gate.py` | 0 clean / 1 GATE-REFUSED (any fail incl.
internal-error unwind, zero_string_gate.py:23-27); `--self-check` builds poison
+ clean artifacts in /tmp, requires the poison scan to FIRE (hits=2, fired=True)
and clean to pass — a non-firing self-check is VACUOUS (nonzero) | --self-check
poison/clean pair | LANDED (GATE-02 witness, both arms exit 0) |
| `scripts/ffi_surface_gate.py` | EXISTS on disk; docstring: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip; `--self-check` runs labeled want-code arms (`[SELF-CHECK] ... want N`) | self-check arms require each mutation to hit its wanted exit | LANDED (file verified; closes RED-03 header gap) |
| `scripts/boundary_gate.py` | EXIT_PASS 0 / EXIT_REFUSE 1 (typed refusal, VACUOUS GATE, red baseline) / EXIT_SUBJECT_MISSING 2 / EXIT_SKIP_LOUD 3 (consts at boundary_gate.py:151-155); `--self-check-deps` mode | self-check-deps: clean=0 + poison=1, both arms must flip | LANDED (GATE-05 witness, exit 0); FM-07 poison-arm closure `lanes/lane-fix05-boundary-poison.md` (arm P poison dep -> exit 1 EXTERNAL, self-check initially REFUSED a mis-placed dev-deps poison — court flipped the fix) |
| `scripts/receipt_lint.py` (receipt-sealer-gate SL rows) | docstring: 0 clean / 1 findings under `--strict` or VACUOUS self-check / 2 fail-closed (zero docs, unreadable, internal exception) / 3 loud skip | claim-sentence proximity law; UNRECEIPTED-CLAIM line emitted per finding | LANDED + IN-CONTROL (lane-dx-04, lane-gap-05, crash closed by `lane-rc09-receipt-lint-keycrash.md`: typed MISSING-KEY guard, strict exit 1 typed no-traceback, --self-check 0, guard-strip court refused) |
| `scripts/receipt_doctor.sh` + `scripts/mint_receipt.sh` (sealer-adjacent) | receipt_doctor: CURRENT/STALE freshness classification vs live tree mtime; mint_receipt: fail-closed — exits nonzero when the minted command exits nonzero; receipt still written and marked FAILED | minted receipt marked FAILED on nonzero, never silent | LANDED (scripts verified on disk; no new run this session — prior receipts stand) |
| cargo test batteries (`fa9_commit_gating`, `receipt_chain`, cgap-git andon courts, csct suite) | cargo exit 0/1; counts asserted in receipts | each court names its mutation (fa9 = alias arm GAP, red-13 §2; csct CSCB_MUTATE) | LANDED per receipts: fa9 3 passed exit 0 (with the red-13 vacuity hole recorded), receipt_chain 5 passed exit 0, csct 41+1+7+9+7 passed exit 0 (fix-04) |
| **FFM-01** alias-denial control (fa9 spill-gate canonical re-key) | `cargo test -p cgap-ffi --test fa9_commit_gating` exit 0, 4 passed incl. new `fa9_alias_race_denied` leg (racer via `dir/./x`, 20,000 attempts: racer_ok==0, racer_deny>=1, spill never shrinks, drained prefix intact) — `lanes/lane-fix01-fa9-alias-rekey.md` | mutant (canonical_key reverted to raw FNV path keying) -> exit 101, racer 20000/20000 ADMITTED (red-13 breach shape) — court non-vacuous, witnessed | LANDED (FIX-01; closes the §5.1 path-alias PENDING row's denial gap) |
| **FM-07** poison-refusal control (pack_demux post-trailer) | `crates/cgap-git/tests/pack_demux_poison_court.rs`: P1 clean baseline zero poison; P2/P3 poison bytes -> `trailing_poison()` typed refusal; mutation m1 (revert Done arm) -> exit 101 VACUOUS-flip witnessed — `lanes/lane-fix05-pack-demux-poison.md` | m1 flip (poison silently absorbed without the guard) is the anti-vacuity arm | LANDED (FIX-05) |
| Exit-101 SPC loop diagram (class-101 breaker/latch) | **DESIGN-INTENT ONLY — no on-disk referent.** grep over crates/: zero `process::exit(101)` hits; **no breaker/halt/latch on the refusal path** (red-14 §1) — scoped claim per EVAL-04: non-refusal-path latches DO exist (`crates/cgap-bn3/src/builtins.rs:199` `log:counters` latch; `builtins.rs:431-443` `BuiltinContext.eval_count` latch; `crates/bench/benches/chains-seal.rs:293-295` one-shot per-row meter latch) and are data-path/bookkeeping state, not refusal gating. class-101 is a typed per-call refusal (`ingress_ring.rs:101-107`, `egress_ring.rs:123-128`, `patent_court.rs:35`), ring state untouched by construction | red-14 flood: 2,000,001 trips leave ring admitting (exit 0) — proves the refusal path non-latching; ADV-12 re-probe (F-03, exit 0): 4,097 refusals -> 1 admit -> `tripped()` STAYS true (level-triggered), 1024-clean-admission hysteresis + `acknowledge()` + `saturating_add` window (`lanes/lane-r-adv12fix-quota-repair.md`, mutants M1-M4 flip) | DIAGRAM = DESIGN-INTENT (cites red-14); trip-gate SIGNAL repaired (level-triggered + saturation, ADV-12 spoilers closed — repair receipt cited); throttle/authz before any future exit mapping remains a standing HOST obligation — quota stays observational-only, no exit mapping exists on disk |

| **CONTENTION-SUSPECT** host-noise marker control (chains-ingress /
  mctq benches) | Run REJECTED for quiet-host-scoped claims when bench
  stderr carries the `CONTENTION-SUSPECT:` marker; rows citing a marked
  run are VOID (`../06-bench-protocol.md` §8). Method: grep the
  receipted bench stderr for `CONTENTION-SUSPECT`. Exit contract is
  typed-procedural today (consumer discipline, NOT a machine gate):
  0=clean / marked run quoted = claim VOID / missing stderr disclosure
  = receipt incomplete. Machine gate (fail-closed stderr grep) =
  TARGET, unlanded. Guard: `crates/bench/benches/chains-ingress.rs:716-802`
  (`CONTENTION_MARK` :745, emission :802/:854); mctq copy
  `mctq_consolidation.rs:31,65,98`. | Anti-vacuity = RC-11's flip
  witness (guard trips on injected noise, flip margin 2.1-30x,
  `../lanes/lane-rc11-contention-guard.md:30-34`); a consumer quoting a
  marked run without the check is the failure the row refuses | LANDED
  as procedural control (guard ALIVE per RC-11; machine gate TARGET) |

### 5.3 Standing

| Claim | Standing | Ground |
|---|---|---|
| Path-alias court ran (exit 7 x2) | ALIVE (as-of red-13 receipt, re-read) | red-13 §2/§5 |
| Alias fix denial receipt | LANDED, ALIVE — canonical re-key + `fa9_alias_race_denied` leg (shared tree exit 0, 4/4; mutant flip exit 101) | `lanes/lane-fix01-fa9-alias-rekey.md` |
| SRAM gate bn3 closure | LANDED, ALIVE | lane-fix02-bn3-sram-gate.md |
| Flood liveness 2,000,001-trip | ALIVE | red-14 §3 |
| Exit-101 latch/breaker | UNSUPPORTED (no referent) | red-14 §1 |
| 18-crate dry-run matrix | UNRUN | lanes/waves grep, this session |
| Receipt-lint --strict crash | IN-CONTROL (RC-09 typed MISSING-KEY guard) | `lanes/lane-rc09-receipt-lint-keycrash.md` |
| FFM-01 alias-denial control | LANDED, ALIVE | `lanes/lane-fix01-fa9-alias-rekey.md` |
| FM-07 poison-refusal (pack_demux + boundary deps) | LANDED, ALIVE | `lanes/lane-fix05-pack-demux-poison.md`,
`lanes/lane-fix05-boundary-poison.md` |
| CONTENTION-SUSPECT consumption rule | LANDED as procedural control; machine gate TARGET | §5.2 row above; `../06-bench-protocol.md` §8 |
| CONTENTION-SUSPECT guard flip witness | ALIVE (RC-11) | `../lanes/lane-rc11-contention-guard.md:30-34` |
