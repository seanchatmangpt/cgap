# Lane TS5 — table-driven DFA: NO-TARGET assessment receipt

Date: 2026-11-02. Lane: TS5. Branch: main, /Users/sac/cgap. No git writes.
Files written: this receipt only. Standing: **NO-TARGET (audit landed, nothing implemented)**.

## Decision

**NO-TARGET.** The dispatch premise ("eliminates nested if/match when parsing
multi-stage penstock frames") is FALSE for the shipped code. No genuine hot
multi-byte sequential stateful parse exists anywhere in the audited surface.
Per the lane contract step (3): land nothing, record NO-TARGET with audit
evidence (file:line per site), note the DFA pattern as library-capability only.

## Audit table

| # | Site | file:line | Per-byte state? | Verdict |
|--:|---|---|---|---|
| 1 | penstock `ingress` hot path | crates/cgap-gate/src/penstock.rs:515-600 | none — straight-line fixed-offset compares; each check is an independent O(1) branch with typed early return, no state variable mutated across bytes/fields | not a stateful parse |
| 2 | digest all-zero scan | penstock.rs:546-553 | single boolean (`digest_all_zero`) set once at first nonzero, early-break; a predicate, not a state machine | not a DFA candidate |
| 3 | `read_u64_le`/`read_u16_le` | penstock.rs:629-640 | fixed 8/2-byte copy loop, no state; the loop shape is load-bearing for bounds-check elimination (measured 4.9 vs 6.3 ns/frame, noted in source at :621-624) | must not be converted |
| 4 | `instrument_header_span` | crates/cgap-p28/src/attestation.rs:290-312 | line-oriented scan (`position(b'\n')` per line), stateless across lines; not hot | not a DFA candidate |
| row | ELF section walk `find_in_elf` | attestation.rs:347-360 | per-section structural walk, fixed 64-byte entries, no per-byte state; fail-closed structural loader, not hot | not a DFA candidate |
| 6 | P28 `from_bytes` | attestation.rs:151-171 | fixed 80-byte record, `copy_from_slice` fixed spans, no loop state | not a DFA candidate |
| 7 | asc450 proof/state parse | crates/cgap-p28/src/asc450.rs:276-291 | fixed-offset field checks with typed `MalformedProof(...)` per site — same lawful shape as penstock rows | not a DFA candidate |
| 8 | eval_bn3 / builtins dispatch | crates/cgap-bn3/src/builtins.rs:525 | compile-time closed `match` over the opcode enum, one site (spec §2 ADR: the closed match is the whole grammar) — already table-equivalent (jump-table codegen) | already table-driven |
| 9 | CGAP_SCHEMA walk | no `CGAP_SCHEMA` symbol in any crate source (grep, 0 hits) | — | no such site exists |
| 10 | cold typed refusals :524-599 | penstock.rs:524-599 | lawful typed per-site branches; error-accumulation DFA form is explicitly barred from weakening these | preserve as-is |

## Why the DFA pattern does not apply

A table-DFA (TRANSITION_TABLE[state][input_class] + arithmetic error
accumulation) pays off when a parse loop carries state across per-byte
iterations. Site 1 (the only hot parse in the repo) carries zero state across
checks; sites 2-7 are fixed-offset or early-exit predicates; site 8 is already
a closed match (jump-table codegen). Converting site 1 would ADD a table load +
state variable to a measured 8.19-9.45 ns straight-line path — a regression
risk, not a lever. The H4/H13 witnesses (8.19-9.45 ns) are consistent with the
code shape at :515-600. A table-DFA belongs in cgap-branchless only as a
future library capability if a genuine hot stateful parse (e.g. a streaming
tokenizer) lands in a later campaign; it is a candidate, not a target, this
wave.

## Verification

- Audit method: direct read of the hot path (penstock.rs:515-640) and grep +
  read of every candidate loop site in cgap-gate, cgap-p28 (attestation,
  asc450, trap), cgap-bn3 (builtins), cgap-branchless (penstock.rs module
  already there; no dfa.rs exists — none needed).
- No tests, benches, or source files were modified; nothing to run. Command
  receipts this session: `wc -l`, `grep -n` sweeps over penstock.rs /
  attestation.rs / asc450.rs / trap.rs / builtins.rs (exit 0 each),
  `sed -n 400,640p` / `sed -n 284,360p` reads (exit 0).
- Anti-vacuity note: this is a negative-result receipt, not a court. Its
  falsifier is concrete: name a file:line hosting a per-byte state variable
  mutated across iterations (e.g. a tokenizer or streaming parser) in any
  cgap crate, and TS5 reopens as an implementation lane against that site
  with the differential court + no-regression bench per the dispatch.
