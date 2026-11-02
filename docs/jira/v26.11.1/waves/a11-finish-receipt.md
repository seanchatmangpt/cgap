# A11 Finish Receipt

- **Lane**: A11 (`crates/cgap-bn3` — `src/arena.rs`, `src/sigil.rs`), lane-finisher
  pass, v26.11.1, repo `/Users/sac/cgap` (one canonical checkout)
- **Date**: 2026-11-01
- **Dispatch**: assess the wave landed in 9be9109d; freeze the 24B triple-and-rule
  layout; land the fixed-capacity deductive arena with hard compile-time +
  init-time capacity bounds; witness the host gate; one sandbox flip battery;
  receipt; delete the lane target dir
- **Verdict**: **ALIVE** — canonical gate 111/111 exit 0 this session
  (5 new fixed-arena courts), sandbox battery 2/2 flips exit 0
  (`BATTERY VERDICT: FLIP-OK`), lease files sha256-unchanged by the battery,
  canonical tree never shimmed.

## Assessment (what was already landed; nothing rewritten)

`crates/cgap-bn3` on `main` @ cfd085a9 carries Waves 1-9 in full (module docs
are the receipts): Wave 1 froze the BN3 ABI (24-byte `repr(C)` `BN3Statement`,
16-byte `BN3FormulaHeader`, `Sigil` codes 0x00-0x04, RESERVED refusal), Wave 2
the codec + typed `BN3Error` surface, Wave 4 the `u32` index guards, Wave 7 the
transactional flatten (refused programs leave no residue), Wave 9 the A04 LE
identity images, the A05 power-cut sweep, the A14 hostile-image dam, and the
A20 1M-statement stress court. The finisher mission named one missing
capability: the **fixed-capacity deductive arena** (deterministic, zero heap at
capacity, hard compile-time + init-time capacity bounds). That is the delta
this receipt lands.

## The fixed-capacity deductive arena (mission item 2, landed)

- Compile-time hard bounds: `MAX_ARENA_STATEMENTS` / `MAX_ARENA_FORMULAS`
  = 2^32 (the frozen `u32` index space of spec 4.4), pinned by const asserts
  (`MAX_ARENA_STATEMENTS == 1u64 << 32`, `== u32::MAX as u64 + 1`) — a silent
  widening is a build failure.
- Init-time bound: `Arena::with_capacity(statements, formulas)` refuses above
  the bounds typed (`BN3Error::ArenaCapacityExceeded { requested, bound }`,
  third named extension beyond the spec Section 7 table; recorded divergence)
  **before any allocation** — the Err itself proves the guard precedes the
  96 GiB reserve.
- Zero heap at capacity: backing buffers are reserved exactly once at init;
  the push/reserve guards fire before the allocator is touched; an
  over-capacity flatten refuses typed with no admitted residue (the Wave-7
  rollback). `Vec::capacity` observables never change; a positive-control
  court proves the observable detects real growth.
- Deterministic: same admission sequence -> byte-identical images.
- `Arena::new()` remains the growable dynamic arena; one flatten path serves
  both (no duplicated calculus).
- Authority pin held: `size_of::<BN3Statement>() == 24` unchanged
  (`repr(C)`, const-asserted); the u128-widening shim aborts the build on
  exactly those asserts.

## Commands (this session, exact)

```text
CARGO_TARGET_DIR=/tmp/cgap-lane-A11-target cargo test -p cgap-bn3
  -> exit 0: 111 passed; 0 failed (102 pre-existing + 5 new fixed-arena
     courts; a seam-mate's concurrent +4 also in the count), 0 ignored
python3 /tmp/a11_fin_sandbox_battery.py -> exit 0 (BATTERY VERDICT: FLIP-OK)
  sandbox /tmp/a11-fin-sb: crate copies byte-identical to canonical before
  each shim (byte-equality asserted), one shim at a time, restore =
  canonical bytes written back + byte-exact verified after each; canonical
  tree never shimmed; sha256 UNCHANGED for arena.rs/sigil.rs/unify.rs/
  project.rs/builtins.rs/lib.rs pre/post battery
git status --short / git diff --stat (read-only)
```

## Courts run (name -> pass / flip evidence, this session)

- `size_court_statement_is_24_bytes` (+ const asserts) — green. Flip: M-F1
  (s: u64 -> u128) build-abort rc=101: E0080 size/align/offset asserts +
  E0308 x4, "could not compile due to 9 previous errors".
- `abi_court_arena_composition_laws` (24+24+16=64 cache line) — green.
  Flip: M-F1 fires the same const-assert family.
- `fixed_arena_court_init_capacity_bound_refuses_typed_above_compile_time_max`
  — green. Flip (structural): init-guard neuter would attempt the 96 GiB
  reserve (abort = flip); the executed flip rides M-F2.
- `fixed_arena_court_over_capacity_admission_refuses_rolls_back_no_growth`
  — green. Flip: M-F2 (guard neuter `if false &&`) rc=101: exactly this
  court FAILED 110/1.
- `fixed_arena_court_deterministic_image_and_sequential_scan` — green.
  Flip class: slab reorder (not executed; determinism rides the
  image-equality asserts).
- `fixed_arena_court_zero_capacity_refuses_first_admission` — green.
  Flip class: same guard-neuter class as M-F2.
- `fixed_arena_positive_control_capacity_observable_detects_growth` —
  green. Reads the allocator's real growth policy; no in-crate mutation
  class.
- Wave 1-9 courts (size, abi, round-trip, flatten, refusal, scope, depth,
  atomicity, hostile, wave9 identity/powercut/dam/stress) — 106/106 green.
  Prior-wave flip campaigns in the module docs (W5: 5/5, W7: 7/7,
  W9: 6/6, rc=101 each).

## Divergences

1. **Third named extension**: `BN3Error::ArenaCapacityExceeded { requested,
   bound }` joins `StatementIndexOverflow` and `ImageMalformed` as a named
   extension beyond the spec Section 7 seven-row table. Recorded in-code;
   spec is authoritative and yields on this point (the fixed-capacity
   requirement is dispatch text, not spec text).
2. **"cache-aligned sequential scan" (mission wording)**: implemented as the
   ABI cache-line tiling (2 statements + 1 header = 64 B, existing abi court)
   plus a contiguous sequential-scan court (slice order == index order,
   24 B/record, aligned pointer). A 24-byte record cannot be individually
   64-byte-aligned; the mission phrase is satisfied by the tiling law.
3. **Concurrent seam-mate activity**: unify.rs (+209) and project.rs (+56)
   carried A12/A13 in-flight edits during this pass; the crate compiled green
   throughout (no compile-freeze event, no unblock needed). Their +4 tests
   are inside the 111.
