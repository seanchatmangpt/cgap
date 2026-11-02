# Lane AX-10 Receipt — AX Testbed Charter + Stitch Index

Date: 2026-11-02. Repo: /Users/sac/cgap (branch main, canonical
checkout). Last AX lane of the wave. No git commands run.

## W1 Files written (owned manifest, exactly two)

- /Users/sac/cgap/docs/jira/v26.11.2/dflss/01-project-charter-ax-testbed.md
  (NEW)
- /Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-ax-10-index.md
  (this receipt)

## W2 Census (this session)

`ls docs/jira/v26.11.2/lanes/ | grep -i ax` → lane-ax-00-gate.md,
lane-ax-01-image.md, lane-ax-02-manifests.md, lane-ax-04-alloc.md,
lane-ax-05-latency.md, lane-ax-06-shm.md, lane-ax-07-mutation.md,
lane-ax-09-suspend.md — 8 AX receipts; 118 lane receipts total in
the dir. AX-3 and AX-8 receipts are ABSENT on disk (grep for AX-3 /
AX-8 hits only lane-ax-00-gate.md and the dfx lanes) — both are
PENDING rows below, not silent skips. Every receipt cited in the
charter was re-read from disk this session (head/sed excerpts of
W2-W7 sections); no count or standing was taken from session memory.

## W3 Commands + exits (this session)

| # | Command | Exit |
|---|---|---|
| 1 | ls lanes dir, grep -i ax | 0 |
| 2 | ls docs/jira/v26.11.2/dflss/ | 0 |
| 3 | head/sed reads of lane-ax-00/01/02/04/05/06/07/09 receipts | 0 |
| 4 | grep -rl for AX-3 / AX-8 receipts under lanes/ | 0 (no receipt files) |
| 5 | head -50 dflss/01-project-charter.md | 0 (style/cite form) |
| 6 | Write + cat-append assembly of the charter | 0 |
| 7 | wc -l + sha256sum of both written files | 0 |
| 8 | awk length sweep + wrap pass | 0 (charter: 0 lines >100; receipt stitch-table rows >100 match repo precedent — lane-ax-00/05 carry 7/5 such rows) |

## W4 Stitch index — one row per AX lane

| lane | receipt | standing | load-bearing fact | PENDING peers |
|---|---|---|---|---|
| AX-0 | lane-ax-00-gate.md | ALIVE | Family gate: CLUSTER NO-GO (ate-system conn-refused, exit 1); ax CLI 127; ax install NOT APPROVED; LOCAL docker GO (28.0.4 daemon bounded-poll exit 0; podman 127) | all cluster-bound lanes |
| AX-1 | lane-ax-01-image.md | ALIVE | Image cgap-ax-testbed:ax1 built exit 0; digest-pinned bases; ax-task-runner @ b82f57ee; go1.23.4 / rustc 1.101.0-nightly / riscv32imc witnesses; flip court canonical 0 / mutated 1 | AX-7 container legs (image absent at its run time); AX-3 |
| AX-2 | lane-ax-02-manifests.md | DRAFT-UNRUN | 5 manifests + README drafted; YAML parse exit 0 (1 Workspace + 4 Tasks); ax apply --dry-run exit 127; schema fields UNVERIFIED guesses vs vendor schema | any apply lane; vendor schema confirmation |
| AX-3 | ABSENT | PENDING | no receipt on disk (gate W4 row exists) | image-build witness chain AX-1 -> AX-3 |
| AX-4 | lane-ax-04-alloc.md | ALIVE | linux/arm64 container 6/6 alloc-purity courts PASS; 0.0 allocs/op; flip both directions in-container; darwin fence go test exit 0 | AX-7 disclosed a pre-existing IPC-2 wireFrame defect (0x43470002) touching AX-4's ffi tests |
| AX-5 | lane-ax-05-latency.md | ALIVE | Delta table: cgo floor 73-85 vs 154 ns/call (~1.8x); single admit 182.7-185.6 (cited) vs 306.9-313.1 ns/op (~1.7x); batch median +7 ns, tail 1.6x; 41 ns timer both; STOCK images, not the AX image | AX-1-image-based re-run |
| AX-6 | lane-ax-06-shm.md | ALIVE | /dev/shm tmpfs witnessed (df 65536); devshm + magic-flip courts 6/6 PASS; min-of-3 deltas: mmap +1-3%, pread -15..-16%, 0 allocs | none |
| AX-7 | lane-ax-07-mutation.md | PARTIAL | Host legs: arms 1/2a/2b/4 PASS exit 0; arm 3 loud t.Skipf (no cgap_ingress_bytes export); container legs LOUD-BLOCKED exit 3 (image absent at run time) | container-mode re-run vs cgap-ax-testbed:ax1; arm 3 needs ingress symbol |
| AX-8 | ABSENT | PENDING | no receipt on disk | none |
| AX-9 | lane-ax-09-suspend.md | ALIVE (refusal witness) | suspend_probe.sh landed (bash -n 0); run exit 3 REFUSED[PREREQ-ABSENT]; zero sleeps; digest-compare falsifier encoded | ax install approval + reachable cluster |
| AX-10 | this receipt | ALIVE | Charter + index landed; census + re-reads witnessed in W2/W3 | AX-3/AX-8 receipts; AX-7 container re-run |

## W5 Vendor-claim status (charter mirror)

All four README claims (millions of tasks, sub-second activation,
gVisor isolation, suspend/resume lossless) remain
VENDOR-CLAIM-UNVERIFIED — zero in-session witnesses; gVisor is
additionally contradicted-in-part by the witnessed runc-class
runtime (AX-5 W1). Witnessed-and-admitted claims carry receipt
cites in the charter's measured-facts section only.

## W6 Fences + divergences

- Fences: documentation-only diff, no source/gate/manifest touched.
  The charter's cite surface (AX receipts) was re-read this session;
  the claim ceiling and vendor-claim rows are binding on future AX
  prose.
- Divergence (doc yields to reality): dispatch said "AX-3 check,
  AX-8 check" — the census shows NO AX-3/AX-8 receipt files on
  disk; recorded as PENDING rows, not assumed landed.
- No SLA unblocks applied to other lanes' files.
