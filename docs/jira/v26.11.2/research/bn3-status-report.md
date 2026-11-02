# bn3-status-report

80/20 survey of `cgap-bn3` (binary N3 execution crate) as a structural
input to the Arrow-seam and no_std plans. Two read-only Explore lanes +
first-hand coordinator verification; every claim cites file:line. Repo
/Users/sac/cgap, 2026-11-02. Read-only lanes; this file assembled by the
coordinator.

## 1. Physical representation (lane 1)

- Statement = flat 24-byte `#[repr(C)]` `{s,p,o: u64}` — sigil.rs:305-313;
  size courts sigil.rs:428-446, test `size_of == 24` :459-463. ADR-003
  forbids Box/Vec payloads (:307).
- Sigil tagging: term word = `sigil << 56 | payload56`; table sigil.rs:91-117
  (0x00 SlotConst, 0x01 VarUni, 0x02 VarExi, 0x03 FormulaRef, 0x04
  BuiltinOp, 0x05..=0xFF RESERVED — decoder refuses).
- Terms are NOT BLAKE3-derived in the payload: `SlotConst` = A06 SlotId
  truncated to u56 (`to_u56 = a06_slot >> 8`, sigil.rs:294-296) — drops the
  SlotId's LOW byte; PROVISIONAL collision-vetting note :300. SlotId itself
  is minted in cgap-bont (primitives.rs:61 newtype; truncate64 = first 8 LE
  bytes of BLAKE3, primitives.rs:136-144), not cgap-core (grep 0 hits).
- Quoted triples/formulas: input-side recursive `FormulaNode` AST flattens
  to pointer-free slab intervals + 16-byte `BN3FormulaHeader` repr(C)
  (arena.rs:272-345); nesting depth cap sigil.rs:172.
- Wire form == memory form: 24-byte LE image fns sigil.rs:396-427. Arena
  snapshot = counts + statements + headers (arena.rs:857-863). `from_image`
  copies (`copy_from_slice`) — mmap zero-copy ABSENT; "FFI reinterpret"
  rests on repr(C) alone (:299).
- Arrow ABSENT: execution is `StatementColumns<'a> { s,p,o: &'a [u64] }`
  (unify.rs:356-362); every Arrow mention is a deferred "Wave-3
  integration seam" (unify.rs:21-27, project.rs:22-26).

## 2. Execution engine (lane 1)

- Live default: radix-bucketed semi-join probe — `JoinProbeScratch{radix:
  bool}` unify.rs:661-664 (radix:true default :653-655); RADIX_BUCKETS=4096
  = 16 KiB directory (:714-718); bucket fn `(y ^ y>>16) & 0x0FFF` over
  payload window (:728-731); chain walk :1246-1310 (the hot path's one
  pointer-chase).
- Retired nested-loop arm kept as M-L3-1 differential (:1301-1322).
  Candidate cache chunk-independent (FNV key, :738-757; gate :1121);
  per-chunk rebuild survives as opt-in arm (:1152-1166).
- Rules → static joins: `RuleCompiler::compile` :474-527; body atoms 1..
  = semi-join expansion :1078-1080; fixpoint driver
  `unify_to_fixpoint_opt` :1387-1420 — NAIVE (full-store re-eval per
  round), not semi-naive delta. Heap-copy sites: `cols.*.to_vec()`
  :1371-1373; re-snapshot :1394; delta pushes :1400-1402; probe scratch
  fresh per rule per round via unify_batch_into :956 (leased
  unify_batch_probe :965 exists, unwired in driver).
- No semi-naive in bn3: the semi-naive delta loop lives in cgap-storage
  closure.rs:94-136; bn3 terminates on `new_rows == 0` (:1408) with
  max_rounds budget (:1411). P28/blake3 per-delta fold: ABSENT — no
  `p28_counter` symbol anywhere; blake3 only at attestation/commit
  boundaries (attestation.rs:271/319/507) and crypto builtins.
- Eytzinger: zero repo hits (confirmed dead design-spec).

## 3. no_std posture (lane 2) — the 438-error census decomposed

- `#![no_std]` NOT at root (lib.rs 7 lines, mods only). `extern crate
  alloc` arena.rs:219; `use alloc::vec::Vec` :222.
- Production std sites: unify.rs:98 `std::collections::{HashMap,HashSet}`,
  :99 `std::hash::{BuildHasherDefault,Hasher}`, :1315-1316 `std::mem::swap`.
  Everything else std:: is test-only (unify.rs:1424, 2370-2372;
  arena.rs:1614; sigil.rs:739-746; builtins.rs:1132-1137,1475).
- builtins.rs module body verified std-free by its own hygiene court
  (builtins.rs:1281-1285; must-kill pattern documented :97).
- m2-nostd-census receipt (m2-nostd-census.md:13): 438 errors on
  riscv32imc — dominated by cfg(test) bodies (119 inline tests:
  assert!/format!/std-collections) + the two unify.rs import lines.
  Verdict: no_std flip is a ~3-line production change + test gating,
  NOT a dialect purge. FixpointHasher (D5) already removes the
  default-hasher dependency for the seen-set.
- Host build clean; tests 130/130 pass (lib 119, bgp_hop 4, lever 7,
  0 doc-tests). LOC: 9,518 total (unify 2724, arena 2172, project 1436,
  builtins 1431, noalloc 914, sigil 834, lib 7).
- Release-profile object size: UNMEASURED (lane lacked llvm-size PATH;
  dev/debug numbers are not footprint). blake3 comparison class: linked
  z/fat-LTO/cgu1 ELF is the method of record (40,208 B witnessed).

## 4. Courts and coverage (lane 2)

- Differential vs naive oracle throughout: lever_courts.rs:88 (raw-word
  nested-loop oracle), bgp_hop_courts.rs:142 (naive k-hop).
- Anti-vacuity: probe-drop flip (:189), mask-emit must-kill (:378),
  vacuous-court refusals (bgp_hop_courts.rs:232/235), builtins arm-deletion
  flip (:97), noalloc capacity-refusal (:350), nostd-hygiene must-kill
  (builtins.rs:1277-1285).
- Seeds: SplitMix64 seed 0xBF58_476D_1CE4_E5B9 is the SEED LAW
  (lever_courts.rs:15-17,49); 0xA20C_0FFE_E5A7_E000 is the bitslice
  corpus-spread constant — never a seed. Conventions hold.
- Suite cost: ~11 min wall, ~10 min of it the bgp_hop 262_144-row debug
  legs — release-mode or reduced-N legs cut ~95% with no coverage loss.
- P28/penstock wiring: NONE (no dependency edge; only must-kill pattern
  lineage comments at penstock.rs:1841, asc450.rs:233). Real dependents:
  cgap-ffi (sovereign.rs:596-649, xip_tables.rs:359-379), cli, bench,
  cgap-court (ledger-only).

## 5. 80/20 verdict

- KEEP (the execution core): unify.rs semi-join + fixpoint, sigil.rs
  codec, arena.rs slab, project.rs projector, noalloc.rs bounded arm.
- KEEP-IF-BUILTINS: builtins.rs — std-free, self-guarded, orthogonal to
  joins (only cgap-blake3 dep).
- PRUNE UPSTREAM: nothing — no Turtle/N3 parser, string builder, or
  prefixed-name machinery exists in cgap-bn3 (grep 0 hits). The dialect
  weight sits upstream (stardf) by construction.
- The three real levers, in order: (1) no_std flip = ~3 production lines
  + cfg(test) gating; (2) zero-copy fixpoint = borrowed-prefix +
  owned-delta-tail driver (BFX lane in flight; heap copies at
  :1371-1373, re-snapshots :1394, leased-scratch hoist at :956→:965);
  (3) court-time reduction = bgp_hop leg profiling, not code deletion.

## See Also

- m2-nostd-census.md (census of record)
- w5-bn3-hasher-fix.md, w5-asm-census-repair.md (blr census state)
- w6-bn3-borrowed-prefix.md (zero-copy driver, in flight)
- 13-arrow-star-schema.md (L1 radix contract; 16 KiB correction owed at :44)

## Commands

| cmd | exit |
|---|---|
| wc -l over crates/cgap-bn3/src (lane 2) | 0 (9,518 total) |
| cargo check -p cgap-bn3 (host) | 0 |
| cargo test -p cgap-bn3 | 0 (130/130) |
| grep std::/turtle/n3/eytzinger inventories (lanes 1-2) | 0 |
