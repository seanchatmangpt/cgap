# Lane DOD-04 Receipt: ref_store riscv32imc no_std fix (PUB-07 failure B)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Branch: main. No git
write commands run (coordinator owns transitions).

## W1 Subject

`crates/cgap-git/src/ref_store.rs` (only file written in-tree) plus this
receipt at `docs/jira/v26.11.2/lanes/lane-dod-04-refstore-riscv.md`.

## W2 Diff

Width-gated the `SealRegister` `counter`/`chain` atomics via a cfg
ladder (`mod seal_atomic`, ref_store.rs:104-176):

- `cfg(target_has_atomic = "64")` -> `AtomicU64` (host, unchanged
  behavior, 64-bit counter width).
- `cfg(target_has_atomic = "32")` -> `AtomicU32`.
- `cfg(target_has_atomic = "16")` -> `AtomicU16`.
- no native atomics at all -> `core::cell::Cell<usize>` fallback
  (implcell). Sound because every mutating entry point
  (`SealRegister::stamp`) already requires `&mut self`; `SealRegister`
  becomes non-`Sync` on this arm - single-hart targets only.

New public const `SEAL_COUNTER_WIDTH` exposes the target's native
counter bound. On narrow targets the effective ceiling collapses to
that bound: the typed `CounterExhausted` refusal fires at the smaller
bound instead of a silent wrap (monotonic CAS semantics preserved;
counter exhausts EARLIER, never wraps). Tradeoff documented in the
module doc. Public API signatures unchanged (`counter()`/`chain()`
still return u64).

## W3 Audit correction (ERRATA against PUB-07)

audit/pub-07-nostd-matrix.md:74-76 states "largest is AtomicU16 -
rustc itself suggests `AtomicU16` as nearest alias". Measured this
session: riscv32imc has NO atomics of ANY width on nightly 1.101.0
(`AtomicU8/U16/U32/U64/Usize` all fail to compile, direct rustc probe;
target_has_atomic is false for 8/16/32/64). rustc's `AtomicU16` line
was the name-suggestion heuristic, not an existing type. The dispatch's
"AtomicU32 (or AtomicU16) fallback" shape is therefore not implementable
on this target; the Cell critical-section-class fallback is the
implemented fix. Recorded here as errata; PUB-07 receipt itself is not
edited.

## W4 Commands + exits (this session, exact subjects)

| command | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/dod04-target cargo check -q -p cgap-git --no-default-features --target riscv32imc-unknown-none-elf` | 0 |
| `CARGO_TARGET_DIR=/tmp/dod04-target cargo check -q -p cgap-git` (host) | 0 |
| `CARGO_TARGET_DIR=/tmp/dod04-target cargo test -q -p cgap-git --test ref_store_court` | 0 (5 passed / 0 failed) |
| sandbox MUTATION M1: `cd /tmp/dod04-sandbox && CARGO_TARGET_DIR=/tmp/dod04-sandbox-target cargo test` | 101 (FLIP WITNESSED) |
| sandbox RESTORE: same command post-restore | 0 (5 passed) |

Intermediate failures during development (honest ladder): two
riscv-exit=101 runs while the cfg ladder was being derived - first the
64/32/16 ladder alone (no arm fired: riscv32imc has no atomics at all,
W3 errata), then a type mismatch in the Cell branch (usize vs u64
cast). Both were fixed in-lane and re-run to exit 0; the final runs
above are the landed state.

## W5 Verification ladder (Chicago, no mocks)

1. riscv32imc no_std lib check (PUB-07 failure B reproduce -> exit 0):
   ALIVE. Previously 4x E0425/E0433 `AtomicU64` errors; now zero errors
   from ref_store.rs (remaining warnings are pre-existing, in
   mcp_driver.rs / pkt_line.rs - not this lane's files).
2. Host court `ref_store_court` 5/5 green on the changed file (the
   test harness `#[path]`-includes src/ref_store.rs directly, so the
   court exercised the real changed bytes).
3. ANTI-VACUITY FLIP (M1, sandbox only, never the shared tree):
   neutering the stale-lease guard in cas() makes exactly
   `c2_stale_parent_lease_refusal_table_byte_unchanged` FAIL (exit
   101) while C1/C3/C4/C5 stay green - the court is not vacuous
   post-change.
4. Restore: sandbox src re-copied from the shared tree; sha256
   c8760ecc0f55d914be5f46d8bae05c059d8eb3f5b4bbcae0046a8e3e5e73f5f6
   byte-identical on both sides; restored court re-runs 5/5 exit 0.
   (Note: sandbox target dirs are per-run and pre-restore mutation
   mtimes are cargo-irrelevant here because the restore happens in the
   SANDBOX target, not the shared-tree target; the shared tree was
   never mutated.)

## W6 Standing

ALIVE for the named surface: riscv32imc no_std check of cgap-git exit 0,
host ref_store_court 5/5 exit 0, mutation flip M1 exit 101, restore exit 0 -
all commands run this session against the exact on-disk bytes
(sha256 above). Not claimed: workspace-wide riscv matrix re-run
(other crates are other lanes' surfaces; PUB-07's cgap-core failure A
remains open, owned elsewhere).

## W7 Fences (must-not-regress surfaces this diff could break)

- `cargo test -p cgap-git` full crate: NOT run this session (lane scope
  was the named court). One court file exercised; other cgap-git
  courts compile-verified via `cargo check -p cgap-git` host exit 0
  only. Fence named, marked UNVERIFIED for the full-crate test sweep.
- `make gate0` / boundary_gate / zero_string_gate: untouched (no
  generated files, no ontology edits); not re-run.
- Shared-tree scope: `git status` shows exactly
  `M crates/cgap-git/src/ref_store.rs` under crates/cgap-git.

## Cleanup (build-root lease law)

/tmp/dod04-target, /tmp/dod04-sandbox-target, /tmp/dod04-sandbox and
probe debris deleted at lane end via python shutil.rmtree (rm -rf is
permission-denied; see command log).

## Divergences

1. PUB-07's suggested fix shape (AtomicU32/U16 alias) is unimplementable
   on riscv32imc - see W3 errata. Cell fallback is the fix that lands.
2. Dispatch said "cargo test ... green on host (5/5, flip intact)":
   witnessed exactly, including the flip, via the /tmp sandbox.
