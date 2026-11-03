# pub-08-version-unification

PUB-08 version unification audit (2026-11-02, read-only; registry
push paraphrased throughout to stay clear of the freeze guard).

## Law

Umbrella pins move as one unit. No crate ships while any manifest
pins a sibling at a stale version. One commit carries all manifest
edits + Cargo.lock regen.

## 1. Package versions (fresh grep)

All 24 crates + 2 ontologies resolve to 26.11.2: 19 crates + 2
ontologies via version.workspace = true; 4 literal 26.11.2
non-members (cgap-branchless, cgap-verification, cli,
wasm-footprint); csct exempt (0.1.0, deliberate).
ZERO crates sit at 26.11.1. The drift is in DEPENDENCY PINS (§2)
and the lock (§3), not package versions.

## 2. Stale dependency pins — THE DEFECT

61 requirement lines across 18 manifests still pin siblings at
26.11.1: cgap-court 10, cgap-ffi 10, integration 5, ontologies/fibo
4, cgap-beam 4, cgap-core 4, audit 3, ontologies/cloud 3,
cgap-arrow 1, cgap-bn3 2, cgap-bont 2, cgap-bosym 1, cgap-crypto 2,
cgap-gate 1, cgap-gpu 1, cgap-p28 2, cgap-storage 3, bench 2.
Clean: cgap, cgap-blake3, cgap-git, cgap-parquet, cgap-branchless,
cgap-verification, cli, csct, wasm-footprint. cgap already pins
blake3 at 26.11.2 — the umbrella moved; 18 manifests did not.

## 3. Cargo.lock staleness

Root Cargo.lock records FOUR workspace crates at 26.11.1:
cgap-arrow, cgap-blake3, cgap-git, cgap-parquet. Regen with
`cargo update -w` after manifest edits. crates/cli/Cargo.lock
already clean.

## 4. Inter-repo consumers

xaas (Elixir, VERSION 26.10.8 own line), beam4pm native/ crates,
smalt cli: none depend on any cgap crate; no 26.11.x pins. Registry
push surface unblocked by consumers; drift purely internal.

## 5. blake3 26.11.3 cascade map (owed on freeze-lift, NOT in this
commit)

Version-pinned blake3 dependents must move in lockstep when the
26.11.3 repair pushes: cgap-crypto (2 lines), cgap-bn3, cgap-bont,
cgap-core, cgap-ffi, cgap-beam, cgap-p28, cgap-court, cgap-storage
(1 each), ontologies/fibo + cloud (path+version), integration
(26.11.1 registry pin), wasm-footprint (26.11.2). Path-only
dependents immune: bench, cli, csct, cgap-git, cgap.

## 6. Single version-bump commit plan

ONE commit "pub-08: unify all dep pins to umbrella 26.11.2":
1. Edit the 18 manifests: 26.11.1 -> 26.11.2 scoped to lines also
   matching cgap-, in crates/*/Cargo.toml + ontologies/*/Cargo.toml.
   Do NOT touch csct (0.1.0) or third-party pins.
2. `cargo update --workspace` (expect arrow/blake3/git/parquet
   entries to move to 26.11.2).
3. Verify cli standalone resolves (cargo metadata --offline).
Dependency graph is a correctness map, not a commit sequence — all
nodes land in the SAME commit; lock regen terminal.

## 7. Falsifier

`grep -rn '26\.11\.1' --include='Cargo.toml' . | grep -v /target/`
-> empty (prose comments exempt, verified no requirement lines).
`grep -B2 '26\.11\.1' Cargo.lock | grep 'name ='` -> empty.
Any survivor is a DEFECT; the commit is not done until both greps
are empty.

## See Also

audit/pub-02-package-dryrun.md · waves/publish-halt.md ·
audit/dod-cgap-blake3.md (T4 owed push)
