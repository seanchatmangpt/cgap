# Receipt: v2-04-stardf-closure

Lane: V2-STARDF-CLOSURE (lane-runner). Date: 2026-11-01.
Charter: CHARTER-CGAP-STARDF-v26.11.2. Subject: this receipt binds the
spec at `docs/jira/v26.11.2/04-stardf-closure.md` (222 lines, all ≤100
chars, awk-swept this session).

## Files written (complete list)

- `docs/jira/v26.11.2/04-stardf-closure.md` (spec)
- `docs/jira/v26.11.2/waves/v2-04-stardf-closure.md` (this receipt)

No other files touched. No git writes. Spec-only lane: no TTL, script,
workflow, or Rust code was edited — items (a)-(e) are specified for
executing lanes with current-state -> work -> exit-criterion per item.

## Commands run this session (command -> exit)

| command | exit | witness |
|---|---|---|
| `node --check …/workflows/v26.11.1-full-parity.workflow.js` | 1 | Illegal return, line 541 |
| `python3 …/arrow/scripts/verify_stardf_licenses.py` | 1 | [SKIP] register missing; P5 bug live |
| grep census, 11 typedefs in purrdf.h | 0 | all present (lines in spec (a)) |
| `sed -n '70,100p' …/stardf_contract.ttl` + grep | 0 | 13 TypeDefs, TTL:75-147, 5264 lines |
| `sed -n '285,300p' …/stardf-hash/src/dispatch.rs` | 0 | Vec-collect read; 392 UNVERIFIED |
| `awk length>100` sweep, both files | 0 | spec 0; receipt fixed |
| `find …/stardf/docs -name build-baseline*` | 0 hits | ABSENT — named in spec |

(`…/stardf` = `/Users/sac/stardf`; `…/arrow` = `/Users/sac/stardf/arrow`
— paths abbreviated to hold the 100-char line law.)

## Courts run

Spec-only lane; no court was authored or flipped by this lane. The spec
names the required courts and their anti-vacuity mutations per item:
(a) F1 census + `--mutate drop-type` flip; (b) license-gate shim flip;
(c) KD-12/KD-14 differential courts + fixture-byte mutation; (d) node
--check re-flip; (e) no_std gate + std-dep re-introduction flip. All
court runs are the executing lanes' obligation and stand UNVERIFIED.

## Divergences (dispatch vs reality)

1. build-baseline.md — dispatched to read; ABSENT on disk (WORK-BOARD
   confirms S-2 landed-later never happened). Cited as named-absent.
2. verify_stardf_licenses.py "witnessed never-written in either tree" —
   false at execution time: it exists in stardf/arrow/scripts (read);
   what is witnessed broken is its root resolution (parents[3], [SKIP]
   exit 1). Spec item (b) re-scoped to the re-point + SPDX header
   invariant arm + flip criterion.
3. workflow.js path — dispatch said
   `workflows/v26.11.1-full-parity.workflow.js`; actual subject is
   `/Users/sac/stardf/workflows/…` (repo root), not `arrow/workflows/`.
   Line-541 syntax error confirmed regardless (exit 1 witnessed).
4. Charter §5 is "Prohibitions," not a stardf closure section — the
   dispatch's "spec §5" is read as charter gate G2 (no_std split) plus
   the task items (a)-(e); recorded here rather than silently resolved.
5. 392 riscv errors at stardf-hash/src/dispatch.rs:292 — dispatch-
   claimed; file located and region read, probe not re-run this session.
   Marked UNVERIFIED in spec (e); counts are targets until re-run.

## Standing

SPEC-ALIVE / PARTIAL — spec authored and swept from on-disk receipts
plus live witnesses (table above); every item's work and court runs are
UNVERIFIED until an executing lane lands them with command+exit runs.
