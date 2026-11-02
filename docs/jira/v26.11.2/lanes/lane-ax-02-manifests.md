# Lane AX-2 — AX Workspace/Task manifests for the interop courts (DRAFT-UNRUN)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout).
No git write commands run.

## W1 Files written (owned, all on disk this session)

- `/Users/sac/cgap/courts/ax-testbed/manifests/workspace-cgap.yaml`
- `/Users/sac/cgap/courts/ax-testbed/manifests/task-alloc-purity.yaml`
- `/Users/sac/cgap/courts/ax-testbed/manifests/task-membrane-latency.yaml`
- `/Users/sac/cgap/courts/ax-testbed/manifests/task-transport-shm.yaml`
- `/Users/sac/cgap/courts/ax-testbed/manifests/task-mutation-matrix.yaml`
- `/Users/sac/cgap/courts/ax-testbed/manifests/README.md`
- this receipt

## W2 Commands + exits (this session)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `git rev-parse HEAD` (read-only) | 0 | `9a730320e9c536814e4dbbc88b66af43b7a11da8` — Workspace revision pin |
| 2 | `ax version` | 127 | command not found — ax CLI ABSENT |
| 3 | `ax apply --dry-run -f .../workspace-cgap.yaml` | 127 | command not found — apply UNVERIFIED-by-apply, stated plainly |
| 4 | `python3 yaml.safe_load` over all 5 manifests | 0 | all parse: 1 Workspace + 4 Tasks, names/kinds as authored |

## W3 DRAFT-UNRUN law

AX-0 gate (`lane-ax-00-gate.md:44-49`) verdict for AX-2 is NO-GO →
LOCAL-ONLY = DRAFT-UNRUN: cluster endpoint conn-refused (gate cmd #3, exit 1),
ax CLI absent (gate cmd #1, exit 127), install NOT APPROVED (gate W3).
Zero applied. No mocked apply output anywhere in this set. Re-run
`ax apply --dry-run` exit 127 THIS session (W2 #3) — the absence is fresh,
not hearsay from AX-0.

## W4 Schema status: UNVERIFIED

`courts/interop-chicago/README.md` contains NO YAML schema (it is a
Go-module readme). No vendor ax README/schema found on disk this session
(grep zero hits). The `ax.io/v1alpha1` Workspace/Task field names are
best-effort UNVERIFIED guesses and must be corrected against the real
vendor schema before first apply. YAML well-formedness (W2 #4) is the only
verified property.

## W5 Court→Task mapping

| Task | Court source | Receipt status |
|---|---|---|
| alloc-purity | `courts/interop-chicago/ffi/alloc_court_test.go` (TestSteadyStateAllocPurity, TestVersionAllocPurity, TestAntiVacuityFlip, TestAdmitPositiveControl) | ports lane-ipc-02 — receipt ABSENT on disk (divergence below) |
| membrane-latency | `go test -bench BenchmarkAdmit{Single,Batch1365} -benchtime 200x -count 5` per lane-ipc-03:16 | landed (lane-ipc-03-latency-court.md) |
| transport-shm | `go test ./shm/ -v` + CGO_ENABLED=0 + bench per lane-ipc-04:21-23 | landed (lane-ipc-04-shm-transport.md) |
| mutation-matrix | ports lane-ipc-05/AX-7 — composed from in-tree anti-vacuity flips (TestAntiVacuityFlip, TestMagicCorruptionFlip) + b06 C7 CSCB_MUTATE digest/rate mutations | receipt ABSENT on disk (divergence below) |

Resource-why (uniform): 2 vCPU / 4Gi caps mirror lane-b06's measured
load-sensitivity class (`lane-b06-cscb-c7.md:26-28` — the C7 rate-floor
gate flips exit 1 under machine contention; the cap keeps verdicts
load-bounded rather than noisy-neighbor-dependent).

Image pin: every Task uses `cgap-interop-court:PENDING-DIGEST` — no
lane-ax-01 receipt on disk at authoring time (census this session);
digest-pin owed on AX-1 landing.

## W6 Fences

None touched: manifests are new files outside any gate's path boundary;
no shared lib.rs/Cargo.toml edits; compile-freeze SLA not engaged.
Must-not-regress surfaces named: `make gate0` / boundary_gate /
zero_string_gate are untouched by this diff (verified: all six files are
new under courts/ax-testbed/manifests/ + docs/).

## W7 Standing

- Manifest authoring + YAML parse: **ALIVE** (W2 #4, exit 0)
- `ax apply` (even dry-run): **UNVERIFIED-by-apply** — CLI absent exit 127
  this session; DRAFT-UNRUN stands; no simulation performed
- Court→lane port fidelity: **PARTIAL** — 2 of 4 mapped lane receipts
  (ipc-02, ipc-05) absent on disk; task commands grounded in the in-tree
  court sources, not the missing receipts' exits
- Image digest: **BLOCKED** on lane-ax-01

## Divergences (doc-yields-to-reality notes)

1. Dispatch said "vendor README schema" — the only vendor README on disk
   (`courts/interop-chicago/README.md`) has no YAML schema; schema authored
   UNVERIFIED, named as such here and in manifests/README.md.
2. lane-ipc-02 and lane-ipc-05 receipts do not exist on disk (census this
   session); the alloc-purity and mutation-matrix tasks port the in-tree
   court test files, not receipted exits.
3. lane-ipc-04's shm mechanism is macOS-specific; container exec is a
   named divergence in task-transport-shm.yaml (in-container behavior
   UNVERIFIED until first real run).
4. No lane-ax-01 receipt exists; image refs carry PENDING-DIGEST markers.
