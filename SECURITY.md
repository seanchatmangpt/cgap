# SECURITY.md

Security policy for the cgap sovereign substrate (v26.11.2). Every
claim below cites its on-disk receipt; UNVERIFIED never rounds up.

## Reporting

Report vulnerabilities to the maintainers directly (private channel;
no public tracker filing for uncoordinated disclosure). Include:
exact subject (file:line), a repro command, and the observed-vs-
expected exit class. Expect a typed response: status vocabulary
ALIVE / PARTIAL / BLOCKED / UNVERIFIED / REFUSED (never rounded up).

## Status and exit law

- Gate exit codes (fail-closed, repo CLAUDE.md "Gate exit codes";
  exit-code contract witness: waves/overnight-readiness.md,
  docs/jira/v26.11.2): 0 = pass, 1 = gate fail, 2 = subject missing,
  3 = loud skip. A skip is loud, never a fake pass.
- A court that cannot flip is FAIL; a mutation-free green gate run
  is VACUOUS (exits nonzero).
- Perf numbers are TARGETs until measured with a stated method.

## Spoliation class

P28-SIL spoliation is the pinned status code
`CGAP_ERR_P28_SPOLIATION = 0x5032_38FF`
(`crates/cgap-ffi/src/sovereign.rs:90`, asserted at :1989). Never
conflate with CGAP_ERR_LEASE_EXHAUSTED 0x4347_0004 (:80) or
CONCURRENT_WRITER_DENIED 0x4347_0006 (:87).

## Conformance baseline

The conformance-kit exporter (`scripts/conformance_kit_export.py`,
lane `docs/jira/v26.11.2/lanes/lane-xc05-conformance-kit-exporter.md`)
inventories 95 courts as of 2026-11-02 (85 cargo-test targets + 5
corpus-doc entries + 5 gate-mode entries). Regeneration witness (X-05
audit): `conformance_kit_export.py` byte-identical across two runs;
live count corroborated by X-05
(`docs/jira/v26.11.2/lanes/lane-x05-court-corpus-audit.md`) and
DOD-12's exporter `--list` (95). The count is re-walked at each
export; a subsequent court drop is inventory 94, never silently 95 —
the corpus doc pins the count
(`docs/jira/v26.11.2/07-court-corpus.md:186-196`).

ERRATA: XC-05's frozen receipt
(`docs/jira/v26.11.2/lanes/lane-xc05-conformance-kit-exporter.md`)
recorded 91 courts (81 cargo-test + 5 corpus-doc + 5 gate-mode) at its
run time; the +4 delta is wave-added courts landing post-XC-05. The
frozen receipt is historical (not rewritten); 95 is the live count.

## Threat boundaries (v26.11.2 corrosion ledger)

Authority: docs/jira/v26.11.2/DEFECT-REGISTER.md RW-09 rows.

- REPAIRED, courts green — rename-mid-lease TOCTOU (ADV-10) and
  symlink key-flip (ADV-17): spill-gate identity keys are now
  (dev,ino) with grant-time verify + rebind + symlink refusal
  (RW-09-5; repair `lanes/lane-r-adv10fix-spill-gate-rename-toctou.md`).
- RESIDUAL, OPEN — RW-09-O2: the verify->create window inside
  claim is kernel-serialization-bound; O_NOFOLLOW|O_EXCL creation
  semantics would close it (finder
  `lanes/lane-r-adv10fix-spill-gate-rename-toctou.md:39-40`). No
  caller-visible fix exists today; do not claim symlink safety
  across that window.
- RESIDUAL, OPEN — RW-09-O1 case-variant portability: identity
  canonicalization is PROTECTED ON THIS HOST ONLY (APFS
  case-insensitive fold collapses cap/X vs cap/x). On case-SENSITIVE
  filesystems the fold does not happen and the alias is unprotected
  (finder `redteam/corrosion/adv-10-fmea-risk-laundering.md:73-78`).
  Close condition: flip-witnessed court on a case-sensitive mount.

## Guard blind-spot: CONTENTION-SUSPECT

Dispatch framing vs disk: the claim "CONTENTION-SUSPECT is 2-of-3
channels" is NOT found on disk and is not asserted here. What IS on
disk (control plan §5.2, `dflss/06-control-plan.md:143-157`):

- CONTENTION-SUSPECT is a stderr-marker control over bench runs
  (marker emitted at `crates/bench/benches/chains-ingress.rs:716-802`,
  mctq copy `mctq_consolidation.rs:31,65,98`), receipted at
  `lanes/lane-r-adv09fix-contention-doc-wiring.md`.
- It is typed-PROCEDURAL (consumer discipline: a marked run VOIDs the
  bench claim, exit law 0 clean / marked = VOID / missing stderr
  disclosure = receipt incomplete) — NOT a machine gate.
- The fail-closed stderr-grep machine gate is a stated
  [TARGET-v26.11.2] and is unlanded. Until it lands, the defense is
  scope qualifiers: every bench-cited figure carries its quiet-host
  scope, and rows citing a marked run are VOID
  (`06-bench-protocol.md` §8).

## Supported platforms (honest note)

Kernels target `#![no_std]` including riscv32imc. The riscv32imc
story is loud-until-bound:

- Width-gated atomics landed for ref_store (commit a0431b70; receipt
  `docs/jira/v26.11.2/lanes/lane-rc02-ref-store-width-arms.md:25-29`).
- Behavioral RMW coverage of implcell remains OPEN (RW-09-O3,
finder `redteam/red-11-riscv-reentrancy-corruption.md:93`). Silicon
footprint figures are pre-link OBJECT measurements, not linked-ELF
truth (measured-claims row 15,
`11-measured-claims.md:39`; method + scope stated there). No host OS
is required by the kernels; FFI/membrane crates are host-bound.
