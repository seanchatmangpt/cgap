# Receipt: cgap-mcp (Wave-3 bridge, pulled forward)

- Subject: crates/cgap-mcp (new crate, this tree only)
- Date: 2026-11-02
- Lane: D
- Standing: ALIVE (all commands below run this session, exits recorded)

## Commands + exits

| Command (cwd crates/cgap-mcp) | Exit | Result |
|---|---|---|
| cargo build | 0 | lib + bin cgap-mcpd |
| cargo test | 0 | 5 passed, 0 failed (courts.rs 4 + zero_alloc_court 1) |
| daemon stdio demo (initialize + tools/call echo) | 0 | real JSON-RPC lines above |
| sandbox mutation court (mutated lib.rs) | test FAILED (flip) | round_trip_court panicked: BadMagic 0x00000000 via -32603 |
| sandbox restore + cargo clean -p cgap-mcp; cargo test | 0 | 5/5 pass restored |

## Courts (verdicts)

1. **Round-trip court** ALIVE: JSON -> wire frame -> JSON field equality
   through handle_line on a real WireRing file; 4 message cases (plain,
   unicode, empty, 1000-char); on-disk slot ordinal cross-check. Also
   byte-cross-checks against the wire-shm frame format via slot bytes.
2. **Protocol court over REAL stdio** ALIVE: spawns cgap-mcpd via
   std::process::Command (CARGO_BIN_EXE), initialize shape
   (protocolVersion 2024-11-05, capabilities, serverInfo 26.11.3),
   tools/list {tools:[echo]}, tools/call missing name -> -32602, unknown
   method -> -32601, parse error -> -32700 with null id, echo round trip
   through the live daemon, clean exit.
3. **Zero-alloc window court** ALIVE: CountingAlloc global allocator;
   ZERO allocations inside the encode_frame_into window
   (encode + crc verify); anti-vacuity control (allocating Vec) flips the
   counter nonzero in the same test.
4. **Anti-vacuity mutation** FLIPPED in /tmp/cgap-mcp-mut sandbox (never
   the shared tree): read_frame skips correct ordinal
   (`ordinal.wrapping_add(1)`), round_trip_court FAILED
   ("wire error: bad magic: 0x00000000" via -32603). Restore sha256-exact
   (0e27b15f... == 0e27b15f...), verified. Shared tree untouched.
5. **Persistence court** ALIVE: stamp, drop handle, reopen segment file,
   read_frame(0) == payload.
6. **Internal-error court** ALIVE: 5000-char echo -> -32603 with typed
   data kind=FrameTooLarge.

## Test counts (re-read at use time)

cargo test (canonical): 5 passed, 0 failed: courts.rs 4
(round_trip_court, protocol_court_real_stdio, persistence_court,
internal_error_typed_wire_data) + zero_alloc_court 1.

## Divergences from the original contract

- **Receipt location**: contract said docs/ receipt; receipt placed at
  crates/cgap-mcp/RECEIPT.md inside the crate (cgap-cloud RECEIPT.md
  packaging-exclusion precedent), naming this divergence.
- **Zero-alloc window subject**: wire-shm's public stamp_frame/read_frame
  allocate internally (Vec bookkeeping/payload buffers) and cgap-mcp owns
  neither file, so ZERO allocs across those calls is not reachable without
  duplicating ring logic. The court instead measures cgap-mcp's own
  zero-alloc stack encoder `encode_frame_into`, byte-identity-checked
  against the wire-shm frame format by the round-trip court's on-disk slot
  cross-check (drift guard). Divergence recorded here, not silently.
- **Stale-mtime witness**: after sha256-exact sandbox restore, the
  sandbox run still executed the MUTATED binary (round_trip + protocol
  courts FAILED) until `cargo clean -p cgap-mcp`; mtime bump alone did not
  invalidate cargo's fingerprint in this layout. Fresh witness for the
  CLAUDE.md mtime-bump restore law.
- **Lane error, disclosed**: a stray `rm` removed the restored sandbox
  lib.rs mid-verification; immediately restored byte-exact from the
  canonical file and re-verified green (5/5) after clean rebuild.

## Fence law (fences my diff could break)

- Root workspace: cgap-mcp is a standalone [workspace]; NOT a root member;
  no root Cargo.toml edits, so no root compile surface touched. Root build
  not rerun (no root surface changed; standalone crate).
- wire-shm shared tree: read-only consumption; sandbox copy mutated, never
  the shared tree; sha256 restore verified.
- Boundary/zero-string gates: crate contains literal strings (JSON keys);
  those gates scope the campaign tree, not this standalone crate; not run
  (out of scope for a standalone crate lane).

## Cleanup

Lane build roots /tmp/cgap-mcp-target and /tmp/cgap-mcp-mut-target plus
sandbox /tmp/cgap-mcp-mut deleted at lane end (lease law).
