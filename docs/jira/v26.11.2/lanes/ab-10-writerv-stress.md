Canonical location: docs/jira/v26.11.3/receipts/ab-10-writerv-stress.md

# Lane AB-10 Receipt: StreamWriter/VectoredSink Stress Suite

Lane: STRESS (AB-10) · Subject: `crates/cgap-arrow/src/stream_writer.rs`
(`StreamWriter` + `VectoredSink` over real UDS) · Read-only subject; only
file written: `crates/cgap-arrow/tests/stream_writer_stress.rs` (new).

## Gate (ran this session, exits witnessed)

| Command | Exit |
|---|---|
| `cargo test -p cgap-arrow --test stream_writer_stress` (pass 1) | 0 |
| `cargo test -p cgap-arrow --test stream_writer_stress -- --nocapture --test-threads=1` (pass 2) | 0 |
| `cargo test -p cgap-arrow` (suite unregression) | 0 |

Pass 2 full line: `test result: ok. 4 passed; 0 failed; 0 ignored;
0 measured; 0 filtered out; finished in 0.77s`.

## Deliverables

1. **Soak (50k frames)** `soak_50k_frames_byte_identity_and_throughput`:
   50,000 frames over ONE live UDS pair, per-frame
   emit → writev scatter → peer `read_exact`, each frame byte-identical to
   an independent raw `ipc::emit` reference (seq stamped by the writer over
   the caller value). Receipt (serial pass):
   **50,000 frames, 25,657,040 bytes, 441.962 ms, 113,132 frames/sec,
   55.4 MB/s**. Timing-tolerant: throughput is printed, never fenced.
2. **Partial-accept storm** `partial_accept_storm_firstseg_10k`: a real
   `FirstSegSink` (one segment per call) drives 10k frames; reassembly is
   byte-identical to the single-shot reference concatenation; `calls >
   frames` proves multiple accepts per window; `ZeroProgress` never fired.
3. **Stalled-sink recovery** `stalled_sink_recovery_seq_lockstep_through_u32_wrap`:
   full-accept sink injects `Ok(0)` every 4th call for 1,000 frames started
   at `u32::MAX - 2` (seq walks THROUGH the u32 wrap). Refusals recovered
   via `into_inner` + retry at the same frame start; per-frame wire-seq
   asserts (read from meta offset 0x08+0x30) prove no skips, no doubles;
   full reassembly byte-identical to the reference concatenation; exact
   arithmetic court `calls == frames + injects` (1,333 = 1,000 + 333) is a
   skip/double detector.
4. **FD hygiene** `fd_hygiene_1k_uds_pairs`: 1,000 sequential UDS pairs
   created and torn down; lsof `-p <pid> -F n` total **8 → 8 (delta 0)**
   (serial pass; parallel runs showed delta 0 with sibling-test sockets
   excluded by the `-fd` path filter), 0 sockets left in the test's path
   namespace (asserted == 0). Lock-fd law: no leaked fds.

## Anti-vacuity

- Soak/storm/stall compare against an INDEPENDENT reference path (raw
  `ipc::emit`), so any StreamWriter framing regression flips all three.
- Stall court asserts `zero_injects > 0` (stalls actually landed); fd court
  asserts lsof listed a nonzero fd count pre-loop.
- FirstSegSink returns `Ok(0)` on an empty first segment specifically so
  the storm court can flip if the writer ever passes empty segments.

## Fixture defect found and fixed in-suite (not a subject defect)

The first stalled fixture (periodic `Ok(0)` against a one-segment-per-call
sink) was adversarially aligned: every retry attempt made exactly 3 accepts
then died on the injected zero — attempts never completed and the caller's
retry loop spun forever. Witnessed live: stray test binaries burned 15+
CPU-minutes each before being killed. The writer itself behaved lawfully
(it refused `ZeroProgress` on every inject). The fixture is now full-accept
with periodic inject; the termination law is documented in the fixture doc
comment (`stream_writer_stress.rs`, `StalledSink`).

## Standing

ALIVE — gates ran and passed this session against the exact subject,
commands + exits above.
