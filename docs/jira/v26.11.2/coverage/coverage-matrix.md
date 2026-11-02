# Coverage matrix (LANE C01 llvm-cov baseline, HEAD 9a730320 snapshot)

| crate | functions cov/tot | lines cov/tot | regions cov/tot | fn % |
|---|---|---|---|---|
| cgap-arrow | 764/931 | 4104/4480 | 9242/10959 | 82.1 |
| cgap-blake3 | 87/89 | 837/904 | 3113/3283 | 97.8 |
| cgap-gate | 75/76 | 594/699 | 1092/1190 | 98.7 |
| cgap-branchless | 115/125 | 1006/1011 | 2925/3677 | 92.0 |
| cgap-parquet | 837/925 | 6772/7321 | 14741/16384 | 90.5 |
| cgap-p28 | 193/241 | 1937/2104 | 3422/3987 | 80.1 |
| cgap-ffi | 243/305 | 2472/2831 | 3503/3856 | 79.7 |
| cgap-audit | 0/0 | 0/0 | 0/0 | 0.0 |
| cgap-gpu | 6/8 | 31/40 | 39/49 | 75.0 |
| cgap-bont | 221/354 | 2098/2432 | 3113/4585 | 62.4 |
| cgap-bosym | 53/62 | 632/729 | 920/1017 | 85.5 |
| cgap-court | 0/0 | 0/0 | 0/0 | 0.0 |
| cgap-fibo | 0/0 | 0/0 | 0/0 | 0.0 |
| cgap-cloud | 0/0 | 0/0 | 0/0 | 0.0 |

## Top uncovered / cold functions (lowest execution count)

### cgap-arrow

- count=0 regs=0 `_RNvNtCsiKYuwnqUMBw_10cgap_arrow5c_abi17release_slot_swapCs2Ysa0NuUWki_8cgap_ffi` (cgap-arrow/src/c_abi.rs)
- count=0 regs=0 `_RINvNtCsiKYuwnqUMBw_10cgap_arrow5c_abi13release_arraylECs2Ysa0NuUWki_8cgap_ffi` (cgap-arrow/src/c_abi.rs)
- count=0 regs=0 `_RNCNvMs2_NtCsiKYuwnqUMBw_10cgap_arrow5batchNtB7_5Batch3get0B9_` (cgap-arrow/src/batch.rs)
- count=0 regs=0 `_RNCNvMs2_NtCsiKYuwnqUMBw_10cgap_arrow5batchNtB7_5Batch7try_new0B9_` (cgap-arrow/src/batch.rs)
- count=0 regs=0 `_RNCNvMs2_NtCsiKYuwnqUMBw_10cgap_arrow5batchNtB7_5Batch7try_news_0B9_` (cgap-arrow/src/batch.rs)
- count=0 regs=0 `_RNCNvMs4_NtCsiKYuwnqUMBw_10cgap_arrow5batchNtB7_12BatchBuilder10column_mut0B9_` (cgap-arrow/src/batch.rs)
- count=0 regs=0 `_RNCNvMs4_NtCsiKYuwnqUMBw_10cgap_arrow5batchNtB7_12BatchBuilder3len0B9_` (cgap-arrow/src/batch.rs)
- count=0 regs=0 `_RNCNvMs4_NtCsiKYuwnqUMBw_10cgap_arrow5batchNtB7_12BatchBuilder6finish0B9_` (cgap-arrow/src/batch.rs)

### cgap-blake3

- count=0 regs=0 `_RNvCs4GEL50CBiVg_11cgap_blake319hash_many_simd_4way` (cgap-blake3/src/lib.rs)
- count=0 regs=0 `_RNvCs4GEL50CBiVg_11cgap_blake38compress` (cgap-blake3/src/lib.rs)
- count=1 regs=7 `_RNvCs3BwHdrZrkJj_11cgap_blake38compress` (cgap-blake3/src/lib.rs)
- count=1 regs=7 `_RNvNtCs4GEL50CBiVg_11cgap_blake311multi_chunk20compress_chunks_4way` (cgap-blake3/src/multi_chunk.rs)
- count=1 regs=140 `_RNvNtCs4GEL50CBiVg_11cgap_blake311multi_chunk25compress_chunks_4way_neon` (cgap-blake3/src/multi_chunk.rs)
- count=1 regs=13 `_RNvNtCs4GEL50CBiVg_11cgap_blake35testss_28test_gpu_crossover_heuristic` (cgap-blake3/src/lib.rs)
- count=1 regs=22 `_RNvNtCs4GEL50CBiVg_11cgap_blake35testss_50test_simd_parent_reduction_parity_64_to_128_chu` (cgap-blake3/src/lib.rs)
- count=1 regs=30 `_RNvNtCs4GEL50CBiVg_11cgap_blake35testss_56test_official_blake3_vectors_multi_block_and_me` (cgap-blake3/src/lib.rs)

### cgap-gate

- count=0 regs=0 `_RNvNtNtCs42YstLIBEzq_9cgap_gate8penstock6courts13null_baseline` (cgap-gate/src/penstock.rs)
- count=1 regs=3 `_RNvNtCsalPnqDAbZ5U_9cgap_gate8penstock10prefilter8Cs4LWIV285TCU_12lever_courts` (cgap-gate/src/penstock.rs)
- count=1 regs=5 `_RNvNtNtCs42YstLIBEzq_9cgap_gate8penstock19nostd_hygiene_court20module_body_stripped` (cgap-gate/src/penstock.rs)
- count=1 regs=5 `_RNvNtNtCs42YstLIBEzq_9cgap_gate8penstock19nostd_hygiene_courts_42nostd_hygiene_module_bod` (cgap-gate/src/penstock.rs)
- count=1 regs=1 `_RNCNvNtNtCs42YstLIBEzq_9cgap_gate8penstock6courtss_43allocator_hook_witness_bare_call_zer` (cgap-gate/src/penstock.rs)
- count=1 regs=1 `_RNCNvNtNtCs42YstLIBEzq_9cgap_gate8penstock6courtss_43allocator_hook_witness_bare_call_zer` (cgap-gate/src/penstock.rs)
- count=1 regs=1 `_RNCNvNtNtCs42YstLIBEzq_9cgap_gate8penstock6courtss_43allocator_hook_witness_bare_call_zer` (cgap-gate/src/penstock.rs)
- count=1 regs=1 `_RNCNvNtNtCs42YstLIBEzq_9cgap_gate8penstock6courtss_43allocator_hook_witness_bare_call_zer` (cgap-gate/src/penstock.rs)

### cgap-branchless

- count=0 regs=0 `_RNCNvNtCseczIvLhif7F_15cgap_branchless6select22checked_uniform_select0B5_` (cgap-branchless/src/select.rs)
- count=0 regs=0 `_RNCNvNtCseczIvLhif7F_15cgap_branchless6select22checked_uniform_selects_0B5_` (cgap-branchless/src/select.rs)
- count=0 regs=0 `_RNvNtCs68lbrfqMOdY_15cgap_branchless4scan15prefix_max_u8x8` (cgap-branchless/src/scan.rs)
- count=0 regs=0 `_RNvNtCs68lbrfqMOdY_15cgap_branchless4scan15prefix_or_u64x8` (cgap-branchless/src/scan.rs)
- count=0 regs=0 `_RNvNtCs68lbrfqMOdY_15cgap_branchless4scan15swar_lt_harvest` (cgap-branchless/src/scan.rs)
- count=0 regs=0 `_RNvNtCs68lbrfqMOdY_15cgap_branchless4scan16prefix_and_u64x8` (cgap-branchless/src/scan.rs)
- count=0 regs=0 `_RNvNtCs68lbrfqMOdY_15cgap_branchless9transpose13transpose_8x8B3_` (cgap-branchless/src/transpose.rs)
- count=0 regs=0 `_RNvNtCs68lbrfqMOdY_15cgap_branchless9transpose15transpose_64x64B3_` (cgap-branchless/src/transpose.rs)

### cgap-parquet

- count=0 regs=0 `_RNCNvNtCs8B6tsDSUbde_12cgap_parquet9byteslice17scan_eq_byteslice0B5_` (cgap-parquet/src/byteslice.rs)
- count=0 regs=0 `_RINvNtCs8B6tsDSUbde_12cgap_parquet17compiled_manifest5rle_wKt1_EB4_` (cgap-parquet/src/compiled_manifest.rs)
- count=0 regs=0 `_RINvNtCs8B6tsDSUbde_12cgap_parquet17compiled_manifest5rle_wKt8_EB4_` (cgap-parquet/src/compiled_manifest.rs)
- count=0 regs=0 `_RINvNtCs8B6tsDSUbde_12cgap_parquet17compiled_manifest7plain_wKt100_EB4_` (cgap-parquet/src/compiled_manifest.rs)
- count=0 regs=0 `_RINvNtCs8B6tsDSUbde_12cgap_parquet17compiled_manifest7plain_wKt10_EB4_` (cgap-parquet/src/compiled_manifest.rs)
- count=0 regs=0 `_RINvNtCs8B6tsDSUbde_12cgap_parquet17compiled_manifest7plain_wKt20_EB4_` (cgap-parquet/src/compiled_manifest.rs)
- count=0 regs=0 `_RINvNtCs8B6tsDSUbde_12cgap_parquet17compiled_manifest7plain_wKt40_EB4_` (cgap-parquet/src/compiled_manifest.rs)
- count=0 regs=0 `_RINvNtCs8B6tsDSUbde_12cgap_parquet17compiled_manifest7plain_wKt80_EB4_` (cgap-parquet/src/compiled_manifest.rs)

### cgap-p28

- count=0 regs=0 `_RNCNCNvNtNtCsjq6mOKUeUQ7_8cgap_p284trap5testss_23spoliation_loader_courts_00B9_` (cgap-p28/src/trap.rs)
- count=0 regs=0 `_RNCNvNtNtCsjq6mOKUeUQ7_8cgap_p284trap5tests16run_loader_childs0_0B7_` (cgap-p28/src/trap.rs)
- count=0 regs=0 `_RNCNvNtNtCsjq6mOKUeUQ7_8cgap_p284trap5tests16run_loader_childs1_0B7_` (cgap-p28/src/trap.rs)
- count=0 regs=0 `_RNCNvNtNtCsjq6mOKUeUQ7_8cgap_p284trap5tests16run_loader_childs2_0B7_` (cgap-p28/src/trap.rs)
- count=0 regs=0 `_RNCNvNtNtCsjq6mOKUeUQ7_8cgap_p284trap5tests16run_loader_childs_0B7_` (cgap-p28/src/trap.rs)
- count=0 regs=0 `_RNCNvNtNtCsjq6mOKUeUQ7_8cgap_p284trap5testss_38check_returns_zero_iff_intrinsic_valid0B7_` (cgap-p28/src/trap.rs)
- count=0 regs=0 `_RNvMs4_NtCsjq6mOKUeUQ7_8cgap_p286asc450INtB5_11Asc450MeterpE13ceiling_centsB7_` (cgap-p28/src/asc450.rs)
- count=0 regs=0 `_RNvNtCsjq6mOKUeUQ7_8cgap_p286asc45010bytes32_eq` (cgap-p28/src/asc450.rs)

### cgap-ffi

- count=0 regs=0 `_RNCNvNtNtCs2Ysa0NuUWki_8cgap_ffi9sovereign18wave_f2_p28_courtss_43verify_cdylib_symbol_ta` (cgap-ffi/src/sovereign.rs)
- count=0 regs=0 `_RNCNvNtNtCs2Ysa0NuUWki_8cgap_ffi9sovereign18wave_f2_p28_courtss_43verify_cdylib_symbol_ta` (cgap-ffi/src/sovereign.rs)
- count=0 regs=0 `cgap_batch_rows` (cgap-ffi/src/lib.rs)
- count=0 regs=0 `cgap_version` (cgap-ffi/src/lib.rs)
- count=0 regs=0 `_RNvNtNtCs2Ysa0NuUWki_8cgap_ffi9sovereign12wave9_courts4tail` (cgap-ffi/src/sovereign.rs)
- count=0 regs=0 `_RNvNtNtCs2Ysa0NuUWki_8cgap_ffi11oabi_lookup13surface_court4tail` (cgap-ffi/src/oabi_lookup.rs)
- count=0 regs=0 `_RNCNCNvNtNtCs2Ysa0NuUWki_8cgap_ffi10xip_tables9r2_courts15oabi_shape_gate0s_0B9_` (cgap-ffi/src/xip_tables.rs)
- count=0 regs=0 `_RNCNvNtNtCs2Ysa0NuUWki_8cgap_ffi10xip_tables9r2_courts11cargo_build0B7_` (cgap-ffi/src/xip_tables.rs)

### cgap-gpu

- count=0 regs=0 `_RNvMCspR2o67DNLK_8cgap_gpuNtB2_17MetalSharedBuffer12from_mmu_bufB2_` (cgap-gpu/src/lib.rs)
- count=0 regs=0 `_RNvMCspR2o67DNLK_8cgap_gpuNtB2_17MetalSharedBuffer8is_emptyB2_` (cgap-gpu/src/lib.rs)
- count=1 regs=3 `_RNvMCspR2o67DNLK_8cgap_gpuNtB2_17MetalSharedBuffer10raw_handleB2_` (cgap-gpu/src/lib.rs)
- count=1 regs=9 `_RNvMCspR2o67DNLK_8cgap_gpuNtB2_17MetalSharedBuffer14from_raw_parts` (cgap-gpu/src/lib.rs)
- count=1 regs=3 `_RNvMCspR2o67DNLK_8cgap_gpuNtB2_17MetalSharedBuffer3lenB2_` (cgap-gpu/src/lib.rs)
- count=1 regs=4 `_RNvMCspR2o67DNLK_8cgap_gpuNtB2_17MetalSharedBuffer8from_bufB2_` (cgap-gpu/src/lib.rs)
- count=1 regs=5 `_RNvXs_CspR2o67DNLK_8cgap_gpuNtB4_17MetalSharedBufferNtNtNtCs20ztldruFRR_4core3ops4drop4Dr` (cgap-gpu/src/lib.rs)
- count=1 regs=15 `_RNvNtCspR2o67DNLK_8cgap_gpu5testss_34test_metal_shared_buffer_zero_copy` (cgap-gpu/src/lib.rs)

### cgap-bont

- count=0 regs=0 `_RNvMNtCskYC7u0EHw3h_9cgap_bont10shacl_maskNtB2_9CorpusRng3new` (cgap-bont/src/shacl_mask.rs)
- count=0 regs=0 `_RNvMNtCskYC7u0EHw3h_9cgap_bont10shacl_maskNtB2_9CorpusRng5below` (cgap-bont/src/shacl_mask.rs)
- count=0 regs=0 `_RNvMNtCskYC7u0EHw3h_9cgap_bont10shacl_maskNtB2_9CorpusRng8next_u64` (cgap-bont/src/shacl_mask.rs)
- count=0 regs=0 `_RNvMs_NtCskYC7u0EHw3h_9cgap_bont10shacl_maskNtB4_8MaskPlan3new` (cgap-bont/src/shacl_mask.rs)
- count=0 regs=0 `_RNvMs_NtCskYC7u0EHw3h_9cgap_bont10shacl_maskNtB4_8MaskPlan8evaluate` (cgap-bont/src/shacl_mask.rs)
- count=0 regs=0 `_RNvNtCskYC7u0EHw3h_9cgap_bont10shacl_mask11brute_force` (cgap-bont/src/shacl_mask.rs)
- count=0 regs=0 `_RNvNtCskYC7u0EHw3h_9cgap_bont10shacl_mask13rdf_type_slot` (cgap-bont/src/shacl_mask.rs)
- count=0 regs=0 `_RNvMNtCskYC7u0EHw3h_9cgap_bont14shacl_compilerNtB2_10ReasonCode4name` (cgap-bont/src/shacl_compiler.rs)

### cgap-bosym

- count=0 regs=0 `_RNvMs4_NtCs3U2S9mEhCDq_10cgap_bosym4slabNtB5_11GraphExport8is_empty` (cgap-bosym/src/slab.rs)
- count=0 regs=0 `_RNvXs_NtCs3U2S9mEhCDq_10cgap_bosym4slabNtB4_6Sha256NtNtCs20ztldruFRR_4core7default7Defaul` (cgap-bosym/src/slab.rs)
- count=0 regs=0 `_RNCNvNtNtCs3U2S9mEhCDq_10cgap_bosym4slab14ingress_courtss_30court_ingress_wired_into_pars` (cgap-bosym/src/slab.rs)
- count=0 regs=0 `_RNvNtCs3U2S9mEhCDq_10cgap_bosym4slab17fibo_graph_export` (cgap-bosym/src/slab.rs)
- count=0 regs=0 `_RNCNvNtNtCs3U2S9mEhCDq_10cgap_bosym4slab5testss_39emit_real_vocabulary_archive_when_gated` (cgap-bosym/src/slab.rs)
- count=0 regs=0 `_RNCNvNtNtCs3U2S9mEhCDq_10cgap_bosym4slab5testss_39emit_real_vocabulary_archive_when_gated` (cgap-bosym/src/slab.rs)
- count=0 regs=0 `_RNCNvNtNtCs3U2S9mEhCDq_10cgap_bosym4slab5testss_42t2_truncation_fails_closed_at_every_off` (cgap-bosym/src/slab.rs)
- count=0 regs=0 `_RNCNvNtCs3U2S9mEhCDq_10cgap_bosym4slab17fibo_graph_export0B5_` (cgap-bosym/src/slab.rs)

