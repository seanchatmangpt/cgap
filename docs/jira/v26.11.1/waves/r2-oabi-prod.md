# r2-oabi-prod

Lane W2-10 (Wave R2/FINAL item 10, OABI-PROD-BYTES) — 2026-11-01, repo
`/Users/sac/cgap`, branch `main`, sole active writer on
`crates/cgap-ffi/src/` (GAP-1/R11 concluded; re-Read before Edit honored).

## Files written

- `crates/cgap-ffi/src/xip_tables.rs` (NEW, 734 lines): production
  `.oabi`/`.bn3` flash-rodata tables + three courts.
- `crates/cgap-ffi/src/lib.rs` (+9 lines): module registration
  (`pub mod xip_tables;` + doc). MINIMAL REGISTRATION ONLY — the dispatch
  lease named "minimal registration"; the generic contract's "lib.rs off
  limits" yields to the dispatch lease text (recorded divergence).
- `docs/jira/v26.11.1/waves/r2-oabi-prod.md` (this receipt).

No other file touched: `include/` untouched (read-only honored), `xip.ld`
untouched (read-only), all Cargo.toml files untouched, sovereign.rs
untouched.

## Byte provenance (cited, not redefined)

- `.oabi` = the committed A09 producer pack
  `ontologies/fibo/fibo_sec_dbt_v1.oabi`, 496 B = 64 B A06 §6 header +
  9 x 48 B records (slot_count=9, class_count=34, enum_count=843, flags
  0x3 LE|LEXHASH, reserved zero, exact length law 64+9x48=496 holds on the
  real bytes). Container law: `crates/cgap-bont/src/oabi.rs` (A06 §6
  `OABI_MAGIC`/`OABI_HEADER_LEN`/`OABI_RECORD_SIZE`; full loader law +
  producer-pack ingestion court `w9_fibo_pack_ingestion_court` stay in the
  owner crate — not re-implemented here, O(N^2) drift law).
- `.bn3` = an A11 arena image re-derived through the OWNER API:
  `cgap_bn3::arena::{FormulaNode, Arena}` — flatten of the cited
  `nested_program` shape (`arena.rs:987`, the arena court's own fixture):
  root quotation (2 statements, one FORMULA_REF) + child Q0 (1 statement +
  grandchild leaf) = 4 statements x 24 B + 3 headers x 16 B + 8 B framing
  prefix = 152 B. Wire law: A11-bn3-arena.md §4.4/§6/§7 via
  `crates/cgap-bn3/src/arena.rs` (`to_image` arena.rs:853, `from_image`
  arena.rs:896 restore boundary). Minted this session by a scratch
  derivation crate (/tmp/w210-derive, `cargo run` exit 0), embedded, and
  re-derived live by the court every run (byte-identity assert).
- No committed `.bn3` artifact exists in the repo (searched) — the network
  is therefore owner-API-derived, which is the honest derivation path.

## The production link (measured, this session)

Sandbox lease (/tmp, deleted at court end): byte-copy of the real
`src/*.rs`, staticlib+rlib manifest (dep cgap-arrow only — the none-target
keeps the pre-existing dep set), production entry `cgap_xip_start` with NO
demo fills, entry references the exported statics (`extern "C"` block) —
the archive-extraction roots. Command (nested, from the court):

```bash
RUSTFLAGS="-C link-arg=-T.../xip.ld -C link-arg=--gc-sections" \
  cargo build --release --target riscv32imc-unknown-none-elf
```

Result (court print, from llvm-size -A + llvm-objcopy --dump-section over
the real ELF): rust-lld -T xip.ld EXIT 0 — both xip.ld ASSERTs pass on
REAL bytes:

- flash = 670 B / 65536 B budget (1.0%)
- sram = 0 B / 32768 B budget (0%)
- `.oabi` 496 B at 0x42010000 (64 KiB-aligned, inside FLASH)
- `.bn3` 152 B at 0x42020000 (64 KiB-aligned, inside FLASH)
- section bytes byte-identical: .oabi == committed pack file; .bn3 ==
  live owner-API derivation == embedded const.

## Courts run (all this session, real commands + exits)

- `r2_oabi_table_identity_court` — PASS. Embedded table == committed pack
  (byte-identical). M-R2-0: header flip refused on its named leg
  (header_len 65 != 64).
- `r2_bn3_network_identity_court` — PASS. Embedded network == live
  owner-API derivation; `from_image` restores (4 statements, 3 headers)
  and re-serializes byte-exact. M-R2-0b: comparator flip fired.
- `r2_production_link_court` — PASS. Production link exit 0 on real bytes;
  budgets, placement, and 4-way byte identity asserted. M-R2-1 shim: one
  flipped byte -> real rebuild + relink -> section-identity comparator
  REFUSED (count==1 discipline, scoped to the mutated array span).
- `cargo test -p cgap-ffi` (full) — PASS, exit 0. 45 lib + 4 alloc_courts
  + 5 zero_copy + 0 doc = 54 passed, 0 failed; includes the pre-existing
  T1/ASSERT/surface-freeze courts over the changed crate.

Loud-skip classes remain wired (llvm-tools absent / rv32 target absent ->
`[SKIP]` with reason); neither fired this session (both gates green).

## Divergences (recorded, not silent)

1. Dispatch context vs on-disk xip.ld: the dispatch quoted ".oabi @0x42010000
   (96 B) / .bn3 @0x42020000 (48 B)" demands; the checked-in script demands
   only `SIZEOF > 0` (64 KiB-aligned, 4M FLASH). Disk wins; no size cap was
   applied and none was needed (496 B and 152 B both fit the 4M region).
   The landing addresses happen to be 0x42010000/0x42020000 as the dispatch
   narrative expected, with real sizes.
2. lib.rs registration: the generic lane contract bars lib.rs; the dispatch
   lease explicitly includes "minimal registration" in `crates/cgap-ffi/src/`
   — the mod line is that minimal registration, disclosed here.
3. The pre-existing T1 court (`sovereign.rs::xip_courts`) still carries its
   DEMO fills (0xA7x96 / 0xB3x48) in its sandbox entry; sovereign.rs is not
   this lane's write surface, so they remain. Effect: that court's merged
   sections are demo+production (still budget-green, 54-test suite green).
   The W2-10 court isolates production-only bytes by using an entry with no
   demo fills. A future A19-owner pass may drop the demo fills.
4. A11 network choice: no committed `.bn3` network artifact exists, so "an
   A11 network" is the owner-API-derived arena image of the cited court
   fixture program — a real network, not a committed-output extraction.
   Section bytes are byte-identical to that derivation (asserted every run).
5. The 670 B flash figure counts the ELF's FLASH-resident sections
   (text+rodata+oabi+bn3) per the A19 §4 split; the staticlib `.a` context
   row (unlinked archive, compiler_builtins) is a different subject and is
   not re-measured here (T1 court prints it as labeled context).

## See Also

- `docs/jira/v26.11.1/lanes/A06-bont-scalars.md` (§6 container law)
- `docs/jira/v26.11.1/lanes/A11-bn3-arena.md` (§4.4/§6/§7 wire law)
- `docs/jira/v26.11.1/lanes/A19-libcgap-ffi-xip.md` (§4 budgets, §5 placement)
- `crates/cgap-ffi/xip.ld` (ASSERTs + KEEP placement)
- `crates/cgap-ffi/src/xip_tables.rs` (the module + courts)
