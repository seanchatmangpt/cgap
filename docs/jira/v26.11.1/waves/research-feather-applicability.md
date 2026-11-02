# research-feather-applicability (R-4)

Date: 2026-11-01. Lane: R-4, Explore-type research + synthesis, no code changes.
Receipt path (this file): `docs/jira/v26.11.1/waves/research-feather-applicability.md`.

## Method and grounding

- All external URLs below were fetched live this session (HTTP 200 unless noted).
  GitHub issue numbers were confirmed via the GitHub search API this session.
- Ground truth read on disk: `crates/cgap-parquet/src/footer.rs` (footer framing),
  `docs/jira/v26.11.1/lanes/A02-parquet-mmap.md` (C7 external-reader court).
- Every memory-sourced claim is tagged [MEMORY] and carries a lower confidence note.

## 0. Ground-truth correction (divergence from dispatch)

The dispatch context states "cgap's footer carries a BLAKE3 digest". On disk,
`crates/cgap-parquet/src/footer.rs:31-40` says the integrity digest is
**FNV-1a-64** (non-cryptographic, fixed basis/prime), with `block_digest` as four
FNV-1a-64 lanes (32 bytes), plus an 80-byte attestation in the trailer
(`footer.rs:19-26, 86`). The synthesis below therefore speaks of "cgap footer
digest (FNV-1a-64 lanes / BLAKE3 if A16-class crypto lands)" — the mapping
question is digest-agnostic; only the size changes (32 B either way as 4 lanes).

## 1. Prior art: engines exposing native format + Arrow-IPC compatibility

The Arrow IPC file format ("Feather V2") is a flatbuffer-metadata envelope
around record batches: footer = `{version, schema, dictionaries, recordBatches,
custom_metadata}` + trailing magic `ARROW1` — no digest, no attestation field.
Confirmed from the format source: File.fbs Footer table
(https://github.com/apache/arrow/blob/main/format/File.fbs, fetched live;
format overview: https://arrow.apache.org/docs/format/IPC.html).

Engines that carry both a native columnar format and an IPC compatibility
surface:

1. **arrow-rs parquet <-> arrow bridge** (`parquet::arrow`, the canonical
   example of a columnar format gaining an Arrow view). Cost: Arrow metadata
   (field nullable-ness, logical types, dictionary encoding) must round-trip
   through parquet thrift `KeyValue` / logical-type fields; record-batch
   semantics (offsets, validity bitmaps) are reconstructed on read, so the
   bridge is a *re-materialization* boundary, not zero-copy.
   https://docs.rs/parquet/latest/parquet/arrow/index.html (live 200).
   Consequence for cgap: the same shape — cgap pages stay the native store;
   IPC is an emit/consume projection.
2. **Polars IPC writer** (`polars-io/src/ipc/mod.rs`) — Polars' in-memory
   format is its own chunked Series; IPC is a projection that re-chunks and
   maps Polars dtypes to Arrow types, with lossy mappings for some dtypes
   (e.g. time zones, unsigned dictionaries) [MEMORY for the lossy list].
   https://github.com/pola-rs/polars/blob/main/crates/polars-io/src/ipc/mod.rs
   (live 200). Cost: dtype mapping table + potential chunk copies.
3. **DuckDB over Arrow / Feather**: DuckDB consumes Arrow through the Arrow
   C data interface / `register_arrow`-style scans rather than by parsing IPC
   natively in its storage engine; the native format (its own block storage)
   stays sovereign, IPC/Parquet are ingest surfaces
   (https://duckdb.org/docs/guides/python/sql_on_arrow, live 200). This is the
   closest structural analogue to stardf: the C-ABI is the bridge, not the
   file format. Earlier feather-guide URL
   (duckdb.org/docs/guides/import/feather.html) now 404s — docs moved; treat
   old feather-guide citations as stale.
4. **DataFusion TableProvider over custom formats**: DataFusion's extension
   point is a Rust trait (`TableProvider`) returning Arrow `Schema` + stream
   of `RecordBatch`; every custom-format bridge (delta, ice, custom page
   stores) is a provider that translates its format's batches into Arrow
   batches at scan time.
   https://docs.rs/datafusion/latest/datafusion/datasource/trait.TableProvider.html
   (live 200). This is exactly the cgap-datafusion seam shape: provider over
   cgap pages, decode_into -> RecordBatch via zero-copy from mmap'd slices.

Common cost across all of these: **the compatibility layer owns one copy
boundary** (foreign metadata -> Arrow schema, and possibly buffer re-alignment
to Arrow's 8/64-byte alignment rules), plus a metadata-mapping table that must
be total or the layer refuses.

## 2. The mapping question: integrity metadata into Arrow IPC

Arrow IPC has no digest concept in the format. File.fbs `Footer` fields are
exactly `{version, schema, dictionaries, recordBatches, custom_metadata}`;
the only sanctioned extension hook is `custom_metadata: [KeyValue]` on the
footer (and per-field `custom_metadata` on `Field` in Schema.fbs [MEMORY for
exact Field location]). Verified: File.fbs live fetch shows
`/// User-defined metadata` `custom_metadata: [KeyValue]` as the last Footer
field.

Checksummed-columnar prior art lives on the **Parquet side**, not the IPC side:

- Parquet page CRC: apache/arrow#42461 "[C++][Parquet] Support CRC checksums in
  pages" (confirmed open/closed via GitHub API this session; URL
  https://github.com/apache/arrow/issues/42461), and the still-open follow-up
  apache/arrow#53138 showing a downstream consumer silently reading corrupt
  pages when verification is not exposed — evidence that a checksum nobody
  verifies is dead weight (the exact anti-vacuity lesson for cgap: a digest
  must be *verified at the consume surface*, not merely written).
- Implication: prior art puts integrity in the **native format's page/footer
  layer**, and treats the interchange format (IPC) as unverified transport.
  No engine found that maps its own digest into IPC `custom_metadata` as a
  load-bearing integrity mechanism [MEMORY: delta-lake table JSON digests are
  transaction-log-side, not file-side; unverified live this session].

For cgap: keep the FNV/BLAKE digest in the cgap footer (native, verified by
`verify_seal`), and emit it into IPC footer `custom_metadata` keys (e.g.
`cgap.block_digest`, `cgap.footer_len`, `cgap.magic`) as *advisory* provenance
— never as the integrity mechanism for IPC consumers, because stock
arrow-rs/polars readers ignore custom_metadata. A "checksummed Arrow file"
that stock readers accept but never verify is exactly arrow#53138's failure
mode; the honest claim is "cgap-sealed files embed their digest as provenance;
integrity is enforced on the native path."

## 3. Synthesis: recommended spec shape for a cgap feather/IPC emit+consume surface

Section outline only (no full spec, per dispatch):

- S1 Scope & non-goals: IPC is a projection, cgap sealed pages remain the
  store of record; no cgap footer parsing by foreign readers ever required.
- S2 Emit (`cgap page -> IPC file`): one RecordBatch per pack; mmap-backed
  buffers published zero-copy where 8/64-byte alignment permits (Arrow
  alignment rule [MEMORY]), one copy boundary otherwise; footer
  `custom_metadata` carries `cgap.magic`, `cgap.block_digest`,
  `cgap.footer_len`, `cgap.attestation_sha256` (digest of the 80-B
  attestation, not the raw bytes — raw attestation is P28-shaped, don't leak).
- S3 Consume (`IPC file -> cgap`): accept only cgap-emitted files for sealed
  round-trip (S2 metadata present => verify digest, refuse on mismatch);
  accept foreign IPC files as unsealed ingest (digest = none), typed refusal
  otherwise.
- S4 Schema mapping table: total fn, cgap scalar types -> Arrow types; no
  lossy entries; unknown type => typed UNSUPPORTED, never silent coercion
  (mirrors polars lossy-mapping lesson in S1.2).
- S5 Falsifiable claims (perf): zero-copy path achieves no-allocation decode;
  copy path bounded by one memcpy; numbers are TARGETS until measured with a
  stated method (criterion, pinned page fixtures).
- S6 Courts: (a) round-trip court — emit -> stock arrow-rs reader -> consume,
  assert equal schema+data+digest; (b) mutation court — flip one byte in the
  IPC body, S3 verify must refuse (anti-vacuity: this court must flip);
  (c) foreign-file court — arrow-rs-written IPC consumes as unsealed.
- S7 Divergence register: rows for anything stock readers cannot honor.

**Falsifier for the whole spec shape**: if a stock arrow-rs IPC reader can
read cgap-emitted files AND silently accept a body mutation (no cgap-side
verify enforces integrity on the round trip), then the "IPC as verified
projection of a sealed store" claim is wrong and the surface is just a
converter — at which point the S3 sealed-refusal path is dead code and should
be deleted (ERRC: delete the claim, keep the converter).

## 4. Source register (all fetched live 2026-11-01 unless noted)

| Claim | Source | Status |
|---|---|---|
| IPC footer has no digest; `custom_metadata` is the only hook | https://github.com/apache/arrow/blob/main/format/File.fbs | live 200, fields read |
| IPC format overview | https://arrow.apache.org/docs/format/IPC.html | live 200 |
| Arrow integration/endianness doc | https://arrow.apache.org/docs/format/Integration.html | live 200 |
| parquet<->arrow bridge | https://docs.rs/parquet/latest/parquet/arrow/index.html | live 200 |
| Polars IPC writer location | https://github.com/pola-rs/polars/blob/main/crates/polars-io/src/ipc/mod.rs | live 200 |
| DuckDB Arrow ingest shape | https://duckdb.org/docs/guides/python/sql_on_arrow | live 200 |
| DuckDB parquet docs | https://duckdb.org/docs/data/parquet/overview | live 200 |
| DataFusion TableProvider trait | https://docs.rs/datafusion/latest/datafusion/datasource/trait.TableProvider.html | live 200 |
| Parquet page CRC prior art | https://github.com/apache/arrow/issues/42461 | via GitHub search API |
| Unverified-page failure mode | https://github.com/apache/arrow/issues/53138 | via GitHub search API |
| arrow2 docs (discontinued crate) | https://crates.io/crates/arrow2 | 403/404 — no live citation |
| DuckDB feather guide | https://duckdb.org/docs/guides/import/feather.html | 404 — docs moved |

## Standing

PARTIAL. Synthesis and prior-art register are grounded in live fetches and
on-disk cgap sources; the [MEMORY]-tagged items (Polars lossy-dtype list,
Schema.fbs Field custom_metadata location, Delta transaction-log digest
placement, Arrow 8/64-byte alignment rule specifics) are UNVERIFIED this
session and must be re-derived before any spec lands. No code changed; this
file is the only write.

## Divergences from dispatch

1. Dispatch said "cgap's footer carries a BLAKE3 digest"; on disk
   (crates/cgap-parquet/src/footer.rs:31-40) the digest is FNV-1a-64 lanes +
   80-B attestation. Receipt treats the mapping question as digest-agnostic.
2. Dispatch said "online" research; live web fetch was available and used
   (curl), so this is grounded, not survey-from-memory — except where
   [MEMORY]-tagged.
