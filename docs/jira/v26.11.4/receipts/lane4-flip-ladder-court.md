# LANE-4 — Flip-Ladder Court Receipt (real release sidecar)

Date: 2026-11-04. Lane: LANE-4 (FLIP-LADDER PIN + STATUTORY EXPORT FORMATTER).
Repo: /Users/sac/cgap. New file only: `crates/cgap-ffi/tests/aoq_flip_ladder_court.rs`.

## Subject + binary freshness

Court file: `crates/cgap-ffi/tests/aoq_flip_ladder_court.rs` (7 legs).
Binary: `/Users/sac/cgap/target/release/aoq_sidecar`, rebuilt from the
committed source + working tree before the ladder ran.

- Pre-run sha256: `4dd9f5810e106bce55a1440564d93acac1fcbcd708997a444a820df1b6c0afbf`
  (release binary of 12:55; verified byte-identical after a full rebuild —
  fresh w.r.t. the then-working tree).
- Post-run sha256: `9a6726e45600115b81f505d0373778a22fae13c6cfd5da04ee12617f15d35433`.
  The tree MOVED under this lane mid-run (concurrent lanes committed
  add8a3d4 and edited cgap-aoq/cgap-ffi sources); the binary was rebuilt by
  the concurrent lane activity and the court was RE-RUN green against the
  rebuilt binary (7/7 pass, 1.09 s). The lane-start binary
  (4dd9f581…) witnessed the same 7 legs green earlier in the session.
- C1-mid-commit disclosure: the ladder does NOT pin a cgap commit — it ran
  against the live release binary rebuilt from the working tree both times.

## OBSERVED frozen-band status map (real words are authoritative)

The dispatch's assumed mapping is REFUTED; the real map below is
authoritative (aoq_sidecar.rs `session_status` frozen-band collisions):

- version skew 3→2 (internally consistent header, honest outer blake3 tag)
  → **DIGEST_MISMATCH (0x4347A005)** — Wire(UnsupportedVersion) collides
  into the digest word in the frozen band. NOT EPOCH_MISMATCH.
- unknown epoch (outside closed enum {0,1,2,65535}) → **EPOCH_MISMATCH
  (0x4347A006)** via UnknownEpochWire — the TRUE epoch trigger.
- tampered byte + stale tag → DIGEST_MISMATCH (0x4347A005) at the framing
  tag gate (read_verified BEFORE accepting), never an echo (zero-byte
  post-refusal read asserted).
- legacy 1088B Finish → INCOMPLETE_FRAME (0x4347A004) (LengthMismatch
  expected 1104). Lawful 1104B Finish → OK; session Established; sealed
  AppData → OK + echo window.
- budget: AppData #1..#4096 ACCEPTED, the **4097th AppData frame refused
  with BUSY (0x4347A003)** — the budget counts AppData frames per key
  (handshake frames excluded), refusal at exactly 4097 (off-by-one is a
  red court). Dispatch's "frame 4097" is witnessed as the 4097th AppData
  frame, not the 4097th session frame.
- ordering: version+length-bad Finish dies at the version law (collision
  word DIGEST_MISMATCH), never the length word INCOMPLETE_FRAME — codec
  order magic→version→epoch→digest, parse_frame runs codec::decode before
  the Finish length law. Digest-only contrast fires DIGEST_MISMATCH.

## Legs (7, all green)

L1 tamper/stale-tag → DIGEST_MISMATCH + never-echo (zero-byte post-read).
L2 version skew → DIGEST_MISMATCH (observed collision word).
L2-EPOCH unknown epoch → EPOCH_MISMATCH.
L3 ordering (version+length-bad → version law, not length law) + digest
contrast leg.
L4 legacy 1088B Finish refused (INCOMPLETE_FRAME); lawful 1104B Finish
accepted + sealed AppData OK + echo.
L5 budget boundary walked ONE FRAME AT A TIME with tolerant reads: refusal
at exactly AppData #4097, word BUSY.
AV anti-vacuity: the honest HandshakeEk frame (sidecar ACCEPTS with OK) fed
through L2's expectation path — the leg MUST fail (catch_unwind); a court
that cannot flip is FAIL.

## Commands + exits

- `cd /Users/sac/cgap && cargo build --release -p cgap-ffi --bin aoq_sidecar` → exit 0
  (twice: pre-run sha 4dd9f581…, post lane-move sha 9a6726e4…).
- `cargo test --release -p cgap-ffi --test aoq_flip_ladder_court` → exit 0,
  7 passed / 0 failed (final run 1.09 s). Earlier failures during
  development were REAL red courts against the real binary (wrong expected
  words vs observed map) and were resolved by correcting the court to the
  OBSERVED law, never by weakening an assertion (see L5: the court's own
  BOUNDARY log printed the observed boundary AppData #4097).

## Standing

ALIVE — commands + exits this session against the exact subject (real
release sidecar process per leg, real UDS windows, real ML-KEM handshake).
