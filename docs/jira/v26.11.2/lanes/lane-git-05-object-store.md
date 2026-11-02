# Lane GIT-05 Receipt: no_std BLAKE3 Object Ledger (`object_store.rs`)

Wave: v26.11.2 starDF · Branch main · Date 2026-11-02
Standing: **ALIVE** (court run green this session; commands + exits below)

## Files written (owned, exclusive)
- `crates/cgap-git/src/object_store.rs` (362 lines)
- `crates/cgap-git/tests/object_store_court.rs` (228 lines)
- `docs/jira/v26.11.2/lanes lane-git-05-object-store.md` (this receipt)

## Divergence resolved honestly (dispatch-named)
The dispatch proposed Git's canonical `blob <size>\0<data>` SHA-1 header.
That is canonical **Git SHA-1 loose-object** addressing; this crate's law is
BLAKE3 content addressing. Implemented instead: `"<tag> <len>\n" ++ payload`
framing, digested with the real `cgap_blake3::hash` (dev-dep path dep). No
Git parity claimed; the divergence is named in
`CANONICAL_HEADER_LAW` (object_store.rs:35) and asserted in
`zero_heap_static_invariant` (court line ~200). If Git-interoperable loose
objects are ever needed, the SHA-1 path already exists in-crate
(`pack.rs`/`inflate.rs` SHA-1).

## API surface (additive only; lib.rs untouched — `#[path]` wiring)
- `MAX_OBJECTS=256`, `MAX_PAYLOAD=128`, `Digest=[u8;32]`
- `ObjectKind{Commit,Tree,Blob,Tag}` (tags match Git vocabulary; digest law does not)
- `StoreError{Full, Duplicate(Digest), NotFound(Digest), DigestMismatch{expected,actual}, BufferTooSmall{needed,have}, PayloadTooLarge{len,max}}` — typed, never panics on input
- `digest_of`, `write_frame`, `frame_len`, `CANONICAL_HEADER_LAW`
- `ObjectStore::{new,len,is_empty,slots,slots_mut_for_mutation_court,eytzinger,find,insert,insert_checked,get,verify,check,ledger_digest}`
- Real Eytzinger layout: BFS index `eyt[1..=count]` over digest-sorted slots, rebuilt on insert; search is the classic `k = 2k + (a[k] <= x)` descent with `k >>= trailing_zeros()+1` fallback. Found+fixed during this lane: the initial descent used `a[k] < x` (Equal descended left) which mis-lands the fallback candidate — the court flipped on it (witnessed: 5/6 fail before fix, 6/6 green after).
- Zero heap: no `Vec/String/Box/format!/alloc::` in the module; asserted as a static source-scan instrument in `zero_heap_static_invariant` (stated: source-scan, not allocator count — the test crate has no allocator hook).

## Commands + exits (this session, CARGO_TARGET_DIR=/tmp/git05-target)
- `cargo test -p cgap-git --test object_store_court` → **6 passed / 0 failed, exit 0** (three runs; final green)
- `cargo test -p cgap-git` aggregate → BLOCKED by a concurrent lane: `tests/delta_resolver_court.rs` fails to compile (E0308/E0277/E0621/E0502, references `dag.rs` delta resolver — that lane's in-flight file, `git status` shows `?? crates/cgap-git/src/a2a_swarm.rs` and modified `dag.rs`, none mine). Compile-abort ≠ failure count (census law).
- Per-target fence sweep (all exit 0): delta_refusal_court, envelope_decimal_court, fsck_court, fuzz_codec, git_interop, ladder_mutations, ladder_timing_stress, large_offset, lineage_workflow, packfile_fuzz, perf_baseline, object_store_court, lib tests.

## Courts (all flip-bearing)
1. `roundtrip_all_kinds_and_layout_is_real_eytzinger` — all 4 kinds byte-exact roundtrip; BFS layout law asserted by in-order traversal == sorted digest order.
2. `eytzinger_search_agrees_with_sorted_scan_present_and_absent` — 64 objects, hit parity eytzinger-vs-linear + typed misses on one-byte-perturbed keys.
3. **MUTATION COURT** `mutation_single_payload_byte_flip_yields_typed_digest_mismatch` — control Ok; one-byte payload flip ⇒ typed `DigestMismatch` at `check`, `insert_checked`, and `verify` seams; ledger digest flips. Control (intact) passes ⇒ not vacuous.
4. `ledger_digest_is_byte_stable_across_insertion_orders` — same set, different order ⇒ identical ledger digest; different content flips it.
5. `capacity_and_duplicate_courts` — Full at MAX_OBJECTS+1 (store uncorrupted), typed Duplicate pre-capacity, typed PayloadTooLarge.
6. `zero_heap_static_invariant` — banned-token sweep + framing-law witness (`blob 2\nhi`).

## Must-not-regress fences run
All 12 other cgap-git test targets + lib tests (exit 0 each, listed above); `lib.rs`, `Cargo.toml`, `dag.rs` untouched by this lane (git diff of lib.rs/dag.rs is other lanes' delta-resolver work, present before my first write).

## Cleanup law
Build-root lease `/tmp/git05-target` deleted at lane end:
`python3 -c "import shutil; shutil.rmtree('/tmp/git05-target', True)"` → exit 0 (witnessed this session).

## Standing
ALIVE for the named files: court green (6/6) with command+exit this session; aggregate `cargo test -p cgap-git` convergence is gated on the delta-resolver lane (not this lane's surface). Perf numbers: none claimed (no bench in scope). UNVERIFIED items: none withheld — no crash-path fuzz of ObjectStore itself (out of scope; fuzz_codec covers the pack codec).
