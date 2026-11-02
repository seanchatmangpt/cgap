# Branchless reuse map — cgap-p28 + cgap-crypto (lane BR2, 2026-11-02)

Read-only reuse survey. Every claim below was re-grepped this session;
per-site verdicts are HOT (branch is on a hot path — substitution
candidate) vs COLD (typed refusal / one-shot — branchy is correct, and
branchless would buy nothing but obscurity).

## 1. saturated_subs_p28 / poison_mask — dead-code-first

`crates/cgap-branchless/src/arithmetic.rs:21,39` — both fns have ZERO
production callers (grep this session: definitions + own tests + census
`crates/cgap-branchless/tests/branchless_census.rs:50` only). FA2/FA6-S3
verdict confirmed: any "substitute into p28/crypto" step is premature
until a caller exists. Consume-then-census.

## 2. Site census (HOT/COLD)

| Site | Class | Verdict |
|---|---|---|
| `cgap-p28/asc450.rs:824-834` plan_event | checked_add→Asc450Error::Overflow | COLD — typed refusal on adversarial input; branchless saturating would CHANGE the spec'd all-or-nothing rejection (asc450.rs:1613-14 anti-vacuity court kills saturating_add mutation). Do NOT substitute. |
| attestation.rs:359-444 ELF/WASM walker | checked_add chains in cold loader path | COLD — runs once per artifact; correctness > branchlessness. |
| penstock.rs:574-599 (≥7 sites) | check-then-return CGAP_ERR_P28_SPOLIATION | COLD — A19 sovereign entrypoint validation rows; single-shot header audit, not a loop. errno paths are adversarial inputs. |
| trap.rs:36 const + :57 abort | 0x5032_38FF halt, `-> !` | COLD — fires at most once per process, by design (§6.1). Branchless-halt substitution is a category error; keep branchy. |
| lease.rs:115-126 `tick()` | check-then-increment on the seal path | HOT-relative — runs once per seal (chatman.rs:562); at p28 volume this is the only monotonic-counter branch in the two crates. Marginal. |

## 3. Tick: a TYPE-level branchless mechanism (stronger than a mask)

`cgap-crypto/src/lease.rs:44-55`: `Tick(u64)` field is private, no `Sub`,
no public raw-value constructor — rewind is UNREPRESENTABLE, which
subsumes what a poison_mask buys (masking an exhausted/quota==0 state) at
the type level. Recorded as the reuse map's strongest mechanism; no mask
substitution proposed.

## 4. Tick-overshoot mutant court (design, not yet built)

- Mutant M1 (counter-overflow): `lease.rs:125` `self.tick.0 = t + 1` →
  `wrapping_add(1)`. Diverges only when ceiling == u64::MAX and t ==
  u64::MAX-1 — unreachable by real iteration (2^64 ticks). Current
  constructors cannot seed a tick. Verdict: UNREACHABLE-MUTANT; a court
  for M1 requires a `#[cfg(test)]` crate-internal tick-seed constructor,
  which is an ABI-surface change — flagged for the A16 owner, not this
  lane. Do NOT ship a court that silently can't flip.
- Mutant M2 (guard-inversion, reachable): invert `t >= tick_ceiling`
  at lease.rs:117. Flip witness: `Lease::new(id, 1, root)` then
  `tick(); tick()` — BASELINE traps on call 2; MUTANT returns 1 and
  emits `PhiLease{tick:1 >= ceiling:1}` through `phi()` — killed by
  lease.rs:137-142's own emit guard AND by asserting the second call
  panics. Real process panic = observable, no mocks. Court status:
  DESIGNED (not run — read-only lane).

## 5. Standing

ALIVE (survey): every site above located by grep + sed this session,
commands+exits witnessed. No substitutions recommended in this lane's
two leased crates; the only HOT site (tick guard) is 2 instructions on
a once-per-seal path where the branch IS the §5 spec.
