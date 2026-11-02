# Lane GIT-09 — Pre-receive Admission Gate — Receipt

Date: 2026-11-02. Branch: main. Files owned/written:

- `crates/cgap-git/src/admission_gate.rs` (new, ~525 lines)
- `crates/cgap-git/tests/admission_gate_court.rs` (new, 6 courts)
- `docs/jira/v26.11.2/lanes/lane-git-09-admission-gate.md` (this receipt)

lib.rs, Cargo.toml: UNTOUCHED (outside the lease). The gate is compiled
via the `#[path = "../src/admission_gate.rs"]` include in the court test
— the same pattern pkt_line_court.rs (GIT-02) uses, because pkt_line.rs
and admission_gate.rs are not yet wired into lib.rs.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/git09-target cargo test -p cgap-git --test admission_gate_court`
  → exit 0, **6/6 passed**.
- `CARGO_TARGET_DIR=/tmp/git09-target cargo test -p cgap-git --test delta_refusal_court --test pkt_line_court --test admission_gate_court`
  → exit 0; delta_refusal **4/4**, pkt_line **10/10**, admission_gate **6/6**
  (must-not-regress fences re-run, green).
- `cargo test -p cgap-git` (full): other lanes' in-flight untracked files
  (`pack_demux.rs`, `retract.rs`, `soc_full_chip_court.rs`,
  `pack_demux_court.rs`, `retract_court.rs`) FAIL TO COMPILE —
  PRE-EXISTING to those lanes, not session-introduced by GIT-09; this
  lane owns none of them (compile-abort ≠ failure count).
- Sandbox flip (standalone crate `/tmp/git09-flip`, isolated target
  `/tmp/git09-flip-target`): always-admit mutation of `admit_update` →
  **courts 2/3/4 FAIL** (3 passed / 3 failed). Non-vacuous.

## Courts (tests/admission_gate_court.rs)

1. sound update admits; ref table advances old→new (state assert).
2. cycle (non-converging 600-node-chain transitive closure, ~4.7k facts
   > 4,096 budget) → typed `CyclicDerivation { rounds: 3, facts: 4772 }`
   AND `RefTable::bytes()` byte-identical before/after (real state).
3. contradiction (functional predicate two objects) → typed
   `DeductiveContradiction` AND table byte-unchanged.
4. unbound head variable → `OntologyRuleBroken(UnboundHeadVariable)`.
5. wire round-trip through the REAL GIT-02 pkt codec (framing + parse +
   admit).
6. non-hex SHA / malformed field count → typed `BadHexSha` /
   `MalformedUpdateLine`.

Flip twins named per court (always-admit, skip-functional-law,
accept-unbound-head). Court 2/3 flip witnessed in the sandbox run above.

## Standing: ALIVE

For the exact subject `crates/cgap-git/src/admission_gate.rs` +
`tests/admission_gate_court.rs`: 6/6 courts green, exit 0, this session.

## Divergences (doc-yields-to-reality)

1. **cgap-bn3 dependency edge ABSENT.** `crates/cgap-git/Cargo.toml` is a
   published zero-dep kernel manifest outside the lane lease; no
   `[dev-dependencies] cgap-bn3` edge could be added. The closure is the
   gate's own bounded monotone-conjunctive fixpoint (BTreeSet membership,
   256-round / 4,096-fact budget — bn3 `FixpointBudgetExceeded`
   semantics). Rebinding to `cgap_bn3::unify::unify_to_fixpoint` is the
   named follow-up when a Cargo.toml lease is granted. NOT bn3 — the
   22.99 ms figure stays a witness context figure, never claimed here.
2. **pkt_line.rs is not wired into lib.rs** (GIT-02 landed the file but
   no `pub mod pkt_line;`). The gate composes it by `#[path]`; a single
   `pub mod` line in lib.rs wires both.
3. Refusal vocabulary composes, does not mint: framing reuses
   `pkt_line::PktError`; refusal names mirror bn3's compile refusals
   (`UnboundHeadVariable`, `EmptyRuleBody`) and
   `FixpointBudgetExceeded` semantics (`CyclicDerivation`). The FA9FIX
   `CGAP_ERR_CONCURRENT_WRITER_DENIED` path (cgap-ffi) is untouched.
4. First flip attempt ran against the shared tree by mistake (cwd reset)
   — vacuous, discarded, and redone in the isolated /tmp crate above.

## Cleanup

`/tmp/git09-target` and `/tmp/git09-flip-target` deleted at lane end;
`/tmp/git09-flip` sandbox removed. Shared tree carries only the three
owned files as changes.
