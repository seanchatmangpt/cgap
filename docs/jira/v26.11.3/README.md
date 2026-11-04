# v26.11.3

Wave index for cgap v26.11.3. Receipts live in `receipts/`; this file is the
wave index (created by the cl-close-2 index-only lane, 2026-11-03 — cgap had
receipts but no index; SE census noted). Standing vocabulary:
ALIVE / PARTIAL / BLOCKED / UNVERIFIED / REFUSED. Never round up.

Exit record law: A gate is witnessed by an exit record: either (a) the process
exit status, or (b) cargo's "N passed; 0 failed" summary line. A passed-count
without a failed-count is not an exit record.

## Receipts table

Standing words re-read from each receipt body on disk (cl-close-2 lane,
2026-11-03), never from memory. Lane file: `lanes/cg-1-wire-tier-gates.md`.
Refreshed 2026-11-04 (yd-5 index lane): the table now covers ALL 55
receipt files in `receipts/` — standing words re-read from each body
again at refresh time; UNSTATED where a receipt carries no standing word
(ab-19, af-12, af-24, xc-9). At refresh start no `yd-*` receipt existed
in `receipts/` (disclosed; none to index); the lane's own receipt
(yd-5-cgap-index.md) and the concurrent lanes' yd-1-licenses-clarify.md + yd-3-publish-lines.md
landed during the pass and are indexed (YD-5 self-index, YD-3) — the
count closes over the dir exactly, both disclosed.

Refreshed 2026-11-04 second pass (zd-22 index-refresh2 lane): six more
receipts landed since the yd-5 pass — yd-6, yd-7, zd-1..zd-4 — now
indexed below, standing words re-read from each body at this pass.
yd-1 and yd-3 verified already present (YD-5 mid-pass absorb, re-read
this pass). No zd-8 receipt file exists on disk at this pass's start
(`ls receipts/ | grep zd-8` → no match), so no zd-8 row is written —
never a row for an absent file. UNSTATED standing added this pass:
yd-6 and yd-7 (no standing-vocabulary word in either body; yd-7
carries per-gate MEASURED/RED, not an overall word). Dir census
`ls receipts/ | wc -l` = 61 at this pass's start; the lane's own
receipt (zd-22-index-refresh2.md) landed during the pass and is
self-indexed (YD-5 precedent), closing the count at 62 rows over the
dir's 62 files exactly. Absorbed mid-pass (disclosed, ZD-5's
mid-pass-absorb precedent): the concurrent lane's
zd-19-audit-alloc-blackbox.md landed during the pass and is indexed —
final count 63 rows over the dir's 63 files exactly. Pointer-only:
e80-06/e80-07 rows live in the
claude-code-config-lsp index (see Placement notes) — not counted in
either number.

Refreshed 2026-11-04 third pass (zd-57 index-refresh3 lane): six more
receipts landed since the zd-22 pass — zd-6, zd-13, zd-26, zd-36,
zd-43, zd-55 — now indexed below, standing words re-read from each
body at this pass. UNSTATED standing added this pass: zd-36 and zd-55
(no overall standing-vocabulary word in either body; both carry
per-gate MEASURED/REFUSED words — yd-7 precedent). zd-36's UNSTATED
note: dod-011/012 raw commands REFUSED typed (FM-LAW-015), closed by
ZD-55's disposition. af-43's row refined this pass (disclosed:
threat-model content UNVERIFIED + CBMQS G2 open defect are in the
body's Standing section and were not surfaced in the row). Dir census
`ls receipts/ | wc -l` = 69 at this pass's start; the lane's own
receipt (zd-57-index-refresh3.md) lands during the pass and is
self-indexed (YD-5 precedent), closing the count at 70 rows over the
dir's 70 files exactly. Absent on disk at this pass's start, no row
written (never a row for an absent file, disclosed): zd-8, zd-53,
zd-56, xd-16 (`ls receipts/` grep → no match each). zd-42 exists
nowhere in this tree — it is a cross-repo pointer to arcus and is
disclosed in Placement notes, not counted here. All yd-* files
(yd-1/3/5/6/7) verified already rowed from earlier passes, re-checked
this pass by diff, not memory.

| Lane | Receipt | Standing | Gate (exit record) |
|---|---|---|---|
| CG-1 | receipts/cg-1.md | PARTIAL_ALIVE | `cargo test -p cgap-arrow` exit 0, 220 passed / 0 failed (baseline 214; +6 G1 courts); wire-shm "8 passed"; cgap-gate 12+1+3+5, 0 failed; `cargo check -p cgap-arrow` exit 0; no_std embedded check exit 0 |
| GA-5 | receipts/ga-5-cgap-wiring.md | PARTIAL_ALIVE | `ggen sync run` run 1 EXIT 0 (all 5 outputs written); run 2 EXIT 0, `written: []` (determinism); `cc -std=c99 -ffreestanding` exit 0; `cargo check -p cgap-ffi` exit 0. PARTIAL because the pack is a STAGED fallback, not GA-4's canonical marketplace aoq-pack — see GA-5B below |
| GA-5B | receipts/ga-5b-flipback.md | ALIVE (flip-back to the canonical marketplace aoq-pack; landed mid-pass, absorbed at re-census) | flip-back sync1 exit 0, written = 5 canonical slots; gate0 byte-stable replay PASS (include/cgap_aoq.h sha256 0bd81821…, freestanding cgap_c_data.h compile PASS); GA-4 gates re-verified: pass witness 0 violations (sync exit 0), fail witness ≥1 fail-row per gate with typed exits |
| SA-4 | receipts/sa-4.md | ALIVE | `cargo check` (crates/cli standalone) exit 0; `cargo test` 44 passed, 0 failed; `cargo test -p cgap-arrow` 228 passed, 0 failed |
| AF-30 | receipts/af-30-contender-a.md | ALIVE | `cargo test -p cgap-aoq --test af30_cage_match --release -- --nocapture` exit 0 x2, 1 passed / 0 failed (0.5630 / 0.5551 us per frame; census 0 allocs / 0 bytes); AF-14 cbmqs_runner self-scan exit 0 verdict PASS |
| SA-5 | receipts/sa-5.md | ALIVE (gates 170/180) / pre-existing gap disclosed | gates 170/180 executed at sync time with flipping-court non-vacuity, positive-path exit-0 sync receipt; `gate-court.toml` names runner `qualification/verify.py` which does not exist (prior-lane state, left as-is) |
| AB-1 | receipts/ab-1.md | ALIVE (AB-1-owned courts) / crate-wide suite BLOCKED on sibling in-flight files at write time | `cargo check -p cgap-aoq` exit 0; purity gate 8 passed + codec court 7 passed exit 0; wasm32 check exit 0; `just aoq-check` exit 0; `just aoq-gate` exit 0 GREEN |
| AB-2 | receipts/ab-2-fips202.md | ALIVE | fips202 gates ran and passed this session (exact subject) |
| AB-3 | receipts/ab-3-chacha.md | ALIVE | every gate command exit 0 this session |
| AB-4 | receipts/ab-4-aead.md | PARTIAL_ALIVE | all owned gates green this session; whole-crate `cargo test -p cgap-crypto` blocked by AB-3's in-flight files at write time |
| AB-5 | receipts/ab-5-mlkem.md | ALIVE | all gates ran this session, commands + exits in body (mlkem gates); rekey-cost and on-target estimates UNVERIFIED (disclosed) |
| AB-6 | receipts/ab-6-header.md | ALIVE | header gates green this session; AF-10/AB-6 reconciliation items recorded UNVERIFIED by AB-6 |
| AB-7 | receipts/ab-7-chain-session.md | ALIVE | `cargo test -p cgap-aoq` GREEN ×2 (run 2: 16 targets, 71 tests, 0 failures) |
| AB-8 | receipts/ab-8-wire.md | ALIVE (owned deliverables) / crate-level PARTIAL on AF-10's G2 fence | full `cargo test -p cgap-arrow` ALIVE exit 0, 0 failed; A10-A13 compiled-only (manifest freeze) |
| AB-9 | receipts/ab-9-ipc-stress.md | ALIVE | `cargo test -p cgap-arrow --test ipc_stress` exit 0, 4 passed / 0 failed; full crate 216 tests green; neighbors 5 + 17 passed |
| AB-10 | receipts/ab-10-writerv-stress.md | ALIVE | `cargo test -p cgap-arrow --test stream_writer_stress` exit 0 ×2 (pass 2: 4 passed / 0 failed); full suite exit 0 |
| AB-11 | receipts/ab-11-ring-stress.md | ALIVE | `ring_stress` exit 0, 4 passed, GREEN ×2; wire-shm full suite exit 0 (lib 1 + ring 4 + wire_courts 8) |
| AB-12 | receipts/ab-12-blake3-stress.md | ALIVE | `cargo test --release -p cgap-blake3 --test blake3_scale_stress` green; full `cargo test -p cgap-blake3` debug green; AVX2 path UNVERIFIED-on-host (target-gated) |
| AB-15 | receipts/ab-15-ffi-stress.md | ALIVE | `cargo test -p cgap-ffi --test ffi_stress` 4 passed / 0 failed (re-run green); full 7-binary sweep 28 passed / 0 failed; pytest 4 passed |
| AB-19 | receipts/ab-19-slab-stress.md | UNSTATED (body defers standing to the Verification Ladder; no standing word in body) | src/ untouched; fragmentation-gate mutation flips census (advance `len` on refused op) |
| AF-1 | receipts/af-1-corridor.md | ALIVE for the compiled subset / one typed BLOCKED row (AF1-KD-1 KEM, since CLOSED by BK-1) | corridor_join_court 3 passed / 0 failed ×2; regression sweep all ok (52 tests); mlkem_sanity exit 1 recorded as AF1-KD-1 |
| AF-3 | receipts/af-3-simd-profile.md | ALIVE | af3_court.sh exit 0 (AFTER=PASS BEFORE=FAIL); `cargo test -p cgap-blake3 --release` ×2, 9/9 targets ok; `cargo build --workspace` exit 0 |
| AF-4 | receipts/af-4-parse-meta.md | ALIVE | `cargo test -p cgap-arrow` full suite ×2 exit 0, all targets 0 failed |
| AF-6 | receipts/af-6-version-binding.md | ALIVE | `ggen sync run` exit 0 (`written: [generated_version.rs]`), second sync exit 0 `written: []` (determinism); 20/21 python binding tests pass, lone skip loud + typed |
| AF-8 | receipts/af-8-sandbox-e0583.md | ALIVE | cgap-ffi sandbox-court fix: BEFORE 4 failed / FINAL 58 passed exit 0; cgap-crypto ×2 green (closed via BK-3 takeover unblocks + BK-1 mlkem768 restructure) |
| AF-10 | receipts/af-10-cbmqs.md | ALIVE | cbmqs_gates release GREEN ×2 (8 PASS / 0 FAIL); debug pass disclosed 6 PASS / 1 FAIL / 1 SKIP before driver fix |
| AF-12 | receipts/af-12-flip-demo.md | UNSTATED (courts green; no standing word in body) | kat_flip_demo 4 passed / 0 failed ×2; meta-court flips 324 == 324, restore_ok=true |
| AF-14 | receipts/af-14-cbmqs-runner.md | ALIVE | runner self-test exit 0; `--court` exit 0 PASS with flipping witnesses (exit 2/0 bound test, MOCK_DETECTED exit 3); full suite exit 0 |
| AF-21 | receipts/af-21-g7-perlink.md | ALIVE | `receipt_link_bench --release` `1 passed` ×2 exit 0; G7-row append LANDED (supersedes earlier PARTIAL/BLOCKED on the append) |
| AF-22 | receipts/af-22-percall-latency.md | ALIVE | `cargo test --release -p cgap-blake3 --test percall_latency` green this session |
| AF-24 | receipts/af-24-narrative.md | UNSTATED (doc-propagation receipt; no standing word in body) | POST-FIX sections appended to ab-12 / arcus audit / prompt library; no existing text altered |
| AF-25 | receipts/af-25-g2-convergence.md | ALIVE (release) / loud-SKIP (debug, by design) | cbmqs_gates release exit 0 with amortized path inside the G2 gate; debug SKIP typed, never a fake pass |
| AF-26 | receipts/af-26-clearing.md | ALIVE | scaled 1M ×2 + full 10M, exit 0; src/ untouched; falsifier = census drift / alloc > 0 reproduces as loud assert |
| AF-27 | receipts/af-27-defense-mesh.md | ALIVE (mesh bench) / no_os+EW+1024-scale claims UNVERIFIED | macro_defense_mesh exit 0 ×2 (1 passed / 0 failed); full crate exit 0 |
| AF-29 | receipts/af-29-federation.md | PARTIAL_ALIVE (scenario) / composes ALIVE surfaces | federation scenario gates exit 0 this session; 50M scale TARGET-UNMEASURED (falsifier: any 50M/500-endpoint run) |
| AF-32 | receipts/af-32-saturation.md | ALIVE (scaled runs) / 1B run UNVERIFIED (documented estimate, executed: NO) | scaled 10M exit 0 ×3 (debug 2, release 1: 278,630 frames/s); 200k smoke exit 0 |
| AF-33 | receipts/af-33-density.md | ALIVE | macro_density_bench exit 0 ×2 debug + ×2 release; host ELF linked-truth BLOCKED on this host (macOS `_start`), disclosed |
| AF-34 | receipts/af-34-perf-audit.md | PARTIAL_ALIVE | wall-clock + IPC measured; PMU counters UNVERIFIED (3 independent PMU routes probed and refused); af34_perf_audit.sh exit 0 MEASURED |
| AF-35 | receipts/af-35-transit.md | ALIVE | macro_transit_bench --release exit 0 ×2; 10M mode exit 0; conservation 900,000 + 100,000 = 1,000,000 |
| AF-37 | receipts/af-37-p99-torture.md | ALIVE | macro_p99_torture release ×2 + debug ×2 exit 0 (serial); 22/22 integration binaries ok; one PRE-EXISTING doc-test failure exit 1 (not this lane's) |
| AF-43 | receipts/af-43-cgap-publish.md | ALIVE (six blockers; Standing section) / threat-model content UNVERIFIED (reconstruction pending AF-17 reconciliation — disclosed in body, re-read at zd-57 refresh) | macro_p99_torture rc=0 ×2; cgap-ffi lib 58/58; certificate 3/3 exit 0; `cargo test --workspace` 55 `ok` lines; CBMQS G2 fail-honest (717 ns vs 100 ns ceiling) carried as registered open defect |
| BK-1 | receipts/bk-1-takeover.md | ALIVE (AB-4 + AB-5 + corridor join on real KEM); AF1-KD-1 CLOSED | `cargo test -p cgap-crypto` GREEN ×2 (75 passed / 0 failed); riscv32imc + wasm32 checks exit 0; corridor_join_court 3/3 ×2 |
| BR-2 | receipts/br-2-receipt-lint.md | ALIVE | receipt_lint.sh --self-check PASS (known-good 0 / shorthand 1 / missing-cite 1); known-good corpus CLEAN exit 0 |
| GD-1 | receipts/gd-1-dod-pack.md | ALIVE (pack surface) / PARTIAL overall (dod-006 AB-5 ML-KEM TARGET-UNMEASURED in the DoD graph) | `ggen sync run` ×2 exit 0 (2 dod projections); witness court green + FAIL witness via SHIM-COURT probe (FM-PACK-013); shim-court flip green |
| RX-2 | receipts/rx-2-conformance.md | ALIVE | rfc0089_conformance release ×2 + debug ×2 exit 0 (4 passed / 0 failed); lib courts 14/0 ×2; CBMQS 1/0 ×2; AF-30 1/0 |
| RX-3 | receipts/rx-3-scorecard.md | ALIVE (honest DISQUALIFIED verdict: unmet G7) | rfc0089_scorecard --release exit 0 ×2 (3 passed / 0 failed) + flake-margin pass 3; overall DISQUALIFIED (unmet G7), never rounded up |
| RX-4 | receipts/rx-4-profile-guard.md | ALIVE | profile_guard courts green: lib 14/0 debug + release; CBMQS 1/0 debug + release; riscv32imc check exit 0; pre-existing failures disclosed as not-this-lane's |
| RX-5 | receipts/rx-5-certificate.md | ALIVE | rfc0089_certificate 5 passed / 0 failed ×2 exit 0; tamper mutations typed-refused |
| RX-5C | receipts/rx-5-correction.md | ALIVE | rfc0089_certificate 5/0 ×2 + certificate_gate_table_court 7/0 ×2 exit 0 |
| XC-9 | receipts/xc-9-cgap-main.md | UNSTATED (triage receipt; no standing word in body) | verify_v26 fix: `-p cgap-cli` → `--manifest-path crates/cli/Cargo.toml`, battery exit 0 (7 rows, 0 FAIL, typed skip FR-07); 4 contention-flake gates pass isolated ×1 each; residual re-flake risk disclosed (shared-host fleet co-tenancy) |
| XD-1 | receipts/xd-1-deny-toml.md | ALIVE | `cargo deny check advisories licenses` exit 1 (licenses FAILED — the finding, not a gate gap); ci.yml yaml.safe_load exit 0 |
| XD-5 | receipts/xd-5-error-taxonomy.md | ALIVE | `cargo check -p cgap-aoq` exit 0 for the taxonomy doc edits; attack-scenario population disclosed |
| YD-5 | receipts/yd-5-cgap-index.md | ALIVE (this row is the index's own receipt; census commands cited in the receipt ran this session) | `ls receipts/ \| wc -l` = 55; awk table-row census = 55 receipt rows (self-index closure: the receipt indexes itself); `uniq -d` → no duplicate lane labels |
| YD-3 | receipts/yd-3-publish-lines.md | ALIVE | `publish = false` added to cgap-mcp + wire-shm Cargo.tomls (crates.io accidentally-publishable fix); `cargo check -p wire-shm` exit 0; `cargo check` in crates/cgap-mcp exit 0; workspace grep census cited |
| YD-1 | receipts/yd-1-licenses-clarify.md | ALIVE (XD-1 licenses divergence CLOSED via clarify, not allow-list laxity) | `cargo deny check licenses` exit 4 → clarify entry added → exit 0; advisories exit 0; mutation re-run exit 4 (gate flips); wrong-hash clarify FAILS; advisories gate stays BLOCKING |
| YD-6 | receipts/yd-6-buffer-too-short.md | UNSTATED (rename receipt; no standing-vocabulary word in body — verification section reports `cargo test -p cgap-aoq` green ×2 + `cargo check --workspace` exit 0) | post-edit `grep -rn HeaderTooLong crates/ docs/aoq-header-offsets.md` → 0 matches (exit 1); `cargo test -p cgap-aoq` green ×2 (18 targets, 0 failed each); `cargo check --workspace` exit 0; wire class 0x00_02 unchanged |
| YD-7 | receipts/yd-7-dod-rerun.md | UNSTATED as an overall word (per-gate MEASURED ×11 / RED ×1 in body's gate table; 12/12 gates executed, none skipped) | dod-001..012 via run_checked.py: 11 exits 0 (dod-001 242/0; dod-005 aead 8/8; dod-011/012 sync ×2 `written: []`); dod-006 exit 101 RED classified pre-existing (build-dir race run 1; aead throughput-floor absolute-fence flake run 2, load-bound) — F2 dod-006 graph drift found, closed by ZD-4 |
| ZD-1 | receipts/zd-1-dod10-manifest-red.md | ALIVE (Status header; XA-2 census red #1 CLOSED) | dod10 manifest court red ×3 root-caused as court over-ban (`license-file` is not an SPDX identifier); court fixed in mini-workspace sandbox with flip witness; DoD-05/publish.md/deny-clarify law intact; falsifier = SPDX key/LicenseRef-* edit must fail loudly |
| ZD-2 | receipts/zd-2-fa9-race-grid.md | ALIVE (Standing section: ALIVE for the court) | 3 FA9 release reds = test poll bound in iterations, not time (pre-existing); `spin_until` now Instant-deadline-checked, no sleep; court 7/7 release ×3 + debug ×1 exits 0; lib 58/0; zero_string_gate rc=0 unpiped |
| ZD-3 | receipts/zd-3-alloc-scope.md | ALIVE (Status header: green release + debug) | alloc_report red = LLVM heap elision in release (test bug, not tracker bug); `black_box` on both allocations in error_paths.rs; release 1 passed (flipped at 636 then 640 pre-fix), debug 1 passed; out-of-scope latent fragility in crates/audit unit test disclosed |
| ZD-4 | receipts/zd-4-dod-ontology-f2.md | ALIVE (Status header) | dod-pack ontology dod-006 row TARGET-UNMEASURED → MEASURED citing bk-1-takeover.md, publishBlocking false → true (lawful under gate 020); checklist regenerated, converged byte-identical over two sync runs; YD-7 Finding F2 close condition met |
| ZD-22 | receipts/zd-22-index-refresh2.md | ALIVE (this row is the refresh's own receipt; census commands cited in the receipt ran this session — YD-5 self-index precedent) | `ls receipts/ \| wc -l` = 62 after the receipt landed, 63 after zd-19 absorbed mid-pass; table receipt rows = 63 (self-index closure + mid-pass absorb); `uniq -d` → no duplicate lane labels; `ls receipts/ \| grep zd-8` exit 1 (no zd-8 file → no row) |
| ZD-19 | receipts/zd-19-audit-alloc-blackbox.md | ALIVE (Status header; Standing section: ALIVE for post-fix green ×2 release + ×1 debug, full crate unregressed) — absorbed mid-pass, standing re-read from body | audit alloc_counter.rs:188 `nested_inner_scope_excludes_outer_allocations` black_box fix (ZD-3 defect class); `cargo test -p cgap-audit` green release ×2 + debug ×1, exits shown in body |
| ZD-6 | receipts/zd-6-w2a-session-ffi.md | ALIVE (Status header) | `cargo test -p cgap-ffi` exit 0 ×3 (lib 60/60 = 58 baseline + 2 new; aoq_session_ffi_court 6/0; final runs A/B post-W2-C ×2 gate); `cargo check -p cgap-aoq` exit 0; `cargo check -p cgap-ffi --all-targets` exit 0; `cargo build --workspace` exit 0; coordinator-owned Cargo.toml cross-touch disclosed; 35-min mid-wave blocker on W2-C in-flight file disclosed (owner repaired in window, SLA unblock not exercised) |
| ZD-13 | receipts/zd-13-w2c-sidecar.md | ALIVE (Status line: courts ran this session) | `cargo test -p cgap-ffi --test aoq_sidecar_court` 3 passed / 0 failed ×2 exit 0 (real sidecar process over UDS, real ML-KEM-768, no mocks); full `cargo test -p cgap-ffi` exit 0, every target 0 failed (lib 60/0); XIP courts (riscv elf-size, r2 production link) 1/0 each exit 0 with the new bin landed; corridor_join_court 3/0 unregressed; lane build root target-w2c-lane deleted at receipt time with post-delete proof |
| ZD-26 | receipts/zd-26-frozen-drift.md | ALIVE (Standing section: drifts 1+2 closed at source) — residual EA-admission policy decision UNVERIFIED here (disclosed; closed by ZD-55) | both GGM-SRC-002 drifts lifted into source (kind_enums std::error→core::error into the aoq-pack template; XC-5r doc prose into ontology graph); re-lock syncs exit 0 under disclosed `[FM-LAW-015]` opt-out `--skip-ea-admission`; convergence run `written: []`, 5/5 freeze pins re-verified blake3 MATCH; `cargo check -p cgap-ffi` exit 0; `cargo test -p cgap-ffi --lib` 60/0 |
| ZD-36 | receipts/zd-36-dod-final.md | UNSTATED as an overall word (per-gate MEASURED ×10 / REFUSED ×2 in the body's gate table; 12/12 gates executed, none skipped — yd-7 precedent) | dod-001..012 re-run at 37721e2 + dirty W2 tree: 10 exits 0 (dod-001 242/0; dod-006 full serial crypto suite 0 failed — load-bound flake classification holds); dod-011/012 raw commands exit 1 typed `[FM-LAW-015]` EA-admission refusal (gate postdates ga-5b) — disclosed opt-out variant exit 0 `written: []` ×2; no stale-row revert; resolution UNVERIFIED here, closed by ZD-55 |
| ZD-43 | receipts/zd-43-dryrun-ladder.md | PARTIAL (body header: 7/20 dry-run exits 0; two typed failure classes) | `cargo publish --dry-run` ×20 sequential topo order at 37721e2, no source edits / git transitions / real publishes; 7 exit 0; 11 registry-dep-missing exit 101 (26.11.3 siblings unpublished — self-healing at Stage 3); 2 dirty-worktree exit 101 (cgap-audit, cgap-ffi); the predicted license-file blocker never fired; 22-vs-20 publish=true census discrepancy recorded, not repaired, flagged for Stage 3 |
| ZD-55 | receipts/zd-55-dod-commands.md | UNSTATED as an overall word (Standing section: dod-011/012 MEASURED this session; the skip is disclosed, not silent — yd-7 precedent) | dod-011/012 disposition: `--skip-ea-admission` pinned into packs/dod-pack/ontology.ttl (option (b) ruling; cgap root sync manufactures no SBBs, so a root ea.graph.json would be an admission gate over nothing); ladder: raw sync exit 1 FM-LAW-015 → pack-edit sync exit 1 FM-PACK-008 → documented re-lock exit 0 → fixed point `written: []` exit 0; as-written checklist commands exit 0; ZD-36's open item closed |

## Post-close receipts (zd-116 final coverage pass, 2026-11-04)

Ten receipt stems on disk carried no row in this file at the terminal
mention-level coverage gate (stems vs README.md — this repo's only index
file; it carries no RECEIPTS.md, unchanged by this pass): five landed in
earlier windows without rows (zd-8, zd-53, zd-56, zd-63, zd-76, zd-79)
and four landed after zd-57's 70-census (zd-97, zd-102, zd-103,
zd-105). They are rowed below, standing words re-read from each receipt
body on disk, never from memory; UNSTATED where the body carries none.
Census at this lane's run: `ls receipts/ | wc -l` = 80 before this
lane's own receipt, 81 with zd-116-index-final.md on disk;
zd-113-ladder-refresh.md landed mid-pass during this lane's
verification run — 82 at final re-census — absorbed with its own row.
Coverage gate after these rows: UNCOVERED [] (exit 0). zd-106b has no
receipt file on disk in this tree — nothing to row (disclosed, never a
row for an absent file).

| Lane | Receipt | Standing | Gate (exit record) |
|---|---|---|---|
| ZD-8 | receipts/zd-8-dod006-quiet.md | ALIVE (Standing section: court ALIVE; dod-006 red classified load-bound-flake, close condition satisfied) | `throughput_min_of_n_floor` ×3 exit 0 on quiet host (min 261.26 MB/s open, ~130x the 2 MB/s floor); quiet precondition via bounded pgrep polling with the poller's own self-match defect found and corrected |
| ZD-53 | receipts/zd-53-cgap-git-courts.md | ALIVE (Standing section: zero-alloc claim now per-thread-scoped, serial ×5 green; alloc_counter.rs untouched, per-thread by construction) | soc court serial ×5 → 3 passed / 0 failed (flake reproduced 3/6 before fix, 0/5 after); full release suite ×2 green (244/0); ANDON child re-exec exit 101 with refusal in captured stdout |
| ZD-56 | receipts/zd-56-sidecar-e2e.md | ALIVE (Standing section: every command run this session against the exact subject; final state re-read on disk) | rung-8 E2E courts 3 passed / 0 failed ×2 each (E1 happy path digest-equal, E2 flipped-byte typed DIGEST_MISMATCH, E3 process reuse + stdin-EOF stop); full suite exit 0; XIP courts green; 0 stale aoq sockets post-sweep |
| ZD-63 | receipts/zd-63-membrane-closeout.md | PARTIAL_ALIVE (body Status: documentation-correctness standing, claims no new executions; rungs 7/8 unwitnessed at closeout — rung 8 closed since by zd-56, rung 7 by zd-79's BLOCKED witness then commit f30580ff) | four lane receipts re-read from disk + presence checks on eight lane-owned files (mtimes Nov 4 01:48-03:42); absence checks for zd-56/zd-63 at closeout (self-indexed per YD-5 precedent) |
| ZD-76 | receipts/zd-76-sweep-verify.md | UNSTATED in body (gate table carries the evidence; intermediate red classified environmental — A07 anti-vacuity needs full debug artifacts, resolved by `cargo build --workspace`) | `cargo check --workspace` exit 0; `cargo test -p cgap-ffi` exit 0 (lib 60/60, sidecar court 7/7, 15 suites ok); `cargo test -p cgap-git --release` exit 0 (40 suites, 0 failed); `manifest_court` 3 passed / 0 failed |
| ZD-79 | receipts/zd-79-gate0-witness.md | BLOCKED (Status header; deterministic ×2, not flaky; closed since by commit f30580ff per zd-105) | `make gate0` exit 2 ×2 — typed FM-LAW-015 refusal (no ea.graph.json at repo root; recipe passes no `--skip-ea-admission`); byte-stable replay + header compile unreached, disclosed |
| ZD-97 | receipts/zd-97-dryrun-retry.md | UNSTATED as an overall word (Classification section: registry-dep-missing, the same self-healing class as ZD-43's 11) | `cargo publish -p cgap-audit/-p cgap-ffi --dry-run` exit 101 ×2 (cgap-arrow ^26.11.3 unresolvable on crates.io); ladder closes 20/20: 7 green + 12 registry-dep self-healing + 0 unclassified |
| ZD-102 | receipts/zd-102-cgap-prelanding.md | ALIVE (Standing section: checks 1, 2, 4a, 4b, 5 executed this session with real exits; UNSAFE verdict rests solely on check 3, executed exit 1) | `git status --porcelain` clean; `cargo check --workspace` exit 0; `cargo fmt --all -- --check` exit 1 — 1268 hunks / 190+ files, repo-wide fmt drift, the finding; deny advisories + licenses exit 0; `make gate0` exit 0 |
| ZD-103 | receipts/zd-103-ffi-warnings.md | ALIVE (verification header: exact subject, this session) | `cargo check -p cgap-ffi` → 0 warning sites in owned bin; clippy --all-targets → 0 mentions of aoq_sidecar.rs (200 crate-wide typed, recorded not rounded); sidecar + e2e courts 3/0 + 3/0 unregressed |
| ZD-105 | receipts/zd-105-cgap-closure.md | ALIVE for the seal's contracted scope (both files landed; BLOCKED/DEFERRED carries recorded, not claimed closed) | `cargo check --workspace` exit 0 (13 pre-existing cgap-crypto warnings disclosed); `make gate0` exit 0; `git rev-parse HEAD origin/main` equal (b6d68517); receipt census 78 pre-seal / 80 post-seal |
| ZD-113 | receipts/zd-113-ladder-refresh.md | ALIVE for the plan refresh (Standing section; Stage 3 itself remains UNEXECUTED — nothing published) | `cargo metadata --no-deps` census exit 0 ×2 (25 members, 21 publishable, 4 `publish = []` excluded); plan §3.2 license-file blocker FALSIFIED against ZD-43/ZD-97 evidence; ladder census 17 → 21 pinned; edits confirmed on disk |
| ZD-116 | receipts/zd-116-index-final.md | ALIVE (contracted scope, self-row; the gate UNCOVERED [] is the witness) | mention-level coverage gate exit 0 over 82 receipt stems (10 uncovered at lane start, 0 after rows landed; 1 receipt absorbed mid-pass); census 80-before / 82 at final re-census |

## Post-close receipts, second refresh (zd-143 index refresh 5, 2026-11-04)

Coverage loop re-run after zd-116's 82-census: `ls receipts/ | wc -l` = 87
at this lane's census (zd-141-sed-portability.md landed mid-pass — 86 at
first count, 87 at re-census — absorbed with its own row). Stems-on-disk
vs row-IDs diff found six files with no row: zd-57 (the refresh-3 receipt
itself, narrated in this file since its pass but never rowed), zd-119,
zd-133, zd-134, zd-136, zd-141. zd-136's own body census said 84 (+1
untracked); this lane's fresh census supersedes: 87 before this lane's
receipt, 88 with zd-143-index-refresh5.md on disk. Standing words re-read
from each receipt body on disk this pass, never from memory; UNSTATED
where the body carries none. zd-141 (the task brief's candidate) IS on
disk and rowed below; zd-106b remains absent (still no row, per the
zd-116 note). Coverage gate after these rows + the self-row: every
on-disk stem has a row or a disclosed pointer — closed.

| Lane | Receipt | Standing | Gate (exit record) |
|---|---|---|---|
| ZD-57 | receipts/zd-57-index-refresh3.md | ALIVE (body Standing section; this row is the refresh's own receipt, census commands ran its session — self-indexed per YD-5 precedent) | refresh-3 census 69-before / 70-after; 6 rows added (zd-6/13/26/36/43/55) + af-43 row refinement; absent-file greps (zd-8/53/56/xd-16) exit 1 each — no rows for absent files; duplicate-lane-label check `uniq -d` empty |
| ZD-119 | receipts/zd-119-cgap-preflight.md | PARTIAL (body Whoami verdict, verbatim: token present, registry reachable, login identity unverified — `cargo whoami` exit 101, subcommand not installed) | `cargo search cgap-core --limit 1` resolved (registry API reachable); `~/.cargo/credentials.toml` present (0600, one token entry); crates.io API census ×23 crates at 2026-11-04 — zero 26.11.3 collisions, cgap-aoq NAME-FREE, wasm-footprint max 26.11.1; replay path disclosed (`cargo install cargo-whoami`) |
| ZD-133 | receipts/zd-133-closure-addendum.md | ALIVE (body Standing header; seal claims stay anchored at cfdd58ab — the addendum extends, does not re-grade) | `git diff --stat cfdd58ab..HEAD` → 5 files 270+/40- at c9430086; four post-seal commits listed (zd-103/zd-8/zd-113/zd-119); one source path disclosed (aoq_sidecar.rs warning-fix only, crates/ diff read in full, 14 lines); receipt census 83 at authoring (seal-time 80) |
| ZD-134 | receipts/zd-134-claude-census.md | ALIVE (body Status: all commands ran this session; header-paragraph drift disclosed, not repaired — lane boundary) | `git pull --ff-only` exit 0 (@09e8c7c7); `cargo metadata --no-deps` member count 25; per-manifest publish grep → 25/25 flagged (21 true / 4 false, wire-shm joined false); CLAUDE.md publish-law paragraph re-derived from the zd-119 live census; stale header "19 of 20" sentence left for its owner, disclosed |
| ZD-136 | receipts/zd-136-fleet-census.md | ALIVE (body Standing: all figures read live at as-of 2026-11-04T16:43:02Z, zero git mutations, sole write = this receipt) | per-repo census ×6 (ggen/cgap/arcus/lsp-max/claude-code-config-lsp/ggen-marketplace) via rev-parse + ls-remote; fleet receipts total 366; CARRY closed set 8 re-verified; PR #824 OPEN with headRefOid = ggen local HEAD |
| ZD-141 | receipts/zd-141-sed-portability.md | LANDED (body Status header, verbatim; unpushed at receipt time — body's own word, not one of the ALIVE/PARTIAL/BLOCKED vocabulary terms) | `bash -n scripts/verify_v26.10.31.sh` exit 0; minimal reproduction of the exact classify-branch snippet on the FAIL input state → PRE-EXISTING-FAIL written, tmp consumed by mv (no residue); BSD `sed -i ''` replaced with portable non-`-i` rewrite (ZB-65, GNU-sed parse defect); GNU-side proof deferred to next ubuntu-latest classify-branch run, disclosed |
| ZD-143 | receipts/zd-143-index-refresh5.md | ALIVE (this row is the refresh's own receipt; census + diff commands cited here ran this session — YD-5 self-index precedent) | `ls receipts/ \| wc -l` = 87 before this receipt, 88 after; stems-vs-row-IDs diff → 6 uncovered (zd-57/119/133/134/136/141), 0 after rows landed; zd-141 absorbed mid-pass (86→87); `uniq -d` on row IDs → no duplicate lane labels |

## Placement notes (pointer-only)

- e80-06 and e80-07 receipts are hosted in
  /Users/sac/claude-code-config-lsp/docs/jira/v26.11.3/receipts/ — their
  subject repo is /Users/sac/cgap. Indexed there per the cl-close placement
  note; not relocated, not duplicated here.
- cg-1's receipt cites the lane file `lanes/cg-1-wire-tier-gates.md` in this
  tree; the lane plan is not itself a receipt.
- zd-42 is a cross-repo pointer: its receipt lives in the arcus tree, not
  here (`ls receipts/ | grep zd-42` → no match at the zd-57 pass). It is
  disclosed here, not rowed, not counted.
- zd-8, zd-53, zd-56, xd-16 have no receipt file on disk in this tree as of
  the zd-57 pass (each grep → no match) — no rows written for absent files.

## See Also

- `receipts/` — per-lane receipts
- `/Users/sac/claude-code-config-lsp/docs/jira/v26.11.3/README.md` — hosts the
  e80-06/e80-07 rows whose subject is this repo
