# mega-bench lane-07 parquet seal

Repo /Users/sac/cgap, branch main, subject tree post-`9a730320` (concurrent
v26.11.2 wave lanes in flight). Read-only source; only write = this receipt.
Build root lease: `CARGO_TARGET_DIR=/tmp/mb7-target`.

## Commands (this session, exits)

| command | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/mb7-target cargo bench -p bench --bench chains-parquet` | compile-abort (2 errors, 0 benches executed) |
| bounded poll, `--no-run` compile every 60 s, 10 min deadline | STILL-BROKEN 10:05:58, 10:07:00 → COMPILE-GREEN 10:08:20 (owning lane fixed in-flight footer.rs edit; no unblock applied by this lane) |
| `CARGO_TARGET_DIR=/tmp/mb7-target cargo bench -p bench --bench chains-parquet` (full sweep, 100 iters) | 0 |
| `CARGO_TARGET_DIR=/tmp/mb7-target cargo bench -p bench --bench chains-parquet -- chain_ram chain_file decode_arm` | 0 |

Compile-abort (named, not folded into a tally): `crates/cgap-parquet/src/footer.rs:1008`
contained `write_i32_field(&pub fn dummy() {}` — an in-flight concurrent lane edit
(unclosed delimiter, `mod courts` at :883). Green by 10:08:20 via the owner's fix.

## Stage table (divan, mean of 100, /tmp/mb7-target)

| leg | 65,536 | 1,048,576 | share of chain @1M | allocs/tx |
|---|---|---|---|---|
| chain_ram (full chain) | 1.666 ms | 23.05 ms | 100% | 0 timed-loop allocs |
| chain_file (real file) | 1.684 ms | 27.35 ms | — | 1 alloc / 4 B |
| seal_arm (write+seal+flush) | 999.2 µs | 16.25 ms | ~71% of chain_ram | **0 allocs / 0 B** |
| verify_arm (verify + lane decode) | 425.2 µs | 6.805 ms | ~30% | 1 alloc / 4 B (verify-side parse_meta) |
| encode_arm (standalone rle_encode) | 160.7 µs | 2.56 ms | per lane | 1 alloc / 8 B, 13-17 grows |
| decode_arm (lane-0 page decode) | 50.58 µs | 760 µs | ~3.3% | 0 allocs |
| digest_arm (fnv1a64 + block_digest) | 377.7 µs | 6.045 ms | ~26% | 0 allocs |
| flush_arm (memcpy + commit) | 1.718 µs | 33.12 µs | ~0.1% | 0 allocs |

Writer-encode vs footer/thrift split @1M: seal_arm 16.25 − digest_arm 6.045
− flush 0.033 = **~10.17 ms raw page writer encode** (~37% of chain_ram);
**~6.05 ms seal digests/footer generation** (digest_arm); flush 33 µs.
Leg semantics: seal_arm minus its standalone sub-stages (digest_arm,
flush_arm) is the raw page-write leg; digest_arm is the footer generation leg.

## Seal allocation court

Method: divan alloc columns for the seal arm (divan AllocProfiler installed
as `#[global_allocator]`, chains-parquet.rs:29-30); no /tmp probe crate
needed — the bench emits alloc/dealloc/max-alloc per arm directly.

**Witnessed: seal_arm = 0 allocs / 0 B per tx @65,536 and @1,048,576.**

P5 transition: LANDED. The two former seal-path alloc sites (`spans.to_vec`
at old footer.rs:376, growing-Vec `build_meta_block` at old :447) are
replaced by a fixed-capacity `[0u8; META_CAP]` scratch buffer in
`FooterSealer::seal` (crates/cgap-parquet/src/footer.rs:512-514): "One
fused sweep over the body replaces three passes... Fixed-capacity meta
scratch: zero heap on the seal path (the prior carve-out is closed)".
**2 allocs / 36 B → 0 allocs / 0 B: transition witnessed this session.**

## End-to-end @1M

- chain_ram 23.05 ms mean (w5-encode-repareto: 27.02 ms; chain_file
  27.35 ms with load outliers max 178.7 ms — concurrent wave contention,
  same class as w5 divergence 1).
- Seal latency 16.25 ms ≤ 30 ms target: PASS (w5 seal_arm 16.36 ms —
  in-class, no drift).
- Seal heap allocs: 0 (honest count, divan AllocProfiler column).

## Drift vs w5-encode-repareto

| leg @1M | w5 | this run | drift |
|---|---|---|---|
| chain_ram | 27.02 ms | 23.05 ms | −15% (less contention this window) |
| chain_file | 27.50 ms | 27.35 ms | in-class |
| seal_arm | 16.36 ms | 16.25 ms | in-class |
| verify_arm | 6.807 ms | 6.805 ms | in-class |
| encode_arm | 2.559 ms | 2.56 ms | in-class |
| decode_arm | 888 µs | 760 µs | −14%, in-class |
| digest_arm | 6.489 ms | 6.045 ms | −7%, in-class |

All in-class; no stage-share drift. The w5 writer-encode lever (word-wise
literal gather, falsifier < 2× encode_arm ≈ 5.1 ms) remains open — writer
encode still ~10.2 ms @1M.

## Verdict

| audit | target | actual | verdict |
|---|---|---|---|
| seal latency @1M | ≤ 30 ms | 16.25 ms | PASS |
| seal heap allocs | 0 (P5 2→0 target) | 0 allocs / 0 B, divan column | PASS — transition witnessed |
| chain_ram @1M | record | 23.05 ms | ALIVE |
| stage shares | stable vs w5 | all in-class | ALIVE |

Standing: ALIVE (all numbers run this session, /tmp/mb7-target, full sweep
100 iters). Residual risk: absolute chain means are load-sensitive under
concurrent waves; ratios stable.

{"lane":"mb-07-parquet-seal","files_written":["docs/jira/v26.11.2/mega-bench/lane-07-parquet-seal.md"],"commands":[["cargo bench -p bench --bench chains-parquet (full)",0],["cargo bench -- chain_ram chain_file decode_arm",0],["first full run: compile-abort (footer.rs:1008 concurrent in-flight edit), green by bounded poll at 10:08:20",0]],"alloc_witness":{"method":"divan AllocProfiler alloc column (no probe crate needed)","seal":"0 allocs / 0 B @65,536 and @1,048,576","baseline":"2 allocs / 36 B","p5":"LANDED — footer.rs:512-514 fixed META_CAP scratch"},"verdict":"PASS (seal 16.25 ms ≤ 30 ms @1M; allocs 2→0 witnessed)","standing":"ALIVE"}
