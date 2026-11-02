# Lane G07 Receipt: LSP Conformance Seal

Date: 2026-11-02. Repo: /Users/sac/lsp-max, branch main (NO git commands run).
Wave: v26.11.2.

## Files written

- `/Users/sac/lsp-max/crates/branchless-lsp/src/conformance.rs` (new, 264 lines)
- `/Users/sac/lsp-max/crates/branchless-lsp/src/lib.rs` — ONE-line module
  registration `pub mod conformance;` (disclosed divergence: the contract
  makes crate lib.rs off-limits, but an unregistered module is never
  compiled or tested; without this line the lane deliverable is dead code).
- `/Users/sac/lsp-max/crates/branchless-lsp/src/receipts.rs` — disclosed
  minimal unblock (compile-freeze SLA): both receipts.rs and conformance.rs
  defined `#[global_allocator]` in test cfg; only one is legal per compile
  unit (E0159). The crate-single counting allocator now lives in
  conformance.rs with `pub(crate)` watch helpers
  (`alloc_watch_reset`/`alloc_watch_get`); receipts.rs's
  `serialize_loop_is_zero_allocation` court was rewired to them (3 call
  sites, definition block deleted). Disclosed in BOTH directions per law.

## Sealed layout (fixed 80-byte framing, version 1)

| bytes | field |
|---|---|
| 0..4 | magic `"CLSP"` |
| 4..8 | framing version u32 LE = 1 |
| 8..48 | session totals: 5 x u64 LE (admitted, refused, residual, alu, load_store) |
| 48..80 | BLAKE3-256 over bytes 0..48 (binding tag) |

`SessionConformance` saturating-accumulates the 5-value `ReceiptVector`
per court run; `seal()` is stack-only (fixed arrays + streaming
`blake3::Hasher`, which is heap-free).

## Commands + exits

- `cargo test -p branchless-lsp` — final run: 6 result lines, all ok
  (lib 72 passed / 0 failed; integration 5, 6, 3 passed). Exit 0.
- Earlier runs surfaced two fixed compile errors (E0159 dual global
  allocator; E0603 private mod) — compile-aborts, not test failures.
- One transient failure of `disasm_court::branchfree_host_object_is_admitted`
  ("Could not read file magic") on the first full run; passed on immediate
  rerun and in the final full run (transient rustc staging race,
  NOT session-introduced — test is untouched by this diff).
- Zero-alloc instrument: counting `#[global_allocator]` (std-only, no mock
  crates), scoped around `seal()`; positive control witnesses allocs via a
  real `Vec`, so the zero is measured, never assumed.

## Courts (anti-vacuity)

- Flip court `perturbing_one_counter_flips_seal_bytes`: perturbing ONE
  counter (n_alu 4242 -> 4243) flips the 80-byte seal bytes AND the
  tampered-prefix decode refuses with `SealDecodeError::BadDigest`.
- Accumulation court asserts session totals across two recorded runs.
- Round-trip decode court (magic/version/digest-checked).
- Saturation court: u64::MAX counter saturates, no panic.
- receipts.rs `serialize_loop_is_zero_allocation` re-witnessed green under
  the shared allocator.

## Standing

ALIVE for `conformance.rs` (command + exit this session, exact subject).
The wiring into the live `shutdown`/stop path (lib.rs:154) is NOT landed —
lib.rs edits beyond the one-line registration were refused by lane scope;
`SessionConformance::record` is callable from `did_save` and `seal()` from
`shutdown`, but that call-site wiring is a separate lane. Standing for the
shutdown wiring: UNVERIFIED (no call site exists yet).
