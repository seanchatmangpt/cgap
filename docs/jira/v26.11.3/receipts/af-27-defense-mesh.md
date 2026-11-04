# AF-27: defense telemetry mesh (macro-benchmark)

Status: ALIVE (host std test harness; target-unmeasured items disclosed below)

Lane: AF-27, implementation lane, v26.11.3 (macro-benchmark: defense
telemetry mesh). Owned files only: new
`crates/cgap-aoq/tests/macro_defense_mesh.rs` + this receipt.

## Subject

- `crates/cgap-aoq/tests/macro_defense_mesh.rs` (new, 474 lines)
- `docs/jira/v26.11.3/receipts/af-27-defense-mesh.md` (this file)

## What ran (commands + exits, this session)

- `cargo test -p cgap-aoq --test macro_defense_mesh -- --nocapture`
  - Gate run 1: exit 0, `test result: ok. 1 passed; 0 failed`, wall 10.4 ms.
  - Gate run 2: exit 0, `test result: ok. 1 passed; 0 failed`, wall 90 ms.
- `cargo test -p cgap-aoq` (full crate): exit 0, zero failing targets
  (9+14+1+2+1+1+7+6+2... all `0 failed`; incl. af30 61.95 s target).

## Scenario as measured

64 independent session channels (scale decision: the dossier's 1024 is
scaled 16x down, disclosed — a 1024-channel host run is a follow-up lane;
a true no_os/embedded-target run is a future embedded-target lane), each:
test-KEM handshake through the real AB-7 4-state machine
(`Session::step` over `parse_frame`), then 48 telemetry frames per channel
in 2 bursts of 24 across two supervisor rounds (8 radar / 8 kinetic /
8 launch-state shapes per burst), sealed via `Session::emit_verified`
(compose-with-emit_verified tag law: tag = H(body)[0..4], re-derived by
`AoqChain::append_verified`). The AB-3/AB-4 AEAD seam is pre-admission
(R-K1): every attempted `aead::seal` is a typed
`AeadKernelNotAdmitted` refusal — census'd honestly, never faked as a
seal. Frames append into PER-CHANNEL `AoqChain`s (chain topology
decision: per-channel, not single audit chain — every record of channel c
verifies against GENESIS-parented per-channel `verify_records`, and the
attribution census below is the bleed witness; a single-audit-chain
variant is a follow-up lane).

## Adversarial injection + supervisor churn

- Unannounced link termination: 8 channels (40..48) — the round-1 worker
  is killed mid-stream while 24 frames remain; no Close frame; the frozen
  session stays `Established` at its frozen ordinal and continues
  absorbing (asserted state witness) in round 2.
- Partial-frame truncation: 4 channels (8..12), 3 cuts each (40/52/66 B)
  -> typed `LengthMismatch`/`Wire(HeaderTooShort)` refusals (12).
- Out-of-order ordinals: 4 channels (4..8), 3 ahead-ordinals each ->
  typed `OrdinalMismatch` refusals (12).
- Supervisor churn (AB-17 court pattern): 20 worker threads killed
  mid-stream at peak (round-1 join while bursts remain) and 20 respawned
  (round 2), slots = channels round-robin c % 20.
- Lawful close battery: 4 channels (60..64) closed at the round boundary;
  the close frame is absorbed, an untagged close frame appended to the
  chain is a typed `DigestMismatch` (tag refusal, 4).
- Closed-session refusal battery: 96 typed `SessionClosed` refusals,
  raw-crafted stale frames; no mutation (closed sessions cannot emit —
  the stale frames are header-legal but refused by state law).

## Measured results (gate run 2, host std harness)

- Telemetry frames absorbed: 2976 (+4 lawful close frames).
- Chain records: 2976 (per-channel chains), 357,120 record bytes
  (2976 x 120 B pinned records).
- Attribution bleed: ZERO — every chain record digest maps to exactly one
  channel (global digest->channel map, duplicate = bleed = fail).
- Chain continuity: all 64 per-channel chains verify
  (`verify_records`, GENESIS-parented).
- Refusal census (typed, all observed): ordinal=12, truncation=12,
  session_closed=96, aead_seam=2976, close_tag=4.
- Supervisor churn: 20 killed mid-stream / 20 respawned, worker digest
  operations 2976.
- fd delta: 0 (no descriptor leak).
- RSS delta: 1376 KB / 1488 KB across the two gate runs (before 2272/2256
  KB, after 3648/3744 KB) — recorded, not gated; the true footprint
  truth is a linked no_os ELF (repo law), TARGET-UNMEASURED.
- Wall time: 10.4 ms / 89.7 ms (debug profile, host) — recorded, not
  gated.

## Target-unmeasured (honest disclosures)

- Dossier's no_std-warfare and EW-survivability claims: TARGET-UNMEASURED
  here — this bench runs on the host std test harness with the test-KEM
  (typed disclosure: test-KEM fallback, AB-5's ML-KEM in-flight at
  cgap-crypto). A true no_os embedded-target run is a future
  embedded-target lane.
- 1024-channel scale, release-profile timing: TARGET-UNMEASURED.
- AB-5 ML-KEM: in-flight (not bound into this mesh; the seam is
  test-KEM-bound here, disclosed).

## Standing

- Mesh bench: ALIVE (commands + exits above, this session, exact subject
  files listed under Subject).
- no_os/EW/1024-scale claims: UNVERIFIED (never rounded up).
