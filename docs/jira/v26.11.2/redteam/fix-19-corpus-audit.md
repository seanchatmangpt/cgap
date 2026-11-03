# FIX-19 — redteam corpus audit (lane R-19)

Date: 2026-11-02. Lane: R-19 (=FIX-19), read-only over the corpus + /tmp
re-runs; this file is the sole write. Contract honored: no git write
commands, no edits outside this file.

Standing: **ALIVE** — every verdict below carries a command + exit run
this session (see §2 receipts and §3 grep table). Methods stated, no
Process.sleep, /tmp build-root lease `/tmp/r19-target` only.

## 1. Per-report verdict rows

Audit verdict key: STANDS = as-filed verdict survives fresh
verification; FIXED = a FIX lane closed it and the closure re-ran green
this session (§2).

| report | filed | audit | key |
|---|---|---|---|
| red-01 cleanroom bypass | BREACH (constructability) | STANDS | N/A-CODE (legal layer) |
| red-02 LLM extraction | PARTIAL / trap UNSUPPORTED | STANDS | grep §3 r8 |
| red-03 FFI shim decoupling | ALIVE | STANDS | include/cgap.h 175 lines |
| red-04 corporate smurfing | ALIVE / harbor misplaced | STANDS | $50M ceiling on disk |
| red-05 declaratory complaint | targets UNSUPPORTED | STANDS | §3 r2-3 |
| red-06 ASC450 defusal | PARTIAL / Tranche 2 UNSUPP. | STANDS | §3 r3-5 |
| red-07 retaliatory discovery | ALIVE / P12 UNSUPPORTED | STANDS | §3 r10 |
| red-08 joint defense | PARTIAL / premise UNSUPP. | STANDS | no instrument on disk |
| red-09 alloc sentry | ALIVE, no breach | STANDS | re-run §2.4 exit 0 |
| red-10 timing jitter | ct UNSUPP / latency ok | STANDS | re-run §2.5; cite-drift F2 |
| red-11 riscv reentrancy | BREACH-CONFIRMED | STANDS | open arm rc01/rc02, F3 |
| red-12 SRAM overflow | BREACH-CONFIRMED | FIXED | FIX-02; re-run §2.2 |
| red-13 receipt desync | BREACH-CONFIRMED | FIXED | FIX-01; re-run §2.1; F1 |
| red-14 andon DoS | PARTIAL breach | STANDS | FIX-04 seam §2.3 |
| red-15 PII reconstruction | PARTIAL / UNSUPPORTED | STANDS | §3 r6-7 |
| red-16 headcount paradox | thesis UNSUPPORTED | STANDS | §3 r9-10 |
| red-17 cloud savings | ALIVE / no cost rows | STANDS | §3 r14 |
| red-18 hardware tamper | Fogclave UNSUPPORTED | STANDS | §3 r1; cite-drift F2 |
| red-19 market inertia | PARTIAL | STANDS | §3 r9 |
| red-20 survivor synthesis | 13-row register | 12/13 STAND | row 75 WRONG -> F1 |

Row notes (beyond the table cells):
- red-04: safe-harbor threshold exists in code
  (REVENUE_GENERATION_COMPENDIUM.md:39,180; asc450.rs); fix-10
  reconciles the $50M figure across layers.
- red-11: ref_store arms taken by rc01/rc02 lanes; the reentrancy-unsafe
  arm stays open and UNVERIFIED-flagged (red-11:93).
- red-20: consolidated count "13 distinct targets" should read 12
  verified + 1 refuted row (F1).

## 2. BREACH reproducer re-runs (5)

### 2.1 red-13 path-alias spill-gate bypass — FIXED (FIX-01)

Shape: owner holds `dir/race.par2`, racer attacks via `dir/./race.par2`
alias. Post-fix `sovereign.rs` keys ownership on canonicalized paths
(sovereign.rs:848-893, three-tier canonicalize with parent+name
fallback — fresh read this session).

Command 1: `CARGO_TARGET_DIR=/tmp/r19-target cargo test -p cgap-ffi
--test fa9_commit_gating` → **exit 0, 4 passed / 0 failed** (pre-fix
court had 3 tests; the alias arm `fa9_alias_race_denied` at
fa9_commit_gating.rs:261 is new — the court's vacuity hole red-13 named
is closed with a real racing test, not a comment).

Command 2 (gate self-witness): SRAM gate log §2.2 exit 0.

Fix receipt: `lanes/lane-fix01-fa9-alias-rekey.md`.

### 2.2 red-12 SRAM charge — FIXED (FIX-02)

Command: `python3 scripts/run_checked.py --out /tmp/r19/sram.log --
python3 scripts/verify_cscb_sram.py` → **exit 0**. The gate now prints
the bn3 static state-table charge explicitly: `NaMatches 2,162,696 B +
NaEngine 4,358,288 B = 6,520,984 B`, exactly red-12's 49.75x figure —
the exploit arithmetic is now a first-class gate row. Under
`--subject bn3` the gate REFUSES with a typed failure (log line:
"admission REFUSED (typed gate failure, exit 1)").

Fix receipt: `lanes/lane-fix02-bn3-sram-gate.md`.

### 2.3 red-14 flood — STANDS + FIX-04 seam verified

Probe: `/tmp/r19/red14` (path-dep on cgap-git, build root
/tmp/r19-target). Command: `CARGO_TARGET_DIR=/tmp/r19-target cargo run
--release --manifest-path /tmp/r19/red14/Cargo.toml` → **exit 0**.
1,000,000 oversized (40,000 B) frames all refused typed; ring pending 0;
fill to 32,768 then 1,000,000 further pushes all refused with state
byte-intact; post-flood admission + byte-exact drain of 256 x 0xCD;
`IngressError::Empty.andon_exit_class() == 101`. Output:
`FLOOD-RE-RUN OK overflow_flood=1000000 full=32768 quota_tripped=false
quota_class=101`.

Honest note: `quota_tripped=false` after 2,000,000 refusals is by
design — the FIX-04 quota seam is caller-fed (`note_refusal` is invoked
by the host, not by `push()`), consistent with the "OBSERVATIONAL ONLY,
never gates admission" contract in ingress_ring.rs:95-104. A host that
never calls `note_refusal` gets no escalation signal; that wiring gap
is a design scope note, not a red-14 regression.

Fix receipt: `lanes/lane-fix04-andon-quota.md`.

### 2.4 red-09 alloc-sentry battery — STANDS

Command: `CARGO_TARGET_DIR=/tmp/r19-target cargo test -p cgap-git
--test posh_alloc_sentry_court` → **exit 0, 1 passed** (positive
control flips; sentry windows green; F1/F2 poison closure from FIX-05
in the same crate's court set: lane-fix05-pack-demux-poison.md).

### 2.5 red-10 ingress bench — baseline SURVIVES; hogs [SKIP]

Command: `CARGO_TARGET_DIR=/tmp/r19-target cargo bench -p bench --bench
chains-ingress -- ingress_chain` → **exit 0**, CONTENTION-GUARD
in-run quiet (worst leg pass ratio 1.67 <= 2). 1M leg: fastest 11.29 ms
= 10.77 ns/frame; receipted claim 10.20 ns/frame reproduces within
~5.6% (red-10 measured 4.5% on its run).

[SKIP] contention-hog matrix (bw/cpu load generators): red-10's
/tmp/red10_hog.c generators no longer exist and rebuilding the full 4x
hog matrix is outside this lane's budget — the re-run covers the
baseline leg only. The red-10 conclusion (raw-latency claim survives as
receipted; constant-time has no measured referent) is unaffected: the
baseline reproduces, and the constant-time claim was already marked
UNSUPPORTED (11-measured-claims.md:53, row 29).

### 2.6 N/A-CODE breaches (honest register)

red-01 (cleanroom/constructability), red-04/05/06/07/08 (license and
legal instruments), red-16 (thesis audit), red-18 (physical appliance),
red-19 (strategy referents): not code-reproducible; verified via the
§3 grep table only.

## 3. Fresh grep table (UNSUPPORTED verification, this session)

Method: `grep -rniE <term> crates/ scripts/` from repo root; rc=1 /
files=0 means genuinely absent from code surfaces. Doc-surface hits are
noted where they exist.

| # | term | rc | files | disposition |
|---|---|---|---|---|
| 1 | fogclave | 1 | 0 | red-18 stands |
| 2 | tripartite | 1 | 0 | red-05 stands |
| 3 | tranche | 1 | 0 | red-05/06 stand |
| 4 | arrears | 1 | 0 | red-06 stands |
| 5 | true-up | 1 | 0 | red-06 stands |
| 6 | redact | 1 | 0 | red-15 stands |
| 7 | \bpii\b | 1 | 0 | red-15 stands |
| 8 | deconstruction / anti-llm | 0 | incid. | unrelated "tripwire" code only — N1; red-02 stands |
| 8b | pincer | 1 | 0 | hits only in redteam/*.md — red-19/20 stand |
| 10 | zero.headcount | 1 | 0 | red-16 stands |
| 11 | constant.time | 0 | hits | code comments only, no measured claim — see N2; red-10 stands |
| 12 | shm_open | 0 | 7 | SPOILER, see F1: crates/cgap-beam/src/shm.rs |
| 13 | Sigma_PQC law | 0 | prose | zero code referent, negation comments only — see N3; stands |
| 14 | 90/99 economics | 0 | workload | only churn-protocol rows, zero cost rows — see N4; stands |
| 15 | $50M | 0 | compend. | ceiling EXISTS on disk — see N5; red-04 correct |
§3 row notes:
- N1: the "tripwire" hits are branchless underflow masks
  (cgap-branchless/src/arithmetic.rs:13-25), the bn3 drift pin
  (cgap-bn3/src/project.rs:67,455), and the pack_demux surplus guard
  (tests/pack_demux_poison_court.rs:78) — none is an anti-LLM mechanism.
- N2: descriptive hits at cgap-git/src/{idx_read.rs:1,ladder.rs:1,
  pkt_line.rs:16} and cgap-gate/src/penstock.rs:482; the measured-claims
  ledger itself marks the term UNSUPPORTED (11-measured-claims.md:53,
  row 29).
- N3: receipt_chain.rs mentions Sigma_PQC only in negation comments
  (:19-20, :508); zero code law.
- N4: 90% on disk is the churn-protocol mix (00-ard-prd.md:65;
  01-release-gates.md:128,133,135); zero cost/data-reduction rows.
- N5: REVENUE_GENERATION_COMPENDIUM.md:39,180 and asc450.rs carry the
  $5_000_000_000-cent ceiling; fix-10-license-aggregation.md:117-138
  reconciles the figure across layers.

Net: every red-NN UNSUPPORTED verdict verified except red-20 register
row 75 (SHM boundary), which is WRONG — see §5.

## 4. Overclaim sweep — cite spot-checks (10)

| cite | resolves? |
|---|---|
| red-16 HYPERSCALER:262 "not headcount" | YES — HYPERSCALER_REPLICATION_TIMELINE_REPORT.md:262 |
| red-16 09_brooks_law:78-80 caveat | YES — models/09_brooks_law_and_coordination_drag.md:78-80 |
| red-14 ingress_ring.rs:101-107 andon class | YES (FIX-04 note added at :95-104) |
| red-09 pack_demux.rs:274 feed | YES |
| red-10 chains-ingress.rs:39-43,61 pool | YES |
| red-20 receipt_chain.rs:71 RingReceiptChain | DRIFT — struct now at :84 (post-FIX-08); exists |
| red-12 sram gate :51-54 DEPENDENCY_CRATES | DRIFT: now :52-56 (post-FIX-02); bn3 added |
| red-18 12-ard-prd-cgap.md:31 | DRIFT: line now other sentence; term exists nearby |
| red-13 sovereign.rs:863-892 claim() | DRIFT (post-FIX-01): re-key now :848-893 |
| red-05/20 Tranche/Tripartite rows | YES (self-consistent) |
7/10 resolve exactly; 3 line-drifted — every drift is a post-fix line
shift in a file a FIX lane edited, not a fabricated cite. No verdict
found rounding UNVERIFIED up: red-11 keeps its UNVERIFIED arm flagged
(red-11:93), red-02/red-10/red-15 keep PARTIAL/UNSUPPORTED as filed.

## 5. Findings

### F1 (SPOILER — red-20 register row 75 / red-13 SHM sub-claim is wrong)

red-13:20-25 asserts "zero `shm_open`/`shm` usage anywhere in crates/"
and red-20:75 inherits it ("zero shm_open anywhere in crates/"). Both
are contradicted on disk: `crates/cgap-beam/src/shm.rs` (44,374 B,
mtime 2026-10-31 22:42 — PREDATES the red-13 report, mtime 2026-11-02
17:09) carries 7 `shm_open` hits including a Darwin dependability
caveat (shm.rs:47). The "Darwin-vs-Linux SHM boundary" the redteam
marked UNSUPPORTED has a partial on-disk referent. red-13's grep scope
was wrong at its own report time. red-20's consolidated count ("13
distinct narrated targets") should read 12 verified + 1 refuted row.

### F2 (informational — cite line-drift is systemic, benign)

3/10 spot-checked cites are line-drifted because FIX lanes edited the
cited files afterward. Recommend the receipt lint treat redteam doc
cites as section-level, or FIX lanes append a one-line cite-refresh
note. No action required this lane.

### F3 (open-by-design register, honest)

- red-11 riscv reentrancy: ref_store arms addressed by rc01/rc02 lanes
  (lane-rc01-ref-store-reentrancy.md, lane-rc02-ref-store-width-arms.md);
  the reentrancy-unsafe arm itself remains the open item red-11:93
  flags (UNVERIFIED arm kept, never rounded up).
- red-13 structural item (commit path mints zero receipts): FIX-08
  hardened the doc-code divergence in receipt_chain.rs (the Σ_PQC
  negation comments at :19-20, :508 are FIX-08's), but the commit-path
  wiring remains absent as implemented — red-13 §3 stays UNSUPPORTED as
  implemented, correctly not rounded up.
- red-14 quota seam is caller-fed; no in-tree host calls note_refusal
  outside its own court. Design scope, disclosed in §2.3.

## 6. Commands + exits (this session)

1. `CARGO_TARGET_DIR=/tmp/r19-target cargo test -p cgap-ffi --test
   fa9_commit_gating` → exit 0, 4 passed / 0 failed.
2. `python3 scripts/run_checked.py --out /tmp/r19/sram.log --
   python3 scripts/verify_cscb_sram.py` → exit 0 (run_checked rc=0;
   gate prints bn3 charge 6,520,984 B, --subject bn3 refuses typed).
3. `CARGO_TARGET_DIR=/tmp/r19-target cargo test -p cgap-git --test
   posh_alloc_sentry_court` → exit 0, 1 passed.
4. `CARGO_TARGET_DIR=/tmp/r19-target cargo bench -p bench --bench
   chains-ingress -- ingress_chain` → exit 0 (background task
   bsooeej9t; CONTENTION-GUARD quiet, 1M leg fastest 11.29 ms).
5. `CARGO_TARGET_DIR=/tmp/r19-target cargo run --release
   --manifest-path /tmp/r19/red14/Cargo.toml` → exit 0
   (FLOOD-RE-RUN OK ... quota_class=101).
6. Grep battery (§3) — all receipts above are from live runs this
   session; absent-term rows all rc=1 / files=0.

## 7. Standing

**ALIVE** — 5 reproducer re-runs + grep battery executed this session,
each with the command and exit recorded in §6. Two fix-verified
closures (FIX-01, FIX-02), two stand-as-filed re-runs (red-09,
red-10-baseline), one seam verified with a disclosed design scope note
(FIX-04). One SPOILER finding (F1) against red-20 register row 75 and
red-13's SHM grep. No UNVERIFIED-round-up found in the corpus. Shared
tree untouched except this file. Build-root lease /tmp/r19-target
released below at lane end.
