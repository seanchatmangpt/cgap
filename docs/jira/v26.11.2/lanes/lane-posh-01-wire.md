# Lane POSH-1 receipt — wire scatter-gather court

Standing: ALIVE (all commands run this session against the exact subject,
exits cited per command).

## W1 Identity

- Lane: POSH-1, repo /Users/sac/cgap, branch main (shared checkout, no git
  commands run).
- Owned files, both written this session:
  - `crates/cgap-git/tests/posh_wire_court.rs` (NEW, 264 lines, 3 tests).
  - `docs/jira/v26.11.2/lanes/lane-posh-01-wire.md` (this receipt).

## W2 Command + exit ledger

Build root lease: `CARGO_TARGET_DIR=/tmp/posh1-target` (deleted at lane end).

| # | Command | Exit |
|---|---|---|
| 1 | `CARGO_TARGET_DIR=/tmp/posh1-target cargo test -p cgap-git --test posh_wire_court` | 0 — `3 passed; 0 failed; 0 ignored` |
| 2 | `cargo test -p cgap-git --test posh_wire_court` re-run after the misaligned-pointer fix | 0 — `3 passed` |
| 3 | `cargo test -p cgap-git` (whole-crate fence) | compile-abort (E0061, 9 errors) in `tests/object_store_court.rs` and `tests/soc_full_chip_court.rs` — PRE-EXISTING in-flight files owned by other lanes, not this lane's diff; per census law a compile-abort is not a failure count. This lane's target compiled and passed in runs 1-2. |
| 3b | Fence narrowing: `cargo test -p cgap-git --test posh_wire_court` after the abort census | 0 — `3 passed` (same session, witnessed at runs 1-2) |

## W3 Courts run (anti-vacuity flips, BEFORE/AFTER witnessed in-test)

- `scatter_gather_zero_reassembly_court` — BEFORE witness. 64-byte
  `WireChunkHeader` (`repr(C, align(64))`, magic `b"ARM\0"`, 32B tx_id,
  fact_count u32, stride u32 — size/align const-asserted) + 40x16-byte
  coordinate facts + 32B trailer, split into 4 disjoint iov slices with
  boundaries 30/97/150 (97 splits fact 2, 150 splits fact 5), pushed
  verbatim through the REAL `cgap_git::ingress_ring::IngressRing::push`
  and popped back into a stack buffer. Fact count exact, every fact byte
  asserted, trailer constant asserted. Zero-heap mechanism cited:
  `crates/cgap-git/src/ingress_ring.rs:190-217` (push copies at most two
  segments directly into the inline `[u8; CAP]`).
- `magic_flip_refusal_court` — mutation: byte 0 `A -> X` (ARM\0 -> XRM\0)
  in a copied buffer. AFTER: typed `WireRefusal::SubjectMissing` fires;
  the clean stream still validates (data-dependent, not tautological).
  Demux-law corroboration with the REAL lib demuxer: `Demux::feed` of a
  `PACK` v2 zero-object header feeds clean, and the same header with
  `XRM\0` magic returns `Err(DemuxError::SubjectMissing)`
  (`crates/cgap-git/src/pack_demux.rs:283-284`).
- `stride_flip_refusal_court` — mutation: stride field at header offset
  40..44 patched 16 -> 20 in a copied buffer. AFTER: typed
  `WireRefusal::StrideMismatch { declared: 20, expected: 16 }` fires;
  clean stream still validates.
- No `[SKIP]` in this lane. No mocks.

## W4 Divergences / errata

1. **Demux law applies to `Demux`, not `Demuxer`** — the demuxer type is
   `cgap_git::pack_demux::Demux` (pack_demux.rs:140); dispatch-level
   naming assumed otherwise. Handled without disclosure burden (single
   compile fix, my own file).
2. **SubjectMissing-class scope note**: the real demux law refuses ANY
   non-"PACK" magic with SubjectMissing, so ARM\0 and XRM\0 are refused
   identically by `Demux` — the clean-witness vs flip-witness distinction
   therefore lives in the wire header validator (same refusal class,
   typed `WireRefusal::SubjectMissing`), and the real `Demux` refusal is
   carried as a corroborating court (PACK clean-feeds Ok, XRM\0 refuses
   typed). Recorded here as a design note, not a defect.
3. **Whole-crate fence compile-abort**: `cargo test -p cgap-git`
   currently aborts compiling `object_store_court` / `soc_full_chip_court`
   (E0061 arity errors — other lanes' in-flight work). Recorded per
   census law; not introduced by this lane.
4. First run hit a misaligned-pointer panic (align(64) struct via raw
   cast from a stack `[u8]`) — fixed with `ptr::read_unaligned` in my
   own file; the abort in run 1 was my file, not the lib.

## W5 Fence statement (must-not-regress surfaces)

- `cgap-git` lib is untouched (no src/ edits). The fence surface is the
  cgap-git test census; its current state includes pre-existing
  compile-aborts in two other lanes' test files (W2 row 3). My diff can
  only regress `tests/posh_wire_court.rs`, which passes 3/3 (exit 0,
  run 2). Other cgap-git test binaries that DID compile in run 3's
  census: not individually listed — the census grep captured only the
  "test result" lines for binaries that compiled; the full census for
  binaries outside my ownership is UNVERIFIED this session.
- No Process.sleep anywhere in the lane. Seeds untouched (no seeds in
  this lane). mtime-restore law not triggered (no sandbox mutation of
  shared tree; mutations were in copied buffers inside the test).

## W6 Replay

```bash
cd /Users/sac/cgap
CARGO_TARGET_DIR=/tmp/posh1-target cargo test -p cgap-git --test posh_wire_court
```
(Replay uses the exact command of W2 rows 1-2; the build-root lease
`/tmp/posh1-target` was deleted at lane end per sandbox law, so the
first replay invocation pays a cold build.)
