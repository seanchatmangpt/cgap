# Receipt: W2-A — cgap-ffi session C-ABI surface (zd-6, wave 2)

Status: ALIVE (all commands below ran this session, exit codes cited)
Lane: W2-A, docs/jira/v26.11.3/plans/wave-2-membrane-plan.md §1
Date: 2026-11-04

## Subject (exact)

- `crates/cgap-ffi/src/aoq_session.rs` (new, 5 exports + last-error)
- `crates/cgap-ffi/src/lib.rs` (one cfg-gated `pub mod aoq_session;`)
- `crates/cgap-ffi/tests/aoq_session_ffi_court.rs` (new, 6 courts)
- `crates/cgap-ffi/Cargo.toml` — DISCLOSED CROSS-TOUCH (manifest is
  coordinator-owned; disclosed here per the A19 cgap-p28 precedent): two
  host-gated deps added under the existing
  `[target.'cfg(not(target_os = "none"))'.dependencies]` section:
  `cgap-aoq = { path = "../cgap-aoq", version = "26.11.3" }` and
  `cgap-crypto = { path = "../cgap-crypto", version = "26.11.3" }`. The
  `target_os = "none"` XIP build keeps the pre-existing dep set — the
  staticlib stays allocator-free. Directed by coordinator contract block
  delta 1.

## Surface

- `cgap_aoq_session_new(epoch: u16, key_id: u64, ordinal: u32) -> u64`
  (monotone handle, never reused; 0 = internal fault)
- `cgap_aoq_session_step(handle, frame_ptr, frame_len) -> u32`
- `cgap_aoq_session_emit_verified(handle, seq: u32, body_ptr, body_len,
  out_ptr, out_cap, out_len) -> u32` — `seq` is an explicit parameter
  (deviation from the plan's 6-arg sketch, required by the engine's
  `emit_verified(seq, payload, out)` law; matches contract-block delta 5).
- `cgap_aoq_session_fingerprint(handle, out32) -> u32`
All five exports return closed-set status words, never -1.
- `cgap_aoq_session_free(handle) -> u32`
- `cgap_aoq_last_error() -> u32` (thread-local pull law: the caller
  polls, the kernel never sleeps).
- Handle table: mutex-guarded `BTreeMap<u64, SessionEntry>` + tombstones;
  stale and never-existed handles refuse identically (`KEY_UNKNOWN`, no
  existence oracle). Lock-poison recovery is bounded: guarded mutations are
  plain insert/remove, so `into_inner()` observes a structurally intact
  table.
- Host KemSeam: `HostKem` — real ML-KEM-768 over `cgap_crypto::kem`
  (encaps/decaps), Shake256 KDF, deterministic-from-ek encapsulation seed
  (`b"w2a/mlkem/m"` domain, no RNG at the boundary) — vendored from the
  corridor join court's test-local `MlkemKem` (AB-5, AF1-KD-1 closed).

## Status mapping (contract delta 4; pinned, disclosed collisions)

10 SessionError variants -> 8-word frozen band 0x4347A001..A008,
family-clustered in `status_of`:
- INCOMPLETE_FRAME <- LengthMismatch, HeaderTooShort, BufferTooShort,
  FrameOrdinalRegression, SeqGap, OrdinalMismatch (truncation/cursor law)
- DIGEST_MISMATCH <- BadMagic, UnsupportedVersion, DigestMismatch,
  ReservedNotZero, UnknownFrameKind, TranscriptMismatch (integrity law)
- EPOCH_MISMATCH <- UnknownEpochWire, EpochMismatch, UnexpectedFrame
  (epoch/transition law; the closed band has no "wrong state" word, so the
  UnexpectedFrame collision is PINNED here, not guessed at call sites)
- KEY_UNKNOWN <- KemKernelNotAdmitted, AeadKernelNotAdmitted, KemRefused
  (identity law; also serves stale/never-existed handles)
- BUSY <- RekeyRequired (counted-budget law)
- EPOCH_RETIRED <- SessionClosed (terminal law)
- PENDING reserved (never returned by this surface).

## Courts (crates/cgap-ffi/tests/aoq_session_ffi_court.rs, 6 tests)

1. `court1_happy_path_handshake_and_emit_round_trip` — real ML-KEM-768
   handshake (keygen from Shake256 seeds, ek frame, ct-echo finish) to
   Established, verified emit round-tripped through the real
   `parse_frame` (kind/header/ordinal/seq/payload) + emit tag re-derived
   from the wire bytes. Close + free accepted.
2. `court2_ct_echo_flip_refused_fingerprint_unchanged` — MUTATION A: one
   flipped ct-echo byte -> DIGEST_MISMATCH, fingerprint byte-identical
   (session law 4 across the FFI boundary).
3. `court3_fingerprint_identity_after_every_refused_step` — six-frame
   refusal battery (wrong epoch / wrong ordinal / unknown kind 0x7F /
   truncated header-only / garbage magic / AppData-in-HandshakeInit);
   every word in the frozen band, last-error mirror, LAW 4
   fingerprint-identity per refusal; anti-vacuity leg: an accepted step
   must MOVE the fingerprint (otherwise identity-after-refusal is
   vacuous).
4. `court4_stale_handle_refused_everywhere` — freed handle refused by
   step/fingerprint/emit/free identically (KEY_UNKNOWN); double free
   refuses; never-existed handle refuses identically; fresh handle never
   reuses a freed id (monotone).
5. `court5_hostile_inputs_never_panic` — null frame ptr, zero-len frame,
   null fingerprint buffer, null out / null out_len / too-small out cap:
   all typed refusals, process survives (catch_unwind containment is the
   mechanism; the passing test is the witness).
6. `court6_status_band_is_closed` — all 8 band words round-trip through
   the generated `aoq_status_name`/`aoq_status_word` map; outside-band
   words (retired 0x4347_000x band, 0x4347_A009, 0) map to None.

Plus 2 in-module lib tests (`aoq_session::tests`): frozen-band closure of
the mapping constants and stale-handle word pinning. Lib count 58 -> 60.

## Verification ladder (this session, exact commands + exits)

| # | Command | Exit | Witness |
|---|---|---|---|
| 1 | `cargo test -p cgap-ffi` (run 1, pre-sidecar-landing) | 0 | 14 suites ok: lib 60/60 (58 pre-existing + 2 new), court 6/6 |
| 2 | `cargo test -p cgap-ffi --test aoq_session_ffi_court` (run 2, standalone) | 0 | 6 passed / 0 failed |
| 3 | `cargo test -p cgap-ffi` (final run A, post W2-C sidecar stabilization) | 0 | 14 suites ok, incl. sidecar court 3/3 |
| 4 | `cargo test -p cgap-ffi` (final run B) | 0 | 14 suites ok (×2 gate green) |
| 5 | `cargo check -p cgap-aoq` | 0 | kernel crate untouched |
| 6 | `cargo check -p cgap-ffi --all-targets` | 0 | after W2-C stabilization |
| 7 | `cargo build --workspace` | 0 | restored A07 court precondition during mid-wave churn |

## Blocker disclosure (mid-wave, resolved)

Run 2 of the ladder was blocked for ~35 min by lane W2-C's in-flight
`crates/cgap-ffi/src/bin/aoq_sidecar.rs` (two of its states broke the XIP
link courts — std/Vec/Err unresolved under riscv32imc — and its court was
red while in flux). Coordination: coordinator ruling delivered to W2-C
(cfg-gate the bin body under `not(target_os = "none")` + empty none-target
main stub); after W2-C's file settled, runs A/B went green ×2. No edit was
made to any W2-C file by this lane (SLA unblock not exercised — the owning
lane repaired within the window). The transient A07 zero-string-gate
failure during the churn window was environmental (5 workspace crates
temporarily had zero debug artifacts; cleared by a workspace build), not a
diff regression.

## Deviations / disclosures

- Cargo.toml cross-touch (above) — coordinator-directed (delta 1).
- `seq` parameter on emit_verified — required by the engine law, matches
  contract-block delta 5.
- EPOCH_MISMATCH serves the UnexpectedFrame collision (pinned, court 6
  asserts the band stays closed; the W2-D bijection drift court owns the
  Go-side mirror of this map).
- KEM keypairs in courts use deterministic Shake256 seeds (no RNG), the
  corridor court's disclosed discipline.

## Standing

ALIVE for the surface, the 6 courts, and the ×2 ladder (rows 3-4).
Not yet ALIVE: W2-C sidecar integration courts (W2-C/E lanes), W2-D Go
bijection drift court, plan §2 gates 2-8. The corridor 0.56 us/frame AF-30
number stays a TARGET for the FFI path (unmeasured here).
