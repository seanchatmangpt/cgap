# DOCTRINE-4 / Pillar 7 — Commercial proofs (audit lane, 2026-11-02)

Lane: READ-ONLY explore, repo /Users/sac/cgap @ main. Every number below
carries its command + exit run THIS session. Vocabulary: ALIVE = ran and
passed this session; AS-OF-RECEIPT = a prior receipt's number, stale by
default; UNVERIFIED stays UNVERIFIED. Narrative sentences are hypotheses
until receipted.

## Method (fresh commands, this session)

- `cd crates/csct && CARGO_TARGET_DIR=/tmp/doc4-target cargo test drowning_benchmark` → exit 0; `drowning_benchmark::tests` **4 passed / 0 failed** / 37 filtered out, **including `court_flips_on_zeroed_spark ... ok`**.
- `cd crates/csct && CARGO_TARGET_DIR=/tmp/doc4-target cargo build --release` → exit 0 (pre-existing cgap-core dep warning only).
- Runner over lease binaries: `/tmp/doc4-target/release/cscb_runner --receipt-out /tmp/doc4-run-baseline.md` → **aggregate exit 0**; blake3 root `14e1448...9590707`; all 8 coordinates exit 0 (C1 328ms, C2 10ms, C3 5626ms, C4 568ms, C5 774–788ms, C6 12ms, C7 1868–1875ms, C8 898ms). Repeat baseline run: aggregate exit 0, root `4adb2d45...92a1d95d8`.
- `CSCB_MUTATE=zero-tax` ladder (x2) → **aggregate exit 1 both times**; blake3 roots `d437a609...645500f` / `322b17b9...0887bc6`.
- Direct coordinate runs (lease binaries): C1/C2 x3 direct — C1 171.8M / 177.4M / 182.7M frames/s (floor 1.2M, PASS all); C2 p99 21 / 19 / 21 ns (max 10) → **FAIL exit 1, 3/3 direct runs**; C6 min 241,911,097.7 facts/s → GATE PASS (decided 150M) with loud "3.31x below the aspirational 800M" line; C3 min 805.367ms → DECIDED GATE PASS vs 60,000ms, aspirational 25ms NOT met (32x over, loud).
- Note: the runner resolves sibling bins from `crates/csct/target/release` (its own standalone-crate target dir), not the lease; both ladder paths exercised current code regardless — binaries (mtime Nov 2 13:16–14:03) verified fresh vs sources via stat.

The `/tmp/doc4-target` lease could NOT be deleted (deletion denied by the permission system in this read-only lane); it remains on disk for the operator to remove.

## (a) 351x gross arbitrage — ALIVE

- drowning_benchmark 4/4 + zeroed-spark flip ALIVE (command above).
- REQUIRED_ROI_X ruling comment **confirmed present** at `crates/csct/src/lib.rs:30-35`: gross STATUS_QUO/SUT = 5,068,632/14,400 = 351.99 (net 351.15), pinned `REQUIRED_ROI_X: f64 = 350.0` gross, errata `waves/csct-lane-01-scaffold.md`.
- "351x" assertable ONLY as gross ratio 351.99 vs floor 350.0. "352x" (dod-csct-cscb.md:15) is not the pinned form.

## (b) C1 196.27M fps / C2 p99 6.00 ns — method DISCLOSED, numbers AS-OF-RECEIPT

- Method fully disclosed in code + receipt: `crates/csct/src/bin/cscb_c1_c2_ingress.rs:50` (`BATCH: usize = 64`), gate consts :48-49, runtime `[META] ... c2_method=batch64-amortized-instant` (:221); lane-b01:22-27 discloses the ~40ns clock-overhead rationale; FIX-04 audited the C2 p99 as HONEST (lane-fix04:34-36: p99 over `FRAMES/64` = thousands of batch-amortized samples, genuine tail quantile).
- But 196.27M fps / 6.00 ns are AS-OF-RECEIPT (lane-b01, /tmp/b01-target). This session, same binary vintage: C1 171.8–189.7M fps; **C2 p99 17–21 ns, FAILing the 10 ns gate 3/3 direct runs**. dod-csct-cscb.md:35 calls the ladder C2 miss "one timing flake" — three consecutive direct-run FAILs is variance beyond a one-off flake; the 10 ns gate does not reliably pass on this host today.
- FIX-04's sibling finding on C6 is **NOT fixed**: `cscb_c6_retraction.rs:100` computes `secs[6]` of N=7 (= max) and labels it "p99" at :103,110,113 — the max, not a tail quantile, undisclosed in its output lines. C3 does the same at `cscb_c3_closure.rs:256` but DISCLOSES it in verdict text ("p99 of 7 samples = max sample").

## (c) C3/C6 + MV-5 reconciliation — CURRENT state (re-scoped in code, UNRECEIPTED)

- Code truth: BOTH gates re-scoped to decided budgets. `cscb_c3_closure.rs:40-42`: `DECIDED_BUDGET_MS = 60_000.0` / `ASPIRATIONAL_MS = 25.0`; `cscb_c6_retraction.rs:40-42`: `DECIDED_GATE_FPS = 150_000_000.0` / `ASPIRATIONAL_FPS = 800_000_000.0`, both citing "lane MV-5 gate re-scope, 2026-11-02". Both gates now PASS (commands above).
- **No MV-5 receipt exists.** `grep -rl "MV-5" docs/` → zero files. The re-scope lives only in source comments. release-blockers.md R1/R10 rows and dod-csct-cscb.md:39,60 still describe the OLD 800M honest-FAIL state — docs stale vs code. The re-scope is an unreceipted gate change, exactly the class R1's own close condition demands a "written gate-change receipt" for. It needs an MV-5 append to the register (the R5/R8/R9 pattern).
- Measured today: C6 min 241.9M facts/s (decided PASS, 3.31x aspirational shortfall, loud); C3 min 805.367ms (decided PASS; 32x over aspirational 25ms, loud).

## (d) C6 248M vs 800M

- `crates/cgap-storage/src/dred.rs:66-73` (comment block in the :72-77 region): 839–944M facts/s is MASK-ONLY legs (EXP-05), not full-cascade; the per-lane form was KEPT on the perf fence per repair-16.
- Full-cascade numbers: b05 min 248.06M, GIT-11 323.84M (MEASURED, UNGATED), this session 241.9M — all in a 200–330M band. "Hundreds of millions of facts/s" is the assertable form; "800M" is not assertable bare, and even the EXP-05-qualified form is not the gate's unit. C6's runtime shortfall line is the correct self-qualifying pattern — copy it wherever 800M appears in the narrative.

## (e) CSCB ladder + R10 zero-tax flip court — RESOLVED as-of this session

- Baseline aggregate exit 0 (all 8 coordinates), zero-tax aggregate exit 1, witnessed 2x. Deterministic flips: **C3 and C6 only**. C1/C2 exits under zero-tax are timing variance, NOT mutation flips — `cscb_c1_c2_ingress.rs:96-98` accepts only `CSCB_MUTATE=branchy`; C4/C5/C7/C8 read no CSCB_MUTATE.
- B08's close condition is met on its own terms: passing baseline + children (C3 radix-clamp, C6 skip-sweep, each with the zero-tax alias) implement mutation semantics reading CSCB_MUTATE with a strictly different exit profile. lane-b08's "flip court FAIL (cannot flip)" standing is STALE; release-blockers R10 row likewise.
- Residual defect: `cscb_c3_closure.rs:167-168` claims "`zero-tax` is an alias accepted by every CSCB coordinate" — FALSE as written (see flip census above). One-line comment fix owed by B02; does not affect the witnessed flip.
- Provenance: lane-lspa-01 touched only lsp-max + docs, NOT the runner. `git status --porcelain crates/csct/src/bin/` → whole dir untracked; no git history; the zero-tax aliases and re-scoped gate consts are UNRECEIPTED code changes (no lane receipt names them post-B08).

## Ship-gate table

### SALES-SAFE (assertable with receipts, as-is)

| # | Claim | Receipt / command |
|---|---|---|
| S1 | 351x gross arbitrage (351.99 vs floor 350.0); drowning_benchmark 4/4 incl. zeroed-spark flip | cargo test drowning_benchmark exit 0, 4 passed this session; lib.rs:30-35 |
| S2 | CSCB ladder baseline exit 0, all 8 coordinates exit 0, blake3-rooted receipt | /tmp/doc4-run-baseline{,2}.md, roots 14e1448... / 4adb2d45... |
| S3 | R10 zero-tax flip court FLIPS: baseline 0 vs zero-tax 1; C3+C6 deterministic flips; 2x witnessed | /tmp/doc4-run-mutate{,2}.md |
| S4 | C6 full-cascade "hundreds of millions of facts/s" (min 242–248M; decided 150M gate PASS; integrity 0 phantoms) | C6 run this session; lane-b05 |
| S5 | C6 loudly reports 3.31x shortfall vs 800M aspirational (mask-only, EXP-05) | C6 output line this session |
| S6 | C3 closure 1M facts, min 805ms (decided 60s PASS; 32x over 25ms aspirational, loud) | C3 run this session |
| S7 | C1 172–190M frames/s this session vs 1.2M floor (143x–158x headroom) | direct runs, C1 gate exit 0 all |
| S8 | C2 method = batch64-amortized Instant, disclosed in code + receipt | cscb_c1_c2_ingress.rs:50,221; lane-b01:22-27 |
| S9 | C6 "p99" is max-of-7, undisclosed — FIX-04 sibling finding OPEN, not applied | lane-fix04:34-36; cscb_c6_retraction.rs:100-113 |
| S9b | C3 labels max-of-7 as p99 but DISCLOSES it in verdict text | cscb_c3_closure.rs:256 |
| S10 | MV-5 gate re-scope is in code but UNRECEIPTED (no MV-5 file anywhere in docs/) | grep -rl MV-5 docs/ → 0 files |
| S10b | release-blockers.md R1/R10 + dod-csct-cscb.md stale vs code truth | file reads this session |

### MUST-QUALIFY (needs method disclosure or aspirational marking)

| # | Claim | Required qualifier |
|---|---|---|
| M1 | "C2 p99 sub-10ns" | MUST carry method ("p99 of batch64-amortized per-frame Instant samples") + AS-OF-RECEIPT (6.00 ns, lane-b01). NOT assertable as passing today: 17–21 ns, FAIL 3/3 direct runs |
| M2 | "C1 196.27M fps" | AS-OF-RECEIPT; today 172–190M. Assertable today: "172–190M fps, 143x–158x over the 1.2M floor" |
| M3 | "C6 800M facts/s" | NEVER bare. Only "mask-only legs (dred.rs:66-77, EXP-05), not full-cascade; full-cascade min 242–248M" |
| M4 | "352x" | "351.99 gross ratio (net 351.15), floor pinned 350.0 gross" per lib.rs:30-35 |
| M5 | "C3/C6 honest FAILs" | STALE. Current truth: both PASS decided budgets (C3 60s, C6 150M); aspirational figures not met, reported loudly. Cite MV-5 re-scope — and flag it is UNRECEIPTED |
| M6 | "R10 flip court cannot flip" (lane-b08 / release-blockers R10) | STALE — witnessed flipping 2x this session; close condition met on receipt's own terms |
| M6b | "zero-tax accepted by every coordinate" (cscb_c3_closure.rs:167-168) | FALSE as written — only C3/C6 accept it; C1/C2 takes branchy only |
| M7 | Any "drowning 4/4 + flip" claim without the command | needs exact command + "4 passed / 0 failed" line (S1) |
| M8 | "800M aspirational met" | never met; 3.31x shortfall, loud line only |
| M9 | Ladder subject binaries | runner resolves crates/csct/target/release, not the lease — cite binary mtimes 13:16–14:03, fresh vs sources |
| M9b | /tmp/doc4-target lease | left on disk (deletion denied in read-only lane); operator should `rm -rf /tmp/doc4-target` |

## One-screen answer

ALIVE this session: 351x gross (4/4 courts + zeroed-spark flip), CSCB ladder baseline exit 0 / zero-tax exit 1 — **R10 flip court FLIPS; R10 is RESOLVED as-of-today and the register row is stale**. C6 "hundreds of millions" (242–248M) passes the decided 150M gate; C3 passes the 60s decided budget. The 196.27M fps / 6.00 ns figures are AS-OF-RECEIPT, and C2's 10 ns gate FAILED 3/3 direct runs today (17–21 ns) — the "sub-10ns" narrative claim must carry the batch64-amortized method and the as-of-receipt mark or it overclaims. The C6 800M figure is mask-only (EXP-05); the assertable form is "hundreds of millions". The MV-5 gate re-scope exists only as source comments with no receipt file — an unreceipted gate change; release-blockers R1/R10 and dod-csct-cscb.md are stale vs code and need an MV-5 append. FIX-04's C6 misnamed-p99 sibling finding is NOT fixed (max-of-7 labeled p99, undisclosed in C6's output; C3 discloses). The /tmp/doc4-target lease remains on disk (deletion denied).
