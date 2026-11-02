# W6 lane BGP — budget gate: linked deployment-profile ELFs

Date: 2026-11-02 · Lane: BGP · Repo: /Users/sac/cgap @ main · Branch state:
shared tree carried other lanes' in-flight edits (read-only for this lane
outside its two owned files).

## Order

Add `--profile-deployment` to `scripts/enclave_budget_gate.py`: measure
LINKED deployment-profile ELFs (riscv32imc, z/fat-LTO/cgu1/panic=abort),
not audit rlibs. Rebalanced tier table: Tier0 (admission/attestation, incl.
cgap-blake3) ≤ 48 KiB; Tier1 (arrow) ≤ 48 KiB; Tier2 (parquet/git) ≤ 24 KiB;
Tier3 (C-ABI) ≤ 8 KiB; whole-envelope ≤ 131,072 B. Additive only — legacy
`--check` / `--mutate` / `--self-check` untouched, exit ABI 0/1/2/3
unchanged.

## Files written (ownership)

- `scripts/enclave_budget_gate.py` — added `--profile-deployment`,
  `--subject CRATE=tier`, `--deploy-mutant`; new section between the
  legacy courts and `main()`. No other file edited.
- `docs/jira/v26.11.2/waves/w6-budget-elf-profile.md` (this receipt).

## Method

Per subject crate: generate a minimal no_std caller crate in
`/tmp/bgp-crate/<crate>` (panic_handler, `#[no_mangle] _start`, minimal
bump `#[global_allocator]` — gate/p28/parquet use `extern crate alloc` and
refuse to link without one; the 64 KiB heap static is caller scaffold in
.bss and is NOT charged to the subject), build with
`cargo +nightly build --release --target riscv32imc-unknown-none-elf`
(nightly 1.101.0-nightly (12433c307 2026-10-24); target pre-installed),
then `llvm-size -A` over the LINKED ELF summing .text + .rodata (the
witnessed method; .bss/.data excluded — zero-init RAM / caller scaffold).

Anti-fold discipline (each step witnessed as a smaller vacuum ELF before
the fix): constant inputs let LLVM const-fold the entry API away — blake3
11,136 B / gate 514 B / p28 58 B / parquet 38–126 B were all vacuum; the
prologue now fills the input buffer with volatile+black_box stores and the
snippets observe results via volatile reads. parquet also exposed a real
snag: `ffor_unpack(_, 16, ..)` hits the kernel-set assert (1/2/4/8), so the
caller now calls w=8 and measures the real decode loop.

## Results (measured 2026-11-02, final run)

Audit rlibs: `cargo +nightly build --profile audit -p <crate> --target
riscv32imc-unknown-none-elf` (workspace `audit` profile: lto=false,
codegen-units=16), member Totals summed via llvm-ar x + llvm-size -A.
Note: fat release rlibs embed LLVM-bitcode members that llvm-size refuses;
the audit profile is the parseable audit-subject build.

| crate | audit-rlib (B) | linked ELF (B) | sprawl | tier | budget (B) | verdict |
|---|---|---|---|---|---|---|
| cgap-blake3 | 114,072 | 35,440 | 3.22x | tier0 | 49,152 | OK |
| cgap-gate | 1,048 | 484 | 2.17x | tier0 | 49,152 | OK |
| cgap-p28 | 9,220 | 886 | 10.4x | tier0 | 49,152 | OK |
| cgap-parquet | 55,799 | 2,252 | 24.8x | tier2 | 24,576 | OK |

Envelope: 39,062 / 131,072 B — OK. Gate verdict: BUDGET GATE pass, exit 0.

Divergence from the operator witness (113,360 audit / 40,208 linked):
actual this session 114,072 audit / 35,440 linked — same class and order;
delta is nightly 1.101 vs the witnessed build plus the opaque-input
prologue keeping more real code linked. The actual numbers are the
receipted ones.

## Commands + exits (this session, exact)

Legacy modes intact:
- `python3 scripts/enclave_budget_gate.py --self-check` → 0
  (arms C/P flip; arm M loud [SKIP] with no artifact)
- `--self-check --artifact kernel=<blake3 audit rlib>` → 0 (C, P, M flip)
- `--mutate --artifact kernel=<blake3 audit rlib>` → 0 (mutant refused)
- `--check --artifact kernel=<blake3 audit rlib>` → 1 (114,072 B over the
  16,384 B kernel tier — audit-profile breach verdict, expected)

Deployment mode:
- `--profile-deployment --subject cgap-blake3=tier0 --subject
  cgap-gate=tier0 --subject cgap-p28=tier0 --subject
  cgap-parquet=tier2 --deploy-mutant` → 0; DEPLOY-MUTANT COURT: mutant
  66,176 B > 49,152 — breach flips, anti-vacuity holds
- `--profile-deployment --subject cgap-blake3=tier0 --deploy-mutant` → 0
- `--profile-deployment --subject cgap-blake3=tier9` → 1 (typed refusal)
- `--profile-deployment` (no subject) → 2 (EXIT_MISSING)

## Known-divergence notes

- Mid-run, another lane's in-flight edit left
  `crates/cgap-parquet/src/footer.rs` non-compiling (stray
  `&pub fn dummy() {}` at the write_i32_field call site); one deployment
  run recorded parquet BLOCKED, typed, gate refused all-BLOCKED set with
  exit 1 — the fail-closed path was exercised for real. The owning lane
  repaired it; the final run links parquet normally.
- Gate/p28/parquet need `extern crate alloc`; a caller-side bump allocator
  is a real (non-mock) link prerequisite, documented above and charged to
  the caller, not the subject.
