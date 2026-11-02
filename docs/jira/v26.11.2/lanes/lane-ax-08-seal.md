# Lane AX-8 — in-container honesty-gated receipt sealing (SL-001..004)

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical checkout).
Reads: `lanes/lane-ipc-06-sealer.md` (register of truth),
`lanes/lane-ax-01-image.md` (image ALIVE, digest-pinned).

## W1 Identity

- Lane AX-8. Files written (sha256 this session):
  - `/Users/sac/cgap/courts/ax-testbed/runners/seal.sh`
    `90ea7ccc034a456a154054554d11a6fc41320cac5517df81cffb16566190f438`
  - `/Users/sac/cgap/courts/interop-chicago/cmd/seal_receipt/container_run.sh`
    `70f28bbeb98d06e6536c4d90f602d9fcd1f64f473750324eacc14a236116566b`
  - `/Users/sac/cgap/courts/interop-chicago/cmd/seal_receipt/main.go` (SL-004)
    `18565e4c6eab2c8380ccc7a381b439a668e7a64b62f21ae8b05921b85e959ead`
    (IPC-6 owned this file; this edit is the chartered SL-004 extension,
    disclosed here against lane-ipc-06's receipt)
  - this receipt.
- Sandbox fixtures: `/tmp/ax8-sandbox/` (lane debris, flip replay corpus).
- No git commands run.

## W2 Authority

Dispatch prompt LANE AX-8. SL-004 required code (read first, per dispatch:
the where-it-ran fields did not exist), so the sealer was extended in place.
Mid-run coordinator status check received; work was already past flips,
receipt landing followed immediately.

## W3 Commands + exits (all this session, darwin host, docker 28.0.4 LOCAL)

| # | Command | Exit |
|---|---|---|
| 1 | `go vet main.go` + `go build -o /tmp/ax8-seal_receipt main.go` (darwin, pre-check) | 0 |
| 2 | `seal.sh -in bad_sl001.json ...` (in-container, docker cgap-ax-testbed:ax1) | 1, `REFUSED SL-001: bullet "probe measured 42 units" is quantitative but lacks method disclosure` |
| 3 | same for bad_sl002 | 1, `REFUSED SL-002: ... cites no receipt path` |
| 4 | same for bad_sl003 | 1, `REFUSED SL-003: ... number lacks a this-session command exit` |
|  | NOTE: exits 2-4 witnessed | the SL-001..003 honesty gates hold IDENTICALLY in-container — re-witnessed, not assumed from darwin |
| 5 | same for bad_sl004 (where-it-ran fields = "") | 1, three named `REFUSED SL-004:` lines (digest / runner version / sandbox class) |
| 6 | `seal.sh -in good.json ...` | 0, `EMITTED /work/out/good.md (1033 bytes) transcript-root sha256 sealed`; md records image digest `sha256:52a442e03b3d...373`, runner `ax-task-runner@b82f57ee...`, sandbox `runc` |
| 7 | direct-mode SL-004 flip (no `-fill-where`), in-container `go build` + `seal_direct -in bad_sl004_absent.json` | 1; output `direct_bad.md` NOT written (`ls: cannot access`); three named SL-004 lines |
| 8 | same, `-in good.json` | 0, EMITTED 1033 bytes |

## W5 Courts table

| Court | Mutation (anti-vacuity) | Flip witness | Exit |
|---|---|---|---|
| SL-001 in-container flip | quantitative bullet, method stripped | REFUSED line named; output NOT written | 1 |
| SL-001 fixed | method restored | EMITTED | 0 |
| SL-002 in-container flip | receipt paths stripped | REFUSED SL-002 named | 1 |
| SL-002 fixed | receipt restored | EMITTED | 0 |
| SL-003 in-container flip | command+exit stripped from CTQ row | REFUSED SL-003 named | 1 |
| SL-003 fixed | command+exit restored | EMITTED | 0 |
| SL-004 in-container flip | where-it-ran fields empty AND absent (two fixtures) | three named REFUSED SL-004 lines; output not written | 1 |
| SL-004 fixed / fill-path | fields carried; absent-only fill from real `docker inspect` digest + in-image `ax_commit.txt` | EMITTED with digest+runner+sandbox recorded | 0 |
| fill-path vacuity guard | `-fill-where` must NOT overwrite explicitly empty values | bad_sl004 with `-fill-where` still REFUSED (exit 1); first implementation unconditionally overwrote and emitted (exit 0) — caught and fixed this session | 1 |

## W4 Consequence / fences

- seal.sh: container-side invocation path — mounts sealer source read-only,
  fills SL-004 from real observables (`docker inspect` digest;
  `/usr/local/share/ax_commit.txt` read inside the image), execs
  `container_run.sh` (go vet + build + run inside cgap-ax-testbed:ax1).
- Fences: main.go is IPC-6's file — fence is the IPC-6 court battery; the
  darwin-side pre-check (cmd 1) shows go vet/build still clean; every
  IPC-6 gate re-flipped in-container above, so the fence holds.
- No Rust crates, lib.rs, Cargo.toml, or shared scripts touched.
- Transcript-root divergence stays NAMED: sha256, not BLAKE3 (Go stdlib);
  the emitted receipt carries the DIVERGENCE-NAMED line unchanged.

## W6 Standing

- In-container sealer (SL-001..004, emit + refusals): **ALIVE** — every
  command above ran this session against the exact subject with exits shown.
- seal.sh container path: ALIVE (all 6 runs went through it).
- sandbox_class is DECLARED by the caller (default runc), not probed from
  the runtime: recorded as a named divergence — docker Desktop uses runC
  (AX-0 GO), but the sealer does not read containerd runtime metadata.
- Perf numbers: none claimed; N/A lane.

## W7 Divergences / disclosures

1. SL-004 required code (as anticipated by dispatch); main.go extended:
   version 0.1.0 -> 0.2.0, three new pointer fields (absent vs empty
   distinguished so the fill path cannot launder an empty value), new
   `-fill-where`/`-ax-*` flags, receipt-level gate in selfLint, three new
   Environment lines in emit. Disclosed here against lane-ipc-06-sealer.md
   (the owning lane supersedes this diff on landing).
2. First fill implementation unconditionally overwrote stripped fields,
   flipping bad_sl004 to exit 0 — caught by the court itself, fixed to
   absent-only fill; the vacuity guard row in W5 records this.
3. seal.sh first emitted `name@sha256:...` (RepoDigests form); the gate
   refused it — gate did its job; seal.sh now normalizes to a bare
   `sha256:<64 hex>` digest.
4. sandbox_class declared-not-probed (W6); probing containerd runtime
   metadata is a follow-up lane item, not silently assumed.
