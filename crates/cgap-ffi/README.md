# cgap-ffi

Rewritten for the v26.11.2 reality wave. The v26.11.1 sovereign C-ABI
README is archived byte-exact at
`docs/jira/v26.11.1/readme-archive/cgap-ffi-pre-v26.11.2.md` (sha256
`236b6d2ff880e1aadcde59d6355d0e73f4e10f01719bb4ace080b189d6b339e0`);
the pre-moonshot Arrow C Data Interface README remains archived at
`docs/jira/v26.11.1/readme-archive/cgap-ffi-pre-v26.11.1.md`.

**Status: NOT-PUBLISHED** — as last witnessed (v26.11.1 wave; no crate
record). In-repo cdylib/staticlib membrane; consumers load the built
artifact directly, there is no registry artifact.

- Crate version: **26.11.2** (`version.workspace = true`; workspace
  26.11.2 — reality receipt drift-check MATCH).

## The four sovereign entrypoints

Lane A19 froze the sovereign C-ABI (`src/sovereign.rs`):

- `cgap_admit_frame` — A04 CgapFrameHeader (128 B) admission: parse
  produces a view, never a copy.
- `cgap_eval_bn3` — binary-N3 evaluation over the A11 BN3 layout.
- `cgap_commit_parquet` — Parquet persistence commit.
- `cgap_verify_p28` — P28 spoliation evidence verification: parse + the
  FULL A17 §5 ordered validation of the embedded 80-byte
  `P28LicenseAttestation`, against the pinned LICENSE-P28-SIL.md
  instrument header span (bytes pinned in `PINNED_INSTRUMENT_HEADER`).
  Predicate failure returns `CGAP_ERR_P28_SPOLIATION` (`0x5032_38FF`).

Version gate: `CGAP_ABI_VERSION 1` must match before any sovereign
dispatches. Neither struct (CgapFrameHeader, P28LicenseAttestation)
crosses the ABI — admit and verify return views/verdicts only.

## XIP / RV32 leg

`xip.ld` is the execute-in-place linker script (37 lines); the RISC-V
variant (ESP32-C3 rv32imc) swaps flash origin/len per SoC. `src/sovereign.rs`
carries `cgap_xip_start` legs per target (`wfi` loop, `#[no_mangle]`).

## include/cgap.h — location note

`include/cgap.h` is the frozen C-side twin of the §6.1 header court.
It lives at the REPO ROOT (`include/cgap.h`, 8,740 B), not inside
`crates/cgap-ffi/` — an earlier revision of this README wrongly reported
it missing (that lane had looked only inside the crate dir). The file is
tracked, on disk, and byte-stable.

## Test receipt (v26.11.2 reality wave)

```
cargo test -p cgap-ffi   # exit 0
  45 (unit) + 4 (alloc_courts) + 2 (lever_courts) + 5 (zero_copy) = 56 passed; 0 failed
```

C-ABI census: `nm -gU` over the release cdylib — 11 exported T symbols.
Substance: 5,961 LOC total src. Host `cargo check -p cgap-ffi` and
`cargo check -p cgap-ffi --target riscv32imc-unknown-none-elf` exits are
in the reality receipt
(`docs/jira/v26.11.2/waves/crate-reality/cgap-ffi.md`); A19 historical
receipt `docs/jira/v26.11.1/waves/a19-finish-receipt.md`.
