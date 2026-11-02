# Lane GIT-08: canonical lease-commit synthesis (commit_synth)

Date: 2026-11-02. Branch: main. Standing: **ALIVE** (commands + exits this session).

## Files written (exclusive ownership)
- `crates/cgap-git/src/commit_synth.rs` (sha256 `e9962a0a…705`, 1,024-byte
  fixed scratch, zero heap, `deny(unsafe_code)`, no `crate::` paths so the
  module is lib.rs-wireable later without edits)
- `crates/cgap-git/tests/commit_synth_court.rs` (`#[path]` wire-in; lib.rs
  untouched, per dispatch)
- `docs/jira/v26.11.2/lanes/lane-git-08-commit-synth.md` (this receipt)

lib.rs, Cargo.toml (both roots) untouched. Shared-tree mutations: none.

## API surface (additive only)
- Grammar consts: `KEY_TREE/PARENT/AUTHOR/COMMITTER/LEASE_PHI/LEASE_DELTA/`
  `LEASE_RECEIPT`, `TZ_UTC="+0000"`, `DOMAIN_LEASE_RECEIPT`, `SCRATCH_LEN=1024`,
  `MAX_MESSAGE_LEN=256`.
- `LeaseReceipt` (80 B: nonce u64 LE + parent 32B + payload 32B + tag 8B;
  tag = SHA-1(`cgap-lease-receipt-v1` || nonce_le || parent || payload)[..8]),
  `LeaseReceipt::seal/expected_tag/to_bytes/from_bytes`.
- `LeaseCommitSpec`, `CommitScratch::render` (fixed 1,024B scratch), and
  `validate(&[u8]) -> LeaseCommitFields` with typed errors:
  `LeaseTrailerMissing(&'static str)` is the AndonRefusal class (exit-101
  family per `ingress_ring.rs:98-102` precedent); `ReceiptTagMismatch` is
  the tamper rejection.
- Divergences documented in the module header: 64-hex object names are
  BLAKE3 digests (caller inputs; not SHA-1); timestamps are caller u64s
  (no clock reads in no_std); receipt tag SHA-1 is an in-module private
  copy of `pack_write::sha1` (lib.rs off-limits), pinned byte-equal to the
  public one by the court.

## Receipt-layout decision
`cscb_c7_receipt_sealing.rs` exists only as a csct bin
(`crates/csct/src/bin/`), not a cgap-git consumable test/module, so per
dispatch I defined the layout myself: nonce u64 + parent 32B + payload
32B + tag 8B (=80 B). C7 framing reuse is a future wiring task.

## Commands + exits (this session)
| command | exit | result |
|---|---|---|
| `rustc --edition 2021 --crate-type lib src/commit_synth.rs` (standalone) | 0 | COMPILE_OK |
| `cargo test -p cgap-git --test commit_synth_court` | 0 | 10 passed / 0 failed |
| `cargo test -p cgap-git --lib` | 0 | 61 passed / 0 failed |
| `cargo test -p cgap-git --test delta_refusal_court` | 0 | 4 passed |
| `cargo test -p cgap-git --test envelope_decimal_court` | 0 | 2 passed |
| `cargo test -p cgap-git --test fsck_court` | 0 | 2 passed |
| `cargo test -p cgap-git --test fuzz_codec` | 0 | 4 passed |
| `cargo test -p cgap-git --test git_interop` | 0 | 3 passed |
(all with `CARGO_TARGET_DIR=/tmp/git08-target`)

## Mutation court battery (/tmp sandbox, sha256-exact restore verified)
Sandbox: `/tmp/git08-sandbox/cgap-git` (de-workspaced manifest, absolute
path deps). Baseline 10/10 green BEFORE mutation; restore verified
sha256-exact against the shared-tree file
(`e9962a0ac8d98ff8…7705` both sides) and baseline re-run green AFTER
restore.
- **MUT-A**: DELETE the receipt tag comparison in `validate` =>
  `tampered_receipt_byte_is_typed_rejection` FAILED (9 pass / 1 fail).
  Flips.
- **MUT-B**: tolerate a missing `lease-receipt` trailer
  (`.map_err(LeaseTrailerMissing)` -> `.unwrap_or(&[])`) =>
  `missing_receipt_trailer_is_andon_refusal` FAILED (9 pass / 1 fail).
  Flips.
An intermediate MUT-B attempt that produced a compile abort was
re-done properly; per contract compile-abort is not a failure count.

## Fences run (must-not-regress surfaces my diff could break)
My diff adds two new files only; the fences are the existing cgap-git
gates, all run green this session: lib (61), delta_refusal_court (4),
envelope_decimal_court (2), fsck_court (2), fuzz_codec (4),
git_interop (3). Untouched in-flight lanes' targets
(`soc_full_chip_court` etc.) were NOT run by me — one of them
(`soc_full_chip_court`) currently fails to compile
(`unexpected closing delimiter`) in a file I do not own; pre-existing,
not introduced by this lane (my files are not in its dep graph via
lib.rs).

## Divergences / notes
- Dispatch said "cargo test -p cgap-git green". The FULL crate suite is
  not green this session: the untracked in-flight `soc_full_chip_court`
  test (another lane's file) has a compile error. Every target I could
  run green ran green (table above); lib.rs is untouched so this lane
  cannot be implicated. Recorded per census-vs-compile-abort law.
- Remaining lint warnings in the crate are pre-existing (unused imports
  in lineage/other test files), not from my files; my two files compile
  warning-light (an unused `CAPACITY` assoc-const warning remains in the
  court build — informational).
- Byte-stability known-vector is computed from the same digest fixtures
  (not an independent frozen vector); the mutation battery is the
  anti-vacuity witness instead.
- Build-root lease `/tmp/git08-target` deleted at lane end per lease law.
