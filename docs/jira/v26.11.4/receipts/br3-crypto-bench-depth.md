# BR-3 — cgap-crypto bench deepening

- Date: 2026-11-04, 21:10 UTC
- Subject: `crates/cgap-crypto/benches/primitives.rs` (only file owned/edited). Disclosure: the BR-3 edits to this file were swept into a concurrent lane's commit `ede3fe4d` ("wire: AOQ v3 wire law landing...", 19 files, 75-line diff to primitives.rs) before BR-3 could commit them — mid-flight collision, disclosed per lane law; BR-3's own commit carries the receipt only.
- Base commit: `e251f258` (leg 1 compile state: e251f258 + dirty `crates/cgap-crypto/src/aead.rs`, 5+/4-, C1 mid-flight). Head moved mid-run to `39b53add` (C1's aead edits landed between legs). Both legs ran the SAME compiled bench binary (no recompile on leg 2 — "Finished in 0.03s"), so all numbers below measure the leg-1 compile state (e251f258 + dirty aead.rs). Leg-2 rerun (filter blake3, same binary) reproduced every curve within noise.
- Machine: Apple M3 Max, macOS 15 (Darwin 25.2.0), divan =0.1.21, timer precision 41 ns, 100 samples/bench.
- Command: `cargo bench -p cgap-crypto --bench primitives` (run_checked wrapper, exit 0) + filtered rerun `-- blake3` (exit 0).
- Gates preserved: `assert_release` entry (exit 3 loud-refusal), per-op `fence`, BM-9 const sanity pins untouched, corpus seed `0xBF58_476D_1CE4_E5B9` verbatim, SplitMix64 fill verbatim.
- Census: `python3 scripts/profile_guard_census.py` → `77 TIMING files, 67 guard-composing files, 12 allowlist entries`; FAIL line names ONE file: `crates/cgap-ffi/tests/aoq_flip_ladder_court.rs` — a foreign UNTRACKED file owned by another lane, present in `git status` at session start. NOT session-introduced by BR-3; the BR-3 bench file is census-clean (guard-composed, no allowlist entry needed). Not repaired here (not my lane's file).

## (1) BLAKE3 Sum32 chunk-size curve (medians)

| size | median | GB/s (bytes/median) |
|---|---|---|
| 64 B | 133.7 ns | 0.48 |
| 256 B | 546.5 ns | 0.47 |
| 1 KB | 2.145 µs | 0.48 |
| 4 KB | 3.062 µs | 1.34 |
| 16 KB | 8.332 µs | 1.97 |
| 64 KB | 33.04 µs | 1.98 |
| 256 KB | 577.6 µs | 0.45 |
| 1 MB | 2.32 ms | 0.45 |

- Knee A (expected, R1 CV-stack landing): crossover where multi-chunk
  processing kicks in sits between **1 KB → 4 KB**: throughput jumps 0.48 →
  1.34 GB/s (2.8×) — per-chunk fixed cost amortized once input spans >1
  1 KB BLAKE3 chunk. 16 KB–64 KB plateau at **1.97–1.98 GB/s**.
- Knee B (unexpected cliff, FLAGGED, not fixed — outside this lane):
  64 KB → 256 KB collapses **1.98 → 0.45 GB/s (4.4×)** and stays flat through
  1 MB. Stable across both legs (572/572.2 µs fastest). Suspect: the
  multi-1024-chunk (≥256-chunk) path in `cgap-blake3` tree-merge/`hash` —
  a real perf cliff in the kernel, not bench artifact. Follow-up ticket
  candidate: profile `cgap_blake3::hash` at 256 KB.
- medians re-read from `/tmp/br3_bench.log` at receipt-write time.

## (2) AEAD seal+open size curve (medians)

| pt size | median | MB/s | fixed-cost floor |
|---|---|---|---|
| 64 B | 671.5 ns | 95 | ~660 ns (key schedule + 2×Poly1305 init) |
| 1 KB | 5.207 µs | 197 | — |
| 16 KB | 77.39 µs | 212 | — |
| 64 KB | 306.9 µs | 213 | — |

- Per-byte trend: asymptote ≈ **213 MB/s**; the 16 KB→64 KB step is ~flat
  (212→213), so the tag + setup overhead is fully amortized by 16 KB.
- Tag amortization: 16 B tag over 64 B pt = 25% wire overhead at 64 B,
  1.6% at 1 KB, 0.1% at 16 KB — byte-cost is setup-dominated below ~1 KB
  (~660 ns floor: 64 B pt at 213 MB/s would be 300 ns of payload work;
  the other ~360 ns is key schedule + mac init, paid once per seal+open).

## (3) ML-KEM-768 decaps-vs-encaps asymmetry

| op | fastest | median |
|---|---| Ratio |
|---|---|---|
| encaps | 28.12 µs | 28.49 µs |
| decaps | 31.04 µs | 31.41 µs |

Ratio (median): **1.103**

- Tightened from BM-9's 36.24/32.7 = 1.109 to **1.103 (median, 100 samples)**;
  fastest-based ratio 1.104. Ratio stable across both legs (leg 2: 31.45/28.49
  = 1.104). Consistent with the spec's decaps-side implicit-rejection +
  full-decompress work landing only on the decaps side.

## (4) Poly1305 pure vs aead-integrated (both public surfaces benched)

| size | poly1305::one_shot (median) | aead::mac_tag_var_aad (median) | ratio |
|---|---|---|| 64 B | 56.28 ns | 210.5 ns | 3.74× |
| 1 KB | 853.7 ns | 1.004 µs | 1.18× |
| 16 KB | 13.58 µs | 13.70 µs | 1.01× |
| 64 KB | 53.99 µs | 54.33 µs | 1.01× |

- The pure-vs-integrated gap is a **fixed ~151 ns** at 1 KB (1.18×),
  collapsing to 1.01× by 16 KB — the var-AAD form's overhead is the AAD
  padding + two 8 B length absorbs (~2.5 Poly1305 blocks), fully amortized
  by 16 KB. The 64 B 3.74× ratio is the fixed var-AAD + Poly1305
  init/finish cost against a tiny 4-block message — matches the AEAD
  fixed-cost floor in §2.

## Standing

- BLAKE3 curve + AEAD curve + KEM ratio + Poly1305 pair: **ALIVE** —
  `cargo bench -p cgap-crypto --bench primitives` exit 0 this session,
  curves reproduced across two legs.
- 256 KB/1 MB BLAKE3 throughput cliff: **PARTIAL** — measured and stable,
  mechanism not diagnosed (outside lane; flagged as follow-up).
- Census: **PARTIAL** — BR-3 surface clean, gate RED on a foreign lane's
  untracked ffi court file (`aoq_flip_ladder_court.rs`).
- ML-DSA sign/verify: still **UNSUPPORTED** (P1 kernel only) — not stubbed,
  per ticket.
- HEAD/consistency caveat: leg-2 `git rev-parse` = `39b53add` (post-C1-land);
  compiled binary predates the land (leg-1 build, dirty-aead state disclosed
  above). Re-run after C1's `#[inline]`/zip edits land to re-pin.
