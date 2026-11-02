# W1-12 — Court Corpus Receipt

- Lane: W1-12 (lane-runner), /Users/sac/cgap, 2026-11-01.
- Files written (sole ownership honored):
  - `docs/jira/v26.11.2/07-court-corpus.md`
  - `docs/jira/v26.11.2/waves/w1-12-court-corpus.md` (this file)
- No git commands run. `/Users/sac/stardf` untouched. Shared tree untouched outside
  the two owned files.

## Grounding receipts (file:line re-derivation, this session)

- `crates/cgap-parquet/src/footer.rs:119-127` `fnv1a64_seeded` — read on disk, byte-wise
  FNV loop confirmed (`for &b in data { h ^= b; h *= PRIME }`); wrapper `fnv1a64` :114.
- `crates/cgap-ffi/src/sovereign.rs:83` `CGAP_ERR_P28_SPOLIATION = 0x5032_38FF`;
  refusal sites :299/:307/:315/:384; `std::fs::File::create` at :722 inside the
  :706-730 `cgap_commit_parquet` region — read on disk.
- `crates/cgap-gate/src/penstock.rs:491` `pub fn ingress` — read on disk.
- `crates/cgap-bn3/src/unify.rs` — module docs :11,:35-39 name the nested-loop join
  (v1, mask AND); per-body-atom probe loop :453-455 read on disk.
- `crates/cgap-p28/src/asc450.rs` — sole ASC450 hit across crates/ (grep witnessed).
- `docs/jira/v26.11.1/waves/a02-finish-receipt.md:170-184` mtime-bump restore law —
  read verbatim this session and cited in 07-court-corpus.md section 1 law 3.
- Gate names G1-G8 and per-gate FLIP blocks — read from
  `docs/jira/v26.11.2/01-release-gates.md` (sections G1-PUBLISH:20 ... G8:250,
  session-evidence section 2026-11-01).
- Lever mutations L1-L5 — read from `docs/jira/v26.11.2/02-pareto-levers.md`
  (L1:12 ... L5:108; anti-vacuity mutations at :32, :53, :79, :103, :127).

## Verification

- Markdown law: H1 = `07-court-corpus` (head -1 witnessed); 0 lines > 100 chars
  (`awk 'length > 100' | wc -l` = 0). sha256 of 07-court-corpus.md after final fix:
  see on-disk file; line count 163+ (rewrapped during authoring).
- Standing of the corpus itself: UNVERIFIED as courts — this lane defined the corpus,
  it did not run flips. Court runs belong to wave 2/3 receipts (w2-l*-court,
  w3-g*-court slots named in section 5 of 07-court-corpus.md).

```json
{"lane":"W1-12","files_written":["docs/jira/v26.11.2/07-court-corpus.md","docs/jira/v26.11.2/waves/w1-12-court-corpus.md"],"commands":["grep/ sed reads of footer.rs, sovereign.rs, penstock.rs, unify.rs, asc450 grep, a02-finish-receipt.md, 01-release-gates.md, 02-pareto-levers.md (all exit 0)","awk length>100 sweep = 0 lines"],"courts_run":[],"standing":"ALIVE-for-authoring / courts UNVERIFIED","divergences":["Section-3/5 'kill exit' for absent subjects (G3,G4,G7,G8) records the loud-skip exit from 01-release-gates.md BLOCKED blocks (2 or 3), not a witnessed flip — those subjects are absent on disk today","receipt slots w2-l*-court.md / w3-g*-court.md are named, not yet written; they belong to later waves"]}
```
