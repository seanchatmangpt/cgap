# 01 Project Charter — Interop (DFLSS DMADV, IPC wave)

Interop-lane corrective charter for v26.11.2 starDF. This file corrects
the dispatch prose that circulated for the IPC wave: figures that were
asserted without a this-session command+exit are demoted to PENDING or
superseded by the lane receipt that actually measured them. Corrections
here are MANDATORY for every downstream interop document.

Citation law: every kernel figure cites its lane receipt file:line. A
figure with no landed receipt is PENDING, never rounded up.

## Peer census at receipt time

`ls docs/jira/v26.11.2/lanes/lane-ipc-0*.md` run this session
(2026-11-02) returned ZERO matches: IPC-0 (ground truth), IPC-1..IPC-7
are all PENDING. No IPC receipt has landed; no peer figure is admitted
in this charter. The stitch index
`../lanes/lane-ipc-08-index.md` carries one PENDING row per absent peer.

## Corrected sales bullets

### C-ABI export table

IPC-0 PENDING — no ground-truth receipt
(`../lanes/lane-ipc-00-ground-truth.md`) has landed. The export table is
not asserted here. Observed this session in the shared tree (read-only,
subject to IPC-0's authoritative count): cgap-ffi carries `#[no_mangle]`
/`#[export_name]` symbols across
`crates/cgap-ffi/src/{lib,xip_tables,sovereign}.rs`
(6 / 3 / 10 respectively by grep count), including `cgap_i32_batch`,
`cgap_version`, `cgap_digest_of`, `cgap_xip_oabi_table`,
`cgap_xip_bn3_network`, `cgap_xip_start`. This is a census, not the
export-table truth — IPC-0 PENDING.

### /dev/shm

IPC-4 PENDING — no receipt, so the shared-memory mechanism is NOT
asserted. `/dev/shm` is a Linux tmpfs path; on this macOS host
(Darwin 25.2.0) it does not exist as shipped, so any Linux-flavored
claim is already suspect. The corrected text is: "shared-memory
mechanism per IPC-4's receipt (IPC-4 PENDING)". When IPC-4 lands, its
actual macOS mechanism (whatever it names) supersedes this line.
### 6.04 ns/frame

PROVISIONAL ARITHMETIC ON ASSUMED KERNEL TIME. The 6.04 ns/frame figure
is not measured: no IPC-3 receipt has landed (IPC-3 PENDING), and the
arithmetic assumed a kernel per-frame cost never witnessed this session.
Superseded by IPC-3's measured figure when
`../lanes/lane-ipc-03-*.md` lands. Until then the only honest latency
claim is: PENDING, with the provisional arithmetic retained solely as
the shape of the derivation, not a claim.

### 4.0M seals/s bullet

REPLACED by B06's receipted median: 4,377,052 / 4,356,280 / 4,351,961
receipts/s across three clean release runs (median-of-runs 4,356,280/s),
min 4.25M/s, p99 ~4.4M/s
(`../lanes/lane-b06-cscb-c7.md:23-24`). Load-sensitivity caveat, witnessed
this session by B06: the contended run FAILED with typed
`[C7-FAIL] rate median 727,755/s < floor 1,000,000/s`
(`../lanes/lane-b06-cscb-c7.md:25-27`), and the debug build failed at
204,511/s (`../lanes/lane-b06-cscb-c7.md:30-32`). The figure is
machine-dependent and holds for the uncontended release build on this
box only (`../lanes/lane-b06-cscb-c7.md:51-54`). "4.0M seals/s" as a
flat claim is retired.

## Falsifier (verbatim from dispatch)

interop Verify passes iff every sales bullet emitted carries a
this-session command+exit and the mutation matrix is 4/4 (or loud
[SKIP] with named missing symbols).

## Honest post-drain claim ceiling

Per the coordinator note, corroborated this session by read-only grep:
cgap-ffi contains NO radix entry point and no kernel-call batch surface
beyond `cgap_i32_batch` (grep for `radix` across
`crates/cgap-ffi/src/` returns zero matches). Honest ceiling:
"membrane harness landed; kernel-call path owed in cgap-ffi". Any
end-to-end interop throughput claim is capped at this ceiling until the
kernel-call entry points land.
## Scope and authority

This charter is a documentation lane (IPC-8). It owns exactly two files:
this file and `../lanes/lane-ipc-08-index.md`. Peer receipts are
read-only inputs; landing any is the owning lane's job, not ours. Git
transitions belong to the coordinator.

## Standing

ALIVE for the two corrections that are grounded this session (B06
median replacement, coordinator-note ceiling via grep census); PENDING
for IPC-0, IPC-3, IPC-4 dependent bullets. UNVERIFIED never rounds up.
