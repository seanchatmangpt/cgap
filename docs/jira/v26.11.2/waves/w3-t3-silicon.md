# w3-t3-silicon

Status: ALIVE (cgap-arrow audit-profile riscv32imc object, 0/0 purity) /
PARTIAL (cgap-blake3, cgap-parquet: built, census has undefined external
refs, no defined alloc/panic symbols) / BLOCKED (cgap-git, cgap-gate,
cgap-bn3, cgap-p28, cgap-crypto; wasm-footprint) — measured this session,
2026-11-01.

## Method and OBJECT vs ELF distinction

- Target riscv32imc-unknown-none-elf, toolchain nightly (rustc 1.101
  nightly, LLVM 23). Host llvm (Homebrew LLVM 21) cannot read nightly
  LLVM 23 objects; rust-toolchain-internal llvm-size/llvm-nm/llvm-ar used
  (`$(rustc --print sysroot)/lib/rustlib/aarch64-apple-darwin/bin`), same
  toolchain-internal discipline as gate-5-silicon.md:27-28.
- OBJECT (rlib archive bytes) vs ELF: the release profile sets
  `lto = "thin"` (Cargo.toml:48), which makes the rlib members LLVM
  BITCODE, not ELF ("file was not recognized as a valid object file"
  witnessed; this is the v26.11.1 "lto bitcode variant: 0 object bytes"
  class, gate-5-silicon.md:16). The v26.11.1 30,157 B class was measured
  with `--profile audit` (lto=false), which yields real ELF rcgu.o
  members. All sizes below are therefore `--profile audit` pre-link
  OBJECTS (llvm-size -A over the rcgu.o members; rlib archive bytes given
  separately). NO linked ELF exists in this lane — no bin target was
  linked; the linked-ELF subject remains authoritative per
  paper-audit.md:43 and is NOT claimed here.
- Census: alloc-family = alloc_error_handler / __rust_alloc /
  __rust_dealloc / __rust_realloc / __rust_oom; panic-family = panicked /
  panic_. Defined symbols (T/t/W/w) vs undefined refs (U) counted
  separately; the 0/0 purity law applies to DEFINED symbols.

## Flash budget table (32,768 B budget; pre-link OBJECT totals, audit profile)

| crate | rlib OBJECT bytes | llvm-size Total | .text | .rodata | .data/.bss | vs 32,768 budget |
|---|---|---|---|---|---|---|
| cgap-arrow | 34,198 | 28,679 | 28,354 | 1 | 0/0 | 28,679 = 87.5% PASS |
| cgap-blake3 | 126,490 | 114,072 | 113,360 | 388 | 0/0 | OVER (3.48x) pre-link lib total; no gc/ELF subject |
| cgap-parquet | 164,200 | 55,711 | 48,446 | 5,537 | 0/0 | OVER (1.70x) pre-link lib total; no gc/ELF subject |
| cgap-git | — | — | — | — | — | BLOCKED (E0463 std) |
| cgap-bn3 | — | — | — | — | — | BLOCKED (syntax error, unify.rs:1146) |
| cgap-gate | — | — | — | — | — | BLOCKED (no #![no_std]; 38 errors) |
| cgap-p28 | — | — | — | — | — | BLOCKED (no #![no_std]; E0463) |
| cgap-crypto | — | — | — | — | — | BLOCKED (no #![no_std]; E0463) |
| wasm-footprint | — | — | — | — | — | BLOCKED (registry dep) |

Blake3 breakdown: cgu.0/1/2 text; dominant unit is the blake3 compression
core in cgu.1-class units; the per-crate lib object is a whole-crate
total, not a kernel-function total — the v26.11.1 class measured the
ARROW kernel object (blake3::hash 26,047 B inlined in arrow's cgu.1),
which reproduces this session at 26,046 B (-1 B, within the named
compile-output drift class of paper-rewitness.md:13). .data/.bss = 0
across all three (SRAM .data+.bss term = 0, matches gate-5:106-107).

## nm purity census (rust-toolchain llvm-nm, audit-profile rcgu.o)

| crate | alloc DEFINED | panic DEFINED | alloc refs (U) | panic refs (U) | verdict |
|---|---|---|---|---|---|
| cgap-arrow | 0 | 0 | 0 | 0 | 0/0 PASS (purity law met) |
| cgap-blake3 | 0 | 0 | 0 | 1 (core::panicking::panic_bounds_check, U) | 0/0 defined; 1 undefined external ref |
| cgap-parquet | 0 | 0 | 22 (all U: __rust_alloc/dealloc/realloc/zeroed, alloc shim) | 15 (all U: panic_bounds_check, panic_fmt) | 0/0 defined; refs only |

All census hits in blake3/parquet are UNDEFINED externals resolved by
core/alloc at link time (bounds-check and raw-vec paths in the parquet
lib; one bounds-check ref in blake3). NO alloc_error_handler, NO
__rust_oom, NO panicked/panic_fmt is DEFINED in any object — the linked
image would pull core's panic/alloc machinery only if those refs survive
gc-sections, which is exactly what the authoritative linked-ELF subject
must measure. CRITICAL bar (defined alloc/panic symbol present) NOT met
anywhere. cgap-parquet referencing alloc at all means the parquet lib is
core+alloc (heap-backed paths), NOT a pure no-core-alloc kernel — flagged,
not laundered.

## BLOCKED rows (file:line + exact command + exit)

- cgap-git: `cargo build --profile audit --target riscv32imc-unknown-none-elf
  -p cgap-git` -> exit 101; crates/cgap-git/src/lib.rs:13
  `extern crate std;` — E0463, riscv32imc ships no std. Session-relevant:
  does not build on this target today.
- cgap-bn3: same command form, exit 101; syntax error
  crates/cgap-bn3/src/unify.rs:1146 "unexpected closing delimiter `}`"
  (block opened at :1119 `for m in 0..cur {`). PRE-COMPILATION parse
  failure — file is broken in the tree, independent of target.
- cgap-gate: same command form, exit 101; 38 errors; crate lacks
  `#![no_std]` (E0463 + prelude errors: `Option`/`Some` not in scope at
  crates/cgap-gate/src/penstock.rs:516,604).
- cgap-p28: same command form, exit 101; no `#![no_std]` (E0463,
  "std is required by cgap_p28 because it does not declare #![no_std]").
- cgap-crypto: same command form, exit 101; no `#![no_std]`
  (E0463, same note for cgap_crypto).
- wasm-footprint: `cargo build --release --target wasm32-unknown-unknown`
  (cwd crates/wasm-footprint, standalone) -> exit 101; "failed to select a
  version for the requirement `cgap-arrow = ^26.11.2`; candidate versions
  found which didn't match: 26.11.1, 26.10.29" — 26.11.2 is NOT on
  crates.io (workspace is pre-publish for 26.11.2). The 4,629 B receipt
  class is not re-measurable until publish or a [patch] to the local
  arrow. Read-only lane discipline: no manifest edits made.

## v26.11.1 comparison

- cgap-arrow object class: 30,157 B (gate-5) / 28,355 B (PP-5 rerun) vs
  28,679 B this session — within the named compile-output drift band
  (text 28,354; blake3::hash-dominant cgu.1 26,046 vs 26,047 in
  paper-rewitness.md:13). Session metadata/doc edits do not affect
  codegen; NO code-level growth attributable to session changes observed
  on the arrow subject.
- blake3/parquet whole-lib objects have no v26.11.1 comparable class in
  the receipts read this session — recorded as first baselines, both over
  the 32,768 per-artifact budget as PRE-LINK lib totals (non-authoritative
  subject class).
- release-profile thin-LTO makes release rlib members bitcode: any future
  flash-size claim must name the profile. The v26.11.1 witness class
  (conflating rlib object bytes with linked-ELF flash) is respected:
  OBJECT and ELF kept separate above.

## Commands + exits (this session)

- per-crate release riscv builds: blake3/arrow/parquet exit 0;
  git/bn3/p28/crypto exit 101 (logs /tmp/w3t3-*.log, deleted with lease)
- per-crate audit-profile riscv builds: blake3/arrow exit 0; parquet
  exit 0; git exit 101
- llvm-ar x / llvm-size -A / llvm-nm over extracted rcgu.o: exit 0
- wasm build: exit 101 (registry dep), twice (workspace-root attempt:
  "package ID specification wasm-footprint did not match any packages";
  standalone attempt: version select failure above)

## Cleanup

- Build-root lease /tmp/w3-t3-target deleted via python3 shutil.rmtree;
  /tmp/w3t3-ext extraction dir deleted; /tmp/w3t3-*.log removed.

```json
{"lane":"W3-T3","date":"2026-11-01","repo":"/Users/sac/cgap",
"verdicts":{"cgap-arrow":"ALIVE","cgap-blake3":"PARTIAL","cgap-parquet":"PARTIAL",
"cgap-git":"BLOCKED","cgap-gate":"BLOCKED","cgap-bn3":"BLOCKED","cgap-p28":"BLOCKED",
"cgap-crypto":"BLOCKED","wasm-footprint":"BLOCKED"},
"sizes_bytes":{"cgap-arrow":{"rlib_object":34198,"llvm_size_total":28679,"text":28354,"rodata":1,"budget":"32768","pct":87.5},
"cgap-blake3":{"rlib_object":126490,"llvm_size_total":114072,"text":113360,"rodata":388,"budget":"OVER"},
"cgap-parquet":{"rlib_object":164200,"llvm_size_total":55711,"text":48446,"rodata":5537,"budget":"OVER"}},
"census":{"defined_alloc_panic":"0/0 all three","undef_refs":{"arrow":0,"blake3":1,"parquet":"alloc=22 panic=15"},
"critical":false},
"v26_11_1_compare":{"arrow_total":28679,"prior":"30157/28355","drift":"within named compile-output drift band"},
"profile_note":"release=thin-LTO(bitcode members); audit=ELF objects; no linked ELF measured",
"commands_exits":"per-crate cargo builds 0 (built) / 101 (blocked); llvm-size/nm exit 0; wasm 101",
"cleanup":["/tmp/w3-t3-target","/tmp/w3t3-ext","/tmp/w3t3-*.log"]}
```
