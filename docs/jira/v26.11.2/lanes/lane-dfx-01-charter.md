# Lane DFX-1 — AX-integration charter + VOC->CTQ table (receipt)

Date: 2026-11-02. Repo /Users/sac/cgap (main). No git commands.
Files written (exactly two, both owned):

- `docs/jira/v26.11.2/dflss/02-ax-charter-ctq.md` (new, 209 lines)
- `docs/jira/v26.11.2/lanes/lane-dfx-01-charter.md` (this receipt)

## W1 Commands + exits (this session)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `python3 scripts/verify_cscb_sram.py` | 0 | 36,048 B <= 131,072 B |
| 2 | `... --mutate=sram-bloat` | 0 | mutant 136,068 B > bound, flips |
| 3 | `ls lanes/lane-ipc-02*` (v26.11.2/) | 1 | no ipc-02 receipt on disk (divergence 1) |
| 4 | `grep -rn "deterministic phase" dflss/ lanes/` (v26.11.2/) | 1 | zero matches (divergence 2) |
| 5 | `awk 'length > 100'` over the charter | 0 | zero over-length lines (<=100-char law) |
| 6 | full Read sweep of the landed charter | 0 | token sweep clean, no write-corruption artifacts |

## W2 Substrate (receipts re-read at use time)

Read in full this session before writing: `lane-ax-00-gate.md`
(register of truth), `lane-ipc-00`, `lane-ipc-03`, `lane-ipc-04`,
`lane-b01-cscb-c1-c2.md`, `lane-b04-cscb-c5.md`,
`lane-posh-05-matrix.md`, `lane-g10-flip-tests-reconcile.md`,
`lane-dx-06-fixture-races.md`, `audit/dod-csct-cscb.md`,
`dflss/07-ax-sipoc-c4.md`, `dflss/01-project-charter-interop.md`,
`dflss/02-voc-ctq-matrix.md`. No figure copied from the dispatch; every
cell re-derived from the cited receipt or re-run (W1 cmds 1-2).

## W3 CTQ verdict summary (details in the charter file)

- CTQ-01: DESIGN-CLAIM-UNVERIFIED (AX sandbox = vendor claim; real
  fleet witness is the opposite class — shared /tmp races, G10 + DX-6).
- CTQ-02: ASPIRATIONAL-UNVERIFIED ("deterministic phase transition"
  demoted; AX-9 NO-GO per ax-00:47).
- CTQ-03: PARTIAL (darwin 0 B/op via IPC-4 transport court; cgo arm
  unmeasured — ipc-02 receipt absent; arm64 PENDING).
- CTQ-04: VERIFIED-CONDITIONAL, kernel layer ONLY (p99 6 ns under the
  disclosed batch-64 method); membrane stated separately at 85 ns
  floor / 179-187 ns/frame (ipc-03) — layers kept distinct.
- CTQ-05: VERIFIED (36,048 B; mutant 136,068 B flips; re-run W1).
- CTQ-06: darwin VERIFIED (POSH-5 4/4 exit 0); container PENDING
  AX-4..7 (gate: GO via docker run, not yet executed).

## W4 Fences (must-not-regress surfaces)

Docs-only lane. Fences cross-checked: `dflss/02-voc-ctq-matrix.md`
(canonical CTQ-01..07 numbering for csct/cscb — my file is a PARALLEL
AX register, new filename 02-ax-charter-ctq.md, no collision with
01-project-charter-ax-testbed.md or 02-voc-ctq-matrix.md, verified by
ls this session); `dflss/01-project-charter-interop.md` figures not
superseded here except where its IPC-PENDING rows are now stale — that
file is NOT mine and is untouched; divergence recorded, not edited.

## W5 Divergences (doc yields to reality)

1. Dispatch: "cite lane-ipc-02's darwin measurement" (CTQ-03). No
   lane-ipc-02 receipt exists (`ls` exit 1, W1 cmd 3); IPC-3's W5
   disclosure confirms none was on disk at its close. Cited IPC-4's
   measured 0 allocs/run instead; cgo-membrane arm marked UNMEASURED.
2. Dispatch figure "CTQ-05 mutant 136,068 B flips": confirmed by
   re-run, not copied (W1 cmds 1-2, both exit 0).
3. Dispatch's "deterministic phase transition" draft phrase exists in
   no landed file; reclassified as ASPIRATIONAL per the gate receipt.
4. Dispatch named receipt `lane-dfx-01-charter.md` in lanes/ — no
   prior file existed; verified no collision.

## W6 Disclosure duties

No cross-lane file touched; no SLA unblock applied; no both-directions
disclosure owed. Compile-freeze SLA not engaged (no code edits).

## W7 Standing

ALIVE for the two landed files: charter figures re-derived or re-run
this session (W1 exits above); every receipt citation re-read from disk
at use time. UNVERIFIED/PENDING cells are stated as such in the charter
and never rounded up. Build-root lease: none needed (docs-only lane);
nothing to delete.

