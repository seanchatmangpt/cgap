# Why Frozen Structural Definitions Govern

Why five structural definitions are byte-pinned and what happens when
a runtime copy drifts.

## Single canonical definition

Each structural constant has exactly one canonical home on disk. The
80-byte P28 attestation is defined once, in
`crates/cgap-core/src/frame.rs` (`P28LicenseAttestation`, layout
frozen at Wave 1 per A17 §2.1). The pin-final wave receipt
(`docs/jira/v26.11.1/waves/r14-pin-final.md`, row 2) records the
sweep result: 11 hits, all in `cgap-parquet/src/footer.rs`, with
`footer.rs:84` a *by-value pin declared a registered divergence*,
self-declared framing-only, citing A17; the canonical value (=80)
lives at `cgap-p28/src/attestation.rs:60`.

## Runtime-tied twins

A definition alone can drift from the memory layout the compiler
actually picks. cgap ties them with compile-time twins — const asserts
that fail the build, not a doc note:

- `frame.rs:390`:
  `const _: () = assert!(ATTESTATION_LEN == core::mem::size_of::<P28LicenseAttestation>());`
- the A04 finish receipt (`waves/a04-finish-receipt.md:57-58`) records
  the pair `FRAME_HEADER_LEN == size_of::<CgapFrameHeader>()` and "the
  ATTESTATION_LEN twin" as const pins tied to the wire constants.

The twin exists because a doc comment asserting "exactly 80 bytes" is
prose; the const assert is a gate that cannot be edited around without
breaking the build.

## The registered-divergence pattern

When a runtime copy must differ (or merely coexist), it is not deleted
and not silently tolerated — it is *registered*: the divergent site
self-declares its scope ("framing-only, cites A17") and lands in a
wave receipt row. This converts an invisible drift risk into a
visible, auditable cell. Divergences that cannot be registered this
way are refused. The pattern is the same one the coverage discipline
uses everywhere: every cell is pass / registered-divergence /
loudly-MISSING — never a silent skip.

## Why freeze at all

Frozen byte-pins (the license URN, the SHACL corpus IRIs, the frame
and attestation layouts) are the substrate other lanes compute
against. Repinning is a campaign-level transition with receipts, not a
convenient refactor; that is the price of a wire format that many
independent lanes can cite without re-deriving it.
