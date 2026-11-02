# AX-2 manifest set — cgap interop courts (DRAFT-UNRUN)

Status: **DRAFT-UNRUN** per `docs/jira/v26.11.2/lanes/lane-ax-00-gate.md:44-49`
(cluster NO-GO, ax CLI absent exit 127, install unapproved). Authored, zero
applied. No mocked apply output exists anywhere in this set.

## Files

| File | Kind | Ports |
|---|---|---|
| workspace-cgap.yaml | Workspace | pins /Users/sac/cgap @ 9a730320e9c536814e4dbbc88b66af43b7a11da8 |
| task-alloc-purity.yaml | Task | lane-ipc-02 (receipt ABSENT — divergence) |
| task-membrane-latency.yaml | Task | lane-ipc-03 |
| task-transport-shm.yaml | Task | lane-ipc-04 |
| task-mutation-matrix.yaml | Task | lane-ipc-05/AX-7 (receipt ABSENT — divergence) |

## Schema caveat (UNVERIFIED, stated plainly)

The `ax.io/v1alpha1` Workspace/Task schema is vendor-README material only.
No on-disk vendor README with a YAML schema was found this session
(`courts/interop-chicago/README.md` is a Go-module readme, zero YAML schema).
Field names are best-effort guesses; they are UNVERIFIED and will need
correction against the real `ax` CLI or vendor docs before first apply.

## Resource-why (uniform across tasks)

2 vCPU / 4Gi limits mirror lane-b06's measured load-sensitivity class
(`lane-b06-cscb-c7.md:26-28`): the C7 rate-floor gate flips exit 1 under
machine contention; an uncapped court can pass or fail on noisy-neighbor
load, so the cap makes verdicts load-bounded.

## Image pin

Every task references `cgap-interop-court:PENDING-DIGEST` by name. No
lane-ax-01 receipt exists on disk at authoring time (census run this
session), so no digest is available; digest-pin on AX-1 landing.

## Apply law

`ax apply` requires a reachable Substrate control plane. Do NOT apply from
this set until: (1) ax CLI installed with operator approval, (2) cluster
reaches GO, (3) AX-1 image digest landed, (4) schema verified.
