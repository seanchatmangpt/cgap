# bench-chains-seal

Lane P3-19 (BENCH-4, chains-seal), repo /Users/sac/cgap (canonical checkout),
2026-11-01. Lease: benches/chains-seal.rs + this receipt (overwrite of crash
residue). Business case: tamper-evident governance — every admitted row pays
exactly one attestation verify + one liability metering step, then the chatman
receipt chains it into ONE ledger; governance cost per row is a compliance
line item and its predictability (flat per-row tail across consts) is the
product.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/p319-target cargo bench --bench chains-seal` — exit 0.
- `CARGO_TARGET_DIR=/tmp/p319-target cargo bench --bench seal` (component
  attribution, same host/session) — exit 0. Note: seal.rs itself printed
  `[SKIP] FFI sovereign C-ABI leg PENDING` (its dlsym gate) — that leg is not
  part of this chain and the SKIP did not affect these numbers.

## Measured (divan 0.1.21, bench profile, Apple Silicon, timer 41 ns)

chains-seal `governance_chain_per_row` (fastest / median):

| N | fastest | median | per row (fastest) |
|---|---|---|---|
| 1024 | 2.435 ms | 2.448 ms | 2.378 µs/row |
| 65536 | 155.5 ms | 156.8 ms | 2.373 µs/row |

Flat per-row tail confirmed: 2.378 vs 2.373 µs/row (≤ 0.3% drift) — the
governance cost per row is constant in batch size, which is the product.

## Pareto (DERIVED, method stated)

Method: component benches re-run on the same host/session (`--bench seal`),
per-row fastest at N=65536, chain minus components for the meter residual.
The component corpora are seed-compatible but not byte-identical to the chain
corpus, so shares are approximate (± the corpus delta).

| stage | per row | share |
|---|---|---|
| p28 §5 validate | 0.4587 µs | 19.3% |
| chatman seal+verify roundtrip | 0.8357 µs | 35.2% |
| asc450 metering (residual) | 1.0783 µs | 45.4% |

Dominant tax: asc450 metering (~45%), then the chatman crypto pair (~35%);
p28 validate is the cheapest stage. No single stage reaches 80% — the seal
chain is a three-way split, not a one-stage tax.

## Chicago / non-vacuity

- Real collaborators throughout: real 80 B attestations (§5 full ordered
  validate over the pinned instrument span), real Asc450Meter with real
  SignedProof emission + sealed meter states, real chatman Lease (Mutex), the
  receipt chain linked via `from_root` = previous `blake3_digest`.
- Final-state asserts fired per iteration: validated == N, licensed == N/2,
  crossed == N/2, proven == N/2, sealed_states == N/2, verified == N,
  chain head advanced off genesis. A green run without these is impossible;
  the mutation classes are named in the bench header (bit-flip upstream
  receipt -> validate refuse; demote crossing -> no proof emission; flip
  receipt digest -> verify DigestMismatch).
- Seed 0xBF58_476D_1CE4_E5B9 verbatim; corpus is a pure function of it.

## Standing

ALIVE for chains-seal: command + exit 0 this session, numbers above are the
actual output. Derived shares are DERIVED (approximate, method stated).

## Divergences

1. dlsym(RTLD_DEFAULT) cannot resolve #[no_mangle] symbols statically linked
   from an rlib into a Mach-O executable; liveness in the sovereign chain
   receipt is proven by real invocation instead (see bench-chains-sovereign).
2. The meter stage share is a residual (chain − validate − crypto roundtrip),
   not a directly isolated bench; corpora differ across files by construction.
