# 11 — AX DMADV Rollup (DFX-6, consume-only)

Date: 2026-11-02. Repo: /Users/sac/cgap (main, canonical). Lane DFX-6.
Consumes the landed DFX-1..5 docs: `02-ax-charter-ctq.md`,
`07-ax-sipoc-c4.md`, `08-ax-pfmea.md`, `09-ax-cpk-msa.md`,
`10-ax-control-plan.md` — every cell points at rows, never restates.
Predecessor rollup `00-dmadv-rollup.md` covers the substrate DFSS
campaign; this file covers the AX/container lane family only.
Receipt: `../lanes/lane-dfx-06-rollup.md`.

## Define

- Charter + register table: `02-ax-charter-ctq.md:16-28` (CTQ-01..06
  with per-cell status). Citation law and NO-GO demotion:
  `02-ax-charter-ctq.md:3-14`.
- SIPOC/C4 corrected to the actual verdict (LOCAL docker path, no
  gVisor, no cluster): `07-ax-sipoc-c4.md:14-61,63-105`.
- Scope law (CLUSTER NO-GO / LOCAL GO / ax absent):
  `10-ax-control-plan.md:9-17`, from `../lanes/lane-ax-00-gate.md` W2/W4.

## Measure

- Gate probes of record: `../lanes/lane-ax-00-gate.md` W2 (12 probes:
  ax 127, cluster exit 1 conn-refused, podman 127, docker daemon
  down->`open -a Docker`->bounded poll exit 0, go/cargo ALIVE) and W3
  (ax install NOT APPROVED; AX API = vendor-README material).
- darwin-vs-linux delta tables: PUBLISHED, two of them — latency
  `../lanes/lane-ax-05-latency.md` W4 (cgo floor 73-85 -> 154 ns/call,
  single admit 182.7-185.6 -> 306.9-313.1 ns, batch 179-181 ->
  239-448 ns/frame, timer witness 41 ns identical) and shm
  `../lanes/lane-ax-06-shm.md` W4 (mmap read +1-3%, pread -15%,
  allocs 0 both arms).
- Alloc columns: darwin steady-state 1.0 allocs/op 24 B/op (view
  mint), components 0.0 (`../lanes/lane-ipc-02-alloc-purity.md` W4);
  linux/arm64 container 6/6 courts PASS incl. 0.0 allocs/op legs
  (`../lanes/lane-ax-04-alloc.md` W3).
- PENDING columns AX-4 (charter row) left open: the 10 ns USL has no
  container column that meets it (see Analyze/Verify); the DFX-1
  divergence "lane-ipc-02 receipt absent" is RESOLVED on disk this
  session — `ls ../lanes/lane-ipc-02-alloc-purity.md` exit 0, receipt
  landed post-DFX-1 (errata-by-re-derivation, receipt below W1 #3).

## Analyze

- Top RPNs and their honest DET scores: `08-ax-pfmea.md:27-33`
  (A1 stale-binary 252, A2 vendor-claim drift / A3 dual-target-dir
  224 tie, A4 cluster-apply hearsay 200, A5 docker-daemon-down 60).
- Root causes actually witnessed this wave (each with its receipt):
  1. docker daemon down at first probe -> `open -a Docker` + bounded
     poll, up after 1 poll (AX-0 W2 #6-8) — sanctioned no-sleep
     pattern, resolution landed.
  2. podman-absent re-scope: exit 127 (AX-0 W2 #4) -> docker 28.0.4
     substituted per AX-0 W4 LOCAL verdict (AX-5/AX-6/AX-4 all name it).
  3. cluster NO-GO: exit 1 conn-refused 127.0.0.1:64378 (AX-0 W2 #3)
     -> AX-2 manifests DRAFT-UNRUN, AX-9 UNVERIFIED-by-design, zero
     apply claims anywhere in the family.
  4. vendor-claim drift class: ax.io/v1alpha1 surface asserted only as
     VENDOR-CLAIM-UNVERIFIED (AX-0 W3; `08-ax-pfmea.md` A2, DET=8
     honest — the marking is prose, no mechanical rule on disk).
  5. missing lane-ipc-02 receipt incident: DFX-1 divergence 1
     (`02-ax-charter-ctq.md:168-177`) + DFX-5 W1 #3 both recorded it
     ABSENT; the receipt has SINCE LANDED with the honest 1.0
     allocs/op membrane tax and cross-cited AX-4's SLA unblocks
     (`../lane-ipc-02-alloc-purity.md` W6) — incident RESOLVED, class
     fed back into A1/A4 detection scoring.

## Design

Component court inventory (darwin-side landed code vs
container-executed), each with its flip witness:

| Court | Where executed | Flip witness | Receipt |
|---|---|---|---|
| POSH-5 matrix 4/4 | darwin | flips arms 1-4 (magic/cyclic/stride/alloc) | lane-posh-05:28 |
| B01 C2 p99 gate | darwin | branchy mutant 6->28 ns, exit 1 | lane-b01:20-23 |
| IPC-2 alloc courts | darwin | 1.0 vs 0.0 allocs/op; flip 0x43470002 | lane-ipc-02 W3/W5 |
| IPC-3 latency courts | darwin | 41 ns timer; batch amortization | lane-ipc-03:43-47 |
| IPC-4 transport court | darwin | magic flip both ways; 0 allocs | lane-ipc-04:48-49 |
| AX-7 host legs | darwin | arms 1/2a/2b/4 PASS; arm 3 SKIP loud | lane-ax-07 W3 |
| AX-4 alloc 6/6 | container, stock images | flip both directions in-container | lane-ax-04 W4 |
| AX-5 latency + phase D | container, stock images | TestMutationGateBreach flip | lane-ax-05 W6 |
| AX-6 devshm courts | container, alpine | devshm magic flip both ways | lane-ax-06 W3 |
| AX-7 container arms | container: BLOCKED exit 3 | none in-container | lane-ax-07 W1 |
| AX-9 suspend probe | refuses pre-touch | REFUSED[PREREQ-ABSENT] exit 3 | lane-ax-09 W2 |
| AX-1 image + grep court | container build | grep court 0 / mutated 1 | lane-ax-01 W5 |
| FA10 SRAM ceiling | darwin build | sram-bloat mutant 136,068 B | lane-b04:40 |

## Verify

- Gate matrix: `10-ax-control-plan.md` AX-GATE-01..05 rows, pointed at
  not restated. Status deltas since DFX-5 wrote its "PENDING per
  AX-4/AX-7/AX-8 (no receipt on disk)" cells, re-read on disk this
  session:
  - AX-GATE-02 (alloc): AX-4 LANDED — 6/6 container courts, 0.0
    allocs/op legs, flips both directions. The PENDING cell is
    superseded for the stock-image path; the AX-1-image arm of
    alloc.sh stays UNVERIFIED (AX-4 W8).
  - AX-GATE-03 (SLA): AX-5 PUBLISHED the darwin-vs-linux delta table
    (W4); the 10 ns USL stays unmet at the membrane, re-scope still
    the only lawful control (DFX-5 OCAP stands).
  - AX-GATE-04 (anti-vacuity): AX-4/AX-5/AX-6 each witnessed a
    container flip; AX-7's 4-arm matrix container re-witness remains
    LOUD-BLOCKED (exit 3) and AX-8 (SL sealing container parity) has
    NO receipt on disk.
  - AX-GATE-05 (sealing): darwin flips (IPC-6) stand; container parity
    row stays PENDING (AX-8 absent).
- Honest OPEN list, cross-checked against the register: grep of
  `../audit/release-blockers.md` for ax-/interop/container/ipc/suspend
  returned ZERO rows — the AX-family open items are NOT registered in
  release-blockers.md and live only in lane receipts (registration
  gap, disclosed, not re-registered by this rollup — 00-dmadv-rollup
  law: register nothing new). `audit/audit-ledger.md` likewise has no
  AX/IPC rows (grep zero this session). Open items, all pointed:
  - CTQ-01 isolation court — no AX Workspace court exists
    (02-ax-charter-ctq.md:57-61 close condition).
  - CTQ-02 / AX-9 suspend/resume — probe script landed, refuses exit
    3; UNVERIFIED-by-design (02-ax-charter-ctq.md:63-77).
  - CTQ-06 container re-witness of the 4-arm matrix — AX-7 exit 3.
  - AX-GATE-05 container sealing parity — AX-8 receipt absent.
  - AX-2 manifests DRAFT-UNRUN + image digest pin owed on the landed
    cgap-ax-testbed:ax1 image (lane-ax-02-manifests.md:57-59).
  - A1/A2/A3 PFMEA controls (symbol-count court, receipt_lint rules)
    — none landed (08-ax-pfmea.md:53-66).

### Per-CTQ verdicts

| CTQ | Verdict | Basis (cited) |
|---|---|---|
| 01 isolation | ASPIRATIONAL-UNVERIFIED | charter :30-61; no court; darwin races seen |
| 02 suspend/resume | ASPIRATIONAL-UNVERIFIED | charter :63-77; AX-9 refusal exit 3 |
| 03 membrane allocs 0 B/op | DEFECT-OPEN | steady admit+eval 1.0 allocs/op FAIL (ipc-02 W4) |
| 04 latency <=10 ns | VERIFIED kernel; OPEN membrane | p99 6 ns (b01:19); membrane >18x off |
| 05 SRAM <=131,072 B | VERIFIED | 36,048 B + mutant flip (charter :131-148) |
| 06 mutation arms 4/4 | VERIFIED darwin; container OPEN | POSH-5 4/4; AX-7 container exit 3 |

Tally: 2 VERIFIED (05; 04 kernel layer), 3 DEFECT-OPEN
(03, 04-membrane, 06-container), 2 ASPIRATIONAL-UNVERIFIED (01, 02).

**Falsifier (verbatim, governing):** the AX-cgap integration Verify
passes iff every container court re-witnesses its flip in-image AND
the darwin-vs-linux delta table is published AND suspend/resume is
either witnessed or loudly UNVERIFIED — with zero cluster-apply
claims surviving AX-0's NO-GO.

Applied against the on-disk state this session:
- Clause 1 FAILS: container courts re-witnessed flips in STOCK images
  (rust:1-bookworm / golang:1.23-bookworm / golang:1.23-alpine), not
  the cgap-ax-testbed image; AX-7's container arms never ran (exit 3).
- Clause 2 PASSES: two delta tables published (AX-5 W4, AX-6 W4).
- Clause 3 PASSES: suspend/resume loudly UNVERIFIED (AX-9 exit 3).
- Clause 4 PASSES: zero cluster-apply claims (AX-2 DRAFT-UNRUN; AX-0
  falsifier unviolated across all 8 AX receipts).
Verify = NOT PASSED (clause 1 fails; a clause is all-and-only). The
single unblocking move is `docker build -f courts/ax-testbed/Dockerfile
-t cgap-ax-testbed:ax1` + a re-run of the stock-image courts in-image
and AX-7's container arms — note the image WAS built later the same
day (lane-ax-01-image.md W3 #5, exit 0), so only the in-image court
re-runs remain.

## Receipt: commands + exits (this lane, this session)

| Command | Exit | Purpose |
|---|---|---|
| ls dflss/ ; audit/ ; lanes/ (census) | 0 | all DFX-1..5 docs + AX receipts confirmed on disk |
| ls lanes/lane-ipc-02-alloc-purity.md | 0 | DFX-1 divergence 1 RESOLVED (receipt landed) |
| grep release-blockers.md for AX/interop terms | 1 (zero) | registration-gap finding |
| grep audit-ledger.md for AX/ipc terms | 1 (one hit) | false positive: 'max-consumer' only |
| awk 'length > 100' over both owned files | 0 | <=100-char law holds |
| sha256 of both owned files | 0 | on-disk bytes verified |
