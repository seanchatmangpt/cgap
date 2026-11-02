# Lane POSH-3 Receipt — ETag Court (crates/csct)

W7 fields per `docs/jira/v26.11.2/_LANES-TEMPLATE.md`.

## 1. Subject
- Repo `/Users/sac/cgap`, branch main. Files written:
  - `crates/csct/tests/posh_etag_court.rs` (NEW, 163 lines)
  - `docs/jira/v26.11.2/lanes/lane-posh-03-etag.md` (this receipt)

## 2. Authority
Lane dispatch POSH-3; files scoped to the two paths above. No git
commands run. `crates/csct/Cargo.toml` read fresh (already declares
`cgap-blake3` path dep — no manifest edit needed).

## 3. Consequence
- `format_azure_etag(delta, buf) -> Result<usize, EtagFmtError>`:
  formats `W/"<64-hex>"` (69 bytes) of a real `cgap_blake3::hash` digest
  into a caller-provided fixed buffer, no heap for the wire write; typed
  `EtagFmtError::BufferTooSmall { needed, got }` on undersized buffers.
- Round-trip `parse_etag_body(buf) -> Option<[u8; 32]>`.

## 4. Commands + exits (this session)
- `cargo test --test posh_etag_court` (cwd crates/csct,
  CARGO_TARGET_DIR=/tmp/posh3-target): exit 0,
  **5 passed; 0 failed**:
  - court1_determinism_identical_deltas_byte_identical_etags ok
  - court2_one_attribute_mutation_distinct_digest ok
  - court3_wire_format_shape_and_roundtrip ok
  - court4a_undersized_buffer_typed_err_flip ok
  - court4b_stale_cached_etag_mismatch_detected_flip ok

## 5. Replay
Reproduce with:

```bash
cd /Users/sac/cgap/crates/csct && \
  CARGO_TARGET_DIR=/tmp/posh3-target cargo test --test posh_etag_court
```

Anti-vacuity (court4a/4b are the named flips): shrink the buffer law in
`format_azure_etag` to `buf.len() < ETAG_WIRE_LEN - 1` and court4a
fails; make `parse_etag_body` ignore the revision attribute (compare
caches by resource_id only) and court4b fails. Mutation battery run
in-court (court2) over the real hasher — no sandbox copy needed, the
mutants are in-test delta values, shared tree untouched.

## 6. Standing
- Tests: **ALIVE** (command + exit above, this session).
- Azure wire parity: **UNVERIFIED, NOT claimed** (fence g). The
  `W/"..."` form is pinned as fixture const `AZURE_ETAG_WIRE_FORM`
  in the test file; no in-tree receipt grounding real Azure quoting
  semantics was located, so the court proves fixture self-consistency
  + digest determinism only.
- Perf: no perf claims made; none measured.

## 7. Fences / divergences
- Fences: none touched — new test file only; other csct tests
  (`csct_coverage_court`, `csct_pipeline_court`) and csct bins still
  compiled as part of the same cargo invocation (compilation of the
  full crate succeeded; only dead-code warnings from pre-existing
  bins). No shared-file edits, no unblocks applied to other lanes.
- Divergence: dispatch said "check Cargo.toml fresh" for cgap-blake3 —
  confirmed present (line 6), zero manifest changes.
- Build-root lease `/tmp/posh3-target` deleted at lane end.
