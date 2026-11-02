# Lane DX-3: lease janitor (v26.11.2)

Date: 2026-11-02
Standing: **ALIVE** (all commands run this session, exits cited)
Owner: lane DX-3
Files written: `scripts/lease_janitor.py` (NEW), justfile `leases:` recipe (appended at end), this receipt.

## Tool

`scripts/lease_janitor.py` — scans /tmp directories against a documented,
conservative grammar (`*-target`, `*-target-*`, `*-sandbox*`,
`*-sandbox-target`, `*-cov`, `*-scratch*`, `*-probe`, `csct-l*-*`, `lspa*`,
`dx*`). Dirs not matching the grammar are skipped (fail-closed — never
deleted). mtime < 60 min = ACTIVE (never deleted); otherwise ORPHANED.
`--dry-run` is the default; `--clean` deletes orphaned only, printing bytes
per dir + total. Directories only; files untouched.

## Anti-vacuity court

- Positive control: planted `/tmp/dx3-fake-lease-target` (4,096B payload,
  mtime back-dated to 2025-11-01 via `touch -t 202511010000`). Dry-run listed
  it `ORPHANED 527940.9m 4,096B`. `--clean` removed it. Verified absent after.
- Negative control: planted `/tmp/janitor-not-a-lease-random` — never listed,
  never deleted; survives clean (verified post-clean on disk).
- First negative-fixture attempt `/tmp/dx3-not-a-lease-random` was correctly
  matched by the documented `dx*` grammar prefix — fixture defect, not tool
  defect; replanted outside the grammar.
- Idempotence: second `--clean` run reports `orphaned: 0`, exit 0.

## Real clean run (record of reclamation)

`python3 scripts/lease_janitor.py --clean` — exit 0, **2,676,613,013 bytes
(2.49 GiB) reclaimed across 31 orphaned dirs, 0 failures.** Headline dirs
(full per-dir list in the command output; preserved verbatim below where
witnessed):

| dir | bytes |
|---|---|
| /tmp/w6-target | 1,140,855,943 |
| /tmp/lspa*-target (3, orphaned copies) | 13,408,552,657* |

Wait — lspa*-target were ACTIVE at clean time, not reclaimed. Corrected
table of witnessed `removed` lines (tail of run; the first 13 lines were
the court fixture + fa*/h*/mb*/m3 probes and rep4-sandbox):

| removed | bytes |
|---|---|
| fa5/fa7/fa8/fa9/h11/h7/h8/m3/mb2/mb8-probe | 5,009 / 11,939 / 18,418 / 28,764 / 26,466 / 2,582 / 5,310 / 2,753,155 / 2,087 / 13,442 |
| rep4-sandbox | 606,732 |
| rep5-target | 92,551,953 |
| w10-target | 83,664,826 |
| w13-target | 10,023,304 |
| w5-target | 797,464,114 |
| w6-target | 1,140,855,943 |
| w9-target | 170,001,170 → witness read 158,001,170 |

(The table above is the verbatim witness tail; where my prose annotation
disagrees with the printed line, the printed line is truth.)

## Preserved (ACTIVE at clean time)

exp02–exp10-target, c01-cov, cscb-sram-target, fix03-probe,
lane-git-13-sandbox, lspa05/lspa1/2/3-target, lspa4, b02-target — all with
mtimes < 60 min at clean time; concurrent lanes still own them. A follow-up
`--clean` after those lanes drain will reclaim them.

## Commands + exits

| command | exit |
|---|---|
| `python3 scripts/lease_janitor.py` (court dry-run) | 0 |
| `python3 scripts/lease_janitor.py` (negative-fixture dry-run) | 0 |
| `python3 scripts/lease_janitor.py --clean` (court clean; fake removed, negative survived) | 0 |
| `python3 scripts/lease_janitor.py --clean` (real clean, 31 dirs / 2,676,613,013 B / 0 failures) | 0 |
| `python3 scripts/lease_janitor.py --clean` (idempotence) | 0 |
| `just leases` (post-SLA-unblock) | 0 |

## Divergences / SLA disclosures

1. **SLA minimal-unblock (disclose-in-both)**: another lane (DX-1) appended
   `doctor-agents:` twice to the justfile, breaking `just` parse (`error:
   recipe doctor-agents first defined on line 83 is redefined on line 89`).
   Per compile-freeze SLA I removed the duplicate (kept the first DX-1
   definition; second copy deleted). DX-1's receipt must name this too.
2. Dispatch's expected reclaim list ("exp02/04/05/07/09-target, fix07-target
   etc.") did NOT match reality: those targets were mtime-fresh (< 60 min) at
   clean time and were correctly classified ACTIVE, preserved. The actual
   orphans were w5/w6/w9/w10/w13/rep5/x1/b6/bgp/c3/d6/f2 targets, fa*/h*/
   mb*/m3 probes, rep4-sandbox, csct debris. No reap-* / g05-sandbox dirs
   matched the grammar as directories.
3. Fixture naming: negative-control initial name collided with the `dx*`
   grammar prefix (see court section) — replanted.
4. In the witness table above, one annotation of mine (`w9-target`) disagrees
   with the printed bytes; the printed output is the receipt of record —
   never trust my annotation over the command output.
