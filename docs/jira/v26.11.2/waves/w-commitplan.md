# W-COMMITPLAN — integration commit plan (gate-runner, read-only)

Date: 2026-11-01. Repo: /Users/sac/cgap. READ-ONLY for git (zero git writes
performed); only this receipt written. /Users/sac/stardf never touched.

## Gate verification ladder (this session, CARGO_TARGET_DIR=/tmp/wcp-target)

| Gate | Command (cwd /Users/sac/cgap) | Exit | Evidence |
|---|---|---|---|
| workspace build | `cargo build --workspace` | 0 | Finished dev profile; 1 manifest warning (cgap-integration, pre-existing) |
| court harness | `cargo test -p cgap-court` | 0 | 14 passed; 0 failed (a19_seal_1m_eval_court ok, 27.81s) |
| p28 courts | `cargo test -p cgap-p28` | 0 | 38+2+6 passed; 0 failed across lib/lever_courts/p28_invariant_courts |

Lease /tmp/wcp-target (304 MB) deleted post-run via shutil.rmtree; verified absent.

Prior W2-18 finding F1 (tests/court_verdict.rs:294 K256 mutation) is RESOLVED on
disk: line 294 reads 0x391c0cb3 (correct); current M on court_verdict.rs is the
legitimate FrameView {prefilter, escalate} field fixup matching slab.rs.

## Attribution (receipts' files_written + w3-t7 content attribution)

Only 2/20 W3 lanes filed receipts with files_written (w3-r3, w3-r7); all other
attribution is content-class per w3-t7-hygiene.md. Credential sweep CLEAN per
w3-t7 (re-cited, not re-run).

## Commit plan (coordinator executes; NEVER git add -A; explicit paths only)

Order: manifests+gates first, kernel levers, benches, docs, pack/lock last.

1. MANIFESTS+NEW-CRATE (workspace must be green at this commit):
   Cargo.toml Cargo.lock crates/stardf-datalog/ \
   crates/{audit,bench,cgap-beam,cgap-bn3,cgap-core,cgap-court,cgap-crypto,\
   cgap-ffi,cgap-gate,cgap-gpu,cgap-p28,cgap-storage,integration,\
   wasm-footprint,cgap-verification}/Cargo.toml \
   crates/cgap-verification/Cargo.lock ontologies/{cloud,fibo}/Cargo.toml \
   crates/cgap-arrow/src/slab.rs crates/cgap-bn3/src/{noalloc,unify}.rs \
   crates/cgap-crypto/src/{chatman,lib}.rs crates/cgap-ffi/src/sovereign.rs \
   crates/cgap-gate/src/{lib,penstock}.rs crates/cgap-git/src/pack_write.rs \
   crates/cgap-p28/src/{asc450,attestation,lib,trap}.rs \
   crates/cgap-parquet/src/{footer,mmap_writer}.rs tests/court_verdict.rs
2. LEVER COURTS (new test files, seed-law headers):
   crates/cgap-bn3/tests/lever_courts.rs crates/cgap-gate/tests/lever_courts.rs \
   crates/cgap-ffi/tests/lever_courts.rs crates/cgap-p28/tests/lever_courts.rs \
   crates/cgap-parquet/tests/lever_courts.rs
3. BENCHES: crates/bench/Cargo.toml crates/bench/benches/{chains-bn3,\
   chains-ingress,chains-parquet,chains-seal,chains-sovereign,ingress}.rs \
   crates/bench/benches/{dred_churn,mat_closure}.rs (new)
4. GATES/TOOLS: justfile scripts/{boundary_gate,release_check,receipt_lint,\
   registry_sweep,zero_datafusion_gate}.py
5. CLI (standalone ggen crate, gen-surface): crates/cli/Cargo.toml \
   crates/cli/Cargo.lock crates/cli/ontology/cgap-cli.ttl \
   crates/cli/.clap-noun-verb/ocel.json crates/cli/src/bin/doctor.rs
6. GENERATED + DOCS + RECEIPTS: generated/{bench-registry.txt,campaign-board.md,\
   court-table.md,manifest-facts.txt} all 120 untracked docs/ paths \
   (docs/jira/v26.11.2/** incl. waves/*, patent/, plans/, research/, \
   readme-archive/INDEX.md)
7. DELETION: `git rm ggen.lock` (D ggen.lock; w3-r7/coordinator-owned regen
   artifact)

## Unowned files flagged

- scripts/receipt_lint.py, scripts/registry_sweep.py: W2-18 flagged unowned;
  w2-05 files_written claims registry_sweep.py (supersedes F2 for that file);
  receipt_lint.py remains UNOWNED (w3-r3 ran its courts; no files_written).
- generated/* (4 files): G1K outputs; g1k-01 claims manifest-facts.txt only.
- docs/jira/v26.11.2/{GOAL-LOOP,OVERNIGHT-PROMPT,HANDOFF}.md, plans/, \
  waves/bp-7-stardf-claude.md: coordinator-owned per w2-18 census.
- crates/*/readme-archive/*-2026-11-01.md (5 files): no receipt claims them.

## /tmp leases to verify deleted BEFORE committing (report-only; not deleted)

Lane W3 leases live at census: w3-l2-target, w3-l4-target, w3r1-target,
w3-r7-backup, avec-w3-rehearsal (w3-t7 census); also observed w3-l3-target
(62M), w3-r4-target (15M), w3t8-sd-target (9.5M), w3t1-bench, plus stale
cd2/cd3/lane1/a*-target debris (~21 GB class per w3-t7). wcp-target (this
lane, 304M) DELETED this session, verified absent.

<!-- json-footer
{"lane":"W-COMMITPLAN","receipt":"docs/jira/v26.11.2/waves/w-commitplan.md","files_written":["docs/jira/v26.11.2/waves/w-commitplan.md"],"commands":["cargo build --workspace CARGO_TARGET_DIR=/tmp/wcp-target (exit 0)","cargo test -p cgap-court (exit 0, 14 passed)","cargo test -p cgap-p28 (exit 0, 46 passed across 3 binaries)","shutil.rmtree /tmp/wcp-target (verified absent)","git status --porcelain (exit 0, 148 entries)","git diff --stat (exit 0, 64 files)"],"standing":"ALIVE","git_writes":0,"divergences":["attribution content-only for ~50/52 tracked files (W3 receipt gap per w3-t7)","ggen.lock deletion assigned to commit 7 (git rm, not add)"]}
json-footer -->
