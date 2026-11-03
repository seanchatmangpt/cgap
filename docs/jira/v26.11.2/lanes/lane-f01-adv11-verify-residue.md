# Lane F-01 — ADV-11 repair verification + attestation/doc residue

- Repo /Users/sac/cgap (branch main). Owned files: `crates/cgap-p28/src/attestation.rs`
  (:1446 comment only), `crates/cgap-git/src/ref_store.rs` (:30-39 module doc
  prose only), `docs/jira/v26.11.2/dflss/08-ax-pfmea.md` (PY-5 row),
  `docs/jira/v26.11.2/redteam/red-11-riscv-reentrancy-corruption.md`
  (append-only ERRATA). No git commands; coordinator owns transitions.
- Read first: `lanes/lane-r-adv11fix-implcell-slide.md` (ALIVE), corrosion
  adv-11 + adv-17. Standing: **ALIVE** — every command below ran this
  session with the exit shown.

## W1 Independent repair verification (all this session)

| check | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/f01-target cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features` (no opt-in) | **101** — typed refusal: "no CAS-capable atomic on this target... rebuild with `RUSTFLAGS=\"--cfg implcell_critical_provided\"`..." (/tmp/f01-rv32-noflag.log:4, compile_error at ref_store.rs:226-227) |
| same build + `RUSTFLAGS="--cfg implcell_critical_provided"` | **0** (Finished dev profile) |
| host `cargo build -p cgap-git` | **0** |
| `cargo test -p cgap-git --test ref_store_width_arms_court` | **0** — 8/8 passed |

Verdict: repair REPRODUCED-INDEPENDENTLY. Loud-until-bound contract holds
exactly as R-ADV11FIX receipted; no discrepancy found.

## W2 Residue fixes landed

1. `attestation.rs:1446`: stale arithmetic comment corrected 26 B/42 B ->
   **27 B/43 B** (MAC_DOMAIN = b"cgap-p28/attestation-mac/v1" = 27 B,
   :1383). The const assert `assert!(INPUT_LEN == 43)` (:1448) untouched —
   comment now matches the pin. ADV-17 Attack 2 doc-drift flag closed.
2. `ref_store.rs:30-45` module doc: replaced the stale "rv32imc selects
   impl32" prose with the post-slide truth — rv32imc selects implcell,
   LOUD-until-bound without `--cfg implcell_critical_provided`, firmware
   binds the critical section. Logic untouched (doc prose only).
3. PFMEA PY-5 row (`08-ax-pfmea.md:128`): status -> **LANDED-CODE-REPAIRED**;
   stale `riscv32imc=impl32` premise marked STALE/superseded; slide repair
   cited (lane-r-adv11fix); F-01's independent re-verification recorded;
   impl16/implcell execution-certification residual kept loud REGISTERED.
4. red-11 ERRATA appended (never-rewrite law; original §1-§5 bytes
   untouched): §1 impl32-on-rv32imc premise SUPERSEDED by adv-11 + slide
   repair + F-01 verification; §2/§3 marked as describing the PRE-slide
   arm.

## W3 Fences run

- `cargo test -p cgap-p28` -> exit **0** (5 suites, 50 tests, 0 failed).
- `cargo test -p cgap-git` -> exit **101**: 235 passed, 2 suite failures —
  both classified, neither session-introduced:
  - `octopus_merge_court::mutation_conflict_injection_flip`: PRE-EXISTING,
    proven foreign by R-ADV11FIX W4 via HEAD baseline (/tmp/radv11-headcheck,
    identical failure). Isolated rerun this session: **exit 0 (5/5)**.
  - `soc_full_chip_court` zero-alloc window (9 allocs, expected 0):
    transient parallel-suite artifact (foreign binaries allocating inside
    the Serial-mutex window). Isolated rerun: **exit 0 (3/3)**.
- Fences NOT run (outside lane): `make gate0`, rv32imc workspace builds
  (they now REQUIRE the opt-in flag or accept the loud refusal —
  disclosed loud-until-bound change, R-ADV11FIX W5).

## W4 Divergences

1. Dispatch said pfmea:128; the PY-5 row is at :128 — verified, no drift.
2. 100-char line law: pfmea table rows and red-11's pre-existing prose
   exceed it file-wide (sibling rows 757-1348 chars); my additions match
   the file-local style, no new violation introduced beyond the table-row
   convention.
3. Dispatch expected `cargo test -p cgap-git` green; actual 101 with the
   two classified failures above (isolated reruns 0/0). Recorded, not
   rounded up: full-suite parallel standing is PARTIAL, isolated standing
   ALIVE.

## W5 Replay

- Shared-tree diffs: the four owned-file edits only (comment/doc/row/
  errata — zero logic changes). No mutation of shared bytes; no /tmp
  restores needed (no sandbox court this lane — verification lane).
- Lease cleanup: /tmp/f01-target deleted at lane end via
  `python3 -c "import shutil; shutil.rmtree('/tmp/f01-target', True)"`.

## W6 JSON terminus

```json
{"lane":"F-01","files_written":["crates/cgap-p28/src/attestation.rs","crates/cgap-git/src/ref_store.rs","docs/jira/v26.11.2/dflss/08-ax-pfmea.md","docs/jira/v26.11.2/redteam/red-11-riscv-reentrancy-corruption.md","docs/jira/v26.11.2/lanes/lane-f01-adv11-verify-residue.md"],"commands":["cargo build -p cgap-git --target riscv32imc-unknown-none-elf --no-default-features -> exit 101 typed implcell_critical_provided refusal","same + RUSTFLAGS=--cfg implcell_critical_provided -> exit 0","cargo build -p cgap-git (host) -> exit 0","cargo test -p cgap-git --test ref_store_width_arms_court -> exit 0 (8/8)","cargo test -p cgap-p28 -> exit 0 (50 tests)","cargo test -p cgap-git -> exit 101 (235 passed; octopus flip + soc zero-alloc; both isolated rerun exit 0)","isolated reruns octopus_merge_court / soc_full_chip_court -> exit 0 / exit 0"],"courts_run":["ref_store_width_arms_court (8/8, incl. anti-vacuity flip legs w4/w5/w7 per R-ADV11FIX)"],"standing":"ALIVE","divergences":["cgap-git full suite: 2 transient/pre-existing parallel failures, isolated reruns green (octopus proven foreign by R-ADV11FIX W4; soc load-sensitive)","100-char law: pfmea/red-11 table-row style pre-exceeds; no new violation"]}
```
