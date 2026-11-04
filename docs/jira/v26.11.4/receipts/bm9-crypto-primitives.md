# BM-9 — cgap-crypto primitives first bench

Ticket: BM-9 · Repo: /Users/sac/cgap · Branch: main · 2026-11-04T20:56:45Z
Machine witness: Mac15,9 (Darwin 25.2.0), 16 cores, 48 GB, load 51.17/35.17/24.83
(at bench time; shared fan-out host, other lanes active — numbers are
min-of-samples fastest, divan 0.1.21, bench profile = optimized/release).

## Standing

ALIVE (ran this session: `cargo bench -p cgap-crypto --bench primitives`,
exit 0, table below; `python3 scripts/profile_guard_census.py` exit 0).

## What landed

- `crates/cgap-crypto/benches/primitives.rs` (new, divan membrane) — first
  dedicated bench for the crypto kernels.
- `crates/cgap-crypto/Cargo.toml`: `[[bench]] primitives harness=false` +
  `divan = "=0.1.21"` dev-dep (exact pin, same law as crates/bench).

## Gate receipts

- ZP-18 census gate: `python3 scripts/profile_guard_census.py` → PASS rc=0
  ("76 TIMING files, 67 guard-composing files, 12 allowlist entries"). The
  new bench composes `profile_guard::assert_release` at harness entry
  (loud refusal, exit 3, on a dev build — the gate ran the release/bench
  profile, no SKIP) and `profile_guard::fence` per benched op. **No
  allowlist entry needed**; file is census-guarded, not allowlisted.
- Fence did NOT skip: bench executed and produced real numbers (witness:
  divan output below, timer precision 41 ns).

## Sanity pins (compile-time in-bench court — build fails on drift)

- ML-KEM-768: SS_LEN=32, CT_LEN=1088, EK_LEN=1184 — asserted.
- ML-DSA-65: SIG_BYTES=3309 (chatman constant), PK_BYTES=1952, SK_BYTES=4032
  — asserted (seconded by src/mldsa.rs:101/108 static asserts).
- AEAD: KEY_LEN=32, TAG_LEN=16, NONCE_LEN=12 — asserted.

## Measured (verbatim, divan table)

```
primitives                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ aead_seal_open_1kb            5.999 µs      │ 37.29 µs      │ 6.124 µs      │ 6.529 µs      │ 100     │ 100
├─ blake3_sum32                                │               │               │               │         │
│  ├─ 64                         153.1 ns      │ 163.6 ns      │ 158.3 ns      │ 158.3 ns      │ 100     │ 3200
│  ├─ 4096                       3.52 µs       │ 3.645 µs      │ 3.562 µs      │ 3.576 µs      │ 100     │ 200
│  ╰─ 65536                      38.29 µs      │ 1.164 ms      │ 38.66 µs      │ 75.14 µs      │ 100     │ 100
├─ mldsa65_intt_tomont           1.025 µs      │ 1.046 µs      │ 1.036 µs      │ 1.035 µs      │ 100     │ 800
├─ mldsa65_ntt                   733.9 ns      │ 4.52 µs       │ 749.5 ns      │ 882.5 ns      │ 100     │ 800
├─ mldsa65_pointwise_montgomery  193.5 ns      │ 200 ns        │ 194.8 ns      │ 194.7 ns      │ 100     │ 3200
├─ mlkem768_decaps               36.24 µs      │ 39.2 µs       │ 36.62 µs      │ 36.69 µs      │ 100     │ 100
├─ mlkem768_encaps               32.7 µs       │ 617.7 µs      │ 33.37 µs      │ 47.7 µs       │ 100     │ 100
╰─ mlkem768_keygen               29.41 µs      │ 32.95 µs      │ 29.95 µs      │ 29.99 µs      │ 100     │ 100
```

Derived throughput (fastest, single-shot from the table — UNVERIFIED beyond
arithmetic): blake3 64 KB ≈ 1.75 GB/s; aead round trip 1 KB ≈ 341 MB/s
aggregate (pt through both directions, 6.0 µs for 2 KB moved).

## Honest residue

- ML-DSA-65 keygen/sign/verify top-level does NOT exist yet in
  `src/mldsa.rs` (P1 kernel only: NTT/poly surface, 434 lines). The bench
  measures the landed P1 kernel ops (`ntt` 734 ns, `intt_tomont` 1.03 µs,
  `pointwise_montgomery` 194 ns). Bench-measuring an unlanded sign/verify
  would measure nothing; the file documents this loudly in its header.
  When sign/verify land (Y2's surface), add three benches to this target.
- blake3 65536 slowest/mean show scheduler noise on the loaded host
  (1.164 ms slowest vs 38.66 µs median); fastest/median are the lawful
  read on a shared fan-out machine.

## Commands / exits

- `cargo bench -p cgap-crypto --bench primitives` → exit 0 (8.18 s build,
  bench ran, table above)
- `python3 scripts/profile_guard_census.py` → exit 0 (PASS, bench counted
  guard-composing)
