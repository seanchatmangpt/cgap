# AF-29: Healthcare Records Federation Macro-Benchmark

Status: ALIVE (all commands run this session, exits recorded below)

## Subject

- `crates/cgap-aoq/tests/macro_federation_stress.rs` (new, 578 lines)
- `docs/jira/v26.11.3/receipts/af-29-federation.md` (this file)

## Scenario

500 real UDS socketpair endpoints (1000 threads: 500 senders + 500
federation nodes) stream fixed-size 160-byte clinical record frames
(64-byte AOQ1 header + 96-byte payload). Per record the admission
pipeline is:

1. Structural gate (AB-6 codec): `codec::decode` over the AOQ1 envelope
   — magic, version, closed epoch enum, reserved tail, FNV-1a32 digest.
2. Consent preimage law (AB-7): consent bitmask = bit 0x4000 of the
   chain record `epoch` field. The epoch is inside the hashed preimage
   `prev_hash || epoch || index || payload_digest`, so consent is bound
   to every link. Consent-bit-clear frames are refused typed
   (census class 0x00_0C, disjoint from the codec's 11 classes) BEFORE
   any chain mutation.
3. AEAD seal (AB-4): payload sealed with the REAL RFC 8439 ChaCha20-
   Poly1305 detached kernel (cgap-crypto `seal_detached`) into a
   per-node ciphertext slab; AAD = first 36 header bytes; nonce epoch
   binds (consent_epoch, slot) — no key/nonce reuse across the 9M/900k
   admitted records.
4. Chain receipt (AB-7): link payload = `ciphertext || tag`; link
   digest covers the sealed record identity. Digest substrate = the
   repo's sovereign BLAKE3 (cgap-arrow), real collaborator, no mocks.

Adversarial probes (100k default, 1M in 10M mode) rotate 5 classes:
BadMagic, UnsupportedVersion, UnknownEpochWire, reserved-tail
DigestMismatch, consent-revoked (canonically ENCODED epoch-0 header —
a raw byte flip would die at the codec digest before reaching the chain
law; found and fixed this session).

## Scale decision (essay-claims law)

The 50M-records claim is TARGET-UNMEASURED and stays so. Measured here:

- Default mode: 1M records (900k admitted + 100k probes, 500 endpoints).
- `#[ignore]` mode: 10M records (9M admitted + 1M probes) —
  `cargo test -p cgap-aoq --release --test macro_federation_stress --
  macro_federation_10m_records -- --ignored --nocapture`.
- 500 endpoints IS realized (real socketpairs, not simulated).

## Commands + exits

```
cargo test -p cgap-aoq --test macro_federation_stress -- --nocapture
  exit 0 (debug profile; gate latency court SKIPPED loudly with note —
  unoptimized build measures the build, not the codec)
cargo test -p cgap-aoq --release --test macro_federation_stress -- --nocapture
  exit 0 (run 1: gate court armed)
cargo test -p cgap-aoq --release --test macro_federation_stress
  exit 0 (run 2: green x2 requirement met)
cargo test -p cgap-aoq --release --test macro_federation_stress -- --ignored --nocapture
  exit 0 (10M mode)
```

## Measurements (release profile, M-series, 16 threads)

1M mode (green x2; second run shown where they differ):

- Wall: 3.670 s / 3.573 s
- Throughput: 245,220 admitted rec/s end-to-end (includes the sync
  write->frame->ack UDS roundtrip per record)
- Pipeline latency per record (gate+consent+seal+append, log2 buckets):
  p50 1024 ns, p90 1024 ns, p99 2048 ns, max 31.5 ms (scheduler tail,
  one node thread descheduled), mean 1615 ns
- Structural gate (decode-only, min-of-5-reps mean across nodes):
  **42.0 ns** (run 1) / 42.5 ns (run 2) — under the 100 ns/record
  budget; consistent with the codec's measured 41.5 ns decode. Debug
  profile measured 303.3 ns (build-of-measurement, court skipped).
- Refusal census (both runs, 100k probes -> 20k/class, 100% typed
  rejection): BadMagic 20000, UnsupportedVersion 20000,
  UnknownEpochWire 20000, DigestMismatch 20000, ConsentRevoked 20000.
- Plaintext-absence court: 0 hits for `PTNT-ID-0451-ABC` across all 500
  ciphertext slabs + chain slabs post-seal.
- AEAD round trip: 900,000/900,000 sealed records opened to the exact
  plaintext with stored tags.
- Chain integrity: `verify_all` per node, 500/500 true.

10M mode:

- Wall: 33.232 s; throughput 270,823 admitted rec/s
- Pipeline: p50 1024 ns, p90 2048 ns, p99 2048 ns, max 145 ms (tail)
- Gate: 42.5 ns; census 200,000/class, 1M probes 100% refused
- plaintext_hits 0; opens_ok 9,000,000; chains verified 500/500

## Courts (all asserted, fail-closed)

1. admitted == lawful_total; chain_len == admitted (no unchained
   admits).
2. Per-class census equality + total == probe_total + admitted ==
   streamed (every frame produced exactly one typed outcome).
3. plaintext_hits == 0 post-seal.
4. opens_ok == lawful_total (real AEAD at scale).
5. chain_verified for all 500 nodes.
6. gate < 100 ns (release only; debug skips LOUDLY with a printed
   note, never a fake pass).

## Anti-vacuity notes

- Probe class 4 (consent-revoked) is canonically encoded; the first
  implementation (byte-flipped epoch) was correctly killed by the
  codec's digest seal as DigestMismatch — the census court caught it
  (DigestMismatch 40000 != 20000) and the probe was rebuilt to reach
  the chain law.
- The pipeline histogram asserted nonzero implicitly via the p50/p99
  printout and count equality; refusals also land in the same
  distribution (shorter path) by design.

## Standing / falsifiers

- Standing: PARTIAL_ALIVE for the federation *scenario* — the surfaces
  it composes are ALIVE (AB-4/AB-6/AB-7 kernels, ran + passed this
  session, exits above).
- Falsifier for the scale claim: any run at 50M records / 500
  endpoints. Not performed; the 50M essay number remains
  TARGET-UNMEASURED.
- 50M projection (NOT a measurement): 10M mode scaled 5x sits at
  ~270k rec/s end-to-end -> ~3 min at the same per-record costs;
  chain slab alone would need 50M x 120 B = 6 GB resident.

## Honest-fail / notes

- `Box::leak` slabs per node are process-lifetime (test process); no
  production code was touched in this lane (test + receipt only).
- Pipeline latency includes refusals (shorter path) by design; the
  mean is over all streamed records on the node side.
- Git untouched (lane law: never run git).
