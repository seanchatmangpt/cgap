# Lane ADV-06 — standalone crate hermeticity escape (corrosion court)

- **Lane**: ADV-06, read-only over `crates/` (probes in /tmp copies);
  only this receipt written. No git commands.
- **Repo**: /Users/sac/cgap, branch main.
- **Subject**: RC-13's `crates/cgap-fleet-ops` + standalone `crates/cli`;
  prior receipts `docs/jira/v26.11.2/lanes/lane-rc13-fleet-ops.md`,
  `docs/jira/v26.11.2/lanes/lane-r06-standalone-fence.md`.
- **Commands + exits (this session)**:
  1. Iso-hermeticity: `cp -R` each crate ALONE into /tmp/adv06-iso/
     (no parent context); `CARGO_TARGET_DIR=/tmp/adv06-target cargo
     build` in each:
     - fleet-ops → rc **0** (Finished dev profile 0.83s) — HERMETIC
       CONFIRMED. R-06's standalone claim holds with zero leaks.
     - cli → rc **101**, `failed to read .../cgap-arrow/Cargo.toml` —
       12 relative `path = "../cgap-*"` deps + `../audit` escape the
       crate dir. Not hermetic alone; this is the KNOWN ggen
       sibling-dependency pattern (own `[workspace]`, not parent-
       independent), so it is a claim-scope clarification, not a
       spoiler: "standalone" = standalone workspace, not
       parent-independent. RC-13/R-06 never claimed cli is zero-dep.
  2. Mutation flip (anti-vacuity of THIS court): injected
     `leak-probe = { path = "../does-not-exist" }` into the sandbox
     copy of fleet-ops Cargo.toml → build rc **101**, `failed to read
     /private/tmp/adv06-iso/does-not-exist/Cargo.toml`. The court
     flips; the fleet-ops pass is non-vacuous. Shared-tree
     `crates/cgap-fleet-ops/Cargo.toml` sha256
     `fb9a953066fd2d1da765fcca2d8a6774418c16f20277a6772471d87110ef1ffc`
     (unmutated, verified on disk).
  3. Workspace boundary: root `cargo metadata --no-deps` → rc 0,
     exactly **23 members**, fleet-ops ABSENT, cli ABSENT; root
     `cargo build -p cgap-fleet-ops` → rc **101**, typed error
     "package ID specification `cgap-fleet-ops` did not match any
     packages". No silent discovery path.
  4. Target-dir leakage: standalone crates build into their OWN
     `crates/cgap-fleet-ops/target` (9.1M) and `crates/cli/target`
     (811M); root `target/debug/deps` contains 0 fleet_ops/cgap_cli
     artifacts (grep -ci = 0). Build isolation law holds — no root
     target pollution.
- **Courts run**: iso-build court (fleet-ops PASS / cli fail-expected)
  + its own mutation flip. Fences that could regress: root workspace
  membership (re-witnessed 23/absent) and root-target isolation
  (re-witnessed clean). Both green.
- **Standing**: ALIVE (every command + exit above run this session).
- **Verdicts**:
  - fleet-ops hermeticity: CONFIRMED (rc 0 + flip).
  - cli hermeticity: NOT parent-independent (rc 101, 12 sibling path
    deps + absolute `[patch.crates-io]` to /Users/sac/clap-noun-verb
    — machine-local, non-portable by design per
    crates/cli/Cargo.toml:45-47); R-06's "standalone" wording is
    accurate only under the workspace sense. Record as ERRATA against
    future readers, not against R-06 (whose subject was the pattern,
    which holds).
  - Workspace boundary: HELD (typed 101, no discovery).
  - Root target isolation: HELD (0 foreign artifacts).
- **Cleanup**: /tmp/adv06-target + /tmp/adv06-iso deleted via python
  rmtree (rc 0, re-verified absent on disk).
- **Divergences**: dispatch said "failure on relative paths =
  SPOILER-CONFIRMED" — but the relative-path failure is cli's
  documented ggen sibling pattern, and the dispatch named fleet-ops as
  the zero-dep subject; fleet-ops passes clean, so no spoiler stands.
  First cargo outputs were piped (exit captured after pipe = tail's
  rc, the `$?`-after-pipe trap in CLAUDE.md); both reruns captured
  unpiped and are the rc values cited above.
