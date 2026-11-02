# Research Receipt: Feather Format Fundamentals (R-1)

Lane: R1-feather-format · 2026-11-01 · Agent: Explore-type research (R-1)
Method: fetched primary sources over network with curl this session; every claim carries an inline source URL. No code changed. No spec invented — anything not confirmed by a fetched source is marked UNVERIFIED.

## 1. History — Feather v1 (wesm/feather, 2016)

**Origin.** Feather was "a portable file format for storing Arrow tables or data frames ... created early in the Arrow project as a proof of concept for fast, language-agnostic data frame storage" for Python (pandas) and R — source: https://arrow.apache.org/docs/python/feather.html . Initial bindings: Python written by Wes McKinney, R written by Hadley Wickham (source: https://raw.githubusercontent.com/wesm/feather/master/README.md ). Development moved into Apache Arrow; the standalone repo's README now says "Feather Development is in Apache Arrow now" and the Python `feather` package "is now a wrapper around pyarrow.feather" (same README URL).

**On-disk layout (v1).** From the v1 format spec, https://raw.githubusercontent.com/wesm/feather/master/doc/FORMAT.md :

```
<4-byte magic number "FEA1">
<ARRAY 0>
<ARRAY 1>
...
<ARRAY n>
<METADATA>
<uint32: metadata size>
<4-byte magic number "FEA1" >
```

- Magic is the 4-byte string `FEA1`, at BOTH head and tail (tail = head mirrored, metadata sandwiched before the trailing size+magic). Source: FORMAT.md above.
- Arrays are laid end-to-end; metadata is appended at the END of the file, preceded by a little-endian uint32 metadata size (FORMAT.md). The v1 spec text does not state the metadata encoding in FORMAT.md itself; the metadata is a Flatbuffers-defined table set in the C++ implementation — this specific claim is UNVERIFIED from the sources fetched this session and is not load-bearing for cgap.
- Primitive arrays: optional byte-aligned null bitmask (1 bit/value, 1=valid, LSB right-to-left bit-numbering), omitted entirely when `null_count = 0` in metadata, followed by a contiguous fixed-width values array (FORMAT.md).
- Variable-length (BINARY/UTF8): bitmask + `int32` offsets (N+1 offsets so the last value's end is derivable) + raw value data, explicitly reusing "the Apache Arrow encoding" (FORMAT.md).
- Dictionary-encoded: `<uint32 dictionary size> <dictionary values array> <int32 indices array>` (FORMAT.md).

**Why v1 was deprecated in favor of v2.** Three converging primary-source statements:

1. Type coverage: v1 supported "only a small subset of Arrow types" (historical note in the Arrow Columnar spec, https://arrow.apache.org/docs/format/Columnar.html#format-ipc — the "Feather V2" historical-note block under IPC File Format; page fetched via https://arrow.apache.org/docs/format/IPC.html which redirects there). pyarrow's page agrees: V1 files "lack many features, such as the ability to store all Arrow data types" ( https://arrow.apache.org/docs/python/feather.html ).
2. Structural limits: the 0.17.0 release announcement states Feather V2 "resolves the 2GB-per-column limitation for large amounts of string data that the original Feather implementation had" ( https://github.com/apache/arrow-site/blob/main/_posts/2020-04-21-0.17.0-release.md , "Feather V2" section).
3. No compression in v1: "V1 files also lack compression support" ( https://arrow.apache.org/docs/python/feather.html ).

pyarrow deprecation status: "Reading and writing V1 files is deprecated as of 25.0.0 and will be removed in a future version. Rewrite V1 files in the Arrow IPC file format (Feather V2)." ( https://arrow.apache.org/docs/python/feather.html ).

## 2. Feather v2 == Arrow IPC file format

**The exact relationship.** "Version 2 (V2), the default version, ... is exactly represented as the Arrow IPC file format on disk. V2 files support storing all Arrow data types as well as compression with LZ4 or ZSTD. V2 was first made available in Apache Arrow 0.17.0" ( https://arrow.apache.org/docs/python/feather.html ). The Columnar spec's historical note confirms files of this format "were sometimes called 'Feather V2' or named with the .feather extension" ( https://arrow.apache.org/docs/format/Columnar.html#format-ipc ). So v2 is not a wrapper with any additional framing — it IS the IPC file format; the only "wrapper" aspect is the `.feather` naming and the default-LZ4 write convention in pyarrow's `feather.write_feather` (the docs explicitly note `feather.write_feather` defaults to LZ4 while `ipc.new_file` does not compress by default — same URL). "A thin wrapper over the stream format" in the lane doc's phrasing is imprecise: v2 wraps the STREAM as the payload of the file envelope, but the v2 on-disk bytes are byte-for-byte an IPC FILE, not a stream.

**File structure (Columnar spec, IPC File Format section, https://arrow.apache.org/docs/format/Columnar.html#format-ipc ):**

```
<magic number "ARROW1">
<empty padding bytes [to 8 byte alignment]>
<STREAMING FORMAT with EOS>
<FOOTER>
<FOOTER SIZE: spec says little-endian int32>
<magic number "ARROW1">
```

- Tail-sealed: 6-byte magic `ARROW1` at head (padded to 8-byte boundary), then the full IPC stream including optional EOS marker, then the footer flatbuffer (`Footer` table, defined in `File.fbs`), a little-endian int32 footer size, and the 6-byte magic again. The footer carries "a redundant copy of the schema ... plus memory offsets and sizes for each of the data blocks in the file", enabling random access to any record batch (same URL).
- Footer-vs-stream equivalence rules: compliant writers MUST make the footer's metadata version, schema and custom_metadata identical to the first stream Message's; SHOULD list dictionaries/record batches in the same order as the stream (same URL, "Equivalence with the IPC Streaming Format").
- File-format dictionary semantics differ from streams: no ordering requirement for initial dictionary batches, NO dictionary replacement support, delta dictionaries applied in footer order (same URL, "Deviations from the IPC Streaming Format").
- Stream framing inside the envelope: each encapsulated message = 32-bit continuation `0xFFFFFFFF` (introduced in Arrow 0.15.0 "in part to address the 8-byte alignment requirement of Flatbuffers"), 32-bit LE metadata size, Message flatbuffer, padding to 8 bytes, then the body; the complete message must be a multiple of 8 bytes so messages are relocatable between streams (same URL, "Encapsulated message format"). Stream EOS = `0xFFFFFFFF 0x00000000` or stream close. Message types: Schema, RecordBatch, DictionaryBatch.
- RecordBatch body: buffers written end-to-end with padding to ensure a minimum 8-byte alignment; the Buffer metadata gives offset+size relative to the batch body start; `size` need not include padding and should be the actual memory size (same URL, "RecordBatch message").
- Alignment guarantees summary: in-IPC alignment is 8-byte (messages and record-batch bodies); the 64-byte alignment/padding recommendation is an in-MEMORY columnar-buffer property ("for interprocess communication" recommended 64-byte alignment/padding, recommended per Intel performance guidance), not an IPC file-layout guarantee ( https://arrow.apache.org/docs/format/Columnar.html#buffer-alignment-and-padding ).

## 3. Compression

- Frame-legal codecs: exactly three options for record-batch body buffers — uncompressed, `lz4`, `zstd`. "lz4" means the LZ4 FRAME format ( https://github.com/lz4/lz4/blob/dev/doc/lz4_Frame_format.md ), explicitly NOT the raw/block format ( https://github.com/lz4/lz4/blob/dev/doc/lz4_Block_format.md ). Source: https://arrow.apache.org/docs/format/Columnar.html#buffer-compression .
- Where compression sits: PER-BUFFER, inside the record-batch message body. All buffers in a batch "must be compressed separately using the same codec"; individual buffers may be left uncompressed (e.g. when compression would not help). Codec selection lives in the optional `compression` field of the RecordBatch message's data header; default is uncompressed (same URL).
- Compressed layout: each compressed buffer stores the 64-bit LE **uncompressed length in its first 8 bytes**; `-1` there marks that buffer as left uncompressed. The data header then lists compressed lengths/offsets (same URL).
- Zero-copy story when uncompressed: the entire IPC design goal is that a reader can reconstruct arrays "using pointer arithmetic and thus no memory copying" — batch metadata provides each buffer's location/size so Array data is rebuilt without copying, and the encapsulated message format exists so a reader can parse metadata and read the body without moving data ( https://arrow.apache.org/docs/format/Columnar.html#format-ipc , Encapsulated message format + RecordBatch message sections). Double compression is discouraged when the transport already compresses ( https://arrow.apache.org/docs/format/Columnar.html#buffer-compression ).
- V1 comparison: Feather v1 had NO compression ( https://arrow.apache.org/docs/python/feather.html ).

## 4. Version support matrix

| Reader/writer | Feather v1 | Feather v2 (= Arrow IPC file) | Source |
|---|---|---|---|
| pyarrow | read+write, DEPRECATED as of 25.0.0, removal planned; `version=1` flag still exists | read+write (default), via `pyarrow.feather` or `pyarrow.ipc` | https://arrow.apache.org/docs/python/feather.html |
| arrow-rs | not found in this session's fetch of the arrow-rs README feature list (only arrow-flight/IPC rows surfaced); V1 support UNVERIFIED | yes — Feather V2 read/write is documented as "exactly the Arrow IPC file format", consumed via arrow-ipc `FileReader`/`FileWriter` (crate docs: https://docs.rs/arrow-ipc ) | https://github.com/apache/arrow-rs README (fetched) + https://docs.rs/arrow-ipc |
| DuckDB | no support found — the `arrow` community extension reads/writes Arrow IPC files/buffers only (`.arrow`/`.arrows`, `read_arrow`, `to_arrow_ipc`); its predecessor core extension read only in-memory buffers | yes, via the `arrow` community extension (nanoarrow-based) since the 2025-05 redesign | https://duckdb.org/community_extensions/extensions/arrow ; https://raw.githubusercontent.com/duckdb/duckdb-web/main/_posts/2025-05-23-arrow-ipc-support-in-duckdb.md |

arrow-rs V1 row is honest-UNVERIFIED: I did not fetch a primary arrow-rs page that states V1 support status; do not cite this cell as measured.

## 5. Closing note — what cgap's footer-sealed page format would need

cgap's footer-sealed page format already shares Feather v2's architecture (head/tail magic seal, end-of-file footer with batch offsets, per-batch buffers with recorded sizes) — the divergence is the payload grammar, not the envelope. To emit a consumable ARROW1 file, cgap would need: (1) a Flatbuffers compiler step generating the `Message`/`Schema`/`File` tables from the canonical .fbs files (Message.fbs, Schema.fbs, File.fbs at https://github.com/apache/arrow/blob/main/format/ — file list verified via repo fetch this session), since flatbuffers are the entire metadata grammar and cgap is zero-external-dep; (2) an 8-byte-aligned encapsulated-message writer with the 0xFFFFFFFF continuation and LE int32 sizes; (3) a footer that duplicates schema+block offsets with the MUST/SHOULD equivalence rules honored, plus int32 footer size and trailing ARROW1; (4) per-buffer LZ4-frame or ZSTD codecs with the 8-byte uncompressed-length prefix per compressed buffer (or no compression, preserving zero-copy mappability); and (5) bit-identical physical layout to Arrow's memory layout (LSB bitmask, Arrow offsets encoding) so buffers can be mapped rather than decoded. Consuming is the harder direction: a consumer must parse flatbuffers without the runtime — feasible (flatbuffers is an offset-based access model, no allocation required) but it is real engineering, not a header check. Honest caveat: cgap could emit ARROW1-compatible bytes before it can claim conformance; conformance requires validating against a real pyarrow/arrow-rs reader, which is the natural court for that lane.
