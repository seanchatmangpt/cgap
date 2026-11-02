# W2-10: release gate repair (release_check.py)

Date: 2026-11-01. Lane: W2-10. Repo: /Users/sac/cgap (canonical checkout,
branch main). Sole writes: `scripts/release_check.py` +
`docs/jira/v26.11.2/waves/w2-10-release-repair.md`.

## Three witnessed gate-law defects, repaired

1. **Empty-`[dependencies]` law was vacuous-failing on shipped reality.**
   The old law failed every publishable crate with any dependency, but
   cgap-bont depends on cgap-blake3 (`path = "../cgap-blake3"`,
   `version = "26.11.1"`) and is PUBLISHED-26.11.1 on crates.io — the law
   contradicted the registry. Origin: inherited verbatim from an earlier
   zero-dep-kernel era gate (the docstring's "path-only deps fail crates.io
   publish" concern was real but was over-generalized to "must be empty");
   it was never re-derived against the published surface. New law: every
   `[dependencies]` entry of a publishable crate must carry a version pin;
   a path dep must also carry `version` (path+version). Registry
   version-only specs stay lawful. 14 crates previously failed by the old
   law now pass with "all version-pinned".
2. **CHECK 3 `cargo package --list` leg exited 101 with "pass the
   --allow-dirty"** — the second invocation (line 276 old) lacked the flag
   its sibling carried. Repaired: `--allow-dirty` added to the --list
   command for both member and standalone invocations. Disclosed: the flag
   is correct here because the canonical checkout carries uncommitted
   campaign docs by design; the gate constant
   `ALLOW_UNCOMMITTED = True` now records that law.
3. **Member-note logic corrected against `cargo metadata`.** Witnessed
   pre-repair: `cgap-verification` and `cgap-cli` FAILed "not a workspace
   member" while `wasm-footprint` — also disk-only — resolved as OK.
   Verified against `cargo metadata --no-deps` (22 members; none of the
   three are members) and each manifest: all three declare their own
   `[workspace]` table (standalone by law; cli is the ggen six-pack).
   Non-member disk crates are now classified from the manifest: standalone
   (own `[workspace]`) → OK + note; inherited version → OK + note; else
   → FAIL. cli is a standalone non-member by law — recorded, not assumed.

## Additional repair found during run (same defects class)

- **`cargo package -p` used directory names, not manifest package names.**
  Witnessed pre-repair: audit/cloud/fibo/integration exited 101 with
  "package ID specification did not match any packages". Repaired: `-p`
  now uses the manifest's `package.name` (cgap-audit, cgap-cloud,
  cgap-fibo, cgap-integration).
- **Standalone crates cannot be selected with `-p` from the root.**
  wasm-footprint is packaged via `--manifest-path <crate>/Cargo.toml`.
- **Manifests dict was keyed by directory name, not package name** (the
  old key expression `man.get("name", d.name)` read a nonexistent top-level
  key). Keys are now `package.name`.

## Post-repair run (this session)

- `python3 -m py_compile scripts/release_check.py` → exit 0.
- `python3 scripts/release_check.py` → **exit 1**. Full output at
  `/tmp/w2-10-run.txt`. Census:
  - CHECK 1: all 22 members OK; 3 standalone disk crates (cli,
    cgap-verification, wasm-footprint) OK + notes.
  - CHECK 2: 0 dep-law FAILs (was 14). license/repository/description:
    0 FAILs (as expected — none residual).
  - CHECK 3: 19 member crates package exit 0 with artifacts and
    --list exit 0 (was 15 `--list` 101s + 4 wrong-`-p` 101s).
- **Residual FAIL (classified, pre-existing, OUTSIDE lane scope)**:
  `crates/wasm-footprint/Cargo.toml` declares `version.workspace = true`
  inside a standalone crate whose own `[workspace]` lacks
  `[workspace.package].version` — cargo itself refuses to parse the
  manifest ("`workspace.package.version` was not defined", exit 101). The
  gate is correctly reporting a real defect in a manifest this lane does
  not own. Classification: BUILD_BROKEN (manifest), owner: coordinator or
  a wasm-footprint lane; fix is either pin `version = "26.11.2"` literally
  or add `[workspace.package]` to its own workspace root.

## Anti-vacuity court (mutation, /tmp sandbox)

- Method: imported `release_check` from the shared tree, loaded the REAL
  cgap-bont manifest as baseline (no mocks; real function, real data),
  asserted BASELINE fails == [] (clean). Mutation in the in-memory
  manifest copy only (shared tree untouched): injected
  `evil-dep = { path = "../evil-dep" }` (bogus external unversioned dep).
- Result: court FLIPPED — mutated run fails with
  `unversioned dependencies would fail crates.io publish: evil-dep
  (path-only, no version pin)`; also asserted the old vacuous law
  ("[dependencies] not empty") did NOT resurface. Court exit 0.
- Old-law flip check: the pre-repair run (witnessed above, exit 1 with 14
  empty-deps FAILs + 19 exit-101 lines) demonstrates the gate could fail
  before; the post-repair flip demonstrates it still can, on the right
  law.

## Standing

- scripts/release_check.py repairs: ALIVE (commands + exits this session,
  cited above).
- wasm-footprint manifest defect: BLOCKED for this lane (file ownership),
  handed to coordinator with the classification above.
- Gate standing overall: PARTIAL — exit 1 solely on the out-of-scope
  wasm-footprint manifest defect; every in-scope law passes.

## Divergences

- Dispatch said wasm-footprint "resolves" as a disk-only crate; cargo
  metadata and the manifest show it resolves as a STANDALONE crate (own
  `[workspace]`), which the repaired note logic now records. Its packaging
  leg separately surfaced the version-inheritance defect above.
- The receipts directory note: this receipt is the only file this lane
  wrote besides scripts/release_check.py.

