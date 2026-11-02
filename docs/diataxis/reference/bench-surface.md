# Bench surface

Manifest: `crates/bench/Cargo.toml` (harness = false, divan 0.1.21).
All numbers are TARGETS/receipts from the cited wave docs, measured on
Apple Silicon, divan 0.1.21, bench profile — not fresh measurements.

## chains-* targets (five)

Declared once by BENCH-1 (Cargo.toml:72-97 comment block).

| Target | Receipt | Chain result (measured) |
|---|---|---|
| `chains-ingress` | `waves/bench-chains-ingress.md` | 1,048,576 frames: stage sum 109.7 ms (digest 93.4 / penstock 9.7 / A04 6.6) vs chain mean 102.7 ms; ~89 ns/frame hashing of ~98 ns total |
| `chains-parquet` | `waves/bench-chains-parquet.md` | 1,048,576 rows: chain 41.5 ms = encode 5.1 (12%) + decode 0.75 (2%) + seal/verify/flush ~35.7 ms (86%, FNV-1a byte-wise) |
| `chains-bn3` | `waves/bench-chains-bn3.md` | statement_scan_24b 80.32 GB/s (65,536 stmts, 19.58 us); stage_scan_24b 12.27 GB/s at 1,048,576 |
| `chains-seal` | `waves/bench-chains-seal.md` | governance_chain_per_row 2.378 us/row at N=1024, 2.373 us/row at N=65536 (flat tail, <=0.3% drift) |
| `chains-sovereign` | `waves/bench-chains-sovereign.md` | N=4096: commit 164.4 ms (87.6%), eval 14.36 (7.7%), verify 3.70 (2.0%), admit 3.362 (1.8%); LEASE_EXHAUSTED boundary at lane 65377 |

## Component and prior-campaign targets

| Target | Cargo.toml line | Role |
|---|---|---|
| `v26_10_31` | :29 | prior-campaign surface (retained) |
| `kernels` | :36 | Phase-2 scaffold stub (F2-10) |
| `ingress` / `parquet` / `bn3` / `seal` | :40/:44/:48/:52 | component benches (F2-11..F2-14); seal prints `[SKIP] FFI sovereign C-ABI leg PENDING` for its dlsym gate |
| `challenger_parquet` / `challenger_arrow` / `challenger_git` / `challenger_crypto` | :57-:69 | challenger sales-report targets |

## Reproduce commands (from receipts)

- `CARGO_TARGET_DIR=/tmp/p316-target cargo bench -p bench --bench chains-ingress`
- `CARGO_TARGET_DIR=/tmp/p317-target cargo bench -p bench --bench chains-parquet`
- `cargo bench --manifest-path crates/bench/Cargo.toml --bench chains-bn3`
  (isolated `CARGO_TARGET_DIR=/tmp/p318-target`, deleted post-run)
- `CARGO_TARGET_DIR=/tmp/p319-target cargo bench --bench chains-seal`
- `CARGO_TARGET_DIR=/tmp/p319-target cargo bench --bench chains-sovereign`

## Anti-vacuity (per receipts)

- chains-ingress: every timed pass asserts its folded checksum against
  the untimed preflight fold; corrupting any slot byte fires the assert.
- chains-bn3: differential oracle (raw-word O(N) hash-join, no unify
  code in the oracle path); projected row counts == match counts.
- chains-seal: final-state asserts per iteration (validated == N,
  licensed == N/2).
- chains-sovereign: LEASE_EXHAUSTED boundary bisect is run untimed at
  startup; refusal is typed, never silent.
