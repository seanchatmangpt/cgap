# W6-B1 scaffold receipt — crates/cgap-branchless

Lane B1 scaffolded `crates/cgap-branchless` and landed the workspace member line
in the root manifest. Concurrent lanes B2/B3/B4/B5 landed their branchless forms
into the same module files during this session (see Divergences).

## Subject

- New crate `crates/cgap-branchless` v26.11.2, `#![no_std]`, zero deps,
  edition 2021, `publish = false`, `#![forbid(unsafe_code)]`.
- `src/lib.rs` registers `select`, `arithmetic`, `swar`, `eytzinger`, `radix`.
- One shared-file edit (disclosed): root `Cargo.toml` members list, two lines
  added after `"crates/cgap-court",`:

  ```toml
  # v26.11.2 W6 lane B1: centralized branchless ALU/SWAR mechanics.
  "crates/cgap-branchless",
  ```

## Files written / owned

- `crates/cgap-branchless/Cargo.toml` (new)
- `crates/cgap-branchless/src/lib.rs` (new)
- `crates/cgap-branchless/src/select.rs` (new, B1 naive forms)
- `crates/cgap-branchless/src/arithmetic.rs` (scaffold; branchless form landed
  concurrently by lane B2)
- `arithmetic.rs` unblock fixes (disclosed in both lanes):
  - borrow formula: B2's xor-sign borrow was wrong for unsigned underflow
    (0-1 yielded no borrow); exact 128-bit borrow-out with `(hi | hi.neg()) >> 63`
    fold (B1) — B2 later converged to `((full >> 64) & 1)` (equivalent).
  - `poison_mask`: B2's `((q-1) as i64 >> 63)` fired on every quota >= 2^63,
    not only quota == 0; B1 unblocked with the exact
    `((q | q.wrapping_neg()) >> 63) & 1` is-nonzero fold.
- `src/swar.rs` (scaffold; branchless form landed concurrently by lane B3).
  B1 unblock fix (disclosed): B3's add-based equality detector misses the
  all-equal case (x == 0); restored the classic
  `x.wrapping_sub(LO) & !x & HI` zero-byte detect.
- `src/eytzinger.rs` (scaffold; branchless form landed concurrently by lane B4).
  B1 unblock fixes (disclosed): (1) layout builder recursed over the real
  array, so non-power-of-two n misplaces elements; fix = lay out the virtual
  padded sequence and keep slots <= n; (2) B4's hand-derived trace asserted
  `eytzinger_next(5, false) == 11` (left child of 5 is 10); B4 fixed that on
  disk before the B1 patch applied (my patch assert showed the string gone and
  the suite went green in the same window).
- `src/radix.rs` (scaffold; branchless/shipped-parity form landed concurrently
  by lane B5, sha256-pinned to cgap-bn3 unify.rs kernel).

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/b1-target cargo test -p cgap-branchless`
  -> exit 0, `26 passed; 0 failed` (log /tmp/b1-gate.log).
- `CARGO_TARGET_DIR=/tmp/b1-target cargo build --workspace`
  -> exit 0, `Finished dev profile ... in 2.59s` (/tmp/b1-ws.log).
- Differential/pin courts inside the crate exercised this session: B5's
  100k-word differential vs shipped `cgap-bn3` radix kernel, B2's 4096-word
  differential vs naive oracles, B3's byte-edge-exhaustive SWAR courts, B4's
  exhaustive sizes-1..=64+1024 upper-bound differential vs partition_point.

## Standing

ALIVE for the B1 scaffold scope (command + exit above, this session). The
branchless implementations in arithmetic/swar/eytzinger/radix carry their
owner lanes' (B2-B5) standing, re-anchored by the same green gate run; the
flip courts named in this receipt are the anti-vacuity mutations already
present in those modules' test blocks.

## Divergences

1. Dispatch expected B1 to own the five module files; concurrent lanes
   B2/B3/B4/B5 landed their branchless forms into those same files during
   this session (files changed under me repeatedly; several of my fixes were
   overwritten mid-flight and re-applied). Net effect: the crate converged
   green with mixed ownership — disclosed above per file.
2. `select_slice` naive reference cannot return a borrowed mixed-mask
   selection without a caller buffer; the naive form handles uniform masks
   (returns the input slice) and panics loudly on mixed masks. Branchless
   buffer form is lane B2's.
3. mtime-bump / `cargo clean -p` needed after concurrent writes; witnessed
   stale-binary runs otherwise (documented CLAUDE.md law, re-witnessed here
   after multi-lane rewrites).
