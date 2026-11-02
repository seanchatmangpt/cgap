# Lane B02 — CSCB-C3 1M-fact transitive-closure SLA (receipt)

Date: 2026-11-02. Repo /Users/sac/cgap (main). Files written:
`crates/csct/src/bin/cscb_c3_closure.rs`,
`crates/csct/Cargo.toml` (one-line dep addition, disclosed below),
this receipt.

## Commands + exits (this session, this host)

| command | exit |
|---|---|
| `CARGO_TARGET_DIR=/tmp/b02-target cargo build --release --bin cscb_c3_closure` (in crates/csct) | 0 |
| `/tmp/b02-target/release/cscb_c3_closure --self-gen` (1M edges, N=7) | 1 (SLA FAIL) |
| same with `--chunk=262144` (earlier binary rev) | 1 (SLA FAIL) |
| `CSCB_MUTATE=radix-clamp CSCB_RUNS=1 ... --self-gen` (1M) | 1 (COURT-FLIPPED) |
| `CSCB_MUTATE=radix-clamp CSCB_EDGES=100000 CSCB_RUNS=1 ... --self-gen` | 1 (COURT-FLIPPED) |
| debug build + run `--self-gen` | 3 (loud skip, typed line) |

## Verdict: SLA FAIL — recorded honestly, not rounded up

- 1M-edge self-gen corpus (500k disjoint two-hop chains, seeded affine
  permutations, SplitMix64 stepped from corpus seed
  0xA20C_0FFE_E5A7_E000; multiplier 0xBF58_476D_1CE4_E5B9 is PRNG
  internals only).
- Fixpoint via cgap-bn3 real entry point
  `unify_to_fixpoint_opt(rules, cols, 64, 4096)` (radix semi-join probe
  is the kernel's default path; I did not rebuild any join).
- closure = 1,500,000 = closed-form expectation (correctness holds).
- Timing N=7: min 42.394 s, median 48.478 s, p99 62.498 s.
- SLA target 25.0 ms: missed by ~3 orders of magnitude. Chunk 262144
  did not help (min 34.1 s, earlier rev). FAIL is the output.
- The dispatch's "prior witness 22.99 ms" did not reproduce through
  `unify_to_fixpoint_opt` at this scale/shape on this host. Whether
  that witness measured a different corpus shape or a lower-level
  columnar path is UNVERIFIED from this lane — recorded as divergence,
  not re-defined.

## Anti-vacuity (both modes recorded)

- Mutation `CSCB_MUTATE=radix-clamp`: 8-bit clamp of the join-key
  partition mask (`key & !0xFF` on s/o words) in the driver input.
- Full scale (1M, runs=1): closure 4,837,284 != baseline 1,500,000 —
  COURT-FLIPPED, exit 1 (201.7 s run time recorded).
- Court scale (100k, runs=1): closure 109,432 != baseline 150,000 —
  COURT-FLIPPED, exit 1.
- A vacuous result (closure == baseline) would exit 1 as
  VACUOUS-COURT; that arm is implemented but not witnessed (the real
  mutation flips).
- Debug profile: `CSCB-C3 SKIP debug-profile ...` exit 3 — loud skip,
  no fake pass.

## Divergences + disclosures

1. `crates/csct/Cargo.toml` gained `cgap-bn3 = { path = "../cgap-bn3" }`
   — not in the named file list; required for the driver to build
   (csct is a standalone workspace, no shared-compile risk). Disclosed
   per compile-freeze SLA law.
2. `CSCB_EDGES` / `CSCB_RUNS` / `--chunk=` knobs added so the
   anti-vacuity court can run at court scale; SLA runs stay pinned at
   1M edges / N=7 (defaults enforce this).
3. Court knobs (CSCB_EDGES=100000) take precedence over the edge-count
   check only via the same `edges` variable — no hidden path.
4. Receipt default location is waves/; dispatch named
   `docs/jira/v26.11.2/lanes/` — recorded per dispatch.
5. Standing: SLA coordinate FAIL (min 42.4 s vs 25.0 ms target);
   correctness + anti-vacuity courts ALIVE (commands+exits above).
   No fence regressions: cgap-bn3 untouched (read-only), no shared
   lib.rs/Cargo.toml edits, root workspace not built.
