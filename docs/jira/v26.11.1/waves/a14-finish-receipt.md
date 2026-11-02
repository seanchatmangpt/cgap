# A14 Finish Receipt

- **Lane**: A14 (dam & penstock — `cgap-gate`, `penstock.rs` ingress
  boundary), v26.11.1 — lane-finisher/assessment pass over the landed
  artifact
- **Subject**: repo `/Users/sac/cgap`, branch `main`, HEAD `cfd085a9`;
  lane lease `crates/cgap-gate/src/` CLEAN — `penstock.rs` untouched all
  session (canonical sha256 `709707c4cd496501de72bfd07d3aab990f8ab4ad63f881a3f863abb9f0d32555`
  identical before and after every run; shim mutations existed only under
  `/tmp/a14w9-sb/`)
- **Mode**: assessment first (dispatch: "a wave landed in `9be9109d`;
  ASSESS first"), then finish: `9be9109d` did NOT touch `crates/cgap-gate/`
  (last crate commit remains `62215326`); it DID heavily edit the two
  sibling files the crate's cross-file drift courts read at test time
  (`crates/cgap-ffi/src/sovereign.rs`, +578 lines) — the assessment gate
  was re-running both cross-file courts against the post-wave siblings;
  both held green, no repair needed
- **Verdict**: **ALIVE** — real gate run fresh this session, both
  profiles, exit 0; anti-vacuity flip witnessed in a /tmp sandbox
  (exit 101) with sha256 fail-closed restore

## Gate (commands + exits, this session)

- `CARGO_TARGET_DIR=/tmp/cgap-lane-A14-target cargo test -p cgap-gate`
  -> exit 0, **11 passed / 0 failed / 0 ignored** (58.30 s, dev).
- `CARGO_TARGET_DIR=/tmp/cgap-lane-A14-target cargo test -p cgap-gate
  --release -- --nocapture` -> exit 0, **11/11** (7.18 s, release).
- `cargo test -p cgap-court --test court_verdict a14_dam_ingress_court`
  (same lane target dir) -> exit 0, **1 passed** (9.60 s, 12 filtered).
- Sandbox flip (all under `/tmp/a14w9-sb/`, canonical tree untouched):
  - siblings staged byte-identical (`frame.rs` sha256 `b179d268…8edb`,
    `sovereign.rs` sha256 `825bda4c…48e3`, both equal to canonical);
  - BASELINE `penstock.rs` sha256 `709707c4…32555`; sandbox dam corpus
    -> exit 0 (55.84 s);
  - SHIM (marked `SHIM-COURT`): magic compare removed (lane doc §8.3
    shim 1 — weaken an admission predicate), sha256 `87c0cacd…4a3f`;
    dam corpus -> **FAILED, exit 101**: `frame 15: wrong verdict for its
    manifest class` — the §8.3-predicted misclassification (mutated
    MC-1 magic frames no longer return `CGAP_ERR_INVALID_MAGIC`);
  - RESTORE byte-exact (sha256 `709707c4…32555`, fail-closed); sandbox
    re-run -> exit 0 (55.79 s).
- Concurrent-lane re-witness (same session, later in the wall clock):
  sibling lanes concurrently edited `crates/cgap-core/src/frame.rs`
  (additive `CgapFrameHeader::project` ingress-projection API) and
  `crates/cgap-ffi/src/sovereign.rs` while this lane ran; the pinned
  surfaces are unchanged (the four `CGAP_*` consts byte-identical at
  `sovereign.rs:71-83`; `size_of::<CgapFrameHeader>() == 128` pin still
  present in `frame.rs`), and both cross-file drift courts were
  RE-RUN against the post-edit siblings:
  `cargo test -p cgap-gate -- canonical_layout status_register` ->
  exit 0, 2 passed / 0 failed.

## §8.4 TARGET row (measured this session, release, this host)

- `null-baseline 1.93 ns/frame | full penstock 7.52 ns/frame |
  ratio 3.91x` — `< 4 ns/frame` = **GAP**, `<= 3x baseline` = **GAP**;
  named, not rounded (divergence-8 measuring-court form). The 10x
  gross-regression ceiling PASSes; the exit gate remains the dam court,
  never the nanoseconds (§8.4). Advisory only. Wave-7/8 witnesses
  measured 2.40x/3.97x — run-to-run host variance straddles the 3x
  line; the TARGET rows stay GAP until a sub-3x/sub-4ns receipt exists.

## Mission items -> witnessed surfaces

1. **Penstock/dam ingress, constant-time structural admission,
   drop-before-allocate.** The landed `ingress` (fixed §4 order:
   geometry -> magic/version -> O-ABI coherence -> P28 chain) is
   witnessed by the full in-crate court set in both profiles: taxonomy
   rows 1-6 individually injected (`taxonomy_rows_are_individually_
   injectable`), order pinning (`order_pinning_first_failure_wins`),
   content independence, scoped zero-alloc over all 10M corpus frames,
   per-thread allocator-hook witness with in-court liveness control.
2. **1M-row ingress readiness (Gate-3).** Gate-3's harness DOES name
   this crate as the ingress stage: `tests/court_verdict.rs` line 1117
   imports `cgap_gate::penstock::ingress` and `a14_dam_ingress_court`
   (line 1513) drives it over every taxonomy row — ran green this
   session (exit 0 above). The in-crate dam corpus is 10,000,000
   frames x (0 misclassified, 0 allocs, 0 bytes), an order of magnitude
   past the 1M-row ledger scale — the dam sustains Gate-3's ingress
   leg. What is MISSING (coordinates, not repaired by this lane):
   (a) ctq-01's instrument `cargo bench -p bench --bench
   ctq01_admit_frame` does not exist — `crates/bench/benches/` holds
   only `v26_10_31.rs` (ls exit 0 this session); ctq-01
   `instrument_status: PENDING-LAND` stands
   (`docs/jira/v26.11.1/dflss/ctq-registry.yaml` ctq-01). (b) The
   Gate-3 leg-4 status note `gate-runner-playbook.md:144-147` is STALE:
   it reads "court_verdict harness = PROPOSED scaffold, Cargo.toml
   only"; the harness has landed (root `tests/court_verdict.rs`, owned
   by `crates/cgap-court` via its `[[test]] path` override, commit
   `9be9109d`) — A20/coordinator-owned doc, not amended by this lane.

## Court inventory run this session

dam_corpus_10m_zero_alloc_zero_misclassified; corpus_generator_is_
deterministic; taxonomy_rows_are_individually_injectable;
order_pinning_first_failure_wins;
content_independence_same_header_different_payloads;
alloc_tracking_hook_is_live;
allocator_hook_witness_bare_call_zero_alloc;
declared_len_governs_not_slice_length;
canonical_layout_pins_match_cgap_core_frame_rs;
status_register_is_pure_a19_subset;
throughput_ratio_fence_null_baseline — all 11 both profiles, exit 0;
plus cgap-court `a14_dam_ingress_court` exit 0; plus the §8.3 shim-1
flip (exit 101) with fail-closed restore. No `[SKIP]` rows fired
(dev-profile throughput skip is by-design §8.4 debug behavior; the
release run enforced the fence).

## Standing

**ALIVE** — every court named above ran and passed this session with
cited commands + exits; the anti-vacuity flip flipped; the restore is
sha256-witnessed.

## Divergences (recorded, not silent)

1. Dispatch framing vs. reality: `9be9109d` did not touch
   `crates/cgap-gate/`; the wave's relevance to this lane is that it
   edited the sibling register file (`sovereign.rs`) that the
   status-purity court parses at test time — re-verified green
   (post-wave four `CGAP_*` consts confirmed on disk at
   `crates/cgap-ffi/src/sovereign.rs:71-83`).
2. `gate-runner-playbook.md:144-147` Gate-3 leg-4 status is stale
   (harness has landed); A20/coordinator-owned, cited not edited.
3. ctq-01 bench target `ctq01_admit_frame` absent at
   `crates/bench/benches/` — the FFI-level ~4 ns/frame measurement
   (ctq-01) remains PENDING-LAND; the in-crate §8.4 fence is the only
   measured ingress-latency court on disk, and its TARGET rows are GAP
   (7.52 ns, 3.91x) this session.
