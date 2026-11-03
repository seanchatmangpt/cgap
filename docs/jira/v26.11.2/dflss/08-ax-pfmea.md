# v26.11.2 DFSS — AX-Fleet PFMEA (08)

Phase: Analyze (DFSS lane DFX-3). Supersedes the AX-relevant rows of
`04-pfmea-risk-matrix.md` and extends them with the interop/AX failure
classes earned this session (2026-11-02). RPN = SEV x OCC x DET,
1-10 scales. DET columns are scored from ACTUAL witnessed evidence;
a control with no witnessed flip is DET scored HIGH, never low
aspirationally. Charter-level FMEA retained at
`docs/jira/v26.11.2/14-dflss-charter.md:94-105` (cross-link, no
duplication).

## Detection-evidence citations (witnessed, on disk)

| evidence | path:lines | what it witnesses |
|---|---|---|
| AUD-01 stale-binary catch | audit/dod-lsp-max-wire.md:139-142 | release binary G05 cited was stale vs post-12:53 source edits; caught by post-hoc audit, rebuilt fresh |
| G05 wire rebuild | lanes/lane-g05-lsp-wire-rebuild.md:11-16 | fresh rebuild + literal-count witness (ANDON_REFUSED count=3 both profiles) |
| AX-0 gate probes | lanes/lane-ax-00-gate.md:15-26 | docker daemon down exit 1 -> `open -a Docker` exit 0 -> bounded poll exit 0; `ax` absent exit 127; cluster exit 1 conn-refused |
| AX-0 vendor marking | lanes/lane-ax-00-gate.md:71-76 | ax CLI BLOCKED (absent, install unapproved); AX API surface UNVERIFIED (vendor-README material) |
| GAP-05 DOC-RECEIPT class | lanes/lane-gap-05-receipt-lint.md:48-50,62-76 | DOC-RECEIPT-002/003 STRICT-FAIL exit 1 witnessed; counts 002=81 / 003=145 findings |
| DXD-03 cgo drift | audit/dxd-03-go-ergonomics.md:12-16,61-64 | two divergent /tmp cgo -L/-rpath target dirs (ipc2 vs ipc3); absolute paths bake /tmp + /Users/sac |

## Matrix (ranked by RPN, desc)

### AX-fleet rows (new this session)

| # | Risk | SEV | OCC | DET | RPN | Detection control (witness status) | Rank |
|---|------|-----|-----|-----|-----|-----------------------------------|------|
| A1 | Stale-binary false-pass (darwin binary predates source; container analog = frozen .so in image) | 9 | 4 | 7 | 252 | darwin WITNESSED post-hoc: AUD-01 caught G05's stale release binary (audit/dod-lsp-max-wire.md:139-142), rebuilt per G05 (:11-16). Detection is manual audit, no mechanical gate. Container analog: AX-3 symbol-count court PENDING (no lane-ax-03 receipt on disk, checked this session) — that channel ungated. DET=7 scored for CURRENT state. | 1 |
| A2 | Vendor-claim drift (AX README mechanics asserted as fact) | 7 | 4 | 8 | 224 | Detection = AX-10's VENDOR-CLAIM-UNVERIFIED marking. Currently MANUAL: grep of docs/jira/v26.11.2 found ZERO occurrences of the token; the only landed marking is AX-0's prose UNVERIFIED verdict (lane-ax-00-gate.md:73-76). No mechanical rule. DET=8 honest. | 2= |
| A3 | Dual-target-dir drift (two divergent /tmp cgo -L/-rpath dirs; wrong dylib linked, court passes against stale artifact) | 8 | 4 | 7 | 224 | WITNESSED occurrence: dxd-03 found /tmp/ipc2-target vs /tmp/ipc3-target with divergent -L/-rpath and absolute paths baking /tmp + /Users/sac (audit/dxd-03-go-ergonomics.md:12-16,61-64). Prevention only: AX-3's fixed /opt/cgap/lib path (design, not yet built). Container court PENDING. DET=7 honest. | 2= |
| A4 | Cluster-apply hearsay (doc cites `ax apply`/k8s apply with no exit code) | 8 | 5 | 5 | 200 | Partial-mechanical: GAP-05 DOC-RECEIPT-002 (cited lanes/waves path does not resolve -> STRICT-FAIL exit 1, witnessed lane-gap-05-receipt-lint.md:49,66) + DOC-RECEIPT-003 (status-token/as-of header, :50,68) catch unbacked docs AFTER landing; they do not verify that an exit code exists behind a resolvable path. AX-0's falsifier (lane-ax-00-gate.md:60-62) defines the class; OCC witnessed across the IPC fleet. Residual detection manual. DET=5. | 4 |
| A5 | docker-daemon-down at gate time | 5 | 4 | 3 | 60 | LANDED, WITNESSED: AX-0 probe sequence daemon-down exit 1 -> `open -a Docker` exit 0 -> bounded poll on `docker info` exit 0 (lane-ax-00-gate.md:20-22). No Process.sleep; observable poll with stated deadline. Detection works; DET=3 for a loud, fast, typed failure. | 5 |

### Carried rows from 04 (kept ONLY where detection witness exists)

| # | Risk | SEV | OCC | DET | RPN | Detection control (witnessed flip) | Rank |
|---|------|-----|-----|-----|-----|-----------------------------------|------|
| C1 (=04 R7) | Agent long-file write corruption | 7 | 6 | 2 | 84 | DX-2 PostToolUse hook landed, court battery incl. fire->clean byte-restore flips (lane-dx-02-write-integrity.md:7-10,26-43) | 6 |
| C2 (=04 R9) | Receipt drift in landed docs | 5 | 7 | 2 | 70 | receipt_lint --strict exit 1 (444 findings); summary-path KeyError crash noted (04-pfmea-risk-matrix.md:18,33) — findings list prints fully before crash | 7 |
| C3 (=04 R1) | Forbidden branch in branchless kernel | 8 | 3 | 2 | 48 | disassembly-invariants court, fresh exit 0 incl. arm E binary mutant (04-pfmea-risk-matrix.md:15,25) | 8 |
| C4 (=04 R3) | SRAM/.text envelope overflow | 7 | 3 | 2 | 42 | sram-bloat mutant measured 136,068 B > bound — output flips, process exit 0 report-only (04:17,27) | 9 |
| C5 (=04 R2) | Heap alloc in no_std kernel path | 8 | 2 | 2 | 32 | CountingAlloc breach court, CSCB_MUTATE=alloc -> exit 1 (lane-b03-cscb-c4.md:18-20) | 10 |

### Dropped from 04 (detection witness ABSENT — stay in 04, not carried)

| 04 row | why not carried |
|---|---|
| R6 shared-fixture race (RPN 336) | 04 scored DET=7 for the then-missing DX-6 court; lane-dx-06-fixture-races.md NOW EXISTS on disk (landed after 04) — its witness must be re-read and re-scored by its owner before this matrix can carry it. Dropped rather than carried UNVERIFIED. |
| R8 stale-binary (RPN 288) | superseded by A1 (same failure class, AX-fleet scope, darwin witness now on file). |
| R10 429 lane death (RPN 180) | detection is manual receipt-absence inventory; no mechanical witness — honest DET=6, stays in 04. |

## Top-3 flagged for DFX-6 control plan

1. **A1 stale-binary false-pass — RPN 252.** Control: land the AX-3
   symbol-count container court (frozen-.so analog of the darwin catch);
   re-score DET 7->2 once a lane-ax-03 receipt with a witnessed flip
   exists. Darwin side: promote AUD-01's post-hoc audit into a mechanical
   mtime-freshness gate.
2. **A2/A3 tie at RPN 224.** A2 vendor-claim drift: make VENDOR-CLAIM-UNVERIFIED
   a receipt_lint rule (mechanical), re-score DET 8->4. A3 dual-target-dir:
   land AX-3's fixed /opt/cgap/lib as the sole cgo -L/-rpath root + a court
   asserting one resolved dylib path; re-score DET 7->3.
3. **A4 cluster-apply hearsay — RPN 200.** Control: extend receipt_lint with
   an exit-code-existence rule for apply/apply-adjacent citations
   (complements DOC-RECEIPT-002/003); re-score DET 5->3.

A5 (60) needs no DFX-6 control — its detection is landed and witnessed.

## Anti-vacuity of this matrix

Mutation named: delete any DET-evidence citation and the corresponding
RPN rises and the ranking changes — e.g. A1 without the AUD-01 cite
falls back to 04-R8's ungated DET=8 -> RPN 288 and A5 without the AX-0
poll witness rises to DET=8 -> RPN 160, reshuffling ranks 4-5. The
ranking is falsifiable by receipt absence. Each honest-high DET (A1
container channel, A2, A3 residual) is scored for the CURRENT on-disk
state, checked this session; PENDING courts are named, never assumed run.

## Divergences (doc-yields-to-reality)

- VENDOR-CLAIM-UNVERIFIED as a mechanical rule does not exist on disk;
  only AX-0's prose marking lands. Dispatch phrasing "AX-10's
  VENDOR-CLAIM-UNVERIFIED marking" is treated as planned, not landed.
- No lane-ax-03 (symbol-count court) receipt exists; both A1 and A3
  container channels are PENDING and scored DET-high honestly.
- 04-R6's dispatch-named missing receipt has since landed
  (lane-dx-06-fixture-races.md); not re-scored here (not this lane's
  evidence to re-derive) — flagged for the 04 owner.
- receipt_lint strict summary crash (04:18) remains open; GAP-05's
  landed rule class (lane-gap-05-receipt-lint.md) is the later state
  and is what this matrix cites.
- fix-09 manifest cross-refs, corrected against disk (re-read
  2026-11-02, this lane): (a) the chain link law
  `BLAKE3(prev_hash || block || license)` lives at
  `crates/cgap-ffi/src/receipt_chain.rs:117-121` (`fn link`) — the
  manifest's ":104-106" referent is the `receipts()` accessor, stale
  (manifest errata at fix-09-wire-constant-manifest.md:118-123 names
  :104-106; superseded by this read). (b) `trap.rs` is
  `crates/cgap-p28/src/trap.rs` (`ERR_P28_SPOLIATION = 0x5032_38FF`
  at :36); cgap-ffi's pin is `sovereign.rs:90`. No fix-09 dispatch
  text is amended by this lane — corrections recorded here per the
  ERRATA-never-rewrite law.

## 4. Counter-hardening design (DF-04, 2026-11-02)

Extends this PFMEA with the design response to the RED wave
(`redteam/red-20-hardened-survivor-synthesis.md` §1, 18 breach rows).
Two registers: §4.1 technical poka-yoke, §4.2 legal/governance
redesign. Per-item status vocabulary:

- LANDED-CODE: the code change is on disk in the shared tree this
  session (verified by reading it); a fix-NN receipt may or may not
  exist yet.
- REGISTERED-DESIGN: the repair is designed and registered (redline
  text or named edit) but NOT yet applied to its subject on disk.
- WITNESSED-RUN: this lane ran the artifact this session with exit
  codes captured.

### 4.1 Technical poka-yoke (six items)

| # | Item | Status | Design + on-disk evidence |
|---|---|---|---|
| PY-1 | Lease identity: spill-gate path aliasing (red-13 B6) | LANDED-CODE | Registry key derived from `fs::canonicalize` (`crates/cgap-ffi/src/sovereign.rs` `canonical_key` :898-949; claim keying `fnv1a(&canonical_key(path))` at :945), collapsing `dir/./x`/symlink aliases into one lease; racer denied typed before IO. Court arm `fa9_alias_race_denied` (`crates/cgap-ffi/tests/fa9_commit_gating.rs:261`, re-read this session). FIX-07 compile-freeze unblock disclosed in-file (sovereign.rs:900-902) and in lane-fix07-ffi-surface.md:57-64. red-13 finding: `redteam/red-13-receipt-desync-exploit.md:27-56`. fix-01 receipt NOW ON DISK: `lanes/lane-fix01-fa9-alias-rekey.md` (ALIVE, sha256-pinned subject, court + flips) — prior "receipt pending" superseded. |
| PY-2 | SRAM capacity: bn3 outside every SRAM claim (red-12 B8) | LANDED-CODE + WITNESSED-RUN | `scripts/verify_cscb_sram.py` parses NA_MAX_* consts from `crates/cgap-bn3/src/noalloc.rs` (`parse_na_consts` :158), builds a static capacity model (`bn3_capacity_table` :174-204), cross-checks against red-12 host size_of pins (BN3_PIN_MATCHES/ENGINE_BYTES :67-68, pin checks :196-203, drift = typed refuse), and charges the 131,072 B envelope (:206-218). This lane ran `python3 scripts/verify_cscb_sram.py --subject bn3` — exit 1, `BN3_ENCLAVE_REFUSED: 6,520,984 B > 131,072 B (49.75x)` — the gate SEES the engine and refuses loud. Anti-vacuity mutation `--mutate bn3-pin` registered (:377-385, halved NA_MAX_MATCHES must trip the pin check). fix-02 receipts NOW ON DISK: `lanes/lane-fix02-bn3-sram-gate.md` + `lanes/fix02.alive`; RC-03 added the ENGINE-side init-time bound (`lanes/lane-rc03-bn3-capacity-bound.md`, ALIVE: court exit 0, mutation flip, gate fence re-run exit 1 refusal + exit 0 baseline). |
| PY-3 | Demux poison-tail: post-trailer absorption (red-09 F1/F2) | LANDED-CODE + WITNESSED-RUN | `DemuxError::TrailingBytes(u64)` typed refusal (`crates/cgap-git/src/pack_demux.rs:77`, fired :379, observable `trailing_poison()` :270); surplus guard refuses `ObjectSurplus{seen,declared}` (:85, close_record guard :454) — F2's guard is a typed invariant arm, public-API surplus refused via TrailingBytes. F1 and F2 flip courts BOTH witnessed: mutations m1/m3 exit 101, sha256-exact restores (`lanes/lane-fix05-pack-demux-poison.md`, ALIVE, :48-56) — prior "fix-05 receipt not on disk" superseded. |
| PY-4 | Refusal quota/backoff: free class-101 flood (red-14 B15) | LANDED-CODE-REPAIRED + WITNESSED-RUN | `RefusalQuota` observational seam (`crates/cgap-git/src/ingress_ring.rs` + egress twin, `DEFAULT_MAX_REFUSALS = 4_096`, `CLEAN_ADMISSIONS_TO_CLEAR: u32 = 1024` named consts), wired into push/pop refusal paths: window trip at 4,096; escalation LEVEL-TRIGGERED — `tripped` survives admitted frames (ADV-12 spoiler-1 repair) and clears only via `acknowledge()` or 1024 consecutive clean admissions; `window_refusals` saturating_add at `u32::MAX` (ADV-12 spoiler-2 repair, no debug panic / release wrap); `total_refusals` wrapping_add; NEVER gates admission (a flood cannot wedge the ring) — red-14's rate-limit seam for any future host mapping `andon_exit_class` to `process::exit`. Repair mutants 4/4 FLIP witnessed (M1-M4 both twins, self-clear revert + `+=` revert; `lanes/lane-r-adv12fix-quota-repair.md` §3, ALIVE). Independent F-03 re-verify (2026-11-02): andon 5 passed + purity 1 passed exit 0; /tmp probe exit 0 — ring-level flood -> 1 admit -> `tripped()` still true, 1023/1024 hysteresis boundary exact, saturation + ack pinned. OBSERVATIONAL residual stays: throttle obligation remains with the host. Prior edge-triggered row (fix-04-era, `lanes/lane-fix04-andon-quota.md` :41-42 mutants) superseded by the level-triggered repair. |
| PY-5 | Single-core atomic fallback (red-11 B7) | LANDED-CODE-REPAIRED (execution-reentrancy leg REGISTERED) | `SealRegister::reserve` (`crates/cgap-git/src/ref_store.rs:321-330`) is the indivisible check-and-increment: ceiling guard `c < bound` lives INSIDE the `fetch_update` closure; full counter yields None -> typed `CounterExhausted`, wrap-to-0 impossible by construction. `stamp` (:283-311) calls `reserve` (:302); slot copy + chain fold happen AFTER the reservation (:306-309) — a reentrant observer sees a reserved lease, never a duplicated one. `Counter::fetch_add` (the unguarded RMW) removed. ARM SELECTION CORRECTED (slide repair, `lanes/lane-r-adv11fix-implcell-slide.md`, ALIVE): the floating nightly's `target_has_atomic` slide moved riscv32imc onto implcell; the ladder now REFUSES LOUD (typed compile_error naming `implcell_critical_provided` and "no CAS-capable atomic") until firmware binds the critical section via `RUSTFLAGS="--cfg implcell_critical_provided"` — the prior row premise `riscv32imc=impl32` is STALE and superseded. Independently re-verified this session (lane F-01): rv32imc no-opt-in build exit 101 with the typed refusal; opt-in build exit 0; host build exit 0; `ref_store_width_arms_court` 8/8 exit 0. implcell `critical` seam context: RC-01, made `pub(crate)` by RC-02's SLA unblock — disclosed in BOTH receipts (lanes/lane-rc01-...md:63-78, lane-rc02-...md:63-76). Execution-reentrancy court FLIP witnessed: reverted unguarded-RMW shape -> exit 101 (C1+C2+C5 red), restore -> 0 (`lanes/lane-rc01-ref-store-reentrancy.md` W3/W5, ALIVE). Width-arm compile witnesses: impl32/impl16/implcell mirrors + `compile_error` gate compile on host + riscv32imc (`lanes/lane-rc02-ref-store-width-arms.md`, ALIVE, courts w1-w5 exit 0 incl. 2 anti-vacuity flips). RESIDUAL, kept explicit: impl16/implcell arms are compile-witnessed only — NOT execution-reentrancy-court-certified (host=impl64; rv32imc=implcell, opt-in-bound; no court executes those arms' RMW). That leg stays REGISTERED; firmware critical-section binding + interrupt-safe RMW on rv32imc still owed (`redteam/red-11-riscv-reentrancy-corruption.md:96-103`). |
| PY-6 | FFI contract: header vs export surface (red-03 §1/§2) | LANDED-CODE + WITNESSED-RUN | Frozen slot surface mirrored into `include/cgap.h` (11 prototypes incl. `cgap_i32_batch` :193, `cgap_p28_invariant_check` :214; re-read this session) and gated by `scripts/ffi_surface_gate.py` (cdylib exports vs header prototypes 1:1, fail-closed exits 0/1/2/3). This lane ran `--self-check`: exit 0; baseline 11==11, and three mutation legs FLIPPED (fake prototype exit 1, dropped prototype exit 1, hostile 1-symbol shim dylib exit 1) with sha256-exact restores witnessed — the court is non-vacuous. fix-07 receipt NOW ON DISK: `lanes/lane-fix07-ffi-surface.md` (ALIVE; discloses its sovereign.rs:898-912 SLA unblock, cited under PY-1). |

### 4.2 Legal/governance redesign (four items)

| # | Item | Status | Design + on-disk evidence |
|---|---|---|---|
| LG-1 | Revenue-aggregation clause (red-04 B2) | REGISTERED-DESIGN (redline text only) | fix-10 supplies the full redline: Controlled-Group definition with five prongs (fix-10 §1b, §2.2.1), Group Aggregation Level (§2.2.2, "measurement is at the group, never at the individual license holder"), Anti-Structural-Avoidance (§2.2.3), numeric $50M floor §2.4 (fix-10 Part 2, with TTL operand remap coordination note fix-10:164-167). ALL THREE RED-04 attack structures trip per fix-10 Part 3 — as designed text. The license edit is COORDINATOR-OWNED (fix-10 header: "this lane owns NO license file"); LICENSE-P28-SIL.md is NOT amended on disk today — do not cite the instrument as amended. Landing court spec registered: delete §2.2.1 in a /tmp sandbox, structure-trip checks must flip TRIPPED→ESCAPES (fix-10:219-224). |
| LG-2 | Covenant/evidentiary repair register (red-05/06/08 B1/B3/B4) | REGISTERED-DESIGN (redline text only) | fix-11 consolidates R1-R10: covenant→condition restructure (R1), prospective-only §2.3 (R2, S-triage: concedes back-epoch revenue), presumptions→verification mechanics (R3), notice-and-demand replacing the ASC 450 mandate + SOX deletion (R4), privilege override deletion (R5), assent gate + privity rebind (R6, small-code), reciprocal covenant (R7), confidentiality/grantback (R8), arbitration-scope widening (R9), tariff-symbol definitions (R10, mandatory). Unfixable-without-changing-the-deal set named (fix-11:274-278). License text NOT edited on disk; rider/ERRATA pattern per LICENSE-P28-SIL.md:279-290 (fix-11:5-8). |
| LG-3 | AHDCLT pivot | LANDED (RETIRED-REFUTED on disk) | `docs/jira/v26.10.31/16-ahdclt.md:3-4` Status REVISED (2026-11-02): compute-kernel half of Pillar 1 RETIRED-REFUTED (fired falsifier RED-01, cited :20-25); Pillar 2 anti-LLM tripwires RETIRED-REFUTED (:80-84). No "unconstructable" or "tripwire" claim may be cited forward (:31-33). Surviving moat: state-chaining + adversarial courts + C-ABI integration cost (:35-44). Cite the pivot as LANDED; the retired claims are dead, not weakened. |
| LG-4 | Landing protocol for the license edit | REGISTERED-DESIGN | Sequence: coordinator lands LG-1+LG-2 redlines into LICENSE-P28-SIL.md (single edit, ERRATA pattern, not silent rewrite), then runs the fix-10 sandbox mutation court (delete §2.2.1 → trip checks must flip) and a TTL operand remap (fix-10:164-167) in the SAME transition, or instrument and machine layer diverge in the opposite direction. fix-10/11 produced diff text only; no license file edit is claimed or has occurred. |

Fence law: this is a docs-only lane; no code, gates, or courts were
edited. Fences that could regress: the two WITNESSED-RUN artifacts
were executed read-only (gate self-check restores verified
sha256-exact by the gate itself, output above); `verify_cscb_sram.py
--subject bn3` runs the gate's own read paths and mutates nothing in
the shared tree. Prior matrix rows A1-A5/C1-C5 and their scores are
unchanged by this addition; the ranking mutation (§ Anti-vacuity)
still applies to the new rows — deleting a PY/LG citation deletes the
status claim it grounds.
