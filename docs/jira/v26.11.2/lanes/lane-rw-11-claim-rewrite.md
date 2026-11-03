# Lane RW-11 — claim rewrite (charter CTQ + control plan)

Date: 2026-11-02. Repo /Users/sac/cgap @ main. Owned files:
`docs/jira/v26.11.2/dflss/02-ax-charter-ctq.md` +
`docs/jira/v26.11.2/dflss/10-ax-control-plan.md`. No git commands.
Receipt file is the only other file written.

## W1 Per-claim before/after

| # | Claim | Before (as read) | After | Authority (re-read this session) |
|---|---|---|---|---|
| 1 | CTQ-03 membrane allocs | PARTIAL; "ipc-02 receipt ABSENT"; only IPC-4 0 B/op | MEASURED (NOT MET at membrane): ipc-02 LANDED — steady admit+eval 1.0 allocs/op, 24 B/op (View mint), components 0.0; IPC-4 0 B/op kept as distinct no-cgo instrument | lane-ipc-02-alloc-purity.md:53-67 (landed; re-read) |
| 2 | CTQ-05 SRAM 36,048 B | unscoped "VERIFIED" | VERIFIED (subject-scoped per red-12: gated slice = blake3+branchless caller, verify_cscb_sram.py:51-54); bn3 REFUSED: 6,520,984 B > 131,072 (49.75x), typed exit 1 | red-12:59-87; fix-18:64-71; my own gate runs (W2) |
| 3 | AX-GATE-03 10 ns USL | "179-192 ns/frame darwin" (192 is the CONTAINER median, misattributed) | 179-187 darwin (IPC-3 5-run 179-181, 7-run 183/185/187; AX-5 darwin this-session 176/178/182); 10 ns NOT met at membrane, RE-SCOPED per MV-5, kept on the books | lane-ipc-03:44-47; lane-ax-05-latency.md:68-71 |
| 4 | ~10 ns quiet-host class | absent from both files | kernel-layer-only note: row 5 L1 ingress 10.20/10.30, reproduced 10.26-10.65 in-tolerance by R-15; different counter from membrane ns/frame (unit law 06-bench-protocol §7) | 11-measured-claims.md:29; fix-18:22-37 |
| 5 | ipc-02 absence records | divergence 1 (charter) + OUT-OF-CONTROL row (control plan) said receipt absent | both marked SUPERSEDED — receipt landed; historical absence window preserved via IPC-3 W5.1 disclosure cite | ls exit 0 this session; ipc-03:56-64 |
| 6 | stale line cites | sipoc-c4.md:113 (a section heading) | :119 (actual suspend/resume row) | grep -n re-read |
| 7 | CTQ-03 flip cite | ipc-04:49 (cited for alloc flip) | :46-48 magic flip + :50,79-82 ipc-02 alloc-instrument flip | ipc-04:45-53; ipc-02:50,79-82 |

## W2 Commands + exits (this session)

| command | exit |
|---|---|
| `python3 scripts/verify_cscb_sram.py` | 0 (36048 B <= 131072) |
| `python3 scripts/verify_cscb_sram.py --subject bn3` | 1 (BN3_ENCLAVE_REFUSED, 6,520,984 B, 49.75x) |
| `python3 scripts/verify_cscb_sram.py --mutate=sram-bloat` | 0 (mutant 136,068 B flips) |
| `ls lanes/ \| grep ipc` | 0 (ipc-00/01/02/03/04/06/08 on disk) |
| grep -n stale-cite sweep over both owned files | 0 stale after fix |

## W3 Courts / anti-vacuity

sram-bloat mutant flip re-run this session (exit 0, overflow flips) —
the CTQ-05 cell's court, re-witnessed. bn3 typed refusal re-witnessed
(exit 1). Fences: none broken — both files are docs-only; the gate
scripts' exit ABI untouched; zero_string_gate / boundary_gate surfaces
not engaged (no code diff).

## W4 Divergences

1. Dispatch said "row 5: 10.20-10.65 reproduced in-tolerance by R-15";
   fix-18 records the band as 10.4-10.8 with fastest 10.26-10.65
   straddling it (leg 3 at 10.26, 1.3% below floor, inside the 4.5%
   tolerance). Quoted fix-18's own typed wording.
2. Dispatch's "red-12: bn3 6,520,984 B REFUSED" is red-12 W5 arithmetic;
   the gate-level typed refusal (`--subject bn3` exit 1) is R-15's W4
   witness; both cited, and re-run by this lane this session.
3. Pre-existing duplicate-tail in CTQ-03 (old close-condition bullets
   survived my rewrite) — caught in the on-disk token sweep, removed;
   the stale `:49` cite it carried is gone.

## W7 Standing

- ALIVE: all W2 commands + exits above, this session, exact subjects.
- UNVERIFIED (unchanged, never rounded up): CTQ-01, CTQ-02, CTQ-06
  container arm, AX-GATE-02/04/05 container cells, membrane 10 ns USL
  (NOT MET / RE-SCOPED).
- Receipt-cited (not re-run): R-15 bench legs, IPC-2/3/4 Go benches,
  AX-5 delta table, B01 kernel p99.

```json
{"lane":"RW-11",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/dflss/02-ax-charter-ctq.md",
  "/Users/sac/cgap/docs/jira/v26.11.2/dflss/10-ax-control-plan.md",
  "/Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-rw-11-claim-rewrite.md"],
 "commands":[
  "python3 scripts/verify_cscb_sram.py (exit 0, 36048 B)",
  "python3 scripts/verify_cscb_sram.py --subject bn3 (exit 1, 6520984 B refused)",
  "python3 scripts/verify_cscb_sram.py --mutate=sram-bloat (exit 0, flip)",
  "ls lanes/ | grep ipc (exit 0, ipc-02 landed)"],

 "courts_run":["verify_cscb_sram --mutate=sram-bloat flip (re-witnessed)"],
 "standing":"ALIVE",
 "divergences":[
  "R-15 band wording: 10.4-10.8 band, fastest 10.26-10.65, typed in fix-18; dispatch phrasing normalized",
  "bn3 refusal gate-level witness is R-15 W4; red-12 carries the arithmetic; both cited",
  "CTQ-03 duplicate-tail corruption from own rewrite caught + removed in token sweep"]}
```
