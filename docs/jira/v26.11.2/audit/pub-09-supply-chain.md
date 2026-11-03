# PUB-09: Supply-Chain Audit (v26.11.2)

Date: 2026-11-02. Read-only sweep; evidence from manifests, Cargo.lock,
and `grep -rn "/Users/sac/" crates/*/src/ include/`.
Scope: 25 crates; 18 with `publish = true` are the audit surface.
Publish freeze (waves/publish-halt.md) remains in force; nothing published.

## 1. Per-crate supply-chain table (publish = true, 18)

| crate | (a) runtime deps vs lock | (b) /Users/sac in pkg | (c) license | (d) secrets |
|---|---|---|---|---|
| cgap | 10 cgap-* path+ver 26.11.2; lock 26.11.2 | none | license-file OK | clean |
| cgap-arrow | 0 runtime deps; core/gate/storage are dev-only | none | license-file OK | clean |
| cgap-blake3 | 0 deps (any kind); lock has 26.11.1+26.11.2 | none | license-file OK | clean |
| cgap-parquet | 0 runtime; arrow dev-only; lock dual ver | 2 comment hits | license OK | clean |
| cgap-git | 0 runtime; blake3/parquet/bn3 dev-only | none | license-file OK | clean |
| cgap-bn3 | blake3 path+26.11.1; lock has 26.11.1+26.11.2 | none | license-file OK | clean |
| cgap-bont | blake3 path+26.11.1 | none | license-file OK | clean |
| cgap-bosym | 0 deps | none | license-file OK | clean |
| cgap-core | 0 runtime; blake3/gate dev-only; FIX-01 | none | license-file OK | clean |
| cgap-storage | blake3 26.11.1 runtime; git dev-only | none | license-file OK | clean |
| cgap-gate | BLOCKED: branchless path-only, no version | none | license-file OK | clean |
| cgap-crypto | blake3 path+26.11.1 (also dev dup) | none | license-file OK | clean |
| cgap-p28 | blake3+crypto path+26.11.1 | none | license-file OK | clean |
| cgap-ffi | arrow 26.11.1 runtime; audit dev | 2 lane hits | license-file OK | clean |
| cgap-gpu | arrow path+26.11.1 | none | license-file OK | clean |
| cgap-audit | arrow+parquet path+26.11.1 | none | license-file OK | clean |
| cgap-integration | 0 runtime; cgap-* dev-only | none | license-file OK | clean |
| wasm-footprint | arrow/parquet/blake3 registry "26.11.2" | none | license-file OK | clean |

Publish=false (out of publish scope, noted for hygiene): bench, cgap-beam,
cgap-branchless, cgap-court, cgap-verification, cgap-cli, csct.

## 2. (a) External runtime deps vs Cargo.lock

Lockfile: 51 packages. Every non-workspace source is crates.io registry
(no git/path-only externals, no vendored forks in the graph).
All 18 publish-true crates declare ZERO external runtime dependencies;
the only external deps in the workspace live in cgap-cli (blake3 1.8,
clap-noun-verb =26.8.22, linkme 0.3, serde 1, serde_json 1; publish=false)
and as dev-deps elsewhere. nanoarrow is referenced only in comments
(validate.rs) — not a dep; vendor/nanoarrow is reference material, not
in the lock graph.

Honesty check (task focus): cgap-arrow's Cargo.toml [dependencies] is
EMPTY. cgap-core/cgap-gate/cgap-storage appear only under
[dev-dependencies] as bare `path =` (no version) — legal for publish
(dev-deps never ship) and honestly disclosed as court collaborators
(lane A01 comment in the manifest). Not a violation.

Pin drift: 12 publish-true crates pin cgap-* at "26.11.1" while the
workspace version is 26.11.2 (superseded by PUB-08; semver-compatible
but the lockfile carries BOTH 26.11.1 and 26.11.2 for cgap-arrow,
cgap-blake3, cgap-git, cgap-parquet — dual-version skew is real on
disk).

## 3. (b) Absolute host paths in shipped source

Full grep across crates/*/src/ and include/: 8 hits, no code-path hits
in any publish-true crate (all are doc comments):

- crates/cgap-parquet/src/compiled_manifest.rs:7 — doc comment
- crates/cgap-parquet/src/rle.rs:582 — doc comment
- crates/cgap-ffi/src/oabi_lookup.rs:48 — @lane receipt comment
- crates/cgap-ffi/src/xip_tables.rs:47 — @lane receipt comment
- crates/cgap-beam/src/lib.rs:5, resources.rs:4 — doc comments
  (publish=false; still ships in repo, cosmetic)
- crates/cli/src/custom_handlers/handlers_sovereign.rs:234 — RUNTIME
  `.current_dir("/Users/sac/cgap")` (publish=false)
- crates/cli/src/bin/doctor.rs:20 — RUNTIME arg to
  `/Users/sac/cgap/scripts/doctor_agent_lanes.py` (publish=false)

Verdict: no publish-true crate executes an absolute host path; four
comments leak the username/host in published docs (cosmetic leak,
house style already documents repo paths — fix optional).
cgap-cli's two runtime hits break the binary on any other host —
real bug, but outside the publish surface.

## 4. (c) License files

All 18 publish-true manifests carry
`license-file = "../../LICENSE.md"` (house form). FIX-01 landed
2026-11-02 on cgap-core (lane-fix01-core-license.md): verified via
`cargo package` that the tarball ships
`cgap-core-26.11.2/LICENSE.md`, 0 warnings. The sweep confirms no
other publish-true manifest is missing the field — FIX-01's
report-only sweep finding is confirmed closed. Root instruments:
LICENSE.md + LICENSE-P28-SIL.md; no SPDX expression by design (URN).
Not independently re-verified per crate (no dry-run in this read-only
pass); FIX-01's tarball proof is the reference evidence for the form.

## 5. (d) Secrets/tokens in packaged files

Regex sweep (api_key/secret/password/AKIA/ghp_/github_pat_/sk-ant/
xoxb-/PEM BEGIN PRIVATE KEY) over crates/*/src, crates/*/tests,
include/: ZERO true hits. Matches were Rust `Token` enums and lexer
functions (next_token, lz77_tokens, extract_fibo_tokens) — false
positives only. No tokens, no credentials, no private key material.

## 6. Blocked crates

1. cgap-gate — publish=true but [dependencies] contains
   `cgap-branchless = { workspace = true }`; the workspace entry is
   `path = "crates/cgap-branchless", default-features = false` with NO
   version, and cgap-branchless itself is `publish = false`. Cargo
   publish requires a registry-resolvable version for every runtime
   path dep: cgap-gate cannot publish until branchless gains a version
   + publish=true, or the edge is dev-gated/removed. Only hard blocker.
2. All crates pinning cgap-* "26.11.1" (storage, p28, crypto, bn3,
   bont, gpu, ffi, audit, plus cgap-parquet via lock 26.11.1) — not
   hard-blocked (semver-compatible), but the 26.11.1/26.11.2 dual
   versions in Cargo.lock mean a v26.11.2 release published as-is
   would interleave stale 26.11.1 resolutions. Track under PUB-08.

## 7. Standing

ALIVE against current disk state. No commands mutated the tree.
