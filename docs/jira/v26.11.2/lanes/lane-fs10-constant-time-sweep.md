# Lane FS-10 — constant-time straggler sweep

Date: 2026-11-02. Scope: `docs/jira/v26.11.2/` + `README.md`,
pattern `constant-time` (hyphenated), plus variant sweep
`constant time` / `constant_time` (case-insensitive).

## Method

```
grep -rn "constant-time" docs/jira/v26.11.2/ README.md
grep -rni "constant time\|constant_time" docs/jira/v26.11.2/ README.md
```

30 hyphenated hits in 17 files; 1 variant hit
(adversarial/forensic-lane-03.md:65). README.md: 0 hits.

## Classification (per lane contract)

- **QUALIFIED** — carries the 11-measured-claims.md row 29
  (constant-time = UNSUPPORTED) pointer, or is the row-29 demotion
  machinery itself:
  - `12-ard-prd-cgap.md:55-59` (X-04FIX qualifier, both P1 sites)
  - `audit/cgapprd-prd-v2.md:82-91` (X-04FIX qualifier)
  - `04-stardf-closure.md:229`
  - `03-ost-design-around-register.md:151`
  - `dflss/00-dflss-master-dossier.md:28,73`
  - `redteam/red-10-timing-jitter-exploit.md:20-24`
  - `redteam/fix-19-corpus-audit.md:120-121`
  - `redteam/red-20-hardened-survivor-synthesis.md:51,217`
  - `redteam/corrosion/adv-15-claims-boundary-injection.md:73,80`
  - `lanes/lane-dod-06-changelog.md:60`
  - `lanes/lane-rw03-ost-stardf-claims.md:80`
  - `lanes/lane-rw02-claim-rewrite.md:13,28-31`
  - `hardening/lane-11-arrow-swar-validate.md:85-93` (the BR3
    rejection record — the row-29 referent)
- **MECHANISM-DESCRIPTOR** — describing real branchless code
  mechanics, not claiming a timing guarantee (pass with note):
  - `lanes/lane-git-02-pkt-line.md:18` — frozen receipt; "constant-time
    hex validation (all 4 nibbles decoded + validity-masked before any
    branch)" describes the mask-before-branch code shape in
    pkt_line.rs, not a latency claim. Note: if a future reader takes
    it as a timing guarantee, row 29 governs; no edit (frozen receipt,
    prefer record-over-edit).
  - `lanes/lane-rc04-attestation-mac.md:20,72` — frozen receipt;
    "constant-time tag compare" describes the MAC compare mechanics +
    its SHIM-2 mutation. Mechanism, not a latency claim.
  - `explore/branchless-reuse-lane-11.md:40` — "16-stage constant-time
    ladder" describes swar ladder mechanics. Living doc but a
    descriptor; pass with note.
- **NEGATING** (extra class, strengthens rather than claims):
  - `adversarial/forensic-lane-03.md:65` — "...comment is
    per-BUCKET-CHAIN complexity, NOT constant time" — refutes a
    complexity claim; no fix.
- **STRAGGLER: 0.** No living-doc unscoped constant-time claim found;
  no fixes applied; no ERRATA appends required.

## Fences

No code or frozen-receipt bytes touched; only this receipt file
written. Fence: row 29 itself must not regress — re-verified on disk
this session (11-measured-claims.md:53, standing UNSUPPORTED).
