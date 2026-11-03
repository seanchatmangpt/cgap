# pub-06-readme-plan

PUB-06 README staleness remediation plan (2026-11-02, read-only
planner). Scope: the 18 publish=true crates. ALL 18 have README.md
(2026-11-01) — zero missing; this is a staleness audit + skeleton
upgrade, not from-scratch authoring.

## 1. Inventory

| crate | README | verdict |
|---|---|---|
| cgap | 120 L | STALE (says 26.11.1, "22 crates") |
| cgap-git | 40 L | STALE: title/scope = packfile v2 only vs 23 src modules (a2a_swarm, admission_gate, commit_synth, delta_resolver, egress/ingress_ring, mcp_driver, object_store, octopus_merge, pack_demux, patent_court, ref_store, retract, tree_synth...) |
| cgap-crypto | 45 L | CURRENT; add seals/s caveat |
| cgap-gpu | 30 L | CURRENT; needs Apple-Silicon row |
| wasm-footprint | 43 L | CURRENT; add wasip1 matrix row |
| other 13 | 30-61 L | CURRENT; add caveats + docs.rs matrix |

Errata: cgap-bosym's Cargo.toml description drifts from its README
(reported, not fixed — PUB-01 owns manifest edits).

## 2. Confirmed staleness

cgap-git: README "Git packfile v2 kernel" + Cargo.toml "Git
Packfile v2 writer" describe a fraction of the surface. cgap
umbrella README: "version tracks the workspace at 26.11.1" and
"22 crates" — must be re-derived at rewrite time, not copied.

## 3. Per-crate skeleton (mandatory section order)

1. # <crate> — honest one-line identity (from src census, not
   description drift).
2. Orientation paragraph: layer (kernel/membrane/container),
   no_std posture, zero-dep claim only if dep-count = 0.
3. ## Status — PUBLISHED-26.11.2 + census pointer.
4. ## Capabilities (receipt-cited) — no number without a
   citation.
5. ## Honest failures and caveats — mandatory where applicable:
   POSH-2 23.35 ns gate FAIL (lane-posh-02:29-64); C6 242-324M
   band vs DECIDED 150M, 800M retracted + loud 3.31x shortfall
   (release-blockers:14-23,233-238); seals/s "~3.95-4.35M median,
   machine-dependent, gate = 1M floor, never '4.0M'" (tps-08:137-
   139); container delta published as delta only (1.7-1.8x)
   (tps-08:143-145); layer qualifiers mandatory on every ns
   figure (tps-07 C1 law).
6. ## docs.rs build matrix (§4).
7. ## Install — cargo add <crate>@26.11.2.
8. ## Lane receipt — census path + historical receipts.

## 4. docs.rs build matrix

All 18: stable toolchain (zero portable_simd/nightly gates
witnessed), edition 2021, no .cargo/config.toml target rustflags
— docs.rs x86_64 linux default-features --docs builds clean
without target features. Rows: cgap (default=host; core+storage
gated behind host), arrow (cfg(target_arch) fast paths), audit +
ffi (cdylib+rlib, docs via rlib; riscv courts cfg'd out on
docs.rs), blake3 (NEON/AVX2 cfg-gated), bn3 (no_std body, std
under cfg(test)), crypto (core+alloc body, std re-links under
cfg), gate (penstock courts cfg-gated), git (no_std, SHA-1/CRC32
in-crate), gpu (Metal cfg-gated to Apple; docs.rs builds
fallback), parquet (aarch64 cfg-gated), p28 (wasm32/bare-metal
split), wasm-footprint (wasm32-wasip1 real target; docs.rs host
fallback).

## 5. Sequencing

1. cgap-git README rewrite (confirmed stale — name the
   23-module surface).
2. cgap umbrella re-census (26.11.2, crate count re-derived).
3. Remaining 16: append Honest-failures + docs.rs matrix where
   applicable; fix bosym description drift check.
4. Post-pass: <=100-char lines, no emoji, every number cited or
   deleted.

## 6. Non-goals

No manifest edits (description drift reported here; PUB-01 owns).
No lint mechanization of the layer-qualifier law (tps-07 C1).
No republish; README-only changes ride the next publish attempt.

## See Also

audit/pub-01-manifest-census.md · audit/tps-08-shipgate-plan.md ·
lanes/lane-posh-02-batch.md
