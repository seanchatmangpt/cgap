# Lane EX11 Census — cgap-core, cgap-ffi, cgap-crypto, scripts

Read-only census, 2026-11-02. No builds. Static reads only; standing UNVERIFIED where noted.

## cgap-core/src (4 files)
- lib.rs (4 ln): module decls only. 0 fns. No branches.
- dma_ring.rs (263 ln): fns AtomicCell{new,load,store}; Ring{new, slot_size, capacity,
  pending, claim, publish, read, release}. HOT: claim/publish/read/release (SPSC,
  O(1) atomic head/tail, no loops). COLD: Ring::new validation (align, slot_pow2,
  slot_size, cap bounds — typed RingError). No fsync/msync/flush anywhere.
- frame.rs (692 ln): P28LicenseAttestation{validate, encode_into, decode};
  CgapFrameHeader{encode_into, decode, project, validate, payload,
  verify_payload}; FrameError fmt. HOT: project (align+zero-copy), payload
  (bounds), encode/decode (len). COLD: validate() typed-refusal ladders
  (magic/version/reserved/lineage-popcnt/ceiling/upstream-xor; keep-branchy).
  Tests: 14 incl. 6-way attestation-corruption mutation loop (line 499).
- isomorphism.rs (164 ln): identity_assert, TierMismatch, Tier::name. HOT: byte-zip loop
  (first-divergence). COLD: tier Display.

## cgap-ffi/src (5 files)
- lib.rs (358 ln): C-ABI cgap_i32_batch, cgap_schema_addr, cgap_array_addr, cgap_batch_rows,
  cgap_version, cgap_digest_of; internals fold_word, digest_of_raw, cgap_xip_panic. COLD: all —
  thin C-ABI projections, no validation ladders needed.
- oabi_lookup.rs (365 ln): .oabi slot-lookup export surface (see header note line 5).
- receipt_chain.rs (405 ln): ChainReceipt, RingReceiptChain {new, len, is_empty, receipts,
  receipts_mut, append, verify}. COLD: append/verify typed-refusal ladders
  (ReceiptChainRefusal). GENESIS = [0u8;32].
- xip_tables.rs (734 ln): OABI_PACK_FIBO_SEC_DBT_V1 [496], BN3_NETWORK_A11_W2_10 [152],
  cgap_xip_start. Data tables + _start; no branch census.
- sovereign.rs (4631 ln, ~122 fns incl. tests, AS-OF-IN-FLIGHT):
  - FA9FIX LANDED: spill_gate per-lane-slot claim (spill_gate::Claim::Denied ->
  CGAP_ERR_CONCURRENT_WRITER_DENIED = 0x4347_0006, sov:758, const at :87); COMMIT_LEASE
  thread-local RefCell (:819); OWNERS Mutex<Vec<(tid,ptr)>> (:907); per-(path,thread) keying so
  same-lane reentry succeeds and foreign-lane reentry denied pre-IO (:826-869).
  - AS-OF-IN-FLIGHT: internals NOT adjudicated. Landed-state check only.
  - FA9 FINDINGS STILL CURRENT (verified this session, static read): zero fsync/msync/sync_all
  calls exist; spill reaches file ONLY at ring-rotation drain; drain uses flush()-only (comment
  block :740-745 states this exactly).
  - Entrypoints: cgap_verify_p28(:293), cgap_admit_frame(:413), cgap_eval_bn3(:534),
  cgap_commit_parquet(:698); C-signature static binds :1573-1576/:1652-1655; riscv XIP variant
  cgap_xip_start :2808/:2826/:3149 with `wfi` loops.
  - Branch census: C-ABI entrypoint validation = COLD typed CgapStatus refusals (keep-branchy).
  Hot loops: none inside entrypoints beyond payload iteration; test harness run_bounded(:2243)
  is test-only.

## cgap-crypto/src (3 files)
- lib.rs (16 ln): module decls only.
- lease.rs (219 ln): Lease {new, lease_id, tick_ceiling, issued_root, current_tick, tick, phi,
  to_bytes, from_bytes}. phi() pub(crate) PhiLease projection.
- chatman.rs (676 ln): keyed_digest, digest_then_id_le, trait DigestSealer {algorithm_id, seal,
  verify_seal}, UnsealedReceipt {to_bytes, from_bytes}, seal_unsealed, verify_unsealed,
  admissions_root. Test: nostd_hygiene (no std paths in module body).

## scripts (audited 10)
- boundary_gate.py (2211 ln): exits {0,1} only — EXIT_PASS=0(:139)/EXIT_REFUSE=1(:140); 25+
  sys.exit(EXIT_REFUSE) refusal sites. NO exit 2/3 paths (no EXIT_MISSING/EXIT_SKIP constants).
  Divergence from the 0/1/2/3 contract as documented in cgap/CLAUDE.md.
- zero_string_gate.py (2258 ln): exits {0,1} only (return 1 on violation at :1322/:1398/:2183;
  SystemExit(main()) :2258). No 2/3. Same divergence.
- enclave_budget_gate.py (738 ln): full contract {0,1,2,3} (:72-75): PASS/REFUSE/MISSING/SKIP.
  Intact.
- verify_ost_design_around.py (316 ln): exits {0,1}. main():307/312.
- release_check.py (402 ln): exits {0,1} (:374,:396,:398).
- registry_sweep.py (251 ln): exits {0,1} (:242,:245,:247).
- projection_coverage.py (649 ln): full {0,1,2,3} (:34-37). Intact; includes mutation court
  (flipped = rc==EXIT_FAIL, :571) and loud-skip verdicts.
- hop_ratio.py (201 ln): full {0,1,2,3} (:38-41). Intact.
- shacl_negative_corpus.py (303 ln): exits {0,1}; anti-vacuity all_flipped gate (:299).
- shacl_negative_corpus_b.py (546 ln): exits {0,1} (:411).

## ENTRY_SNIPPETS coverage (FA10 follow-up)
- enclave_budget_gate.py:388 ENTRY_SNIPPETS: 4 crates — cgap-blake3 (hash + hash_many_simd),
  cgap-gate (penstock::ingress), cgap-p28 (P28LicenseAttestation::from_bytes 80B), cgap-parquet
  (ffor_unpack + rle_encode).
- Exercised call sites counted this session: 6 visible statements across 4 crates (blake3 2,
  gate 1, p28 1, parquet 2). FA10 cited 7 — one-call-site counting-convention delta (parquet
  `enc.len()` black_box); confirm at next FA10 rerun.
- parquet scan: ABSENT from snippets (std-gated; no_std caller cannot reach it) — FA10 finding
  still current.
- No other script defines ENTRY_SNIPPETS (grep -l: only enclave_budget_gate.py).

## Coverage ledger: 100% of dispatched files enumerated above (4 core + 5 ffi + 3 crypto + 10
  scripts = 22 files).
## Standing: all claims are static reads (grep/sed) run this session against /Users/sac/cgap
  main; no builds; perf-free. sovereign.rs FA9FIX = AS-OF-IN-FLIGHT landed-state check only.
