# W-HYGIENE lane receipt — /tmp lease cleanup

Date: 2026-11-01 23:27 PST. Repo: /Users/sac/cgap (read-only; only this receipt written).
Method: python3 census + shutil.rmtree only; no `rm -rf`; no git writes; /Users/sac/stardf untouched.

## Classification law

- KEEP: w3-*/w4-* targets (lanes not landed), a1-a5 targets (excluded by dispatch),
  and any lease with mtime inside the live session (post-12:00 today, owner lane live).
- DELETE: coordinator-owned (census-target, coord-publish-target — instructed),
  landed-lane leases (a01/a06/doc-4/r13/w1/w2 receipts present in waves/), and
  prior-session stale leases (mtime 2026-10-30/31, owning wave terminated: w1-20-terminus,
  w2-20-terminus).

## Deleted (19 leases, 11,315,392,373 bytes = 11.32 GB freed)

| lease | bytes | basis |
|---|---|---|
| /tmp/lane1-n3-target | 6,715,140,019 | stale 10-31 |
| /tmp/census-target | 953,571,162 | coordinator (instructed) |
| /tmp/w2-l14-target | 1,202,616,604 | stale 10-31 / w2 landed |
| /tmp/w1l02-target | 886,271,271 | stale 10-31 / w1 landed |
| /tmp/w1-lane7-target | 335,180,478 | stale 10-31 / w1 landed |
| /tmp/doc4-target | 256,067,489 | doc-4 receipt landed |
| /tmp/purrdf-xsd-court-target | 244,150,589 | stale 10-31 |
| /tmp/purrdf-rev-target | 172,293,210 | stale 10-31 |
| /tmp/r13-target | 128,413,094 | r13-quad64 receipt landed |
| /tmp/stardf-deflate-lane-target | 108,861,216 | stale 10-31 |
| /tmp/stardf-lane-rdfcore-slice-target | 93,365,258 | stale 10-31 |
| /tmp/cgap-audit-A01-target | 53,054,322 | a01 receipt landed |
| /tmp/coord-publish-target | 83,858,678 | coordinator (instructed) |
| /tmp/avec-lane18-target | 35,431,626 | stale 10-30 |
| /tmp/v2-lane-target | 28,392,621 | stale 10-30 |
| /tmp/a6-target | 292,685 | a06 receipt landed |
| /tmp/g3-target | 3,369,202 | stale 10-30 |
| /tmp/l14-target | 13,542,434 | stale 10-31 |
| /tmp/wh6-target | 1,520,415 | stale 10-30 |

## Kept (active / excluded, 18.75 GB remaining campaign-class)

w3-* (l3,l4,r4,r1,t1,t8-cli,t8-court,t8-sd), w4-* (d4,d5,d8,k1,k3,k3-bench,mw2,p1,p2),
a1-a5-target, cd2/cd3-target, r2m/r17/r3-target, l4-court-target, cgap-court-cli-target,
gap-3, c11, doe1, doc-current (p311, vac*, v7, s4, pa6-8, b1/b3/b4, w29, wcp, whandoff,
newcomers), r2m — all mtime within the live session or dispatch-excluded.

## Commands / exits

- census + du: exit 0 (2026-11-01 23:2x)
- python3 rmtree batch (19 dirs): exit 0
- post-delete verification (6 spot checks GONE + remaining-bytes census): exit 0

Standing: ALIVE (all commands run this session against the exact /tmp subjects).
Divergence: "STOPPED this session" not directly observable from /tmp; operationalized as
mtime-stale (pre-session) OR owning-lane receipt landed in waves/. Restores/mtime bumps N/A
(no tree files touched).
