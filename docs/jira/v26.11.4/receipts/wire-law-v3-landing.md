# Wire Law v3 Landing — IMPLEMENTATION Lane (Stage 2)

Date: 2026-11-04. Lane: v3 wire law Stage 2 (AEAD emit/open + version bump
2→3 + AeadMismatch unblock). Repos: /Users/sac/cgap +
/Users/sac/ggen-marketplace (aoq-pack).

## Ruling citation

- R-K1: crypto bodies live in cgap-crypto; cgap-aoq hosts the facade.
- AAD ruling Option b (see decision-aad-ruling.md; F1 commit 83207531
  "var-AAD promotion (AAD ruling Option b)"): the RFC 8439 mac_data layout
  authenticates variable-length AAD; the pinned `AAD_LEN = 36` wrapper no
  longer binds the session layer.
- R-1 LE ruling: seq is little-endian; preserved in the v3 tail.
- WAIT-GATE re-derivation: `grep -c 'doc(hidden)'
  crates/cgap-crypto/src/aead.rs` = 0 at lane start (gate satisfied; F1
  promotion re-verified in commit 83207531).

## Per-stage diffs

1. Ontology bump (ggen-marketplace, packs/aoq-pack):
   - ontology.ttl: `aoq:hf-version aoq:value 2 -> 3` (single line; a sed
     over-replacement of epoch-rotated-2/stream-eos was caught in the same
     session diff and reverted — graph diff is one line).
   - templates/frame_rs.tmpl `AOQ1_VERSION 2 -> 3`;
     templates/header_h.tmpl `#define CGAP_AOQ1_VERSION 3`.
   - crates/cgap-aoq/src/generated_aoq_header_layout.rs `AOQ1_VERSION: u16
     = 3` (the layout file is a checked-in projection; repo sync writes
     only include/cgap_aoq.h, so the frozen output was updated to match the
     graph + template).
2. AEAD facade (cgap-aoq/src/aead.rs): const-refusal stubs replaced by
   delegation to cgap-crypto var-AAD seal/open; typed mapping
   Mismatch -> AoqError::AeadMismatch; `verify_detached` added (delegates
   to the new kernel `verify_tag_var_aad`, a plaintext-free folded
   constant-time tag check). The M2a lane's interim stub re-addition was
   superseded once no caller remained (disclosed both sides).
3. session.rs emit tail rewrite (v3 wire law):
   `[header 64B v3][kind 2][seq LE 4B][ct][tag 16B]`; sealed under
   send_key, nonce `Nonce96::new(epoch as u32, (ordinal<<32)|seq)`,
   AAD = covered span `header ‖ kind ‖ seq` (buf[..69]); 16B Poly1305 tag
   replaces the 4-byte kdf tag. Receive side: `verify_appdata` in the
   step AppData arm (verify BEFORE budget/advance mutation) and
   `open_verified` (open_detached under recv_key). BLOCKED(AAD-LAW)
   comments at the RandomnessSeam docs, AeadMismatch variant docs, and
   the rng field docs: DELETED (unblocked).
4. codec.rs version gate: admits only the generated `AOQ1_VERSION = 3`;
   v2 -> typed `UnsupportedVersion { got: 2 }` (law unchanged in code; the
   gate binds the generated const).
5. chain.rs CHAIN-IS-INTEGRITY: `append_verified` links the whole wire
   frame under the chain's OWN hash — no wire-tag re-derivation (the
   chain holds no AEAD keys); min-length floor 64+5+16; tamper surfaces
   at the chain walk (verify_all/verify_span/Spoliation).
6. Consumer updates (disclosed): cgap-arrow/src/ipc.rs AOQ1 version gate
   2->3 (v3 must be admitted by the IPC1-prefix bridge); cgap-ffi
   aoq_session.rs / aoq_sidecar.rs and tests session_courts /
   chain_courts / macro_defense_mesh / aoq1_codec_court /
   aoq1_layout_drift_court updated to the v3 law (several concurrently
   with the M2a confirm-tag lane — collisions resolved by reading both
   sides; the M2a stub re-addition superseded with disclosure).
7. cgap-aoq Cargo.toml: cgap-crypto promoted dev-dep -> real dep (the
   facade delegation); cgap-aoq is no longer zero-dependency — see
   decision-d2-zero-dep-reinterpretation.md for the standing
   reinterpretation context; the default build remains core-only
   (cgap-crypto is no_std).

## Courts (all real-collaborator, named mutations)

- Emit→open roundtrip (paired role-swapped sessions, real facade keys):
  `session::tests::v3_emit_open_roundtrip`.
- Nonce determinism + uniqueness (same epoch/ordinal/seq = same nonce;
  ordinal/seq both bind): `session::tests::nonce_determinism_and_uniqueness`.
- Anti-vacuity battery: flip one ciphertext byte -> AeadMismatch with
  fingerprint byte-identical; flip one AAD byte -> AeadMismatch; step-path
  refusal no-mutation:
  `session::tests::tamper_battery_ct_and_aad_refused_no_mutation`.
- v2 refusal + fingerprint unchanged: `session::tests::v2_header_refused_typed`
  (+ codec law unchanged: UnsupportedVersion{got:2} typed).
- Chain-is-integrity: session inline court + chain_courts c7/c9: tampered
  wire frame appends, lands a different record hash, verify_all still Ok;
  short frame refuses typed HeaderTooShort with no mutation.
- Existing suites re-pinned to the v3 law: ab8_courts, session_courts
  (S1-S8, incl. 10k-frame ordinal-wrap soak), aoq1_codec_court,
  aoq1_stress, macro_* stress suites, purity_gate (source scan: green
  with the delegation facade).

## Commands + exits (this session, exact subject)

- `cargo test -p cgap-aoq` -> rc=0; 28 test binaries, 130 passed / 0
  failed (re-run after all concurrent lanes settled).
- `cargo build -p cgap-ffi` -> rc=0; `cargo build -p cgap-ffi --tests` ->
  rc=0 (bridge consumers compile against the promoted signatures).
- `cargo test -p cgap-crypto --test aead_courts` -> rc=0, 9 passed / 0
  failed.
- `make gate0` -> PASS: ggen sync x2 byte-identical replay
  (5 surfaces hashed and cmp'd), second sync `written: []`, freestanding
  C header compile PASS. FM-PACK-008 checksum gate fired on first sync
  (expected: pack content changed); remediation `rm ggen.lock && ggen
  sync run --skip-ea-admission` x2 applied.
- Drift court: `aoq1_layout_drift_court` green (VERSION pins 3).
- Purity gate: green (lib source scan, delegation facade registered).

## Standing

- ALIVE: every court named above ran green this session against the exact
  subjects (commands + rc above).
- CONCURRENT-LANE NOTE: session.rs, session_courts.rs, chain_courts.rs,
  and the aoq-pack header_h.tmpl carry interleaved M2a (confirm-tag)
  lane edits landed mid-flight in the same files; both lanes' work is
  present and green together. The coordinator folds commits; lane did
  not run git write commands (no stash, no add, no commit).
