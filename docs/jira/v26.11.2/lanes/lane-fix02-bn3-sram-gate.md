# Lane FIX-02 — cgap-bn3 gated in verify_cscb_sram.py (RED-12 closure)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Owner files:
`scripts/verify_cscb_sram.py` only.

## 1. Subject

RED-12 (`docs/jira/v26.11.2/redteam/red-12-sram-overflow-exploit.md:27-42,
51-75`) found the SRAM gate's subject list ungated cgap-bn3: an 8-atom
cyclic join within every `NA_MAX_*` const bound charges static
NaEngine+NaMatches state tables of 6,520,984 B = 49.75x the 131,072 B
ceiling, silently. This lane adds cgap-bn3 as a GATED subject with a
static capacity/charge table.

## 2. Diff

`scripts/verify_cscb_sram.py`:
- `DEPENDENCY_CRATES` now includes cgap-bn3 (missing-dir check exits 2).
- New `BN3_SOURCE` + red-12 host `size_of` pins
  (`BN3_PIN_MATCHES_BYTES=2162696`, `BN3_PIN_ENGINE_BYTES=4358288`,
  red-12 W5).
- `parse_na_consts` reads `NA_MAX_*` consts from noalloc.rs (parsed
  from source of truth, never hardcoded); missing consts = exit 1.
- `bn3_capacity_table` static model: NaMatches = 8 + 4*A*M + 8*V*M;
  NaEngine = 2*(4*A*M + 8*V*M) + 2*8 (cur_len/nxt_len) + V + 4*ROWS
  (A=NA_MAX_ATOMS, M=NA_MAX_MATCHES, V=NA typed VARS, ROWS=8192).
  Divergence from the red-12 size_of pins = typed refusal (exit 1) —
  a stale charge model can never run silently.
- `bn3_enclave_verdict`: over ceiling = loud `BN3_ENCLAVE_REFUSED`
  typed gate failure, exit 1.
- CLI: `--subject {baseline,bn3}` (default baseline — documented
  workloads unchanged), `--mutate bn3-pin` court, hidden
  `--bn3-source` court seam.
- Baseline pass line now carries a report-only bn3 admission verdict.

## 3. Commands + exits (this session, exact subject)

- `python3 scripts/verify_cscb_sram.py --subject bn3` → exit 1,
  `BN3_ENCLAVE_REFUSED: static noalloc state-table charge 6520984 B
  (NaMatches 2162696 B + NaEngine 4358288 B) > 131072 B hard bound
  (49.75x ceiling; red-12 topology)`.
- `python3 scripts/verify_cscb_sram.py` (baseline) → exit 0; measured
  36048 B (matches red-12 W2/W5 referent); headroom 95024 B; report-only
  bn3 line: "in ceiling: NO — admission REFUSED".
- `python3 scripts/verify_cscb_sram.py --mutate bn3-pin` → exit 0,
  court flipped: NA_MAX_MATCHES 2048→1024 mutant trips the red-12
  size_of pin check (exit 1, DIVERGED, "consts drifted?").
- `python3 scripts/verify_cscb_sram.py --mutate sram-bloat` → exit 0,
  pre-existing court still flips (136068 B > 131072 B).
- Sandbox court (all under /tmp/fix02-sandbox, shared tree untouched):
  injected the red-12 topology into a /tmp copy of the gate
  (`default="bn3"  # RED-12-INJECTED`), ran → exit 1 with the loud
  BN3_ENCLAVE_REFUSED line; restored sha256-exact
  (5a3215c9...34be469) + mtime bump; corrupted-restore probe detected
  (sha e58db8e0... != baseline — restore-detection non-vacuous).

## 4. Anti-vacuity ledger

- bn3-pin court: mutation = NA_MAX_MATCHES halved in a sandboxed
  noalloc.rs copy; flip = DIVERGED refusal exit 1. Witnessed.
- sram-bloat court (pre-existing, B04): mutation = static 100 KB blob;
  flip = measured 136068 B > bound. Witnessed this session.
- Sandbox shim court: mutation = bn3 admitted as enclave subject;
  flip = gate pass→exit 1 refusal. Witnessed.
- Corrupted-restore non-vacuity: deliberate post-restore corruption
  detected by sha256 mismatch. Witnessed.

## 5. Standing

ALIVE. Every claim above cites a command+exit run this session against
the exact subject. Perf numbers: none claimed.

## 6. Fences (must-not-regress)

- Baseline B04 gate (36048 B, exit 0) — rerun this session, unchanged.
- sram-bloat anti-vacuity court — rerun this session, flips.
- House exit ABI (0/1/2/3) preserved; no lib.rs/Cargo.toml touched; no
  git commands run; build leases /tmp/cscb-sram-{target,caller} and
  /tmp/fix02-sandbox deleted via shutil at lane end (exit 0).

## 7. Divergences

- Dispatch said "sizes from noalloc.rs consts, statically" and named
  6,521,032 B — that figure is the riscv32imc LINKED-ELF charge
  (.text 56 B + .rodata 6520976 B, red-12 W5 cmd 2); the static struct
  model gives 6,520,984 B (host size_of ground truth, red-12 W5 cmd 1).
  The gate charges the struct model and cross-checks against its pins;
  both figures exceed the ceiling by ~49.75x — no behavioral
  difference in the verdict.
- One mid-edit token corruption of the gate script (dropped
  `_CALLER_MAIN` opener) was caught by ast.parse + --help before any
  gate run; fixed in-session, no other lane affected.
