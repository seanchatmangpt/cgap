# Lane RC-04 — keyed attestation MAC (RED-18 §3.3 clone vector, honest scope)

Subject: /Users/sac/cgap @ main, files `crates/cgap-p28/src/attestation.rs`
(owned, append-only section) and NEW
`crates/cgap-p28/tests/attestation_mac_court.rs`. No git commands run.

## W1 Identity / diff

- `crates/cgap-p28/src/attestation.rs`: +145/−5 (formatter-normalized), a
  single appended RC-04 section; existing 80-B layout, codec, validate, ELF/
  WASM carriers, and courts untouched. Keyed MAC layer on the in-repo
  keyed-BLAKE3 engine `cgap_crypto::chatman::keyed_digest` (BLAKE3-spec
  keyed mode, KAT-pinned by cgap-crypto's own courts). No new deps, no
  new crypto code: zero-dep law honored via the existing
  `[dependencies] cgap-crypto` line (A18's disclosed deviation precedent).
- API: `MAC_DOMAIN`, `ATTESTATION_MAC_LEN=48`,
  `AttestationMac { key_id: [u8;16], tag: [u8;32] }` (+48-B wire codec,
  typed refuse on non-48), `attestation_mac(key, key_id, att)`,
  `verify_with_key(key, key_id, att, mac) -> Result<(), MacMismatch>`
  (constant-time tag compare, opaque `MacMismatch` per the §5
  non-fingerprinting law). `mac_subkey = keyed(domain || key_id, device_key)`,
  `tag = keyed(mac_subkey, attestation wire bytes)`. The key never appears on
  disk in the record or the MAC wire; `key_id` is a selector into an
  out-of-band key store only.
- HONEST SCOPE (in doc comments at the RC-04 section header): closes the
  clone vector ONLY when the device key lives outside flash (secure element /
  OTP / host secret). Designed-in software seam, NOT a hardware-security
  claim; an attacker who reads the key defeats it trivially. trap.rs was NOT
  touched (not owned); `verify_with_key` is the exposed seam for whoever owns
  the loader path.

## W2 Commands / exits (this session)

- `CARGO_TARGET_DIR=/tmp/rc04-target cargo test -p cgap-p28` — exit 0;
  38 lib + 4 attestation_mac_court + 2 lever + 6 invariant + 0 doc = 50
  passed, 0 failed (counts re-read from the run output above).
- `cargo build --workspace` — FAILED at `cgap-ffi` E0277
  (`crates/cgap-ffi/src/stats.rs:241` `Atomic<u64>: Copy`). NOT my diff:
  `git diff HEAD --stat -- crates/cgap-ffi/src/stats.rs` is empty (file
  unmodified from HEAD), stats.rs is not in any lane's owned set I can see,
  and no rc04 file can produce an ffi trait-bound error. Recorded as a
  pre-existing red workspace member, left to the owning lane (compile-freeze
  SLA disclosure: stats.rs is on the cgap-ffi surface where lane-fix07 has
  in-flight modified files receipt_chain.rs/sovereign.rs — the owner
  supersedes; I did NOT edit it).
- Sandbox court runs — see W4.

## W3 Courts

`tests/attestation_mac_court.rs`, 4 tests:
1. `valid_key_admits_and_is_deterministic` — positive control; deterministic
   re-derivation; different key => different tag; wrong key with a valid
   tag refuses typed.
2. `tampered_and_cloned_refuse_typed` — cloned (att+mac, wrong key) refuses;
   wrong key_id refuses; full 640-bit single-bit sweep over the 80-B body:
   all refuse, count pinned; full 384-bit sweep over the 48-B MAC wire: all
   refuse, count pinned.
3. `key_never_in_mac_wire` — no 32-B window of the MAC wire equals the device
   key; 48-B wire round trip; truncated wire refuses typed.
4. `tag_binds_whole_body` — ceiling-field edit changes the tag; its own MAC
   verifies; old MAC refuses.

## W4 Anti-vacuity mutation battery (/tmp sandbox, sha256-exact restore)

Sandbox: /tmp/rc04-sb (standalone manifests, deleted at lane end).
Baseline: MAC court 4/4 green under identical manifests.

- SHIM 1: `mac_subkey` derives with the zero key instead of the device key.
  Result: court FAILED 2/4 (`valid_key_admits_and_is_deterministic`,
  `tampered_and_cloned_refuse_typed`) — court flips. RESTORE: sha256-exact
  (`0137bce2…860e` OK), mtime bumped, rerun 50/50 green.
- SHIM 2: `verify_with_key` skips the constant-time tag compare (always Ok).
  Result: court FAILED 3/4 — court flips. (First SHIM-2 attempt reported an
  all-pass: the mutation had not persisted to disk when cargo ran — the
  stale-binary/mtime hazard from `a02-finish-receipt.md:170-184`. The flip
  above is from the verified on-disk mutation; the all-pass run is recorded
  here as a vacuous run, superseded.)
  RESTORE: sha256-exact OK, mtime bumped, rerun: 38+4+2+6 green, exit 0.
- Zero skips. No [SKIP] emitted.

## W5 Fences run

- `cargo test -p cgap-p28` (the crate gate) — exit 0, run twice post-restore.
- `cargo build --workspace` — attempted, blocked by the pre-existing
  cgap-ffi stats.rs E0277 above (W2). Fence PARTIAL: every workspace member
  but cgap-ffi compiled (error output shows only the ffi failure); cgap-p28
  and its two downstream test binaries are green.
- Root lib.rs/Cargo.toml untouched (off limits).

## W6 Standing

**ALIVE** for the lane surface (crate gate exit 0 this session, commands +
exits above). Root workspace build = PARTIAL / pre-existing ffi break
outside lane ownership.

## W7 Divergences

1. `cargo build --workspace` red at cgap-ffi `stats.rs:241` — pre-existing
   (unmodified from HEAD), not lane-introduced; disclosed for the owning
   lane (fix07-ffi-surface surface), no unblock applied (not an in-flight
   compile-freeze case; a committed break is outside my repair authority).
2. SHIM-2 first run was vacuous (stale binary, mutation not on disk);
   superseded by the verified flip in W4. Recorded, not hidden.
3. cgap-crypto's `keyed_digest` is single-chunk (≤1024 B) — the MAC inputs
   (42 B and 80 B) are far under the bound; documented in the source, not a
   waiver.
