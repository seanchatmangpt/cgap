# Lane H13 — enclave budget re-witness post-hardening

Date: 2026-11-02 · Lane: H13 · Repo: /Users/sac/cgap @ main · READ-ONLY lane;
sole write = this receipt. /tmp scratch only; no source edits; no
Process.sleep. Nightly: 1.101.0-nightly (12433c307 2026-10-24). llvm-size:
`~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-size`.

## Order

Re-witness the 128 KiB enclave budget envelope post-hardening via
`scripts/enclave_budget_gate.py --profile-deployment` (linked riscv32imc
deployment-profile ELFs, z/fat-LTO/cgu1/panic=abort), attempt cgap-bn3
(no_std via BN3NS) as a new subject, record per-section breakdowns, re-run
the anti-vacuity deploy-mutant arm, and frame coverage honestly per FA10.

## Gate run (this session, exact command + exit)

```
python3 scripts/enclave_budget_gate.py --profile-deployment \
  --subject cgap-blake3=tier0 --subject cgap-gate=tier0 --subject cgap-p28=tier0 \
  --subject cgap-parquet=tier2 --deploy-mutant
```
Exit 0. Gate stdout: envelope 39,184 / 131,072 B; verdict=OK;
tier_breaches=0; measured=4 blocked=0. Mutant arm (same run):
`DEPLOY-MUTANT COURT: mutant ELF 66,176 B > tier0 budget 49,152 B — breach
flips (anti-vacuity holds)`.

Anti-vacuity re-run (single subject): `--profile-deployment
--subject cgap-blake3=tier0 --deploy-mutant` -> exit 0, identical mutant
flip (66,176 > 49,152). Green runs WITH a witnessed mutant flip are
non-vacuous. The final baseline sweep below ran `--profile-deployment`
WITHOUT `--deploy-mutant` (exit 0, same 39,184/131,072 envelope) because
the mutant arm rebuilds into the SAME ELF path and would otherwise leave
the mutant on disk — recorded as a gate measurement-hygiene caveat.

## Per-subject table (linked baseline ELFs; llvm-size -A)

Gate total = sum of section families `.text`/`.rodata` via
`size_elf_sections` (`_DEPLOY_SECTIONS` at
scripts/enclave_budget_gate.py:485; .bss/.data excluded — parquet's 65,540 B
.bss is the caller-side 64 KiB bump-heap scaffold + 4 B state, charged to
the caller, not the subject, per the BGP receipt method). Independent
per-section sweep reconciles exactly with the gate total on every subject.

| crate | tier | .text | .rodata | .bss | gate total | budget (B) | verdict |
|---|---|---|---|---|---|---|---|
| cgap-blake3 | tier0 | 35,344 | 96 | 0 | 35,440 | 49,152 | OK |
| cgap-gate | tier0 | 606 | 0 | 0 | 606 | 49,152 | OK |
| cgap-p28 | tier0 | 886 | 0 | 0 | 886 | 49,152 | 886 ≤ 49,152 OK |
| cgap-parquet | tier2 | 2,252 | 0 | 65,540 | 2,252 | 24,576 | OK |

## Coverage caveat (FA10, mandatory disclosure)

These are MINIMUM-ENTRY-SURFACE binaries: each caller exercises only the
gate's ENTRY_SNIPPETS entry API; everything else is dead-stripped. FA10
coverage vs audit rlib: blake3 31.1% / gate 46.2% / p28 9.6% /
parquet 4.0% (docs/jira/v26.11.2/adversarial/forensic-lane-10.md:51).
The maximal one-link no_std consumer = 102,187 B .text+.rodata (FA10 probe,
docs/jira/v26.11.2/adversarial/forensic-lane-10.md:81) — under the
131,072 B envelope with ~22% headroom; cited here as the
envelope-compatibility witness. This lane did NOT re-measure the maximal
consumer; FA10's figure is cited, not re-derived. Verdict semantics:
"minimum entry surface fits", never "the engine fits".

## New-subject attempts (typed skips, honest)

ENTRY_SNIPPETS (scripts/enclave_budget_gate.py:388) covers exactly
{cgap-blake3, cgap-gate, cgap-p28, cgap-parquet}.

- cgap-bn3: `--profile-deployment --subject cgap-bn3=tier0 --deploy-mutant`
  -> KeyError('cgap-bn3') traceback, exit 1.
  Verdict: [SKIP: no entry snippet] — bn3 NOT measured, not force-passed.
- cgap-arrow: same KeyError, exit 1. [SKIP: no entry snippet].
- cgap-ffi: same KeyError, exit 1. [SKIP: no entry snippet].

Gate defect (recorded, NOT fixed — read-only lane): the missing-snippet
path raises an unhandled KeyError traceback
(scripts/enclave_budget_gate.py:582, `ENTRY_SNIPPETS[crate]`) instead of
the house typed-skip ABI (loud [SKIP], exit 3). The known-crate check in
parse_subject_args (:499) only fires for crates absent from crates/;
bn3/arrow/ffi exist on disk, so the snippet lookup is the first failure.
Dispatcher follow-up: add bn3/arrow/ffi entry snippets or a typed
[SKIP: no entry snippet] guard with exit 3.

## Drift cross-check vs BGP receipt (w6-budget-elf-profile.md:52-59)

| crate | BGP receipt | H13 this run | drift |
|---|---|---|---|
| cgap-blake3 | 35,440 | 35,440 | 0 B |
| cgap-gate | 484 | 606 | +122 B |
| cgap-p28 | 886 | 886 | 0 B |
| cgap-parquet | 2,252 | 2,252 | 0 B |
| envelope | 39,062 | 39,184 | +122 B |

Nightly drift expected per lane order; observed drift confined to
cgap-gate (+122 B, 484 -> 606), consistent with in-flight hardening edits
to crates/cgap-gate/src/penstock.rs on the shared tree — same class and
order, all tier budgets still OK. blake3 35,440 is byte-identical to the
BGP receipt: the 35,440 class anchor holds.

## Standing

- 4 measured subjects ALIVE (gate exit 0 + mutant flip witnessed, this
  session, exact commands above).
- cgap-bn3 / cgap-arrow / cgap-ffi: SKIP (typed, no entry snippet; gate
  currently defects with KeyError rather than a loud skip).
- Maximal-consumer 102,187 B: CITED (FA10), not re-measured.
- No build roots orphaned: lease /tmp/h13-target never created; the gate's
  own /tmp/bgp-crate + /tmp/bgp-target are gate-owned scratch, untouched.
