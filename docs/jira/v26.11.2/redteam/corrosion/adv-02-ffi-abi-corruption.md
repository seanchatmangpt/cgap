# ADV-02 — C-ABI symbol & dylib ABI desynchronization (corrosion court)

Standing: ALIVE. Repo /Users/sac/cgap @ main, session 2026-11-02.
READ-ONLY over crates/ and include/; every mutation ran in /tmp against
sandbox copies; the only shared-tree write is this receipt. Zero git
write commands. Build lease: /tmp/adv02-target.

## Referents read first (per dispatch)

- `lanes/lane-xc01-ffi-surface-truth.md` — truth-based header↔nm 1:1
  gate, 12/12 symbols, fresh-build default.
- `lanes/lane-rc05-stats-block.md` — CgasStatsBlock contract: repr(C),
  120 B, align 8, 15 pinned offsets (0..116), FNV-1a-32 CRC over
  bytes [0,112), `cgap_stats_snapshot` extern.
- `redteam/red-03-ffi-shim-decoupling.md` — symbol surface decouples
  in <48h against a weak consumer; semantic surface does not.

## Probe 1 — foreign C client vs fresh dylib (IMMUNE)

Build: `CARGO_TARGET_DIR=/tmp/adv02-target cargo build --release -p
cgap-ffi --manifest-path /Users/sac/cgap/crates/cgap-ffi/Cargo.toml`
→ exit 0 (3.04s), dylib 616,528 B at
/tmp/adv02-target/release/libcgap_ffi.dylib (mtime 18:16, fresh —
never the stale repo-target artifact RED-03 flagged).

Client: /tmp/adv02/client.c — clang, includes
/Users/sac/cgap/include/cgap.h, links the fresh dylib with
`-Wl,-rpath,/tmp/adv02-target/release`. LINK_OK, run exit 0:

- `cgap_version()` → "26.11.2".
- `sizeof(CgasStatsBlock)` = 120, `_Alignof` = 8 — AS SEEN BY C.
- All 16 offsetof probes (magic 0, layout_version 4, ingress_admitted
  8, refused_invalid_magic 16, refused_oabi_mismatch 24,
  refused_p28_spoliation 32, refusals_class_101 40,
  quota_max_refusals 48, quota_window_refusals 52, quota_tripped 56,
  _pad0 60, quota_total_refusals 64, receipts_minted 72,
  last_receipt_digest 80, block_crc 112, _pad1 116) — 0 mismatches
  vs the RC-05 offset court pins.

## Probe 2 — byte-level CRC seal (IMMUNE)

C client poison-fills its own CgasStatsBlock with 0xBF, calls
`cgap_stats_snapshot`, then recomputes FNV-1a-32 INDEPENDENTLY over
its own view of bytes [0,112) and compares to `block_crc`:

- status 0x00000000 (CGAP_OK); magic seen by C = 0x43535442;
  layout_version = 1.
- C-computed CRC = 0x6e3a2c48 == block_crc = 0x6e3a2c48 — the seal
  matches the header-declared coverage window exactly; no offset
  drift between cgap.h decl and dylib reality.
- Null `out` → nonzero typed refusal (no crash, no fake OK).

Cross-language corroboration: the Go client (Probe 3) reads the same
0x6e3a2c48 from an independent process+toolchain.

## Probe 3 — Go cgo client (IMMUNE, compile-tested)

Go 1.23.4 darwin/arm64 present; NOT a static-analysis-only sketch —
really built and run. /tmp/adv02/go/main.go: cgo preamble includes
the same cgap.h, C helpers for sizeof/_Alignof/offsetof (cgo cannot
take unsafe.Offsetof of a C struct directly — compile error witnessed,
exit 1, then fixed), `#cgo LDFLAGS:` placed in the preamble
(preamble-after-import placement also failed linking, exit 1, then
fixed — both failures are part of the hazard record for Go consumers).

Result: GO_BUILD_OK; run exit 0: sizeof 120, align 8, off_crc 112,
off_digest 80, status 0x0, magic 0x43535442, crc 0x6e3a2c48 —
ADV02-GO-CLIENT: IMMUNE.

## Anti-vacuity mutation court (court non-vacuity proof)

Mutation lives in /tmp/adv02/cgap_mut.h (sandbox copy of cgap.h;
shared header untouched — sha256 of canonical cgap.h
c2321d63507af7d32e2d7a0f99d938d430ee59e8142a5ee6cc2215ef22399540
verified unchanged pre/post). Two vacuous mutants were tried and
RETIRED honestly before the flip:

1. drop `_pad0` → clang compile-abort (offsetof of a missing member),
   exit 1 — a compile-abort, NOT a verdict flip; retired.
2. `ingress_admitted` u64→u32 → alignment re-absorbs the 4 bytes
   (next u64 re-aligns to 8); client still exits 0 — vacuous mutant,
   retired. Law: on this struct, 4-byte shrink mutations below an
   8-aligned field are invisible to offsetof/sizeof.
3. `last_receipt_digest[32]`→`[31]` → block_crc still re-aligns to
   112; ALSO vacuous. Retired.
4. `last_receipt_digest[32]`→`[40]` → compiles clean, run exit 1:
   "MISMATCH sizeof: got 128 want 120", "MISMATCH off block_crc:
   got 120 want 112", "MISMATCH off _pad1: got 124 want 116",
   "SPOILER-CONFIRMED (4 mismatches)". FLIP WITNESSED.

## Verdicts

- SPOILER: none. The C-ABI struct surface is synchronized: header
  decl == dylib reality as measured by two independent foreign
  toolchains (clang, cgo). No offset drift, no size/align drift, CRC
  seal recomputes exactly from the C-side byte view.
- Immunity receipts: Probe 1 (16/16 offsets + size + align),
  Probe 2 (CRC seal 0x6e3a2c48 match + typed null refusal),
  Probe 3 (Go view identical, real compile+run).
- Red-03's residual exposure stands UNCHANGED and is confirmed from
  the consumer side: the surface is mirrorable; resistance lives in
  the semantic courts, not the ABI. Nothing in this lane weakened
  that fence.

## Fences audited (must-not-regress)

- FFI surface gate (XC-01, scripts/ffi_surface_gate.py): the header
  was not edited by this lane; gate surface unchanged. Fence status
  as-of XC-01's receipt (12/12, exit 0) — re-read at use time; not
  re-run here because no owned file of that gate was touched.
- RC-05 stats court (tests/stats_block_court.rs): untouched;
  layout pins corroborated EXTERNALLY by this lane's C/Go probes,
  which is a strengthening observation, not a regression surface.
- c_header_abi_court static_asserts: no cgap_frame_view edit.
- Shared tree: `git status -- include/cgap.h` shows M — that diff is
  XC-01's landed pre-existing change (present in the session-start
  status), NOT this lane's; shasum above pins the bytes this lane
  observed.

## Command log (all this session, absolute subjects)

- cargo build --release -p cgap-ffi (CARGO_TARGET_DIR=/tmp/adv02-target) → exit 0.
- clang client build → LINK_OK; ./client → exit 0 IMMUNE.
- mutant builds/runs: pad0-drop → exit 1 compile-abort; u64→u32 →
  exit 0 (vacuous, retired); [31] → exit 0 (vacuous, retired);
  [40] → exit 1 SPOILER-CONFIRMED (flip).
- go build (3 attempts: offsetof misuse, unused import, LDFLAGS
  placement → fixed) → GO_BUILD_OK; run → exit 0 IMMUNE.
- shasum -a 256 include/cgap.h vs sandbox copy → distinct hashes,
  canonical bytes pinned.

## Cleanup + standing

- Build lease /tmp/adv02-target deleted at lane end (below).
- Reproducers retained: /tmp/adv02/{client.c, client, cgap_mut.h,
  mutant_out.txt, go/}.
- ALIVE — every claim above carries a command + exit run this
  session against the exact on-disk subject. No BLOCKED, no
  UNSUPPORTED. No divergences from the dispatch beyond: (a) Go probe
  ran for real rather than falling back to static analysis (stronger
  than dispatched), (b) two vacuous mutants documented rather than
  hidden.
