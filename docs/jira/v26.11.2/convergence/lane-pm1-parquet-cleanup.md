# Lane PM1 — parquet post-hardening cleanup (W9's list)

Receipt for lane PM1, 2026-11-02, repo /Users/sac/cgap, branch main.
Ownership honored: only `crates/cgap-parquet/src/{footer.rs,mmap_writer.rs}`
(doc-only edits + one const-assert test) and this receipt. Sandbox work in
`/tmp/pm1-sb`, target lease `/tmp/pm1-target` (deleted at lane end).

## Item 1 — `with_dyn` doc refresh (post-AE1 seal)

`crates/cgap-parquet/src/mmap_writer.rs` — the `with_dyn` doc said
"inject any `SpillSync` impl"; post-AE1 the trait is sealed on a
crate-private marker (`private::Sealed`), so the doc now states the
sealed reality: only `std::fs::File` and the in-crate `#[cfg(test)]`
recorder can implement it; no out-of-crate impl can exist (E0277
boundary).

## Item 2 — FA9 structural-arm anchor widened (choice: widen, justified)

Chose WIDEN over "document the window bound as deliberate". Justification:
a fixed 700-byte window breaks silently if the impl body grows or is
reordered — the assertion would then scan the wrong span and a no-sync
mutation would not flip (a vacuous court). The arm now anchors on
`impl SpillSync for std::fs::File` (impl start) to the impl's closing
brace at column 0 (first `\n}` after the header), so the scanned span
always tracks the production impl body however it grows or moves. The
`impl_end > impl_start` non-degenerate assert guards the anchor itself.

## Item 3 — compile-time META_CAP pin (new, footer.rs)

Added `const fn worst_case_meta_len(n)` (structural worst case of the
§6.2 meta block, field-by-field as `build_meta_block` emits it: delta-form
field headers, zigzag varints, long-form list header for n > 14, 8-byte
field-5 digest) plus

```rust
const _: () = assert!(
    worst_case_meta_len(MAX_ROW_GROUPS) < META_CAP, ...
);
```

so any future meta field addition or span-width growth past the fixed
scratch fails AT COMPILE TIME. Worst case at rg=64 computes ~353 B
(field 4's 64x5B spans dominate) < 512 META_CAP. Anti-vacuity:
`meta_cap_mutation_probe::cap_pin_is_not_vacuous` asserts the SAME
computation at `MAX_ROW_GROUPS * 2` EXCEEDS META_CAP — the const pin is
not trivially satisfiable. (A true compile-abort mutation of META_CAP
cannot run inside the shared compile; the runtime dual assertion is the
loud flip witness.)

## Item 4 — finding #6: CONTRACT DECISION for the operator (not codable)

Question for the operator: should `RingSink` (or the writer fronting it)
expose scan-level stats (rotation count, bytes-spilled, io_failed events)
at drain/commit boundaries, and if so under which typed surface — a
return value on `commit`, a stats snapshot method, or an OCEL-style event
emission? Codifying stats emission now would pick the operator's contract
without authority; the lane refuses to implement and records the decision
as open. Until decided, `wraps()` / `io_failed()` accessors remain the
only observation surface.

## Verification (commands + exits, this session)

- `CARGO_TARGET_DIR=/tmp/pm1-target cargo test -p cgap-parquet --features std`
  -> EXIT 0: 195 passed / 1 ignored (lib, includes new
  `cap_pin_is_not_vacuous`), plus all landed courts green
  (stats_skip 4, std_feature 2, integration suites 17/3/2/4x3, 1).
- FA9 mutation court (sandbox /tmp/pm1-sb/ws, zero-dep standalone
  workspace):
  - BASELINE (post-edit, sha256 b61b1e9243298e10...): fa9 test PASS.
  - MUTATION (`self.sync_all()` tail -> `Ok(())` in the File impl,
    sandbox copy only): fa9 test FAILED (flip witnessed, MUTEXIT=1) —
    under the WIDENED anchor, not the old 700-byte window.
  - RESTORE: sha256-exact b61b1e9243298e10... confirmed; baseline PASS
    re-witnessed after restore.
- Shared tree never mutated by the court; shared mmap_writer.rs sha256
  b61b1e9243298e1038efbaf4f7a6d0b031d990d8d40412952d6ebd58e18d7d98
  unchanged throughout.

## Sandbox mutation note (disclosed)

A first mutation attempt hit the wrong string occurrence (the assert
literal lives in the same file) and produced a vacuous pass inside the
SANDBOX only; root-caused (replace(.,1) matched the court's own literal),
sandbox rebuilt from the shared file, mutation re-targeted to the impl
tail only. The shared tree was untouched the entire time; the vacuous
pass never left /tmp.

## Standing

ALIVE for items 1-3 (commands + exits above, this session). Item 4:
OPEN decision, UNVERIFIED by design (not codable in this lane).
