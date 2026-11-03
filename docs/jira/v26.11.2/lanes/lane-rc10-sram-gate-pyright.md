# Lane RC-10 — verify_cscb_sram.py pyright `proc` possibly-unbound fix

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Owner files:
`scripts/verify_cscb_sram.py` only (FIX-02 lane complete — safe to edit).

## 1. Subject

Pyright reported `proc` possibly-unbound at the `run_tool` error path
(~lines 117-120 pre-edit): after `except (OSError, TimeoutExpired)` the
handler calls `fail()`, but `fail` was not typed `NoReturn`, so the
checker could not narrow `proc` on the fallthrough.

## 2. Diff

- `import` block: added `from typing import NoReturn`.
- `fail()` and `skip()` annotated `-> NoReturn` (with a comment citing
  lane RC-10). This resolves the narrowing in BOTH `run_tool` and
  `build_caller` (same pattern). No exit codes, refusal strings, charge
  arithmetic, or court logic touched.

## 3. Commands + exits (this session, exact subject)

- `python3 -m py_compile scripts/verify_cscb_sram.py` → OK.
- `pyright scripts/verify_cscb_sram.py` → "0 errors, 0 warnings,
  0 informations", exit 0. (pyflakes unavailable in this env —
  `No module named pyflakes`; pyright IS available and clean.)
- Baseline `python3 scripts/verify_cscb_sram.py` → exit 0, measured
  36048 B, headroom 95024 B — identical to FIX-02's receipt.
- `--subject bn3` → exit 1 with the exact `BN3_ENCLAVE_REFUSED`
  refusal line (6520984 B, 49.75x ceiling) — unchanged.
- `--mutate bn3-pin` → exit 0, witnessed flip: "mutant refused (exit 1):
  DIVERGED ... NaMatches 1081352 B != pinned 2162696 B".
- `--mutate sram-bloat` → exit 0, witnessed flip: "mutant measured
  136068 B > 131072 B — overflow flips (anti-vacuity holds)".

## 4. Anti-vacuity ledger

Behavior-preserving fix, so anti-vacuity IS the four-run battery: any
drift in exit code or refusal text would falsify the diff. All four legs
match FIX-02's landed receipt byte-for-byte in outcome. Both mutate
courts flip (witnessed lines above) — the fix did not deaden either.

## 5. Standing

ALIVE. Every claim cites a command+exit run this session against the
exact subject.

## 6. Fences (must-not-regress)

- Baseline B04 gate (36048 B / exit 0) — rerun post-edit, unchanged.
- bn3 typed refusal (exit 1, exact string) — rerun, unchanged.
- Both anti-vacuity courts — rerun, both flip.
- House exit ABI (0/1/2/3) untouched; no git commands; no lib.rs/
  Cargo.toml touched; build leases /tmp/cscb-sram-{target,caller} are
  FIX-02's, reused, not re-created.

## 7. Divergences

- Dispatch offered a "typed None-guard with loud message" alternative;
  the NoReturn annotation is the minimal equivalent (narrowing without
  any new branch), chosen so the error-path behavior is provably
  unchanged. pyflakes is not installed in this environment; pyright
  0-error output substitutes per the dispatch's fallback clause.
