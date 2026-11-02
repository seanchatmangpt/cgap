# w5-linker-budgets receipt (lane D3, v26.11.2, 2026-11-02)

Deliverable 4.3 — 128 KiB budget enforcement at link time, fail-closed.

## Files written (lane-owned only)

- `crates/cgap-ffi/enclave_128k.ld` — NEW. MEMORY: `ENCLAVE_FLASH (rx)`
  128K @ 0x10000000, `ENCLAVE_SRAM (rw)` 64K @ 0x30000000 (SRAM origin
  not dispatched; D3 chose 0x30000000, documented divergence). Sections
  `.text`/`.rodata` (rodata also absorbs `.oabi`/`.bn3`) into flash,
  `.data`/`.bss` into SRAM with `AT>ENCLAVE_FLASH`. Fail-closed ASSERTs:
  `.text <= 131072`; backstop `(text+rodata) <= 131072`;
  `(data+bss) <= 65536`.
- `crates/cgap-ffi/xip.ld` — EDITED. Added per-tier budget ASSERTs after
  the existing .oabi/.bn3 presence ASSERTs (presence ASSERTs unchanged):
  gate tier <=16384 on `.oabi`; tier1 <=49152 on `.bn3`; tier2 <=49152 on
  `.t2`; tier3 <=12288 on `.t3`, the last two in latent
  `SIZEOF==0 || budget` form with real (empty) output-section anchors
  `.t2`/`.t3` added to SECTIONS so the ASSERTs always evaluate and
  activate the moment those input sections land — never a silent skip.
- `docs/jira/v26.11.2/waves/w5-linker-budgets.md` — this receipt.

## Commands + exits (this session, host rustc 1.97 default vs pinned nightly)

Toolchain gate: the workspace pins floating nightly
(rust-toolchain.toml); plain `cargo` in a /tmp fixture resolves the
1.97.0 default WITHOUT riscv32imc and fails E0463 (exit 101, observed).
All link legs below ran under `rustup run nightly-aarch64-apple-darwin`.

- `cargo test -p cgap-ffi --lib xip_ld -- --test-threads=1`
  (CARGO_TARGET_DIR=/tmp/d3-target) — exit 0; 2 passed, 0 failed
  (xip_ld_asserts_fire... + xip_ld_assert_mutant_section_absent...).
  Baseline proves the new budget ASSERTs do not disturb the green path.
- Full `cargo test -p cgap-ffi --lib xip -- --test-threads=1` — exit 0,
  7 passed / 0 failed (includes `xip_t1_riscv_elf_size_court`,
  `xip_t2_xtensa_toolchain_gate`, `r2_production_link_court` — the
  real-link courts stay green under the new budgets).
- Baseline fixture link vs xip.ld (nightly) — exit 0.
- MUTANT 1 (xip.ld, sandbox /tmp/d3-court): `.oabi` grown 96 -> 20000 B
  (>16K gate budget) — exit 101, stderr names
  `gate-tier .oabi budget 16384 exceeded`. FLIPPED on the exact message.
- Baseline fixture link vs enclave_128k.ld — exit 0.
- Mutant 40 KiB `.bn3` (rodata 40960 B < 128K budget) vs
  enclave_128k.ld — exit 0: correct budget-satisfied outcome, NOT a flip
  (recorded; the flip leg is the 200 KiB mutant below).
- MUTANT 2 (enclave_128k.ld, sandbox): `.bn3` grown to 200 KiB —
  exit 101; stderr names `cgap ENCLAVE 128K: text+rodata exceeds the
  131072-byte enclave flash budget` AND lld `section '.rodata' will not
  fit in region`. FLIPPED (two independent refusal mechanisms).
- Post-restore re-link vs enclave_128k.ld — exit 0 (sandbox restored).
- Shared-tree sha256 before/at end, byte-exact:
  - xip.ld `14049de3c5544c2a259ed6fc2f8ea2d5636af73fb3871059316d334c4c5f8bd7`
  - enclave_128k.ld `6d9c786430d5a3e9d295d1d4586a64848ccf1f0894de4f59c08015f81e1aa387`
  (sha256sum after each mutation round; both scripts were never mutated
  in the shared tree — mutations were applied to the /tmp fixture only).

## Standing

ALIVE — every gate/flip leg above is a command+exit run this session
against the exact on-disk scripts (sha256s pinned above). Build lease
`/tmp/d3-target` deleted at lane end.

## Divergences

1. Dispatch named four tier budgets without section anchors; the tier ->
   section mapping (gate=.oabi 16K, tier1=.bn3 48K, tier2=.t2 48K latent,
   tier3=.t3 12K latent) is the D3 definition, recorded in xip.ld
   comments.
2. Dispatch's "append a synthetic .space 30000 / 40 KB static array into
   the probe link": a 40 KiB array does not exceed the 128 KiB budget, so
   that literal mutant cannot flip the 128K gate (recorded exit 0 leg);
   the flip witness is the 200 KiB oversize mutant (exit 101 on the named
   ASSERT text), plus the 20000 B .oabi mutant flipping the xip.ld
   gate-tier ASSERT.
3. ENCLAVE_SRAM origin 0x30000000 not dispatched; chosen by D3.
4. The sovereign.rs xip courts were NOT edited (not lane-owned); their
   fixtures stay within the new budgets so the existing court family
   remains green (7/7 this session).
