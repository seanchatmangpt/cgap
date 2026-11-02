# A05 Finish Receipt

- **Lane**: A05 (`crates/cgap-storage` — `atomic_commit.rs`, power-cut WAL),
  lane-finisher pass, v26.11.1, repo `/Users/sac/cgap` (one canonical
  checkout)
- **Date**: 2026-11-01
- **Dispatch**: a wave landed in 9be9109d/62215326; a prior wave died
  mid-flight — assess first. Witness the host gate and >=1 adversarial
  sandbox flip this session; close the missing-receipt gap (no `a05-*`
  receipt existed under `docs/jira/v26.11.1/waves/` before this file).
- **Verdict**: **ALIVE** — host gate exit 0 (2 unit + 19 court tests, 0
  failed) and the sandbox flip battery (2 source mutations, both flip with
  exit 101, byte-exact restores, green re-run) all run this session. Repo
  lease untouched by mutation runs: every shim executed in
  `/tmp/a05-fin-sandbox` against copies; `src/atomic_commit.rs` sha256
  `19b21f60e77d3020…98680440` identical in canonical tree and sandbox
  baseline, before and after the battery.

## Assessment (what was already landed; nothing rewritten)

`crates/cgap-storage` on `main` carries the full lane in three prior-wave
slices: the `atomic_commit` module (1035 lines: `RECORD_MAGIC` 0x4A414353
framing, 48B header + 48B descriptor + 16B trailer, CRC-32 IEEE +
96-bit BLAKE3 split header/trailer, compile-time layout/offset static
asserts, `FlashMedium` NOR model with AND-semantics + progressive erase,
`Medium`/`JournalWriter` traits, `Txn` begin/write/commit with readback
self-check, `recover`/`recover_mode` with F3 dedup + typed
`JournalAmbiguous`/`SeqRegression` refusals, typed `MutationMode` M1-M4,
divergences D1-D3 recorded in-code) and five court files (925 lines). The
assessed "died mid-flight" wave left NO truncated or corrupt file: all six
read coherent end-to-end, and every court below passes as found. The
finisher's delta is evidence, not code: the witnessed host gate, the
sandbox flip battery, and this receipt.

## Commands (this session, exact)

```text
CARGO_TARGET_DIR=/tmp/cgap-lane-A05-target cargo test --locked -p cgap-storage
  -> exit 0: 2 unit + 3 crc-identity + 5 faults + 3 flash + 6 mutations
     + 2 sweep = 21 tests, 0 failed
sandbox /tmp/a05-fin-sandbox (copies of cgap-storage + cgap-blake3 +
  cgap-git + minimal workspace manifest; baseline sha256 of
  src/atomic_commit.rs 19b21f60e77d3020…98680440, identical to canonical):
  baseline full `cargo test` -> exit 0
  SHIM-COURT M-FENCE (delete the §9 `artifacts_fenced` guard in
    `write_record_cut`) -> `cargo test --test court_sweep` exit 101:
    commit_ack_means_durable FAILED at court_sweep.rs:180
    (Ok(()) vs Err(ArtifactsNotFenced))
  restore (cp from canonical) -> sha256 byte-exact
  SHIM-COURT M2-SRC (strict top-selection flipped to lowest-seq) ->
    `cargo test --test court_sweep --test court_mutations` exit 101:
    sweep_per_offset_rollback FAILED at court_sweep.rs:104 (wrong root),
    mutation_m2_lowest_seq_flips FAILED at court_mutations.rs:125
    ("VACUOUS COURT: M2 flipped nowhere"), mutation_m3_…_flips also fires
  restore -> sha256 byte-exact -> all five court binaries re-run exit 0
shasum -a 256 src/atomic_commit.rs  # pre == post == 19b21f60…
git status --porcelain  # read-only; only other lanes' files + this receipt
```

## Courts run (name -> pass / flip evidence, this session)

- `sweep_per_offset_rollback` (doc §7 t1: C0-C4, I1-I3): 113 cut offsets x
  2 scenarios x 2 backends, every case enumerated, green; per-case I3
  determinism re-run on a rebuilt medium. Flip: M-FENCE and M2-SRC both
  fire it (below).
- `commit_ack_means_durable` (§7 t5 / C4 + §9 fence): full `Txn` path on
  the real-file backend, green. Flip: M-FENCE fires it exactly —
  `Ok(())` vs `Err(ArtifactsNotFenced)` at court_sweep.rs:180.
- `aging_corruption_flip` (§7 t3 / F1): 112 record offsets x mask 0x01 x
  2 backends — base root stands, Degraded reported, I3 checked. Flip: M1
  fires on the header-CRC-only surface (court_mutations).
- `sweep_both_slots` (§7 t2 / F2 + divergence D2): 113 offsets — torn k
  gives base+Degraded, k=N honors the C4 commit. Flip: M2-SRC also fires
  the sweep court.
- `both_slots_damaged_reports_degraded` (F2): typed Degraded, both slots
  `Damaged`, green.
- `ambiguous_refuses` (§7 t4 / F3): conflicting same-seq -> typed
  `JournalAmbiguous`; identical replay -> deterministic dedup, green.
- `seq_regression_refused` (§6 A03): dag 8/9 vs base 9 -> typed
  `SeqRegression`; dag-10 boundary resolves. Flip: M4 admits the
  regressing root.
- `flash_erase_semantics` (§7 t6): erased 0xFF = Clean/Empty; an all-zero
  slot = Damaged/Degraded (never zero-fill confusion) — the all-zero
  assertion is itself the flip detector.
- `overwrite_requires_erase`: NOR AND-semantics — `NeedsErase` typed, zero
  bytes moved, prior root intact; commit lands after the erase.
- `cut_during_erase_sweep` (§9): 2x113 erase offsets on a standing seq-2
  root (Clean); loss window pinned — partial erase Degraded, completed
  erase Clean/Empty.
- M1-M4 (§7.1, `court_mutations`): `strict_is_the_production_default` and
  the M1 write-sweep backstop pinned. Flips: M1 on the header-CRC-only
  surface (offsets 8..15, 28..31), M2 at k=N only, M3 on k in 100..111,
  M4 on the regression case — all four green-strict vs flipped-mutant.
- `court_crc_identity` (wave-5 repair): 6 zlib KATs + 262-row differential
  vs `cgap_git::pack_write::crc32` + streaming decomposition, green this
  session. Its M-C1/M-C2/M-C3 flips were witnessed in the wave-6 sandbox
  session (module doc header) and were NOT re-run this session.

## Divergences

1. **Doc §3.3 HEADER_CRC range contradiction (D1, recorded in-code)**: the
   spec's literal "bytes 0..24 and 32..44" leaves BLAKE3 digest bytes
   44..48 uncovered and contradicts its own coverage summary; implemented
   as h[0..24] ++ zeros(24..32) ++ h[32..48]. Doc yields.
2. **Doc §5 F2 k-range (D2, recorded in-code)**: "cut during commit k plus
   earlier damage in the other slot" is unbounded; at k=N the C4 law wins
   (a fully valid commit is honored). `sweep_both_slots` pins both sides.
3. **Doc §7.1 mutation protocol (D3, recorded in-code)**: M1-M4 are typed
   in-module `MutationMode` variants rather than scratch-copy source edits
   (the doc's protocol would edit outside this lane's lease). The
   scratch-copy protocol is nonetheless executed THIS session on real
   source in the sandbox (M-FENCE, M2-SRC above), closing the gap the
   typed-mode design left.
4. **Wear rotation open (lane doc §9)**: slot-generation rotation across
   erase blocks is an explicitly open implementer parameter; the current
   fixed two-slot journal pins the loss-window semantics instead
   (`cut_during_erase_sweep`). Not a defect; open scope.
5. **Receipt gap closed**: no `a05-*` wave receipt existed on disk; this
   file is that receipt.
6. **CRC-identity flip battery scope**: the court's M-C1/M-C2/M-C3
   mutations were witnessed in the wave-6 sandbox session (module doc
   header) and NOT re-run this session; their pass evidence (KATs +
   262-row differential) is re-witnessed. Standing for the flip evidence
   itself: as-of wave-6 receipt, not this session.
7. **Concurrent lanes**: during this run, other lanes held in-flight edits
   in `crates/cgap-bont`, `crates/cgap-core`, `scripts/zero_string_gate.py`
   and landed `a16-finish-receipt.md`; no overlap with this lease. The
   host gate ran `--locked` (root `Cargo.lock` unmodified).

## Standing

- Journal framing, descriptor codec, flash model, `Txn` commit path,
  `recover`: **ALIVE** (exit-0 host gate + green sandbox baseline, this
  session).
- Crash-matrix courts C0-C4, F1-F3, A03 refusal, erase semantics: **ALIVE**
  (green this session) with witnessed flips (M-FENCE, M2-SRC, M1-M4 typed
  courts) — zero vacuous courts in the swept space.
- CRC cross-lane identity: **ALIVE** as pass evidence this session; its
  named mutations' flips are wave-6-session evidence (divergence 6).
- ESP32 flash-partition backend and slot-wear rotation: **UNSUPPORTED**
  here by design (open scope, lane doc §9); the in-process flash model is
  the courted substitute.
- Perf numbers: none claimed, none measured.

## See Also

- `docs/jira/v26.11.1/lanes/A05-powercut-wal.md` (normative lane doc)
- `crates/cgap-storage/src/atomic_commit.rs` (module; divergences D1-D3
  recorded at the divergence section, lines 324-349)
- `crates/cgap-storage/tests/` (five court binaries + shared `common/mod.rs`)
