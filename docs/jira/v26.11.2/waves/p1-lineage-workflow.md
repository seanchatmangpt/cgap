# p1-lineage: statement lineage via Git (wave receipt)

Lane p1-lineage, v26.11.2, repo `/Users/sac/cgap` (one canonical
checkout, no git writes). Ground-truth charter:
`docs/jira/v26.11.2/12-ard-prd-cgap.md`. Status: **ALIVE**.

## Files written

- `crates/cgap-git/Cargo.toml` — added `[dev-dependencies]` path deps
  (`cgap-blake3`, `cgap-parquet`), same crate-as-dev-dep pattern as
  cgap-parquet's codec_properties court; `[dependencies]` stays empty
  (kernel zero-external-dep law intact; test-only, publish-safe).
- `crates/cgap-git/tests/lineage_workflow.rs` (new, 259 lines) — the
  cross-kernel workflow court.

## Workflow under test (all real collaborators, no mocks)

Seal 3 successive statement segments in the real cgap-parquet sealed
slab format (`FooterSealer` over `buf[..body_len]`; 189-byte slabs,
footer = meta block + 80-byte attestation slot + fnv1a64 checksum +
`PAR2` magic + footer length). Content-address each slab with the real
`cgap_blake3::hash` (distinct per segment; one flipped slab byte moves
the digest — non-vacuity witnessed in-court). Pack the 3 slabs as
packfile blobs (`build_pack_and_idx`); read back through cgap-git's own
readers: `parse_pack_v2` (3 objects, type 3, declared sizes == slab
lengths), per-object `deflate::zlib_decompress` (bodies byte-identical
to the sealed slabs), CRC32 over the entry spans, `IdxV2::parse` +
`verify_idx_checksum` + `lookup` (offsets/crcs agree, OID-keyed — the
idx writer sorts by OID, position-matching is wrong). Commit one
3-leaf tree via `commit_blocks` (genesis parent, node_count 2L-1 = 5);
commit a child snapshot parented on
`git_object_oid("tree", dag_file_a)` — 2 commits in a parent chain.
`decode_dag_file` round-trips both `.cgap-dag` wires byte-exact;
parent vs child decode shows different roots + different parent OIDs
(time-travel court: different content witnesses). Lineage proofs:
clean proof verifies; wrong parent -> `DagError::LineageMismatch`;
flipped sibling byte -> `DagError::RootMismatch` (both typed).

## Courts run (flip-or-FAIL, typed refusals)

- pack tamper (one flipped byte) -> `GitError::PackChecksumMismatch`;
- `.cgap-dag` tamper (byte 20, root region) -> `DagError::ChecksumMismatch`;
- sealed slab body tamper -> `SealError::ChecksumMismatch { region: Body }`;
- wrong content address -> `SealError::DigestMismatch`;
- wrong lineage parent -> `DagError::LineageMismatch`;
- tampered sibling path -> `DagError::RootMismatch`;
- non-vacuity: flipped slab byte moves `cgap_blake3::hash` output
  ("VACUOUS" panic guard asserts digest movement).

## Commands

- `CARGO_TARGET_DIR=/tmp/p1-lineage-target cargo test -p cgap-git --test lineage_workflow`
  -> exit 0, `1 passed; 0 failed` (zero warnings after unused-import
  sweep; final run witnessed this session).
- `CARGO_TARGET_DIR=/tmp/p1-lineage-target cargo test -p cgap-git`
  -> exit 0, `61 passed; 0 failed; 0 ignored` across the crate's unit +
  integration suites (pre-existing suites green; no regressions).

## Divergences (recorded, doc yields to reality)

1. `entries` from `build_pack_and_idx` are sorted by OID
   (write_idx_v2 sorts in place), NOT in blob order — position-matched
   reads mis-attribute CRCs. The court matches by content address.
2. `FooterSealer::new(buf, body_len, ..)` seals `buf[..body_len]` as
   the body; the caller writes the body into the buffer first. The
   helper does `buf[..body.len()].copy_from_slice(body)` explicitly.
3. The child commit's parent head is content-addressed as
   `git_object_oid("tree", dag_file_a)` — the A03 surface types the
   parent head as a bare `Oid` with no snapshot-head convention; the
   tree-object OID of the parent's `.cgap-dag` bytes is the
   content-bound choice made here (deterministic, replayable).

## Standing

ALIVE — commands above ran this session with the exits shown, against
the exact subjects on disk. Perf: UNVERIFIED (no perf claims made).
Lane target lease `/tmp/p1-lineage-target` removed at lane end.

```json
{"lane":"p1-lineage","files_written":["crates/cgap-git/Cargo.toml","crates/cgap-git/tests/lineage_workflow.rs","docs/jira/v26.11.2/waves/p1-lineage-workflow.md"],"commands":["CARGO_TARGET_DIR=/tmp/p1-lineage-target cargo test -p cgap-git --test lineage_workflow -> exit 0, 1 passed","CARGO_TARGET_DIR=/tmp/p1-lineage-target cargo test -p cgap-git -> exit 0, 61 passed; 0 failed"],"courts_run":["pack tamper -> PackChecksumMismatch","dag tamper -> DagError::ChecksumMismatch","slab body tamper -> SealError::ChecksumMismatch{Body}","wrong content address -> SealError::DigestMismatch","wrong lineage parent -> DagError::LineageMismatch","tampered sibling -> DagError::RootMismatch","blake3 digest movement non-vacuity"],"standing":"ALIVE","divergences":["build_pack_and_idx entries sorted by OID, not blob order — match by content address","FooterSealer body must be written into buf[..body_len] before sealing","child parent head = git_object_oid(tree, parent dag bytes) — Oid has no snapshot-head convention"]}
```
