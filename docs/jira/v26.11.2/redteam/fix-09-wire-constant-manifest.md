# FIX-09 — Wire-Constant Conformance Manifest (the explicit moat)

Lane FIX-09 · /Users/sac/cgap · branch main · 2026-11-02 · writes: this
file only. Response to RED-02 (`red-02-llm-extraction-exploit.md` W5):
every gate in the repo courts the ORIGINAL tree's compiled properties,
so a foreign rewrite never enters their subject universe. This manifest
makes the moat EXPLICIT by inverting that fact: the enumerated wire
constants and receipt-law surfaces below are the conformance suite a
foreign rewrite MUST satisfy byte-exactly to pass the repo's own
courts. All path:line cites re-read on disk this session (grep -n /
sed). Divergences from the dispatch are in §6.

## §1 Status-code register (A19 CgapStatus, u32, 4 B)

| Constant | Value | Source | Pinning court |
|---|---|---|---|
| CGAP_OK | 0x0000_0000 | cgap-ffi/src/sovereign.rs:71 | sovereign.rs:1996 literal sweep |
| CGAP_ERR_INVALID_MAGIC | 0x4347_0001 | sovereign.rs:73 | sovereign.rs:1996 |
| CGAP_ERR_OABI_MISMATCH | 0x4347_0002 | sovereign.rs:75 | sovereign.rs:1996 |
| CGAP_ERR_SHACL_VIOLATION | 0x4347_0003 | sovereign.rs:77 | sovereign.rs:1996 |
| CGAP_ERR_LEASE_EXHAUSTED | 0x4347_0004 | sovereign.rs:80 | sovereign.rs:1837 |
| CGAP_ERR_CONCURRENT_WRITER_DENIED | 0x4347_0006 | sovereign.rs:87 | sovereign.rs:1839 |
| CGAP_ERR_P28_SPOLIATION | 0x5032_38FF | sov.rs:90, trap.rs:36 | sov.rs:1838; trap.rs:42-46 |

Foreign-rewrite obligation: implementer must replicate the full A19
register AND its structure — 0x4347 prefix ("CG") + per-fault byte,
0x5032_38 + 0xFF ("P28"+FF sentinel). trap.rs's ABI-freeze const-assert
pins the derivation from the magic bytes; a rewrite that hardcodes the
literals without the magic-derivation relation passes the value court
but fails the structural law the repo asserts.

## §2 Wire magics

| Constant | Value | Source | Pinning court |
|---|---|---|---|
| MAGIC ("CGAP") | [u8;4] `*b"CGAP"` | penstock.rs:212 | row 2 (:506/:513); offset law :384 |
| P28_LICENSE_MAGIC ("P28S") | [u8;4] `*b"P28S"` | pen.rs:278, att.rs:34 | row 4; att.rs:194/240 |
| HEADER_LEN | 128 | penstock.rs:209 | asserts penstock.rs:381, :414 (ATTESTATION_LEN sum) |
| ATTESTATION_LEN | 80 | attestation.rs:60 | size_of assert attestation.rs:87 |

A17 Wave-1 divergence note (penstock.rs:403-406 doc): attestation magic
is `P28S`, not A04 §6.2's draft `P28\0` — a rewrite following the A04
draft text fails ingress row 4.

## §3 A04 CgapFrameHeader layout (128 B, align 64)

Defined `CgapFrameHeaderAbi` in cgap-gate/src/penstock.rs (static
asserts :381-382: size_of==128, align_of==64). Offset table pinned by
`offset_of!` asserts penstock.rs:384-391 and internal-consistency
asserts :407-414:

magic(0..4) -> version(4..5) -> flags(5..6) -> reserved(6..8) ->
payload_byte_len(8..16, u64 LE) -> oabi_digest(16..48, 32 B BLAKE3) ->
attestation(48..128, 80 B embedded P28 record; OFF_ATTESTATION=48,
penstock.rs:234).

Ingress six-row fail-closed taxonomy: table doc penstock.rs:505-511,
`pub fn ingress` at :513. Row laws a rewrite must reproduce exactly:
(1) len<128 -> INVALID_MAGIC; (2) magic!="CGAP" or version!=1 ->
INVALID_MAGIC; (3) digest all-zero or payload_byte_len > len-128 ->
OABI_MISMATCH; (4) att magic!="P28S" -> P28_SPOLIATION; (5) att
version/lineage/reserved/ceiling/receipt laws -> P28_SPOLIATION;
(6) header flags bits 4-7 set -> ADMITTED (FLAGS_RESERVED_MASK
divergence note). Receipt-offset law: P28_RECEIPT_AT=48,
P28_RECEIPT_LEN=32 (penstock.rs:450/452; consumed :595).

## §4 A17 P28LicenseAttestation layout (80 B, align 8)

Source: cgap-p28/src/attestation.rs; size assert :87. Pinned offsets
(doc comments :70+; mirrored by the gate's offset_of! asserts
penstock.rs:393-400): license_magic 0 (4 B), version 4 (u16),
attestation_flags 6 (u16), safe_harbor_ceiling_cents 8 (u64 LE, MUST
be <= SAFE_HARBOR_CEILING_MAX_CENTS = 5_000_000_000, attestation.rs:57),
origin_authority_digest 16 (32 B BLAKE3), upstream_receipt 48 (32 B
BLAKE3 of parent; zero only if Root FLAG_ROOT=1<<0, attestation.rs:40).
Flag mask law: FLAG_LINEAGE_MASK 0b0111 (:49), FLAG_RESERVED_MASK
0xFFF8 (:53) — reserved bits must be zero (validate :194/:240).
Underlying record type: P28LicenseAttestation (A17); the gate's
`P28LicenseAttestationAbi` is the byte-frozen twin.

## §5 Receipt-chain + SWAR algebra law

- BLAKE3 chain link: `link(prev_hash, block, license[80]) =
  BLAKE3(prev_hash || block || license)` — cgap-ffi/src/receipt_chain.rs:104-106
  (law stated :44; RingReceiptChain struct at :71). GENESIS prev-hash
  at index 0 (:42-43). A rewrite must reproduce not just the digest
  formula but the append law: `validate_intrinsic` gates every append
  (typed refusal, chain unchanged on failure, receipt_chain.rs:108-110).
- BLAKE3 kernel: cgap-blake3/src/lib.rs:824 `hash_many_simd` (multi-
  chunk SIMD path selected at lib.rs:215). Byte-identical to the
  scalar BLAKE3 spec; 32 B digests throughout.
- SWAR prefilter mix: PENSTOCK_PREFILTER_MIX = 0x2544_5491_4F6C_DD1D,
  `penstock_prefilter8_mix(w) = (w ^ (w >> 7)) * MIX` —
  cgap-branchless/src/penstock.rs:30, :35 (consumed by
  penstock_prefilter8 at :44).
- SWAR all-zero detect: HI 0x8080_8080_8080_8080, NOT_HI
  0x7F7F_7F7F_7F7F_7F7F (branchless/penstock.rs:52-53),
  `swar_all_zero_vl` at :64 — `((v & !HI) + !HI) | v` per-word nonzero
  detect, 16-byte unrolled, byte-identical to `is_zero_at` semantics
  (byte loop tail for len%8). The gate consumes this for row-3 digest
  all-zero and row-5 receipt-zero checks (branchless/penstock.rs:41
  comment; penstock.rs:595).
- swar.rs kernel family (cgap-branchless/src/swar.rs:22 HI; swar_lt
  :33, swar_eq_u64 :44 w/ `(nonzero ^ HI) >> 7) * 0xFF` expansion,
  swar_has_zero_byte :53, swar_range_mask :60): the borrow-free forms
  are the pinned algebra — RED-01 already demonstrated the doc comments
  alone are a complete construction recipe (red-01 §3), so the manifest
  obligation is byte-exact outputs on the exhaustive 524,288-lane
  swar_lt domain sweep (swar.rs:173-186 court).
- SplitMix64/corpus seed non-conflation: multiplier 0xBF58_476D_1CE4_E5B9
  vs corpus seed 0xA20C_0FFE_E5A7_E000 (arithmetic.rs:71-82;
  cgap-parquet/src/bitslice.rs:687-699). Swapping them breaks
  byte-identical replay — a rewrite passing the value courts with
  conflated seeds produces replay-divergent output.

## §6 Divergences from dispatch (doc yields, recorded)

1. Dispatch cites "trap.rs:36" under cgap-ffi; the on-disk referent is
   crates/cgap-p28/src/trap.rs:36 (`ERR_P28_SPOLIATION`). cgap-ffi has
   no trap.rs; the 0x5032_38FF pin in cgap-ffi is sovereign.rs:90.
2. Dispatch cites "BLAKE3 chain link law receipt_chain.rs:71"; :71 is
   `pub struct RingReceiptChain`. The link law itself is
   receipt_chain.rs:104-106 (stated in doc at :44). Manifest cites the
   on-disk truth.
3. red-02's "attestation laws at penstock.rs:366/401/449-451" — verified
   as: A17 field doc comments ~:360-366, offset_of! P28 asserts
   :393-400, P28_RECEIPT_AT consts :450/:452. Cited here at corrected
   precision.

## §7 Standing + fences

- Standing: ALIVE for the manifest itself — every path:line above was
  re-read on disk this session (grep -n / sed outputs captured in
  transcript). No build/test executed: this lane writes ONE doc file;
  there is no compiled subject, so no court could run without
  manufacturing one outside the contract's file list. UNVERIFIED is
  NOT claimed for any constant — the constants are compile-time-pinned
  in their owning crates (verified by reading the const-asserts), not
  by this lane's execution.
- Fences could-break: none — zero diffs to guarded files; the only
  write is this file. Fence law satisfied by stating the surface: the
  cited line numbers are drift-sensitive; any lane editing
  sovereign.rs / penstock.rs / attestation.rs / trap.rs /
  receipt_chain.rs / branchless sources invalidates a §1-§5 cite and
  must update this manifest (ERRATA note, never silent edit).
- Anti-vacuity: the manifest's own falsifier is §1-§5 against the
  tree — if any cited constant/line no longer matches disk, the
  manifest is stale and the divergence must be receipted.

## See Also

- docs/jira/v26.11.2/redteam/red-01-cleanroom-bypass.md
- docs/jira/v26.11.2/redteam/red-02-llm-extraction-exploit.md
- docs/jira/v26.10.31/16-ahdclt.md
- docs/jira/v26.11.2/00-ard-prd.md (A04/A17/A19 specs)
