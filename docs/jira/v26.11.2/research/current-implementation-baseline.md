# current-implementation-baseline

State-of-implementation extraction, lane S1, 2026-11-02.
Subject: /Users/sac/cgap @ main 9a730320e9c536814e4dbbc88b66af43b7a11da8.
Every number is tagged MEASURED (run this session, command+exit in the
tail), RECEIPT (cited wave receipt line), IN-FLIGHT, or [TARGET].

## 1. Workspace census

Root `Cargo.toml` members: 23 member paths (22 crates + 2 ontology
members `ontologies/fibo`, `ontologies/cloud`); `crates/cli` is a
standalone crate with its own `[workspace]` (root Cargo.toml:26-27
NOTE comment). Workspace `version = "26.11.2"` (root Cargo.toml:37).

| crate | #![no_std] | alloc site | size (du -sk) |
|---|---|---|---|
| cgap | 5:#![no_std] | none | 16KB |
| cgap-arrow | 18:#![no_std], extern crate alloc at :36 | 2 sites | 508KB |
| cgap-blake3 | 11:#![no_std] | none | 124KB |
| cgap-bn3 | none — std-dependent | 1 site | 464KB |
| cgap-bont | none — std-dependent | 1 site | 252KB |
| cgap-bosym | none — std-dependent | 0 | 76KB |
| cgap-branchless | 15:#![no_std] | 1 (tests only) | 72KB |
| cgap-core | none — std-dependent | 0 | 180KB |
| cgap-court | none (lib clean no_std-class) | 0 | 32KB |
| cgap-crypto | 6:#![no_std], extern crate alloc :9 | 1 | 124KB |
| cgap-ffi | no attr; NO_STD-CLEAN per m2 | 0 | 332KB |
| cgap-gate | 6:#![no_std], extern crate alloc :8 | 1 | 108KB |
| cgap-git | 9:#![no_std] + `extern crate std;` :13 | 1 | 388KB |
| cgap-p28 | 11:#![no_std], extern crate alloc :18 | 1 | 236KB |
| cgap-parquet | 10:#![no_std], extern crate alloc :13 | 1 | 776KB |

| cgap-storage | none — std-dependent | 0 | 152KB |
| cgap-verification | not a workspace member (on disk only) | 0 | 52KB |
| cgap-beam/gpu/audit/bench/integration | none (std host surfaces) | 0 | 228/20/168/524/96KB |
| cli | standalone, 757MB on disk (incl. target/) | 0 | — |

Binary-size grounding (RECEIPT, m1-footprint-rerun.md:24-34): gate rlib
.text=940 B / .rodata=0 (riscv32imc); blake3 rlib .text=113,360 B;
p28 rlib .text=7,140 B / .rodata=1,212 B; wasm CODE sums arrow=84,951,
parquet=66,780 (drifted from p2 under floating nightly — per-nightly
numbers, m1:38-41). blake3 dominant object `hash_wide_simd` 50,088 B,
`hash` 30,176 B (m1:44-46). A/B witness for hash_wide_simd portability:
throughput table in p1-blake3-optlevel-ub-fix.md:38
(hash_wide_simd=0.43 GiB/s vs hash_many_simd=1.85 GiB/s); source
`crates/cgap-blake3/src/lib.rs:696` is unconditional portable; arch
gates are aarch64 at lib.rs:306/:653 and x86_64 at :453/:629 with
`target_feature = "avx2"` at :632.

no_std census grounding (RECEIPT, m2-nostd-census.md table): NO_STD-CLEAN
= {arrow, blake3, court(lib), crypto, ffi, gate, p28, parquet} — note
m2 also classifies cgap-ffi NO_STD-CLEAN despite no attribute line I
could find in its lib.rs head; STD-DEPENDENT set = {cgap-core, cgap-bont,
cgap-bosym, cgap-bn3, cgap-storage, cgap-git (explicit `extern crate
std;` at lib.rs:13), cgap-audit, cgap-integration, bench, cgap
(transitive)}; cgap-beam/wasm-footprint TARGET-UNSUPPORTED (cdylib);
cgap-gpu host-only toolchain-missing.

## 2. Structural primitives

- Statement struct `BN3Statement`: crates/cgap-bn3/src/sigil.rs:305-313
  (`#[repr(C)]`, three u64 term words = 24 B, FFI-reinterpretable).
- Sigil tags bits 56-63: sigil.rs:85-117 (enum `Sigil` 0x00..0x04,
  0x05..=0xFF decoder-refused).
- u56 truncation `to_u56(a06_slot) = a06_slot >> 8` (sigil.rs:294-296):
  drops the LOW byte, PROVISIONAL per spec Section 10 risk 1.
- SlotId minting: crates/cgap-bont/src/primitives.rs:61 (`SlotId(pub
  u64)`, frozen first-8-bytes LE window), `truncate64` at
  primitives.rs:136-144, `lex_hash` = blake3 of namespace-tagged lexeme.
- cgap-branchless (crates/cgap-branchless/src/): five modules, landed
  mid-wave per w6-b1/b2/b3/b4/b5 receipts:
  - arithmetic.rs — BRANCHLESS-LANDED (B2: `saturated_subs_p28` with
    128-bit borrow-out + tripwire, `poison_mask` zero-fold; differential
    4096-pair court exit 0, w6-b2-arithmetic.md).
  - swar.rs — BRANCHLESS-LANDED (B3: eq/zero/range/lt masks, 50k-pair
    differential; 0 branch instructions in asm probe, w6-b3-swar.md).
  - eytzinger.rs — BRANCHLESS-LANDED (B4; exhaustive size differential
    vs partition_point, w6-b1-scaffold.md).
  - radix.rs — SHIPPED-PARITY REPLICA (B5, sha256-pinned to
    unify.rs:728-731 kernel; M1 court retired cannot-flip,
    w6-b5-radix.md).
  - select.rs — NAIVE-REFERENCE ONLY ("branchless form lands in lane
    B2" header; never landed).
- Consumers: cgap-gate has the path dep landed
  (crates/cgap-gate/Cargo.toml:16) but ZERO body-site swaps
  (w6-b7-gate-consume.md verdict PARTIAL/BLOCKED-on-B2/B3: no shipped
  primitive matches prefilter8 or is_zero_at semantics); cgap-p28 has
  NO dep (w6-b8-p28-consume.md: zero eligible swap sites in p28 — the
  only `saturating*` hit is a test comment at asc450.rs:1491; dep
  deliberately not added to a publish=true crate without consumption).

## 3. Tier 0 (gate / crypto / blake3)

- Penstock two-stage admission (crates/cgap-gate/src/penstock.rs): the
  admit path mints `view.prefilter = prefilter8(bytes)` and
  `view.escalate = u16::from(bytes[OFF_FLAGS]) & ESCALATE_MASK != 0`
  (penstock.rs:608-615). `prefilter8` = `(w ^ (w>>7)) *
  0x2544_5491_4F6C_DD1D` on first 8 header bytes (penstock.rs:667-682,
  explicitly NOT the frozen seeds). `ESCALATE_MASK` at penstock.rs:271
  (= 0x00F0 on header byte 5 per w4-p3-escalation-corpus.md:16).
- Skip-not-pay law (RECEIPT, w4-p3-escalation-corpus.md:9-51): W3-L1's
  10.1x holds but as the ADMISSION LAW itself, not a hot escalated
  branch; escalated corpus rows are UNDEFINED-INTEREST by design —
  a vacuous escalated branch is the named vacuity, corrected from a
  mechanism claim to a law claim.
- P28 monotonic counter: the lease tick is a monotonic INCREMENT
  (`Lease::tick`, crates/cgap-crypto/src/lease.rs:20-58 — Tick newtype
  with no set_tick; rewind unrepresentable). p28 consumes via
  `LeaseEpochSealer` (asc450.rs:364-415, calls only `lease.tick()`).
  There is NO saturating-decrement site anywhere in the substrate
  (w6-b8-p28-consume.md census: one test-comment hit asc450.rs:1491).
- `ERR_P28_SPOLIATION = 0x5032_38FF`: real home
  crates/cgap-p28/src/trap.rs:36, const-assert-pinned to LICENSE_MAGIC
  at trap.rs:42-48 (build fails on drift).
- cgap-gate alloc/panic census: 0 alloc symbols / 0 panic symbols
  (RECEIPT m1-footprint-rerun.md:50-53, llvm-nm, 7 symbol lines).
- blake3 cfg inventory: aarch64 gates lib.rs:306/:653 (neon target_feature
  :382), x86_64 gates :453/:629 (avx2 :529/:632); `hash_wide_simd`
  (lib.rs:696) is UNCONDITIONAL portable and byte-identical to `hash`
  (lib.rs:823 doc). .text under z/fat-LTO/cgu1: 40,208 B
  [TARGET — coordinator run 2026-11-02 /tmp/blk-gate per dispatch;
  m1 measured 113,360 B rlib .text under the audit profile, so the two
  figures differ by profile, not contradiction].

## 4. Tier 1 (arrow / bn3 join / storage closure)

- cgap-arrow: `&[u64]` slabs (crates/cgap-arrow/src/slab.rs, 1,930
  lines; 64-byte cache-aligned slab, digest-sealed, typed refusals
  W6/W8 repair slices, slab.rs:1-25). C-ABI: c_abi.rs 284 lines
  fixed-width; c_abi_nested.rs 521 lines varlen (wc -l, MEASURED this
  session).
- Join kernel: bn3 radix semi-join in crates/cgap-bn3/src/unify.rs
  (radix bucket fold unify.rs:728-731, 4096 buckets/16 KiB heads;
  candidate cache + mask_s survival mask). The pre-L3 nested-loop arm
  is RETIRED but still on disk at unify.rs:1301-1322 as the M-L3-1
  differential arm. Verdict LANDED-NOT-MET (RECEIPT
  w5-l3-remeasure.md): radix:nested = 0.97-0.98x @1M
  (1.648 s vs 1.674 s fastest), target <75 ms @1M missed 22x; join
  stage dominates the chain baseline (~1.5-1.7 s). Zero allocs across
  all bench samples (divan omitted all-zero columns).
- Storage closure: semi-naive fixpoint
  (crates/cgap-storage/src/closure.rs:94-136, `semi_naive_closure`
  over `&[u32]` edge columns, delta-join until empty, SeenSet dedup);
  dred antijoin landed per p1-dred-retraction-antijoin.md.
- Heap-copy sites (all MEASURED on-disk this session):
  - unify.rs:1371-1373 and :1394 — `push` loops into nxt_rows/nxt_vars
    per match extension; the borrowed-prefix lane is IN-FLIGHT (not
    landed).
  - closure.rs:118-133 — fresh `Vec::new()` next_x/next_z delta per
    round.
  - closure.rs:39 — `SeenSet` = `std::collections::HashSet` (blr
    remainder, w5-asm-census-repair.md:29-31).
  - footer.rs:376 and :447 (crates/cgap-parquet/src/footer.rs) — seal
    path allocs 2/36 B; documented lawful carve-out at footer.rs:52-55
    (zero-alloc law scopes to the FLUSH path C4, not the seal path).

## 5. Tier 2 (parquet parse / git pack write)

- Parse surface: footer parse via compact-protocol forward parser
  (crates/cgap-parquet/src/thrift.rs:463 `read_page_header`, lenient
  forward-compat skip-by-wire-type); PLAIN payload direct aliasing
  (scan.rs:26-28: "PLAIN payload bytes ARE the Arrow buffer bytes").
  NEW this wave: min/max stats whole-row-group pruning landed
  (scan.rs:295 `Bounds`, :320 `rg_prunes`, stats_skip_court.rs court;
  w5-stats-skip.md exit 0, sandbox mutation flip witnessed exit 101
  then sha-exact restore).
- Alloc sites: footer.rs:376/:447 seal 2 allocs/36 B; :704 verify path
  1 alloc/4 B (spans Vec::with_capacity at footer.rs:704 region,
  read this session).
- git pack_write + RingSink commit path (crates/cgap-parquet/src/
  mmap_writer.rs:721-842 RingSink: 64 KiB boxed ring + spill File,
  io_failed typed refusal; crates/cgap-ffi/src/sovereign.rs:729-794
  COMMIT_LEASE: single File::create at first lane slot, steady-state
  zero-alloc except the 2/36 B seal residue). Ring re-witness
  (RECEIPT w5-ring-rewitness.md): 0.755-0.834 µs/tx stage_commit
  (drifted +28-42% vs 0.589-0.603 receipted, environment; G5 target
  <4 µs/tx still MET); lease-edge refusal 65,377 -> 0x43470004
  byte-identical; chain 6.19-6.75 µs/tx REPRODUCED.

## 6. Tooling / gates (first-hand this session unless cited)

- enclave_budget_gate.py --check (no --artifact): exit 2,
  "GATE REFUSED: --check/--mutate need at least one --artifact" —
  subjectless check correctly refuses; matches w5-tollgate.md row 7a
  (exit 2 BLOCKED, subject missing). Non-vacuity --self-check exit 0
  (RECEIPT w5-tollgate.md:7b).
- verify_ost_design_around.py AS-OF 2026-11-02T18:02Z, exit 1 FAIL:
  3/4 DISTINCT-MECHANISM (US11,216,456 / US11,409,698 / US11,645,278,
  each "zero src hits for patented mechanism; landed cite verified")
  and US10,817,467 PENDING ("status column PENDING (no landed evidence
  recorded)") — G7-PATENT failing row unchanged from w5-tollgate.md
  row 8. Note: a G7R lane may be re-flipping rows mid-flight; this is
  the as-of-run record, not a claim about later state.
- zero_string_gate.py: exit 0 (scanned=62 hits=0 per w5-tollgate.md:6a;
  rerun exit 0 this session). boundary_gate.py --strings: exit 0 rerun
  this session. Self-check non-vacuity: RECEIPT w5-tollgate.md:6b.
- asm census (RECEIPT w5-asm-census-repair.md): bn3 fixpoint region
  0/0/0 (blr/br_xN/brk) post-D5 static-seeded hasher; remainders:
  RuleCompiler::compile std RandomState TLS blr x2 (unify.rs asm lines
  3990-3994) and storage closure.rs:39 HashSet — both blr-class, both
  D5-backlog.
- CLI: crates/cli standalone ggen six-pack; generated_cli.rs carries
  62 verb/fn markers (grep count, MEASURED); custom_handlers.rs
  exposes self_version/bench_mask/bench_parquet/align_test (the
  cnv:CustomBehavior seam, 4 handlers).
- Workspace test build: `cargo test --workspace --no-run` exit 0
  (MEASURED this session; 2 pre-existing cgap-integration unused-dep
  manifest warnings). Full suites NOT run (dispatch scope).

## 7. Summary table

| Crate | Real no_std | Real Heap Free | Main Landed Mechanism | Blocking Defect/Seam |
|---|---|---|---|---|
|cgap-gate|yes (6)|yes, 0/0 alloc/panic|penstock 2-stage|branchless dep, 0 swaps (w6-b7)|
|cgap-crypto|yes (6)|yes (alloc :9)|monotonic Lease::tick|tick ceiling owner unnamed (w6-b8)|
|cgap-p28|yes (11)|yes (alloc :18)|trap 0x5032_38FF|no saturating site; no br-dep|
|cgap-blake3|yes (11)|yes|portable hash_wide_simd|0.43 GiB/s slowest SIMD leg [RECEIPT]|
|cgap-arrow|yes (18)|yes (:36)|64B slab + C-ABI 284/521 ln|from_bytes accepts repaired W6/W8|
|cgap-parquet|yes (10)|yes (:13)|PLAIN alias + rg_prunes skip|seal/verify 2/36B + 1/4B (lawful)|
|cgap-git|attr yes; std deliberate (:13)|partial|pack_write + mmap writer|D5 backlog: explicit std|
|cgap-ffi|no attr (m2 clean)|yes|ring commit 0.755-0.834 us/tx|stage_commit drift +28-42%|
|cgap-bn3|no (std-dep)|no|radix semi-join, 0/0/0 fixpt|L3 LANDED-NOT-MET 0.97-0.98x@1M|
|cgap-storage|no (std-dep)|no|semi-naive closure, antijoin|closure.rs:39 HashSet blr|
|cgap-bont|no (std-dep)|no|SlotId mint, blake3 first-8|D5 std strip backlog|
|cgap-core/bosym|no (std-dep)|no|frame / offset_of|D5 backlog (E0463-class)|
|cgap-branchless|yes (15)|tests only|4/5 modules branchless|select.rs still naive-reference|
|cgap|no (transitive std)|no|umbrella crate|transitive std via cgap-git|
|cli (standalone)|host tool|host|62-verb CLI, 4 handlers|cnv:CustomBehavior seam|

## Receipt tail (commands + exits, this session)

1. `sed`/`grep`/`wc` source reads over exact files/lines above — exit 0
2. `python3 scripts/enclave_budget_gate.py --check` — exit 2
   (subjectless refusal, loud)
3. `python3 scripts/verify_ost_design_around.py` — exit 1 (G7-PATENT
   FAIL 3/4 DISTINCT, US10,817,467 PENDING)
4. `python3 scripts/zero_string_gate.py` — exit 0
5. `python3 scripts/boundary_gate.py --strings` — exit 0
6. `cargo test --workspace --no-run` — exit 0
7. `git rev-parse HEAD` — 9a730320e9c536814e4dbbc88b66af43b7a11da8
8. Timestamp of gate runs: 2026-11-02T18:02:05Z

Standing: ALIVE for every first-hand command above; RECEIPT-cited rows
carry their receipt's standing (L3 LANDED-NOT-MET, ring drift typed,
tollgate NOT CLEARED pending G7-PATENT).
