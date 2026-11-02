# Authority pins

Five frozen layout constants, cited from canonical source files
(line numbers witnessed 2026-11-01).

## 128-byte CgapFrameHeader

- `pub const FRAME_HEADER_LEN: usize = 128` —
  `crates/cgap-core/src/frame.rs:41`.
- `#[repr(C, align(64))]`, size asserted 128 (frame.rs:383, :389).
- Offset table (A04 §6.1; field list frame.rs:172-219):
  attestation spans offset 48..128 (80 B, frame.rs:192, :210, :219).

## 80-byte P28LicenseAttestation

- `pub const ATTESTATION_LEN: usize = 80` — frame.rs:44.
- `#[repr(C, align(8))]`, size asserted 80 (frame.rs:385, :390).
- Field offsets (frame.rs:77-96):
  - 0, 4 B: `license_magic` = `*b"P28S"` (`LICENSE_MAGIC`, frame.rs:49)
  - 4, 2 B: `version` (u16 LE, currently 1)
  - 6, 2 B: `attestation_flags` (u16 LE bitfield, A17 §2.2)
  - 8, 8 B: `safe_harbor_ceiling_cents` (u64 LE)
  - 16, 32 B: `origin_authority_digest` (BLAKE3)
  - 48, 32 B: `upstream_receipt` (BLAKE3; zero only if Root)
- Flag constants (frame.rs:52-66): `LICENSE_FLAG_ROOT` 0x01,
  `LICENSE_FLAG_DERIVATIVE` 0x02, `LICENSE_FLAG_TRANSPILED` 0x04,
  lineage mask 0x07 (exactly one bit), reserved mask 0xFFF8 (must be 0).

## 24-byte BN3Statement

- `pub const BN3_STATEMENT_SIZE: usize = 24` —
  `crates/cgap-bn3/src/sigil.rs:81`.
- Three term words, exactly 24 bytes; size asserted
  (sigil.rs:428). `to_le_bytes`/`from_le_bytes` round-trip
  (sigil.rs:384, :405).

## SlotId derivation

- `pub struct SlotId(pub u64)` —
  `crates/cgap-bont/src/primitives.rs:61`.
- Derivation: `SlotId = u64::from_le_bytes(LexHash[0..8])` — low 64 bits
  of the LexHash taken little-endian (primitives.rs:57, :94).
- ABI freeze: 8 bytes, 8-byte aligned (primitives.rs:99, :115).

## Safe-harbor ceiling

- `pub const SAFE_HARBOR_CEILING_CENTS_MAX: u64 = 5_000_000_000` ($50M) —
  `crates/cgap-core/src/frame.rs:68` (A17 §2.1); field at attestation
  offset 8 MUST be <= ceiling (frame.rs:84, enforced in `validate()`,
  frame.rs:116).
