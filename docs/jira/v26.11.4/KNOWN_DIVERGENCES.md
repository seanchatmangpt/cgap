# KNOWN DIVERGENCES — v26.11.4

Probe-receipt register of SDK/wire divergences. A divergence closes only via
its close condition; a court that documents a row is not a fix.

## KD-1 — version-refusal status-word collision (OE-4)

- **Surface**: `crates/cgap-ffi/src/aoq_session.rs` `status_of` and
  `crates/cgap-ffi/src/bin/aoq_sidecar.rs` `session_status` (twin mapping sites,
  same pinned collisions); wire-visible via the sidecar's status word.
- **Divergence**: `SessionError::Wire(AoqError::UnsupportedVersion)` projects to
  `DIGEST_MISMATCH 0x4347A005`, the same word as a genuine digest/authenticity
  fault. A consumer cannot distinguish "tampered" from "wrong version" on the
  wire.
- **Why not fixed here**: the frozen status band 0x4347A001..A008 is fully
  allocated (8/8 words, `AOQ_STATUS_COUNT = 8`, asserted by pack gates;
  `crates/cgap-ffi/src/aoq/status_maps.rs` is a generated ggen projection —
  DO NOT EDIT). No version-appropriate word exists in the band, and remapping
  `UnsupportedVersion` onto EPOCH_MISMATCH/KEY_UNKNOWN would project a false
  class. Refusal fires at the correct parse stage (codec order
  magic→version→epoch→digest, witnessed by the L2/L3 flip-ladder legs); only
  the emitted word collides.
- **Evidence**: OE-4 receipt `docs/jira/v26.11.4/receipts/lane4-flip-ladder-court.md`
  (real release sidecar, L2 leg: version skew 3→2 with honest re-tag →
  DIGEST_MISMATCH; L3 ordering leg proves version fires before digest).
- **Close condition**: band extension at the next wire-version bump — a new
  `aoq:StatusConstant` (e.g. `UNSUPPORTED_VERSION 0x4347A009`) in
  `packs/aoq-pack/ontology.ttl`, regenerated `status_maps.rs`, then remap the
  `UnsupportedVersion` arm in BOTH mapping sites and re-pin the L2/L3 court
  legs to the new word.
- **Registered**: 2026-11-04 (FIX-or-REGISTER lane, option b).
