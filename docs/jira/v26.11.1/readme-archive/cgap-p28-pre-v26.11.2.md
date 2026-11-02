# cgap-p28

**Status: NOT-PUBLISHED** — verified against crates.io this session
(no crate record). Workspace-local moonshot crate.

## What it is

Lane A17: the 80-byte `P28LicenseAttestation` (`#[repr(C, align(8))]`,
`src/attestation.rs`) — the sovereign license record embedded in every
derivative artifact. Fixed offsets 0/4/6/8/16/48, `ATTESTATION_LEN = 80`,
align 8, magic `P28S`. Lineage bitfield at offset 6: bits 0-2 are the
lineage (root / receipt / transpile, exactly one MUST be set), bits 3-15
reserved-MUST-be-zero. License URN is a frozen byte-pin:
`urn:avec:license:p28-sil:v2.1.0-sovereign`.

Also home to the ASC-450 trapdoor checks (`src/asc450.rs`) and the trap
surface (`src/trap.rs`).

## Test receipt (run this session)

```
cargo test -p cgap-p28   # exit 0
  36 passed (lib) + 6 passed (p28_invariant_courts) = 42/0
```

Receipt of record: `docs/jira/v26.11.1/waves/a17-finish-receipt.md`
(baseline pre-write 36+0, post-write 42/0; mutation gate — poisoning the
attestation flips the court to exit 101 with the named verdict).

## The 640-bit sweep and the 11-survivor law

`tests/p28_invariant_courts.rs` sweeps all 640 bits of the record
(bit index k asserted within `ATTESTATION_LEN * 8`), executes the §5
ordered validation, and asserts the survivor law (§7.1, D-A17-1
re-derived): with the ceiling pinned at MAX the survivor set stays
`<= MAX` (§5.8) and the hand-count pin `popcount(5_000_000_000) = 11`
predicts the survivor count. The anti-vacuity mutation is attestation
poisoning — it must flip the court to exit 101.
