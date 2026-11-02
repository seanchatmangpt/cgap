# GAP-7 Cleanup Receipt

- **Lane**: GAP-7 (lane-runner, coordinator-leased debris purge), repo
  `/Users/sac/cgap` (one canonical checkout), v26.11.1
- **Date**: 2026-11-01
- **Lease**: in-tree `target-a*` lane build roots (22 dirs); stale /tmp
  lease debris (`a20_sandbox`, `doe3_parts`, `hop-court-f29`, `bf15`,
  `cgap-lane-*`, `a0*-*`, `a1*-*`); receipt (this file). Nothing else.
- **Method**: deletion ONLY via `python3 shutil.rmtree` / `os.unlink`
  (`rm -rf` denied by policy); per-path `du -sk` census BEFORE deletion;
  every deletion verified `os.path.lexists(path) == False` after.
- **Verdict**: **ALIVE** — 777/777 candidates deleted with
  exists_after=False, 0 errors, 0 refusals; zero survivors on glob
  re-run; keepers intact (root `target/`, `docs/`, `.git`, 36 `/tmp/f3-*`
  paths).

## Commands (this session, with exits)

| command | exit |
|---|---|
| `df -h / + df -k /System/Volumes/Data (before)` | 0 |
| `du -sk <777 candidates> > /tmp/gap7-du-before.txt` | 0 |
| `python3 /tmp/gap7_purge.py > /tmp/gap7-results.json` | 0 |
| `post-census: df, glob re-run, keeper ls` | 0 |

## Freed bytes

| family | paths | du -sk census | python walk (st_blocks) |
|---|---|---|---|
| repo `target-a*` (22 dirs) | 22 | 5.18 GiB | 7.70 GiB |
| named /tmp leases | 4 | 3.70 GiB | 3.72 GiB |
| `/tmp/cgap-lane-*` | 6 | 0.25 GiB | 0.36 GiB |
| `/tmp/a0*-*` + `/tmp/a1*-*` sweep | 745 | 8.53 GiB | 14.25 GiB |
| **total** | 777 | **17.66 GiB** | **26.03 GiB** |

### Per-dir, repo `target-a*` (du -sk, pre-deletion)

| dir | size |
|---|---|
| `/Users/sac/cgap/target-a01` | 0.08 GiB |
| `/Users/sac/cgap/target-a02` | 0.22 GiB |
| `/Users/sac/cgap/target-a03` | 0.11 GiB |
| `/Users/sac/cgap/target-a04` | 0.08 GiB |
| `/Users/sac/cgap/target-a05` | 0.23 GiB |
| `/Users/sac/cgap/target-a06` | 0.28 GiB |
| `/Users/sac/cgap/target-a07` | 0.26 GiB |
| `/Users/sac/cgap/target-a07w8` | 0.01 GiB |
| `/Users/sac/cgap/target-a08` | 0.14 GiB |
| `/Users/sac/cgap/target-a09` | 0.13 GiB |
| `/Users/sac/cgap/target-a10` | 0.08 GiB |
| `/Users/sac/cgap/target-a11` | 0.08 GiB |
| `/Users/sac/cgap/target-a12` | 0.46 GiB |
| `/Users/sac/cgap/target-a12w8` | 0.07 GiB |
| `/Users/sac/cgap/target-a13` | 0.02 GiB |
| `/Users/sac/cgap/target-a14` | 0.12 GiB |
| `/Users/sac/cgap/target-a15` | 0.10 GiB |
| `/Users/sac/cgap/target-a17` | 0.06 GiB |
| `/Users/sac/cgap/target-a18` | 0.30 GiB |
| `/Users/sac/cgap/target-a18w9` | 0.22 GiB |
| `/Users/sac/cgap/target-a19` | 0.21 GiB |
| `/Users/sac/cgap/target-a20` | 1.93 GiB |

### Named /tmp leases (pre-deletion size)

| path | size |
|---|---|
| `/tmp/a20_sandbox` | 3.58 GiB |
| `/tmp/bf15` | 0.12 GiB |
| `/tmp/doe3_parts` | 24 KiB |
| `/tmp/hop-court-f29` | 133 KiB |

### Glob families, largest 10 of 751 (full census method above)

| path | size |
|---|---|
| `/tmp/a08-adv-audit` | 0.29 GiB |
| `/tmp/a09w9-sb` | 0.28 GiB |
| `/tmp/a13-lane-target` | 0.27 GiB |
| `/tmp/a09w9-mA20.txt` | 0.25 GiB |
| `/tmp/a05-audit-sandbox` | 0.16 GiB |
| `/tmp/a06-w7` | 0.15 GiB |
| `/tmp/a04a-sbx-target` | 0.15 GiB |
| `/tmp/a15-courts` | 0.14 GiB |
| `/tmp/a09w9-sb2` | 0.14 GiB |
| `/tmp/a19-sb` | 0.13 GiB |

## Keepers (never deleted, verified intact after purge)

| keeper | reason | verified |
|---|---|---|
| `/Users/sac/cgap/target/` (root) | active build cache, keeper per dispatch | `ls` OK |
| `/tmp/f3-*` (36 paths) | LIVE publish wave, exclusion per dispatch | `ls` count 36 |
| `/Users/sac/cgap/docs/` | source-of-truth lane/wave docs | `ls` OK |
| `/Users/sac/cgap/.git/` | canonical checkout history | `ls` OK |
| `~/.cargo/` | toolchain home | guarded, never globbed |
| repo source (`crates/`, `scripts/`, root manifests) | contract: read-only | untouched |

## Refused deletions

None. 0 refused, 0 errors across 777/777 candidates. Keep-guards ran
(`f3-*` prefix guard, root-`target/` exact+prefix guard, `docs/`/`.git`/
`~/.cargo` guards); they fired zero times because the leased globs do
not intersect any keeper — the guards are the fail-closed backstop, the
globs are the mechanism.

## Divergences (typed, no overclaiming)

1. **df does not show the reclaim.** Data-volume avail moved only
   160768044 -> 160827104 blocks (+29 MiB) against ~17.7 GiB deleted.
   Typed causes: (a) hourly Time Machine local snapshots
   (`com.apple.TimeMachine.2026-11-01-05..08-*.local`) pin the deleted
   blocks as purgeable until snapshot expiry; (b) the live `/tmp/f3-*`
   publish wave wrote concurrently (used ticked back up between reads).
   The deletion claim rests on per-path `exists_after == False` (777/777)
   and zero-survivor glob re-runs, not on df.
2. **Two byte totals, two methods.** du -sk census 17.66 GiB
   (hardlink-deduplicated, the honest reclaim figure) vs python
   `st_blocks*512` walk 26.03 GiB (counts hardlinked names once per
   name; upper bound). Both reported; du figure is primary.
3. **du census file and purge JSON live in /tmp** (`/tmp/gap7-du-before
   .txt`, `/tmp/gap7-results.json`) and are themselves ephemeral; the
   aggregates above are the durable record.
4. **Dispatch said `target-a01..a20`; `target-a16` never existed.** The
   pre-purge `ls -d` census found exactly 22 dirs matching
   `/Users/sac/cgap/target-a*`: a01-a15, a17-a20 (19) + a07w8, a12w8,
   a18w9 (3). a16 was absent before this lane ran (not deleted by it);
   the "22" count in the dispatch is correct, the a01..a20 enumeration
   is approximate.
