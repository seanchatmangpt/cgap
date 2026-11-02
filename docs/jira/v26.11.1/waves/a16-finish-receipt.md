# A16 Finish Receipt

- **Lane**: A16 (`crates/cgap-crypto` — `chatman.rs`, `lease.rs`), lane-finisher
  pass, v26.11.1, repo `/Users/sac/cgap` (one canonical checkout)
- **Date**: 2026-11-01
- **Dispatch**: assess the wave landed in 9be9109d; witness §6 seal/replay,
  §5 lease monotonicity, the host gate, and >=1 sandbox flip this session;
  close the missing-receipt gap the wave-6 audit filed as D-A16-1
- **Verdict**: **ALIVE** — host gate 24/24 exit 0 and the full 9-shim flip
  battery exit 0 (every shim exits 101, `SANDBOX==REPO after battery: YES`),
  all run this session. Repo lease files byte-identical (sha256) pre/post
  battery.

## Assessment (what was already landed; nothing rewritten)

`crates/cgap-crypto` on `main` @ 62215326 (crate sources; audit-certified
bytes) carries Waves 1+2 in full: the 112-byte `DeltaS` codec, the 80-byte
`PhiLease` codec with compile-time offset pins, the keyed single-chunk BLAKE3
engine pinned to official `test_vectors.json` `keyed_hash` KATs (0/1/63/64/
192/1024) plus a b3sum-derived 192-byte production vector, the §4 `Sealer`
trait seam, and the §4 UNSUPPORTED-clause `Unsealed` tier (magic `"CGRU"`,
version `0x00`, `algorithm_id 0x0000` — loudly typed, MAC-class, per the
A18 D-A18-4 honest-scope precedent). `seal_unsealed` is the only tick
consumer; `verify_unsealed` is pure. Wave-3 PQC (ML-DSA-65) remains open by
design, `RECEIPT_LEN = 2649` pinned for it. No code changes were needed; the
finisher wrote the receipt and re-witnessed every gate.

## Commands (this session, exact)

```text
CARGO_TARGET_DIR=/tmp/cgap-lane-A16-target cargo test -p cgap-crypto
  -> exit 0: 18 chatman + 6 lease + 0 doc, 0 failed
python3 crates/cgap-crypto/scripts/wave7_shim_battery.py -> exit 0
  (sandbox /tmp/a16-w7-sb; 9 shims; each run exit 101; restores byte-exact)
shasum -a 256 crates/cgap-crypto/{src/chatman.rs,src/lease.rs,src/lib.rs,
  Cargo.toml,tests/chatman_courts.rs,tests/lease_courts.rs}  # pre == post
git log --oneline -- crates/cgap-crypto/ ; git status --short   # read-only
```

## Courts run (name -> pass / flip evidence, this session)

- **§8.1 Monotonicity (tick side)** — pass:
  `monotonicity_court_tick_sequence_strictly_increasing` green; flip: M1
  (tick advance dropped) fires it and `current_tick_read_is_pure_never_advances`.
- **§8.1 Monotonicity (phi side)** — pass:
  `monotonicity_court_phi_sequence_strictly_increasing_and_distinct` green;
  flip: M1 fires it (plus roundtrip + seal-exhaust probe).
- **§8.2 Exhaustion trap** — pass:
  `exhaustion_court_trap_observed_in_real_process` green (real subprocess,
  stderr signature demanded); flip: M2 (trap -> warning) fires exactly it;
  M1 also flips it (wave-7 D-A16-4 repair: wrong-reason deaths no longer
  satisfy it).
- **§5 Constructor ceiling-0** — pass: `constructor_court_zero_ceiling_traps`
  green; flip: M3d fires exactly it.
- **§8.3 Byte sweep** — pass: 458 flips (229 offsets x 2 masks),
  region-typed errors; flip: M4 (first-byte-only digest compare) fires
  exactly it.
- **§6 Seal/replay/state-bit** — pass: `seal_then_verify_roundtrip` +
  `determinism_court_identical_inputs_identical_bytes` +
  `full_receipt_bytes_are_pinned_end_to_end` green; flip: M6 fires
  determinism + pin; M5b fires 5 incl. the sweep.
- **§8.4 Chaining / epoch swap** — pass:
  `chaining_court_receipts_chain_and_epoch_swap_fails` green; flip: M5b
  fires it; M5 (key-ignore) does NOT — epoch binding is structural
  (disclosed robustness positive; suite still exits 101 via the KAT).
- **§8.5 Determinism** — pass: byte-identical receipt across independent
  builds; flip: M6 fires it.
- **Wave-6 Hostile inputs** — pass: all-zero / all-0xFF / cross-tier magic
  -> typed refusals, no panic; flip: M7 (magic check dropped) fires exactly
  it.
- **§4 Sealer seam** — pass: 68-flip seal-body sweep green; flip: M8 fires
  exactly it.

Dispatch mission item (2), "lease n-1 after n -> typed refusal": refusal is
TYPE-LEVEL per spec §5 — rewind is unrepresentable (private `Tick`, no
`set_tick`, no API accepts a caller tick; M3-class rewind shims are
compile-aborts, recorded as such, not failure counts). The flip court for the
monotonicity law is M1 -> the two monotonicity courts fire. Runtime rewind
refusal by error return would violate §5's trap discipline; spec is
authoritative, dispatch wording maps to §5.

Zero vacuous courts: 9/9 mutation classes flip, every mutated run exits 101.

## Divergences

1. **FFI seam not wired (assessment finding)**: dispatch said the 9be9109d
   sovereign C-ABI in `crates/cgap-ffi` is fed by this crate; grep over
   `crates/cgap-ffi/` finds zero references to `cgap-crypto`/`cgap_crypto`.
   The feed seam is unwired; wiring it is outside this lane's lease.
2. **Receipt gap closed (was D-A16-1)**: the wave-6 audit receipt records
   that no A16 lane receipt existed on disk; this file is that receipt.
3. **Doc-vs-impl (disclosed in-code, doc yields; unchanged from audit
   D-A16-2)**: §3.3's "117 bytes + seal size" vs pinned 229 + 2420 = 2649;
   provisional `algorithm_id` values 0x0000/0x0001/0x0002; unsealed-tier
   encoding "CGRU"/0x00; admissions tree construction; lease_id randomness
   as caller policy; `Unsealed` MAC-class scope.
4. **Battery sandbox debris**: battery leaves `/tmp/a16-w7-sb`; deleted
   after the run together with the lane target dir (lease-lease cleanup).

## Standing

- Keyed BLAKE3 engine, delta/phi codecs, `seal_unsealed`/`verify_unsealed`,
  admissions root: **ALIVE** (exit-0 host gate + green sandbox baseline, this
  session).
- Courts §8.1-§8.5 + constructor + hostile + KAT + sealer: **ALIVE**, 9/9
  flips, zero vacuous.
- Wave-3 PQC (ML-DSA-65) seal and §3.4 key-rotation story: **UNSUPPORTED by
  design** (the §4 UNSUPPORTED clause; `Unsealed` loudly typed) — open scope,
  not a defect.
- Perf numbers: none claimed, none measured.

## See Also

- `docs/jira/v26.11.1/lanes/A16-chatman-pqc.md` (normative lane doc)
- `docs/jira/v26.11.1/waves/a16-wave6-audit-receipt.md` (independent audit;
  D-A16-1 closure)
- `crates/cgap-crypto/scripts/wave7_shim_battery.py` (replayable battery)
