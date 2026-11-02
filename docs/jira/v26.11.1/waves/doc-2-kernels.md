# DOC-2: Published kernel crate READMEs (receipt)

Lane DOC-2-kernels, 2026-11-01, repo /Users/sac/cgap. Lease:
crates/{cgap-blake3,cgap-arrow,cgap-git,cgap-parquet}/README.md +
docs/jira/v26.11.1/readme-archive/ + this receipt.

## Archive (pre-rewrite snapshots, sha256)

| crate | archive path | sha256 |
|---|---|---|
| cgap-blake3 | readme-archive/cgap-blake3.md | 069bc78646096cb31607b3a675e810776a04cb620fa6fb08283c2f3077184716 |
| cgap-arrow | readme-archive/cgap-arrow.md | 1a010d6c58e2f04c1ced30f04204a36b7a9195909ea92467f471ca17ec15159c |
| cgap-git | readme-archive/cgap-git.md | 762d995301de3515fd4092b9cb2c4943acceb3c7a1c852c5c651f582171e4ffd |
| cgap-parquet | readme-archive/cgap-parquet.md | 89912301a048957b22935811897089824be41bf3eb0d340d474beb678b78e1b8 |

## Per-crate section lists

- cgap-blake3: title; what-it-is paragraph; Status (PUBLISHED-26.11.1);
  Evidence (27/0 via r12-bench-tmp.md; KAT suites official_vectors.rs et
  al.; 5-suite census + bench_tmp purge); Install (cargo add
  cgap-blake3@26.11.1); Lane receipt (r12-bench-tmp, f3-09-pub-blake3,
  p3-api-verify-all).
- cgap-arrow: title; paragraph; Status; Evidence (30,157 B riscv32imc rlib
  gate-5-silicon.md; purity 0/0 a01-finish-receipt.md; host 177/0
  a01-finish-receipt.md; riscv build exit 0); Install; Lane receipt
  (a01-finish, a01-wave9, gate-5-silicon, p3-api-verify-all).
- cgap-git: title; paragraph (dual-register); Status; Evidence (88/0
  a03-finish-receipt.md; SHIM-OFFSET/STABILITY/PARENT flips; dual-register
  round-trip court); Install; Lane receipt (a03-finish,
  a03-wave5-audit-receipt, p3-api-verify-all).
- cgap-parquet: title; paragraph; Status; Evidence (0 B alloc decode
  bench-chains-parquet.md; +27% staged-vs-direct bench-parquet-receipt.md;
  216/236 suites a02-finish-receipt.md; 46.45 us direct);
  Install; Lane receipt (a02-finish, bench-parquet-receipt,
  p3-api-verify-all).

## Divergences from dispatch

1. Registry witness for PUBLISHED-26.11.1: dispatch said cite
   f3-15-api-verify.md re-sweep; that receipt's verdict is FAIL
   (26.11.1 absent at both its sweeps 01:22:56Z / 01:59:00Z, 2026-11-02).
   The PUBLISHED status is witnessed by waves/p3-api-verify-all.md instead
   (all five kernels PASS, max_version 26.11.1). Both cited.
2. Arrow host suite is 177/0 in a01-finish-receipt.md:134, not 178/0 as
   dispatched; 178/0 not found in any waves receipt. Cited 177/0.
3. blake3 "27/0" is the post-purge suite count in r12-bench-tmp.md:49, not
   a court verdict; the KATs live in tests/official_vectors.rs (canonical
   test_vectors.json recipe). Described as such.
