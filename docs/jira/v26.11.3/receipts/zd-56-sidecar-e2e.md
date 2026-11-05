# ZD-56 — Sidecar Sequential Serve + Rung-8 E2E Court
Standing: REFUSED

Date: 2026-11-04 · Lane: ZD-56 · Repo: /Users/sac/cgap
Plan: wave-2-membrane-plan.md:189 (rung 8: "E2E ... decrypted == pre-seal bytes, tag gate fires on flipped byte")

## Diff (owned files only)

1. `crates/cgap-ffi/src/bin/aoq_sidecar.rs` — serve-loop extension:
   - `main()` now binds once and LOOPS `listener.accept()`, serving each
     connection sequentially. Each connection constructs a FRESH
     `Session` (serve() builds its own), so multi-hop courts reuse one
     process with zero cross-connection state.
   - Stop signal = stdin EOF (blocking read in a watcher thread; the
     runtime owns time — no spin, no sleep). The watcher unlinks the
     socket path before `exit(0)` — the sidecar now cleans up its own
     socket, closing the zd-42 stale-socket class at the source.
   - Fail-closed-per-window law unchanged: any framing/tag/session fault
     maps to one frozen-band word, closes THAT connection only, and the
     accept loop continues on the next connection.
   - XIP cfg-stub pattern preserved: body inside
     `#[cfg(not(target_os = "none"))] mod imp`; `fn main() {}` stub.
2. `crates/cgap-ffi/tests/aoq_sidecar_court.rs` (disclosed compat touch,
   court intent unchanged) — the stop signal is now stdin EOF, so the
   harness pipes sidecar stdin and holds it open for the court's
   lifetime; J3 leg 1 now drops the pipe after the zero-byte client close
   to trigger the exit-0 witness. J1/J2 assertion bodies untouched.
3. `crates/cgap-ffi/tests/aoq_sidecar_e2e_court.rs` (NEW, 413 lines) —
   the rung-8 court, driven cgap-side (arcus cannot do the ML-KEM
   handshake; zd-42 finding). Three courts, each with a named mutation:
   - **E1** happy path: real ML-KEM-768 keygen(d,z) → court encapsulate
     → HandshakeEk (ord 0) OK → HandshakeFinish (ord 1, pending ct) OK
     → Established → AppData (ord 2, seq 7) OK → response window passes
     `ipc::read_verified` framing tag gate → `sess::parse_frame` →
     emit_verified kdf tag re-derives over the covered frame → echoed
     payload == pre-seal bytes. Transcript:
     `preseal_len=53 preseal_blake3[0..8]=07 f6 94 9e 14 28 34 a4,
     echoed_blake3[0..8]=07 f6 94 9e 14 28 34 a4, equal_digests=true`.
   - **E2** flipped-byte control: one flipped tag byte
     (`ek_win[AOQ1_TAG_AT] ^= 0x01`) on a lawful HandshakeEk window →
     typed `AOQ_STATUS_DIGEST_MISMATCH`, no window, zero-byte
     post-refusal read (mutation B: an echo-instead-of-refuse sidecar
     would have returned request bytes).
   - **E3** process reuse + stop signal: fresh full handshake + AppData
     echo on connection 3 of the SAME process (mutation C': the pre-ZD-56
     one-connection binary fails this connect), then stdin EOF → bounded
     exit-0 poll passes AND socket path absent (janitor witness).
   - No sleeps anywhere: bounded polls on socket existence, connect
     readiness, child exit; sockets unique-per-run + Drop-unlinked.

## Commands / exits (this session, exact subject /Users/sac/cgap)

| Command | Result |
|---|---|
| `cargo test -p cgap-ffi --test aoq_sidecar_e2e_court` ×3 (incl. --nocapture transcript run) | `3 passed; 0 failed` each |
| `cargo test -p cgap-ffi --test aoq_sidecar_court` ×2 | `3 passed; 0 failed` ×2 (ZD-13 courts unregressed) |
| `cargo test -p cgap-ffi` (full) | all suites `ok`, 0 failures (lib 60 passed; e2e 3 passed) — exit 0 |
| `cargo test -p cgap-ffi --lib xip` | `7 passed; 0 failed` |
| `cargo test -p cgap-ffi --lib xip_t1_riscv_elf_size_court` | `1 passed; 0 failed` |
| `cargo build --workspace` | Finished dev profile, exit 0 |

## Notes / disclosed

- **Pre-existing A07 gate precondition**: first `cargo test -p cgap-ffi`
  run failed wave9_a07 (exit 2: `GATE-REFUSED: crate-no-artifacts ...`) —
  the gate requires prior `cargo build` artifacts. Fixed forward with
  `cargo build --workspace`; not caused by this diff (bin/test-only
  changes; cgap-ffi scanned=2 clean). Session-introduced failure: none.
- **Pre-existing (NOT regressed, NOT fixed here)**: `cargo check -p
  cgap-ffi --target riscv32imc-unknown-none-elf` fails E0463 ("bin does
  not declare `#![no_std]`") — structural to bin targets, unchanged by
  this diff (edits only inside the cfg-gated `mod imp`). The verified
  XIP gate is the lib court set above, per zd-13.
- **Backward-compat touch on ZD-13 court**: stdin piping only (above);
  J1/J2/J3 assertion bodies unchanged; 3/0 ×2 after the change.
- **Janitor sweep (pre-existing stale sockets, zd-42 ps witness)**:
  found 17 stale `aoq_sidecar_court_*.sock` in `~/.cache/tmp` (Nov 4
  01:59–02:32). 13 unheld → removed; 4 held by orphaned sidecar
  processes from this session's earlier pre-fix court runs (PIDs 13496,
  50029, 51294, 57951) → killed, then names removed. Post-sweep: 0
  aoq sockets in `~/.cache/tmp` and 0 in `$TMPDIR`. No pkill of
  unknown-ownership processes; every killed PID was read via ps first
  and belonged to this session's failed runs.
- No git commands run, per lane contract.

## Standing

ALIVE — every command above run this session against the exact subject;
final state re-read on disk (courts 3/0 ×2 each, full suite exit 0,
XIP courts green, 0 stale sockets).
