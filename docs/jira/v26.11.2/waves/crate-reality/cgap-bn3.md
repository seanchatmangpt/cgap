# REAL lane receipt: cgap-bn3

Wave: crate-reality (reality wave), lane REAL-cgap-bn3. Date: 2026-11-01.
Repo: /Users/sac/cgap (canonical checkout; no git commands; /Users/sac/stardf untouched).

## 1. Test run (exact subject, this session)

`CARGO_TARGET_DIR=/tmp/real-bn3-target cargo test -p cgap-bn3` — EXIT 0.

| suite | result |
|---|---|
| unittests src/lib.rs | 119 passed, 0 failed (11.4s) |
| tests/lever_courts.rs | 7 passed, 0 failed (36.9s), incl. `l3_antivacuity_probe_drop_flips` |
| doc-tests | 0 |

Total: 126 passed / 0 failed / 0 skipped. Compile clean — the W3-L3 kill
mid-edit on `unify.rs` did NOT leave a broken tree; no BROKEN-MID-EDIT
condition to report, no repair performed by this lane.

## 2. rdflib p:CrateManifest drift check

Graph: `/Users/sac/ggen-marketplace/packs/cgap-pack/ontology.ttl` (9,354-line
read; parsed via rdflib, in-process SPARQL). Individual `p:manifest-cgap-bn3`
(numeric literal check at lines 703-708), namespace
`https://seanchatmangpt.github.io/packs/cgap-pack#`:

| fact | graph | disk (crates/cgap-bn3/Cargo.toml) | match |
|---|---|---|---|
| crateDescription | "Binary N3 (BN3) reasoning kernel: slab arena, sigil codec, BN3 unifier and projection for the cgap substrate." | same | True |
| cratePublish | true | `publish = true` (P3-2b flip note) | True |
| crateVersion | "26.11.2" | workspace version 26.11.2 (root Cargo.toml) | True |

Zero drift.

## 3. Substance

- LOC: 9,354 across 7 src files (unify.rs 2,560; arena.rs 2,172; project.rs
  1,436; builtins.rs 1,431; sigil.rs 834; noalloc.rs 914; lib.rs 7).
- `pub fn` count: 71 (arena 24, project 18, unify 15, noalloc 8, sigil 4,
  builtins 2, lib 0).
- `#[test]` count: 126 (matches executed test count exactly — no silent skips).

## 4. Verdict

REAL — all three checks green with commands + exits above. Anti-vacuity
witnessed in-suite: `l3_antivacuity_probe_drop_flips` passes (7/7 lever
courts). Compile-abort count: 0.

## JSON receipt

```json
{
  "lane": "REAL-cgap-bn3",
  "files_written": ["/Users/sac/cgap/docs/jira/v26.11.2/waves/crate-reality/cgap-bn3.md"],
  "commands": [
    "CARGO_TARGET_DIR=/tmp/real-bn3-target cargo test -p cgap-bn3 -> exit 0 (126 pass / 0 fail)",
    "rdflib p:CrateManifest drift check cgap-bn3 -> desc/publish/version all True, 0 drift",
    "wc -l / grep pub fn / grep #[test] substance census -> 9354 LOC / 71 pub fn / 126 tests"
  ],
  "courts_run": ["l3_antivacuity_probe_drop_flips (in-suite, pass)", "126-test suite exit 0"],
  "standing": "ALIVE",
  "divergences": [
    "p:CrateManifest namespace is https://seanchatmangpt.github.io/packs/cgap-pack# (not cgap.chatmangpt.com guessed from cgap.ttl)",
    "W3-L3 killed mid-edit on unify.rs: no break observed; tree compiles and 126/126 pass",
    "ontology/cgap.ttl carries 9 project-identity triples only; crate facts live in the cgap-pack ontology.ttl"
  ]
}
```
