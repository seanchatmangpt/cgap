# Lane CL4 — boundary gate witness + negative mutant

Date: 2026-11-02. Lane: CL4, repo /Users/sac/cgap, branch main.
Scope: read-only + /tmp sandbox; sole write = this receipt.

## 1. Clean run (shared tree)

Command: `python3 scripts/boundary_gate.py --deps --report /tmp/cl4-deps-clean.json`
Exit: 0.

Verdict line: `DEPS GATE: E=0 runtime externals across 15 covered crates (§6.4)`.

Covered crates = 15 (E=0 each): cgap-core, cgap-storage, cgap-bont,
cgap-bosym, cgap-bn3, cgap-gate, cgap-crypto, cgap-p28, cgap-court,
audit, cgap-arrow, cgap-blake3, cgap-git, cgap-parquet, ontologies/fibo.
Internal runtime-dep counts observed: storage=1, bont=1, bn3=1, gate=1,
crypto=1, p28=2, audit=2, fibo=1; all workspace-internal (path inside
repo), so E=0 per §6.4.

## 2. Anti-vacuity mutant (sandbox, first-ever --deps flip witness)

Sandbox: /tmp/cl4-sandbox — copy of scripts/boundary_gate.py plus the 15
covered crate dirs (target/ excluded). REPO is derived from the script's
own path, so the sandbox gate audits the sandbox tree.

Sandbox baseline: `--deps` exit 0 (clean copy passes — proves the
injection, not the copy, causes the flip).

Injection (sandbox only):
- file: /tmp/cl4-sandbox/crates/cgap-core/Cargo.toml
- diff: appended
      [dependencies]
      serde = "1"
- pre-mutation sha256 of the shared file:
  418d4c5205f6ff6faf078b741a9c60ec87643880088849a28d2ab77485787bfc

Mutant run: `python3 scripts/boundary_gate.py --deps --report /tmp/cl4-deps-mutant.json`
Exit: 1 (EXIT_REFUSE).

Gate output: `DEPS GATE FAILED: 1 runtime external(s) across the covered
crates (§6.4: E=0)` and `crates/cgap-core  E=1  runtime_deps=1  EXTERNAL: serde`.

The gate FLIPS: exit 0 intact -> exit 1 under the injected external dep.
--deps non-vacuity witnessed for the first time this campaign.

## 3. Restore witness (shared tree untouched)

- shared crates/cgap-core/Cargo.toml sha256 after the sandbox run:
  418d4c5205f6ff6faf078b741a9c60ec87643880088849a28d2ab77485787bfc
  (byte-identical to the pre-mutation hash)
- `git status --porcelain -- scripts/boundary_gate.py
  crates/cgap-core/Cargo.toml`: cgap-core/Cargo.toml NOT listed
  (unmodified). `M scripts/boundary_gate.py` is pre-existing session
  dirt present in the wave's starting git status; this lane never wrote
  it (shared gate sha256
  45254e376a4343f247a72d0138541bf282433febb06643a7c2da658a41510e62,
  on-disk content = the file this lane only read).
- sandbox deleted after witness (build-root lease law).

## 4. Divergences

1. Lane dispatch suggested an "illegal circular dependency" injection.
   The gate does not parse dependency graphs or cycles — it counts
   runtime EXTERNALS per covered crate (scripts/boundary_gate.py
   parse_runtime_deps). The minimal mutant shape the gate actually
   flags is a registry dep (`serde = "1"`); that is what was injected.
   Cycle detection is not a --deps capability (recorded, not
   redefined).
2. /tmp/cl4-deps-clean.json and /tmp/cl4-deps-mutant.json ledgers were
   written under /tmp (lane scratch), not committed.

## 5. Standing

ALIVE — clean exit 0 and mutant flip exit 1 both ran this session,
commands and exits cited above; shared tree sha256-verified untouched.
