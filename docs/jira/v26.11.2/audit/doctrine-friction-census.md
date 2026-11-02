# doctrine-friction-census

DOCTRINE-6 operating-overhead census (2026-11-02). Every count from a
command run this session. Repo /Users/sac/cgap.

## (a) Lane census

ls docs/jira/v26.11.2/lanes/ | grep lane- | wc -l -> 65 receipt files
(5571 total receipt lines). Fleet coverage:

| fleet | dispatched | receipts | missing |
|---|---|---|---|
| CSCT L1-14 | 14 | 14 (in waves/) | 0 |
| G1-20 | 20 | 20 | 0 |
| C1-6 | 6 | 5 | C01 |
| B1-8 | 8 | 8 | 0 |
| GIT 1-16 (+2 retries) | 16 | 17 files | 0 |
| FIX 1-7 | 7 | 7 | 0 |
| DX 1-8 | 8 | 6 at census | DX-06/08 then in flight |
| MV 1-8 | 8 | 2 at census | MV-01/02/03/05/06/07 (dedup'd) |
| LSPA 1-6 | 6 | 1 at census | LSPA-2..6 in flight |
| EXP 1-10 | 10 | 0 in lanes/ (dod-* in audit/) | attribution gap |
| SEED/DOCTRINE/AUD | 11 | audit docs | attribution gap |

- 429 deaths CONFIRMED: 2 (GIT-03, GIT-07) — both receipted, both
  needed MANUAL re-dispatch; zero auto-retry existed.
- Silent-death class: ~25 lane-slots had no receipt at census time;
  nothing distinguishes "never dispatched" from "died silently" —
  the detectability gap IS the finding.

## (b) Write-corruption incidents

grep -ilE "corrupt|truncat|mojibake|heredoc|placeholder" lanes/*.md
audit/*.md -> 17 files; verified individually. CONFIRMED: 10 incidents
across 7 lanes (~11 corrupt writes): B01 x3, G08 tail, GIT-11, C05 x2,
SEED-A heredoc, L2 (incl. a CROSS-LANE corruption into mmap_writer.rs,
waves/w3-l2-swar-fnv.md:35,51), L5. All caught by manual re-verify or
compile/court; DX-2's write-integrity hook landed mid-session and does
NOT cover heredoc/python-assembly paths (all 10 incidents used them).

## (c) SLA unblocks + disclosures

grep -> 10 lane receipts; 9 lanes applying unblocks + 1 explicit
non-use. Known collision C05->FIX-07 disclosed-in-both. No second
undisclosed collision found; near-misses (FIX-06 WIRE-PENDING, GIT-16
sibling race) were disclosed — but would have been silent if the
disclosing lane had died (silent-death class feeds this).

## (d) Permission-blocked cleanup

DX-3 --clean reclaimed 2,676,613,013 B (2.49 GiB, 31 dirs, 0
failures). rm-denied disclosures: 9 hits / 6 files. Est. ~18 wasted
turns (2/retry x 9).

## (e) Stale-premise dispatches

Confirmed: G12, G13, G14 (premise false at launch, self-corrected);
G11 partial; GIT-16 partial (sibling race); MV-01..07 dedup'd at
dispatch (5 slots avoided post-ruling, 2 landed). Est. 8-9 wasted
lane-turns pre-dedup.

## Per-class table

| class | incidents | caught by | wasted turns |
|---|---|---|---|
| 429 death + manual re-dispatch | 2 | manual receipt | 2-4 |
| silent death (undetectable) | ~25 slots | nothing | 25+ if dispatched |
| write corruption | 10 / 7 lanes | manual/compile/court | 10-20 |
| SLA unblocks | 9 lanes | disclose-in-both law | ~0 (law worked) |
| rm-denied | 9 / 6 files | manual disclosure | ~18 |
| stale premise / dedup | 8-9 | self-detection | 8-9 |

## Top-3 remaining DX investments

1. 429 backoff/retry automation + death detection (dispatch ledger +
   heartbeat; silent-death becomes a WARN not a census audit).
2. Dispatch-premise validation in the lane template (one grep at
   dispatch time kills the stale class).
3. rm-permission preflight + per-lane stale-binary autodetect;
   extend write-integrity to heredoc/python-assembly write paths.

## See Also

audit/doctrine-ship-plan.md · lanes/lane-dx-03-lease-janitor.md ·
lanes/lane-dx-02-write-integrity.md
