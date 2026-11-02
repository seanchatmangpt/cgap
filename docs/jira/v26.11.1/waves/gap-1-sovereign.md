# gap-1-sovereign

Lane GAP-1 (lane-runner dispatch, 2026-11-01) — repo /Users/sac/cgap, branch main.
Coordinator-order lane: the three missing `#[no_mangle]` sovereign bodies, the
`_cgap_p28_invariant_check` header decision, and the T1 re-anchor to the
§4-conformant ELF subject.

## Deliverables

| Item | Standing | Evidence |
|---|---|---|
| cgap_admit_frame body | ALIVE | lib compiles (host exit 0); positive + corpus courts green this
  session |
| cgap_eval_bn3 body | ALIVE | lib compiles (host exit 0); positive + corpus + pin-parity courts
  green this session |
| cgap_commit_parquet body | ALIVE | lib compiles (host exit 0); positive + corpus courts green
  this session |
| _cgap_p28_invariant_check header decision | DECIDED (option b) | typed UNSUPPORTED(scope)
  recorded: sovereign.rs `P28_INVARIANT_CHECK_UNSUPPORTED`; include/cgap.h UNCHANGED |
| T1 size-gate repin | ALIVE (green on ELF subject) | in-court measured ELF flash=150 B / 65536 B,
  sram=0 B / 32768 B |
| cargo test -p cgap-ffi | green | 42 passed; 0 failed (lib), exit 0; full suite exit 0 (final run
  below) |
| nm -gU four sovereign symbols | ALIVE | all four T symbols witnessed (below) |
| riscv32imc check | exit 0 | /tmp/gap1-riscv.log |

## Commands + exits (this session, CARGO_TARGET_DIR=/tmp/gap-1-target)

- `cargo check -p cgap-ffi` -> 0 (post-body landing; /tmp/gap1-test-run.log era)
- `cargo check -p cgap-ffi --tests` -> 0 (courts compile)
- `cargo check -p cgap-ffi --target riscv32imc-unknown-none-elf` -> 0 (twice:
  post-landing and final; /tmp/gap1-riscv.log)
- `cargo test -p cgap-ffi --no-fail-fast` run 1 -> 101 (36 passed / 6 failed —
  the six courts caught six real defects; see Courts section)
- `cargo test -p cgap-ffi --no-fail-fast` run 2 -> 101 (41/1 — zero-copy site
  offset defect caught)
- `cargo test -p cgap-ffi --lib --no-fail-fast` run 3 -> 0 (42 passed / 0 failed)
- `cargo test -p cgap-ffi --lib wave_gap1 -- --nocapture` -> 0 (witness lines
  extracted below; /tmp/gap1-witness.log)
- `cargo build -p cgap-ffi --lib` + `nm -gU /tmp/gap-1-target/debug/
  libcgap_ffi.dylib` -> 0; exports witnessed:
  `_cgap_admit_frame` T, `_cgap_eval_bn3` T, `_cgap_commit_parquet` T,
  `_cgap_verify_p28` T (plus `_cgap_p28_invariant_check` T riding the link,
  `_cgap_i32_batch` T, `_cgap_version` T, ...)
- final full `cargo test -p cgap-ffi --no-fail-fast` -> 0 (receipt tail below)

## The three bodies (owner-table wiring)

All three are `#[cfg(not(target_os = "none"))]` `#[no_mangle] pub unsafe extern
"C"` in crates/cgap-ffi/src/sovereign.rs, delegating to the real owner paths:

- admit: A14 `cgap_gate::penstock::ingress` (limiter, A19 register values,
  fixed §4 order) then A04 `CgapFrameHeader::{project,decode,validate,payload,
  verify_payload}` with `cgap_blake3::hash` as the digest substrate (the
  cgap-core tests/common canon). Fault->register mapping helper
  `map_frame_error` (BadMagic/BadVersion/Misaligned/BufferTooSmall ->
  INVALID_MAGIC; PayloadLenMismatch/DigestMismatch -> OABI_MISMATCH;
  ReservedBitsSet -> SHACL_VIOLATION; AttestationMismatch -> P28_SPOLIATION).
- eval: A12 `RuleCompiler::compile` + `unify_batch` +
  `ConsequentProjector::project` over LE-decoded rule words; lane[i] =
  projected head row i object word, refused typed outside [0, i32::MAX];
  cap overflow -> LEASE (fail-closed, never truncation).
- commit: A02 `StaticPageBuffer::<65536>::writer().write_slab(SlabView bits=8)
  -> seal(None) -> flush_to(FileSink)`; domain gate [0,255] refuses what the
  inherited packer truncates.

## _cgap_p28_invariant_check — header decision (dispatch branch b)

include/cgap.h is UNCHANGED this wave. Typed record (also in sovereign.rs as
`P28_INVARIANT_CHECK_UNSUPPORTED`, assertable in courts):

> UNSUPPORTED(scope): _cgap_p28_invariant_check (cgap_p28_invariant_check,
> cgap-p28 trap.rs:82) rides the link undeclared BY DESIGN; declaring it in
> include/cgap.h is refused — A19-libcgap-ffi-xip.md:47 (boundary = exactly
> four functions), :57-60 (a fifth verb belongs in a lane, not the ABI);
> oabi_lookup.rs::OABI_LOOKUP_UNSUPPORTED (same decision form, F2-6);
> trap.rs:8-9 (A17 owns the constant + halt semantics; FFI contract A19's)

Link truth (nm -gU, this session): the symbol IS present in the cdylib
(`_cgap_p28_invariant_check` T) — it rides the link through the host cgap-p28
dep edge; the frozen header intentionally does not declare it.

## T1 repin (coordinator-ordered)

- Subject re-anchored: staticlib `.a` (NOT the §4 budget subject — unlinked
  archive, every compiler_builtins TU, nothing GC'd) -> the xip.ld-linked
  final ELF (rust-lld, `-C link-arg=--gc-sections`), measured with
  `llvm-size -A`; budgets UNCHANGED 64 KiB flash / 32 KiB SRAM (spec §4
  prose; the subject moved, never the budget).
- Asserted (green): ELF flash=150 B / 65536 B, sram=0 B / 32768 B.
- CONTEXT rows (not asserted, printed in-court): staticlib raw .a file size
  264,881 B (the coordinator-cited receipt number); llvm-size -A flash-split
  of the same .a this session = 444,213 B / sram 4 B (basis: summed input
  sections; grows with the lease bin/rlib manifest shape).
- §4 SRAM stack term: "worst-case eval stack (measured with a stack-paint
  test on target)" — hardware/qemu absent this session; recorded UNVERIFIED
  (labeled in-court comment), never silently dropped.
- The ELF subject carries the landed XIP surface: entry (cgap_xip_start) +
  the lib's none-target panic handler (KEEP'd .text.cgap_xip_panic) + demo
  .oabi/.bn3 rodata (the xip.ld ASSERTs require them). XIP rule EXECUTION
  stays BLOCKED-on-owner: the A12 unifier/eval path is an alloc API and the
  XIP budget carries no allocator (cgap-bn3 unify.rs: Vec/HashMap) — the
  sovereign eval/commit/verify bodies are host-surface this wave (same
  disclosed XIP divergence class as the existing cgap_verify_p28 block).

## Courts run (name -> pass + flip evidence)

Every court green in run 3/final; flip legs EXECUTED in-court every run.

### admit (wave_gap1_courts)

- `admit_positive_control_real_frame_zero_copy_view`: real BLAKE3-digested
  frame admits; view.payload points INTO the caller buffer (zero copy);
  NULL probe admits; aligned projection path + unaligned decode path agree.
  Flip: always-refuse stub refuses the valid frame (stub leg executed).
- `admit_sandbox_corruption_corpus_typed_refusals`: 7 fault sites in /tmp
  sandbox files (read-from-disk bytes), 7 TYPED refusals with exact register
  class per site: magic->INVALID, version->INVALID, reserved-flag->SHACL,
  payload_byte_len+1->OABI, digest byte->OABI, attestation magic->P28,
  payload byte->OABI; sha256/byte-exact fail-closed restore. Flip:
  always-admit stub ADMITS a mutation (executed).
- Flip witness line: "ADMIT CORPUS: 7 fault sites, 7 typed refusals, restore
  byte-exact".

### eval (wave_gap1_courts)

- `eval_positive_control_deterministic_zero_copy`: real one-rule network over
  a real 3-row store; 2 matches; lanes = projected head objects [1, 5]
  (?0 binds the matched row's subject); deterministic re-call; eval-side
  zero-copy witness (mutate payload byte -> count drops; restore -> [1, 5]).
  Flip: always-refuse stub (executed).
- `eval_sandbox_mutation_corpus_typed_refusals`: 6 legs, all from /tmp disk
  bytes: reserved sigil 0x05 -> OABI; body_len framing runaway -> OABI;
  aliased non-row-multiple payload -> SHACL; NULL view -> SHACL; head object
  3e9 outside the declared i32 lane domain -> SHACL; cap-1 lease -> LEASE
  (fail-closed, never truncation). sha256/byte-exact restore. Flip: the
  always-admit stub + the lane-domain pin.
- `eval_encoding_pin_parity_real_compiler_round_trip`: the PINNED eval
  encoding round-trips through cgap-bn3's OWN RuleCompiler to the
  owner-identical CompiledRule; executed framing mutation (rule_id <-> head_p
  word swap) flips (assert_ne passes on the mutated form).

### commit (wave_gap1_courts)

- `commit_positive_control_matches_owner_api_bytes`: the committed file is
  BYTE-IDENTICAL to a sealed block produced locally through the A02 owner
  API (write_slab -> seal); deterministic across two commits.
  Flip: always-refuse stub (executed).
- `commit_sandbox_refusal_corpus_typed`: lane 256 -> SHACL (the domain pin:
  the surface refuses what the packer truncates); negative lane -> SHACL;
  zero rows -> SHACL; NULL path -> SHACL; 70k-row INCOMPRESSIBLE lane batch
  beyond the 64 KiB documented lease -> LEASE (RLE would collapse a constant
  fixture — the fixture alternates values). Flip: the positive twin (lane
  255) admitted through the same gate.

### cross-cutting (wave_gap1_courts)

- `gap1_stub_probes_can_flip`: refuse-stubs flip all three positive controls;
  the admit always-admit stub flips the corpus.
- `gap1_cdylib_all_four_sovereign_symbols_present`: nested lease build, nm
  -gU on the fresh cdylib: ALL FOUR sovereign bodies exported, 6 frozen slot
  exports intact (additive-only law), rename controls absent.

### updated standing courts

- `status_court_reachability_ledger`: 5/5 fault values now fault-reachable
  (3 via cgap-gate::ingress + 2 via the sovereign bodies in-file, counted
  mechanically); the former 2-value [SKIP] pin is updated per its own
  reality-moved clause.
- `xip_t1_riscv_elf_size_court`: the repinned gate (above).
- All pre-existing courts green: header court (M1-M12 corpus + C11/C++
  compile legs), verify_p28 corpus, wave-9 integration courts (A04/A14/A20/
  A05/A07), xip.ld ASSERT court, F2-6 surface-freeze court (green probe +
  cc-compiled fifth-verb fixture flip).

## Divergences (all recorded, none silent)

1. Cargo.toml dep edges added (DISCLOSED UNAVOIDABLE CROSS-TOUCH; lease was
   sovereign.rs + cgap.h + receipt): cgap-core, cgap-gate, cgap-bn3,
   cgap-blake3, cgap-parquet (host-gated `cfg(not(target_os = "none"))`,
   same form as the Wave-F2 cgap-p28 precedent; cgap-parquet carries
   `features = ["std"]` for the FileSink Tier-H handoff). Coordinator owns
   the manifest; review-or-revert per the audit/cgap-gate precedent.
2. Eval rule-table framing is ABI-LOCAL (PINNED, disclosed): A11's
   rule-record wire framing is future work (A11 §4: wire/pack future), so
   cgap_eval_bn3 pins a word-stream framing over the landed A11/A12 laws
   (sigil words, LE wire form via BN3Statement::to_le_bytes law). Owner is
   A11/A12; when a landed wire format exists, the pin re-anchors and the
   pin-parity court is the flip sensor. Word encoding (sigil + LE) is NOT
   invented — only the record framing is.
3. Lane semantics pinned: lane[i] = projected head row i OBJECT word,
   refused typed outside [0, i32::MAX]; cap overflow -> LEASE, fail-closed
   (never truncation). A19 §2 does not define the lane value mapping — this
   is the sovereign-surface pin, flip-sensored by the domain court.
4. schema_id/rule_count serve 0 in the admitted view: the v1 A04 §6 wire
   header carries neither field; a future frame version (the only
   field-addition path) may populate them.
5. Unaligned C buffers take CgapFrameHeader::decode (same layout parse,
   alignment-free) while aligned buffers take project() — both then run the
   same validate/digest gates (the C surface cannot promise 64-B alignment;
   both legs courted).
6. eval/commit allocate on host via the owner APIs (A12 unifier buffers, A02
   lane vector): the §2 "callee performs no allocation in admit/eval/verify"
   law holds where it is budget-lawful — on the XIP surface, where the three
   symbols are cfg'd out; §2 reachability holds on host (same disclosed
   divergence class as the existing verify_p28 XIP block).
7. Commit sink IO failure -> CGAP_ERR_LEASE_EXHAUSTED (the register has no
   IO class; the documented-capacity-exhaustion class is the least-wrong
   typed value). Recorded mapping divergence.
8. T1 SRAM stack term of §4 ("worst-case eval stack, stack-paint on target")
   UNVERIFIED this session: no hardware/qemu. The measured SRAM leg is
   data+bss only, labeled in-court.
9. Staticlib context row basis: 264,881 B is the raw .a file size from the
   coordinator receipt; this session's in-court llvm-size -A flash-split of
   the same artifact class = 444,213 B / sram 4 B (different basis: summed
   input sections, post-GAP-1 source). Both labeled CONTEXT, never asserted.
10. Status-ledger producer counts now include the in-file sovereign-body
    sites (9 SHACL + 5 LEASE in sovereign.rs); penstock remains the
    owner-lane producer for the other three values.
11. LANE-DOC vs REALITY: the dispatch said header is include/cgap.h + lease
    sovereign.rs; Cargo.toml was fenced. The body wiring was impossible
    without dep edges; the fenced-file touch is disclosed in-item 1 (the
    dispatch's own parenthetical authorizes unavoidable touch + disclosure).

## Receipt tail

Filled from the FINAL suite log at finalize time (below).
## Receipt tail (final run, this session)

- `cargo test -p cgap-ffi --no-fail-fast` (FINAL) -> exit 0:
  lib 42/0 (53.16s), tests/alloc_courts.rs 4/0, tests/zero_copy.rs 5/0.
  Total 51 test bodies green, 0 failed, on the final on-disk sources.
- T1 ELF: flash=150 B / 65536 B, sram=0 B / 32768 B — budgets met, measured
  in-court (llvm-size -A, xip.ld + rust-lld --gc-sections).
- nm -gU (/tmp/gap-1-target/debug/libcgap_ffi.dylib, this session): all four
  sovereign symbols `T` + the six frozen slot exports + `_cgap_p28_invariant_
  check` riding the link (undeclared by design, typed record above).
- `cargo check -p cgap-ffi --target riscv32imc-unknown-none-elf` -> 0.
- Standing: ALIVE on all four dispatch deliverables (three bodies, header
  decision b, T1 repin, courts); one disclosed lease extension (Cargo.toml
  dep edges, item 1).

## Files written

- crates/cgap-ffi/src/sovereign.rs (bodies + courts + T1 repin + ledger)
- crates/cgap-ffi/Cargo.toml (DISCLOSED: 5 host dep edges, item 1)
- docs/jira/v26.11.1/waves/gap-1-sovereign.md (this receipt)
- include/cgap.h: UNTOUCHED (decision b — typed UNSUPPORTED record)
