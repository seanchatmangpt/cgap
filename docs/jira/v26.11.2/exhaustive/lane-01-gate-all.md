# EX1 — exhaustive census: cgap-gate (lane-01-gate-all)

Date 2026-11-02. Read-only; zero builds. find census = 3 files, all covered.
Dispatch-named ingress.rs / header.rs / frame.rs: ABSENT (find over
crates/cgap-gate returns exactly src/{lib,penstock}.rs +
tests/lever_courts.rs). BR1 cite:
`explore/branchless-reuse-lane-01.md` (S1-S9) + MASTER-MAP rows 22-29.
Reads this session (all exit 0): find census, wc -l, sed/grep of the three
files, BR1 lane map, BR12 master map, gate Cargo.toml.

Coverage: 3/3 files (find wc -l = 3; matches table below).

| file | lines | fns | branch sites (grep if/match/else/while/for/?/unwrap/panic/expect) | hot/cold |
|---|---|---|---|---|
| src/lib.rs | 14 | 0 fns (module decls only) | 0 | n/a — KEEP, nothing to census |
| src/penstock.rs | 1869 | 33 (7 src + 26 test-module) | 88 total; src region 515-627 carries 15 `if`-class sites | mixed: src hot ingress + inline helpers; test module cold |
| tests/lever_courts.rs | 153 | 4 (3 courts + helper) | 3 | cold (test-only) |

## src/penstock.rs — function inventory (src region, pre-#[cfg(test)])

- `ingress` :515 — hot entrypoint. 15 branch sites, classified below.
- `read_u64_le` :628 — byte loop, KEEP (codegen-note :620-626: slice forms
  measured 6.3 vs 4.9 ns/frame; BR1 S9/Mutant E pins this).
- `read_u16_le` :641 — 2 fixed reads, KEEP.
- `is_zero_at` :651 — AS-OF-IN-FLIGHT (REP5 landed): now delegates to
  `cgap_branchless::penstock::swar_all_zero_vl`; local early-exit loop GONE.
  This consumed BR1 S2+S5. Differential court named in comment
  (all_zero_differential_vs_shipped).
- `prefilter8` :676 — AS-OF-IN-FLIGHT (REP5 landed): delegates to
  `cgap_branchless::penstock::penstock_prefilter8`. Consumed BR1 S6+S7
  (mask+test stays in `ingress` :615). Mutant D still applies (corrupt mix
  constant proves callee identity).
- ABI mirror structs :332/:355 (offset_of! pins) + FrameView :466 — no
  branches; layout pins, KEEP.

## ingress :515 branch-site classification (S-codes = BR1 S1-S9)

| line | site | hot/cold | verdict |
|---|---|---|---|
| 524 | `len < HEADER_LEN` -> INVALID_MAGIC | COLD typed-refusal | KEEP (BR1 MUST-STAY) |
| 530 | `len > bytes.len()` geometry | COLD typed-refusal | KEEP (MUST-STAY) |
| 536 | magic compare | COLD typed-refusal | KEEP; S1 swar_eq fold = candidate only if fused with S6's single-load — [ANALYTIC], BR12 row 1 |
| 541 | version compare | COLD typed-refusal | KEEP; S1 fold companion |
| 550-558 | digest all-zero scan | was-hot, CONSUMED by REP5 (see is_zero_at) — the row is `if digest_all_zero` now, verdict branch COLD | KEEP as-is; Mutant B court owns the callee |
| 566 | declared > bound | COLD typed-refusal | KEEP (S3 leave-alone, ~0) |
| 573 | attestation magic slice eq | COLD typed-refusal | S4 candidate (swar fold with version word) [ANALYTIC]; KEEP until implementing lane |
| 577 | att version | COLD typed-refusal | KEEP |
| 582 | reserved-bits mask | COLD typed-refusal | KEEP |
| 587 | lineage==0 || popcount!=1 | COLD typed-refusal | KEEP |
| 591 | ceiling bound | COLD typed-live-callee-of-court | KEEP (lever_courts l1_ceiling_law_flip named mutant) |
| 597-598 | receipt-zero xor law + branch | was-hot loop, now REP5 callee + COLD refusal branch | KEEP |
| 606 | `if let Some(view)` | per-frame API-shape branch | KEEP (BR1 §MUST-STAY :606) |
| 614 | `if view.prefilter = prefilter8` (admit-side) | HOT | AS-OF-IN-FLIGHT — REP5 delegation live |
| 615 | escalate mask+test | HOT, already branchless (poison_mask class) | KEEP (S7) |

## src/penstock.rs — `#[cfg(test)]` module :700-1869 (26 fns)

Cold, test-only; out of substitution scope per BR1 ("courts module :700+
test-only, out of scope"). Notable: CountingAlloc/SplitMix64 real
collaborators, DAM corpus court :1399, ratio fence :1724, no_std-hygiene
court :1862. Zero substitution candidates; 26 fns KEEP.

## tests/lever_courts.rs (3 courts)

- :47 l1_attestation_byte_flip_escalates_to_spoliation — named mutation,
  real bytes; KEEP.
- :83 ceiling-law flip = the named mutant; KEEP.
- :106 escalation-policy court; KEEP. No gaps found; no new sites missed.

## Verdict

- 15 ingress sites: 12 COLD typed-refusal KEEP (lawful branches), 1 API-shape
  KEEP, 2 AS-OF-IN-FLIGHT (REP5 owns; landed on tree via cgap-branchless
  dep, gate Cargo.toml:16).
- Substitution candidates BR1 missed: NONE in src. Additions recorded:
  (a) gate Cargo.toml branchless dep is landed (BR1 pre-dated it — divergence
  noted); (BR1 S1/S4 swar_eq fusions remain the only open [ANALYTIC]
  candidates, both cold-refusal branches — low value, correctly deferred.
- 100% file coverage asserted: find count 3 == table rows 3.

## Receipt tail (exits 0 unless noted)

find (3 files) 0; wc -l 0; sed/grep reads 0; BR1 + MASTER-MAP reads 0;
gate Cargo.toml grep 0. No builds, no mutations, shared tree untouched.
Standing: census complete, ALIVE as a read-only census; every substitution
verdict inherits BR1's UNVERIFIED/[ANALYTIC] standing.
