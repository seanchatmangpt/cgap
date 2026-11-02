# lane-git-12-octopus-merge

Lane GIT-12 — N-head octopus merge over coordinate bitplanes (v26.11.2 starDF).

## Subject

- `crates/cgap-git/src/octopus_merge.rs` (NEW, 380 lines) — additive
  module, `#![no_std]`, `#![forbid(unsafe_code)]`, zero external deps
  (`[dependencies]` untouched). Module included by the court via
  `#[path = "../src/octopus_merge.rs"]` (lib.rs is off-limits to this
  lane; wire-in `pub mod octopus_merge;` in lib.rs is the
  coordinator's landing step — **not done by this lane**).
- `crates/cgap-git/tests/octopus_merge_court.rs` (NEW, 320 lines) —
  5-court binary over the real module (no mocks; counting-allocator
  zero-alloc court; child re-exec for the exit-101 witness).
- This receipt.

## API (additive)

- `HeadInput::try_new(id, s, p, o) -> Result<Self, MergeError>` (ragged
  SoA columns refuse typed).
- `octopus_merge(&[HeadInput], &mut [RowRef], &mut OctopusScratch,
  out_s, out_p, out_o) -> Result<usize, MergeError>` — single pass,
  12-bit partition radix (4096 buckets), `radix_bucket(s, p)` uses the
  bn3 seedless payload mix `y ^= y>>16; y & 0x0FFF` (bn3
  unify.rs:850-852 discipline) over the coordinate key. Identical
  triples unify; same (s,p) different o ⇒ `MergeError::DomainClash`
  naming both heads.
- `octopus_merge_andon(...)` (std) — refusal escalates to panic
  naming heads (process exit 101 class).
- Typed refusals: `HeadShape`, `TooManyHeads` (cap 20), `Capacity`,
  `DomainClash`.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/git12-target cargo test -p cgap-git --test
  octopus_merge_court` → exit 0, 5 passed / 0 failed (the intermediate
  `test result: FAILED` line is the court's own exit-101 child
  witness, by design — parent harness 5/5, cargo exit 0).
- Full-suite green-per-target spot gates: octopus_merge_court,
  delta_refusal_court, envelope_decimal_court, fsck_court,
  ladder_mutations — all exit 0.
- `cargo test -p cgap-git` (whole crate) → exit 101: **compile-abort in
  OTHER lanes' in-flight files** (`tests/delta_resolver_court.rs:176`,
  `tests/retract_court.rs`, `tests/pkt_line_court.rs`,
  `tests/ingress_ring_court.rs`, `tests/mcp_driver_court.rs`,
  `tests/patent_court_git.rs`) — compile-abort is not a failure count;
  not this lane's files, not fixed here (ownership law).
- Sandbox mutation court (subject = /tmp/git12-sandbox/cgap-git
  scratch-standalone copy, NOT the shared tree):
  - baseline: exit 0, 5/5.
  - MUTATION (SHIM-COURT): neuter the o-comparison in the conflict
    check (`if eo == o || true`) → exit 101, 5/5 courts fail (flip
    witnessed).
  - restore: sha256-exact
    (`0d7bb84b471209df9f0f8fd9bf4253df401273ba27ef4cd3cf26772694b2f5b7`,
    matches the shared tree byte-exact after restore), mtime bumped
    per mtime-restore law, post-restore rerun exit 0, 5/5.
- REPORT (measurement, not gate): 20-way fixpoint closure, best of 3,
  debug profile: **1873 us = 1.873 ms — UNDER the 33.3 ms release
  target by ~17.8x** (note: measured in debug; release would be
  faster; no round-up claimed).

## Standing

- octopus merge surface: **ALIVE** (commands + exits above, this
  session, exact subject).
- Whole-crate `cargo test -p cgap-git`: BLOCKED by concurrent lanes'
  in-flight compile breaks (files named above) — not this lane's diff.
- Zero-alloc invariant: instrumented (global counting allocator
  delta == 0 across the merge) + static (no heap in module; caller
  buffers; `forbid(unsafe_code)`).

## Divergences / disclosures

- No `#[path]` include of cgap-bn3 was made (dispatch allowed it): the
  radix discipline is re-derived locally (seedless payload mix, same
  shape) because `cgap-git` must stay zero-external-dep and Cargo.toml
  edits are forbidden. Disclosed divergence: bn3 buckets a single term
  word; here the key is `(s ^ p.rotl(17))` before the same mix.
- Topology guard refused a full-repo /tmp rsync; the sandbox court used
  a scratch-standalone crate copy (SP4 precedent, /tmp/sp4-sandbox)
  with a minimal standalone manifest; sandbox-only lib.rs pruned to the
  court subject (sandbox is a mutation-subject copy, not a checkout).
- Andon exit-101 witness: re-exec of the test binary with a sentinel
  env; child takes the Andon path and exits 101 — asserted by parent.
- lib.rs wire-in (`pub mod octopus_merge;`) intentionally NOT done —
  coordinator landing step.
