# Charter §7 Gate Run — T-3 / T-5 (Lane CE-10)

- Date: 2026-11-02. Lane: CE-10 over `/Users/sac/cgap` (branch main).
- Contract: writes only to this file; no git commands; cargo from repo
  root; /tmp build lease `/tmp/ce10-target` (deleted at lane end).
- Standing: **ALIVE** — every command below ran this session with the
  exit shown. No divergences from the dispatch.

## T-3 — embedded gates (verify_cscb_sram.py + riscv32imc legs)

| leg | command | exit | result |
|---|---|---|---|
| T-3a baseline | `CARGO_TARGET_DIR=/tmp/ce10-target python3 scripts/verify_cscb_sram.py` | 0 | measured SRAM 36,048 B ≤ 131,072 B; headroom 95,024 B; stack estimate static max leaf frame 2,048 B (UNVERIFIED as worst-case chain, report-only); empirical delta vs 39,184 B baseline −3,136 B (report-only) |
| T-3b bn3 subject | same + `--subject bn3` | 1 | typed refusal `BN3_ENCLAVE_REFUSED`: 6,520,984 B (NaMatches 2,162,696 + NaEngine 4,358,288) > 131,072 B, 49.75x ceiling — the expected loud refusal |
| T-3c slide arm | `cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features` | 101 | typed compile_error naming `implcell_critical_provided` / "no CAS-capable atomic" (ref_store.rs:232) — loud-until-bound holding |
| T-3d opt-in arm | `CARGO_TARGET_DIR=/tmp/ce10-target RUSTFLAGS='--cfg implcell_critical_provided' cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features` | 0 | green with 11 warnings |

Working riscv32imc command (record per dispatch): `RUSTFLAGS='--cfg
implcell_critical_provided' cargo build -p cgap-git --target
riscv32imc-unknown-none-elf --no-refuse...` — exact string:
`CARGO_TARGET_DIR=<lease> RUSTFLAGS='--cfg implcell_critical_provided' cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features`.

## T-5 — invariants (cgap-ffi courts + seed pins)

| leg | command | exit | result |
|---|---|---|---|
| T-5a | `cargo test -p cgap-ffi --test stats_block_court` | 0 | 5/5 passed (counters, health degradation, reset, C-ABI getter typed-null, global recorder) |
| T-5b | `cargo test -p cgap-ffi --test c_header_abi_court` | 0 | 4/4 passed (header inventory, rust twins layout, C compile leg, rust mirror leg — both reorder-flip legs green) |
| T-5c | `grep -n <seeds> crates/cgap-parquet/src/bitslice.rs` | 0 | pins intact: SplitMix64 multiplier `0xBF58_476D_1CE4_E5B9` at bitslice.rs:687,694 (and 728/759/808/868); corpus constant `0xA20C_0FFE_E5A7_E000` at bitslice.rs:688,699 |

## Fences

- Fences this diff could break: none — write surface is this receipt
  only. Gates were run fresh, read-only against the shared tree except
  the lease target dir.
- Anti-vacuity: T-3b is itself the flip witness for T-3a's gate (typed
  refusal on an over-ceiling subject). The script's own mutation
  courts (`--mutate`) were NOT in dispatch scope; not run here.
- Not run (out of scope): `--mutate=sram-bloat` / `bn3-pin` courts,
  `make gate0`, publish steps.

## Terminus (JSON)

{"lane":"CE-10","files_written":["docs/jira/v26.11.2/dod/charter-t35-gate-run.md"],"commands":["verify_cscb_sram.py exit 0 (36048 B)","--subject bn3 exit 1 (typed refusal)","riscv32imc build exit 101 (typed refusal)","riscv32imc build + RUSTFLAGS cfg exit 0","stats_block_court 5/5 exit 0","c_header_abi_court 4/4 exit 0","seed grep exit 0"],"courts_run":["verify_cscb_sram baseline","bn3 subject refusal","stats_block_court","c_header_abi_court"],"standing":"ALIVE","divergences":[]}
