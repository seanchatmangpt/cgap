# DOD-03 — FFI/C-ABI + header parity seal

Identity: /Users/sac/cgap @ main, branch main, session 2026-11-02.
Owned file: `docs/jira/v26.11.2/dod/dod-03-abi-seal.md` (this receipt). All
other surfaces read-only. No git write commands. Build leases used:
`/tmp/xc01-target` (gate), `/tmp/dod03-target` (tests) — both deleted at
lane end (LEASE_CLEANED, exit 0).

## Item 1 — ffi_surface_gate --build (fresh dylib)

`python3 scripts/run_checked.py --out /tmp/dod03-gate.log -- python3
scripts/ffi_surface_gate.py --build` → **exit 0**:
`[GATE] PASS: 12 exported cgap_* symbols == 12 header prototypes (1:1)`,
fresh dylib `/tmp/xc01-target/release/libcgap_ffi.dylib`.

## Item 2 — nm -gU census (zero internal leakage)

`nm -gU` on the fresh dylib → exactly 12 exported `T` symbols (all `cgap_*`,
macOS underscore stripped): admit_frame, array_addr, batch_rows,
commit_parquet, digest_of, eval_bn3, i32_batch, p28_invariant_check,
schema_addr, stats_snapshot, verify_p28, version. **Zero non-cgap_ internal
symbols leaked** — nothing beyond the header's 12. FINDING: none.

Note: `cgap_xip_start` is `#[no_mangle]` in src
(`xip_tables.rs:481`, `sovereign.rs:3365`; `sovereign.rs:3024` is inside a
string fixture, not code) but is cfg-gated to
`all(target_os="none", target_arch="riscv32")` (`lib.rs:341-346` module
gate) and is ABSENT from the host cdylib (nm confirms) — not a leakage
finding; it is a documented XIP-only export with no header prototype,
recorded as an observation, not a gate violation (the gate's subject is the
host cdylib by construction).

## Item 3 — #[no_mangle] vs header 1:1 cross-census

Grep census of `#[no_mangle]` across `crates/cgap-ffi/src/` +
`crates/cgap-p28/src/`: 10 code sites (2 others are comments/string
fixtures). Mapping to the 12 exports: lib.rs 6 (i32_batch, schema_addr,
array_addr, batch_rows, version, digest_of), stats.rs 1 (stats_snapshot),
sovereign.rs 4 host-gated (admit_frame, eval_bn3, commit_parquet,
verify_p28), trap.rs 1 (p28_invariant_check) = 12; plus 2 XIP-gated
`cgap_xip_start` definitions (xip_tables.rs:481, sovereign.rs:3365) absent
from host. Every export maps 1:1 to exactly one header prototype
(include/cgap.h); no prototype without an export, no export without a
prototype.

## Item 4 — repr(C) layout seals

- **CgasStatsBlock**: stats.rs (`#[repr(C)]`, src/stats.rs:69-114) field
  order/types match include/cgap.h:223-240 byte-for-byte (explicit `_pad0`
  at 60, `_pad1` at 116); `CRC_AT = 112` (stats.rs:33); FNV-1a-32 with
  offset basis `0x811C_9DC5`, prime `0x0100_0193` over bytes 0..112
  (stats.rs:145-155) — the header's "FNV-1a-32 over first 112 bytes" seal
  comment matches. Size 120 / align 8 witnessed at runtime by the C client
  (Item 5) and by `tests/stats_block_court.rs` (5/5 pass, Item 6).
- **P28LicenseAttestation (80 B)**: attestation.rs pins `ATTESTATION_LEN =
  80` (line 61) with compile-time `size_of == 80`, `align_of == 8`, and
  offset pins 0/4/6/8/16/48 (attestation.rs:88-101); parse is fail-closed
  on non-80-byte inputs (attestation.rs:148).

## Item 5 — ADV-02 C-client ABI court re-run (RETAINED PROBE — RUN, NOT STATIC)

`/tmp/adv02/client.c` present → re-compiled and linked against the FRESH
dylib: `cc client.c -o client_run /tmp/xc01-target/release/libcgap_ffi.dylib
-Wl,-rpath,...` → run → **exit 0, "ADV02-C-CLIENT: IMMUNE (0 mismatches)"**
— all 18 offsets, sizeof 120/align 8, live snapshot magic 0x43535442,
layout_version 1, C-side FNV recompute 0x6e3a2c48 == block_crc (seal holds
across the ABI from a foreign C consumer), null-out typed refusal fired.
version string: 26.11.2.

## Item 6 — cgap-ffi test count re-witness

Fresh lane target `/tmp/dod03-target`, build + test:

- lib unittests: **51 passed / 1 failed** (52 total).
- Integration targets (each `--test <t>` exit 0): alloc_courts 4/4,
  arrow_offset_pins 4/4, c_header_abi_court 4/4, fa9_commit_gating 7/7,
  lever_courts 2/2, stats_block_court 5/5, zero_copy 5/5 = **31/31**.
- **TOTAL: 82 passed / 1 failed.**

The one failure, `wave9_a07_zero_string_gate_over_all_built_crates`, is the
KNOWN foreign/environmental refusal (FIX-07 W5 precedent): the court
delegates to `scripts/zero_string_gate.py`, which structurally REFUSES
(exit 2, "execution-crate-no-artifacts ... zero scanned artifacts at
profile=debug") because a lane-leased CARGO_TARGET_DIR contains only
cgap-ffi artifacts. Reproduced against the SHARED target dir too
(`CARGO_TARGET_DIR=/Users/sac/cgap/target cargo test ... wave9_a07` →
FAILED, exit nonzero) — same refusal shape. Pre-existing, foreign to this
lane's read-only audit; owning A07 lane to chase (unchanged from FIX-07).

**DIVERGENCE (recorded):** dispatch said "80/0"; disk truth this session is
**82/1** (the 1 is the environmental wave9_a07 refusal above; lib count
51+31=82, not 80). Not rounded; recorded as measured.

## Standing

ALIVE for items 1-5 (command+exit this session against the exact fresh
subject). Item 6: PARTIAL — 82 pass / 1 foreign-environmental fail, both
legs witnessed.

## Fences (must-not-regress, run this session)

- FIX-07/XC-01 gate (`scripts/ffi_surface_gate.py --build`) — exit 0.
- ADV-02 foreign C-client court — exit 0 (IMMUNE).
- `c_header_abi_court` 4/4, `stats_block_court` 5/5 — exit 0 each.
- Header C11 static_assert pins untouched (header read-only this lane); C
  client compile doubles as the header-compile fence.
- Receipt law: this receipt is the only file written.

## Replay

```
python3 scripts/run_checked.py --out /tmp/g.log -- python3 scripts/ffi_surface_gate.py --build
cc /tmp/adv02/client.c -o /tmp/adv02/client_run <fresh dylib> -Wl,-rpath,<dir> && /tmp/adv02/client_run
CARGO_TARGET_DIR=/tmp/<lease> cargo test -p cgap-ffi --manifest-path crates/cgap-ffi/Cargo.toml
```

## Divergences / disclosures

- "80/0" dispatch figure vs measured 82/1 (above).
- `cgap_xip_start` has no header prototype (XIP-only export, not in host
  cdylib) — observation, not a violation; flagged for a future header note
  if the XIP surface is ever frozen at header level.
- wave9_a07: second run against shared target matched and failed 1/2 with
  the filter (`wave9_a07` filter also caught an adjacent court); primary
  census figures are from the unfiltered per-target runs.
- No SLA unblocks applied to other lanes' files; no cross-lane disclosures.
