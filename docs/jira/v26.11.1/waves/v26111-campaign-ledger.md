# v26.11.1 Campaign Ledger (GAP-9, court-auditor draft)

- **Lane**: GAP-9 CAMPAIGN-LEDGER (court-auditor, read-only), repo `/Users/sac/cgap`,
  one canonical checkout, branch `main`, HEAD `cfd085a9` at draft.
- **Date**: 2026-11-01, draft window 16:43-17:05 PST (2026-11-02T00:43-01:05Z).
- **Method**: every landed receipt under `docs/jira/v26.11.1/waves/` read in
  full; claims cite file:line as read. Audited state is AS-WITNESSED AT DRAFT
  TIME: the fan-out was live while this ledger was drafted (gate-3/gate-4 and
  f2-15 landed mid-audit), so absent receipts are OPEN legs, never guesses.
- **Contract**: single write is this file; no git writes; no source edits.

## 1. Wave F1 - lane standing (A01-A20)

18 ALIVE / 2 PARTIAL across the 20 lanes. Per lane, from the finish receipts:

- **A01 cgap-arrow - ALIVE.** host 177/0 + riscv32imc exit 0 + nm 0 alloc/0 panic + 3 flips
  (a01-finish-receipt.md:14-23, 58-101); C6a smoke binary UNVERIFIED (a01:141-149).
- **A02 cgap-parquet - ALIVE.** 216 default + 236 std tests exit 0; C5 release 516.8us vs 2000us
  TARGET MET; MUT-FOOT-4 flip 101 (a02:74-93, 157-166).
- **A03 cgap-git - ALIVE.** 88/0 post-edit; SHIM-OFFSET / SHIM-STABILITY / SHIM-PARENT all flipped
  101, restores byte-exact (a03:64, 78-86).
- **A04 cgap-core - ALIVE.** 75/0 across 11 suites; FRAME_HEADER_LEN 128->129 flip = E0080 compile
  abort (a04:22-26, 33-45).
- **A05 cgap-storage - ALIVE.** 21 tests exit 0; M-FENCE + M2-SRC flips 101 (a05:38-55);
  CRC-identity flips are wave-6 evidence, not re-run (a05:119-124).
- **A06 cgap-bont - ALIVE.** 35/35 post-T6; M2 tag-drop flip 101; b3sum cross-process anchor
  (a06:22-27, 62-71).
- **A07 cgap-bosym - ALIVE.** 14/14; zero-string gate 1.4.1 + self-check PASS; Pyright 0/0/0;
  flags shim flip 101 (a07:64-98, 109-120).
- **A08 shacl-bitmask - ALIVE.** 35/35 twice; compiler-level M1c/M3c flips 101, sha-fail-closed
  restores (a08:29-34, 79-92).
- **A09 fibo O-ABI - ALIVE.** 21/21; byte-stable re-derivation across a fresh emit; M-FLIP exit
  101 + emit --check exit 4 (a09:41-48, 84-98); DoD-3 distribution UNVERIFIED (a09:122-125).
- **A10 cloud O-ABI - PARTIAL.** T1-T4 + raw consumer ALIVE (9/9); T5 BLOCKED on A19; lane
  standing PROPOSED per doc (a10:42, 96-110).
- **A11 bn3-arena - ALIVE.** 111/111; battery 2/2 flips; capacity-bound courts green (a11:10-13,
  66-90).
- **A12 simd-unifier - ALIVE.** final gate 111/111; SHIM-DOE4 flip 101 (4 courts); BM0 mispredict
  gate BLOCKED/TARGET, no perf counter (a12:66-77, 128-133).
- **A13 bn3-builtins - ALIVE.** 102/102 + builtins 26/26; asm census 0 blr / 0 br xN / 0 brk; M-GT
  flip 101 (a13:18-25, 37-64, 126).
- **A14 dam-penstock - ALIVE.** 11/11 both profiles; shim-1 flip 101; sec8.4 TARGET rows GAP (7.52
  ns, 3.91x), advisory (a14:24-41, 53-61, 110-113).
- **A15 powl-ocel2 - ALIVE.** 37/37 incl. two 1M-row stress courts; M-D3-R flip isolated to
  exactly the D3 court (a15:24-29, 65-87).
- **A16 chatman-pqc - ALIVE.** 24/24; 9/9 mutation classes flip, zero vacuous; PQC open by design
  UNSUPPORTED (a16:9-12, 82-110).
- **A17 p28-derivative - ALIVE.** 42 total (36+6) exit 0; 1280-mutation sweep, 1258 refused;
  sandbox neuter flip 101 (a17:11-13, 37-56).
- **A18 asc450 - ALIVE.** 36/36; F1/F2 courts landed; off-by-one flip 101 (7 failed); float sweep
  clean (a18:63-91, 100-101).
- **A19 libcgap-ffi-xip - PARTIAL.** DoD ladder PARTIAL/BLOCKED rows: producers 3/5, four
  sovereign bodies BLOCKED (owner lanes), T1 size RED 264,881 vs 65,536 (a19:87-93, 151-177);
  sec.11 re-verify deferred bodies to F2-5/6 (a19:253-259). Post-receipt: F2-5/6 landed exactly
  one body, cgap_verify_p28 (sovereign.rs:290-292, auditor grep); GAP-1 in flight for
  admit/eval/commit + the T1 repair (gate-3:28-41).
- **A20 court-verdict - ALIVE.** F1 pass PARTIAL-ALIVE superseded by the F2-2 addendum: full sec.9
  DoD first-hand, kill_rate 1.0, meta-flip 101 (a20:131-139, 189-192).

Notes: A10's receipt words T5 as BLOCKED on A19 (not UNSUPPORTED); lane standing
stays PROPOSED until T5 unblocks (a10:97-108). A19's BLOCKED rows are
owner-lane scoped; the lane count above is 18 ALIVE + A10/A19 PARTIAL.

## 2. Wave F2 - 15 items

On-disk F2 receipts at draft: F2-1 (a19 sec.11), F2-2 (a20 addendum), f2-7,
f2-8, f2-9, f2-10, f2-11 (bench-ingress), f2-12 (bench-parquet), f2-13
(bench-bn3), f2-14 (bench-seal), f2-15 (bench-flip marker). Items 3/4: no
receipt marker located under waves/ at draft -> OPEN (section 6). Items 5/6:
no marker, but landed in-code - F2-6 is the .oabi seam decision record
(oabi_lookup.rs:1, "Lane F2-6 (SEAM-OABI)"); F2-5 wired the sovereign verify
body (sovereign.rs:290-292). Receipts for 5/6 remain owed.

- F2-1 A19 re-verify: ALIVE - check/build/nm/gate3 legs exit 0; four sovereign symbols 0/4 as-of
  then; T1 size BLOCKED carried as-of-date (a19:228-270).
- F2-2 A20 close-out: ALIVE - 13/13, kill_rate 1.0000, ledger row-for-row match, VACUOUS-GATE
  negative control exit 1 (a20:154-162).
- F2-7 doc-standing: ALIVE - five lane-doc status lines reconciled, diffs cited (f2-7:21-48);
  stale-but-out-of-scope prose inventoried (f2-7:51-73).
- F2-8 projection baseline: ALIVE - projection_coverage.py exit 0, 12/12 replayed byte-identical,
  baseline 0.4386% of tracked bytes regenerable, self-check flip OK (f2-8:14-19, 50-63).
- F2-9 hop ratio: ALIVE - instrument + court legs; steam baseline machinery share 0.0 for both
  historical runs, disclosed as a reconstructed contract-hops floor (f2-9:35-36, 62-72).
- F2-10 bench membrane: ALIVE - divan =0.1.21 pin, five [[bench]] targets linked, purity courts
  flip (f2-10:18-36, 38-52).
- F2-11..14 benches: all four lanes ALIVE; MEASURED table below.
- F2-15 bench flips: FLIPPED 4/4, zero VACUOUS (f2-15:22-23). PROVENANCE CAVEAT: the marker is a
  GAP-6 transcription of the F2-15 audit's final message, not a first-hand re-run; GAP-6 states it
  held no re-measurement (f2-15:6-9, 35-39). Rows 1-2 corroborated by landed receipts: M-F2-11-1
  exit 101 (bench-ingress:89-91) and the oracle flip exit 101 (bench-parquet:109-116).

### Bench MEASURED table (verbatim from the four bench receipts)

- dam penstock admission 1M / 10M: MEASURED 5.79 / 5.70 ns/frame - GAP vs < 4.0 ns (1.45x / 1.42x)
  (bench-ingress:58-73)
- cgap_admit_frame (A04) 1k/64k/1M: MEASURED 1.99 / 3.64 / 5.18 ns/frame - info rows; the 1M row
  is GAP (bench-ingress:62-64)
- parquet rle_decode_direct: MEASURED 0 allocs / 0 B, every stat column - zero-intermediate-heap
  MET, TARGET->MEASURED (bench-parquet:90-96)
- parquet rle_decode_staged: MEASURED 1 alloc / 262.1 KB per iter - direct-vs-staged ~+27% time
  (63.28 vs 49.95 us) (bench-parquet:98-99)
- bn3 statement_scan_24b 64k / 1M: MEASURED 82.24 / 78.94 GB/s - MET vs > 1.0 GB/s
  (bench-bn3:67-70)
- bn3 unify_join_2atom_pairs: MEASURED 6.499 ns/pair (64k: 94.36) - MET vs sub-us/pair
  (bench-bn3:71-72)
- bn3 builtin dispatch: MEASURED 3.115845 ns/eval - datapoint, no stated target (bench-bn3:73)
- p28 attestation verify: MEASURED ~460 ns/record (64k: ~459) - MEASURED per A17 charter row
  (bench-seal:62-65)
- crypto seal/verify receipt: MEASURED ~835 / ~837 ns - MEASURED per A16 sec.6 (bench-seal:66-67)
- ffi boundary pair (Rust-side): MEASURED ~137 / ~138 ns per pair - direct C-ABI leg
  [SKIP]-pending, banner re-probes each run (bench-seal:68, 76-88)

Phase-change instrument baselines (both new this wave): projection coverage
0.4386% (f2-8:19) and machinery share 0.0 (f2-9:35-36, 68-72, floor reading
with the reconstruction disclosure).

## 3. Wave F3 - state as witnessed at draft

Bounded poll executed first per dispatch (30s cadence, 10-min deadline, early
exit on a stable set; stable at ~90s). Census at 16:49:30 PST: 14 of 20 legs
landed - 01,02,03,04,05,06,07,08,09,10,11,13,16,20. ABSENT (OPEN): f3-12,
f3-14, f3-15, f3-17, f3-18, f3-19. Supersedes f3-20's own census (7 of 19 at
its terminus read, f3-20:97-100): f3-03/09/10/11/13/16 + gate-3/gate-4/f2-15
landed after it. Per leg:

- 01 version-lock - ALIVE: five crates resolve 26.11.1, pins byte-identical (f3-01:9-13, 38).
- 02 cli-regen - ALIVE: regen byte-stable apart from version; cargo test 19/0 (f3-02:21-32).
- 03 workspace-green - BLOCKED: cargo test --workspace exit 101; sole failure
  xip_t1_riscv_staticlib_size_court, 264,881 >= 65,536 B (4.04x); "Publishers must NOT proceed
  past F3-03" (f3-03:5, 15, 25-29).
- 04 courts-green - GREEN: court_verdict 13/0; kill_rate 1.0; shim kill + VACUOUS-GATE inert
  refusal exit 1; self-check PASS (f3-04:15-19).
- 05 auth - ALIVE (presence-level): credentials.toml present; dry-run publish 0; token validity
  UNVERIFIED by lane design (f3-05:35-43).
- 06 pins - ALIVE: zero undeclared duplicate authority pins; sole by-value pin footer.rs:84
  ATTESTATION_LEN=80, self-declared framing-only (f3-06:3-14).
- 07 package-audit - PASS: no contamination; flagged: bench_tmp.rs ships in cgap-blake3
  (f3-07:26-41). Internal count conflict: "one flagged row" (:26) vs "two flagged rows" (:61).
- 08 registry-diff - CLEAR: all five at 26.10.29, no immutable-version collision, ok-to-publish
  YES x5 (f3-08:10-30).
- 09 pub-blake3 - BLOCKED: publish NOT executed; its f3-03 gate never went green inside the
  00:39:13Z poll expiry (f3-03 landed 00:43Z); one-attempt budget intact (f3-09:9-18, 52-60).
- 10 pub-arrow - BLOCKED: chain law from f3-09; publish never invoked (f3-10:10-16, 40-44).
- 11 pub-git - BLOCKED: chain law; the f3-10 poll timed out, then f3-10 landed BLOCKED
  (f3-11:10-16, 59-66).
- 12 pub-parquet - OPEN: no marker at draft.
- 14/15/17/18/19 - OPEN: no markers at draft (verify / ledger / warden legs).
- 16 replay - BLOCKED: nothing published to replay; mechanics finding: dry-run aborts at upload,
  so it can never witness already-exists (f3-16:9-17, 49-54).
- 20 terminus - TERMINAL-LOUD: all five BLOCKED at its 00:35:10Z API stamp; census superseded as
  noted (f3-20:18-36, 59-72).

Auditor registry stamp (first-hand, draft window): live crates.io API rows -
all five crates max_version 26.10.29, num_versions 1; 26.11.1 on zero of five
slots. The BLOCKED publish coordinates are grounded, not transcribed.

## 4. The five DoD gates

Gate definitions: gate-runner-playbook.md:48 (GATE-1 Workspace Purity), :78
(GATE-2 Binary/Symbol Purity), :123 (GATE-3 1M-Row Sovereign Ledger Proof),
:172 (GATE-4 640-Bit Spoliation & Liability Metering), :213 (GATE-5 Bare-Metal
Silicon Fit).

- GATE-1: build leg exit 0 (f3-03:14); --deps E=0 x15 (a20:37, 157); register v2 green (a20:37;
  a07:66-74). Workspace TEST leg exit 101 on the T1 size court (f3-03:15-29). Net: RED/BLOCKED at
  f3-03, coordinates sovereign.rs:2399 (flash assert, 264,881 vs 65,536 B).
- GATE-2: all legs green - 13/0 court_verdict, kill_rate 1.0, shim kill exit 0, inert negative
  control exit 1, self-check PASS (f3-04:15-19); --strings 0 hits, cross-gate-court +
  interference-court exit 0 (a20:37-42). GREEN as-witnessed; zero_string gate trio green at 1.4.1
  (a07:64-78).
- GATE-3: PARTIAL - stages ALIVE (37/0 audit, 11/0 gate, a19_seal court, 111/0 bn3, zero-alloc
  self-courts) but composition is 3-of-4: CgapEvalBn3Fn is declared and UNIMPLEMENTED; the commit
  body is also absent (gate-3:8-41). Close condition: GAP-1 lands eval+commit bodies, then a
  4-of-4 re-witness (gate-3:32-33).
- GATE-4: ALIVE - 42/0; 640-bit sweep 640 executed / 629 refused / 11 tolerated per record == the
  predicted D-A17-1 class; three vacuity probes distinguishable; 2560 mutations / 2516 real
  SIGABRT deaths; boundary at the real 5e9 ceiling; _cgap_verify_p28 in the fresh cdylib
  (gate-4:1-23).
- GATE-5: OPEN - gate-5-silicon.md ABSENT at draft (auditor ls; gate-3/gate-4 landed mid-audit,
  gate-5 did not). Nearest landed evidence, as-of-date: A19 T1 measured RED (a19:88-93, 160), C6a
  smoke binary UNVERIFIED (a01:141-149); bench receipts defer silicon fit to llvm-size
  (bench-seal:108-109).

## 5. Registered divergences

1. P28 twin in cgap-core (outside the five-crate publish set): frame.rs:44 ATTESTATION_LEN: usize
  = 80, const-pinned to the layout at frame.rs:390; runtime-tied and court-enforced (f3-06:9-13,
  18-19).
2. oabi-lookup UNSUPPORTED(scope): crates/cgap-ffi/src/oabi_lookup.rs:1-31 is a typed decision
  record per the A19 four-verb surface law (A19:47, 157-166 cited in-file); A10 T5's blocker stays
  BLOCKED-on-A19 (oabi_lookup.rs:36-40).
3. Quad64 double-def in cgap-arrow: layout.rs:36 and transpose.rs:26 both define #[repr(C,
  align(64))] struct Quad64 with the identical field lattice; transpose.rs:3-7 self-documents the
  self-contained-by-design note that predates the layout lane's landing - collapse is the recorded
  coordination point (auditor grep: exactly two struct Quad64 definitions repo-wide).
4. cgap-blake3/tests/bench_tmp.rs ships in the package (f3-07:38-41; auditor ls: 564 B, mtime Oct
  31 22:40). Real content, scratch-flavored name; rename recommended in a later wave.
5. _cgap_p28_invariant_check header question: zero occurrences repo-wide (auditor grep over
  crates/, include/, tests/ this session); include/cgap.h carries the four frozen prototypes only
  (cgap.h:164 = cgap_verify_p28). GAP-1 in flight is resolving the question; nothing on disk to
  cite yet.

Also carried (recorded by receipts, not re-adjudicated): A01 C6a smoke binary
UNVERIFIED (a01:145-149); A12 BM0 mispredict TARGET/BLOCKED (a12:129-133); A14
sec.8.4 TARGET rows GAP 7.52 ns / 3.91x, advisory only (a14:54-61); A09 DoD-3
pinned-distribution UNVERIFIED:no-distribution-on-disk (a09:122-125); A13
lane-doc sec.5.1 carries a SHA-256 literal where BLAKE3 is meant (a13:134-138);
A16 FFI feed seam unwired as of its receipt (a16:87-89).

## 6. BLOCKED / OPEN set (coordinates)

1. F3-03 gate RED - crates/cgap-ffi/src/sovereign.rs:2399: T1 FLASH 264,881 B >= 65,536 B budget
  (4.04x), deterministic (f3-03:25-29). Blocks the whole publish chain: f3-09/10/11/13 all BLOCKED
  chain law, zero publish attempts, one-attempt immutability budgets intact (f3-09:52-60,
  f3-10:40-44, f3-11:62-66, f3-13:24). Registry at draft: 26.10.29 x5 (auditor curl stamp).
2. GAP-1 (in flight) - sovereign bodies cgap_admit_frame / cgap_eval_bn3 / cgap_commit_parquet
  (gate-3:28-41); the T1 size repair; the _cgap_p28_invariant_check header question. After it
  lands: Gate-3 4-of-4 re-witness (gate-3:32-33). Landed so far: cgap_verify_p28 only
  (sovereign.rs:290-292).
3. GATE-5 OPEN - no gate-5-silicon.md at draft; A19 T1 red + the C6a smoke binary UNVERIFIED are
  the as-of-date evidence (a19:160, a01:141-149).
4. F3 legs 12/14/15/17/18/19 OPEN - no markers at draft; the publish legs
  blake3->arrow->git->parquet->cgap are unwitnessed; f3-16's mechanics finding means publication
  must be witnessed by live API rows, never a dry-run conflict (f3-16:49-54, 119-121).
5. F2 items 3/4 OPEN - no receipt markers located; F2-5/6 landed in-code (oabi_lookup.rs:1;
  sovereign.rs:290-292) but their receipt markers are owed.
6. A10 T5 BLOCKED on A19 (a10:97-108); flip path = post-hoc execution after the owner lanes land a
  boundary surface naming slot resolution.
6. A09 DoD-3 distribution UNVERIFIED (a09:122-125); admission is coordinator scope.
7. Cleanup debts (coordinator): permission-refused /tmp debris named by the receipts -
  /tmp/a20_sandbox 3.6 GB (a20:101-106), /tmp/cgap-lane-A19-target (a19:207-212),
  /tmp/cgap-lane-A04-target + a04 sandbox (a04:84-91), /tmp/cgap-lane-A08-target (a08:158-162),
  /tmp/a17-vac* (a17:113-114), /tmp/hop-court-f29 (f2-9:93-94).

## 7. Auditor command ledger (this session, exits mine)

```text
ls waves/ + lanes/ -> 0 (initial census 16:43)
find v26.11.1 -name '*gate*' -> 0 (gate ledgers absent at 16:43)
bounded poll f3-* 30s/10min -> stable ~90s, 14 legs landed (16:49:30)
grep Quad64 repo-wide -> layout.rs:36 + transpose.rs:26 (double-def found)
grep _cgap_p28_invariant_check -> 0 hits repo-wide
grep ATTESTATION_LEN frame.rs -> :44 pin + :390 const assert
sed -n 39p Cargo.toml -> version = "26.11.1" (re-verified f3-01)
cargo pkgid -p cgap-blake3 -> #26.11.1, exit 0 (re-verified f3-09 leg)
ls cgap-blake3/tests/bench_tmp.rs -> present, 564 B (re-verified f3-07 flag)
sed -n 84p footer.rs -> ATTESTATION_LEN = 80 (re-verified f3-06 pin)
ls crates/bench/benches/ -> bn3/ingress/parquet/seal/kernels present
grep no_mangle + fn cgap_* in cgap-ffi -> verify_p28 only (:290-292)
curl crates.io x5, UA cgap-publish-receipt -> max_version 26.10.29 x5
```

No court flips were executed by this lane: GAP-9 is the aggregation audit;
every flip row above cites its exit-101 / compile-abort witness in the source
receipt. Zero executed commands would have been VACUOUS; this ledger ran 13.

## 8. Standing

Draft-time campaign standing: F1 18 ALIVE / 2 PARTIAL; F2 benches MEASURED with flips 4/4
  (transcribed); F3 hygiene legs green through 08 with the publish transition BLOCKED at a single
  coordinate; GATE-2 GREEN, GATE-4 ALIVE, GATE-1 RED, GATE-3 PARTIAL, GATE-5 OPEN. The campaign's
  one blocking coordinate is sovereign.rs:2399; unblocking it (GAP-1) flips f3-03, which unblocks
  the publish chain and the Gate-3 composition re-witness.

## 9. Refresh (post-publish) — A-2 campaign-ledger-refresh (court-auditor)

- **Lane**: A-2 CAMPAIGN-LEDGER-REFRESH (court-auditor, read-only except this
  append), repo `/Users/sac/cgap`, branch `main`, HEAD `cfd085a9` (unchanged;
  no git writes). As-of: 2026-11-02T03:39Z. Prior draft (sections 1-8)
  preserved byte-exact above; pre-append sha256
  `1c92d2f4b97e9dea636ab20335bb3d068322ef2fe7e7f90817f89b96ae4a24ab`.
- **Method**: every superseding receipt re-read in full this session; live
  registry re-swept first-hand; on-disk pins re-verified. As-of-date
  conflicts with the old draft are named inline and collected in 9.7.

### 9.1 Per-section updated standing (as-of 2026-11-02T03:39Z)

- **Sec.1 F1 (A01-A20)**: 18 ALIVE / 2 PARTIAL unchanged. A19's PARTIAL is
  narrowed: all four sovereign bodies landed (gap-1-sovereign.md — admit/
  eval/commit ALIVE + `nm -gU` 4x T); A10 T5's blocker remains BLOCKED-on-A19
  (r2-wire-parity.md "What remains").
- **Sec.2 F2**: unchanged except as noted; F2-15's transcription caveat
  stands. Phase-change baselines re-stated in 9.4.
- **Sec.3 F3**: superseded terminus-wide by f3-20-terminus.md v2
  (TERMINAL-PUBLISHED, 02:47:51Z registry truth) + coord-publish-record.md.
  f3-15's two FAIL sweeps (00:20/00:49Z, 01:22/01:59Z) are historical;
  its W2-7 section (02:23:18Z) is PASS x5.
- **Sec.4 gates**: G1 GREEN, G2 GREEN, G3 ALIVE 4-of-4 (r10-gate3.md: new
  `a19_seal_1m_eval_court`, 14/0 suite exit 0, eval-neuter sandbox flip
  exit 101, shared-tree sha invariant), G4 ALIVE, G5 ALIVE-with-recorded-
  divergence (dod-five-gates.md aggregate; gate-5-silicon.md R11 section).
- **Sec.5 divergences**: #2 (oabi-lookup UNSUPPORTED) stands; #3 Quad64
  double-def — r13-quad64.md exists (landed after draft; not re-adjudicated
  here); #4 bench_tmp.rs — r12-bench-tmp.md landed (purge receipted);
  #5 invariant-check DECIDED UNSUPPORTED(scope), header unchanged
  (gap-1-sovereign.md) — closed as a decision.
- **Sec.6 open set**: corrected in 9.5. **Sec.7/8**: draft-time snapshots,
  superseded by this section.

### 9.2 Measured table additions (chain PARETOs; receipts verbatim)

- chains-ingress (bench-chains-ingress.md, exit 0): 1M-frame chain mean
  102.7 ms (10.2 Mframe/s); **digest fold ~85% of segment cost / ~91% of
  chain mean** — the ~80% segment; >10x lever = trigger-based or
  prefilter-escalated BLAKE3.
- chains-sovereign (bench-chains-sovereign.md, exit 0): N=4096 chain
  187.6 ms; **commit = 87.6%** of the per-transaction sovereign tax (A02
  sealed-block FileSink); LEASE boundary finding: commit accepts 65,376
  lanes, lane 65,377 typed CGAP_ERR_LEASE_EXHAUSTED (not the dispatch's
  64 KiB framing).
- chains-seal (bench-chains-seal.md, exit 0): flat 2.378 vs 2.373 us/row
  (<=0.3% drift); **three-way split** p28 19.3% / chatman seal+verify
  35.2% / asc450 metering 45.4% (derived, method stated) — no 80% stage.
- chains-bn3 (bench-chains-bn3.md, exit 0): **unify 81.2% at 65k ->
  99.9% at 1M**; lever = hash/trie join (the raw-word oracle computes the
  same match count in setup time).
- chains-parquet: bench source landed (crates/bench/benches/
  chains-parquet.rs present, exit 0 ls) but NO receipt marker
  (bench-chains-parquet.md absent, ls exit 1) — MEASURED row loud-open.
- Divan migration witness: f2-10 five [[bench]] targets + P3-16's
  five chains-* targets confirmed in crates/bench/Cargo.toml.

### 9.3 Publish transition (PUBLISHED — supersedes sec.3/6 item 1/4)

- coord-publish-record.md: all five crates PUBLISHED at 26.11.1,
  coordinator-side 2026-11-02, one invocation per crate (one-attempt law
  held; subagent attempts R2..R6 REFUSED pre-execution, zero uploads).
- This lane's live registry re-sweep 2026-11-02T03:39Z (UA
  cgap-publish-receipt, HTTP 200 x5, exit 0 x5): all five max_version
  26.11.1, num_versions 2 EXACTLY, newest 26.11.1, yanked false.
  Sec.3's "26.11.1 on zero of five slots" stamp is historical.
- Terminology: f3-20 v2 TERMINAL-PUBLISHED (registry-verified 02:47:51Z);
  W2-7 PASS x5 (f3-15:198-227); W2-6 consumer ALIVE (registry-sourced,
  f3-14 append).

### 9.4 Phase-change baselines (as-of this refresh)

- Projection coverage 0.4386% of tracked bytes (f2-8:19) — NOTE: a
  re-measure agent was dispatched and may be in flight; this figure is
  as-of-draft, superseded at use time by the re-measure receipt.
- Machinery share 0.0 (f2-9:35-36, 68-72; reconstructed contract-hops
  floor, disclosed) — no new reading landed; stands as-of-draft.

### 9.5 Corrected open set (supersedes sec.6)

1. f3-03 RED / sovereign.rs:2399 T1 flash 264,881 B — RESOLVED: T1 court
   re-anchored staticlib -> xip.ld-linked ELF subject, budgets unchanged;
   ELF FLASH 150/65,536 B, SRAM 0/32,768 B (gap-1-sovereign.md; R11 in
   gate-5-silicon.md; R17 first-hand re-run in dod-five-gates.md).
2. GAP-1 — CLOSED: three sovereign bodies landed (nm 4x T witnessed by
   R10/R11/R17); invariant-check DECIDED UNSUPPORTED(scope); Gate-3
   re-witnessed 4-of-4 ALIVE (r10-gate3.md).
3. GATE-5 — OPEN->ALIVE-with-recorded-divergence, then further: W2-10
   production bytes LANDED (r2-oabi-prod.md): production link exit 0 on
   real bytes, flash 670/65,536 (1.0%), SRAM 0, .oabi 496 B + .bn3 152 B
   real bytes at 0x42010000/0x42020000, production-only entry (no demo
   fills). Carried: sovereign.rs T1 court still carries DEMO fills
   (demo+production merge, budget-green); XIP rule EXECUTION BLOCKED-on-
   owner (A12 alloc API); xtensa T2 [SKIP] toolchain absent; SRAM
   worst-case eval-stack UNVERIFIED (no hardware/qemu).
4. Publish legs f3-09..13 — RESOLVED by coord-publish-record.md + live
   registry (this section 9.3). Publish ledger never gained a re-dispatch
   section (aggregation gap, superseded by the registry rows).
5. F2-3/4 markers remain OPEN; F2-5/6 markers remain owed (in-code).
6. A10 T5 BLOCKED-on-A19; A09 DoD-3 distribution UNVERIFIED (coordinator
   scope). A10 parity leg: W2-11 ALIVE (facade == raw == golden CEW1
   frames, 4 flip/parity courts); lane standing still PROPOSED until T5.
7. Parity lane additions: parity ALIVE (r2-wire-parity.md); production
   bytes CLOSED (r2-oabi-prod.md).
8. chains-parquet receipt missing (bench present, marker absent).
9. Cleanup debts stand as sec.6 item 7 (coordinator scope).

### 9.6 Campaign standing (refresh-time)

F1 18 ALIVE / 2 PARTIAL (A10, A19-narrowed); F2 benches MEASURED + 4 chain
PARETOs landed (chains-parquet open); five gates GREEN/GREEN/ALIVE/ALIVE/
ALIVE-with-divergence (G5 production bytes now real, 670 B flash); all
five kernel+umbrella crates PUBLISHED 26.11.1 (registry-verified
03:39Z); W2 parity ALIVE. Remaining block-set: F2 marker debts, XIP rule
execution (owner-blocked), xtensa T2, SRAM eval-stack term,
chains-parquet receipt, A10 T5, A09 DoD-3.

### 9.7 As-of-date conflicts with the old draft (explicit)

1. Sec.3/8 "BLOCKED publish chain / 26.10.29 x5" — falsified by the
   publish (coord-publish-record.md; live registry 03:39Z). The old
   auditor registry stamp was true at 00:5xZ only.
2. Sec.4 GATE-5 "gate-5-silicon.md ABSENT at draft" — the file landed
   16:54:58 before ledger mtime 16:58 (named by dod-five-gates.md
   divergence-2); superseded.
3. Sec.4 GATE-3 "PARTIAL 3-of-4" — superseded by R10 completion (4-of-4,
   eval court landed 17:47).
4. Sec.3 "f3-19 OPEN" — f3-19 IS f3-publish-ledger.md (its own header);
   no separate leg exists (f3-20 v2 census). Sec.6 item 4 superseded.
5. Sec.6 item 1 (single blocking coordinate) — resolved; the campaign's
   blocking set is now the carried list in 9.5.
6. f3-15's FAIL sweeps vs W2-7 PASS: both true as-of-their-own-timestamps;
   W2-7 (02:23:18Z) is terminal. f3-13/10/11/12 markers still read
   BLOCKED — stale-not-false (f3-20 v2 divergence-3, W2-7 divergence-1/2).
7. bench_tmp.rs flagged row (sec.5 #4): r12-bench-tmp.md purge landed
   after draft — conflict resolved in the direction of the purge.
