# Lane G-16 / C-09 — final /tmp WAVE-grammar sweep (closure gate)

Standing: **ALIVE** — every claim cites a command + exit run this session
(2026-11-02, evening) against the exact subject. Contract: filesystem
sweep only, no repo-tree edits, no git writes, fail-closed on ambiguity
(RETAIN + record).

## Inputs read first (per dispatch)

- `docs/jira/v26.11.2/lanes/lane-s14-tmp-janitor.md` (S-14 grammar,
  SKIP-ACTIVE law, ${~p} probe erratum).
- `docs/jira/v26.11.2/audit/dod-master-workspace.md` (E-13 findings:
  exp10-target build root, bench compile blockers, warning census).
- `docs/jira/v26.11.2/dod/retained-evidence-ledger.md` (DOD-14:
  digest-recorded retainers — adv02, adv11-probe, adv11-disasm.txt,
  df4-cpk*).

## Method

Python census over `os.listdir('/tmp')` (8,034 entries) against the
extended WAVE-grammar pattern list (29 patterns: f01* f02* ocel*
bench01* bench02* bench03* adv10fix* adv14* s04b* pub01* pub02* pub03*
dod0* x0* r0* r1* s0* adv* fix0* rc* xc* df* red* eval* a07* gap07*
exp10* pub14* target-xc*). Exclusions applied BEFORE deletion: the four
RETAINED-BY-RECEIPT families. Age = time - mtime; mtime < 2h ⇒
SKIP-ACTIVE (live lane lease, never deleted; re-verified cold per-item
immediately before each deletion — race check logged).

Census: 211 matched / 45 COLD / 166 ACTIVE. 45 cold entries deleted via
shutil (dirs rmtree, files remove), exit paths all verified
`exists()=False`. **Freed ~83.9 MiB (85,944 KB du -sk sum).**

## Deletions (45, all DELETED, 0 failed)

target-xc03, dod0{1,2,3,5}-* logs/probes (18 files), gap07-l*/sb-*
court logs (17 files), exp10-build{,2,3}.log, dod01-sandbox.log,
dod05-rv.log, dod03-probe-{on,off}. Notable: /tmp/target-xc03 (the
S-14 SKIP-ACTIVE divergence item) was cold at sweep time and reclaimed
here.

## Post-sweep rescan: cold-remaining = 0

Rescan with the same pattern list after deletion: 166 remaining, ALL
SKIP-ACTIVE (mtime < 2h; B-07/B-08/A-lane leases — r14-leg1-target,
bench01-work, pub02-*.json, r15-*, adv03-*, x03/x05-*, dod01b-*, etc.).
Cold residue is empty. Full census at /tmp/g16-sweep/remaining.json
(lane workspace, ephemeral by design).

## Retained-by-receipt check (fail-closed)

- /tmp/adv02 — PRESENT, untouched.
- /tmp/adv11-disasm.txt — PRESENT, untouched.
- /tmp/df4-cpk{,-inject}.{json,log} — all PRESENT, untouched.
- **/tmp/adv11-probe/ — ABSENT at check time. NOT deleted by this lane**
  (it was in the exclusion set in both census and deletion passes; the
  deletion log contains no such entry). Bytes are digest-recorded in
  `dod/retained-evidence-ledger.md` (Cargo.lock/Cargo.toml/
  rust-toolchain.toml/src/lib.rs, sha256 each), so audit reproducibility
  survives per the ledger's digest-recorded rationale. Recorded as an
  external-actor divergence below.

## Anchored hog pgrep

`pgrep -fl 'if=/dev/zero|red10_hog|stress-ng'` → no matches, rc=1. No
hogs running.

## Repo tree

- `ls -d /Users/sac/cgap/target-*` → no matches (rc=1). CLEAN.
- `git status --short | grep target-` → empty (rc=1). CLEAN.
- Pre-existing known item (NOT touched, outside /tmp-only grant):
  `/Users/sac/cgap/ontologies/fibo/target-a09` (~147 MiB cold cargo
  build-root lease inside the checkout), already flagged by
  `dod/retained-evidence-ledger.md` — coordinator action, recorded here.

## Fences

- Zero repo-tree edits; zero git write commands. Only /tmp entries
  matching the extended grammar, cold (>=2h), non-retained were
  deleted; every deletion re-verified cold immediately prior.
- SKIP-ACTIVE (166) never touched — a later re-sweep after wave drain
  reclaims them.
- Anti-vacuity of the sweep itself: the census and deletion used the
  same pattern list; the deletion pass re-checked mtime per item, so a
  mis-classified active lease would have been SKIP-RACE, not deleted
  (0 such events).

## Divergences

1. /tmp/adv11-probe/ (RETAINED-BY-RECEIPT) found absent at closure
   check; not this lane's deletion (exclusion applied both passes; no
   log entry). Digest-recorded in the DOD-14 ledger, so the loss is
   reproducibility-safe; coordinator may restore from digests if the
   bytes reappear nowhere.
2. Dispatch expected REMAINING=[]; reality: 166 SKIP-ACTIVE leases
   remain (mtime < 2h). Cold-remaining IS empty, which satisfies the
   sweep's closure intent; SKIP-ACTIVE is by-contract, never a defect.
