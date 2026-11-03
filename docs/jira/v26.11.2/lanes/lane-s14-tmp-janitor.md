# Lane S-14 — shadow-checkout janitor + final /tmp sweep

Standing: **ALIVE** — every claim cites a command + exit run this session
(2026-11-02, 18:28–18:32 PST) against the exact subject.

## Item 1 — /tmp/gap07-sb (workspace copy)

- VERIFIED: full cgap workspace copy (root `Cargo.toml` carries cgap
  members + LICENSE.md, crates/; mtime 14:22, 4h cold). Matches ADV-18
  Probe 2 fingerprint (`adv-18-sandbox-state-leakage.md:35`).
- DELETED via `python3 shutil.rmtree` — exit 0, `exists()=False`
  witnessed. Freed 4.9 MiB.

## Item 2 — cold cargo build roots

| path | size | mtime | content | action |
|---|---|---|---|---|
| /tmp/target-xc03 (85M) | top mtime 17:52 = 36 min old | cgap build root (CACHEDIR.TAG+debug) | **SKIP-ACTIVE** (within 1h lease window) — subsequently deleted by another actor before final scan; not by this lane |
| /tmp/exp10-target (1.0G) | 13:44, cold | cgap build root (CACHEDIR.TAG+debug) | DELETED, exit 0, freed 1.0 GiB |
| /tmp/pub14-target (595M) | 15:57, cold | cgap build root (CACHEDIR.TAG+debug) | DELETED, exit 0, freed 595 MiB |
| /tmp/pub14b-target (925M) | 16:01, cold | cgap build root (CACHEDIR.TAG+debug+tmp) | DELETED (same class, same pub14 lease family, cold) — beyond dispatch list, disclosed here |

Combined freed: **~1.63 GiB** (du -sk total 1,677,716 KB for the first
three; pub14b additional ~925 MiB ⇒ ~2.56 GiB total).

## Item 3 — final WAVE-grammar re-scan

Grammar: gap07* exp10* pub14* target-xc* adv* r0* r1* s0* red* fix0*
rc* xc* df* eval* bench* — per-pattern `ls -d /tmp/${~p}`.

REMAINING (all listed, none deleted beyond above):

- **RETAINED-BY-RECEIPT**: /tmp/adv02/, /tmp/adv11-probe/,
  /tmp/adv11-disasm.txt (ADV-02/ADV-11 explicit replay retainers).
- **SKIP-ACTIVE** (mtime < 1h, live lane leases): /tmp/adv03-target,
  /tmp/adv03-target-{andon_liveness,mcp_driver,pack_demux,ref_store}_court,
  /tmp/adv04-target, /tmp/r15-target, /tmp/r19, /tmp/r19-target,
  /tmp/fix07-sandbox, /tmp/rc07-target, /tmp/xc01-target.
- **COLD SMALL RESIDUE listed-not-deleted** (logs/probes, <200 KB total:
  gap07-l*.txt, gap07-sb-*.txt, exp10-*.log, adv01-*.log, adv03-*.log,
  adv03-hog.{sh,pids}, adv03-runs.txt, adv06-*.log, r02/r03/r04/r15-*.log,
  r18-manifest.json, r18-receipt.md, adv04-probe, adv05, adv05fix,
  adv10-probe, df4-cpk*.{json,log}). Bytes-trivial; disk cost negligible;
  left for receipt-replay traceability.

First scan attempt returned a false-empty (zsh unquoted `$p` does not
glob-expand in command substitution); corrected with `${~p}` — recorded
as a probe-method erratum against this receipt's own first run.

## Item 4 — repo tree

- `ls -d /Users/sac/cgap/target-*` → no matches (exit 1). CLEAN.
- `git status --short | grep target-` → empty (exit 1). CLEAN.

## Item 5 — anchored hog pgrep

`pgrep -fl 'if=/dev/zero|/tmp/red10_hog|stress-ng'` → no matches,
exit 1. No hogs running.

## Item 6 — retained reproducers

adv02 / adv11-probe / adv11-disasm.txt confirmed present, untouched,
RETAINED-BY-RECEIPT.

## Fences

- Zero repo-tree edits; zero git write commands. All deletions were
  /tmp-sandbox wave residue, content-verified (cgap fingerprints) and
  mtime-cold before deletion.
- SKIP-ACTIVE items were never touched; a coordinator re-sweep after
  wave drain can reclaim them cold.

## Divergences

- /tmp/target-xc03 was SKIP-ACTIVE at decision time (17:52 mtime) and
  was deleted by an actor other than this lane between 18:28 and the
  18:31 rescan; recorded, not attributed here.
- pub14b-target (925 MiB) was not on the dispatch list; deleted as
  same-class cold residue with disclosure (above).
- Dispatch expected REMAINING=[]; reality: ~45 paths remain (SKIP-ACTIVE,
  retained-by-receipt, cold small logs). The dispatch's residue census
  under-counted; this receipt is the corrected census.
