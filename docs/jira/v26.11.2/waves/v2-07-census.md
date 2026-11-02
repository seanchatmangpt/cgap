# V2-7 wind-down census — docs/jira/v26.11.1/waves/

Lane: V2-7 WIND-DOWN-CENSUS (court-auditor, read-only except this marker).
Repo: /Users/sac/cgap, branch main. Sole write: this file. No git writes,
no publish, no mocks, no Process.sleep.

## 1. Census vs S-7 baseline (f3-20-terminus v2 census: 87 files)

- `ls /Users/sac/cgap/docs/jira/v26.11.1/waves/ | wc -l` -> 152 (exit 0).
- Delta: +65 files, enumerated: a5-sram, a6-flip-rerun,
  a7-phase-change-remeasure, a8-standing-notes, assess-{beam,gpu,
  integration,wasm-footprint}, bench-chains-{ingress,parquet,bn3,seal,
  sovereign}, cd-1-scaffold, cgap-datafusion-removal, cli-{1..5},
  dia-{1..6}, doc-{1..6}, errc-{eval-ad,eval-el,eval-mr,eval-sz,
  f-diataxis-pack,backlog}, paper-{audit,comparison-audit,dred,
  rewitness,sip}, p3-pub-{bont-bosym,core-storage,bn3-gate,beam-gpu,
  integration-wasm,court-audit,guard-flip,aux-final,crypto-p28,
  fibo-cloud}, p3-recon-manifests, p3-api-verify-all, r2-xip-noalloc,
  rdf-10-verification, research-{duckdb-arrow,feather-applicability,
  feather-format,motherduck}, umb-1-readme, vac-bench-{bn3-chain,
  dead-surface}. 4+4+5+1+1+5+6+6+6+5+13+1+1+4+1+2 = 65; 87+65 = 152.

## 2. Classification (152 files, each in exactly one bucket)

### landed-ALIVE (139)
- F1 (45): a01-a09, a11-a18, a20 finish x18; a01-wave9, a02-wave8,
  a03-wave5, a07-wave5, a07-wave6, a08-wave7, a09-wave9-backfill,
  a11-wave4, a12-wave8, a13-wave9, a15-wave8, a16-wave6, a17-wave2,
  a17-wave2-re-audit, a18-wave3, a18-wave9, a20-wave3, a20-wave5,
  a20-wave9 audits x19; a5-sram, a6-flip-rerun, a7-phase-change-
  remeasure, a8-standing-notes; assess-{beam,gpu,integration,
  wasm-footprint}.
- F2 (20): bench-{bn3,ingress,parquet,seal} x4; bench-chains x5;
  f2-{7,8,9,10,15} x5 (f2-15 FLIPPED-4/4); vac-bench x2; r12, r13, r14;
  r2-xip-noalloc (ALIVE, 119/0, 8 in-module courts).
- F3+gates (25): f3-01..f3-08 x8; f3-10..f3-16 x7 (landed ALIVE witness
  appends supersede their stale BLOCKED markers); f3-publish-ledger
  (BLOCKED header superseded by coord-publish-record + registry);
  coord-publish-record; dod-five-gates; gate-{1,3,4};
  v26111-campaign-ledger; r2-oabi-prod, r2-wire-parity, r10-gate3.
- Down-wind (49): gap-{1,6,7,8}; challenger-sales-report;
  cgap-datafusion-removal; cd-1-scaffold (green at its own state;
  superseded by the datafusion removal); cli-{1..5} (cli-5
  handler-green ALIVE); research x4; rdf-10-verification; paper x5;
  dia x6; doc x6; umb-1-readme; errc-{eval-ad,eval-el,eval-mr,
  f-diataxis-pack,backlog} x5; p3-recon-manifests; p3-pub-{bont-bosym,
  core-storage, bn3-gate, beam-gpu, court-audit, guard-flip, crypto-p28,
  ffi} x8.
- Total: 45+20+25+49 = 139.

### landed-PARTIAL (11)
a10-finish (PROPOSED until T5, BLOCKED-on-A19); a19-finish (T1 size
budget BLOCKED 264,881 B); errc-eval-sz (PARTIAL_ALIVE);
p3-api-verify-all (13/23 PASS, 10 FAIL); gate-5-silicon (size law ALIVE;
SRAM term UNVERIFIED, xtensa loud SKIP); W5-baselines-hardening-prd-ard
(charter UNVERIFIED until wave-5 execution); f3-09 (REFUSED authority
edge — publish later done by coordinator); f3-20-terminus (v4
MIXED-TERMINAL); p3-pub-aux-final (wasm BLOCKED(rate-limit) at write);
p3-pub-integration-wasm (both crates SKIP, structural);
p3-pub-fibo-cloud (fibo leg BLOCKED on unpublished core dev-dep at
write). 11.

### in-flight-at-crash (2)
f3-17-hygiene and f3-18-warden: window-3 appends ABSENT at the v4
deadline 04:51:03Z (f3-18's file changed in-window without the window-3
marker) — legs in flight at wave close, named loudly by f3-20 v4.

### superseded (0 as file bucket; superseded ROWS inside landed files)
f3-09..f3-15 stale BLOCKED/REFUSED sections (registry-superseded);
f3-20 v1 TERMINAL-LOUD (superseded by v2/v4 appends); cd-1-scaffold
(receipt retained, spike removed by cgap-datafusion-removal.md);
r7/r8/r9 in-file sections (superseded by W2-6/W2-7 appends);
f3-20-v4 storage BLOCKED row (superseded by live registry 05:03:46Z).

## 3. Cross-checks (first-hand this session)

### cgap-datafusion-removal.md
Read full. REMOVED per operator order; verified in-tree: no
crates/cgap-datafusion, no member line, zero references; CD-1 superseded.
CONSISTENT.

### cli-5-verify.md handler-green section
Read full. Build 0; cargo test 25/0; 17 new verbs exit 0 each with real
kernel output; frobnicate refuses (1); regen replay byte-identical.
Standing: ALIVE (handler-green). CONSISTENT.

### f3-20-terminus.md v4 MIXED-TERMINAL rows (live re-verify, UA
### cgap-publish-receipt)
- cgap-storage: PUBLISHED-26.11.1 num=1, updated 2026-11-02T05:03:46Z.
  v4's BLOCKED row STALE — published minutes after the 04:55Z terminus.
- wasm-footprint: PUBLISHED-26.11.1 num=1, 04:43:42Z — matches v4.
- cgap-ffi: PUBLISHED-26.11.1 num=1, 04:53:44Z — matches v4.
- cgap-integration: HTTP 404 — still unpublished (v4 open row holds),
  but its structural blocker CLEARED: cgap-audit PUBLISHED 26.11.1
  num=1 at 03:53:39Z (live).
- bench: max 1.1.0 (2021 squatter) — v4 name-collision CONFIRMED.
- cgap-beam, cgap-court, cgap-cli: HTTP 404 x3 — open rows CONFIRMED.
- cgap-fibo: PUBLISHED-26.11.1 num=1, 05:13:37Z; cgap-cloud: PUBLISHED
  26.11.1 num=1, 03:43:38Z — both O-ABI ontology crates live.
- Net: 17/22 of the v4 table PUBLISHED (storage flipped post-terminus)
  + fibo + cloud = 19 cgap-family crates registry-verified at 26.11.1.

## 4. Commands + exits (this lane)

- ls waves/ ; ls | wc -l -> 152 (exit 0)
- standing-line sweep over all 152 receipts (grep, exit 0)
- full reads: cgap-datafusion-removal.md, cli-5-verify.md,
  f3-20-terminus.md, p3-pub-aux-final.md, p3-pub-fibo-cloud.md,
  p3-pub-integration-wasm.md (exit 0 each)
- python3 urllib crates.io API x11 (UA cgap-publish-receipt): HTTP 200
  x6 (storage, wasm-footprint, fibo, cloud, ffi, audit) + bench 200
  (squatter 1.1.0); HTTP 404 x4 (integration, beam, court, cli) — the
  404s are the data, not failures (exit 0 harness)
- mkdir -p + chunked writes of this receipt (exit 0 x5)

## 5. Courts run

None — the census lease holds no court. Anti-vacuity: the 404 flips are
the court — a census claiming all-published is refuted by the four 404
rows; a census claiming all-open is refuted by the six 200 rows. Zero
commands would be VACUOUS; 152-file sweep + 11 live API checks executed.

## 6. Standing (lane V2-7)

ALIVE. Census executed this session; delta +65 enumerated; 152/152
classified; cross-checks first-hand. Divergences:
1. f3-20-v4's cgap-storage BLOCKED row is stale by minutes (live
   PUBLISHED 05:03:46Z); the wave's real terminal count is 17/22 (+2
   ontology crates), not 16/22.
2. p3-pub-core-storage claims the storage publish; registry corroborates
   state (num=1) but lane attribution/timestamp between the p3 lanes is
   unreconciled — state agrees, provenance ambiguous.
3. Family-level counting required two correction passes mid-census (a
   p3-pub-ffi omission, then a roster/family recount); final arithmetic
   reconciles 139+11+2 = 152.
