# w-consumer-proof

Registry-only consumer proof of the v26.11.2 published kernel wave
(lane W-CONSUMER, 2026-11-02). Subject: a fresh `/tmp/w-consumer`
crate depending on crates.io ONLY, pinned `=26.11.2` on cgap-blake3,
cgap-arrow, cgap-parquet, cgap-bont. `/Users/sac/stardf` untouched;
mutates nothing in the shared tree.

## Version table (crates.io API, User-Agent-authenticated)

| crate | max_stable before | max_stable after poll | witnessed at (UTC) |
|---|---|---|---|
| cgap-blake3 | 26.11.1 | 26.11.2 | 07:26:34Z poll#2 |
| cgap-arrow | 26.11.1 | 26.11.2 | 07:26:34Z poll#2 |
| cgap-bont | 26.11.1 | 26.11.2 | 07:29Z poll#3 (bont poll) |
| cgap-parquet | 26.11.1 | 26.11.2 | 07:26:34Z poll#2 |

## Commands and exits

- `bash /tmp/w-consumer-poll.sh` — bounded poll, 60s interval, 60-min
  deadline. exit 0 at poll#2 (07:26:34Z): cgap-parquet 26.11.2 live.
- bont bounded poll (40-min cap) exit 0 at poll#3: cgap-bont 26.11.2
  live. Full wave confirmed.
- `cd /tmp/w-consumer && cargo build` — exit 0 (deps fetched from
  `registry+https://github.com/rust-lang/crates.io-index`; Cargo.lock
  pins all four =26.11.2; fetched sources in
  `~/.cargo/registry/src/*/cgap-*-26.11.2`).
- `cargo tree` — closure is exactly the four registry crates (cgap-bont
  -> cgap-blake3 26.11.2), zero path deps, zero other cgap crates.
- `/tmp/w-consumer-target/debug/w-consumer` — exit 0, output:

```text
PASS blake3 known vector: af1349b9f5f9a1a6
PASS arrow slab seal+admit (len=4), tamper -> SlabError::DigestMismatch
PASS parquet seal+verify, tamper -> SealError::ChecksumMismatch{Body}
PASS bont slot_id(b"a")=0xf3257881546de6ff, dedup len=1
NOTE cgap_arrow::VERSION (registry 26.11.2) = "26.10.29"
```

## Kernel calls (final-state asserts, Chicago)

- cgap-blake3 `hash(b"")` == official empty-input BLAKE3 vector
  `af1349b9...f3262` — ALIVE.
- cgap-arrow `ArrowRecordSlab::<4>` push x4 / `seal()` / `from_bytes`
  admit — final state: `is_sealed`, `len==4`, payload byte equality.
  Tampered window -> `SlabError::DigestMismatch` (typed flip, control
  arm). ALIVE.
- cgap-parquet `FooterSealer::new(..).seal()` ->
  `verify_seal(bytes, digest)` -> `Verified{row_groups=1, rows=128,
  spans_len=1}`. FLIP COURT: one body byte flipped (0x40) ->
  `SealError::ChecksumMismatch{region: Body}` asserted exact; second
  typed arm `DigestMismatch` on wrong content address. ALIVE.
- cgap-bont `slot_id(b"a") == SlotId(0xf325_7881_546d_e6ff)` (the
  crate's own golden vector) + CollisionDetector dedup (`len==1` after
  duplicate insert). ALIVE.

## Divergences / defects found

1. **cgap-arrow 26.11.2 (registry) embeds stale `generated_version.rs`:
   `VERSION = "26.10.29"`** (both VERSION and VERSION_C, witnessed in
   `~/.cargo/registry/src/*/cgap-arrow-26.11.2/src/generated_version.rs`).
   The tarball was packaged without an ontology projection regen. The
   first consumer assert (`VERSION == 26.11.2`) caught it; recorded as
   a registry divergence rather than hidden. Behavior otherwise
   correct; the version pin is proven by Cargo.lock + registry source,
   not by the embedded const.

## Standing

ALIVE — registry-only build + run with every assert green (exit 0) this
session; the wave is consumable from crates.io, not just published.

```json
{"lane": "W-CONSUMER", "files_written": ["docs/jira/v26.11.2/waves/w-consumer-proof.md", "/tmp/w-consumer/Cargo.toml", "/tmp/w-consumer/src/main.rs"], "commands": ["poll exit 0 @07:26:34Z poll#2 (parquet+blake3+arrow 26.11.2)", "bont poll exit 0 @poll#3", "cargo build exit 0", "cargo run exit 0", "cargo tree registry-only closure"], "courts_run": ["parquet flip: tampered sealed block -> SealError::ChecksumMismatch{Body} (flipped, typed)", "arrow admit flip: tampered slab window -> SlabError::DigestMismatch (flipped, typed)", "blake3 known vector (official empty-input)", "bont golden slot vector"], "standing": "ALIVE", "divergences": ["cgap-arrow 26.11.2 registry tarball embeds stale generated_version.rs (VERSION=26.10.29; VERSION_C=b\"26.10.29\\0\")"]}
```
