# Lane FIX-05 — boundary_gate.py deps non-vacuity arm (G16)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Files written:
`scripts/boundary_gate.py` (additive), this receipt.

## Audit finding (G16)

`boundary_gate.py --deps` had no non-vacuity arm: the three existing
self-checks (mutation campaign shim-court, strings seam, interference
court) did not cover the deps gate. Poison spec was pre-recorded: append
a `1.0.0` registry dep to a sandbox `cgap-core/Cargo.toml`, expect the
EXTERNAL refusal to fire (exit 0 -> 1).

## Diff

Additive only:

1. `mode_self_check_deps(report=None)` — G16 court. Arms:
   - G (clean): /tmp sandbox copy of the real
     `crates/cgap-core/Cargo.toml` classifies E=0 -> EXIT_PASS.
   - P (poison): same fixture with
     `\n[dependencies]\npoison-registry-dep = "1.0.0"\n` appended; the
     gate's own classifier (`parse_runtime_deps`, unmodified) must mark
     it EXTERNAL and the gate verdict rule (E!=0 -> EXIT_REFUSE) yields
     exit 1 with evidence `EXTERNAL: poison-registry-dep`.
   - Both arms share the real classifier — no second scanner, no mocks;
     assertions on final exit-class state. /tmp-only fixture
     (`tempfile.mkdtemp`, `finally: rmtree`); the shared tree is never
     written.
   - Exit contract mirrors the gate: 0 pass / 1 refuse / 2 subject
     missing (real manifest absent -> loud [SKIP] exit 2) / 3 unused.
   - Ledger mode `self-check-deps` with per-arm exits + evidence.
2. CLI: `--self-check-deps` flag + dispatch (before the deps/strings
   branch) + appended to `CLI_MODES`; imports `shutil`, `tempfile`.
3. Witnessed first-run vacuity catch (the court is not decorative): the
   first implementation appended the bare poison line, which landed
   under the manifest's tail `[dev-dependencies]` section and was NOT
   classified external — the court REFUSED
   (`DEPS SELF-CHECK FAILED (arm P): poison registry dep NOT classified
   EXTERNAL ... VACUOUS GATE`, exit 1) before the fix. Dev-deps are not
   runtime deps; the poison now lands under an explicit runtime
   `[dependencies]` header (commented in the source).

## Commands + exits (this session, exact subject)

```
python3 -m py_compile scripts/boundary_gate.py         # COMPILE-OK
python3 scripts/boundary_gate.py --self-check-deps     # exit 1 (witnessed vacuity refusal, pre-fix append)
python3 scripts/boundary_gate.py --self-check-deps     # exit 0 (both arms flip, post-fix)
python3 scripts/boundary_gate.py --deps                # exit 0 (no regression; E=0 across 15 covered crates)
python3 scripts/boundary_gate.py --self-check-deps --report /tmp/fix05-ledger.json  # exit 0
```

Ledger arms field (read from /tmp/fix05-ledger.json on disk):

```json
{"G_clean": {"exit": 0, "externals": []},
 "P_poison": {"exit": 1, "externals": ["poison-registry-dep"],
              "evidence": "EXTERNAL: poison-registry-dep"}}
```

## Flip transcript (before/after)

- BEFORE (clean fixture, arm G): `E=0`, verdict exit 0 —
  `G clean      cgap-core fixture E=0 -> exit 0  OK`
- AFTER (poison appended, arm P): `EXTERNAL: poison-registry-dep`,
  verdict exit 1 —
  `P poison     registry dep -> EXTERNAL: poison-registry-dep -> exit 1  OK`
- Anti-vacuity mutation: the appended registry dep is the named
  mutation; a classifier change that drops the registry/`version`
  branch leaves the poison un-flipped -> arm P refuses with the
  VACUOUS GATE message (witnessed live, first run).

## Fences (must-not-regress, run)

- `--deps` gate over the shared tree: exit 0 unchanged (15 covered
  crates, E=0). No other arms touched; `--mutate`/`--strings` code
  paths unmodified (imports + one dispatch line + CLI flag only).
- Shared tree: only `scripts/boundary_gate.py` modified by this lane;
  `crates/cgap-core/Cargo.toml` shows `M` in git status but that is
  PRE-EXISTING (present in the session-start git snapshot; this lane
  only read it as the fixture source).

## Standing

ALIVE — every claim above carries a command + exit run this session
against the exact subject. No divergences from the lane doc.
