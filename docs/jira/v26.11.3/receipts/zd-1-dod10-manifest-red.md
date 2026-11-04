# ZD-1 Receipt: DoD-10 manifest court red ×3 — root cause + fix (XA-2 census red #1)

Date: 2026-11-04
Repo: /Users/sac/cgap
Status: ALIVE (commands + exits below, run this session against the exact subject)
Owned files: crates/cgap-core/Cargo.toml (license-file line only — NOT changed, see
below), crates/cgap-core/tests/manifest_court.rs (changed), this receipt (new).

## Red

`dod10_no_spdx_license_identifiers_anywhere`
(crates/cgap-core/tests/manifest_court.rs:49, panic :56 pre-fix) failed
deterministically ×3. The court scans include_str!("../Cargo.toml") and README.md
line-by-line for three banned substrings: `LicenseRef-`, `license =`, and
`license-file`. The `license-file = "../../LICENSE.md"` line (Cargo.toml:11,
landed in d090c040 as the DOD-01 F3a / DOD-05 FLAG flip) trips the third ban.

## Root cause: the court over-banned, not the manifest

DoD 10's letter is "no SPDX identifiers". Of the court's three banned shapes:

- `license =` — the SPDX expression slot; banning it is correct.
- `LicenseRef-*` — SPDX token shape; banning it is correct.
- `license-file` — NOT an SPDX identifier. It is a path pointer with no SPDX
  expression, and it is REQUIRED by three standing laws:

  1. DOD-05 (`v26.11.2/dod/dod-05-package-manifest.md` finding 1): cgap-core
     shipped no license artifact — the FLAG was "zero license lines in
     --list"; the named fix is `license-file = "../../LICENSE.md"` (branchless
     pattern, crates/cgap-branchless/Cargo.toml:5).
  2. `.claude/rules/publish.md` house style: "workspace-relative
     license-file = .../LICENSE.md".
  3. YD-1's deny.toml: the `[[licenses.clarify]]` table pins
     `license-files = [{ path = "../../LICENSE.md", hash = 0x8273ca50 }]` —
     removing/renaming the key would loudly re-break
     `cargo deny check licenses` (the intended tripwire).

Precedent: `v26.11.2/lanes/lane-dod-01fix-cargo-manifests.md:66-69` already
ruled the path-pointer form lawful ("license-file (URN pointer form) is the
lawful form per lane-dod-01-spdx — no SPDX key added") and flagged the court
for a gate-run reconciliation. This lane is that reconciliation.

## Fix: the COURT, not the Cargo.toml

Per the DoD-05 law the manifest line stands. Fix forward on the court:
`dod10_no_spdx_license_identifiers_anywhere` now bans only the true SPDX
shapes (`LicenseRef-`, `license =`); the `license-file` ban is removed and the
court comment records the lawful-form ruling and this receipt. Cargo.toml:11
was read and verified unchanged (sha256
f3c97ca3d81a070c724960fc951dccba7cd1d3b91be42df2947a372b143e0e76 before,
during, and after — taken twice this session). deny.toml needed NO matching
entry (license-file path unchanged, so YD-1's clarify table is untouched; I
did not edit deny.toml or ci.yml).

## Correction to the YD-1 receipt record (receipt not edited)

yd-1-licenses-clarify.md:19-23 claims "No Cargo.toml was touched ...
(tests/manifest_court.rs unprovoked)". The first half is true for the YD-1
lane's diff; the parenthetical is FALSE as a standing claim: the
manifest_court red predates YD-1 (the license-file line landed in d090c040,
before YD-1 ran) and stayed red through it. YD-1's cargo deny scope did not
include running cgap-core's manifest_court, so the blanket "unprovoked" claim
was asserted without a witness command. Corrected standing, per this receipt:
the court was RED from d090c040 until this fix. The yd-1 receipt is left
byte-identical (immutable record; this section is the correction of record).

## Commands / exits (this session)

Baseline red (pre-fix subject, shared tree):

- `cargo test -p cgap-core --test manifest_court` → FAILED:
  `dod10_no_spdx_license_identifiers_anywhere`; `2 passed; 1 failed`.

Post-fix, shared tree:

- `cargo test -p cgap-core --test manifest_court` → **ok; 3 passed; 0 failed**
  (run 1).
- Same command → **ok; 3 passed; 0 failed** (run 2 — green ×2).
- `cargo deny check licenses` → **exit 0**, `licenses ok`.

Non-vacuity (court must still flip on a real SPDX key):

- Sandbox: mktemp mini-workspace at /tmp (workspace-inherited keys stubbed,
  [dev-dependencies] stripped — the court test uses none), manifest mutated
  `license-file` → `license = "MIT" # MUTATION` on the line above it.
- `cargo test -p cgap-core --test manifest_court` in the sandbox →
  **FAILED: `dod10_no_spdx_license_identifiers_anywhere` +
  `dod1_manifest_has_zero_runtime_dependencies` (1 passed; 2 failed)** — the
  narrowed court flips on the mutated SPDX key. Sandbox dirs removed after
  (`python3 shutil.rmtree`; `ls -d /tmp/zd1-*` → none remain). Shared-tree
  sha256 confirmed byte-identical throughout.
- Wash run disclosed: a first attempt mutated a plain cp of the crate in /tmp
  while running the court from the shared tree — the mutation never reached
  include_str! (court stayed green, proving nothing about the mutation). It
  is counted as a wash, not a flip witness; the mini-workspace run above is
  the flip witness.

## Standing

XA-2 census red #1 (`dod10_no_spdx_license_identifiers_anywhere`): CLOSED
(court bug fixed; DoD-05/publish.md/deny-clarify law intact). Falsifier: a
future edit introducing an SPDX expression key (`license =`) or a
LicenseRef-* token into cgap-core's Cargo.toml or README.md must fail this
court loudly (sandbox flip witnessed above).
