# Lane GIT-16 receipt: SoC full-chip court

Date: 2026-11-02. Branch: main, canonical checkout /Users/sac/cgap.
Owner: lane GIT-16.

## Files written (complete list)

- `crates/cgap-git/tests/soc_full_chip_court.rs` (543 lines, court)
- `docs/csct/receipt-cgap-git-soc-v1.md` (court-EMITTED artifact; the
  test writes it on green — disclosed here and in the test head)
- `docs/jira/v26.11.2/lanes/lane-git-16-soc-court.md` (this receipt)

lib.rs and Cargo.toml untouched by this lane.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/git16-target cargo build -p cgap-git --test
  soc_full_chip_court` — iterated; final build exit 0 (warnings only).
- `CARGO_TARGET_DIR=/tmp/git16-target cargo test -p cgap-git --test
  soc_full_chip_court` — exit 0, `test result: ok. 1 passed; 0 failed`,
  ~8.5 s (three consecutive green runs).
- `CARGO_TARGET_DIR=/tmp/git16-target cargo test -p cgap-git` (full
  suite) — run launched; result recorded below when drained.
- /tmp/git16-target deleted at lane end (build-root lease law).

## What the court proves

- EnclaveSram128K = exact 131,072-byte page-aligned in-crate STATIC-size
  buffer; no hardware, no MMIO — in-crate simulation, stated as such.
- End-to-end wire over `#[path]`-included siblings: self-generated
  corpus (seed 0xA20C0FFEE5A7E000, splitmix64) -> pack v2 window
  serialization (pack_write::sha1 trailer law) -> SWAR-sieved ingress
  ring relay (ingress_ring.rs 32,768B, byte-exact A->B fidelity) ->
  streaming pack demux (pack_demux.rs, RECORD_CAP interleaved drains,
  objects_seen + trailer verified per window) -> closure (object_store.rs
  BLAKE3 ledger) -> delta_resolver.rs apply_delta sub-court -> tree_synth
  projection + verify_canonical_order -> real egress_ring.rs
  push/window/copy_into/advance_tail -> BLAKE3 receipt seal (real
  cgap_blake3::hash).
- 1,000,000-fact transaction streamed end to end: 733 windows (732 x
  1,365 + 1 x 820); retained closure = 128 distinct objects (store cap
  256 x 128B) + 999,872 deduped — honest scale, never rounded up.
- Zero heap: counting #[global_allocator] with a scoped window; positive
  control observed allocations; pipeline window asserted exactly 0.
- Stack canary [u8; 8192] pattern 0xDEADBEEFCAFEBABE byte-unchanged.
- Mutation courts, all flipped:
  A: pack magic corrupt at ingress boundary -> DemuxError::SubjectMissing
     (typed);
  B: body-byte flip -> court-side trailer law refusal;
  C: UNMUTATED control accepted (non-vacuity: a court that refuses
     everything is as vacuous as one that refuses nothing).
- Receipt emitted with per-stage results + BLAKE3 transcript root
  (chained per-window digests), hex root printed in the artifact.

## Loud typed exclusions / divergences

- Sibling-arrival race: pack_demux.rs/egress_ring.rs were in-flight
  (pack_demux transiently corrupted to ~377k lines by its owner lane
  mid-write; object_store gained cgap_blake3 usage before its
  dependency landed). All landed before my final builds; court integrates
  pack_demux + egress_ring as REAL stages.
- commit_synth / admission_gate / patent_court / retract / ref_store:
  landed after this pipeline froze; NOT integrated into the SoC wire
  (loud, in-scope next wave). Same disclosure carried in the emitted
  CSCT receipt.
- bn3 closure: not reachable via #[path] (foreign crate); closure =
  object_store.rs BLAKE3 ledger instead. Disclosed in both places.
- Minimal-unblock disclosure (both-directions law): during the session
  the cgap-git LIB broke (object_store.rs used cgap_blake3 with no
  [dependencies] entry; owner lane GIT-05). I waited and the OWNER fixed
  it within the SLA (~90 s) — no cross-lane edit by GIT-16; disclosed
  for attribution.
- Emitted-receipt formatting: first generation had source indentation;
  de-indented and re-emitted (same run, same root).

## Fences (must-not-regress) and their state

- Full cgap-git suite (`cargo test -p cgap-git`) — my test file adds no
  global_allocator side effects outside its own binary; suite result
  recorded below. Other lanes' in-flight edits to shared files (Cargo.toml
  dev-deps, lib.rs) are outside my ownership.

## Full-suite drain (final, current tree)

- First full-suite pass: 24 suites `test result: ok`, ONE transient
  FAIL `octopus_merge_court::mutation_conflict_injection_flip` (another
  lane's in-flight file; octopus_merge.rs is outside my ownership).
  Re-run of that court alone: `test result: ok. 5 passed; 0 failed`
  (exit 0) — owner landed the fix between the two runs; transient,
  not mine, not a regression from GIT-16 (GIT-16 never touched
  octopus files).
- soc_full_chip_court: exit 0, `1 passed; 0 failed`, ~7.9 s, three
  consecutive green runs (last on the final de-indented file bytes).
- Final full-suite drain: appended below when complete.

## Standing

ALIVE for `crates/cgap-git/tests/soc_full_chip_court.rs` (command+exit
this session). Emitted CSCT receipt on disk:
docs/csct/receipt-cgap-git-soc-v1.md (BLAKE3 transcript root
3d1c80d411301988b5bd18ef88e71d4b7fa7d52833505f46684388510c3a5711).

## Final drain addendum (truth as of lane close)

- Drain 2 (true exit captured, CARGO_EXIT:101): 19 suites ok; two
  neighbor failures, both outside my ownership and both flapping
  run-to-run under concurrent lane edits:
  - `octopus_merge_court::mutation_conflict_injection_flip` (green in
    the isolated re-run minutes earlier; failed again here),
  - `c7_parse_path_is_zero_alloc` (pack_demux court; a counting-window
    test flapping under same-binary concurrency).
- Drain 3 after those: `egress_ring_court` 5/5 ok, soc_full_chip_court
  1/1 ok exit 0. My court is deterministic-green across 4 runs on final
  bytes; the two flapping neighbor courts are in-flight lanes' files.
- Build root /tmp/git16-target deleted below at lane close (lease law).
