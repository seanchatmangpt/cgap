## 4. Design: the wire-shm zero-copy byte path

### 4.1 Frame format (implemented, ALIVE)

Big-endian, fixed 16-B header: `[u32 magic 0x5749_5230 ("WIR0")] [u32
crc32(payload)] [u32 payload_len] [u32 frame_ordinal]` followed by payload
(receipt: crates/wire-shm/src/lib.rs:3-4,19-20). CRC is table-driven
CRC-32/IEEE, reflected, poly 0xEDB8_8320, init/xorout 0xFFFF_FFFF
(receipt: ../lanes/lane-wire-shm.md:10-12; known vector
crc32_ieee(b"123456789") == 0xCBF4_3926 asserted at
crates/wire-shm/tests/wire_courts.rs:21-24). Slots are 4096-B stride:
16-B header + up to 4080 payload bytes (receipt:
crates/wire-shm/src/lib.rs:21-23). Typed errors, no stringly-typed
failure: FrameTooLarge, CrcMismatch, StaleOrdinal, BadMagic, Io
(receipt: crates/wire-shm/src/lib.rs:26-32; witnessed instances
FrameTooLarge {len 5000, max 4080} and typed StaleOrdinal at
../lanes/lane-wire-shm.md:33-36).

### 4.2 Segment layout — offset-addressed file segment, NOT mmap

Honest scoping: the segment is an offset-addressed file-backed segment
(std::fs + FileExt read_all_at/write_all_at), NOT mmap — mmap requires
libc and the zero-dep law forbids it (receipt:
../lanes/lane-wire-shm.md:13-15; implementation
crates/wire-shm/src/ring.rs:7-10). Layout: 4096-B bookkeeping region
(u64 BE next_ordinal, u32 BE capacity) + fixed 4096-B slots
(receipt: crates/wire-shm/src/ring.rs:1-5). Ordinals are strictly
increasing per writer; next_ordinal is persisted AFTER the frame so
recovery never sees a torn frame (receipt:
crates/wire-shm/src/ring.rs:113-115; invariant proven at
../lanes/lane-wire-shm.md:30-36). Because the path is positioned
read/write into one file segment rather than zero-copy pointer sharing,
"zero-copy" here means zero-serialization on the byte path (binary frames
never re-encode payloads), not zero-copy in the mmap page-sharing sense.

### 4.3 cgap-mcp bridge (landed, ALIVE)

The bridge is BUILT: crates/cgap-mcp (v26.11.3, bin cgap-mcpd), mapping
wire-shm frames onto the MCP JSON-RPC surface with field equality
preserved — JSON -> wire frame -> JSON round-trip asserted
field-for-field through handle_line on a real WireRing file, with
on-disk slot byte-cross-checks against the wire-shm frame format
(receipt: ../../../../crates/cgap-mcp/RECEIPT.md:20-23). Courts, all
green (cargo test exit 0, 5 passed / 0 failed): (1) round-trip field
equality across 4 message cases; (2) protocol shape over REAL stdio via
a spawned cgap-mcpd (initialize protocolVersion 2024-11-05, tools/list,
tools/call error codes -32602/-32601/-32700); (3) zero-alloc encode
window with an allocating anti-vacuity control; (4) mutation court
FLIPPED in a /tmp sandbox (skipped ordinal -> BadMagic via -32603),
sha256-exact restore verified; (5) persistence across reopen (receipt:
../../../../crates/cgap-mcp/RECEIPT.md:12-16,20-42). Honest scope: a
MINIMAL JSON-RPC MCP-shape subset (initialize, tools/list, tools/call),
NOT full MCP spec conformance (receipt:
../../../../crates/cgap-mcp/RECEIPT.md:24-29). No latency claim is made
for the frame path itself — p17 measured sizes only, no time figures
(receipt: ../bench-validate/p17-token-churn.md:20-24); bridge wire
round-trip latency remains [TARGET-v26.11.2]. The MCP JSON-RPC shape the
bridge preserves is the p01 envelope: {"jsonrpc","id","method",
"params"} newline-delimited on stdio (receipt:
../bench-validate/p01-stdio-baseline.md:23-28).
