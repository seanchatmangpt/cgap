# tps-07-kaizen-backlog

TPS-7 kaizen backlog (2026-11-02, read-only lane). Audits what the
TPS manual + doctrine-friction-census + dxd-10 wave plan leave
UNCOVERED. Fence: nothing re-proposes the landed DX fleet or
W10-DX-A..F lanes.

## (a) Muda — remaining waste

A1 (P0) write-corruption heredoc/python path UNCOVERED: 10 incidents
/ 7 lanes all used Bash heredoc or python3 -c assembly; DX-2's hook
gates Edit/Write tool_input only (write-integrity.sh:27). No W10 lane
owns the Bash-path extension. Mechanization: PreToolUse Bash hook
running the same mojibake/placeholder/control-byte detector on
write-shaped payloads; WARN not block. Scope S-M.
A2 (P0) 429 silent-death detection NOT mechanized: 2 confirmed
deaths (GIT-03/07) + ~25 indistinguishable slots. Mechanization:
dispatch ledger (lanes/dispatch-ledger.jsonl: lane id, ts, fleet,
premise hash) + doctor check 5 (ledger row without a matching
receipt after N min = WARN) + per-lane .alive heartbeat in the
template contract. Scope M.
A3 (P1) dispatch-premise validation NOT mechanized: 8-9 wasted
turns (G11-G14, GIT-16, MV dedup); _LANES-TEMPLATE.md has no
premise-check step. Mechanization: mandatory step 0 in the template
(re-run the premise command, paste output before any owned-file
write) + `just premise-check <lane>`. Scope S.
A4 (P2) rm-permission preflight + stale-binary autodetect: 9
rm-denied disclosures ~18 turns; partially fenced by W10-DX-C.
Mechanization: doctor build-artifact mtime-vs-source check per
active lane root (check_stale_server pattern). Scope M.

## (b) Muri — remaining overburden

B1 (P1) serialize-under-lock for latency courts is PROSE only:
witnesses lane-b06:52-54 (contended FAIL), lane-ax-05:68-70 (1.7-1.8x
shared-host delta); G10's flock covers the census court only.
Mechanization: `just court <name>` wrapping latency courts in
flock /tmp/cgap-court-<name>.lock, typed exit 2 if held; W10-DX-C
gains the lock dir to FILE_GRAMMAR; wired via W10-DX-F. Scope S-M.

## (c) Mura — remaining unevenness

C1 (P1) layer qualifiers are doc-discipline only: no lint ties
"ns/frame|ns/call|ns/op" to a co-occurring layer token
(bare|membrane|kernel|containerized). AX-5:88 and IPC-8:39-41 are
prose laws policed by hand. Mechanization: receipt_lint strict rule
+ fixture arm in W10-DX-E's ownership (extend, do not fork). Scope S.
C2 (P3) C01 receipt/census attribution gap — subsumed by A2's ledger.
d-items: EXP attribution pointer file (S); LSPA/MV receipt-landing
check (subsumed by A2); DXD-06's wrong GAP-06-absent errata (S).

## Ranking

| rank | id | scope | why first |
|---|---|---|---|
| 1 | A1 | S-M | all 10 incidents used the uncovered path |
| 2 | A2 | M | 25+ invisible slots; enables A4/C2/d-items |
| 3 | C1 | S | one lint rule kills claim-category drift |
| 4 | A3 | S | 8-9 turns/wave, trivial template+recipe |
| 5 | B1 | S-M | 1.7-1.8x non-reproducibility witness |
| 6 | A4 | M | ~18 turns, partially fenced by DX-C |
| 7 | C2/d | S | bookkeeping, subsumed by A2 |

## Top-3 next-wave

1. A2 dispatch ledger + heartbeat doctor check.
2. A1 write-integrity v2 (Bash/heredoc/python path).
3. C1 layer-qualifier lint arm (cheapest Mura countermeasure).

## See Also

audit/doctrine-friction-census.md · audit/dxd-10-dx-wave-plan.md ·
lanes/lane-dx-02-write-integrity.md · lanes/lane-ax-05-latency.md
