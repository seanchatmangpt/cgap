# Lane AX-0 Gate Receipt — AX Substrate Verdict (Gates AX-1..10)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout).
Role: fleet gate — AX-1..AX-10 held on this verdict.

## W1 Identity

- Lane: AX-0 (gate). Owned file: this receipt only. No git commands run.
- Verdict type: GO / LOCAL-ONLY / NO-GO per downstream lane family.

## W2 Commands + Exits (all run this session, 2026-11-02)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `ax version` | 127 | `command not found: ax` — ax CLI ABSENT |
| 2 | `kubectl version --client` | 0 | Client v1.35.2, kustomize v5.7.1 |
| 3 | `kubectl get svc api -n ate-system` | 1 | connection refused 127.0.0.1:64378 — no reachable cluster |
| 4 | `podman --version` | 127 | not installed |
| 5 | `docker --version` | 0 | Docker version 28.0.4 |
| 6 | `docker info` (initial) | 1 | daemon NOT running at first probe |
| 7 | `open -a Docker` | 0 | Docker Desktop launched |
| 8 | bounded poll `docker info` (60x2s) | 0 | daemon up after 1 poll; ServerVersion 28.0.4 |
| 9 | `go version` | 0 | go1.23.4 darwin/arm64 |
| 10 | `cargo --version` | 0 | cargo 1.101.0-nightly (cad07cf8e) |
| 11 | `kubectl config get-contexts` | 0 | 12 contexts (colima, kind-* incl. smalt); none reachable this session |
| 12 | `kubectl get nodes` | stderr conn-refused | same unreachable endpoint |

## W3 Operator approval status (ax install)

- `ax` CLI absent (exit 127). Install (`go install github.com/google/ax/cmd/ax@latest`)
  requires operator pre-approval per dispatch.
- Searched /Users/sac/cgap/docs (grep for ax.io / ate-system / google/ax /
  AX README / ax apply): ZERO matches. No operator approval statement exists
  on disk.
- Standing: ax install = NOT APPROVED this session. `ax version` remains
  UNVERIFIED-by-absence; the whole AX API surface (ax.io/v1alpha1, Task/
  Workspace kinds, ax apply/describe/ssh/suspend/resume) is VENDOR-README
  material, UNVERIFIED.

## W4 GO / LOCAL-ONLY / NO-GO table

| Lane family | Needs | Verdict | Basis |
|---|---|---|---|
| AX-1 / AX-3 (image build) | podman OR docker | **GO** (docker) | docker 28.0.4 CLI + daemon ALIVE (exit 0); podman absent. Build root: /tmp lease per sandbox law. |
| AX-2 (manifests) | cluster for real apply | **NO-GO → LOCAL-ONLY = DRAFT-UNRUN** | ate-system svc unreachable (exit 1, conn refused). Manifests may be drafted, never applied; any apply claim is hearsay. |
| AX-4/5/6/7/8 (container court migrations) | image + exec | **GO via LOCAL exec mode (`docker run`)** | daemon ALIVE; cluster-independent courts must exec via docker run, not k8s exec. Each court still names its anti-vacuity mutation. |
| AX-9 (suspend/resume) | live cluster + ax CLI | **NO-GO → UNVERIFIED-by-design** | both `ax` (127) and cluster (exit 1) absent. No simulated pass permitted. |
| AX-10 (whatever dispatch defines) | — | follows its dependencies | if cluster-bound: NO-GO; if docker-bound: GO. |

Fleet re-scope verdict, stated plainly:
- CLUSTER PATH: NO-GO this session. Nothing in AX-1..10 may claim a real
  `ax apply` / k8s-apply / describe / ssh / suspend / resume witness.
- LOCAL PATH: GO. Docker builds + `docker run` exec courts are lawful.
- `ax` install: NOT APPROVED — no operator statement found on disk
  (docs/ grep zero hits). Lanes must not self-approve.

## W5 Verification ladder

Local probes (exit codes above) are the ground truth; no build was run in
this lane — AX-1 owns the first build. Falsifier for this gate: any
downstream lane citing a cluster-apply success contradicts command #3
(exit 1) and is hearsay.

## W6 Fences (must-not-regress surfaces my diff could break)

None — this lane writes one markdown receipt. No code, no gates touched.
No shared lib.rs/Cargo.toml edits; compile-freeze SLA not engaged.

## W7 Standing

- docker local build path: **ALIVE** (daemon server 28.0.4, command+exit above)
- kubectl client: ALIVE (v1.35.2); cluster endpoint: **NO-GO** (exit 1)
- ax CLI: **BLOCKED** (absent, install unapproved)
- podman: absent
- go 1.23.4 / cargo 1.101.0-nightly: ALIVE
- AX API surface: UNVERIFIED (vendor-README material)
