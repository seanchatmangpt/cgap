# ERRC slice evaluation AD

Lane C2, v26.11.14. Range: crate dir names starting a-d; 29 dirs. Subject /home/user/cgap/crates; every cite is file:line re-read this session (paths relative to that dir). "no candidate found" means nothing grounded was located; none invented.

| crate | action | candidate | cite |
|---|---|---|---|
| audit | no candidate found | cdylib wrapper exports; nothing dead or duplicated located | audit/src/lib.rs:1 |
| bench | REDUCE | blanket allow(dead_code) on `mod provenance`; narrow or drop | bench/src/main.rs:16 |
| cbmqs-cli | REDUCE | local `hex` duplicates cgap-aoq `hex` | cbmqs-cli/src/cert_io.rs:565 |
| cgap | ELIMINATE | stub `fn main() {}` with allow(dead_code) for non-none targets | cgap/src/bin/enclave_self_play_entry.rs:14-16 |
| cgap-aoq | REDUCE | `hex` also in cbmqs-cli and cli handlers; one shared helper | cgap-aoq/src/rfc0089_certificate.rs:347 |
| cgap-arrow | REDUCE | blanket `#![allow(dead_code)]` on generated layout hides unused items | cgap-arrow/src/generated_layout.rs:10 |
| cgap-beam | REDUCE | `BatchEnvelope` carries dead_code allow awaiting readers of ptr/bit_width | cgap-beam/src/resources.rs:129 |
| cgap-blake3 | ELIMINATE | `hash_many_simd_4way` has a definition and zero callers in the crate | cgap-blake3/src/lib.rs:868-869 |
| cgap-bn3 | REDUCE | private crc32_ieee is one of 5 CRC-32/IEEE copies (also wire-shm, journal-export, storage, git) | cgap-bn3/src/project.rs:607 |
| cgap-bont | no candidate found | oabi/primitives/shacl modules; no dead or duplicate item located | cgap-bont/src/lib.rs:2-5 |
| cgap-bosym | no candidate found | slab-only crate; nothing grounded | cgap-bosym/src/lib.rs:1 |
| cgap-branchless | REDUCE | eytzinger fns kept dead_code only for differential courts; cfg(test)-gate them | cgap-branchless/src/eytzinger.rs:43 |
| cgap-core | ELIMINATE | ATTESTATION_LEN twin (KD-3; other copy cgap-p28 attestation.rs:60); lane E1 owns it | cgap-core/src/frame.rs:44 |
| cgap-court | no candidate found | single test file, test-only crate | cgap-court/tests/shared_fixtures_court.rs:1 |
| cgap-crypto | REDUCE | module-wide `#![allow(dead_code)]` for Wave-3 PQC consts; scope to the consts | cgap-crypto/src/chatman.rs:24 |
| cgap-ffi | REDUCE | module-wide `#![allow(dead_code)]` in import_arrow; scope to items | cgap-ffi/src/import_arrow.rs:25 |
| cgap-fleet-ops | no candidate found | host membrane; no dead or duplicate item located | cgap-fleet-ops/src/slo.rs:25 |
| cgap-gate | no candidate found | single penstock module; nothing grounded | cgap-gate/src/lib.rs:14 |
| cgap-git | REDUCE | splitmix64 duplicates cgap-storage dred.rs:34; frozen seed constants stay put | cgap-git/src/retract.rs:40 |
| cgap-gpu | no candidate found | macOS-gated bridge; build.rs keys on CARGO_CFG_TARGET_OS correctly | cgap-gpu/build.rs:4 |
| cgap-mcp | no candidate found | no non-test unwrap/expect found by grep | cgap-mcp/src/lib.rs:23 |
| cgap-ocel2 | no candidate found | standalone projection membrane; nothing grounded | cgap-ocel2/src/tap.rs:234 |
| cgap-p28 | ELIMINATE | ATTESTATION_LEN twin (KD-3) with cgap-core frame.rs:44; const asserts at :87-88 | cgap-p28/src/attestation.rs:60 |
| cgap-parquet | no candidate found | no dead_code allow or duplicate located | cgap-parquet/src/lib.rs:23 |
| cgap-receipt | no candidate found | single engine plus receipt_print bin | cgap-receipt/src/lib.rs:199 |
| cgap-storage | REDUCE | crc32 copy in atomic_commit; splitmix64 copy in dred.rs:34 | cgap-storage/src/atomic_commit.rs:387 |
| cgap-verification | no candidate found | not a root member; nothing grounded | cgap-verification/src/lib.rs:1 |
| cli | REDUCE | `hex` copied in handlers_kernel.rs:10, handlers_sovereign.rs:6, verify_v26.rs:362 (hand seams only; generated files untouched) | cli/src/custom_handlers/handlers_kernel.rs:10 |
| csct | REDUCE | splitmix64 copy-pasted in three bins (c7:48, c4:44, c1_c2:52); move function only, seeds frozen | csct/src/bin/cscb_c4_alloc.rs:44 |
