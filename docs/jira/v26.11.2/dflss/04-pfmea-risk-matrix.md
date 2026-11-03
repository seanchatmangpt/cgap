# v26.11.2 DFSS — PFMEA Risk Matrix (04)

Phase: Measure->Analyze (DFSS lane DF-3).
Method: RPN = SEV x OCC x DET, 1-10 scales each.
DET columns are re-scored from ACTUAL gate evidence, each detection
control citing its witnessed flip (receipt + fresh re-run where stated).
Charter-level FMEA lives at `docs/jira/v26.11.2/14-dflss-charter.md:94-105`
— that table is retained there; this matrix supersedes its DET column
with witnessed evidence. Cross-link, no duplication.

## Detection-evidence fresh runs (this session, 2026-11-02)

| control | command | exit | note |
|---|---|---|---|
| disassembly-invariants gate | `python3 scripts/verify_disassembly_invariants.py` | 0 | CLEAN; arm E binary mutant `[PASS] anti-vacuity M-G03-E`; original archive sha256-verified unchanged |
| SRAM gate baseline | `python3 scripts/verify_cscb_sram.py` | 0 | 36,048 B <= 131,072 B |
| SRAM gate mutant | `python3 scripts/verify_cscb_sram.py --mutate=sram-bloat` | 0 | measured 136,068 B > 131,072 B — overflow flips in OUTPUT; process exit stays 0 (report-only mode, see divergences) |
| receipt_lint strict | `python3 scripts/receipt_lint.py --strict` | 1 | 444 claim findings on 189 docs; then CRASHED in summary (`KeyError: 'rule'`, scripts/receipt_lint.py:436) — defect recorded, not fixed (not owned) |
| n/a | fresh `cargo build`/`cargo test` | not run | not required by dispatch; standing claims re-read from receipts |

## Matrix (ranked by RPN, desc)

| # | Risk | SEV | OCC | DET | RPN | Detection control (witnessed flip) | Rank |
|---|------|-----|-----|-----|-----|-----------------------------------|------|
| R1 | Forbidden branch in generated branchless kernel code | 8 | 3 | 2 | 48 | disassembly-invariants court: census + linked witness + source-mutant + arm E binary mutant (lane-g03-disasm-gate.md:47-71); fresh exit 0 this session | 4 |
| R2 | Heap alloc in no_std kernel path | 8 | 2 | 2 | 32 | CountingAlloc breach court `cscb_c4_alloc`, `CSCB_MUTATE=alloc` -> exit 1 (lane-b03-cscb-c4.md:18-20) | 6 |
| R3 | SRAM/.text envelope overflow (131,072 B) | 7 | 3 | 2 | 42 | sram-bloat mutant measured 136,068 B > bound, overflow flips (lane-b04-cscb-c5.md:40); fresh re-measure this session, output flip witnessed, exit 0 (report-only) | 5 |
| R4 | DRed retraction leaves phantom survivors | 7 | 2 | 2 | 28 | skip-sweep mutant detects exactly 10 injected phantoms -> exit 1 (lane-b05-cscb-c6.md:26-35,61-62) | 8 |
| R5 | Certificate bitflip accepted by chain court | 8 | 1 | 2 | 16 | c7 bitflip court rejects mutated certificate; self-fails if bitflip no-ops (lane-b06-cscb-c7.md:40-46) | 10 |
| R6 | Shared-fixture race: fixed /tmp work dir -> false gate FAIL | 8 | 6 | 2 | 48 | LANDED (F-02 refresh; was MANUAL): `lanes/lane-dx-06-fixture-races.md` on disk — classifier court `crates/cgap-court/tests/shared_fixtures_court.rs` (3 passed, exit 0, pre/post flip t1=0 vs t2=101 witnessed in receipt), per-process `work()` OnceLock fixes in transpose/scan courts, SERIAL-mutex fix for in-process alloc-counter races | 4 |
| R7 | Agent long-file write corruption (~1 in 5) | 7 | 6 | 2 | 84 | write-integrity PostToolUse hook DX-2 LANDED (lane-dx-02-write-integrity.md:7-10), court battery incl. fire->clean byte-restore flips (lines 26-43) | 3 |
| R8 | Stale-binary / rustc-error save reads as 0 refusals | 9 | 4 | 8 | 288 | CURRENT STATE HONEST: no mechanical gate yet; `lanes/lane-lspa-02` and `lane-lspa-04` NOT PRESENT on disk (checked this session; only lspa-01/03 exist). DET=8 until LSPA-2/4 land | 2 |
| R9 | Receipt drift (stale/unbacked claims in landed docs) | 5 | 7 | 2 | 70 | receipt_lint --strict fires (exit 1, 444 findings) — fresh run this session; NOTE: strict summary path crashes KeyError 'rule' (scripts/receipt_lint.py:436); findings list itself printed fully before crash | 7 |
| R10 | 429 lane death mid-wave (silent partial edits) | 6 | 5 | 6 | 180 | CURRENTLY MANUAL: retry lanes GIT-03/GIT-07 re-landed after API 429 (lane-git-03-ingress-ring-receipt.md:1; lane-git-07-egress-ring.md:3); detection is receipt-absence + coordinator inventory, no mechanical court | 9 |

## Top-3 for DF-6 control plan

1. R10 429 lane death — RPN 180, detection manual inventory. Control:
   receipt-absence watchdog at wave drain; re-score DET 6->3.
2. R8 stale-binary false-pass — RPN 288, SEV 9, no gate until LSPA-2/4
   land. Control: mechanical rustc-error/refusal-count parity gate;
   re-score DET 8->2.
3. R9 receipt drift — RPN 70, lint fires but strict-summary crash
   (scripts/receipt_lint.py:436) leaves the gate without a clean exit-0
   path. Control: fix the summary path; re-score DET 2->1.

F-02 note: R6 (was top-1 at RPN 336, MANUAL) dropped out of top-3 —
DX-6 landed (classifier court + per-process fixes), re-scored
DET 7->2 / RPN 336->48; new rank 4 in the matrix.

R7 (write corruption, RPN 84) is held out of top-3 only because its
control DX-2 is already landed and witnessed; it is DF-6's cheapest win
(verify hook fires in settings.json, re-score DET 2->1).

## Anti-vacuity of this matrix

Mutation named: if any DET evidence cell is deleted, RPNs rise and the
top-3 set changes (R1 RPN 48->96 without the court cite) — the ranking
is falsifiable by receipt absence. Fresh runs above are the non-vacuity
witness that DET=2 scores were re-derived, not carried from the charter.

## Divergences (doc-yields-to-reality)

- SRAM mutant run exits 0 (report-only); the flip is in the measured
  byte count, not the exit code. lane-b04-cscb-c5.md:30 lists exit 0
  for the final court run consistently with this. A gate that reports
  a flip but exits 0 is a latent R8-class defect for DF-6.
- `receipt_lint.py --strict` crashes at scripts/receipt_lint.py:436
  (KeyError 'rule') after printing findings — DX-4 owner notified via
  this receipt (cross-lane disclosure; file not owned by DF-3).
- RESOLVED by DX-6 landing (was: `lanes/lane-dx-06` absent on disk,
  R6 DET scored for ungated state): `lanes/lane-dx-06-fixture-races.md`
  now on disk with classifier court + per-process fixes; R6 re-scored
  DET 7->2, RPN 336->48 by lane F-02 from the receipt's witnessed
  exits (14+3 passed, exit 0; flip t1=0/t2=101). Fresh re-run of the
  classifier court by F-02 NOT executed — evidence is as-of-receipt.
- Dispatch-named `lanes/lane-lspa-02`/`lane-lspa-04` do not exist on
  disk; R8 DET=8 reflects current ungated state honestly.
- rep1-census incident (R6 OCC evidence, per dispatch): grep across
  waves/ and lanes/ found no matching receipt line — recorded as
  session-hearsay OCC evidence, receipt locate UNVERIFIED.


## 3.1 Red-team FMEA (FM-01..FM-08) — landed 2026-11-02, lane DF-03

Added per lane DF-03 contract. Every row cites its red-team receipt;
STATUS reconciled against the fix wave AS OF THIS WRITE (fix receipts
re-read from disk, never recalled). RPN values are ASSESSMENT VALUES
(SEV x OCC x DET, 1-10 each), not measurements. Code lines re-read on
disk this session; drift vs the receipts is noted per row. Per-row
block format (not a wide table) to hold the 100-char line cap; column
order is # / receipt / code cite / SEV / OCC / DET / RPN / STATUS.

### FM-01 — FA9 spill-gate path-alias O_TRUNC

- Receipt: red-13 (`red-13-receipt-desync-exploit.md:27-56`) — exit-7
  repro, 2x; fa9 court green with the alias arm outside its grid
  (court-vacuity note, red-13:56-62).
- Code cite (current, verified on disk this session): `crates/
  cgap-ffi/src/sovereign.rs:889` (`mod spill_gate`); canonical
  text key = `canonical_key` at `sovereign.rs:931` (canonical-
  parent bytes + NUL + file-name bytes, raw-text fallback only
  for a nonexistent chain); identity key = `identity_key` at
  `sovereign.rs:976` ((dev,ino) FNV-1a when the path resolves,
  consumed at `:1024`/`:1056`); `path_is_symlink` at `:995`;
  grant-time `verify` at `:1049` (text key consumed at
  `:1023`/`:1055`/`:1075` in claim/verify/rebind); `rebind` at
  `:1071`. red-13's `:863-892` region now sits at
  `:889+` (fix01 + adv10fix deltas moved the lines). ADV-10's
  pre-repair cites (`canonical_key` `:898`, consumed `:945`) are
  superseded by this re-key — see
  `lanes/lane-r-adv10fix-spill-gate-rename-toctou.md`.
- SEV 8 / OCC 3 / DET 3 / RPN 72 (assessment; DET 3 carries from
  the fix01 re-score and is confirmed by the adv10fix court grid,
  see below).
- STATUS: CLOSED-RECEIPTED — two repair receipts, both landed
  2026-11-02. Layer 1: `lanes/lane-fix01-fa9-alias-rekey.md`
  (static-alias re-key, `dir/./x` flip, exit 101). Layer 2:
  `lanes/lane-r-adv10fix-spill-gate-rename-toctou.md` — (dev,ino)
  identity keying (rename-mid-lease no longer re-keys), grant-time
  verify (closes claim->grant world-change), synchronous rebind
  (no stale dual-entry ownership), and final-component symlink
  refusal (ADV-17 dangling-link case). Court: `cargo test -p
  cgap-ffi --test fa9_commit_gating` -> 7 passed, exit 0, re-run
  by lane F-02 this session (alias :275, rename :400, symlink
  :529, rebind :562 legs all green). Mutant flips witnessed by
  the repair lane: MUTANT-ADV10 (`identity_key` -> None) exit
  101, racer ADMITTED; MUTANT-ADV17 (`path_is_symlink` -> false)
  exit 101, owner ADMITTED (receipt W3).
- SCOPE LINE (stays loud; this closure does NOT claim the
  FS-portability frontier done): (a) O_NOFOLLOW|O_EXCL residual
  — the verify->create intra-syscall swap remains kernel-
  serialization-bound, documented in `spill_gate` code, separate
  lane (adv10fix W2 residual 5); (b) case-variant aliasing
  cross-FS portability UNVERIFIED by construction — ADV-10
  verdict C; closed ON THIS HOST, open on case-insensitive FS
  whose realpath does not normalize case; (c) 64-bit FNV-1a
  collision bound unchanged. CLOSED-RECEIPTED here means the
  rename/symlink keying variants are court-closed with witnessed
  flips; it is not a zero-residual claim.

### FM-02 — SRAM gate subject-list gap (cgap-bn3 ungated)

- Receipt: red-12 (`red-12-sram-overflow-exploit.md:27-75`) —
  verify_cscb_sram.py:51-54 DEPENDENCY_CRATES = blake3+branchless
  only; static NaEngine+NaMatches = 6,520,984 B host model /
  6,521,032 B riscv32imc ELF = ~49.75x the 131,072 B ceiling.
- Code cite (current, verified on disk): `crates/cgap-bn3/src/
  noalloc.rs:100` (NaMatches) and `:171` (NaEngine — drifted from the
  previously cited `:149`) — red-12's `:100-160` range still spans
  both structs; `scripts/verify_cscb_sram.py:51-54` now gates
  cgap-bn3 (FIX-02 delta).
- SEV 7 / OCC 3 / DET 2 / RPN 42 (assessment).
- STATUS: CLOSED — `lanes/lane-fix02-bn3-sram-gate.md` landed:
  bn3 gated, `--subject bn3` -> exit 1 BN3_ENCLAVE_REFUSED; bn3-pin,
  sram-bloat, and sandbox shim courts all flip; baseline 36,048 B
  exit 0 unchanged. Figure note: 6,521,032 B (dispatch) is the ELF
  charge; 6,520,984 B is the host struct model the gate enforces.

### FM-03 — Corporate-smurfing (no revenue-aggregation clause)

- Receipt: red-04 (`red-04-corporate-smurfing-loophole.md:70-124`) —
  affiliate rings, dispersed LLC trusts, unconsolidated 50/50 JVs
  escape the §2.2 tests at every node.
- Instrument cite: `LICENSE-P28-SIL.md:103` (§2.2 statutory
  boundaries), verified on disk this session.
- SEV 9 / OCC 4 / DET 7 / RPN 252 (assessment).
- STATUS: REGISTERED-DESIGN — `redteam/fix-11-covenant-repair-
  register.md` registers the aggregation-clause repair; no
  instrument edit on disk yet.

### FM-04 — ASC 450 meter self-reporting (circular evidence)

- Receipt: red-06 (`red-06-asc450-audit-defusal.md:100-139`, D4/D5) —
  SignedProof meter runs in-Deployer-process, in-memory, lossy by the
  instrument's own §6.2; estoppel clause anoints its own telemetry
  "conclusive" (circular); $0 reserve is the correct audit posture.
- Code cites (current, verified on disk): `crates/cgap-p28/src/
  asc450.rs:722` (`record_licensed`); `LICENSE-P28-SIL.md:155` (§4.1
  accrual mandate) and `:171` (SOX officer cert).
- SEV 9 / OCC 3 / DET 7 / RPN 189 (assessment).
- STATUS: REGISTERED-DESIGN — covenant repair register (fix-11)
  holds the structural repair; no meter redesign on disk.

### FM-05 — Clean-room constructability (Pillar-1 falsifier fired)

- Receipt: red-01 (`red-01-cleanroom-bypass.md:130-166`) — C99
  clean-room rebuild of all SWAR kernels + prefilter mix from doc
  comments alone, differential + exhaustive 524,288-lane sweep clean,
  1.02 ns/frame kernel-proxy; the pre-registered falsifier
  (`16-ahdclt.md:104-106` per red-01) fires.
- Cite drift: current `docs/jira/v26.10.31/16-ahdclt.md:90-91`
  carries the fired-falsifier statement; `:104-106` is now the
  See-Also block. Content verified, lines moved.
- SEV 6 / OCC 5 / DET 6 / RPN 180 (assessment).
- STATUS: REGISTERED-DESIGN — moat re-read as the measured
  three-layer form (`16-ahdclt.md:100-101`); no code delta applies.

### FM-06 — Timing jitter (unqualified latency claim dead)

- Receipt: red-10 (`red-10-timing-jitter-exploit.md:61-88`) —
  quiet-host 10.20 ns/frame reproduces within 4.5%; combined
  bw+cpu contention -> slowest sample 124.06 ns/frame (12.2x the
  claim), median 19.03 breaks the <15 ns target; bandwidth channel
  dominant; cpu-only fastest column survived (-3.4%).
- Code cite: `crates/bench/benches/chains-ingress.rs` (divan, 1M
  const); red-10 W3/W5 tables carry the numbers.
- SEV 5 / OCC 4 / DET 6 / RPN 120 (assessment).
- STATUS: REGISTERED-DESIGN — repair is claim-scoping (quiet-host
  qualification; unqualified/hostile reading dead), not code;
  11-measured-claims.md governs wording.

### FM-07 — Demux malformed-input surface (zero-alloc sentry)

- Receipt: red-09 (`red-09-alloc-sentry-breach.md:44-56`) — 15-case
  attack battery (bad magic, version lie, 1-byte segmentation,
  declared-count lie, 12x-0x80 varint, type-5, OFS flood, size lie,
  pkt-line quartet) all allocs=0 bytes=0; positive control green.
- Code cites (current, verified on disk): `crates/cgap-git/src/
  pack_demux.rs:301` (`feed`; red-09 cited `:274` pre-wire-in drift),
  sticky `TrailingBytes` refusal at `:379`, `ObjectSurplus` guard at
  `:454`, in-module flip courts at `:479+` (FIX-05 delta, landed).
  Varint shift guard per red-09 now under the feed path.
- SEV 5 / OCC 2 / DET 2 / RPN 20 (assessment; re-scored 30→20 by
  F-02 per lane-fix05's witnessed flips — header/body consistency
  erratum, 2026-11-02).
- STATUS: CLOSED-RECEIPTED — `lanes/lane-fix05-pack-demux-poison.md`
  (landed 2026-11-02): post-trailer poison now increments a
  `trailing_bytes` ledger and `feed` returns the sticky
  `Err(DemuxError::TrailingBytes(cumulative))`; `close_record` gains
  a typed `ObjectSurplus{seen, declared}` guard. F1 mutant m1 (Done
  arm reverted to `{}`) flips the poison court exit 101 (P2+P3
  VACUOUS); F2 guard mutant m3 (guard deletion) flips q2 exit 101
  with q1 twin green; both restores sha256-exact + mtime-bumped.
  Scoping honesty (per fix05 divergence 1): the `ObjectSurplus` arm
  is UNREACHABLE via the public API (the phase law enters ObjHeader
  only while `objects_seen < declared`), so its flip is proven by an
  in-module `#[cfg(test)]` court forcing the private state; the
  public-API surplus exploit (declare 1, supply 2) is refused via
  the TrailingBytes arm (court P3, flipped under m1). Zero-alloc
  fence holds: `posh_alloc_sentry_court` 1 passed, exit 0 (this
  session). DET re-score 3->2, RPN 30->20 (assessment).

### FM-08 — Andon trip-gate (forward DoS surface, no latch)

- Receipt: red-14 (`red-14-andon-dos-exploit.md:31-53`) — 2,000,001
  unauthenticated class-101 trips, 539.292 µs/1M refusals, ring
  liveness intact, NO latch exists; `octopus_merge_andon` panics on
  caller-supplied clashing head; no process::exit mapping on disk.
- Code cite (refreshed this session, verified on disk):
  `crates/cgap-git/src/ingress_ring.rs:210-212` (Empty => 101,
  post-adv12fix line drift; red-14's `:101-107` is pre-repair);
  no exit mapping anywhere in the tree.
- SEV 6 / OCC 2 / DET 4 / RPN 48 (assessment).
- REPAIRED (2026-11-02, lane F-03 verification of
  `lanes/lane-r-adv12fix-quota-repair.md`): the two ADV-12
  spoilers are fixed in code on both `RefusalQuota` twins.
  SPOILER-1 (edge-triggered self-clear): escalation is now
  LEVEL-TRIGGERED — `tripped` survives admitted frames and clears
  only via `acknowledge()` or `CLEAN_ADMISSIONS_TO_CLEAR: u32 =
  1024` consecutive clean admissions. SPOILER-2 (u32 window
  overflow): `window_refusals` uses `saturating_add` (no debug
  panic / release wrap); `total_refusals` hardened to
  `wrapping_add`. Mutation battery 4/4 FLIP witnessed (M1-M4,
  repair receipt §3).
- Independent re-verification this session (lane F-03): `cargo
  test -p cgap-git --test andon_liveness_court --test
  quota_alloc_purity_court` exit 0 (5 passed + 1 passed); /tmp
  path-dep probe exit 0 re-running the ADV-12 spoiler-1 shape at
  ring level (4,097 FrameTooLarge refusals -> ONE admitted frame
  -> `tripped()` STAYS true; 1023 admits still tripped, 1024th
  clears; saturate at `u32::MAX`; ack clears).
- RESIDUAL, kept explicit (IMMUNE — was not breached): the quota
  remains OBSERVATIONAL-ONLY; it never gates admission
  (`ingress_ring.rs` doc law). The throttling obligation stays
  with the host; any future host/CLI mapping `andon_exit_class()`
  -> `process::exit` MUST front a rate-limit/authenticity gate
  (RED-14 design law, unchanged).

### 3.1 STATUS register (per-row, as landed)

| FM | STATUS | Fix-wave referent |
|---|---|---|
| FM-01 | CLOSED-RECEIPTED | lanes/lane-fix01-fa9-alias-rekey.md + lanes/lane-r-adv10fix-spill-gate-rename-toctou.md (both landed; rename+symlink flips witnessed; O_NOFOLLOW and cross-FS case residuals stay explicit — see row scope line) |
| FM-02 | CLOSED | lanes/lane-fix02-bn3-sram-gate.md (landed) |
| FM-03 | REGISTERED-DESIGN | redteam/fix-11-covenant-repair-register.md |
| FM-04 | REGISTERED-DESIGN | redteam/fix-11-covenant-repair-register.md |
| FM-05 | REGISTERED-DESIGN | 16-ahdclt.md moat re-read (no code delta) |
| FM-06 | REGISTERED-DESIGN | claim-scoping under 11-measured-claims.md |
| FM-07 | CLOSED-RECEIPTED | lanes/lane-fix05-pack-demux-poison.md (landed) |
| FM-08 | PARTIAL BREACH/REGISTERED -> REPAIRED-RECEIPTED | adv-12 spoilers repaired in code: `lanes/lane-r-adv12fix-quota-repair.md` (4/4 mutants flip) + F-03 independent re-verify (courts 5+1 exit 0; probe exit 0); host-throttle residual stays explicit |

### 3.1 Divergences (dispatch vs disk)

- Dispatch named fix-01/02/05/04 for FM-01/02/07/08. Disk truth at
  the original write: only fix-02 mapped. SINCE RESOLVED: fix-01
  (`lane-fix01-fa9-alias-rekey.md`) and fix-05
  (`lane-fix05-pack-demux-poison.md`) landed against FM-01 and FM-07
  (note: the fix-05 dispatch name resolved to the pack-demux poison
  receipt, NOT lane-fix05-boundary-poison.md, which remains a
  boundary-gate deps court). lane-fix01-core-license.md and
  lane-fix04-cscb-p99.md still do not touch FM surfaces; no fix-03
  receipt exists on disk. FM-08 superseded: now REPAIRED-RECEIPTED
  (was REGISTERED-DESIGN at the original write) — ADV-12 spoilers
  repaired in code per `lanes/lane-r-adv12fix-quota-repair.md`
  (4/4 mutants flip) + lane F-03 independent re-verify
  (`lanes/lane-f03-pfmea-quota-refresh.md`; courts exit 0).
- F-02 refresh (2026-11-02): the dispatch for this lane described
  FM-01 as reading CLOSED-RECEIPTED-WITH-RESIDUAL; disk truth was
  plain CLOSED-RECEIPTED (the WITH-RESIDUAL intermediate status was
  never landed on disk). This refresh lands the final
  CLOSED-RECEIPTED state directly, with the residual scope line
  (O_NOFOLLOW|O_EXCL intra-syscall swap; cross-FS case-variant
  UNVERIFIED) carried in the row body. FM-01 row cites refreshed:
  spill_gate region drifted again post-repair (mod at :889,
  identity_key :976, path_is_symlink :995, verify :1049, rebind
  :1071; court legs fa9_alias :275, fa9_rename :400, fa9_symlink
  :529, fa9_rebind :562).
- RESOLVED (was: FM-01/FM-07 deltas unreceipted): FIX-01 landed
  `lanes/lane-fix01-fa9-alias-rekey.md` and FIX-05 landed
  `lanes/lane-fix05-pack-demux-poison.md`; both rows re-scored
  CLOSED-RECEIPTED this session. FIX-05's disclosed minimal unblock
  to `tests/posh_refusal_matrix_court.rs` (stored-framing
  clean_pack) is owned by that receipt, disclosed symmetrically.
- cgap-git lib transiently did not compile this session (E0063,
  egress_ring.rs:401, missing `quota` — another lane's in-flight
  edit, since landed by its owner); FM-07 sentry court re-run then
  passed (exit 0). Transient recorded loud; no lasting break.
