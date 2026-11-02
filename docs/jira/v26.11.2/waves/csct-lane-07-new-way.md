# CSCT Lane 07 — THE NEW WAY receipt

Lane: CSCT-L7 · Subject: `crates/csct/src/new_way.rs` (+ this receipt)
Date: 2026-11-02 · Branch: main · Standing: **UNVERIFIED:compile-pending**

## Scope

Implemented CSCT Stage 5 THE NEW WAY per RFC-0042 §4 machine-verifiable
capabilities contract:

- `ContractSpec` (Default: footprint 131,072 B; ingress < 10.0 ns/frame;
  closure ≤ 33.3 ms per 1M triples).
- `ArchitectureClaim` (footprint / ingress / closure + has_gc,
  uses_dynamic_vectors, uses_network_serialization).
- `evaluate(&ContractSpec, &ArchitectureClaim) -> (Vec<StateAssertion>, bool)`
  — three numeric assertions plus three typed refusal assertions
  (`refusal_has_gc`, `refusal_uses_dynamic_vectors`,
  `refusal_uses_network_serialization`); any capability flag true fails the
  contract regardless of numbers.
- `pub fn new_way() -> StageResult` (stage_index 5, THE_NEW_WAY), cgap SUT
  claim (39,184 B / 7.37 ns / 22.99 ms / all flags false) folded with a
  cgap_blake3 receipt over the claim + assertions.

## Commands + exits (this session)

| Command | Exit | Result |
|---|---|---|
| `cargo test --manifest-path /tmp/csct-l7-sbox/Cargo.toml` (sandbox: `new_way.rs` + on-disk `types.rs` copy + path-dep `cgap-blake3`) | 0 | 8 passed / 0 failed; includes all 3 lane courts: `new_way_passes_on_cgap_claim`, `mutation_has_gc_flips_verdict`, `mutation_footprint_flips_verdict` — both anti-vacuity mutants flip |
| `cargo test --manifest-path /Users/sac/cgap/crates/csct/Cargo.toml --lib` | 1 | compile-abort: `error[E0583]: file not found for module 'court'` — sibling lane's `court.rs` not yet landed; compile-abort is not a failure count |

Sandbox verification used the on-disk `types.rs` byte copy and the real
path-dependency `cgap-blake3` — real collaborators, no mocks. Anti-vacuity
mutations run against the same `evaluate` the stage function uses; the
footprint mutant asserts exactly one cell flips (5/6 pass), not just the
verdict.

## Blocker (sibling lane, not mine to fix)

`crates/csct/src/lib.rs:17` declares `pub mod court;` but
`crates/csct/src/` contains no `court.rs` (~10+ min of bounded polling,
8 attempts, SLA window elapsed). `lib.rs` and `court.rs` are outside my
owned-file set, so I record rather than write them. Re-run the shared-crate
test after the court lane lands; my module is self-contained and sandbox-
proven.

## Anti-vacuity mutations

1. `has_gc=true` (rest identical) → contract FAIL, typed refusal
   `refusal_has_gc` off — witnessed flip.
2. `static_footprint_bytes=200_000` → contract FAIL, footprint cell off,
   exactly one mutated cell flips — witnessed flip.

## Fences (must-not-regress surface of my diff)

My diff adds one new file and one new module — it cannot regress existing
gates (no shared-file edits, no git transitions, shared tree otherwise
untouched). The one live fence is the shared `csct` lib compile, currently
aborting on the sibling `court` module, independent of this diff.

## Divergences

- Dispatch said receipt status `UNVERIFIED:compile-pending` — confirmed:
  sandbox ALIVE for the module, shared-crate compile BLOCKED on
  `crates/csct/src/lib.rs:17` → missing `crates/csct/src/court.rs`
  (sibling lane in-flight).
- Observed sibling-lane churn this session: `emotional.rs` was mid-edit
  (delimiter error) and was fixed by its owner during my polling; `court.rs`
  still absent at receipt time.
