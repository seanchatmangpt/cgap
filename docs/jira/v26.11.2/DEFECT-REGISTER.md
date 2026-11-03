# DEFECT-REGISTER

Unified defect register for the v26.11.2 starDF campaign. Created by lane
G08 (2026-11-02), branch main @ 9a730320. Register-of-truth inputs:
`adversarial/THE-SMOKING-GUN-REPORT.md` plus the 2026-11-02 re-derivations
cited per row. No source edits; two doc files only (this file + the G08
receipt).

Standing vocabulary: CLOSED requires a command+exit witness run THIS
session on the exact subject; OPEN/BLOCKED carry the exact missing step.
Every CLOSED row names the receipt that would re-open it (anti-vacuity law).

## D-G08-1 -- WG2/G2-PARETO rollup rows (L1/L2/L4/L5): OPEN

- HANDOFF.md:128 (2026-11-02, lane F11): "rollup rows still UNVERIFIED".
  No post-dating receipt contradicts it: grep for the gate thresholds
  (`L1 <`, `L5 <`, `L4 <`) across docs/jira/v26.11.2/ this session hits
  only 00-ard-prd.md, 01-release-gates.md:56-88, 02-pareto-levers.md,
  adversarial/, repair-06, MEGA-REPORT and baseline research -- no
  per-level command+exit+flip witness for L1/L2/L4/L5 as a G2 receipt.
- Partial: L3 radix is CLOSED as a single level, not the rollup:
  repairs/repair-06-radix-default-flip.md:58-66 (5.6x @65,536; 78.3x @1M
  median; 23.72 ms fastest vs <75 ms; floor >20x MET) + :76-81 flip court
  (radix:true->false -> court FAILED, cargo exit 1, sha256-exact restore,
  mtime bumped, rerun exit 0). HANDOFF.md:129 says LANDED-MET only for L3.
- waves/w2-13-l4l5-courts.md:1 ("court shells (pre-lever controls)")
  records L4/L5 bench-level courts as "recorded, not faked" -- i.e. NOT
  executed as threshold courts; its cargo-test rows (:16-17) are control
  shells (exit 0), not L4/L5 threshold measurements.
- mega-bench lane-01 / lane-09 carry numbers (10.65 ns/frame;
  0.755-1.001 us/tx) but not the G2 receipt format of
  01-release-gates.md:56-88 (five numbers + per-level flip witness,
  scripts/divan2receipt.py as parser of record).
- Close condition (re-open receipt): a G2-PARETO receipt carrying all
  five level numbers + per-level flip witnesses per
  01-release-gates.md:56-88. L3 re-open trigger: any run where
  radix:nested @1M median <20x, or a flip court that accepts a mutated
  `radix` default.
- Lane G08 ran NO benches; nothing here overclaims a measured rollup.

## D-G08-2 -- FA10 102,187 B maximal no_std consumer: CLOSED (re-derived)

- Re-derived THIS session by the /tmp caller-crate method (CLAUDE.md
  footprint law), probe crate `/tmp/fa10-max-consumer` (FA10's own crate,
  same profile: opt-level=z, lto=fat, codegen-units=1, panic=abort,
  riscv32imc-unknown-none-elf):
  `cd /tmp/fa10-max-consumer && CARGO_TARGET_DIR=/tmp/g08-target cargo
  +nightly build --release --target riscv32imc-unknown-none-elf` -> exit 0
  (this session, 12:55 local).
- `llvm-size -A /tmp/g08-target/riscv32imc-unknown-none-elf/release/
  fa10-max-consumer` -> .text 102000 + .rodata 389 = **102,389 B**
  text+rodata family sum (exit 0). Verdict: under the 131,072 B envelope
  with ~22% headroom, MATCHING the FA10 witness (102,187 B) within +202 B
  nightly-toolchain drift -- same drift class already witnessed on the
  gate's live numerator (gate 606 B vs FA10's 484 B,
  countermeasures/lane-fa10r-verdict-rename.md:37-40).
- Corroboration on disk: adversarial/forensic-lane-10.md:81,115;
  hardening/lane-13-enclave-budget-witness.md:58,107 ("CITED (FA10), not
  re-measured" -- now re-measured, this row).
- FA10 remediation status: fix (c) verdict rename + fix (a) coverage
  caveat LANDED (scripts/enclave_budget_gate.py:667,670 grep witness, exit
  0 this session; scenario exits in
  countermeasures/lane-fa10r-verdict-rename.md:28-35). Fix (b) -- a
  BINDING maximal-consumer gate arm -- remains OPEN (lane-fa10r:46-48).
- Re-open trigger: a maximal-consumer build whose text+rodata sum exceeds
  131,072 B, or any no_std image gaining the std-gated scan reader
  (forensic-lane-10.md:87-93) without a gate re-measure.

## D-G08-3 -- FA1 bench-doc framing (chains-ingress.rs:24): OPEN

- crates/bench/benches/chains-ingress.rs:24 still reads:
  "`ingress_chain` -- the full 3-stage chain (the product)." FA1 witness:
  the escalated BLAKE3 leg is structurally unreachable on chain-admitted
  frames (escalate predicate == stage-1 refusal over all 256 flag bytes;
  escalated+dam-admitted = 0 of 65,536 frames; honest escalated cost ~316
  ns/frame = 33x headline) -- adversarial/forensic-lane-01.md:13,36,88-89;
  THE-SMOKING-GUN-REPORT.md section 1 row 1.
- This file is NOT G08's to edit. Recommendation to the owning lane: exact
  edit = replace the line-24 phrase "the full 3-stage chain (the product)"
  with "the 3-stage fast path (escalated leg structurally unreachable on
  chain-admitted frames, 0/65,536; FA1)" -- wording-only, no bench code
  change. Grade per smoking-gun section 2: HONEST-DISCLOSED at kernel,
  OVERCLAIM at framing.
- Close condition: the owning lane's receipt quoting the edited
  chains-ingress.rs:24 plus a re-run of the ingress bench (exit 0).

## D-G08-4 -- FA2 / Cpk n=1 (branchless 300.5M dec/s): OPEN (owned by G09)

- THE-SMOKING-GUN-REPORT.md section 3 item 3: "Single-run Cpk is
  impossible... the honest citation is single-session, this-session re-run
  246.1 M/s (18% noise band)"; forensic-lane-02.md:45-50 (re-run 4.06 ns /
  246.1 M-ops/s vs receipt's 3.33 ns / 300.5 M/s), :186 (verdict row).
- Pointer: G09 owns the 30-trial rig per dispatch. Close condition: a G09
  receipt with a 30-trial distribution (n>=30, stated method + machine)
  and a capability statement computed from it -- not from n=1. Until then
  the lawful citation is "single-session, 246.1-300.5 M/s band".

## D-G08-5 -- RV1 per-lane select_slice semantics: OPEN (law registered)

- Witness: forensic-lane-06.md:73-92 (RV1 section) and
  THE-SMOKING-GUN-REPORT.md section 4 RV1 row ("LANDED but INSUFFICIENT";
  census exits 1 on the current tree, 18/19, host select_slice BRANCHY
  1x cbz).
- Formal law (typed, no code edit by this lane):

  LAW RV1-PERLANE: `select_slice(a, b, cond)` is a per-lane select iff
  for every lane index i, result[i] is derived solely from a[i], b[i] and
  cond[i]. A form returning a whole slice chosen by cond[0] (the current
  repair, select.rs:31-38) is a SEMANTICS RETRACTION: it satisfies
  pointer-identity tests but is not a per-lane select and still emits a
  data-dependent branch (cbz on cond[0]), violating the hot-path contract
  "zero data-dependent control flow" (select.rs:3-10).
  Admissibility of any "branch-free select" claim requires BOTH:
  (1) semantic: differential oracle over the full lane-width domain
      (all cond bit patterns, mixed masks included) against a naive
      per-lane oracle;
  (2) codegen: census cell PASS (zero branch mnemonics) on host AND
      riscv32imc for the exact shipped symbol, via the manual
      rustc-build-and-run census path -- cargo test cannot see the court
      (forensic-lane-06.md:94-101, witnessed: cargo test exit 0 while
      standalone census exits 1).
  A claim meeting only (2) is codegen-true but semantics-false; only (1)
  is a retraction-flavored lie; both are required. Census 19/19 may be
  re-claimed only when the standalone census binary exits 0 on the tree.

- Close condition: a repair receipt with (1)+(2) witnesses and census
  exit 0. Re-open trigger: census exit 1 on any later tree.

## D-DOD-20 refresh (2026-11-02, lane DOD-20) -- APPEND-ONLY status lines per row

Re-census after the ~60-lane landing wave. Source of truth for this
append: `audit/tc-10-open-recensus.md` re-read fresh this session, plus
each cited receipt re-read from disk (not session memory). No history
row rewritten; every line below is dated and cites file:line.

- **D-G08-1 / R4 G2-PARETO (rollup)**: still OPEN,
  measured-but-failing -- L1 MET, L2 NOT MET 2.31x, L4@4096 NOT MET
  9.35 us/tx, L5 NOT MET with fused 29% REGRESSION (lane-gap-07:50-58),
  L1 flip court VACUOUS (lane-gap-07:90), divan2receipt.py REFUSED
  chains-* exit 1 (lane-gap-07:14-17). L3 remains CLOSED
  (repairs/repair-06-radix-default-flip.md:58-81 unchanged).
- **D-G08-2 / FA10 maximal consumer**: stays CLOSED as re-derived
  2026-11-02 (this register, D-G08-2 above); fix (b) binding-gate arm
  still OPEN per countermeasures/lane-fa10r:46-48. No new evidence.
- **D-G08-3 / FA1 bench-doc framing**: still OPEN. No lane receipt
  quoting an edited chains-ingress.rs:24 exists on disk this session
  (grep over docs/jira/v26.11.2/lanes/ exit 1). Close condition
  unchanged (owning lane's receipt + bench re-run exit 0).
- **D-G08-4 / FA2 Cpk n=1**: still OPEN (owned by G09); no 30-trial
  receipt on disk (grep exit 1 this session).
- **D-G08-5 / RV1 per-lane select_slice**: still OPEN; LAW RV1-PERLANE
  unchanged; no census-exit-0 witness landed.
- **FR-06 checkpoint prototype: DEFECT-OPEN -> PARTIAL (2026-11-02)**.
  Prototype EXISTS and is witnessed: zero-alloc-stage checkpoint.rs +
  tracking_alloc.rs landed under crates/cgap-core/src/ and
  #[path]-mounted as TEST-HARNESS modules, not lib members
  (crates/cgap-core/src/lib.rs:4-5; lane-cgap-prd-06-checkpoint.md:11,:29).
  Owed: (1) real lib mount (lib.rs membership), (2) fixpoint-scope --
  the 2,662-alloc fixpoint sweep is lane-posh-04-sentry.md:21, not in
  the checkpoint. Close = lib mount + fixpoint-scope receipt.
- **R1 C6/C3 gates: CLOSED** (2026-11-02). MV-5 DECIDED re-scope;
  c6 exit 0 min 211,174,510.4 facts/s vs 150M; c3 exit 0 min
  36,918.401 ms vs 60,000 ms; anti-vacuity mutants exit 1 (both)
  (lane-mv-05-sla-reconcile.md:22-27; tc-10-open-recensus.md section 1
  row R1). 800M / 25 ms are ASPIRATIONAL, never gating.
- **R2 parquet default-feature abort: CLOSED** (2026-11-02). Per-item
  std gates landed: default `default = []`, std-only court under
  `#[cfg(all(test, feature = "std"))]` (lane-fix02-parquet-features.md:7-13;
  tc-10-open-recensus.md section 1 row R2).
- **R8 build debris: CLOSED** (2026-11-02) per lane-gap-11-hygiene.md:13-20
  (tc-10-open-recensus.md section 1 row R8).
- **R9 member-count stale: CLOSED** (2026-11-02). "22-crate" corrected
  to 23-crate truth (bench/beam/court/integration/cgap-verification
  not counted the same way; publish=false table) --
  lane-mv-08-doc-refresh.md:7-15, R9 CLOSED by that lane (:34).
- **R10 zero-tax: OPEN-NARROWED, appends conflict flagged** (2026-11-02).
  GAP-04 claimed CLOSED-by-append (lane-gap-04-zero-tax.md:56,:81);
  MV-5's later append supersedes -- alias flips C3+C6 only;
  C1/C2/C4-runner/C5/C7/C8 ignore zero-tax
  (tc-10-open-recensus.md section 2 row R10; release-blockers:248-255).
  Dispatch named "GAP-04+DOD-11" as the closing pair: NO
  lane-dod-11*.md receipt exists on disk this session (ls exit 1), so
  the DOD-11 half of that closure is UNVERIFIED and NOT cited here.
- **R11 stale commit plan: CLOSED** (2026-11-02). R11 REFRESHED by
  MV-8 (lane-mv-08-doc-refresh.md:34); 252 fresh entries at
  release-blockers:185 per tc-10-open-recensus section 1.
- **NEW-7 cgap-core manifest_court::dod10_no_spdx: FIXED this session**
  (2026-11-02). Previously the ONLY failure, full-suite exit 101
  (lane-cgap-prd-06:5-8; owner unassigned then). DOD-01 landed the SPDX
  fix: suite green exit 0, anti-vacuity sandbox mutation re-flips the
  court at 101 (lane-dod-01-spdx.md:28-35). DOD-20 independent
  re-witness THIS session: `CARGO_TARGET_DIR=/tmp/dod20-target cargo
  test -p cgap-core` -> exit 0, 13 test binaries all `test result: ok`,
  0 failures, dod10_no_spdx passing. Close condition satisfied; row
  moves to CLOSED (re-open trigger: any later full-suite exit != 0 on
  manifest_court).
- **NEW-5 POSH-2 gate decision: still pending** (2026-11-02). Honest
  typed GATE FAIL, batch min=23.22/med=23.35/max=23.43 ns/frame = 2.3x
  (lane-posh-02-batch.md:29-32); release-only gate, debug build is a
  LOUDSKIP (:38). Close = operator gate-change receipt (R1 precedent)
  or repair to <=10 ns with flip court. Dispatch named DOD-12's
  package: no lane-dod-12*.md receipt on disk (ls exit 1) -- decision
  remains UNVERIFIED until that receipt lands.
- **AX-family registration gap (disclosed by DFX-6)**: DFX-6's census
  over the defect/audit corpus found ZERO AX rows registered in this
  register or release-blockers -- gap disclosed, not self-registered
  (lane-dfx-06-rollup.md:21-22). This append is the registration
  corrective: AX legs live in lanes/ax-* receipts (AX-0 GO through
  AX-9 NO-GO, lane-ax-09:8-11) and tc-10-open-recensus.md NEW-3/NEW-4,
  not as register rows until now.

## RW-09 append (2026-11-02, lane RW-09) -- corrosion/fix-wave reconciliation

Merge-not-clobber append: every row below was cross-checked against the
existing D-G08-1..5 rows and the DOD-20 refresh above -- no duplicates.
Finder receipt, repair receipt, and witnessed flip are named per row.
Statuses re-read from disk this session (2026-11-02).

### REPAIRED-RECEIPTED (defect, finder, repair, witnessed flip)

- **RW-09-1 RED-13 FA9 path-alias spill-gate bypass**: finder
  `redteam/red-13-receipt-desync-exploit.md:27-45` (breach reproduced,
  exploit exit 7 twice, 64/64 alias commits CGAP_OK); repair
  `lanes/lane-fix01-fa9-alias-rekey.md` (canonical_key re-key; mutant
  sandbox revert -> court exit 101, racer 20000/20000 ADMITTED, :42,52-53);
  status REPAIRED-RECEIPTED. Extended by RW-09-5 below.
- **RW-09-2 RED-11 ref_store ISR reentrancy window**: finder
  `redteam/red-11-riscv-reentrancy-corruption.md:66-74` (preempted
  check-then-act window; lease 6 > ceiling 5 witness, exit 0); repair
  `lanes/lane-rc01-ref-store-reentrancy.md` (indivisible
  `SealRegister::reserve`, guard inside fetch_update; court C1-C5 exit 0);
  status REPAIRED-RECEIPTED. Residual impl16/implcell legs -> RW-09-O3.
- **RW-09-3 RED-11-R2 toolchain cfg slide (width arms regress to u32 on
  rv32)**: finder+repair `lanes/lane-r-adv11fix-implcell-slide.md` (W1
  cfg probe :13-27 quantified the slide; compile_error names both cfg
  families; flip legs w4 width-mutation + w5 std-gate flip, 8/8 court
  exit 0, :63-69); status REPAIRED-RECEIPTED.
- **RW-09-4 RED-12 SRAM gate gap (cgap-bn3 ungated)**: finder
  `redteam/red-12-sram-overflow-exploit.md:26-37` (gate caller sees only
  two crates) + :61-73 (6.5 MB SRAM charge vs 131,072 B ceiling); repair
  `lanes/lane-fix02-bn3-sram-gate.md` (DEPENDENCY_CRATES + size_of pins
  from source of truth); flip `--mutate bn3-pin` exit 1 (:45-47); status
  REPAIRED-RECEIPTED.
- **RW-09-5 ADV-10 rename-mid-lease TOCTOU + ADV-17 symlink key-flip**:
  finder `redteam/corrosion/adv-10-fmea-risk-laundering.md:53-63` (B
  verdict SPOILER-CONFIRMED, exit-7 shape via rename) +
  `redteam/corrosion/adv-17-unblock-semantic-collision.md` (attack 1:
  absent->symlink key-flip, write-through to /d/y witnessed); repair
  `lanes/lane-r-adv10fix-spill-gate-rename-toctou.md` ((dev,ino)
  identity_key + grant-time verify + rebind + symlink refusal in claim
  AND verify); flips MUTANT-ADV10 / MUTANT-ADV17 exit 101 each (:43-45),
  fa9 court 7/7 exit 0 (:42), full cgap-ffi 83 tests exit 0 (:46);
  status REPAIRED-RECEIPTED. Residuals -> RW-09-O1/O2.
- **RW-09-6 ADV-12 quota x2 (observational self-clear + u32 overflow)**:
  finder `redteam/corrosion/adv-12-control-plan-evasion.md` section 2
  (2M refusals tripped, ring admits at full rate; 100k cycles 1.277 ms);
  repair `lanes/lane-r-adv12fix-quota-repair.md` (level-triggered
  escalation + acknowledge() + 1024-clean-streak hysteresis; saturating_
  add window / wrapping_add total; disclosed unblock to
  quota_alloc_purity_court.rs, disclose-in-both-directions noted); flip
  battery M1 ingress self-clear revert -> andon court FAILED (section 3
  table); status REPAIRED-RECEIPTED.
- **RW-09-7 ADV-05-A gate-battery UTF-8 crash**: finder
  `redteam/corrosion/adv-05-gate-battery-vacuity.md` probe 2b
  (UnicodeDecodeError traceback, exit 1 by crash); repair
  `lanes/lane-r-adv05fix-gate-battery-utf8.md` (bytes capture +
  typed binary-output prefix); flip probe 2b post-fix exit 1 with typed
  row, no traceback; status REPAIRED-RECEIPTED.
- **RW-09-8 ADV-07 unqualified 23.13 ms L2 figure in exec docs**: finder
  `redteam/corrosion/adv-07-ctq-metric-arbitrage.md` probe 1
  (dod-12:26,81,85,93,261 + dod-19:58 UNQUALIFIED); repair
  `lanes/lane-r-dodfix-2313-qualifier.md` (5 dod-12 sites + dod-19:58
  qualified: row 9 = 42.58 ms governs, UNVERIFIED-FOR-PAPER); status
  REPAIRED-RECEIPTED.
- **RW-09-9 ADV-09-R2 CONTENTION-SUSPECT unwired**: finder
  `redteam/corrosion/adv-09-msa-gauge-falsification.md` (probe 4)
  via `lanes/lane-r-adv09fix-contention-doc-wiring.md` W1; repair same
  receipt W2 (06-bench-protocol section 8 lines 177-193 rejection law +
  control-plan SPC row :143-157); anti-vacuity arm = RC-11 flip
  (margin 2.1-30x); machine gate still TARGET -> RW-09-O7; status
  REPAIRED-RECEIPTED (doc-layer enforcement only).
- **RW-09-10 wave9_a07 zero-string debris refusal**: finder
  `lanes/lane-fix07-ffi-surface.md:66` + reproduction in
  `lanes/lane-rc08-zero-string-debris-skip.md` W2 (exit 1 GATE-REFUSED
  sections-unreadable lib.rmeta_); repair same receipt W3 (v1.4.3
  two-case debris-skip law); flips: pre-fix exit 1 -> post-fix loud
  [SKIP] exit-3 path + primary-subject exit 2 fixture (W4); status
  REPAIRED-RECEIPTED.
- **RW-09-11 receipt_lint --strict KeyError crash**: finder
  `lanes/lane-rc09-receipt-lint-keycrash.md` section 1 (KeyError('rule')
  repro, exit 1 fail-open-by-crash); repair same receipt section 2
  (normalize_finding_keys -> typed RECEIPT-LINT-MISSING-KEY row);
  flip key_crash_court arm 3 wired into --self-check; status
  REPAIRED-RECEIPTED.
- **RW-09-12 red-19-F1 shm_open SPOILER (zero-shm claim wrong)**: finder
  `redteam/fix-19-corpus-audit.md` section 5 F1 (red-13:20-25 +
  red-20:75 refuted; 7 shm_open hits in crates/cgap-beam/src/shm.rs,
  mtime predates red-13); repair = ERRATA per never-rewrite law:
  `lanes/lane-f04-redteam-errata.md:10-12,33` (section 6 ERRATA
  appended, originals untouched); status REPAIRED-RECEIPTED.

### OPEN (finder + owner TARGET; no repair receipt on disk)

- **RW-09-O1 case-variant portability residual**: finder
  `redteam/corrosion/adv-10-fmea-risk-laundering.md:73-78` (PROTECTED
  ON THIS HOST only -- APFS case-insensitive canonicalize collapses
  cap/X vs cap/x; case-SENSITIVE filesystems whose realpath does not
  fold case are unprotected); owner TARGET: a portable
  case-folding/identity-key lane; close = flip-witnessed court on a
  case-sensitive mount.
- **RW-09-O2 O_NOFOLLOW|O_EXCL intra-syscall swap residual**: finder
  `lanes/lane-r-adv10fix-spill-gate-rename-toctou.md:39-40` (verify->
  create window is kernel-serialization-bound) + finder
  `lanes/lane-f02-fm01-refresh.md:25` (residual kept loud); owner
  TARGET: O_NOFOLLOW|O_EXCL creation semantics lane.
- **RW-09-O3 impl16/implcell behavioral execution legs**: finder
  `redteam/red-11-riscv-reentrancy-corruption.md:93` (UNVERIFIED arm,
  flagged as filed); partial fix compile-only per
  `lanes/lane-rc02-ref-store-width-arms.md:25-29` (arms compile on every
  target; "behavioral RMW coverage of implcell remains open"); owner
  TARGET: behavioral RMW court on implcell hardware/emulator leg.
- **RW-09-O4 commit-leg receipt wiring**: finder
  `lanes/lane-ocel01-cgap-ocel2-adapter.md:99` ("commit leg itself has
  no receipt") + `lanes/lane-fix08-receipt-desync.md:31` (commit-leg
  non-repudiation UNSUPPORTED); owner TARGET: wire the commit leg into
  the receipt chain; until then never quote commit-leg non-repudiation
  as a claim.
- **RW-09-O5 R1-R10 execution legs still owed**: finder
  `audit/tc-10-open-recensus.md` (rows R4 G2-PARETO measured-honestly-
  failing, R10 OPEN-NARROWED, NEW-5 gate decision pending; section at
  :38,:93-96); cross-ref rows D-G08-1 and DOD-20 refresh above; owner
  TARGET: G2-PARETO receipt per 01-release-gates.md:56-88 + operator
  gate-change receipt for NEW-5.
- **RW-09-O6 M-CTQ cross-instrument wire legs UNVERIFIED**: finder
  `11-measured-claims.md:126-127` (cgo admit, Go-harness mmap read,
  1M-window seal "remain UNVERIFIED until re-measured on the wire
  instruments"; D1-D3 divergences :113-125); owner TARGET: re-measure
  on cgo/Go-harness instruments; never promote consolidation-addendum
  figures to MEASURED (wire) until then.
- **RW-09-O7 machine contention gate**: finder
  `lanes/lane-r-adv09fix-contention-doc-wiring.md` W2 item 1 (enforcement
  is PROCEDURAL; "machine gate = TARGET, unlanded"); owner TARGET:
  machine-consumed gate over CONTENTION-SUSPECT stderr markers; doc
  layer closed by RW-09-9.

## Lane receipt (RW-09)

- Files written: `docs/jira/v26.11.2/DEFECT-REGISTER.md` (append only;
  no existing row rewritten; stale-cite refresh N/A -- all D-G08/DOD-20
  cites re-read and still resolve on disk this session).
- Added rows: RW-09-1..12 (REPAIRED-RECEIPTED), RW-09-O1..O7 (OPEN,
  finder + owner TARGET). Commands: read-only greps + file reads only
  (all exit 0 except one benign sed BSD-arg error, no state change).
  No benches, no git, no shared-tree mutation.
- Standing: ALIVE (register reconciliation is a read-merge-write of
  receipts re-read from disk this session; every row cites file:line
  verified on disk above).
- Standing vocabulary honored: REPAIRED-RECEIPTED rows cite command+exit
  witnesses in their repair receipts; OPEN rows name the missing step;
  nothing UNVERIFIED rounded up.

## RW-09 red-09 F1/F2 closure (2026-11-02, lane FS-09, append-only)

- **RW-09-F1 red-09 post-trailer absorption**: finder
  `redteam/red-09-alloc-sentry-breach.md` W6 F1 (`Phase::Done => {}`
  silently absorbed bytes after the pack trailer, pack_demux.rs pre-fix
  :343). REPAIRED-RECEIPTED: `lanes/lane-fix05-pack-demux-poison.md`
  -- typed `Err(DemuxError::TrailingBytes(cumulative_count))`, sticky
  across finish. Witness this session: cargo test -p cgap-git --test
  pack_demux_poison_court, exit 0, 3/3 ok, including
  p2_post_trailer_poison_is_typed_refusal (m1-flipped).
- **RW-09-F2 red-09 surplus-object check**: finder same W6 F2
  (close_record had no surplus tripwire). REPAIRED-RECEIPTED: same
  fix05 receipt -- `DemuxError::ObjectSurplus{seen, declared}`
  invariant guard + private-state flip court (P3 exploit: declare 1,
  supply 2 refused; q2_surplus_close_refuses_typed m3-flipped).
  Same court run witness, exit 0.
- Register state found: no red-09 F1/F2 rows existed prior (neither
  PARTIAL nor REPAIRED anywhere; greps this session, zero hits for
  TrailingBytes/ObjectSurplus/red-09 F1/F2). This append is the first
  register record of the FIX-05 closure; no prior row rewritten.

## Second-wave append (2026-11-02, lane REG-01, append-only)

Merge-not-clobber: every row below cross-checked against D-G08-1..5,
the DOD-20 refresh, RW-09-1..12/O1..O7, and RW-09-F1/F2 -- no
duplicates. Finder, repair (or OPEN + owner TARGET), and witnessed
flip named per row; statuses re-read from disk this session. CE-05
egress-padding was checked at write time per dispatch and needs NO
row: CE-05 chose path (a) -- PaddedIndex #[repr(align(64))] landed
with a 3-test layout court whose mutation (drop repr(align(64)))
flips via 3x E0080 const-assert aborts
(lanes/lane-ce-05-egress-padding.md:20-26,30-37,44-45).

### SW-1 CONTENTION-SUSPECT guard blind spots (two witnessed classes): OPEN

- Finder 1 (evict-class channel):
  `redteam/corrosion/adv-15-claims-boundary-injection.md` W3 -- both
  evict legs ran GUARD-GREEN (0 markers) while degrading fastest
  +10.4%/+18.2% past the red-10 4.5% band; mechanism: uniform
  low-bandwidth cache-pollution keeps per-pass max/min ratio under
  the 2.0 in-run ceiling and shows the bw calibration no DRAM
  streaming. Guard certifies "not bw/cpu-contended", NOT "quiet".
  Corroborated by
  `bench-validate/opt-03-channel-curve.md` §3.2 leg 4 (bw6): in-run
  worst ratio exactly 2.00 at-but-not-over the ceiling, 0 markers,
  fastest +13.8% -- common-mode (uniform distribution shift)
  blindness, a second trigger mechanism for the same defect class
  (registered there as a lane-rc11 follow-up datum).
- Status: OPEN. Owner TARGET: evict-class detector channel lane +
  rc11 follow-up on common-mode shift; close = a witnessed flip on
  an evict-class hog leg that today runs guard-green
  (adv-15 W6 leg 1/leg 2 reproductions are the ready-made mutation
  battery), plus an rc11 guard-law update carrying the third channel.
- Anti-vacuity note: the guard itself flipped 4x under load in
  OPT-03 (in-run 3.89 / 2.45 / cal 3.13 / in-run 11.67) and fired
  both channels in adv-15 bw6/both legs -- the court is not vacuous;
  the defect is the false-negative class, not a broken guard.

### SW-2 FM-07 header/body RPN inconsistency (30 vs 20): CLOSED

- Finder: `dod/charter-s5-fmea-reconciliation.md` contradiction
  flag 3 (:143-149, lane CE-07) -- matrix row header carried
  SEV 5 / OCC 2 / DET 3 => RPN 30 while body STATUS text recorded
  the re-score "DET 3->2, RPN 30->20" (:224 vs :240 as found).
- Repair: coordinator-fixed on disk; re-read this session the row
  now reads DET 2 / RPN 20 in the body with an explicit
  "header/body consistency erratum, 2026-11-02" at
  `dflss/04-pfmea-risk-matrix.md` (FM-07 row block) and a
  consistent re-score line; the header/body disagreement is gone.
- Witnessed flip: N/A (numeric-assessment doc consistency fix; the
  underlying FM-07 court flips are fix05's m1/m3 exit 101, cited
  in the same row). No gate exercises the RPN digits; a court that
  cannot flip is registered honestly as doc-consistency class.

### SW-3 FM-01 risk-laundering (row predates its repair evidence): CLOSED

- Finder: `redteam/corrosion/adv-10-fmea-risk-laundering.md`
  (verdict B SPOILER-CONFIRMED) + CE-07 flag 2
  (charter-s5-fmea-reconciliation.md:138-142) + the E-09 audit
  folded mid-flight by F-02 (lane-f02-fm01-refresh.md:39-44).
- Repair: `lanes/lane-f02-fm01-refresh.md` -- FM-01 STATUS now
  cites BOTH repair layers (fix01 static-alias re-key + adv10fix
  (dev,ino) identity keying), keeps the O_NOFOLLOW|O_EXCL and
  case-variant residuals loud, and refreshes all code cites from
  disk (canonical_key :931, identity_key :976, courts :275/:400/
  :529/:562). Fresh court witness this session-of-F-02:
  cargo test -p cgap-ffi --test fa9_commit_gating -> exit 0,
  7 passed (f02 W3 table).
- Witnessed flips cited as-of-receipt: MUTANT-ADV10/ADV17 exit 101
  each (lane-r-adv10fix W3); F-02 explicitly did NOT re-run them.

### SW-4 workspace rustdoc gate FAIL (31+ typed errors): CLOSED

- Finder: lane DOD-02 census,
  `dod/dod-02-rustdoc-gate.md` -- workspace
  `cargo doc -D warnings -D rustdoc::broken_intra_doc_links`
  exited 101 with 31 located errors across cgap-arrow/bn3/bont/git/
  p28/parquet/storage.
- Repair: `lanes/lane-dod-02fix-rustdoc-gate-repair.md` -- all
  31 doc-side fixed plus 3 crates behind the DOD-02 parallel-abort
  wall (cgap-ffi, cgap root, bench) as a disclosed scope extension.
  Witness: RUSTDOCFLAGS="-D warnings
  -D rustdoc::broken_intra_doc_links" cargo doc --workspace
  --no-deps --document-private-items -> EXIT=0 (post-fix; pre-fix
  101 logged); cargo build --workspace exit 0; per-crate parquet
  re-court exit 0 closing the W6 FileSink divergence with cause
  (std-gated symbol, mmap_writer.rs:603).
- Witnessed flip: the pre-fix 101 exit itself is the flipped state;
  re-flip trigger = any later gate exit != 0 (the gate command
  above is the re-open receipt).

### SW-5 cgap-core missing license-file manifest key: CLOSED (1 residual OPEN)

- Finder: lane DOD-05 FLAG (`lanes/lane-dod-05-core-nostd.md:50`
  region: manifest lacked a license key) + lane DOD-01 F3a census.
- Repair: `lanes/lane-dod-01fix-cargo-manifests.md` --
  cgap-core/Cargo.toml gained `license-file = "../../LICENSE.md"`
  (branchless pattern); F3b/F3c fixed dangling `../LICENSE.md`
  paths (cgap-ocel2, csct); F3d added the key to cgap-fleet-ops;
  ontologies/cloud gained `exclude = ["RECEIPT.md"]` (DOD-05
  Finding 2).
- RESIDUAL OPEN (kept loud, per the receipt's REFUSED section):
  the cli half of F3d was REFUSED by the guard-generated.sh hook --
  cli/Cargo.toml is ggen-generated; the license key must enter via
  the cnv:Cli graph or manifest_patch.py; escalated unchanged to
  coordinator. Owner TARGET: cli regen cycle.
- Witnessed flip: pre-state `cargo package --list` showed zero
  license lines for cgap-core (DOD-05 FLAG); post-fix the LICENSE.md
  ships in the package list (DOD-01FIX courts section).

### SW-6 stale 91-court figure in 07-court-corpus.md §6: CLOSED

- Finder: lane X-05FIX (`lanes/lane-x-05fix-court-count.md`) --
  XC-05's bare 91 figure was historical; X-05FIX replaced it with
  95 (85 cargo-test + 5 corpus-doc + 5 gate-mode) but stood PARTIAL
  because the cited X-05 receipt was absent on disk at edit time
  (hearsay-cited, annotated as such).
- Closure: `lanes/lane-x05-court-corpus-audit.md` has since landed
  (standing ALIVE, its JSON terminus), superseding the hearsay
  citation; re-read this session 07-court-corpus.md:186-191 carries
  91-as-historical plus the live 95/85/5/5 figure. Both legs of the
  dispatch's stale-91 item are now receipted on disk.
- Witnessed flip: X-05's regeneration witness (conformance_kit_
  export byte-identical x2, cited by X-05FIX); re-open trigger =
  any court-count drift between the corpus and a fresh X-05-style
  census.

### SW-7 bench-05 quiet-reference mislabel: CLOSED (E2 carries LOST-WITH-WITNESS)

- Finder: lane A-03 errata, self-reported in
  `bench-validate/bench-05-guard-drift.md` §8 (:151-190) per the
  ERRATA-never-rewrite law (historical table untouched).
- E1 (quiet-ref label wrong, :156-176): CLOSED by the errata
  itself -- §4 row 1 is a HOT-AMBIENT reference (bench-01 leg 1 at
  loadavg 6.11-6.50, never quiet); every "vs quiet ref" delta is
  re-scoped as "vs bench-01 leg-1 hot-ambient reference"; the
  channel-split conclusion survives, the quiet-delta magnitudes do
  not and must not be quoted.
- E2 (:178-190): dangling "bw.err summary above" reference; raw
  calibration stderr lived only in /tmp/f07work (deleted).
  LOST-WITH-WITNESS, recorded not invented: in-run ratios 2.25/7.67
  + SUSPECT marker stand as session-recorded values; §3.1 must not
  be cited as containing raw calibration output. No repair is
  possible (evidence destroyed); the errata is the closure.

## Lane receipt (REG-01)

- Files written: `docs/jira/v26.11.2/DEFECT-REGISTER.md` (append
  only; no existing row rewritten; de-dup checked against every
  prior section).
- Added rows: SW-1 (guard blind spots, OPEN, owner TARGET),
  SW-2 (FM-07 RPN 30/20, CLOSED), SW-3 (FM-01 laundering, CLOSED),
  SW-4 (rustdoc gate, CLOSED), SW-5 (cgap-core license, CLOSED
  with cli-half residual OPEN), SW-6 (stale-91, CLOSED), SW-7
  (bench-05 quiet-ref, CLOSED; E2 LOST-WITH-WITNESS).
- CE-05 egress padding: checked at write time, NO row (path (a)
  landed with witnessed E0080 flip).
- Commands: read-only greps/seds/reads only this session (all exit
  0 except two benign zsh separator errors, no state change); the
  append itself via heredoc cat >> (exit 0).
- Standing: ALIVE for the register write; every CLOSED row cites
  receipts re-read from disk this session; OPEN rows name owner
  TARGET and the exact missing step; nothing UNVERIFIED rounded up.

### SW-5 ERRATA (2026-11-02, lane REG-01, append-only -- cli-half residual was STALE)

- The cli-half OPEN residual in SW-5 above is superseded: the cli
  license-file is CLOSED via the lawful graph path --
  `lanes/lane-cli01-license-via-graph.md` (ALIVE, re-read this
  session) added manifest_patch.py step 1.5 and ran the lawful ggen
  regen; flip witness `grep -c license-file Cargo.toml` = 1;
  byte-identical double-regen replay (gate0 discipline, cmp exit 0);
  cargo build exit 0. On-disk verification THIS session:
  `crates/cli/Cargo.toml:10` = `license-file = "../../LICENSE.md"`.
- Scope note: the DOD-01FIX hook refusal governed the DIRECT-edit
  path only; CLI-01 is the counter-example proving the hook --
  the generated-surface law and the license closure are both
  satisfied. CLI-01 also corrected the dispatch's `../LICENSE.md`
  depth error to `../../LICENSE.md` (crates/LICENSE.md absent).
- Remaining genuine OPEN sub-items under SW-5 (both coordinator-
  owned policy, per DOD-01FIX's escalation section): csct version
  0.1.0 vs fleet 26.11.2 (F1) and workspace MSRV/rust-version
  undeclared (F2). Owner TARGET: coordinator policy lane.
- Re-open trigger: any regen that drops the license-file line from
  crates/cli/Cargo.toml (the step-1.5 patch is the guard).
