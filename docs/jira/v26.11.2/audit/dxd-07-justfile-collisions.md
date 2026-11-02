# dxd-07-justfile-collisions

DXD-07 justfile collision audit (2026-11-02). Scope: /Users/sac/cgap/justfile
(94 lines, single file; no import/mod directives — no modules to inventory).

## (a) Parse verification

- `just --list` -> exit 0 (13 recipes).
- `just --evaluate` -> exit 0; sole variable CENSUS_ARGS := "".
- Parses cleanly today; DX-1's duplicate doctor-agents removal held.
  Residue: duplicate comment block at :81-82 (cosmetic only).

## (b) Variable-name collisions

Sole variable CENSUS_ARGS (:91, export) — correctly census-prefixed.
No collisions, no unprefixed names.

## (c) Recipe-name collisions vs interop-* namespace

Fresh list (13): default, doctor, verify, bench, courts, branchless,
receipts, dod, status, leases, doctor-agents, census (+default). No
existing recipe starts with interop-; the interop-chicago Go module has
zero justfile wiring. Insertion collision-free.

## (d) Shebang-path law

Zero shebang recipes; no ROOT/justfile_directory() usage — the ROOT :=
parent_directory(justfile_directory()) law is currently vacuous. Apply
it from the first interop recipe onward if shebang form is used.

## Collision-free insertion points

1. :66/67 — after receipts, before dod.
2. :74/75 — after status, before leases.
3. :79/80 — after leases, before doctor-agents.
4. EOF after :93 (census last) — RECOMMENDED: zero-shift insertion,
   no renumber risk, audit trail adjacent to the newest recipe.

## Witnessed failure modes as named anti-patterns

### Anti-pattern 1: Duplicate Recipe Parsing Abort (DX-1 / lane-dx-03)

A lane pastes a recipe without reading current state -> second
definition of an existing name -> `just` fails parsing the WHOLE file,
every recipe down for every lane until removal. Law: run `just --list`
and diff against the target name before inserting.

### Anti-pattern 2: Recipe-Param Leak Into Command Form (DX-8)

`census ARGS='...':` drafted with {{ARGS}} in the body but no declared
parameter -> parse/run error or silent-empty substitution. Shipped fix
is the exported-variable form (justfile :91-93): export CENSUS_ARGS :=
'' then {{CENSUS_ARGS}} in the body. Law: {{NAME}} in a body must
resolve to a declared parameter or an explicitly declared file-level
variable.

## Verdict

Parses (exit 0 both probes); zero variable collisions; zero recipe-name
collisions with interop-*; ROOT law prospective-only. All interop-*
recipes land as one contiguous block at EOF.

## See Also

audit/dxd-01-justfile-census.md · audit/dxd-08-janitor-design.md
