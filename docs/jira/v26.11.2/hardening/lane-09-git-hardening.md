# Lane H9 — git hardening verification receipt

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Read-only lane; sole write
= this file. Build root lease: /tmp/h9-target.

## Verify table

| Item | Status | Evidence |
|---|---|---|
| pack_write.rs type-6/7 typed refusal | VERIFIED | pack_write.rs:394-395 — `if obj_type == 6 \|\| obj_type == 7 { return Err(GitError::UnsupportedDeltaEncoding); }` inside `read_varint_header` (386); types 0/5 keep BadVarint (391-392) |
| ladder.rs five cfg sites narrowed | VERIFIED | ladder.rs:28, 39, 43, 53, 142 — all `all(debug_assertions, feature = "std")` (43 is the `not()` complement arm); remaining `#[cfg(debug_assertions)]` at 262 is inside the `#[cfg(test)]` module (153), compile-inert for riscv |
| delta_refusal_court present | VERIFIED | crates/cgap-git/tests/delta_refusal_court.rs — 4 tests, real tempdir bytes, crate's own encode_object_header/zlib_compress/sha1 |
| Full suite | VERIFIED | `cargo test -p cgap-git` (CARGO_TARGET_DIR=/tmp/h9-target): exit 0, 12 binaries all ok |
| delta_refusal_court 4/4 | VERIFIED | `cargo test -p cgap-git --test delta_refusal_court`: 4 passed; 0 failed; exit 0 |
| riscv32imc DEBUG check | VERIFIED | `cargo +nightly check -p cgap-git --no-default-features --target riscv32imc-unknown-none-elf`: exit 0 ("Finished dev profile") |
| riscv32imc RELEASE check | VERIFIED | same + `--release`: exit 0 ("Finished release profile") |

## Anti-vacuity (pre-fix flip, w6 receipt)

`docs/jira/v26.11.2/waves/w6-git-typed-refusals.md:48-58`: pre-fix sandbox
run of the same court, HEAD pack_write.rs parsed the type-6/7 fixtures
`Ok` — `[ParsedObject { obj_type: 6, size: 21, ... }]` at
delta_refusal_court.rs:57, "PREFIX-COURT-EXIT:101". The court flips; the
green run is non-vacuous.

## H10 lever: encode_object_header still scalar

Dispatch named loose.rs / tree.rs / oid.rs — all ABSENT
(`ls` ENOENT for all three in crates/cgap-git/src/). Agent 10's real
surface is `encode_object_header`
(crates/cgap-git/src/pack_write.rs:206-218, BR10's lever). Verified
still scalar this session: byte-at-a-time varint loop (while-loop at
211-215, `out.push(c | 0x80)` per byte, no SIMD/branchless widen).
H10-implementation lane has not touched it.

## Profile matrix

riscv32imc-unknown-none-elf, `--no-default-features`: DEBUG exit 0
(was the pre-fix E0433 failure), RELEASE exit 0. Default-feature riscv
check remains structurally broken at lib.rs:13 (`extern crate std` vs
no std target) — pre-existing, per w6 divergence 1; not re-run here.

## Commands (exits this session)

- `cargo test -p cgap-git` → exit 0 (full suite, /tmp/h9-target)
- `cargo test -p cgap-git --test delta_refusal_court` → exit 0 (4/4)
- `cargo +nightly check -p cgap-git --no-default-features --target
  riscv32imc-unknown-none-elf` → exit 0 (dev profile)
- same `--release` → exit 0

## Standing: ALIVE

## Divergences

1. Dispatch's loose.rs/tree.rs/oid.rs targets absent; real surface is
   pack_write.rs:206-218 encode_object_header (recorded above).
2. Per w6: default-feature riscv32imc is pre-existing-broken (lib.rs:13);
   only the no-default-features profile pair is meaningful and both pass.
3. Working tree carries pre-existing M-state cgap-git files from other
   lanes; untouched by this lane (read-only).
