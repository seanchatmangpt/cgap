# REAL wave receipt: cgap umbrella + cgap-audit

Lane: REAL-umbrella-audit. Date: 2026-11-02. Canonical checkout only; no git
commands; no pack edits. /Users/sac/stardf untouched.

## 1. cgap umbrella crate (crates/cgap, 10 LOC)

`crates/cgap/src/lib.rs` (10 lines, `#![no_std]`, zero own logic): re-exports
4 published kernels verbatim — `pub use cgap_arrow; pub use cgap_blake3; pub
use cgap_git; pub use cgap_parquet;`. All 4 are real published kernel crates
on the path deps at 26.11.1 in Cargo.toml. It is a real re-export facade, not
a stub: every name resolves to a real kernel crate.

Observation (structural, not a defect): `cargo test -p cgap` runs 0 tests —
the facade carries no tests of its own. Tests live in the kernel crates.

Commands run this session:

| command | exit | result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/real-umb-target cargo test -p cgap` | 0 | compiles both kernels + facade; 0 tests (facade carries none) |
| manifest + lib.rs read | 0 | 4-kernel re-export surface confirmed on disk |

Verdict: **REAL** — genuine 4-kernel re-export facade, matching its own
manifest description.

## 2. cgap-audit (crates/audit, 3,719 LOC, package name cgap-audit)

`crates/audit/src/lib.rs` (168 LOC): 13 `#[no_mangle] pub unsafe extern "C"
audit_*` exports wrapping REAL kernel calls into cgap-arrow
(`bitmap::mask_words`, `silicon::mask_words_neon`, `count_valid`, `get_bit`,
`set_bit`, `unpack_{1,2,4,8}bit`, `is_inline`, `Bitmap::is_all_valid`,
`count_one_bits`) and cgap-parquet — opaque-pointer wrappers whose only job is
to keep kernels alive in the cdylib artifact for scripts/branch_audit.py
machine-code inspection. Plus `alloc_counter` (235), `powl` (982, with its own
A20 stress courts), `ocel2_slab` (2,334, with A04/A05/A14 mutation courts).

Not a stub: each export calls the real kernel fn, not a fake body.

Commands run this session:

| command | exit | result |
|---|---|---|
| `CARGO_TARGET_DIR=/tmp/real-umb-target cargo test -p cgap-audit` | 0 | 37 passed / 0 failed (powl + ocel2_slab mutation courts incl. A20 1M-row stress) |
| `CARGO_TARGET_DIR=/tmp/real-umb-target cargo build -p cgap-audit` | 0 | cdylib built |
| `nm -gU /tmp/real-umb-target/debug/libcgap_audit.dylib` | 0 | 13 exported `_audit*` symbols witnessed in the real artifact |

Verdict: **REAL** — genuine branch-audit cdylib; 13 exports proven in the
compiled dylib; 37-test court suite passes.

## 3. rdflib p:CrateManifest drift-check (both crates)

Method: rdflib in-process SPARQL over the gate-110 PASS witness and the
canonical pack `ontology.ttl` (§8), compared via tomllib against each crate's
Cargo.toml + root `[workspace.package]` (workspace inheritance resolved:
`version = {workspace = true}`).

| fact | pass witness | ontology.ttl §8 | Cargo.toml | drift |
|---|---|---|---|---|
| cgap crateVersion | 26.11.2 | 26.11.2 | 26.11.2 | none |
| cgap cratePublish | true | true | true | none |
| cgap crateDescription | "cgap manifest." | full desc but `- ` ASCII dash vs em-dash `—` | full desc with `—` | witness placeholder; ontology ASCII-dash |
| cgap-audit crateVersion | 26.11.2 | 26.11.2 | 26.11.2 | none |
| cgap-audit cratePublish | true | true | true | none |
| cgap-audit crateDescription | "cgap-audit manifest." | exact match | exact match | witness placeholder only |

Findings (same class as the cgap-bont receipt):
1. The gate-110 PASS witness file carries placeholder descriptions for both
   crates ("cgap manifest.", "cgap-audit manifest.") — witness-file-only
   drift; gate 110 checks crateName+crateVersion, so it cannot catch it.
2. Canonical ontology.ttl §8: cgap-audit description matches the manifest
   byte-for-byte; cgap's description matches except the em-dash was projected
   as ASCII `- ` into the graph (so `generated/manifest-facts.txt` carries the
   ASCII dash too). Low severity — one-character projection normalization,
   not semantic drift. Not fixed (out-of-lane files).

## Divergences

1. cgap umbrella runs 0 tests (structurally expected for a pure facade).
2. Pass-witness description placeholders (above).
3. Ontology.ttl em-dash -> ASCII-dash projection for the cgap description.
4. Single lane build root `/tmp/real-umb-target`; deleted at lane end.

## JSON receipt

```json
{
  "lane": "REAL-umbrella-audit",
  "files_written": ["/Users/sac/cgap/docs/jira/v26.11.2/waves/crate-reality/umbrella-audit.md"],
  "commands": [
    "CARGO_TARGET_DIR=/tmp/real-umb-target cargo test -p cgap (exit 0, 0 tests)",
    "CARGO_TARGET_DIR=/tmp/real-umb-target cargo test -p cgap-audit (exit 0, 37 passed)",
    "CARGO_TARGET_DIR=/tmp/real-umb-target cargo build -p cgap-audit (exit 0)",
    "nm -gU libcgap_audit.dylib | grep _audit (exit 0, 13 symbols)",
    "rdflib drift-check witness+ontology vs Cargo.toml (exit 0)"
  ],
  "courts_run": ["cgap-audit 37-test suite (powl/ocel2_slab mutation + A20 1M-row stress courts)", "symbol-export witness on compiled cdylib"],
  "standing": "REAL — both crates are real (facade + branch-audit cdylib), tests run+pass, exports witnessed in artifact",
  "divergences": [
    "cgap facade runs 0 tests (structural, expected)",
    "gate-110 pass witness carries placeholder descriptions for both crates",
    "ontology.ttl projects cgap description em-dash as ASCII dash"
  ]
}
```
