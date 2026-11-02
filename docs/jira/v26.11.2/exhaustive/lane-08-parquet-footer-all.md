# Lane EX8 — exhaustive census: cgap-parquet persistence (footer.rs, mmap_writer.rs)

Read-only census, 2026-11-02. No builds (lane contract). Lines cited are current on-disk.
Census = every non-test fn; courts (test mod) tallied, not enumerated.

## FA7 asymmetry status (as-of-now)
**OPEN, no repair landed.** Seal: `FooterSealer::new` checks body/row_groups/spans bounds
(footer.rs:424-482) but has NO rows bound; `build_meta_block` emits
`w.zigzag_i64(self.rows as i64)` (footer.rs:579) — rows in [2^63, 2^64) wraps to a NEGATIVE
wire i64, and `seal()` completes Ok. Verify: `parse_meta` refuses `rows < 0`
(footer.rs:860) → writable-but-unverifiable block, the W6-F1 asymmetry class at the rows
bound. Same class as W7-F1 (spans, footer.rs:457-469). Repair shape: one up-front refusal
in `FooterSealer::new` (`rows > i64::MAX as u64` → `RunOverrun`), plus a court arm. Writer
side: `rows_total` accumulates u64::from(rows-per-slab); 2^63 rows is scale-BLOCKED in-process
(same class the W7-F1 court documents at mmap_writer.rs:1209-1212 — the >2 GiB arm).
## footer.rs census (production fns, 15)
| fn | lines | branch sites | class | disposition |
|---|---|---|---|---|
| fnv1a64 | 212-214 | 0 | hot (trailer checksum) | KEEP (thin delegate) |
| fnv1a64_seeded | 226-254 | SWAR unroll ×8, tail loop | hot, L2-optimized | KEEP (court-pinned vs byte oracle) |
| block_digest | 267-299 | 4-lane SWAR | hot, L4-optimized | KEEP (interleaved, court-pinned) |
| SealDigests::over_body | 321-334 | 6-state seed | hot, P2-landed | KEEP (fused 3→1 pass, already applied) |
| SealDigests::absorb | 339-362 | SWAR + lane loop | hot | KEEP (P2 fused; no further pass reduction available — 6 states already share one read) |
| SealDigests::{body_digest,checksum,content_digest} | 365-381 | 0 | cold accessors | KEEP |
| MetaBuf::new/overrun | 103-112 | — | cold | KEEP (typed-refusal mechanism) |
| MetaBuf::push | 114-121 | 1 (capacity) | hot (per byte) | KEEP — REJECT P1-batching here: capacity check IS the typed-refusal mechanism (overflow = typed RunOverrun, never silent); a batched bulk write must still bounds-check the whole claim up front, saving at most one branch per byte at B+M scale (~KBs) — noise vs digest sweeps |
| MetaBuf::extend | 123-130 | 1 | cold (8 B/invocation) | KEEP |
| MetaBuf::varint | 133-145 | loop | hot-small | KEEP; P2 varint hoist N/A (already minimal; only 1-2 calls per meta block at fixed fields) |
| MetaBuf::field_header | 148-164 | delta vs long form | cold (6 calls/seal) | KEEP |
| MetaBuf::{zigzag_i32,zigzag_i64} | 166-172 | 0 | cold | KEEP |
| FooterSealer::new | 418-499 | 7 guard branches | cold, typed-refusal cluster | KEEP + **P1 candidacy: the ONLY real P1 site** — see substitution note |
| FooterSealer::seal | 503-562 | capacity/checked-add cluster | hot-once | KEEP (P2 landed: SealDigests fusion; checked arithmetic is the overflow-refusal mechanism) |
| build_meta_block | 569-603 | list short/long form | cold | KEEP except FA7 repair |
| SealedBlock::{bytes,digest} | 615-619 | 0 | cold | KEEP |
| flush_to | 631-645 | 1 (window len) | cold | KEEP (W8-F1 repaired, court-pinned) |
| verify_seal | 706-780 | shape/attest/checksum/digest ladder | hot (read path) | KEEP; substitution rejected: region localization (:754) re-runs fnv1a64(body) — a second full body pass on EVERY checksum mismatch. Cold path (corruption only), so KEEP; if the campaign adds a body-digest pre-check, reuse SealDigests state 0 instead of a second pass |
| parse_meta | 794-880 | per-field match | hot-ish (every verify) | KEEP; Vec::with_capacity(n.min(1024)) :831 is the only seal-adjacent alloc — verify path only, ADR-sanctioned |

**Substitution candidates (P1 batched-capacity / P2 varint hoist, per BR6):**
- P1: `FooterSealer::new` — REJECT for MetaBuf (bounds = typed-refusal mechanism, REJECT
  class per lane doc). ONE genuine candidate: `write_slab`'s per-lane `page_len` pre-walk
  (mmap_writer.rs:243-260) already batches the capacity claim up front — that IS the P1
  form, landed. No further P1 surface in footer.rs.
- P2: already landed (SealDigests fusion, block_digest interleaving, fnv1a64 SWAR). Varint
  hoist sites named by BR6 (encode_hybrid_into) have no varint inside a per-byte loop —
  push_varint is per-header, not per-byte. No open P2 site in footer.rs.

## mmap_writer.rs census (production fns, 20)

| fn | lines | branches | class | disposition |
|---|---|---|---|---|
| StaticPageBuffer::new/as_slice/as_mut_slice/writer | 104-126 | 0 | cold | KEEP |
| StaticPageWriter::new/body_len/row_groups/rows_total/spans/body/into_buffer | 181-220 | 0 | cold accessors | KEEP |
| write_slab | 230-293 | 6 guard branches | hot entry | KEEP (P1 form: up-front page_len sum = batched capacity claim, landed) |
| encode_page_into | 296-311 | 0 | hot | KEEP |
| varint_len | 329-332 | 0 | hot-small | KEEP |
| push_varint | 335-356 | cap check/iter | hot-small | KEEP |
| push_byte | 359-371 | 1 | **hot, per byte** | THE BR6 hot loop. REJECT naive P1 (bulk-claim) ONLY if it drops the per-byte check — the check IS the typed refusal; a lawful P1 variant (claim `groups*8` once per literal batch, then unchecked slice write) would save one bounds branch/byte at ~GB scale but rewires the shared `push_byte` used by headers — churn >> gain at current buffers (64 KiB-4 MiB). KEEP |
| hybrid_len | 376-417 | run/literal scan | hot (pre-pass) | KEEP — this is a SECOND scan of values (encode_hybrid_into re-scans). Candidate: fold length pre-computation into the encode pass. REJECT: the pre-pass is what makes `write_slab`'s up-front all-or-nothing refusal possible (refuse before byte 0 — C6 law); folding it breaks the refuse-before-write contract. KEEP |
| encode_hybrid_into | 421-480 | BR6-mapped :421-480 | hot loop | KEEP (differentially pinned byte-exact vs rle_encode; per-byte push capacity checks = typed-refusal mechanism, REJECT class) |
| page_header_len | 484-503 | 0 | hot-small | KEEP (lockstep w/ writer, differential-pinned) |
| write_page_header_into | 508-566 | 0 | hot-small | KEEP |
| put_varint_scratch | 571-586 | 1 | hot-small | KEEP (debug_assert ties it to page_header_len) |
| size_i32 | 588-590 | 1 | cold | KEEP |
| page_len | 593-596 | 0 | hot (per lane) | KEEP |
| BlockSink trait | 607-614 | — | seam | KEEP |
| StaticWindow::{new,as_slice,window,commit} | 622-654 | 1 | cold | KEEP |
| FileSink::new/window/commit | 670-703 | 2 | cold | KEEP (IO→RunOverrun remap documented) |
| RingSink::{new,head,wraps,io_failed,reset} | 737-768 | 0 | cold | KEEP; sovereign.rs/ring.rs ownership is cgap-ffi's (absent here — mark; FA9FIX owns sovereign 0x4347_0006 + drain guard AS-OF; EX11/ffi lane covers sovereign) |
| RingSink::drain_to_spill | 775-804 | 3 | cold-on-rotation | KEEP; FA9 durability note on-file :776-780 (flush ≠ sync_all) — FA9FIX territory, AS-OF |
| BlockSink for &mut RingSink::{window,commit} | 808-844 | 5 | steady-state commit path | KEEP (rotation drain-fail refuses typed; head untouched on failure) |

## Coverage ledger

footer.rs production fns: 20 rows (incl. accessor clusters + SealDigests 5 fns) — 100%.
mmap_writer.rs production fns: 24 entries (incl. accessor clusters + sinks) — 100%.
Court inventory (witnessed in-file, mutation-tagged): MUT-FOOT-1..11 minus gaps, MUT-IDENT-1,
MUT-CUT, W6-F1/F2/F3, W7-F1, W8-F1, W9-F1, MUT-AUD-SUMMARY, C1-C7, A04 identity court,
A05 power-cut sweep, A20 1M-row stress, C5 ratio fence. No court without a named mutation
witness except C7 (external python; loud SKIP if python3 absent — loud, typed).

## Divergences / marks

- FA7 rows>=2^63 asymmetry: OPEN (footer.rs:579 seal-Ok vs :860 verify-Err). No repair
  landed as of this census; owner FA7.
- sovereign.rs / ring.rs: live in cgap-ffi (not in this lane's two files) — marked; FA9FIX
  in flight owns sovereign 0x4347_0006 + drain guard (AS-OF); EX11 or the ffi lane covers.
- encoding_hybrid BR6 line range shifted post-edit: current span is :421-480 (was
  :421-482); content matches the BR6 map.
- No P1/P2 open substitution site found; both levers already landed in these two files.
