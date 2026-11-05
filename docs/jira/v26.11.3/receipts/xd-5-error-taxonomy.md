# XD-5 Receipt: AOQ Error Taxonomy Documentation Split (XC-5r rec #4, doc half)

Lane: XC-5r documentation half. Date: 2026-11-04.

## Subject

- `docs/aoq-header-offsets.md` (edited)
- `crates/cgap-aoq/src/error.rs` (doc-comments ONLY)

## Diff

1. `docs/aoq-header-offsets.md`: split the FROZEN band table into two
   sections — **Success/lifecycle statuses (FM-AOQ-001..003)** (OK/PENDING/
   BUSY are statuses, not refusals) and **Refusals (FM-AOQ-004..008)**.
   Added a "Seams-placeholder kernel refusals (XC-5r census)" subsection
   annotating `KemKernelNotAdmitted`/`AeadKernelNotAdmitted` (class codes
   0x00_0A/0x00_0B) as seams-placeholder with no attack scenario; AB-8
   owns population.
2. `crates/cgap-aoq/src/error.rs`: extended doc-comments on
   `KemKernelNotAdmitted` and `AeadKernelNotAdmitted` with the same
   seams-placeholder / no-attack-scenario annotation. No behavior change,
   no variant renames, no signature edits.

## Verification

- `cargo check -p cgap-aoq` → Finished dev profile in 0.40s, **exit 0**
  (doc-only proof: no behavior change).

## Scope boundaries honored

- `HeaderTooLong` NOT renamed/retargeted — owned by a follow-up lane.
Status: UNKNOWN
  than the fixed 64-byte envelope" vs the name's "too long" remains an
  open measurement item for the follow-up lane; this lane did not touch it).
- Doc-only diff; no error.rs match arms, Display strings, or class codes changed.

## Standing

ALIVE for the doc edits + check receipt above. Attack-scenario population
of the two seams-placeholder variants remains UNKNOWN (AB-8).
