# FA10 — hostile forensic lane: enclave budget gate, --profile-deployment

Date: 2026-11-02 · Lane: FA10 · Repo: /Users/sac/cgap @ main · Adversarial
presumption: the 39,062 B deployment measurement is a vacuum binary whose
minimal callers let `--gc-sections` discard ~80% of each engine.

## 1. What the gate actually exercises (ENTRY_SNIPPETS, cite scripts/enclave_budget_gate.py)

`--profile-deployment` generates a minimal no_std caller per crate under
/tmp/bgp-crate and builds riscv32imc z/fat-LTO/cgu1/panic=abort. The
exercised surface per subject, verbatim from `ENTRY_SNIPPETS` (lines
388-414):

- **cgap-blake3** (lines 390-392): `hash(&data)` +
  `hash_many_simd(&data)`. Two entry fns.
- **cgap-gate** (393-395): `penstock::ingress(&data, len, None)` — one fn.
- **cgap-p28** (396-399): `P28LicenseAttestation::from_bytes(&data[..80])`
  — one fn.
- **cgap-parquet** (400-413): `ffor::ffor_unpack(_, w=8, ..)` +
  `rle_encode` + a volatile read of the results. Two entry fns.

Total: seven entry calls across four crates. No scan, no footer write,
no thrift parse, no page writer, no dictionary, no bitslice, no
mmap_writer, no level_free writer, no p28 ELF/WASM finders.

## 2. Symbol-table truth (fresh run this session)

Fresh gate run reproduced the receipt exactly:

    python3 scripts/enclave_budget_gate.py --profile-deployment \
      --subject cgap-blake3=tier0 --subject cgap-gate=tier0 \
      --subject cgap-p28=tier0  →  envelope=39062/131072, exit 0
    (corrected subjects: blake3=tier0, gate=tier0, p28=tier0,
    parquet=tier2, --deploy-mutant; DEPLOY-MUTANT COURT: 66,176 > 49,152
    breach flips; exit 0)

llvm-nm --size-sort over the four minimal ELFs:

| ELF | gate number | what survives (nm) | absent heavyweights |
|---|---|---|---|
| blake3 | 35,440 B | `hash_many_simd_x2` 30,444 B + hash folded in — the SIMD engine is the ELF | nothing material |
| gate | 484 B | `read_u64_le` 56 B + inlined `ingress` | none — penstock's whole audit rlib is 1,048 B |
| p28 | 886 B | `from_bytes` 234 B + unwrap glue | `find_in_elf`, `find_in_wasm`, `instrument_header_span`, `origin_authority_digest`, `trap`, all of `asc450` (2,363 lines) |
| parquet | 2,252 B | inlined ffor_unpack + `rle_encode` 456 B + `write_varint` 72 B + alloc glue | scan reader, footer seal (`block_digest` 458 B present only in maximal), thrift parse (`skip_value` 708 B only in maximal), page writer, dictionary, bitslice, packing, mmap_writer, level_free writer |

Coverage ratios (linked minimal ELF / fresh audit-profile rlib member
totals, measured this session):

| crate | linked ELF | audit rlib | coverage |
|---|---|---|---|
| cgap-blake3 | 35,440 | 114,072 | **31.1%** |
| cgap-gate | 484 | 1,048 | **46.2%** (crate is genuinely tiny) |
| cgap-p28 | 886 | 9,220 | **9.6%** |
| cgap-parquet | 2,252 | 55,799 | **4.0%** |

The audit-rlib denominators were re-measured this session
(/tmp/fa10-audit-target lease; llvm-ar x + llvm-size -A member Totals)
and match w6-budget-elf-profile.md exactly (114,072 / 1,048 / 9,220 /
55,799).

## 3. The adversarial maximal consumer

Caller crate: /tmp/fa10-max-consumer (not in the repo). It calls every
reachable pub fn across the four crates under the SAME profile
(riscv32imc, z/fat-LTO/cgu1/abort):

- blake3: compress_scalar, compress, hash_chunk, hash,
  hash_wide_simd, hash_many_simd, hash_many_simd_x2, reduce_level_simd
- gate: ingress (None and Some(&mut FrameView)), prefilter8
- p28: from_bytes, instrument_header_span, origin_authority_digest,
  find_in_elf, find_in_wasm, trap::spoliation_halt (fn-pointer keep-alive)
- parquet: ffor_pack/unpack, rle_encode/decode, packing, unpack,
  unpack_filter, packed_filter, bitslice (pack_vertical,
  scan_range_vertical x3, sum_masked, semi_join_masks, semi_join_two_hop),
  byteslice, stream_compact, oblivious, encode_fast, page_aligned, page,
  dispatch_table, compiled_manifest, dictionary, footer (fnv1a64,
  fnv1a64_seeded, block_digest, verify_seal), thrift, level_free writer
  (write_schema_elements, parse_schema_elements, write_footer, write_file,
  write_page U32, admit_schema, align_rows_256b, write_row_group_page)

**TRUE whole-engine size: 102,187 B .text+.rodata** (gate method,
llvm-size -A family sum) in ONE link, versus the gate-reported 39,062 B —
a 2.62x underreport. Symbol-table witnesses: `hash_many_simd_x2` 30,470 B,
`hash_wide_simd` 17,514 B, `block_digest` 458 B, `ingress` 484 B,
12 thrift symbols, B-tree node split glue from dictionary — all present.

### Structural finding: the scan reader cannot be in any no_std image

`crates/cgap-parquet/src/lib.rs:40-41`:

    #[cfg(feature = "std")]
    pub mod scan;

`scan::parse_footer` — the entire Parquet reader path — is std-gated and
STRUCTURALLY ABSENT from every no_std deployment measurement, including
the maximal consumer. The gate's tier2 subject is a write/decode subset of
a crate whose reader does not exist in the measured image class.

## 4. Verdicts

| suspicion | verdict |
|---|---|
| "39,062 B is a vacuum binary" | **SPLIT.** blake3: NOT vacuum (31.1% coverage; the 30 KB SIMD engine genuinely survives; matches the coordinator A/B symbol truth). gate: NOT vacuum (ingress genuinely is 484 B; coverage 46.2% of a tiny crate). p28: **vacuum-class** (9.6%; four pub fns + trap + asc450 absent, though small in bytes). parquet: **VACUUM** (4.0% coverage; the tier2 verdict passes against ffor+rle while footer/thrift/page/dictionary/bitslice/packing/level_free-writer and the whole std-gated scan reader are absent). |
| "`--gc-sections` discards 80% of the engine" | **CONFIRMED for parquet and p28** (96% and 90.4% stripped); **refuted for blake3** (the engine dominates the ELF). |
| "gate measures minimum entry surface, not 'the engine fits'" | **CONFIRMED — it is the former**. The gate output line ("all 4 linked deployment ELF(s) under ... the 131072 B whole envelope") never states the exercised-subset fact; a reader of gate stdout alone cannot know parquet's 2,252 B is 4% of the crate. |
| "is 'the engine fits' claim fraud?" | **Partially rescuable.** The maximal one-link consumer (every reachable no_std pub fn) is 102,187 B — under the 131,072 B envelope with 22% headroom. So the CLAIM is true for the no_std image class, but the GATE DOES NOT MEASURE IT; the truth was established by this probe, not by the gate. Note the probe excludes std-gated scan by construction. |
| "w6 / lane-10 disclosed the exercised-subset caveat" | **NO.** w6-budget-elf-profile.md discloses the vacuum-history fix (11,136/514/58/38-126 B ELFs), the caller-scaffold exclusion, and the alloc prerequisite — honest on everything except coverage. It contains NO exercised-subset/coverage disclosure. lane-10-enclave-gate.md discloses the "three SEPARATE linked binaries, not one enclave image" caveat verbatim ("HONEST CAVEAT: these are three SEPARATE linked binaries...") — honest on envelope composition, silent on exercised-subset/coverage. |
| "the gate could be legitimate if disclosed" | **YES — with mandatory disclosure**. Required fixes: (a) ENTRY_SNIPPETS per-crate coverage note in the gate stdout/summary line, (blake3 ~31% / gate ~46% / p28 ~10% / parquet ~4%); (b) a maximal-consumer arm (this probe, 102,187 B) as a second, binding court; (c) rename the verdict semantics from "the engine fits" to "minimum entry surface fits". With those, the measurement is a legitimate entry-surface court. As shipped, the stdout wording overclaims. |
| "the std-gated scan module makes the tier2 measurement structurally misleading" | **CONFIRMED.** A no_std enclave can never read Parquet files in this codebase; any deployment claim that implies parquet READ capability in-enclave is false. Only write/decode kernels are enclave-legal. |

## 5. Commands + exits (this session)

- gate rerun (4 subjects + --deploy-mutant) → exit 0, envelope 39,062/131,072, mutant 66,176 > 49,152 flip
- solo blake3 rerun (baseline ELF rebuild) → exit 0
- maximal consumer build: `cd /tmp/fa10-max-consumer && CARGO_TARGET_DIR=/tmp/fa10-target cargo +nightly build --release --target riscv32imc-unknown-none-elf` → exit 0, 102,187 B .text+.rodata
- audit rlibs: `cargo +nightly build --profile audit -p <crate> --target riscv32imc-unknown-none-elf` x4 → exit 0 x4, member Totals 114,072/1,048/9,220/55,799
- llvm-nm --size-sort over 5 ELFs (4 minimal + 1 maximal) → all counts in §2/§3

Standing: ALIVE (all commands + exits this session). Repo source: zero
edits (only write = this file). Leases deleted below.
