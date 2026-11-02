# Lane C8 — budget gate script hardening (H13)

Repo: /Users/sac/cgap (branch main). Files owned/written:
`scripts/enclave_budget_gate.py`,
`docs/jira/v26.11.2/convergence/lane-08-budget-script-hardening.md`.

## Defects fixed

1. **Missing-snippet KeyError traceback** — `profile_deployment` indexed
   `ENTRY_SNIPPETS[crate]` directly; an unknown subject crate raised an
   unhandled `KeyError` (raw exit 1 traceback). Now: typed
   `[SKIP: no entry snippet for <crate>]` per subject on stderr,
   `DEPLOYMENT SUMMARY ... skipped=N`, and exit `3` (house loud-skip ABI)
   when any subject skips.
2. **Mutant path pollution** — `--deploy-mutant` reused the baseline
   caller crate dir, target dir, package name and ELF path, so any
   post-mutant section sweep of the baseline path measured the mutant.
   Now: `generate_caller_crate`/`build_caller_elf` take a `variant`
   (`caller` default, `mutant`), producing distinct dirs
   (`/tmp/bgp-crate/mutant-<crate>`, `/tmp/bgp-target/mutant-<crate>`)
   and ELF name `ebg-mutant-<crate>` — the baseline artifact path is
   never written by the mutant arm. The mutant arm also uses a typed
   `EXIT_SKIP` return if its subject has no snippet (no traceback).

Mechanics: `--check/--mutate/--self-check/--profile-deployment` modes,
tier table, envelope budget, and measurement method unchanged.

## Exit matrix (run this session, 2026-11-02)

| scenario | command (relative to /Users/sac/cgap) | exit |
|---|---|---|
| H13 missing-snippet (bn3/arrow/ffi) | `python3 scripts/enclave_budget_gate.py --profile-deployment --subject cgap-bn3=tier0 --subject cgap-arrow=tier0 --subject cgap-ffi=tier0` | 3 — three `[SKIP: ...]` stderr lines, no traceback |
| baseline 4-subject | `... --subject cgap-blake3=tier0 --subject cgap-gate=tier0 --subject cgap-p28=tier0 --subject cgap-parquet=tier0` | 0 — envelope 39184/131072 B OK (blake3 35440, gate 606, p28 886, parquet 2252) |
| `--deploy-mutant` (same 4 subjects) | baseline run + `--deploy-mutant` | 0 — `DEPLOY-MUTANT COURT: mutant ELF 66176 B > tier0 budget 49152 B — breach flips (anti-vacuity holds)`; envelope still 39184 |
| `--self-check` | `python3 scripts/enclave_budget_gate.py --self-check` | 0 — clean arm passes, poisoned arm refuses (M arm loud `[SKIP]`, no artifact) |

## Pollution-fix witness (H13 sweep sequence re-run)

- Baseline ELF sha256 before mutant run:
  `64ce2754d143fe2ce5f9e752d9e1d50699082e5bbd6114350ad5937d61fe727e`
  (`ebg-caller-cgap-blake3`).
- After the `--deploy-mutant` run: same sha256 (byte-identical), and the
  mutant ELF exists separately at
  `/tmp/bgp-target/mutant-cgap-blake3/.../ebg-mutant-cgap-blake3`.
- Post-mutant `llvm-size -A` section sweep (via `size_elf_sections`):
  baseline path = 35440 B (baseline, unchanged), mutant path = 66176 B.
  The pre-fix behavior (baseline path measuring the mutant) is dead.

## Gates

- `python3 -m py_compile scripts/enclave_budget_gate.py` → OK.
- `pyright scripts/enclave_budget_gate.py` → 0 errors, 0 warnings.

## Divergences

- Mid-lane regression caught by court: my first variant patch doubled the
  `caller-` prefix in the Cargo package name (`ebg-caller-caller-...`),
  which BLOCKED all four baseline subjects; the baseline court itself
  flipped the defect, fixed by renaming the package template to
  `ebg-%(crate)s`. Recorded per the no-vacuous-court law.
- Standing: ALIVE (all commands + exits above executed this session).
