# pub-02 package dry-run matrix

PUB-02 package dry-runs (2026-11-02, fresh run; prior instance
stopped mid-gathering). Tool: `cargo package --no-verify
--allow-dirty`, CARGO_TARGET_DIR=/tmp/pub02b-target. Publishes
FROZEN — packaging only, no registry contact beyond index read.

## Set derivation (ground truth, re-derived)

Members (cargo metadata, 23): 21 crates + 2 ontologies.
Explicit flags on disk: 19 `publish = true`, 4 `publish = false`
(bench: registry name collision; beam, branchless, court).
NOT members (skipped): cgap-verification, csct, cgap-cli,
wasm-footprint (standalone [workspace] tables).
CLAUDE.md drift ("8 true / 4 false / 11 default"): confirmed
stale, matches PUB-01's flag. Default-publishable set is EMPTY.
Effective in-scope set = 19 member crates with publish=true.

wasm-footprint: publish=true on disk but NOT a workspace
member. Dry-run done via explicit --manifest-path (disclosed,
out-of-scope for member sequencing).

## Matrix (sequential dry-runs, exit / warnings / files)

| crate | exit | warn | files | size | verdict |
|---|---|---|---|---|---|
| cgap | 101 | err | 21 | - | NO-GO |
| cgap-audit | 0 | none | 10 | 158.4K | GO |
| cgap-arrow | 0 | none | 44 | 448.5K | GO |
| cgap-blake3 | 0 | none | 15 | 112.8K | GO |
| cgap-bn3 | 0 | none | 17 | 484.3K | GO |
| cgap-bont | 0 | none | 11 | 237.9K | GO |
| cgap-bosym | 0 | none | 8 | 65.0K | GO |
| cgap-core | 0 | none | 24 | 178.8K | GO |
| cgap-crypto | 0 | none | 12 | 100.4K | GO |
| cgap-ffi | 0 | none | 19 | 369.4K | GO |
| cgap-gate | 0->101 | err | 10 | - | NO-GO |
| cgap-git | 0 | none | 61 | 793.8K | GO |
| cgap-gpu | 0 | none | 9 | 8.9K | GO |
| cgap-integration | 0 | none | 11 | 79.5K | GO |
| cgap-p28 | 0 | none | 12 | 225.1K | GO |
| cgap-parquet | 0 | none | 43 | 726.1K | GO |
| cgap-storage | 0 | none | 22 | 152.4K | GO |
| cgap-cloud | 0 | none | 13 | 113.6K | GO |
| cgap-fibo | 0 | none | 14 | 116.1K | GO |
| wasm-footprint | 0 | none | 9 | 42.0K | GO (non-member) |

Notes: cgap-gate "0->101" = the --list probe exits 0 but the
real packaging exits 101. All sizes are .crate totals.

## NO-GO findings + exact blocking edits

### NO-GO 1: cgap (umbrella), exit 101
Error: `no matching package named cgap-branchless found on
crates.io index; required by cgap v26.11.2`.
Root cause: crates/cgap/Cargo.toml:26 declares
`cgap-branchless = { path = "../cgap-branchless",
version = "26.11.2", default-features = false }` but
crates/cgap-branchless is `publish = false` and absent from
crates.io. Packaging strips path deps; resolution then fails.
Exact blocking edit (pick ONE):
(a) Make the dep optional behind a feature and do not enable
    it by default:
    `cgap-branchless = { path = "../cgap-branchless",
    version = "26.11.2", optional = true,
    default-features = false }` + a non-default feature
    `branchless = ["dep:cgap-branchless"]`.
(b) Drop the dep from cgap entirely if the umbrella surface
    does not re-export it (verify src/lib.rs first).
(c) Flip cgap-branchless to publish=true and publish it BEFORE
    cgap (violates current publish=false law; not preferred).

### NO-GO 2: cgap-gate, exit 101
Error: `dependency cgap-branchless does not specify a version`.
Root cause: crates/cgap-gate/Cargo.toml:16 uses
`cgap-branchless = { workspace = true }`; the root
[workspace.dependencies] entry (root Cargo.toml:43) is
`cgap-branchless = { path = "crates/cgap-branchless",
default-features = false }` — NO version key.
Exact blocking edit:
(a) Add version to the root workspace dep:
    `cgap-branchless = { path = "crates/cgap-branchless",
    version = "26.11.2", default-features = false }`.
    WARNING: this unblocks manifest validation only. cgap-gate
    will then hit the SAME registry-absence failure as cgap
    (branchless is publish=false, not on crates.io). So the
    real fix is also option (a)/(b) from NO-GO 1: optional
    feature-gated dep, or drop the dep.
(b) Overriding at the member level
    (`cgap-branchless = { workspace = true, version = ... }`)
    is not supported for workspace-inherited deps in stable
    cargo; fix at the root table.

## Debris / fixture flags (should these ship?)

- cgap-crypto ships `scripts/wave7_shim_battery.py` — a Python
  harness packaged with the crate. Low harm, but likely
  unintended; add `exclude` or move out of crate dir.
- wasm-footprint ships `NOTES-v26.10.31.md` — dated scratch
  notes in a public crate. Add `exclude` in [package] or move
  to docs/ (non-member, low priority).
- cgap-arrow/blake3/git/parquet ship `readme-archive/
  *-2026-11-01.md` — historical readmes. Probably intentional
  (frozen receipts); disclose, not blocking.
- cgap-ffi ships `enclave_128k.ld` + `xip.ld` — linker scripts,
  almost certainly intentional (SGX-style linking). OK.
- cgap-cloud ships RECEIPT.md + tosca_cimi_v1.oabi, cgap-fibo
  ships fibo_sec_dbt_v1.oabi — the O-ABI packs themselves.
  Intentional. OK.
- cgap (umbrella) ships 2 benches + 9 test files — heavy but
  normal. OK.

## License check (FIX-01 verification)

ZERO license warnings across all 20 dry-runs (no
"missing license", no license-file resolution warnings, no
`warning:` lines at all in any packaging output). FIX-01
holds. cargo metadata confirms license-file resolves via
[workspace.package] for every member.

## Verdict

17 GO, 2 NO-GO (cgap, cgap-gate), 1 non-member GO
(wasm-footprint). Both NO-GOs share ONE root cause: publishable
crates depending on the unpublishable cgap-branchless. The
gate is NOT all-GO until cgap and cgap-gate are fixed and
re-dry-run clean. cgap-fibo GO with note: `Updating crates.io
index` observed (normal index read, no error).
