# CULL-07 — IPC-2..8 darwin-host invocation cull (post AX-4..8 container migration)

Date: 2026.11.02. Lanes read fresh this session: lane-ax-04-alloc.md,
lane-ax-05-latency.md, lane-ax-06-shm.md, lane-ax-07-mutation.md,
lane-ipc-02-alloc-purity.md, lane-ipc-03-latency-court.md,
lane-ipc-04-shm-transport.md, lane-ipc-06-sealer.md,
lane-ipc-08-index.md, lane-posh-05-matrix.md.
AX-8 receipt: ABSENT on disk (no lane-ax-08*) — no container leg for
IPC-5/IPC-7 can be credited to AX-8; nothing is culled on its behalf.

## The law applied

- Delta-table law: darwin numbers stay the measured baseline until
  linux/arm64 supersedes them with a published table.
- AX-5's published table shows container execution is ~1.7-1.8x WORSE
  than darwin (cgo floor 154 vs 73-85 ns/call; single admit
  306.9-313.1 vs 182.7-185.6 ns). Supersession would require the
  container to be the better-attested substrate; it is the worse one.
  Therefore darwin REMAINS the perf baseline for latency courts.
- AX-4 vs IPC-2 alloc figures are DIFFERENT LOOPS, not competing
  measurements of one loop: IPC-2 measured the steady admit+eval loop
  WITH the per-op Go `View` mint (honest 1.0 allocs/op, esc trace
  recorded); AX-4's container court asserted final state over
  AllocsPerRun(1M, admit+eval) = 0.0 — the view-mint tax is not in
  that loop shape. Neither number supersedes the other.

## Verdict per court

| Court (darwin lane) | Container leg | Verdict |
|---|---|---|
| IPC-2 alloc-purity (ffi) | AX-4 6/6 PASS in-container | BOTH-REQUIRED |
| IPC-3 latency (ingress) | AX-5 delta table published | BOTH-REQUIRED |
| IPC-4 shm transport | AX-6 6/6 PASS + delta table | BOTH-REQUIRED |
| IPC-5 mutation (absent) | AX-7 4-arm host; container LOUD-BLOCKED exit 3 | KEEP-DARWIN-BASELINE |
| IPC-6 receipt sealer | none (no container analog exists) | KEEP-DARWIN-BASELINE |
| IPC-7 | no receipt on disk, no lane file | PENDING (nothing to cull) |
| IPC-8 charter/index | n/a — documentation-only | KEEP-DARWIN-BASELINE |

## Per-court detail

### IPC-2 alloc-purity — BOTH-REQUIRED

- IPC-2 darwin: 1.0 allocs/op steady admit+eval WITH view-mint is the
  ALLOC-PURITY CANONICAL. It is the honest membrane tax (view escapes
  across the cgo boundary; esc trace at alloc_court_test.go:110/140),
  recorded unmasked per the honesty clause. AX-4's 0.0 does not
  measure that loop — it is the container court's final-state loop,
  view handling not equivalent. Reconciled: cite IPC-2's 1.0 as the
  per-op view-mint tax; cite AX-4's 0.0 only as "container court
  loop, linux/arm64" with the loop difference named.
- KEEP (canonical, darwin): TestSteadyStateAllocPurity,
  TestMembraneTaxDecomposition, TestVersionAllocPurity,
  TestAntiVacuityFlip, both positive controls — the darwin run
  (`CGO_ENABLED=1 go test -count=1 ./ffi`) stays the canonical gate.
- KEEP (container): AX-4 alloc.sh — the only linux/arm64 alloc
  witness; also carries the anti-vacuity flip witnessed in-container
  both directions. Not redundant.
- REDUNDANT: re-running AX-4's container suite as a perf claim is
  barred (no ns/op leg run; receipt says linux ns/op UNVERIFIED).
  Any darwin re-run whose sole purpose is re-deriving 0.0/1.0 on an
  unchanged loop is redundant — the numbers are pinned, drift flips.

### IPC-3 latency — BOTH-REQUIRED (delta-table law)

- Darwin baseline STANDS: 85 ns/call membrane floor (73 ns session
  re-run), 182.7-185.6 ns single admit, 179-181 ns/frame batch
  amortized, ~5.4M frames/s. Container is 1.7-1.8x worse; it cannot
  supersede a faster measured substrate.
- KEEP (canonical, darwin): BenchmarkAdmitSingle,
  BenchmarkAdmitBatch1365, TestBatchStats on the darwin dylib —
  these remain THE numbers any throughput bullet cites.
- KEEP (container): AX-5 latency.sh — sole source of the published
  delta table; bullets derived from it must read
  "containerized execution" (~3.4M frames/s amortized).
- REDUNDANT: further darwin fence bench re-runs of the batch-stats
  row (AX-5 already re-measured it: 176/178/182 vs IPC-3's
  183/185/187) absent a code change to the admit path; the
  timer-resolution witness (41 ns, identical both substrates) needs
  no third re-run.

### IPC-4 shm transport — BOTH-REQUIRED (delta-table law)

- AX-6 published the shm delta vs IPC-4: mmap read 4.574-4.656 vs
  4.52-4.54 ns/op (+1..+3%); pread 1185-1203 vs 1377-1405 ns/op
  (-15..-16%); 0 allocs both arms both OSes. Deltas are
  single-digit-percent on the load-bearing arm — the mechanism is
  portable; neither column supersedes the other.
- KEEP (canonical, darwin): IPC-4's 4 courts + cost table — the
  darwin/arm64 numbers stay the read-path baseline.
- KEEP (container): AX-6 devshm roundtrip + devshm magic flip —
  the only witnesses that /dev/shm tmpfs backing works (tmpfs is
  absent on darwin; darwin cannot cover this arm at all).
- REDUNDANT: re-running IPC-4's pread baseline on darwin beyond a
  fence (it is a cost disclosure, already tabled twice).

### IPC-5 mutation (receipt absent; AX-7 port) — KEEP-DARWIN-BASELINE

- AX-7's container arms are LOUD-BLOCKED (exit 3, cgap-ax-testbed
  image unbuilt; AX-1 image absent). The witnessed baseline is the
  darwin `--host` run: arms 1/2a/2b/4 PASS, arm 3 loud
  [SKIP-MISSING-EXPORT]. Per the receipts' own law, nothing
  supersedes a container run that never executed.
- KEEP: `./courts/ax-testbed/runners/mutation.sh --host` is the
  canonical mutation-court invocation until the container arms
  unblock AND publish a delta vs POSH-5's matrix.
- POSH-5's matrix (cgap-git Rust surface, 4/4) stays canonical for
  the Demux/RefTable/IngressRing surface; AX-7's arms are the
  C-ABI-membrane analog, not a replacement — different subjects,
  both required on their own surfaces.
- REDUNDANT: none yet — no container leg exists to supersede.

### IPC-6 sealer — KEEP-DARWIN-BASELINE

- Pure Go stdlib tool, no cgo, no OS-bound mechanism; a container
  run would measure nothing darwin cannot. No delta table exists or
  is owed. Darwin/host invocations are canonical and uncullable.

### IPC-7 — PENDING

- No lane-ipc-07 receipt, no owned surface named anywhere read this
  session. Nothing to cull; recorded as an honest absence.

### IPC-8 charter/index — KEEP-DARWIN-BASELINE

- Documentation-only; no invocations to cull. Its PENDING rows for
  IPC-0..7 are stale relative to the now-present IPC-2/3/4/6/8
  receipts — an index refresh is owed by its owning lane, not by
  this cull.

## Cull summary

Redundant darwin invocations (drop from routine rotation):
1. Darwin batch-stats bench re-runs (AX-5 fence re-measured them).
2. Timer-resolution witness re-runs (41 ns, identical everywhere).
3. Darwin pread-baseline benches (cost disclosure already tabled).
4. Any darwin or container re-derivation of the 0.0-vs-1.0 alloc
   figures on unchanged loops (both pinned; drift flips the court).

Canonical invocations that MUST stay:
- IPC-2 darwin ffi suite (alloc-purity canonical: 1.0 view-mint).
- AX-4 alloc.sh (sole linux/arm64 alloc witness).
- IPC-3 darwin benches (perf baseline; container is 1.7-1.8x worse).
- AX-5 latency.sh (sole delta-table source; label "containerized").
- IPC-4 darwin mmap courts + AX-6 devshm courts (disjoint arms).
- AX-7 mutation.sh --host (container arms LOUD-BLOCKED) and
  POSH-5's Rust matrix (different subject surface).

Standing: every cited receipt ALIVE this session per its W7; AX-8,
IPC-5, IPC-7, lane-ipc-01, lane-ax-01 receipts ABSENT on disk
(witnessed via lanes/ listing) — recorded, not invented.
