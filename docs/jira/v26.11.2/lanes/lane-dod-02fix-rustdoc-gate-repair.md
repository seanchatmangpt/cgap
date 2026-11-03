# DOD-02FIX — rustdoc gate repair (31 typed errors → gate ALIVE)

Lane: DOD-02FIX. Date: 2026-11-02. Contract: doc-comment-only fixes over
the DOD-02 typed error table
(`docs/jira/v26.11.2/dod/dod-02-rustdoc-gate.md`); no logic/signature/API
changes; no git commands. Build lease: /tmp/dod02fix-target.

## W1 — Verdict

**ALIVE** — workspace rustdoc gate now exits 0. The DOD-02 census's 31
located errors are fixed doc-side; three additional crates surfaced
behind the DOD-02 run's parallel-abort wall (cgap-ffi, cgap root, bench)
and are fixed with the same doc-side classes, disclosed as a scope
extension (D1).

## W2 — Gate command + exit

```
RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links" \
  CARGO_TARGET_DIR=/tmp/dod02fix-target \
  cargo doc --workspace --no-deps --document-private-items
EXIT=0   # post-fix (pre-fix EXIT=101, /tmp/dod02fix-doc.log)
```

Per-crate court (the W6-divergence FileSink extra):
`cargo doc -p cgap-parquet ...` EXIT=0 (/tmp/dod02fix-parquet.log).

Build fence: `cargo build --workspace` EXIT=0 (/tmp/dod02fix-build.log).

## W3 — Per-error before/after (DOD-02 table, all doc-side)

### cgap-arrow (11/11)
| loc | class | fix |
|---|---|---|
| blake3.rs:161,222 | PIL | `[`hash_chunk_window`]` → `` `hash_chunk_window` `` |
| slice.rs:5,46 | REL | `[`Primitive`](crate::primitive::Primitive)` → `[`Primitive`]` |
| slice.rs:7,15,48 | AMB | `[`slice`]` → `[`fn@slice`]` |
| slab.rs:26-28 | BIDL | three cross-crate paths de-linked to backticks |
| foreign.rs:12 | BIDL | `[`ForeignArray`]` → backticks |

### cgap-bn3 (2/2)
arena.rs:32 `[`header_index`]`→bt; unify.rs:13 `[`SplitStore`]`→bt.

### cgap-bont (4/4)
primitives.rs:23 `GOLDEN[2]`→`GOLDEN\[2]`; shacl_compiler.rs:29,376
`production [8]`→`\[8]`; shacl_compiler.rs:826 `[`KNOWN_PROP_LOCALS`]`→bt.

### cgap-git (6/6)
dag.rs:82 test-name link→bt; admission_gate.rs:12,132 PktError→bt;
ref_store.rs:17 `[dependencies]`→`` `[dependencies]` ``; retract.rs:22
`alt_support[F]`→`\[F]` (the doc's `[`F`]` row was the WRONG bracket —
actual shortcut link was `alt_support[F]`), retract.rs:127
`keep[f]`→`\[f]`.

### cgap-p28 (1/1)
asc450.rs:356 `[`Lease`]`→bt (Lease not resolvable at that site).

### cgap-parquet (6/6 incl. FileSink extra)
byteslice.rs:20,84 eq_mask_word→bt; scan.rs:321,329 private→bt;
footer.rs:62 `[`MAX_ROW_GROUPS`]`→full path
`crate::mmap_writer::MAX_ROW_GROUPS`; mmap_writer.rs:603 `[`FileSink`]`→bt
(FileSink is `#[cfg(feature = "std")]`-gated; unresolved under the doc
run's default features — DOD-02 W6 divergence CLOSED with cause).

### cgap-storage (2/2)
dred.rs:52 `keep[f]`→`\[f]`; closure.rs:103 `edges_x[j]`→`\[j]`.

## W4 — Scope extension (disclosed): crates outside the DOD-02 table

The DOD-02 run died on the first failing crate wave ("waiting for other
jobs"); these surfaced only after the 7-crate fixes landed. Same
doc-side classes, no logic touched:
- cgap-ffi sovereign.rs:523 — `lane[i]` → `lane\[i]` (shortcut link).
- cgap root pipeline.rs:52,128,134 — private/gated targets de-linked to
  backticks (`P28LicenseAttestation::validate_intrinsic`,
  `Self::ingress_fn` ×2).
- bench bitslice_bench.rs:20 — `[0,m]` → `\[0,m]`; bench
  arrow_accessors_bench.rs:17,22 — `[`slice::slice`]` → plain
  `cgap_arrow::slice::slice`, `[`RunEndEncoded::get`]` → backticks.

## W5 — Latent raw-angle census (W4c)

- mmap_writer.rs:1603 — FIXED: `k<N arms` → `k `<` N arms` (was outside
  a code span; genuinely failable under -D warnings).
- isomorphism.rs:40, csct_runner.rs:37 — left AS-IS (D2): both sit
  INSIDE code spans, where rustdoc HTML-escapes by construction and
  cannot fire; the dispatch's "escape" would have rendered literal
  backslashes (`tier=\<t\>`) into the docs — a rendering regression to
  fix a non-defect.

## W6 — Divergences

1. Scope extension (W4 above): cgap-ffi/cgap/bench were not in the
   DOD-02 table; their errors existed behind the parallel abort. Doc-only,
   disclosed here. cgap-ffi files were in-flight for other lanes (git
   status); my diff is exactly the one line at sovereign.rs:523 — owner
   supersedes on landing.
2. W5 as described: 1 of 4 latent hits fixed, 2 proven non-failable
   (code-span fenced), csct/mmap 1603 fixed. Net: 2 edits, not 4.
3. Error count ran 31→40 located across the session (waves hidden by
   "waiting for other jobs"), not 31 total; all 40 fixed, gate exit 0.

## W7 — Standing

**ALIVE** — gate exit 0 witnessed this session (command + exit in W2);
per-crate parquet exit 0; build fence exit 0; doc-only diff (W3/W4).

## W8 — Replay

```
RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links" \
  CARGO_TARGET_DIR=/tmp/dod02fix-target \
  cargo doc --workspace --no-deps --document-private-items   # EXIT=0
RUSTDOCFLAGS="-D warnings" CARGO_TARGET_DIR=/tmp/dod02fix-target \
  cargo doc -p cgap-parquet --no-deps --document-private-items # EXIT=0
cargo build --workspace                                        # EXIT=0
```
