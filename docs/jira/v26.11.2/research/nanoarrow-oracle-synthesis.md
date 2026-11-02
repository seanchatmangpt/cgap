# Nanoarrow Oracle Synthesis

Lane N8 (harvest-integration), v26.11.2 starDF campaign.

## 1. Vendoring record

- upstream: apache/arrow-nanoarrow (vendored tree, not a submodule)
- HEAD: `65d2067606289e5dc3d1e25f7dbf27c224cf04ad` (git log -1 in vendor/nanoarrow, exit 0,
  this session)
- describe: `apache-arrow-nanoarrow-0.10.0.dev-43-g65d2067` (git describe --tags, exit 0)
- date: Thu Oct 29 10:43:34 2026 -0600; subject: fix(python): Support iterating map arrays as
  Python objects (#998)
- gitignore: `vendor/` and `**/vendor/` both present (`.gitignore:35-36`); both patterns fire,
  tree is untracked. Working copy is dirty with lane N1 compile-lease objects
  (`/tmp/n1-build`: array.o, schema.o, utils.o, array_stream.o, rv32_*.o.tmp).

## 2. Footprint baseline vs cgap receipts

cgap receipts of record (same-method anchors; methods are NOT interchangeable):

- cgap-gate .text, riscv32imc audit profile: **940 B** (llvm-size -A on rlib sections)
  -- receipts: m1-footprint-rerun.md:28, p2-footprint-flash-ladder.md:30
- cgap-blake3 linked z/fat-LTO/cgu1 ELF: **40,208 B** -- receipt: bn3-status-report.md:78
- whole enclave envelope: **131,072 B** ceiling -- receipt: w5-budget-gate.md:20
- witnessed breach arithmetic: 144,486 B -- receipt: w5-budget-gate.md:55,82

Comparability discipline: 940 B is an rlib `.text` sum, 40,208 B is a linked cgu1 ELF,
144,486 B is a section-total sum. The nanoarrow tree is C with no LTO-cgu model, so any
nanoarrow footprint number is [NOT-COMPARABLE] to the above until a common method exists.
nanoarrow itself has no receipts; per-object sizes N2 measured are [N2-lane pending
verification]. Sections 3-5 are first-hand reads of vendor/nanoarrow and crates/cgap-arrow.

## 3. Rosetta Stone: nanoarrow C construct -> cgap Rust equivalent

Verdicts are first-hand (this lane read both sides). Repo root /Users/sac/cgap.

### R1

- nanoarrow: ArrowArrayView struct: non-owning view carrying offset/length/null_count/storage_type/
  layout/buffer_views/children/dictionary + variadic buffers (common/inline_types.h:826-879)
- cgap: ViewVec / ViewVecBuilder (crates/cgap-arrow/src/views.rs:26,80); Primitive<T> (primitive.rs)
  for typed runs
- verdict: ALREADY-HAVE (partial): cgap has no per-instance layout cache; layout is recomputed at
  type dispatch in layout.rs

### R2

- nanoarrow: ArrowLayout: buffer_type/buffer_data_type/element_size_bits/child_size_elements cached
  per view (inline_types.h:802-815)
- cgap: generated_layout.rs const tables (crates/cgap-arrow/src/generated_layout.rs)
- verdict: ALREADY-HAVE (static) + DIVERGENT-RECORDED: nanoarrow caches per instance, cgap
  precomputes const tables -- cgap's form fits the 128 KiB gate (no heap per view)

### R3

- nanoarrow: ArrowBufferAllocator vtable {reallocate, free, private_data} (inline_types.h:753-763)
  plus ArrowBufferDeallocator callbacks (nanoarrow.h:46-47, 214-218)
- cgap: Buf fixed-cap buffer, no realloc vtable (crates/cgap-arrow/src/buffer.rs:18-107);
  mmu_arena.rs arena (mmu_arena.rs:1-127)
- verdict: DIVERGENT-RECORDED: nanoarrow is malloc-based with dealloc callbacks; cgap is
  arena/static. Adopting the vtable adds a fn-pointer table + heap growth -- rejected under kernel
  no-alloc law (.claude/rules/kernels-no-std.md)

### R4

- nanoarrow: ArrowBitsUnpackInt8/32 word-loop bitmask unpack (common/inline_buffer.h:406-475)
- cgap: PresenceMask<CAP,BYTES> const-generic stack mask (crates/cgap-arrow/src/bitmask.rs:55);
  bitmap.rs Bitmap
- verdict: ALREADY-HAVE: cgap's const-generic form unrolls at compile time, no loop overhead

### R5

- nanoarrow: C-data-interface release protocol: ArrowArrayReleaseInternal walks children +
  dictionary, resets buffers, sets release=NULL (common/array.c:28-73)
- cgap: ForeignArray Drop -> release exactly once (c_abi.rs:199-247); double-release is a CAS-tier
  no-op (c_abi.rs:255-283 test)
- verdict: ALREADY-HAVE and stronger: cgap's CAS no-op on double release beats nanoarrow, whose
  second release call is UB (array.c:73 only NULLs)

### R6

- nanoarrow: ArrowArrayViewSetArray geometry validation (offset/length vs buffer sizes) and
  ArrowArrayViewValidate levels NONE/MINIMAL/DEFAULT/FULL (reader.c:578, 612-616)
- cgap: cgap validates at build time (ViewVecBuilder::push, views.rs:115) and ad-hoc at import
  (foreign.rs)
- verdict: DIVERGENT-RECORDED: cgap has no validation-level ladder; foreign.rs import checks are
  weaker than nanoarrow's four-level contract

### R7

- nanoarrow: IPC continuation state machine: 8-byte continuation+metadata_size read (reader.c:232)
  with legacy-prepend fallback synthesizing 0xFFFFFFFF (reader.c:287-306)
- cgap: CONTINUATION=0xFFFF_FFFF (ipc.rs:36), MetaHeader 8-byte prefix (ipc.rs:54),
  seal_meta/emit/read (ipc.rs:198, 246, 295)
- verdict: ALREADY-HAVE on emit; DIVERGENT-RECORDED on read: cgap rejects legacy producers
  (IpcError, ipc.rs:90) where nanoarrow synthesizes the continuation (reader.c:305-306)

### R8

- nanoarrow: decoder type dispatch switch ladders on
  bitwidth/precision/date_unit/time_unit/interval_unit/union_mode (ipc/decoder.c:823, 843, 872, 894,
  940, 959, 1039, 1125)
- cgap: generated_layout.rs const tables + layout.rs (no runtime switch)
- verdict: ALREADY-HAVE: cgap prunes dispatch at compile time; cgap's form is the oracle-grade
  direction

### R8b

- nanoarrow: union_type_id_map 256-byte type-id<->child map (inline_types.h:863-869)
- cgap: none (no union support in cgap-arrow)
- verdict: DIVERGENT-RECORDED: out of gate scope; recorded, not adopted

### R9

- nanoarrow: ArrowArrayViewGetNumBuffers/GetBufferType/GetBufferView (inline_array.h:921-1046)
- cgap: Buf::as_typed<T> (buffer.rs:100) + slab.rs buffer slots
- verdict: ALREADY-HAVE (narrower: cgap is typed, nanoarrow enum-dispatched)

### R10

- nanoarrow: ArrowSharedBufferInit/Clone refcounted shared buffers (nanoarrow.h:237-265)
- cgap: none; cgap leases via COMMIT_LEASE (slab.rs)
- verdict: DIVERGENT-RECORDED: refcounting is heap-reliant; the cgap lease model is the
  envelope-safe form

### R11

- nanoarrow: variadic buffers for binary/string_view arrays (inline_types.h:871-878)
- cgap: ViewVec data()/views_bytes() (views.rs:48, 54)
- verdict: ALREADY-HAVE (single data buffer, not multi-variadic; adequate for gate scope)

## 4. unsafe pointer/slice invariants for mmap Flash/DMA buffers

Every mmap-wrapped Arrow view must hold all five invariants; each cites authority.


### I1

Alignment: buffer start satisfies the layout's element alignment (8 B for i64/f64/offsets; 64 B for
IPC buffers per Arrow spec). DMA lands page-granular, Flash XIP windows 4 B -- the tighter bound
wins.
- nanoarrow: nanoarrow derives alignment from element_size_bits (inline_types.h:809-810)
- cgap: cgap: slot-aligned leases (slab.rs); ipc.rs:181 meta_span_for pads metadata to alignment

### I2

Length x elem_size: buffer i must cover (offset+length)*element_size_bits/8 exactly as
ArrowArrayViewSetArray computes; under-length is a validation failure, over-length wastes envelope.
- nanoarrow: nanoarrow: array.c SetArray geometry path
- cgap: cgap: foreign.rs import checks; views.rs:60 row() bounds

### I3

Lifetime/lease pairing: the view must not outlive the mmap window. nanoarrow pairs it with
ArrowBufferDeallocator (nanoarrow.h:214-218); cgap pairs it with COMMIT_LEASE + RingSink -- the
lease is the ownership token, the ring slot the storage.
- nanoarrow: nanoarrow: nanoarrow.h:214-218
- cgap: cgap: slab.rs COMMIT_LEASE; the import-side mirror is ForeignArray Drop -> release
  (c_abi.rs:238-247)

### I4

Read-only discipline: Flash is read-only; the view must be const (&[u8] / ArrowBufferView), never
&mut. nanoarrow models it via const ArrowArray* (inline_types.h:829); cgap via the
as_slice/as_mut_slice split (buffer.rs:83-90).
- nanoarrow: nanoarrow: inline_types.h:829
- cgap: cgap: buffer.rs:83-90

### I5

Alignment fallback: if DMA lands unaligned, copy through the arena into a slot; never
ptr::read_unaligned a view path -- it silently violates I1.
- nanoarrow: nanoarrow: no direct oracle (cgap-only rule)
- cgap: cgap: mmu_arena.rs:1-127

Lease-pairing law: every mmap-wrapped view is created after COMMIT_LEASE and dropped
(or explicitly released) before its ring slot is recycled. A view that outlives its lease
is use-after-free by construction -- there is no GC to catch it.

## 5. Immediate structural changes for cgap-arrow (128 KiB gate)

Each item names a falsifier; [TARGET-v26.11.2] unless a receipt backs it.


### C1

Geometry validation ladder ported from ArrowArrayViewValidate (NONE/MINIMAL/DEFAULT/FULL): typed
validate(level) on imported foreign arrays instead of ad-hoc asserts (foreign.rs). Zero-copy is only
sound if geometry is checked once at import, not per access.

Falsifier: Mutation court: truncate an imported length so the buffer under-covers offset+length
under FULL -- FULL must reject, NONE must accept. Court that cannot flip = FAIL.

Tag: [TARGET-v26.11.2]

### C2

Legacy IPC continuation prepend on read (reader.c:305-306 behavior): synthesize the 0xFFFFFFFF
continuation when a legacy producer's first 4 bytes are a metadata size, keeping the 8-byte header
invariant (reader.c:232 / ipc.rs:54).

Falsifier: Shim-court: feed a legacy continuation-less frame to ipc::read -- must parse; feed a
corrupt continuation after the change -- must still reject. Both must flip.

Tag: [TARGET-v26.11.2]

### C3

Const-only mmap view constructor: from_mmap(ptr, len, align) returning a borrowed view tied to a
COMMIT_LEASE guard object, so the borrow checker enforces invariant I3 structurally.

Falsifier: Mutation: hold the view past guard drop -- must be a compile error (compile-abort IS the
falsifier passing); plus a runtime court that guard Drop recycles the slot.

Tag: [TARGET-v26.11.2]

### C4

Keep const-table layout (do NOT port nanoarrow's per-instance ArrowLayout, per R2); instead add a
cross-oracle conformance court asserting generated_layout.rs tables match nanoarrow ArrowLayout
outputs for gate-supported types.

Falsifier: Mutation: flip one element_size_bits entry in a /tmp copy of generated_layout.rs -- the
court must fail; restore sha256-exact + mtime bump.

Tag: [TARGET-v26.11.2]

### C5

Envelope headroom: nanoarrow per-TU object sizes are [N2-lane pending verification]; do not budget
against them until N2's method is receipted. Binding constraint stays the 131,072 B envelope
(w5-budget-gate.md:20) with the witnessed 144,486 B breach arithmetic (w5-budget-gate.md:55).

Falsifier: Re-run the llvm-size -A method post-change; envelope must stay <= 131,072 B or the change
is refused.

Tag: envelope number is receipted (w5-budget-gate.md); the change itself [TARGET-v26.11.2]

## 6. Receipt tail (this lane commands + exits)

All run this session from /Users/sac/cgap.
- `ls vendor/nanoarrow`; grep vend .gitignore -> lines 35-36 (exit 0)
- `git -C vendor/nanoarrow log -1 --format="%H %ad %s"` -> 65d2067...04ad, Oct 29 2026 (exit 0);
  `git describe --tags` -> apache-arrow-nanoarrow-0.10.0.dev-43-g65d2067 (exit 0)
- wc -l sweeps over vendor/nanoarrow/src/nanoarrow/{common,ipc,integration} and
  crates/cgap-arrow/src/*.rs (exit 0)
- targeted greps: nanoarrow.h symbols; inline_types.h structs (760-879); inline_buffer.h ArrowBits
  (366-648); array.c release (28-73); reader.c continuation (192-616); decoder.c dispatch
  (806-1125); cgap pub-surface sweep across views/buffer/bitmask/c_abi/ipc/foreign (exit 0)
- /tmp sibling-output re-check: only /tmp/n1-build (N1 compile lease objects); no N1-N7 findings
  file on disk -- sections 3-5 assembled from FIRST-HAND reads; [N2-lane pending verification] where
  N2 numbers are referenced
- footprint greps: m1-footprint-rerun.md, p2-footprint-flash-ladder.md, w5-budget-gate.md,
  bn3-status-report.md (exit 0)
- doc assembly: python3 /tmp/n8_build_doc.py (exit 0), then awk line-length audit <= 100
