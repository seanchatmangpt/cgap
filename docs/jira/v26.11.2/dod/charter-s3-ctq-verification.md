# Charter S3 CTQ Flowdown Verification (CE-02, 2026-11-02)

Lane CE-02 verification of charter CTQ flowdown against the M-CTQ
register and measured-claims rows, re-derived this session (commands +
exits in the W-R table at the bottom). Subject charter:
`../14-dflss-charter.md`. Register of record:
`../dflss/02-voc-ctq-matrix.md` (M-CTQ-01..07) and
`../11-measured-claims.md`.

## Numbering divergence (recorded, not fixed)

No on-disk charter numbers its CTQs 1..6: `14-dflss-charter.md` has an
UNNUMBERED 4-row "VOC to CTQ Flowdown" table (:68-75; the charter's
6th `##` section, not §3), the csct charter uses CTQ-01..07
(`dflss/01-project-charter.md:33-41`), the master register uses
M-CTQ-01..07, and the AX charter uses CTQ-01..06 with DIFFERENT
content (`dflss/02-ax-charter-ctq.md:21-28`). The CTQ-1..6 numbering
used below is the dispatch's own mapping; disk numbering diverges from
it everywhere. Per the divergence law, the doc yields to disk; the
mapping is recorded here rather than rewritten into the charter.

Per contract, `02-voc-ctq-matrix.md` was NOT edited: its M-CTQ
numbering is already the register the dispatch maps TO, so the
charter-vs-register numbering divergence lives entirely in the
charter files this lane does not own.

## CTQ-1 ingress < 15 ns SLA — VERIFIED (scoped quiet-host)

- Register mapping: M-CTQ-01 = claims row 5 (10.20 fastest / 10.30
  median ns/frame, divan, 1,048,576-frame const, 100 samples) and row
  8 (`11-measured-claims.md:29,32` — MET (10.20 < 15), "MEASURED
  (target met, scoped)", NOT robust under contention, row-28 pointer).
- Row 8 re-read from disk this session: exit criterion carries the
  quiet-host scope verbatim; "MET" never reads unscoped. Contentious
  behavior is row 28 (CTQ-4), not this row.
- No re-run (divan leg not re-benched this lane); row 5/8 figures are
  receipt-cited, re-read at use time — standing VERIFIED (scoped) by
  receipt, never unscoped.
- Verdict: charter goal G4 (<15 ns/frame) is CONFIRMED only as row 8's
  scoped claim. An unscoped "<15 ns" is not admitted by any register.

## CTQ-2 zero-alloc — VERIFIED (scoped, courts run this session)

- Register mapping: M-CTQ-02. Courts:
  `crates/cgap-git/tests/quota_alloc_purity_court.rs`
  (`quota_flood_alloc_purity` — exit 0 this session) and
  `crates/cgap-git/tests/posh_alloc_sentry_court.rs`
  (`posh_alloc_sentry_driver` — exit 0 this session, 5.88 s).
- Scope caveats are part of the claim (red-09 :74-82, re-read): the
  bn3 `unify_to_fixpoint` leg (2,662 allocs / 650,294,994 B) is
  explicitly scoped OUT of the zero claim
  (posh_alloc_sentry_court.rs:3-10, 299-334), and cross-FFI
  zero-alloc is UNSUPPORTED (unaudited; cgap-ffi host builds keep
  std by design).
- Verdict: 0-alloc holds on the ring/seal sentry surface under the
  CountingAlloc window; charter row "0 alloc, 0 blr" is CONFIRMED
  only within that scope.

## CTQ-3 mutation non-vacuity (court kit) — VERIFIED (95, not 91)

- The dispatch flagged the charter's "91" as stale per X-05; disk
  confirms: `07-court-corpus.md:186` marks XC-05's bare 91 as
  superseded (historical) and :191 states the live **95 courts**
  (85 cargo-test + 5 corpus-doc + 5 gate-mode). X-05FIX fix receipt:
  `../lanes/lane-x-05fix-court-count.md:5-15`. X-05 audit:
  `../lanes/lane-x05-court-corpus-audit.md:50-58` (95 = 85/5/5, +4
  post-XC-05 courts; manifest parse exit 0).
- The count is re-read from disk this session (grep 91|95 over
  07-court-corpus.md, exit 0), not hearsay-cited.
- Verdict: court-kit live count = 95. Any charter cell still carrying
  "91" is stale and must cite 95 (85/5/5).
- NOTE (honest): this lane re-read the count; it did not re-run the
  X-05 exporter. The 95 figure is disk-current per X-05FIX's fresh
  manifest parse; independent re-parse marked UNVERIFIED-this-session.

## CTQ-4 contention — VERIFIED (row 28 + adv-15 guard caveat)

- Register mapping: M-CTQ-01's registered divergence = claims row 28
  (`11-measured-claims.md:52`): 14.71 / 19.03 / 124.06 ns/frame
  fastest/median/slowest @1M under combined bw6+cpu9 contention,
  slowest = 12.2x row 5; >50 ns bar CROSSED. MEASURED (registered
  divergence).
- adv-15 guard blind-spot
  (`../redteam/corrosion/adv-15-claims-boundary-injection.md:38-66`,
  re-read): both evict legs ran GUARD-GREEN while degrading fastest
  +10.4% / +18.2% past the row-5 tolerance band — the rc11 guard
  certifies only "not contended in the two guarded channels" (2-of-3
  channels), NOT "quiet host"; evict-class common-mode degradation is
  structurally invisible to it.
- 80-150 ns band: NO on-disk charter or register states an "80-150 ns"
  contention band (grep zero matches this session). If such a band is
  quoted in dispatch prose, it is an ASSESSMENT, not a measured row;
  the measured contention figures are row 28's, and adv-15's 11.26-19.21
  ns/f evict legs. No 80-150 figure exists in any receipt — never
  quote it as MEASURED.
- Verdict: contention behavior is row 28 (registered divergence) with
  adv-15's guard-scope caveat; nothing in the charter admits a
  contention figure outside those receipts.

## CTQ-5 SRAM fit — VERIFIED (gate + mutant flip run this session)

- Register mapping: M-CTQ-03 = claims row 38 (`11-measured-claims.md:166`):
  36,048 B vs 131,072 B hard bound, headroom 95,024 B, gated slice
  (blake3+branchless freestanding caller, riscv32imc); bn3 complement:
  typed refusal BN3_ENCLAVE_REFUSED at 6,520,984 B = 49.75x ceiling.
- This session: `python3 scripts/verify_cscb_sram.py` exit 0
  ("measured SRAM ... 36048 B"; gate line: 36048 <= 131072, headroom
  95024); `--mutate=sram-bloat` exit 0 flip verdict ("mutant measured
  136068 B > 131072 B — overflow flips"). Non-conflation preserved:
  39,184 B baseline stays REPORT-ONLY (gate prints it as such).
- Verdict: ALIVE — gate + anti-vacuity flip both executed this
  session against the exact gated subject.

## CTQ-6 FFI boundary — VERIFIED (courts run this session)

- Register mapping: M-CTQ-05/M-CTQ-06 family; courts:
  `crates/cgap-ffi/tests/stats_block_court.rs` (5 tests, exit 0) and
  `crates/cgap-ffi/tests/c_header_abi_court.rs` (4 tests, exit 0,
  incl. `c_compile_leg_flips_on_field_reorder` — the anti-vacuity
  mutation: header field reorder flips the LP64 layout scorer).
- 12-symbol census: the FFI C-ABI surface is 12/12 symbols as of
  XC-01 (`redteam/corrosion/adv-02-ffi-abi-corruption.md:11`;
  `lanes/lane-r13-docfix-ops-register.md:32`); corroborated by
  lane-fs11 (`lanes/lane-fs11-changelog-crosscheck.md:81`: "12-symbol
  and 95-court cites check out"). The census figure is receipt-cited,
  re-read at use time; the c_header court's parsed-declarator layout
  court ran green this session.
- Verdict: ALIVE — both FFI courts executed this session, exit 0;
  ABI layout + stats-block seal/refuse behavior confirmed.

## W-R commands + exits (this session, 2026-11-02)

| # | Command | Exit | Result |
|---|---------|------|--------|
| 1 | cargo test -p cgap-ffi --test stats_block_court --test c_header_abi_court | 0 | 5+4 pass |
| 2 | cargo test -p cgap-git --test quota_alloc_purity_court
     --test posh_alloc_sentry_court | 0 | 1+1 pass |
| 3 | python3 scripts/verify_cscb_sram.py | 0 | 36,048 B <= 131,072 B |
| 4 | python3 scripts/verify_cscb_sram.py --mutate=sram-bloat | 0 | mutant 136,068 B flips |
| 5 | grep rows 5/8/28/38, 11-measured-claims.md | 0 | figures re-read from disk |
| 6 | grep 91\|95, 07-court-corpus.md | 0 | 91 superseded; live 95 (85/5/5) |
| 7 | grep 80-150 band, v26.11.2/ | 1 | zero matches (band is an assessment) |
| 8 | re-reads: ctq-matrix, charters, adv-15, red-09, X-05 | 0 | cites re-read from disk |

## Standing (W7)

- CTQ-2, CTQ-5, CTQ-6: ALIVE (commands 1-4, exit 0, this session).
- CTQ-1, CTQ-3, CTQ-4: VERIFIED by receipt (rows 5/8, 95-count,
  row 28 + adv-15), figures re-read from disk this session; the divan
  legs and X-05 exporter re-runs are UNVERIFIED-this-session and
  marked as such — never rounded up.
- Divergences recorded: charter CTQ numbering (see the numbering
  section); charter "91" stale (95 governs); no charter §3 exists
  ("VOC to CTQ Flowdown" is the 6th section); 80-150 ns band absent
  from every on-disk file.
- Shared tree untouched except this one owned file; no git commands.

## See Also

`../dflss/02-voc-ctq-matrix.md` · `../11-measured-claims.md` ·
`../07-court-corpus.md` ·
`../redteam/corrosion/adv-15-claims-boundary-injection.md` ·
`../redteam/red-09-alloc-sentry-breach.md` ·
`../lanes/lane-x05-court-corpus-audit.md` ·
`../lanes/lane-x-05fix-court-count.md`
