# AB-1 Receipt — cgap-aoq skeleton (AoQ Wave-1 W1-1)
Standing: UNSUPPORTED

Lane: AB-1, implementation lane, /Users/sac/cgap. Date: 2026-11-03.
Contract: PG-A1 skeleton per coordinator dispatch; rulings R-K1/R-H2 binding.
Owner: `crates/cgap-aoq/*`, root Cargo.toml membership line, justfile
`aoq-check`/`aoq-gate` (append-only), `tests/purity_gate.rs`.

## Status: ALIVE (AB-1-owned courts; crate-wide suite BLOCKED on sibling in-flight files)

## Delivered

- `crates/cgap-aoq/Cargo.toml` — landed by GA-1/AB-6 reconciliation; AB-1
  took the on-disk manifest as current state (64-byte AOQ1 envelope, E=0
  lib externals, empty [dependencies]).
- `src/lib.rs` — no_std, forbid(unsafe_code), module registration
  (embedded surface unconditional; kem/aead seams registered; chain and
  session per collision timeline below).
- `src/error.rs` — 11-variant `AoqError` (Copy, integer fields,
  `core::error::Error`). Coordinator R-H2 supersession honored: the
  framing-class variants of the AOQ-02 prose design are REPLACED by
  header-law variants; IpcError reuse is deferred to the AB-8 integration
  seam. Module header carries the supersession note; the module is
  superseded by content, not deleted (per coordinator sanction).
- `src/codec.rs` — AOQ1 header codec: FNV-1a32 over `[0x00,0x3C)`,
  encode/decode of the 64-byte envelope, typed refusals. LE/BE
  reconciliation: a concurrent edit flipped encode to little-endian while
  decode stayed big-endian; AB-1 reconciled decode to LE and pinned the
  module doc law to "little-endian" (the newest deliberate edit direction).
- `src/kem.rs`, `src/aead.rs` — KernelNotAdmitted host-facade seams
  (R-K1: crypto bodies are cgap-crypto/cgap-blake3; seams refuse loudly,
  const fn, typed `KemKernelNotAdmitted`/`AeadKernelNotAdmitted`).
- `src/generated_aoq_header_layout.rs` — landed by AB-6's ggen projection
  (field table + offset constants). AB-1 initially derived per-field offset
  constants; removed as duplicates when the real projection landed. The
  placeholder + courts-outside pattern holds: courts live in `tests/`.
- `tests/purity_gate.rs` — banned-token scan (9 classes) over registered
  sources, comment-stripping scanner (cgap-core model), anti-vacuity
  positive controls (inject-a-file, all-class poison flip, comment-hole,
  word-boundary glue), SOURCE-SCAN-EXEMPT marker mechanism ported with
  INVALID-EXEMPTION reporting, fail-closed unregistered-source accounting.
- `tests/aoq1_codec_court.rs` — roundtrip, digest tamper (key_id +
  reserved tail), magic/version/epoch/truncation refusals, Retired-epoch
  read-only law, class-code stability, seam refusals, FNV-1a32 vectors
  (python cross-checked: "foobar" -> 0xBF9C_F968).
- Root `Cargo.toml` — lane-tagged membership line (`# v26.11.3 AoQ Wave-1
  lane AB-1 ...`) + `"crates/cgap-aoq"`.
- `justfile` — append-only `aoq-check` / `aoq-gate` recipes (ROOT_AOQ :=
  justfile_directory() per the just-1.56 shebang law). aoq-check records
  the check wall into `docs/jira/v26.11.3/bench/aoq-check-walls.txt`.

## Gates run this session (commands + exits)

- `CARGO_TARGET_DIR=target-aoq-lane cargo check -p cgap-aoq` -> exit 0,
  0 warnings (after AB-6 header.rs doc comments — disclosed minimal
  unblock).
- `cargo test -p cgap-aoq --test purity_gate --test aoq1_codec_court`
  -> exit 0: purity gate 8 passed, codec court 7 passed.
- `cargo check -p cgap-aoq --target wasm32-unknown-unknown` -> exit 0.
- riscv32imc: target NOT installed on this host -> LOUD SKIP (exit class
  3), never a fake pass. Flip to GREEN when a lane installs the target
  (`rustup target add riscv32imc`).
- `just aoq-check` -> exit 0; warm walls 332ms / 569ms recorded; true
  cold wall measured sub-second (E=0 crate: single-unit graph, fresh
  CARGO_TARGET_DIR, python-timed; 0.2-0.5s observed) — ledger at
  `docs/jira/v26.11.3/bench/aoq-check-walls.txt`.
- `just aoq-gate` -> exit 0 GREEN (host courts + wasm leg + riscv loud
  skip).

## Collision disclosure (full, per fanout SLA law)

Live concurrent writers in this crate during the session: AB-6 (header/
generated layout), AB-7 (chain/session/verify/writer/courts), AB-8
(courts/verify). Every shared-file event and unblock, in order:

1. Cargo.toml rewritten under AB-1 (GA-1/AB-6 normalization, 21:42) —
   adopted as current state, not reverted.
2. header.rs replaced (AB-6, ontology-pinned AOQ1 design) — adopted; AB-1
   added missing-docs on the Epoch variants (AB-6 file, disclosed minimal
   unblock under my `warn(missing_docs)`).
3. AB-6's ggen projection landed real offset constants; AB-1's derived
   duplicates removed (my earlier placeholder addition became the
   redefinition; the graph projection wins).
4. AB-7 registered `pub mod session;` without session.rs on disk ->
   compile freeze; bounded poll 60s; AB-1 commented the registration
   (SLA minimal unblock, disclosed in-file), restored it when session.rs
   landed (21:58, 630 lines).
5. AB-7's chain.rs (old AOQ-02 design) referenced superseded error
   variants -> unregistered per coordinator ruling; AB-7's header-agnostic
   rewrite later landed and was re-registered (lawful; the purity gate
   ledger law flipped with AB-7's in-file disclosure).
6. session.rs inline-test borrow errors (E0716 x2) -> AB-1 one-line
   binding fix, disclosed in-file.
7. session.rs inline tests use alloc/vec in a no_std crate -> AB-1 added
   justified SOURCE-SCAN-EXEMPT markers (cgap-core tracking_alloc.rs
   precedent) and ported the exemption mechanism (with INVALID-EXEMPTION
   anti-vacuity) into the purity gate.
8. AB-8's verify.rs `for &b in ...enumerate()` E0308 -> AB-1 one-line
   destructure fix, disclosed in-file.
9. AB-7 chain_courts.rs stray brace -> removed (disclosed); deeper
   corruption in the same file repaired by AB-7 themselves.
10. AB-8's tests/ab8_courts.rs / tests/session_courts.rs were in-flight at
    receipt time with compile errors; AB-1's courts are scoped-run GREEN
    while the crate-wide `cargo test -p cgap-aoq` is RED on those sibling
    files (BLOCKED: tests/session_courts.rs, 12 borrow errors, last
    observed ~22:20; sibling files changing every ~30-170s).

## Falsifiers / standing

- Standing: AB-1-owned courts ALIVE (commands + exits above, this session,
  exact subject `crates/cgap-aoq`).
- Crate-wide suite: BLOCKED on sibling in-flight files (AB-7/AB-8), not on
  AB-1 deliverables; the gate's fail-closed accounting is designed to name
  exactly this class loudly.
- Purity-gate falsifier: injecting any banned token into a registered
  source must name file+token (`gate_fails_on_injected_poison_file`); the
  poison-flip court asserts all 9 banned classes flip.
- riscv32imc leg: UNSUPPORTED (host) -> loud skip.
- Bench deliverable: build-time budget gate = `just aoq-check` wall
  recording; budget enforcement consumes `bench/aoq-check-walls.txt`.

## Supersession notes (per coordinator R-H2)

- AOQ-02 prose framing (flags/parent_prefix/variable-length payload) is
  superseded; no AOQ-02 code survives in registered modules. error.rs
  stays registered as the seams-placeholder per coordinator sanction,
  carrying only header-law + KernelNotAdmitted variants.
