# DOD-01B — Cargo hygiene independent verification

Date 2026-11-02 · repo /Users/sac/cgap (canonical checkout, branch main).
Read-only verification of lane-dod-01fix-cargo-manifests.md; sole write is
this receipt. No git commands. Build lease /tmp/dod01b-target, deleted at
lane end.

## Per-item verdicts (commands + exits this session)

1. license-file resolution — PASS (4/4). Each relative path read from its
   manifest's directory:
   - cgap-core `../../LICENSE.md` → /Users/sac/cgap/LICENSE.md (exists)
   - cgap-ocel2 `../../LICENSE.md` → exists
   - csct `../../LICENSE.md` → exists
   - cgap-fleet-ops `../../LICENSE.md` → exists
   Corroborated by `cargo package --list --allow-dirty` per crate
   (rc=0 each, grep -c '^LICENSE.md$' = 1 each):
   /tmp/dod01b-{cgap-core,cgap-ocel2,csct,cgap-fleet-ops}.list
2. cgap-cloud RECEIPT.md exclusion — PASS. `exclude = ["RECEIPT.md"]`
   at ontologies/cloud/Cargo.toml:11; --list rc=0, grep -c
   '^RECEIPT.md$' = 0. RECEIPT.md confirmed present on disk (24,963 B
   class) so the exclusion is load-bearing, not vacuous.
3. cargo metadata --no-deps — PASS: rc=0, 23 workspace members
   (/tmp/dod01b-meta.json, parsed count 23).
4. cargo build -p cgap-core — PASS: rc=0 (dev profile, Finished).
5. Escalated-open (coordinator-owned), re-confirmed on disk:
   - cli license-file: crates/cli/Cargo.toml has NO license key
     (grep zero lines); hook guard-generated.sh refusal (DOD-01FIX
     receipt lines 26-37) — must enter via ggen graph or
     manifest_patch.py. The refusal is the mechanism working.
   - csct version = "0.1.0" (csct/Cargo.toml:3) vs fleet 26.11.2.
   - workspace MSRV: no `rust-version` in root Cargo.toml (grep
     zero lines; only version = "26.11.2" at line 47).

## Anti-vacuity

Item 2's mutation flip is the DOD-05 named surface (RECEIPT.md on disk,
absent from --list). Items 1's flip: LICENSE.md appears in --list only
because the key exists — the before-state (zero license lines) is the
DOD-05 FLAG witness. No skips.

## Fences

`tests/manifest_court.rs` (cgap-core, SPDX ban): no SPDX `license` key
added anywhere; only `license-file` URN-pointer form — court surface
not run here (no test-runner ownership), flagged for coordinator gate.
Working-tree verdict under --allow-dirty, same caveat as DOD-01FIX.

## Standing

ALIVE for items 1-4 (every command run this session, exits cited).
Escalated-open list stands as coordinator-owned; PARTIAL overall until
cli license-file enters via the ggen regen cycle.

## JSON terminus

{"lane":"DOD-01B","files_written":["docs/jira/v26.11.2/dod/dod-01b-hygiene-verify.md"],
"commands":["ls resolution x4 (all exist)","cargo package --list --allow-dirty x5 (rc=0, LICENSE.md=1 x4, RECEIPT.md=0)","cargo metadata --no-deps (rc=0, 23 members)","cargo build -p cgap-core (rc=0)"],
"courts_run":["cgap-cloud RECEIPT.md exclusion flip","DOD-05 license flip corroboration (4 crates)"],
"standing":"ALIVE items 1-4; escalated-open: cli license-file (ggen graph/manifest_patch.py), csct 0.1.0 version, workspace MSRV",
"divergences":["none beyond DOD-01FIX's recorded escalations"]}
