# git-substrate-baseline.md

Lane GS4 synthesis. Inputs: `git-survey-lane-01.md`,
`git-survey-lane-02.md`, `git-survey-lane-03.md` (ingested in full),
plus `14-dflss-charter.md` and `15-ctq-tree.md`. No source edits, no
builds. All file:line cites are from the three lane receipts or the
charter/CTQ files read this session (receipt tail below).

## 1. Landed state verdict table

| Surface | Verdict | Grounds |
|---|---|---|
| Runtime deps | LANDED: zero | lane-01 `cargo tree` exit 0; Cargo.toml:15-16 empty |
| no_std release (riscv32imc) | LANDED: clean | lane-01 no-def release build exit 0 |
| no_std debug build | BLOCKED | lane-01: ungated `std::thread_local!` ladder.rs:31 |
| Footprint (.text etc.) | NOT-MEASURABLE | lane-01: bitcode rlib; llc lacks RISC-V backend |
| Object envelope | PARTIAL | lane-02: hash-update seq only, pack_write.rs:131-139 |
| Object types | BLOB-ONLY | lane-02/03: writer hardcodes type 3 (:247,:260) |
| Hashing | LANDED | lane-02: in-crate SHA-1/CRC-32/BLAKE3; SHA-256 absent |
| Oid representation | LANDED (thin) | lane-02: `Oid = [u8;20]` lib.rs:25, no newtype |
| Tree objects + sorting | NOT-IMPLEMENTED | lane-03 §1: no mode parse, no name comparator |
| Pack V2 write + idx V2 | LANDED (tested) | lane-03 §3; real-git interop both directions |
| Pack parse + verify | LANDED (tested) | lane-03 §3 verdict table; checksum verify tested |
| OFS delta (type 6) | NOT-IMPLEMENTED | lane-03 §3; latent fail-open gap (§4) |
| REF delta (type 7) | NOT-IMPLEMENTED | lane-03 §3; same latent gap |
| zlib / inflate | LANDED | lane-03 §4: vs python3 zlib + real git fixture |
| Storage seam | ONE-DIRECTIONAL | lane-03 §2: crc32 identity claim only; no sink refs |
| Loose-object path | IN-MEMORY | lane-03 §2: heap Vecs; no std::fs in src/ |
| Alloc posture | alloc-riding | lane-02 H3: 20+ hot heap sites (inventory in §5) |

## 2. CTQ gap matrix

CHARTER-ID MAPPING NOTE: the lane brief named "CTQ-G1..G7"; the
charter on disk carries no such ids. Actual structure: goals G1-G6
(14-dflss-charter.md:70-73) and 14 CTQ characteristic rows across
segments S1-S4 (15-ctq-tree.md:21,45,63,83). The matrix uses the
charter's actual rows. Anything outside the three receipts or charter
text is marked UNASSESSED.

| Charter CTQ (segment, cite) | Target Spec | Landed State | Severity |
|---|---|---|---|
| Binary footprint (S1, :23) | ELF <115 KiB, USL 131,072 B | NOT-MEASURABLE (lane-01 §2) | HIGH |
| Zero-heap sovereignty (S1, :24) | 0 allocs (hard) | NOT LANDED: 20+ sites (lane-02 H3) | HIGH |
| Spoliation fails closed (S1, :25) | 0x5032_38FF exact | UNASSESSED (pin FFI-side) | UNASS. |
| BLAKE3 receipt coverage (S1, :26) | 100% | PARTIAL: BLAKE3 over .cgap-dag, not trees | MEDIUM |
| Frame ingress (S2, :47) | <15 ns/frame | UNASSESSED: git not on the frame chain | UNASS. |
| Commit cost (S2, :48) | <4 us/tx | UNASSESSED: loose path never journaled | UNASS. |
| blr/brk census (S2, :49) | 0/0 | BLOCKED: no cgap-git ELF exists to census | BLOCKED |
| Throughput (S3, :65) | >=1M facts/s | UNASSESSED (TARGET campaign-wide, :69-72) | UNASS. |
| Durable replay (S3, :66) | 0-byte divergence | PARTIAL: roundtrip tested; no WAL seam | MEDIUM |
| Typed refusals (S3, :67) | 100% CgapStatus | MIXED: tests typed + one fail-open (§4) | HIGH |
| Receipt coverage (S4, :85) | 100% cited | LANDED for this doc | OK |
| Court non-vacuity (S4, :86) | every court flips | PARTIAL: courts as-of-date only | MEDIUM |
| License pin (S4, :87) | 0 drift | UNASSESSED (pin in cgap-p28) | UNASS. |
| Seed non-conflation (S4, :88) | pair intact | UNASSESSED (pins in cgap-parquet) | UNASS. |

Footprint detail: the .rcgu.o inside the release rlib is LLVM bitcode
and the pinned toolchain llc has no riscv32imc backend (lane-01 §2);
only host-side proxy bytes exist (rlib 229,928 B, rmeta 323,964 B).
These stay TARGETs until a `-Z build-std=core` + rust-lld route.

## 3. Cheapest first move: thread_local -> no_std debug flip

The single ungated no_std blocker in the crate is
`std::thread_local!` inside `#[cfg(debug_assertions)] mod probe_count`
(ladder.rs:31, gated module ladder.rs:27-32; lane-01 §2). The
release/no-default build is already clean (lane-01 exit 0), so the
entire remaining cost of "cgap-git builds no_std in every profile on
riscv32imc" is one module. Two fixes, both crate-local:

1. Narrow the gate to
   `#[cfg(all(debug_assertions, feature = "std"))]` (lane-01's stated
   fix) — one attribute edit.
2. Or replace `thread_local!` with a core-only debug counter.

Recommendation: option 1 (smallest diff, zero behavior change), then a
court asserting `cargo check -p cgap-git --no-default-features`
(debug) exits 0 on riscv32imc — anti-vacuity mutation is reverting the
gate and expecting exit 101 at ladder.rs:31. After the flip, the
footprint route (`-Z build-std=core` + rust-lld) becomes the next
unblock for the S1 Binary-footprint CTQ, which is currently
NOT-MEASURABLE, not merely unmet.

## 4. Typed defect: type-6/7 fail-open misparse

`parse_pack_v2` rejects only type codes 0 and 5 (pack_write.rs:390-392;
lane-03 §3), while `encode_object_header` syntactically accepts types
1-4, 6, 7 (pack_write.rs:209). Delta resolution is NOT-IMPLEMENTED
crate-wide — no `apply_delta` symbol exists (lane-03 §3). Consequence:
a pack carrying an OFS_DELTA (6) or REF_DELTA (7) header is parsed with
the delta payload consumed as a zlib blob body; on adversarial input
the zlib check can even succeed, yielding a silently wrong blob OID
instead of a refusal. This is a fail-open misparse against the S3
Typed-refusals CTQ (100% typed, 15-ctq-tree.md:67).

Recommendation: refuse types 6/7 in the parser with a typed error
variant, matching the documented delta-free design ("no deltas, no new
type codes", lib.rs:6-7). Court: inject a synthetic type-6 header into
a /tmp pack copy, assert the typed refusal appears post-fix and is
absent pre-fix (flips). Until then cgap-git must be documented unsafe
for third-party pack input, not merely "delta-free by design".

## 5. Allocation sites inventory (zero-alloc law input)

Hot-path heap sites from lane-02 H3 — each awaits the S1 0-alloc law:

- pack_write.rs:208 header-varint Vec per object; :242 pack buffer;
  :276 idx buffer; :322 entries table; :355 owned blob copy (`to_vec`);
  :425 + :513/:526/:527 reader tables (objects, oid/crc/offset).
- dag.rs:930,938,939,973 per-commit index/Oid/level/scratch vecs;
  :947+ `Box<TreeNode>` per node (2L-1 boxes); :1035 proof path vec.
- deflate.rs:80 bit-writer out, :193 token buffer, :288 deflate output.
- inflate.rs:180/:319 Huffman tables, :262 output buffer.

Already zero-alloc (patterns to generalize): CRC table const/static
(pack_write.rs:165), SHA-1/BLAKE3 stack state, `[u8; 256]` idx fanout
stack table (pack_write.rs:286), 128 B stack payload buffer
(dag.rs:839). That stack-buffer pattern is the fix template, but no
caller-sink trait exists in the crate today (lane-03 §2: no
RingSink/BlockSink references), so the conversion is blocked on a
sink/buffer design decision.

## 6. Charter distance (synthesis verdict)

Order of attack:

1. thread_local debug-profile flip (§3) — one attribute; precondition
   for any footprint measurement.
2. Type-6/7 parser refusal (§4) — small typed-error diff; closes the
   only known fail-open path and satisfies a hard CTQ row.
3. Footprint route (§3 tail) — `-Z build-std=core` + rust-lld; without
   it the flagship S1 CTQ is NOT-MEASURABLE.
4. Tree-object encoder + canonical name sort (lane-03 §1) — the
   largest new construction; real git tree interop today exists only
   as a test-level `git_object_oid("tree", ...)` call (lane-02 H4,
   tests/lineage_workflow.rs:193); the production API never mints
   tree OIDs.
5. Delta resolution (OFS/REF) — the largest real charter distance only
   if third-party pack ingestion is in scope; if cgap-git stays a
   writer of its own delta-free packs, the §4 typed refusal is the
   sufficient landing.
6. Zero-alloc conversion (§5) — mechanical but wide (20+ sites);
   blocked on the sink/buffer decision.

Storage seam note: no coupling work is needed. The dependency is
already one-directional via a CRC32 identity claim
(cgap-storage/src/atomic_commit.rs:21,82,382; lane-03 §2), and
cgap-git holds no RingSink/mmap_writer/BlockSink references. Any
journal integration would be new construction in cgap-storage, not a
cgap-git change.

## Receipt tail: commands + exits

| Command | Exit |
|---|---|
| Read git-survey-lane-01/02/03.md (full) | 0 |
| grep -n "CTQ" 14-dflss-charter.md 15-ctq-tree.md | 0 |
| Read 14-dflss-charter.md (offset 60) | 0 |
| Read 15-ctq-tree.md (offset 15) | 0 |
| awk length audit + utf8 read-back of this file | 0 |

Standing: ALIVE for synthesis (all inputs read this session, commands
above); every substantive claim is grounded in a lane receipt or
charter cite; UNASSESSED rows carry no inference. No builds, no source
edits, no Process.sleep.
