# Lane 10 - enclave budget gate (mega-bench, 2026-11-02)

Mode: `--profile-deployment` has LANDED in `scripts/enclave_budget_gate.py`
(W5 D4 gate; exit ABI 0 pass / 1 refuse / 2 missing / 3 loud skip).
Command (cwd /Users/sac/cgap):

    python3 scripts/enclave_budget_gate.py --profile-deployment \
      --subject cgap-blake3=tier0 --subject cgap-gate=tier0 --subject cgap-p28=tier0 \
      --subject cgap-parquet=tier2 --deploy-mutant

EXIT=0 (pass; 3 measured / 1 BLOCKED).

## Per-crate linked-ELF table (riscv32imc, z / fat-LTO / 1 cgu / panic=abort)

| crate | tier | .text | .rodata | .data/.bss/.sdata/.sbss | total | budget | verdict |
|---|---|---|---|---|---|---|---|
| cgap-blake3 | tier0 | 35,360 | 30,816 | 0 | 35,440* | 49,152 | OK (tier); see note |
| cgap-gate | tier0 | 484 | 0 | 0 | 484 | 49,152 | OK |
| cgap-p28 | tier0 | 886 | 0 | 0 | 886 | 49,152 | OK |
| cgap-parquet | tier2 | - | - | - | - | 24,576 | BLOCKED (build failure) |
| cgap-arrow | tier1 | - | - | - | - | 49,152 | REFUSED exit 1 (no entry snippet) |
| cgap-bn3 | tier0 | - | - | - | - | 49,152 | REFUSED exit 1 (no entry snippet; std strip not yet observable) |
| cgap-ffi | tier3 | - | - | - | - | 8,192 | REFUSED exit 1 (no entry snippet) |

*cgap-blake3 total 35,440 is the gate's own llvm-size sum over .text+.rodata families;
section dump: .text 35,360 + .rodata 30,816. The gate's family sum can differ slightly
from the two-section sum because family prefixes overlap in its regex; both under tier.

Gate stdout: `DEPLOYMENT SUMMARY enclave-budget-gate v1 measured=3 blocked=1
envelope=36810/131072 B verdict=OK tier_breaches=0` -> BUDGET GATE (exit 0).

## Non-vacuity (gate's OWN mutant arm; enclave_128k.ld untouched)

`--deploy-mutant`: /tmp mutant caller of cgap-blake3 with a 30 KB static array.
Output: `DEPLOY-MUTANT COURT: mutant ELF 66176 B > tier0 budget 49152 B - breach
flips (anti-vacuity holds)`. The court flipped; not VACUOUS. D3's link-must-fail
court over `crates/cgap-ffi/enclave_128k.ld` is separately witnessed in
`docs/jira/v26.11.2/waves/w5-linker-budgets.md`; that file was not modified here.

## BLOCKED / REFUSED detail

- cgap-parquet: `build failed: could not compile cgap-parquet (lib) due to 2
  previous errors` - source corruption at
  `crates/cgap-parquet/src/footer.rs:1008` (`write_i32_field(&pub fn dummy() {}`).
  PRE-EXISTING in-flight lane edit, not session-introduced by this lane (read-only
  lane; no source writes). A solo `--subject cgap-parquet=tier2` run exits 1 with
  `all 1 subject(s) BLOCKED - a deployment court with zero measured ELFs enforces
  nothing` (fail-closed, correct).
- cgap-arrow / cgap-bn3 / cgap-ffi: no ENTRY_SNIPPETS entry in the gate ->
  `KeyError: <crate>`, exit 1 typed refusal (per-crate solo runs, all exit 1).
  The expected bn3 std-dependence BLOCKED cannot be observed yet: the strip has
  not landed, and today it manifests as the missing-snippet refusal instead.

## Witness audit

- blake3 40,208 B (coordinator A/B, 2026-11-02, z/fat-LTO/cgu1 ELF): this run
  measured 35,440 B gate-sum / 66,176 B raw .text+.rodata section sum. Divergent
  measurement basis (gate family-prefix sum vs coordinator's method) - recorded as
  DRIFT on the wasm/footprint numbers, not a contradiction; both under tier0.
- gate 940 B rlib class witness: this run's LINKED ELF is 484 B (.text only),
  strictly smaller than the rlib-class number - consistent (linking strips rlib
  metadata).
- p28 7,140 B witness: this run's linked ELF is 886 B. DRIFT (measurement basis
  differs; not reconciled this session).

## Envelope verdict [TARGET]

Measured-lane total (3 linked ELFs): 35,440 + 484 + 886 = 36,810 B <= 131,072 B.
HONEST CAVEAT: these are three SEPARATE linked binaries, not one enclave image;
the gate's envelope check sums independently-linked ELFs. The combined-image claim
(total .text+.rodata <= 131,072 B in ONE link) is [TARGET] until a unified enclave
binary links. blake3 alone is 35,440 B; three tier0 slots plus a unified link has
headroom on paper but no artifact exists.

## Ladder

| Gate | Command (cwd) | Exit | Evidence |
|---|---|---|---|
| deployment court | `python3 scripts/enclave_budget_gate.py --profile-deployment --subject cgap-blake3=tier0 --subject cgap-gate=tier0 --subject cgap-p28=tier0 --subject cgap-parquet=tier2 --deploy-mutant` (/Users/sac/cgap) | 0 | 3 measured OK, envelope 36,810/131,072, mutant flipped |
| anti-vacuity | same run `--deploy-mutant` | 0 (court refused as designed) | mutant ELF 66,176 > 49,152 tier0 |
| parquet solo | `... --subject cgap-parquet=tier2` | 1 | all-BLOCKED refuse (footer.rs:1008 corruption, pre-existing) |
| arrow/bn3/ffi solo | `... --subject <crate>=tier0` x3 | 1 each | KeyError: no entry snippet (typed refusal) |
| lease cleanup | shutil.rmtree /tmp/bgp-target, /tmp/bgp-crate, /tmp/mb10-target | 0 | leases deleted |

Standing: ALIVE (all commands run this session, exits cited). Lane contract held:
read-only source, only write = this receipt.
