# lane-dod-09-docsrs (2026-11-02)

PUB-13 config landing: universal docs.rs metadata block + categories/keywords
on the published-crate manifests, and doc-cfg honesty on cgap-gpu.
Spec: `docs/jira/v26.11.2/audit/pub-13-docsrs-plan.md` (§3/§4 tables).

## W1 Subject

18 manifests (the §4 table's rows) + `crates/cgap-gpu/src/lib.rs`.

## W2 Diff

- Per manifest (18): inserted `categories = "..."` + `keywords = "..."`
  immediately after the `publish = true` line, per §4 table verbatim; appended
  `[package.metadata.docs.rs]` / `all-features = true` /
  `rustc-args = ["--cfg", "docsrs"]` at EOF. No other byte changed.
- `crates/cgap-gpu/src/lib.rs`: added `#![cfg_attr(docsrs, feature(doc_cfg))]`
  after the crate docs, and
  `#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]` on
  `pub struct MetalSharedBuffer`. No logic change.

## W3 Commands / exits (this session)

| cmd | exit |
|-----|------|
| `CARGO_TARGET_DIR=/tmp/dod09-target cargo check -p cgap-gpu` | 0 |
| `CARGO_TARGET_DIR=/tmp/dod09-target cargo package --no-verify --allow-dirty -p cgap-gpu -p cgap-blake3` | 0 |
| `cargo metadata --no-deps --format-version 1` (workspace manifest validity, all 18 edited) | 0 |
| `python3 tomllib` assert of all 18 manifests (metadata.docs.rs block + categories/keywords present) | 0 |
| `CARGO_TARGET_DIR=/tmp/dod09-target cargo +nightly rustdoc -p cgap-gpu -- --cfg docsrs` | 0 |
| grep `Available on macOS only` in `/tmp/dod09-target/doc/cgap_gpu/index.html` | 1 hit |

## W4 Anti-vacuity court

doc-cfg is compile-inert on stable, so the flip was witnessed on nightly
(rustc 1.101.0-nightly 12433c307): with `--cfg docsrs` the generated docs HTML
for `MetalSharedBuffer` renders `Available on macOS only` — the
`doc(cfg(...))` attribute is live, not decorative. Without `--cfg docsrs`
(stable `cargo check`, exit 0) the attribute is fully inert; the stable
behavior is unchanged from pre-edit.

## W5 Fences run

- Root workspace manifest validity (`cargo metadata --no-deps` exit 0) —
  the 18-manifest diff could have broken any member; none did.
- cgap-gpu stable compile (`cargo check -p cgap-gpu` exit 0) — the only
  source-touching change.
- Packaging TOML validation (`cargo package -p cgap-gpu -p cgap-blake3`, exit 0).

## W6 Standing

ALIVE for the verified surface (18 manifests parsed+asserted, gpu compiles,
packaging exits 0, docsrs flip witnessed). The remaining 16 crates' docs.rs
builds themselves were not run locally (a per-crate `--all-features` rustdoc
sweep) and stay UNVERIFIED until docs.rs or a local rustdoc sweep runs them;
the manifest configs are validated, not the renders.

## W7 Divergences

1. Dispatch said "17 manifests"; the §4 table names 18 rows, and the tree has
   19 publish=true crates (the table omits `crates/cgap-branchless`,
   publish=true, unlisted). I landed the §4 table's 18 rows exactly;
   cgap-branchless needs a coordinator decision (row absent from spec table).
2. Audit §1/CLAUDE.md say 18 crates on crates.io vs 19 publish=true manifests
   found on disk — the census (pub-01) should reconcile the delta.
