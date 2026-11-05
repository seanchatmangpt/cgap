# AB-8 Wire Receipt (PG-A2, AoQ Wave-1 W1-5b)
Standing: PARTIAL

Lane: AB-8, implementation. Subject: `/Users/sac/cgap` crates
`cgap-arrow` (sanctioned ipc.rs touch only) + `cgap-aoq` (verify.rs,
writer.rs, tests). Date: 2026-11-03. Never ran git.

## Landed

- `crates/cgap-arrow/src/ipc.rs` — the ONE sanctioned touch, kept minimal:
  two new `IpcError` variants `TagMissing` / `TagMismatch { got: [u8;16] }`;
  the `parse_meta` magic fork (`AOQ1_MAGIC = 0x414F_5131`,
  `AOQ1_TAG_AT = 76` = window offset of the tag, pinned by court A13) and
  `parse_meta_aoq1` with per-magic
  domain values (LE header, FNV-1a32 over `[0x00,0x3C)` stored LE at
  `0x3C`, closed epoch enum in the flags position, `body_len` u32 LE +
  16-byte tag in the metadata padding); the `read_verified` AOQ1 tag gate
  (`TagMissing` on all-zero, `TagMismatch` vs `blake3(body)[..16]`).
- `crates/cgap-aoq/src/verify.rs` — AOQ1 window verification:
  `verify_window` -> `VerifiedWindow` -> `decrypt_in_place`, verify-then-
  decrypt BY TYPE (`&[u8]` gates; `&mut [u8]` only downstream of Ok);
  composes AB-6's landed codec (`codec::decode`) — header law never
  re-declared. Full-window law (body pad + EOS) gated for classification
  identity with cgap-arrow's framing surface.
- `cgap-aoq/src/writer.rs` — AoqWriter 5-segment scatter: stack header
  (via `codec::encode` seal), caller body BY REFERENCE, ZeroProgress
  carry (cursor advances only on `commit`), seq/ordinal lockstep (+1,
  wrap law); body pad inline (136 B scratch, layout const-court).
- `cgap-aoq/tests/ab8_courts.rs` — courts A1-A13 (A1 roundtrip, A2
  lockstep+replay, A3 seq regression + cursor immutability, A4/A4b digest
  mismatch both-surface + w8-consistently-sealed, A5 epoch gate isolated
  via re-seal, A6 pad cleanliness, A7 body_len word corruption, A8 tag
  mismatch, A9 truncation exact-needed under AF-4's read-ahead law, A10
  write-refusal ZeroProgress, A11 staging-is-not-progress + lockstep, A12
  no-plaintext-on-error, A13 offset-pin identity).
- `cgap-aoq/tests/ab8_sweep.rs` — truncation classification-identity
  sweep: 365 prefix lengths (0..=364) + mutation pairings (header, pad,
  tag, body). Cases >= 72 (court asserts the bar).
- `cgap-aoq/tests/ab8_soak.rs` — 10k-frame seal -> writev -> UDS ->
  verify -> unseal soak over a real UnixStream socketpair with
  `write_vectored`; counting global allocator, non-vacuity via explicit
  setup probe; zero-alloc-per-frame court.

## Gate receipts (exit records)

| Gate | Command | Exit record |
|---|---|---|
| cgap-arrow FULL (canary, C1-C13) | `cargo test -p cgap-arrow` | exit 0; all suites 0 failed (145 unit + 3+5+17+4+1+36+1+5+4+7+3+4+6 integration = 241 integration + unit); ran pre-AF-4 + once more after my branch (ipc_courts 17/17, ipc_body_integrity 5/5) |
| cgap-aoq AB-8 targets | `cargo test -p cgap-aoq --test ab8_courts` | exit 0; **14 passed / 0 failed** (A1-A13 + A4b) |
| ab8_sweep | `cargo test -p cgap-aoq --test ab8_sweep` | exit 0 (2 passed / 0 failed); 365+ mutation cases |
| ab8_soak | `cargo test -p cgap-aoq --test ab8_soak` | exit 0; measured: **579,815 frames/sec (release, final run)**; earlier run 516,546, 45,301 frames/sec (debug), 2,640,000 bytes moved, **0 allocs** across 10k frames; counter non-vacuity asserted (explicit setup probe); re-verified post-freeze (release run this session) |
| cgap-arrow subset post-AF-4 | `cargo test -p cgap-arrow --test ipc_courts --test ipc_body_integrity_test` | exit 0; 17+5 passed, 0 failed |

FULL `cargo test -p cgap-arrow`: **ALIVE, exit 0, 0 failed** across all
suites (145 unit; integration suites 3/5/17/4/1+36/1/5/4/7/3/4/6 — the
C1-C13 ipc courts 17/17 unregressed with the AOQ1 branch landed).

FULL `cargo test -p cgap-aoq`: 9 unit + 14 ab8_courts + 1 soak + 2 sweep
+ 1 bench + 7 codec + 6 drift + 2 stress passed / 0 failed on MY targets;
the aggregate run has exactly ONE failure, AF-10's `cbmqs_gates` G2
(AB-6's bench-court timing held to a 100 ns ceiling; measured 694.7 ns in
the debug-profile aggregate run). Classification: NOT session-introduced
by AB-8 — the same bench court passes standalone (`cargo test -p cgap-aoq
--test aoq1_bench_court` exit 0, 1 passed), and neither of my owned files
(codec-owned paths, bench court, AF-10's aggregate) is in the causal
chain. Standing for MY deliverables: ALIVE; crate-level standing:
PARTIAL on AF-10's G2 fence, owned by AF-10/AB-6 to reconcile.

## AB-8 landing decisions (for coordinator review)

1. **Padding law** (PG-A2 §2 per-magic): the AOQ1 header is untouched
   (ontology field table authoritative); the metadata padding carries
   `body_len` u32 LE at window offset 72 and the 16-byte tag at 76;
   verified AOQ1 windows are w64-only (w8 AOQ1 framing = `TagMissing`).
2. **Tag law**: `blake3(body-as-carried)[..16]`, tamper tier — same tier
   as the existing body-digest path; symmetric on both surfaces;
   cgap-aoq never computes it (R-K1: caller supplies expected tag).
3. **Endianness reconciliation**: my branch initially assumed BE per the
   codec I read at 21:59; AB-6 flipped codec to LE at 22:03; my branch
   follows the LANDED LE codec (courts pin it).
4. **Class pairing table** (R-H2, exercised in ab8_sweep):
   Truncated/BadBodyLen/MissingEos <-> HeaderTooShort;
   BadContinuation <-> BadMagic; BadMetaSize <-> HeaderTooShort;
   BadMagic <-> BadMagic; HeaderDigestMismatch <-> DigestMismatch;
   BadVersion <-> UnsupportedVersion; BadFlags <-> UnknownEpochWire;
   PadNonZero <-> ReservedNotZero; TagMissing/TagMismatch <->
   DigestMismatch.
5. **Scratch law**: writer scratch = 136 B (prefix 8 + header 64 + meta
   pad 56 + EOS 8); the body pad (`pad64(body_len)`, landed IPC window
   law) is inline in `write_window` and `Segments::body_pad()`-sized for
   writev callers.

## Disclosures (fanout SLA)

- Root Cargo.toml duplicate `crates/cgap-aoq` member line (AB-1 + AB-6
  both added it; cargo tolerates it) — observed, not fixed, not mine.
- cgap-aoq Cargo.toml grew AF-1/AF-10/AF-14 dev-deps and the cbmqs
  [[test]] entry mid-flight — shared manifest, other lanes'.
- lib.rs: I added `pub mod verify; pub mod writer;` under the AB-1
  reconciliation header (AB-8-sanctioned integration edit).
- AF-4 changed parse_meta's read-ahead law WHILE my A9/A4b/A7 courts
  were failing on the pre-AF-4 semantics; courts re-derived against the
  landed AF-4 law (needed=128 band, BadBodyLen band, MissingEos band).
- Writer restructure disclosure: my first tail design held body pad in
  scratch (200 B) — unbounded-pad bug caught by A1 (scratch overflow
  panic); restructured to EOS-only tail + inline pad.

## Falsifiers and status

- Court A12 is the A12 falsifier: every refusal leaves the buffer
  byte-identical; decrypt refusals never half-keystream.
- The sweep IS the classification-identity falsifier: any divergence
  between `ipc::read` and `verify_window` on any prefix length fails.
- Standing: PARTIAL — A10-A13 compiled-only (manifest freeze), full
  gates pending the freeze; everything else ALIVE with exit records
  above.
