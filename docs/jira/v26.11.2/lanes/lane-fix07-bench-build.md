# lane-fix07-bench-build

Lane: FIX-07 (hot — workspace build RED). Repo `/Users/sac/cgap`, branch main. No git commands run.

## Subject

Collateral damage from the C05 warning sweep in `crates/bench`:

1. `crates/bench/benches/chains-parquet.rs:467` — C05 renamed a used
   binding to `_block_len`; its consumer at `:479`
   (`assert_eq!(&mem_probe[..block_len], ...)`) then failed E0425.
   The binding WAS genuinely read → real use excised → restore the
   name (`block_len`), do not underscore the consumer.
2. `crates/bench/benches/chains-seal.rs:492` — C05 removed `mut` from
   the closure's `let lease = lease.lock()...`; consumer at `:493`
   (`seal_unsealed(delta, &mut lease)`) failed E0596. Restore `mut`.
3. Second C05 leftover found during warnings census:
   `chains-seal.rs:478` — outer `let mut prev_root = issued_root` was
   declared `mut` but only shadowed copies are mutated inside the
   closure → dropped `mut` (zero-warning law, no behavior change).

## Diff (3 lines total)

- `chains-parquet.rs:467` `_block_len` → `block_len` (restores real use; line 430's `_block_len` is genuinely unused, left as-is).
- `chains-seal.rs:492` `let lease` → `let mut lease` (restores E0596 fix).
- `chains-seal.rs:478` `let mut prev_root` → `let prev_root` (kills `unused_mut` warning).

## Commands / exits (this session, this tree)

| Command | Exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/fix07-target cargo build --workspace --all-targets` | 0 |
| `CARGO_TARGET_DIR=/tmp/fix07-target cargo build -p bench --all-targets` | 0 |
| `CARGO_TARGET_DIR=/tmp/fix07-target cargo build --release -p bench --all-targets` | 0 (45.06s) |
| `CARGO_TARGET_DIR=/tmp/fix07-target cargo build -p bench --bench chains-seal --bench chains-parquet` | 0, 0 warnings naming chains-seal.rs/chains-parquet.rs (grep -c = 0) |

## Warnings census (fresh build, bench crate)

Remaining bench-surface warnings are PRE-EXISTING, none in the two
owned files:

- `bench (manifest)` ×3 unused deps (`cgap-ffi`, `cgap-git`,
  `cgap-storage`) — pre-existing manifest lint, other lanes' surface.
- `cgap-git` lib ×10 (unused items, `#![no_std]`-at-non-root,
  deprecated `fetch_update`) — pre-existing, cgap-git lane's surface.
- The two bench files: 0 warnings (verified via grep of build output).

## C05 misfire attribution (disclose-in-both)

C05's warning sweep excised two genuinely-used bindings and left an
`unused_mut`. This is a C05 misfire: correctness over silence. C05's
owner must amend its receipt citing this file
(`docs/jira/v26.11.2/lanes/lane-fix07-bench-build.md`); this receipt is
the disclosure per the disclose-in-both law.

## Standing: ALIVE

Build was exit 101 (EXP-10 audit); now exit 0 twice witnessed this
session. Build root `/tmp/fix07-target` lease deleted at lane end.
