# pub-01-manifest-census

PUB-01 manifest census (2026-11-02, from disk; 25 manifests +
root [workspace.package]: version 26.11.2, license-file
LICENSE.md, repository, authors).

## Membership correction (flag)

Root members: 21 crates + 2 ontology = 23. NOT members:
wasm-footprint, cgap-verification, cli, csct (standalone
[workspace] tables). CLAUDE.md DRIFT: "12/23 declare an
explicit flag (8 true, 4 false); 11 default-publishable" is
wrong — all 21 crate members declare explicit publish flags:
17 true, 4 false (bench, beam, court, branchless). The
default-publishable set is EMPTY. CLAUDE.md needs
re-derivation (MV-8's table itself drifted).

## Authoritative per-crate census

| crate | ver | license | desc | authors | readme | publish | type |
|---|---|---|---|---|---|---|---|
| cgap | ws 26.11.2 | LF ../../ | yes | ws | NO | true | lib |
| cgap-arrow | ws | LF ../../ | yes | ws | NO | true | lib |
| cgap-blake3 | ws | LF ../../ | yes | ws | NO | true | lib |
| cgap-parquet | ws | LF LF | yes | ws | NO | true | lib |
| cgap-git | ws | LF ../../ | yes | ws | NO | true | lib |
| cgap-core | ws | LF ../../ | yes | ws | NO | true | lib |
| cgap-storage | ws | LF ../../ | yes | ws | NO | true | lib |
| cgap-bont | ws | LF ../../ | yes | MISSING | NO | true | lib |
| cgap-bosym | ws | LF ../../ | yes | MISSING | NO | true | lib |
| cgap-bn3 | ws | LF ../../ | yes | MISSING | NO | true | lib |
| cgap-gate | ws | LF ../../ | yes | MISSING | NO | true | lib |
| cgap-crypto | ws | LF ../../ | yes | MISSING | NO | true | lib |
| cgap-p28 | ws | LF ../../ | yes | MISSING | NO | true | lib |
| cgap-gpu | ws | LF ../../ | yes | ws | NO | true | lib |
| cgap-ffi | ws | LF ../../ | yes | MISSING | NO | true | cdylib+rlib |
| audit | ws | LF ../../ | yes | MISSING | NO | true | cdylib+rlib |
| integration | ws | LF ../../ | yes | MISSING | NO | true | lib |
| bench | ws | LF ws | yes | MISSING | NO | false | bin |
| cgap-court | ws | LF ws | MISSING | MISSING | NO | false | test-only |
| cgap-beam | ws | LF ws | yes | MISSING | NO | false | cdylib |
| cgap-branchless | 26.11.2 | LF ../../ | yes | MISSING | NO | false | lib |
| wasm-footprint | 26.11.2 hard | LF ../../ | yes | hard | NO | true | cdylib, non-member |
| cgap-verification | 26.11.2 | LF ../../ | yes | MISSING | NO | false | lib, non-member |
| cli | 26.11.2 | MISSING | MISSING | MISSING | NO | false | bin, non-member |
| csct | 0.1.0 | MISSING | MISSING | MISSING | NO | false | bin, non-member |

(LF = license-file. cgap-court's license-file.workspace = true
resolves identically to siblings' literal ../../LICENSE.md.
wasm-footprint hardcodes repository/authors matching workspace
values. cgap-verification uses literal repository, same value.)

## Dry-run blocker analysis

Every publishable crate HAS description + license-file
(resolves to /Users/sac/cgap/LICENSE.md, exists) + version.
No publishable crate should hard-fail `cargo package
--no-verify --allow-dirty` on manifest fields. cargo does NOT
hard-require authors — the missing-authors rows are
conformance gaps, not dry-run failures.

## Exact manifest edits

Priority A (conformance, publishable): add
`authors.workspace = true` to [package] of cgap-bont,
cgap-bosym, cgap-bn3, cgap-gate, cgap-crypto, cgap-p28,
cgap-storage, cgap-ffi, audit.
Priority A2 (member, publish=false, style): cgap-branchless
add repository.workspace + authors.workspace.
Priority B (standalone, publish=false, optional): cli + csct
add description/license-file/repository/authors; cgap-court
add description.
Optional: readme = "../../README.md" per publishable crate
(no per-crate READMEs exist — see pub-06).

## CLAUDE.md actions

1. Re-derive the publish-flag paragraph (17 true / 4 false,
   zero default-publishable).
2. Name wasm-footprint standalone (:61-62 implies membership).

## See Also

audit/pub-02-package-dryrun.md · CLAUDE.md ·
docs/jira/v26.11.2 rules (publish.md)
