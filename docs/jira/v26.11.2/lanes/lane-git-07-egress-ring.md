# Lane GIT-07: Egress ring (synthetic Arrow IPC / Parquet page serve)

- **Lane**: v26.11.2 GIT-07 (retry; prior instance died on API 429 with
  zero partial files on disk, verified clean)
- **Repo**: /Users/sac/cgap @ main
- **Files written** (only these):
  - `crates/cgap-git/src/egress_ring.rs` (611 lines, 22,713 bytes)
  - `crates/cgap-git/tests/egress_ring_court.rs` (~320 lines, 12,837 bytes)
  - this receipt
- **lib.rs**: NOT edited (dispatch law). The module is self-contained
  (no `crate::` paths) and is wired into the test binary via `#[path]`,
  byte-faithful to how lib.rs will wire it later.
- **API**: additive only (crate is publish=true); lib compiles untouched.

## Framing-constant reuse (stated per dispatch)

- Arrow IPC: magic `0x4950_4331` ("IPC1"), 8-byte prefix, 64-byte meta
  block, layout version 1 — REUSED from `crates/cgap-arrow/src/ipc.rs`
  (`MAGIC` / `PREFIX_SIZE` / `META_HEADER_SIZE` / `LAYOUT_VERSION`),
  restated in-module with source pins because cgap-git is a zero-dep
  kernel (same law as ingress_ring.rs).
- Parquet: `b"PAR2"` magic — REUSED from
  `crates/cgap-parquet/src/footer.rs:248`, restated with pin. The page
  body is SYNTHETIC (magic + length + stored payload); NOT a conformant
  Parquet file, no such claim made; no new Parquet format invented.
- Index/publish law reused from `ingress_ring.rs` (free-running
  counters, fail-closed overflow, andon class 101).

## Courts (all ran this session, exit 0)

`CARGO_TARGET_DIR=/tmp/git07-target cargo test -p cgap-git --test
egress_ring_court` — 5 passed, 0 failed, 0 warnings, exit 0:

1. `roundtrip_byte_exact_court` — 40 records pushed across the 32,768-B
   wrap boundary; window gather byte-exact; IPC page served, parsed
   back zero-copy; payload + all 4 descriptors byte-exact. Mutation
   flip: one payload byte corrupted in the blob, parser still accepts
   the frame but the byte-exact assert now FAILS (assert_ne pinned).
2. `flip_twin_wrap_segment_shape_court` — pins the wrap actually
   happened (two segments: 32 B at the buffer's final stride + 32 B at
   the front). A regression making windows always contiguous flips this
   court.
3. `misaligned_column_rejection_court` — dispatch mutation target:
   schema descriptor offset 16 -> 17 on a width-8 column rejected
   `MisalignedColumn{index:2, offset:17, width:8}` on THREE surfaces
   (validate, wire parse, serve). Flip twin: valid schema yields Ok on
   the same surfaces; outcomes differ = gate live. Andon class 101
   pinned. Rejects on the WIRE (splice into a valid page blob), not
   just the struct.
4. `zero_alloc_serve_court` — INSTRUMENTED (stated per dispatch):
   process-wide counting `GlobalAlloc` (System-backed), allocation
   delta asserted EXACTLY 0 across `window`, `serve_ipc_page`,
   `parse_page`, and `serve_parquet_page`. Non-vacuity control:
   `snapshot_owned` (the module's one allocating path) shows delta > 0
   in the same harness — instrument-proves-it-fires, not a static
   claim.
5. `parquet_page_reuse_court` — `PAR2` magic byte-pinned at offset 0,
   u32 payload length pinned at offset 4, payload byte-exact vs the
   zero-copy window.

## What is proven vs aspirational

- PROVEN (instrumented): zero heap allocation and zero re-serialization
  on the serve path (header is a fixed memcpy encode; payload is a
  single gather of already-wire-serialized stored bytes; `window` and
  `parse_page` are structurally zero-copy `&[u8]` borrows).
- ASPIRATIONAL, NOT CLAIMED: wire-level "git cat-file streams" framing.
  The blobs are synthetic IPC-shaped / Parquet-flavored pages served
  from a fixed ring. No git-stream claim is made.

## Commands and exits

| command | exit |
|---|---|
| `cargo test -p cgap-git --test egress_ring_court` | 0 (5 passed) |
| `cargo test -p cgap-git --lib` | 0 (61 passed) |
| `cargo test -p cgap-git --test {ingress_ring,fsck,pkt_line,object_store,ladder_mutations,packfile_fuzz,git_interop}_court…` | 0 (2/3/6/11/6/2/10 passed) |
| `cargo test -p cgap-git --test {delta_refusal,envelope_decimal,fuzz_codec,ladder_timing_stress,large_offset,lineage_workflow,mcp_driver,pack_demux}…` | 0 (4/2/4/0+1 ignored/6/1/6/10 passed) |
| `cargo test -p cgap-git` (all targets incl. soc_full_chip_court) | 1 — compile abort in `tests/soc_full_chip_court.rs` (E0204/E0502), ANOTHER lane's in-flight file, not mine; compile-abort ≠ failure count (census law). Disclosed, not repaired (not my file). |

All CARGO_TARGET_DIR=/tmp/git07-target; root /tmp/git07-target deleted
at lane end per lease law.

## Standing

**ALIVE** for the lane files (commands + exit 0 this session, exact
subjects above). `soc_full_chip_court` compile abort is a pre-existing /
concurrent-lane condition, disclosed per compile-freeze SLA; its owner
owns the fix.

## Divergences from dispatch

1. Receipt path: dispatch named
   `docs/jira/v26.11.2/lanes/…` (contract default is `waves/`);
   followed dispatch.
2. Sibling lane reality fence: `G03 ingress_ring.rs` had already landed
   at lane start; reuse pins cite it as a landed file, not concurrent.
3. `DESC_ENTRY_BYTES`/`DESC_TABLE_OFFSET` used by the court; constants
   exported pub for that purpose (additive).
4. `advance_tail` was added beyond the minimum API — required by the
   wrap flip-twin to construct the wrap state deterministically
   (fail-closed typed refusal, same law as push).
5. Zero-alloc proof: INSTRUMENT (tracking GlobalAlloc), as the dispatch
   offered; static-invariant-only would have been vacuous and was not
   used.

## Fences (must-not-regress, run this session)

- Existing cgap-git suites: all green (counts above) EXCEPT
  `soc_full_chip_court` (another lane's in-flight compile abort,
  disclosed above — this is the fence state as-left, not a regression
  introduced by this diff).
- `cargo test -p cgap-git --lib` 61/0 — lib.rs untouched, still green.
