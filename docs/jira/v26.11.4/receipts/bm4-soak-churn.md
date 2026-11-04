# bm4 soak receipt (cgap) — v3 re-pin disclosure

Standing: ALIVE (1M trips, 55,411 fps, exit 0; v3 re-pin disclosure)

- Subject: cgap @ `23436067ab758dfeb034a2957406aa6906c56c63` + in-flight working tree (uncommitted: Cargo.lock, cgap-aoq `aead.rs`, `generated_aoq_header_layout.rs`, `session.rs`, `aoq1_codec_court.rs` — the M2a v3 Finish law churn).
- Date: 2026-11-04, 12:59–13:02 PST.
- Machine: Darwin 25.2.0, Apple M3 Max (16 cores), 48 GB, load avg 37.28/28.43/25.99 during run (stagger disclosed below).
- Stagger: lock `/tmp/bm-lane4.lock` held 12:00–13:0x; poll-waited (180 s interval) for `/tmp/bm-lane1.done` AND `/tmp/bm-lane3.done`. Lane 3 landed 12:42. Lane 1 never landed → **proceeded at the 60-min cap (13:00), lane-1 contention disclosed**.

## Court re-pin (disclosed, integration-churn repair — NOT silent shrinkage)

First release run (12:59) FAILED both tests at the handshake:
`connect_established never succeeded within 10s: HandshakeFinish refused 0x4347a004` (exit 101).

Classification per the lane contract: **integration-churn (M2a v3 Finish surface), not a soak defect.** Verified mechanism:

- In-flight `session.rs` churn changed the HandshakeFinish law: v2 body = ct-only (1088 B) → v3 body = ct (1088 B) + confirm tag (16 B) = 1104 B (`FINISH_BODY_BYTES`), confirmed-tag refusal word `INCOMPLETE_FRAME` = `0x4347A004` (`aoq/status_maps.rs`).
- The soak court's client was still v2-shaped. Four sibling courts (`aoq_sidecar_court`, `aoq_sidecar_e2e_court`, `aoq_session_ffi_court`, `aoq_events_conformance_court`) were migrated to v3; the soak court (X9's file, not in the in-flight diff) was missed.
- Probe: migrated J1 (`aoq_sidecar_court j1_sidecar_handshake_and_verified_appdata_roundtrip`) passes → sidecar echo path healthy; the defect was the soak client's shape only.

Repair (premise-rot law: re-pin the test to the new lawful behavior):

1. Handshake Finish hop now derives `transcript`, `hs_master`, `recv_key`, `confirm_tag` and sends the 1104-byte v3 body (mirrors J1).
2. `SessionKeys { recv_key, send_key }` returned from the connect helpers.
3. `echo_trip` re-pinned to the v3 AppData law: body `[seq LE][ct][tag 16B]`, AEAD `seal_detached` under `recv_key` (AAD = 69-byte prefix, nonce `EPOCH ‖ ordinal:seq`); echo opened under `send_key` with plaintext **byte-identity assert** (`open_detached` + `assert_eq!(&plain, payload)`).

## Run 2 (final, release, --test-threads=1)

Command: `cargo test --release -p cgap-ffi --test aoq_sidecar_soak_1m_court -- --nocapture --test-threads=1` — **exit 0**, 2 passed.

Verbatim receipt line:

```
SOAK RECEIPT: 1000000 REAL verified echo round trips in 18.047s = 55411 fps (wall budget 120s; per-trip 18.0 us)
```

Phase trace: 10/10 phases complete, 18.05 s elapsed at phase 10. No wall-budget event; no shrinkage (full 1,000,000 trips, all verified).

## Plane disclosure

Soak fps (55,411) is **Rust-reachable echo fps over the real aoq_sidecar UDS transport** (ML-KEM-768 handshake, AEAD per trip). NOT Go-reachable fps — no Go plane participates in this court.

## Interpretation bound

Prior (X9, pre-v3): 15.8 s / 63,223 fps. Current: 18.05 s / 55,411 fps (−12.3%). The delta is the v3 law's per-trip AEAD seal+open (real Poly1305 work per trip where the v2 client did none) — a lawful-work delta, not a regression of the transport. No silent shrinking: trip count is 1,000,000, byte-identity asserts held on every trip.

## Falsifiers / status

- Falsified: "soak court is runnable against the current tree without re-pin" (run 1, exit 101, receipt above).
- ALIVE: soak court passes at 1M trips, exit 0, receipt line captured.
- Residual: the re-pin diff to this test file is lane-owned by X9/M2a integration — left uncommitted in the working tree for the coordinator's integration pass.
