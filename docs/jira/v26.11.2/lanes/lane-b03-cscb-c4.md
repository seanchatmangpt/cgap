# Lane B03 — CSCB C4 steady-state zero-allocation court

Standing: **ALIVE** (both modes ran this session, per-command exits below).

## Files written
- `crates/csct/src/bin/cscb_c4_alloc.rs` (only lane file; `crates/csct/Cargo.toml` dep
  was pre-landed by coordinator, not edited by this lane)

## Commands + exits (this session, 2026-11-02)

```
cd crates/csct && CARGO_TARGET_DIR=/tmp/b03-target cargo build --release --bin cscb_c4_alloc
  -> exit 0 (one dead_code warning on cgap-core's reset_breach, imported wholesale
     by #[path]; harmless, source untouched)
/tmp/b03-target/release/cscb_c4_alloc                     -> exit 0
  cscb_c4: corpus=self-gen frames=1024 frame_w=64 mutate=false
  C4 PASS: steady-state 0 allocs / 0 bytes over 1000000 frames
CSCB_MUTATE=alloc /tmp/b03-target/release/cscb_c4_alloc   -> exit 1
  ALLOC BUDGET BREACH: site="cscb_c4_ingress" allocs=1 bytes=8 offending=8
  C4 MUTATE: breach observed site="cscb_c4_ingress" allocs=1 bytes=8 offending=8 -> exit 1
```

## Court semantics
- `#[global_allocator]` = `EnclaveTrackingAlloc<System>`; window armed via
  `start_hot_path_budget("cscb_c4_ingress", 0, 0)` around the full 1,000,000-frame
  ingress pass (fold closure step over cyclic 1,024-frame corpus window).
- Clean mode asserts `window_counts() == (0,0)` AND `last_breach().is_none()`; exit 0.
- Anti-vacuity (`CSCB_MUTATE=alloc`): one `Box::new(42u64)` injected at frame 0.
  The budget breach mechanism is the **recorded breach** (interceptor disarms +
  serves the allocation; it does NOT panic — panicking inside a GlobalAlloc
  aborts the process per tracking_alloc.rs BREACH PATH note). Court reads
  `last_breach()`, observes allocs=1 bytes=8, exits 1. Flip witnessed.
- Mutate-mode null path: no breach observed => `C4 VACUOUS` exit 1 (court refuses
  to fake a pass).

## Divergences / disclosures
1. **Corpus fixture absent** (`crates/csct/fixtures/corpus.bin`, B08 in flight):
   `load_corpus()` self-generates deterministically from the frozen seeds
   (SEED `0xBF58_476D_1CE4_E5B9`, CORPUS_CONST `0xA20C_0FFE_E5A7_E000`), same
   fallback rule as sibling lanes. Fixture presence is auto-detected
   (64-byte frame chunks); when B08 lands, the court consumes it without edit.
2. **`#[path]` include, not a dependency use**: cgap-core's `lib.rs` does not
   export `tracking_alloc` (lib.rs off-limits to this lane), so the bin
   `#[path]`-includes `crates/cgap-core/src/tracking_alloc.rs` directly —
   single canonical source, zero drift; this is the pattern the module doc
   itself names (cgap-core/src/tracking_alloc.rs:12).
3. Sibling bins `cscb_c1`/`cscb_c3` were NOT landed at write time — loop is
   this lane's own minimal ingress fold (dispatch pre-authorized this).
4. Debug-mode runs were NOT executed as pass evidence (dispatch: debug reports
   typed lines, not fake passes). Release is the court's claim surface; the
   debug profile would count `Vec`/format machinery only if it leaked into the
   window — it cannot, since corpus build and all printing occur outside the
   armed window on both paths.
5. Tracking allocator counts only owner-thread (stack-band) allocations;
   single-threaded main driver, so full coverage.

## Fences could break / checked
- cgap-core source untouched (read-only `#[path]` include; mtime of
  tracking_alloc.rs unchanged — no cargo fingerprint hazard).
- No shared lib.rs / Cargo.toml edits; no git commands; build root
  `/tmp/b03-target` is a lane lease, deleted at lane end.

## Cleanup
- `python3 -c "import shutil; shutil.rmtree('/tmp/b03-target', True)"`

## Errata (2026-11-02, CGAP-PRD-05)
- Line 41 "(64-byte frame chunks)" is stale: the landed CSCB fact frame is
  24 B (3x u64 LE, FRAME_BYTES=24; audit/cgapprd-03-frame-layout.md §1-§2).
  Kept verbatim per append-only law; superseded by this note.
