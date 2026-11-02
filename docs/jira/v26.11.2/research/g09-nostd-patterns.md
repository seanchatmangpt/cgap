# R9 — ggen no_std patterns (Explore brief)

Lane: R9. Source: `/Users/sac/ggen`. Claims cited file:line. Purpose: MW2 std-strip prior art.

## Where #![no_std] lives

No ggen workspace crate is no_std. All no_std usage is in packs/examples/book listings —
ggen's role is generator/verifier of no_std products:

| Artifact | Path | Notes |
|---|---|---|
| TCPS core (production ref) | packs/tcps-core-pack/reference/製品版/crates/tcps-core/src/lib.rs:1 | #![no_std], forbid(unsafe_code) :2 |
| TCPS core (older tree) | packs/tcps-core-pack/reference/豊田コード生産方式_v26.7.19/src/lib.rs:1-2 | same |
| Generated example | examples/tcps-generated/src/lib.rs:1 | no_std, empty [dependencies] |
| Embedded (Cortex-M) | examples/archive/embedded-cross/src/main.rs:3-4 | no_std + no_main, custom panic_handler |
| Book listing ch.128 | book/src/listings/128-128-generate-no-std-code.rs:1 | forbid(unsafe_code), const-only artifact |

## Zero-dep kernel pattern (model to copy)

tcps-core: [dependencies] completely empty; keywords include no-std/embedded (Cargo.toml:8-12).
release.json: `"core": {"no_std": true, "external_dependencies": 0, "unsafe": "forbidden"}`
(packs/tcps-release-pack/reference/release.json).

- Alloc avoidance is **structural, not lint**: every domain type Clone, Copy, fixed fields;
  no String/Vec (品質.rs); const fn accessors, Copy permission-token types, digest by value
  (青い川のダム.rs); `平準化箱::<2>::空()` const-generic fixed-capacity buffer replacing Vec
  (試験.rs:16-22).
- Modules pub mod with #[path]; only std reference is `#[cfg(test)] extern crate std;`
  (lib.rs:5-6, both trees) — std exists only in the test build.

## std quarantine / linkage discipline

**No dedicated abort-if-std-linked gate found** (no std-linkage/nm/otool symbol-audit hits).
Nearest structural guarantee: the crate cannot reach std (cfg(test)-gated extern crate std +
empty [dependencies]) and `--product core` builds universally across all tier 1+2+3 targets
including bare-metal — a std link fails the build. scripts/full-lifecycle-all-targets.sh:11-13.

- **std lives in a separate adapter crate, never the core**: `crates/tcps-std` (環境観測・受領証永続化)
  (packs/tcps-release-pack/reference/Cargo.toml:5,12); lifecycle.py:507 builds it separately
  (`build -p tcps-std --lib`). Book ch. 255-tcps-std.md:132: "no_std core logic, std adapters,
  stable C ABI, WASM boundary, CLI consumer" — std is a product tier, not a core dependency.
- Target matrix receipt-bound: evidence/source/target-matrix-receipt.json (tier-1: 8,
  tier-2: 104, official-snapshot); targets/tier1.txt, tier2.txt; embedded targets in
  official-platform-support-1.97.1.json:144-420. CI products.yml builds core for every target.

## Generator-side support

Templates render no_std bodies from RDF via SPARQL: tcps-core-pack templates/lib.rs.tmpl + 27
module templates incl. 暗号要約.rs.tmpl (body = ontology sourceText via
`SELECT ?source WHERE { tcps:暗号要約Module tcps:sourceText ?source }`). Gate tests are SPARQL
admission gates (010_required.rq: any row = missing required property → fail).

## MW2 recommendations (cgap-side)

1. Adopt tcps-core shape: #![no_std] + forbid(unsafe_code); std behind
   `#[cfg(test)] extern crate std;`; empty [dependencies] or core/alloc only. Note: ggen
   reference crates never use extern crate alloc — fixed-capacity const-generic buffers and
   Copy value types replace Vec/String in hot paths (strongest alloc-avoidance pattern).
2. Split std into an adapter crate rather than feature-gating std inside the core
   (tcps-std analog).
3. Std-linkage gate: nothing in ggen to copy. Practical equivalents: (a) cfg(test)-gated std,
   (b) empty [dependencies], (c) build for a bare-metal target (thumbv6m-none-eabi /
   aarch64-unknown-none / riscv32imc) where any std reach is a hard compile error. For an
   explicit "abort if std linked" gate, pair (c) with a new nm/otool symbol check — must be
   written new.
4. Receipt the target matrix (digest-bound tier snapshot) like target-matrix-receipt.json so
   the gate is replayable evidence; house style gates/*.rq any-row-fails.
