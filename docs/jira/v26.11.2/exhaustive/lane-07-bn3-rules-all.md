# Lane EX7 — cgap-bn3 auxiliary census (READ-ONLY)

Date: 2026-11-02. Method: grep/read-only census, zero builds, zero mutations.
A06 `SlotId`, A04 128B frame, A11 BN3 layout pinned per CLAUDE.md; cited, not redefined.

## Missing files (LOUD)

`dedup.rs`, `closure.rs`, `rules.rs`, `compiler.rs` are ABSENT from
`crates/cgap-bn3/src/` (witnessed via `ls`, exit 0). DRed/closure live in
cgap-storage (cross-ref lane REP4, in flight). `unify.rs` exists but is NOT
in this lane's file list — covered by another lane.

## Per-file census (fn count includes tests; branch terms if/match/for/while)

| file | lines | pub fns | fns | branch sites | status |
|---|---|---|---|---| compile-free census |
| arena.rs | 2284 | 20 | 67 | 77 | 100% fn census |
| builtins.rs | 1495 | 8 | 37 | 62 | 100% fn census |
| project.rs | 1472 | 13 | 45 | 79 | 100% fn census |
| sigil.rs | 857 | 12 | 24 | 36 | 100% fn census |
| noalloc.rs | 975 | 8 | 30 | 90 | 100% fn census |

### arena.rs (2284L) — formula flattening + arena image ABI
- Consts: BN3_FORMULA_HEADER_SIZE=16, MAX_FORMULA_NESTING_DEPTH=64, MAX_ARENA_STATEMENTS/FORMULAS=2^32, ARENA_IMAGE_PREFIX_LEN=8.
- Types: `BN3FormulaHeader` (16B, offset asserts :342-355), `Slab`, `FormulaTable`, `FormulaNode` (input-side AST), `Arena`.
- Fns: header_index, stmt_index, Slab{new,fixed,fixed_capacity,push,get,len,is_empty,statements,truncate_to}, FormulaTable{new,fixed,fixed_capacity,reserve,close,get,len,is_empty,headers,truncate_to}, FormulaNode{leaf,quoted}, assign_headers/resolve_ref/emit, verify_intervals_disjoint_and_nested, check_formula_ref, innermost_scope, Arena{new,with_capacity,flatten,innermost_scope,check_formula_ref,to_image,from_image}.
- HOT: Slab::push/get, FormulaTable::reserve/close (hot admission path). COLD: `FormulaNode`/flatten/assign_headers/emit — input-side AST, one-shot load. to_image/from_image cold (persist boundary).
- Courts: size/ABI/const courts, flatten-overlap mutation, depth 64/65 boundary, atomicity, 1M-statement stress, fixed-capacity rollback (cap observable detects growth — anti-vacuity), wave9 powercut/hostile-image sweep.

### builtins.rs (1495L) — closed 9-opcode builtin ISA
- Consts: OP_LOG_TRACE/ASSERT/COUNTERS (0x01-0x03), OP_MATH_{GREATER_THAN,SUM,MUL10,MIN} (0x10-0x13), OP_CRYPTO_{BLAKE3,BLAKE3_WIDE} (0x20-0x21); OPCODE_COUNT=9, FREE_OPCODE_COUNT=247; 4B `BuiltinPayload`, 256 regs.
- Fns: BuiltinPayload{new,encode,decode}, digest_word56, dispatch (compile-time match, closed), crypto_range, land_digest; Display/Error impls.
- HOT: dispatch + payload encode/decode (hot eval path). COLD: Display/Error, digest KATs.
- Courts: registry closure sweep (247 free opcodes refuse), unknown-opcode typed refusal, nostd-hygiene (module-body std::/panic sweeps :1325-1405), hostile decode/domain sweep, u64-max eval-counter wrap, write-isolation.

### project.rs (1472L) — consequent projection + WAL/frame
- Consts: DELTA_DIGEST_LEN=32, DELTA_COLUMNS=4, BN3_FRAME_FLAG=0x04, BN3_FRAME_HEADER_LEN=128 (A04), WAL 48+48+16=112B record, magics/version pins.
- Types: `DeltaBatch<B>`, `DeltaRows`, `ConsequentProjector`, `Bn3WalDescriptor`, `Bn3WalCommit`, tear/refusal/verdict enums.
- Fns: sha256/sha256_padded (+64-entry K table), DeltaRows{push,clear,len,is_empty,row,canonical_bytes,write_canonical_bytes}, ConsequentProjector{new,project,emit_into}, Bn3WalDescriptor{of_batch,to_bytes,from_bytes}, read_u32/u64_le_at, crc32_ieee, bn3_wal_classify/encode, bn3_frame_header_encode, frame_oabi_digest.
- HOT: ConsequentProjector::project/emit_into (hot consequence path), DeltaRows::push, crc32_ieee. COLD: sha256_padded K-table, WAL classify/encode (persist boundary), frame encode.
- Courts: sha256 KATs (empty/abc/two-block), projector popcount M12.7 invariant, byte-identical repeat/dirty-buffer determinism, WAL CRC check value, powercut sweep (every offset typed), corruption typed refusal, 128B frame header pin.

### sigil.rs (857L) — term codec + BN3Statement 24B ABI
- Consts: SIGIL_SHIFT=56, PAYLOAD_BITS=56, SIGIL_LAST_KNOWN=0x04, SIGIL_RESERVED_FIRST=0x05, REGISTER_COUNT=256, BN3_STATEMENT_SIZE=24.
- Types: `Sigil{SlotConst,VarUni,VarExi,FormulaRef,BuiltinOp}` (0x00-0x04), `BN3Error` (10+ variants incl. UnknownSigil, FormulaInPredicate), `BN3Statement` (24B, offset asserts :434-452).
- Fns: Sigil::from_code/code, encode_term/decode_term, to_u56, BN3Statement{from_words,encode_terms,decode_terms,validate,to_le_bytes,from_le_bytes}, Display/Error.
- HOT: encode_term/decode_term, statement encode/decode (hot codec path). COLD: Display, to_u56 (A06 boundary, cold import).
- Courts: size court 24B, sigil codes spec, 8/56 field split, all-five-sigils-at-bounds round-trip, reserved 0x05..=0xFF typed refusal, hostile full-domain codec totality, from_le_bytes total/unvalidating (validate is the gate).

### noalloc.rs (975L) — zero-alloc match engine
- Consts: NA_MAX_CONSTS=64, NA_MAX_VARS=128, NA_MAX_ATOMS=8, NA_MAX_MATCHES=2048, NA_MAX_ROWS=8192; cross-crate pin to unify consts (:41-42).
- Types: `NaError` (10 variants incl. MatchCapacity), `NaRule`, `NaMatches`, `NaEngine`.
- Fns: NaMatches{zeroed,atom_row,var_value,contains}, boxed_zeroed (pub(crate)), NaEngine::zeroed, na_compile, na_intern, lane_eq_mask, col, atom_local_match, push_match, na_unify, na_unify_radix (radix bucketing, NA_MAX_ROWS bound), na_unify_inner.
- HOT: na_unify/na_unify_inner/atom_local_match/push_match (hottest path in crate). COLD: na_compile, na_intern (one-shot compile).
- Courts: differential noalloc-vs-alloc arm over 1200 rows, alias-trap register-number≠value, zero-alloc counter positive control (anti-vacuity), capacity refusals typed (consts/vars/atoms/match overflow NA_MAX_MATCHES=2048, never truncate), corpus same-seed identical (seed 0xBF58_476D_1CE4_E5B9).

## Heap-allocation inventory (post-BN3NS no_std, extern crate alloc)

- arena.rs: Vec in Slab/FormulaTable/FormulaNode/Arena images — all setup-side (compile/flatten/image), never on the eval hot path.
- builtins.rs: `BuiltinContext.trace: Vec<u64>` (the ONLY hot-path-adjacent alloc; log-trace mode only; counters/assert/crypto allocate none).
- builtins.rs courts read module body to sweep std::/panic (hygiene courts, test-only).
- project.rs: DeltaRows Vecs + canonical bytes Vec (alloc-bearing by design; projector consumes pre-alloc matches).
- sigil.rs: ZERO alloc — pure core, no extern crate alloc.
- noalloc.rs: Vecs test-only (courts); `boxed_zeroed` allocates exactly once at engine init; na_unify allocates ZERO after setup (witnessed by zero_alloc_court positive control).

## Coverage ledger (100%)

All five listed files censused: fn/type/const inventory complete, hot/cold classified, alloc inventoried. No builds run (read-only lane). Nothing
is silently skipped; missing files marked LOUD above; unify.rs out of scope.
Standing: census standing only (compile-free, read-only); no ALIVE claims.
