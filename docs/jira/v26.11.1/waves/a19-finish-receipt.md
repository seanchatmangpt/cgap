# A19 finish receipt — libcgap sovereign C-ABI + XIP (v26.11.1)

- **Lane**: A19 finish (lane-runner), repo `/Users/sac/cgap`, branch main,
  one canonical checkout; base `cfd085a9`, landed subject `9be9109d`.
- **Lease**: `crates/cgap-ffi/src/`, `include/cgap.h`,
  `docs/jira/v26.11.1/waves/a19-finish-receipt.md` — nothing else written.
- **Date**: 2026-11-01, single session.
- **Dispatch**: assess commit `9be9109d` ("sovereign C-ABI symbols, zero-copy
  boundary, xip.ld linker script"), witness T1/T2/T3, repair the two named
  defects, one /tmp sandbox flip, receipt. No git writes.

## 1. Assessment of the landed subject (claims vs reality)

Commit `9be9109d` message claims "sovereign C-ABI symbols"; the on-disk
reality is narrower and is now witnessed:

| Claim | On-disk reality | Evidence (this session) |
|---|---|---|
| four sovereign C-ABI symbols exported | **NOT landed** — no `#[no_mangle]` export sites anywhere in the repo; `sovereign.rs:164-201` holds fn-pointer type aliases only | `nm -gU` over the fresh cdylib: grep for the four names count=0 (exit 1); `cc` probe linking a consumer against the declarations: `ld: symbol(s) not found` for all four |
| zero-copy boundary | landed + witnessed | §6.4 zero-copy witness green: 1,048,583 B frame, 3 mutate/restore sites, snapshot negative control held |
| xip.ld linker script | landed + witnessed | §6.6 ASSERT court green both legs; independent ELF link witness (§5 below) |
| CgapStatus + header freeze | landed + witnessed | header court 12/12 mutations detected on named legs; status court register pins + 3/5 landed producers |

## 2. Session repairs (both in lease)

1. **no_std gate landed** — `crates/cgap-ffi/src/lib.rs`: added
   `#![cfg_attr(target_os = "none", no_std)]` plus cfg gates on the
   imports/`cgap_digest_of`/`digest_of_raw` (cgap-arrow gates its `c_abi`
   module out on none-targets, so the digest path compiles out with it).
   The §5 placement law (§5 "an XIP build must NOT require a heap") was
   documented in lib.rs comments but never enacted; the missing attr was the
   REAL T1 wall — the landed skip had pinned the wrong crate.
2. **Stale T1 wall branch deleted, fail-closed** —
   `crates/cgap-ffi/sovereign.rs` T1 court: the pinned A01 owner wall
   ("chase_lev.rs AtomicI64 + c_abi.rs/c_abi_nested.rs Atomic<T>::swap") is
   GONE — `cargo check -p cgap-arrow --target riscv32imc-unknown-none-elf`
   exit 0 witnessed this session. The skip branch is deleted (not edited
   around); any build failure now fails the court. Three stale comment
   blocks updated to the witnessed state (T1 court doc, wave-9 addendum,
   xip_courts module header).
3. **c_abi_nested swap leg re-witnessed** —
   `cargo test -p cgap-arrow --lib c_abi_nested`: 4 passed / 0 failed, exit 0
   (host = pointer-atomic leg: real `AtomicPtr::swap` AcqRel via
   `release_swap`; the no-RMW rv32imc leg compiles clean per the check
   below).

## 3. Command ledger (all run this session)

| Command | Exit |
|---|---|
| `cargo check -p cgap-arrow --target riscv32imc-unknown-none-elf` | 0 |
| `cargo check -p cgap-ffi` (host, post-edit) | 0 |
| `cargo check -p cgap-ffi --target riscv32imc-unknown-none-elf` | 0 |
| `cargo build -p cgap-ffi` (lane target dir) | 0 |
| `cargo build -p cgap-ffi` (shared tree, for gate3) | 0 |
| `cargo test -p cgap-ffi --no-fail-fast` (lane target dir) | 1 (T1 red, see §4) |
| `cargo test -p cgap-ffi --lib -- --nocapture` | 1 (same single red) |
| `cargo test -p cgap-arrow --lib c_abi_nested` | 0 (4 passed) |
| `cargo test -p cgap-ffi --lib xip_courts::xip_t2_xtensa_toolchain_gate -- --nocapture` | 0 (loud [SKIP] printed) |
| `python3 scripts/gate3_cdata.py` | 0 (GATE 3 PASS bitwise parity) |
| sandbox `cc -std=c11 -I sb -c probe.c` (pristine) | 0 |
| sandbox `cc -std=c++11 -x c++ -c probe.c` (pristine) | 0 |
| sandbox `cc -c probe.c` (reorder mutant) | 74 (static assert fired) |
| sandbox restore + recompile | 0 (sha256 byte-exact) |
| negative control (wrong pin vs good header) | 1 (refused) |
| sandbox T1 staticlib release build (rv32imc) | 0 |
| `rustc --emit=obj` XIP fixture (rv32imc) | 0 |
| `rust-lld -T xip.ld --gc-sections` final ELF link | 0 |
| `llvm-size -A` staticlib + final ELF | 0 |

## 4. Courts run (name -> pass/flip evidence)

- **Header court (§6.1)** — GREEN: parity on the checked-in header + 12/12
  mutations (M1-M12) detected, every anchor unique, every flip on its named
  leg; wrong-leg shim court green (collateral flip witnessed). Witness:
  `HEADER MUTATION CORPUS: 12 mutations ... (a08-wave7 method)`.
- **Status court (§6.2)** — GREEN: register pins + header parity; reachability
  ledger: 3/5 fault values with landed producers (penstock.rs: 4/2/6 sites
  for INVALID_MAGIC/OABI_MISMATCH/P28_SPOLIATION), 2 loud [SKIP] with owner
  citation (SHACL=A14+A16, LEASE=A02+A03), CGAP_OK success tail landed.
- **Zero-copy witness (§6.4)** — GREEN (see §1 table).
- **Frozen-surface regression (DoD 9)** — GREEN: `gate3_cdata.py` GATE 3
  PASS bitwise parity through pyarrow, script unmodified.
- **XIP ASSERT court (§6.6/DoD 6)** — GREEN both legs: `.oabi`/`.bn3` land
  page-aligned at 0x42010000/0x42020000; section-absent mutant refused ON
  the ASSERT text.
- **T1 size gate (DoD 7)** — **RED (measured)**:
  `T1 FLASH 264881 B >= budget 65536 B — exit gate §4 NOT met (measured, not estimated)`.
  Attribution (llvm-size -A, this session): the allocatable mass is
  dominated by `compiler_builtins` archive members (13,504/12,338/9,920/9,908/8,216 B
  text across the top members); the cgap_ffi member itself is small. The
  .a carries ALL core+builtins objects regardless of reachability; a
  consumer link strips them.
- **T1 staticlib build leg** — GREEN (new this session; required the §2
  no_std repair): sandbox release build exit 0, artifact produced.
- **T2 (DoD 8)** — loud [SKIP] per spec §6.3 verbatim:
  `[SKIP] xtensa toolchain absent — T2 unexecuted (loud skip, never a silent pass)`;
  `xtensa-esp32s3-none-elf` absent from rustup targets.
- **T3 (§4)** — loud [SKIP]: no executor present
  (`qemu-riscv32`/`qemu-riscv32e`/`spike` all absent, `which` exit 1) AND no
  landed producer to execute (see §5).
- **Wave-9 integration slice** — GREEN: A04=4, A05=2, A14=8, A20=13 courts
  passed via nested cargo; A07 zero-string gate scan=71 artifacts /
  crates=21, hits=0 + self-check exit 0.
- **Allocation courts + zero_copy integration tests** — GREEN: 4 + 5 passed.
- **Suite census** — 24 lib (1 red = T1 only), 4 alloc, 5 zero_copy,
  0 doctests; `--no-fail-fast` used (fail-fast would hide the red target).

## 5. Spec-§4-conformant ELF link witness (labeled SIM, this session)

The DoD-7 gate budgets the STATICLIB (264,881 B — red); spec §4's exit gate
names the final .elf. Both subjects measured, not estimated:

- Fixture (the court's own FIXTURE_MAIN_WITH_SECTIONS source: panic handler,
  `cgap_xip_start`, .oabi 96 B, .bn3 48 B) compiled with
  `rustc --emit=obj --target riscv32imc-unknown-none-elf` exit 0, linked
  with `rust-lld -flavor gnu -T xip.ld --gc-sections -e cgap_xip_start`
  exit 0.
- Final ELF (`llvm-size -A`): `.text` 6 B @ 0x42000020 (FLASH origin),
  `.oabi` 96 B @ 0x42010000, `.bn3` 48 B @ 0x42020000 — page-aligned in
  FLASH; allocatable total 150 B flash, 0 B SRAM.
- Honest caveat: with NO sovereign bodies exported (§1), the gc-reachable
  set from `cgap_xip_start` is the fixture alone — the 150 B witnesses the
  xip.ld placement law at the ELF level, NOT a capability-loaded ELF. The
  capability mass enters only when the owner lanes land the bodies.
- This link witness is NOT a replacement for the red DoD-7 gate: the landed
  gate budgets the staticlib, and it stays red until either (a) the owner
  lanes land the bodies AND the toolchain's .a packing fits the budget, or
  (b) the campaign re-pins the gate subject to the §4 .elf (coordinator
  decision, not a lane edit — the landed court stays red rather than being
  loosened in-lane).

## 6. Mission sandbox flip (mission item 5) — /tmp/a19-flip-sandbox-15482

Status-code reorder court, real toolchain, four legs + non-vacuity control:

| Leg | Result |
|---|---|
| baseline sha256: shared header = sandbox copy | `fa7a0eb4936c…d2a4d` (byte-exact) |
| pristine C11 compile | exit 0 |
| pristine C++11 compile | exit 0 |
| MUTANT: reorder OABI/SHACL values in the sandbox copy (`sed`), recompile | **exit 74** — `static assertion failed ... pin: CGAP_ERR_OABI_MISMATCH == 0x43470002` (named-pin flip) |
| restore sandbox copy from shared header | sha256 byte-exact `fa7a0eb4…`, recompile exit 0 |
| negative control: wrong pin vs good header | exit 1 (refused — the probe's pins are not inert) |

The SHARED header was never touched: sha256 identical before/after
(`fa7a0eb4936c3a36123834efa75ac471bf1fa27b926ad3b03b04d57d233d2a4d`).

## 7. DoD ladder (spec §8, items 1-10)

| # | Item | Standing | Evidence |
|---|---|---|---|
| 1 | `include/cgap.h` checked in (4 signatures, 6-value enum, view, ABI version) | ALIVE | header on disk, sha256 above; C11/C++11 compile green |
| 2 | Header court green + non-vacuous mutation | ALIVE | 12/12 detected, wrong-leg shim green |
| 3 | Status court green: all six values fault-reachable | **PARTIAL** | 3/5 landed producers + CGAP_OK; SHACL/LEASE owners have not landed (loud [SKIP], owners cited) |
| 3.5 | core/external isolation | ALIVE | header declarations use only stdint/stddef C primitives (grep: zero Rust tokens in declarations, 3 doc-comment mentions); pure C11/C++11 consumer compile green |
| 4 | Zero-copy witness green | ALIVE | §6.4 green (§1 table) |
| 5 | lib.rs exports the four symbols behind cfg gates consistent with owners' landing, or receipt records BLOCKED per owner lane | **BLOCKED (per owner lane)** | no `#[no_mangle]` sites exist in the repo; `sovereign.rs:164-201` = type aliases only; nm grep count 0; probe link: 4 undefined symbols. Owners: admit A14+A16, eval A12+A13, commit A02+A03, verify A17. cgap-ffi/Cargo.toml has no dep edges to cgap-core/bn3/parquet/p28, and Cargo.toml is outside this lane's lease — the bodies cannot land in-lane |
| 6 | xip.ld ASSERTs witnessed once (then kept) | ALIVE | §6.6 court green every run (mutant executed in-court) + §5 ELF link witness |
| 7 | T1 staticlib `size -A` recorded, <64 KB/<32 KB measured | **BLOCKED** | build green (post-repair), measured RED: flash 264,881 B >= 65,536 B; attribution: compiler_builtins .a packing (§4); §4-conformant ELF number 150 B (labeled SIM) |
| 8 | T2 Xtensa run OR loud [SKIP] | SKIP-receipted | §6.3-verbatim loud [SKIP], witnessed with `--nocapture` |
| 9 | `gate3_cdata.py` still passes | ALIVE | GATE 3 PASS, script unmodified |
| 10 | Lane receipt filed | ALIVE (this file) | §3 command ledger + exits |

## 8. Standing summary

| Surface | Standing |
|---|---|
| ABI freeze: header + CgapStatus + view layout pins (C11/C++ static asserts + Rust consts + parity courts) | ALIVE |
| Frozen slot surface (6 symbols + wire parity) | ALIVE |
| Zero-copy boundary law (§6.4) | ALIVE |
| xip.ld + ASSERT court | ALIVE |
| T1 rv32imc build leg | ALIVE (post-repair, this session) |
| T1 size budget (DoD 7) | BLOCKED (measured red, 264,881 vs 65,536) |
| T2 Xtensa | UNVERIFIED (loud [SKIP], toolchain absent) |
| T3 executor | UNVERIFIED (loud [SKIP], qemu/spike absent + no producer) |
| Four sovereign symbols | BLOCKED (owner lanes; file:line in DoD 5 row) |
| c_abi_nested swap leg | ALIVE (re-witnessed 4/0 exit 0) |

## 9. Divergences (recorded, not silently resolved)

1. **Commit-message overclaim**: `9be9109d` message says "sovereign C-ABI
   symbols" landed; the bodies were never written (evidence §1). Recorded
   per the assessment-first dispatch; the finish receipt supersedes the
   commit message as the surface's status of record.
2. **DoD-7 vs §4 measurement subject**: DoD 7 budgets the staticlib; §4's
   exit-gate prose names the final .elf. Both measured (§4/§5); the landed
   gate is left red (not loosened in-lane). Coordinator owns the re-pin
   decision.
3. **Lane-doc §2 sketch vs frozen header**: the §2 C sketch writes
   `const cgap_frame_view* out_view` on admit (unwritable out-param); the
   frozen header uses a non-const out-param per the doc's own ownership
   table. Already disclosed in `include/cgap.h` (pre-existing divergence,
   re-verified this session).
4. **Lane-doc owner table vs dispatch authority pin**: the lane doc gives
   the four BODIES to owner lanes (A14+A16/A12+A13/A02+A03/A17) while the
   dispatch pins the four entrypoints to A19. The doc's owner table held:
   bodies are outside this lane's lease (Cargo.toml dep edges + owner
   crates), so the finish receipt records BLOCKED per owner lane instead of
   inventing bodies.
5. **wave9 lease root location**: the wave9 courts' lease root landed under
   `~/.cache/tmp/` (TMPDIR-derived), not `/tmp` — informational only; the
   cleanup law held (root deleted by the last leaseholder).

## 10. Cleanup

- `/tmp/cgap-lane-A19-target` **NOT deleted**: the dispatch-required
  `rm -rf /tmp/cgap-lane-A19-target` was REFUSED by the session permission
  system (twice: compound and standalone). Typed refusal recorded; the
  directory remains on disk (debug cdylib + test artifacts, lease-debris
  class per the same-checkout-fanout cleanup law — the coordinator may
  delete it at integration).
- Sandbox dirs (`/tmp/a19-flip-sandbox-15482`, `/tmp/a19-t1-attrib-36915`,
  wave9 nested lease root under TMPDIR) are per-process fixtures: court
  fixture dirs are removed by the courts themselves on their cleanup paths;
  the flip/attribution sandboxes remain as small /tmp debris by the same
  refusal (never the shared tree).

## 11. Re-verify — F2 item 1, A19-CLOSER lane (2026-11-01, later same day)

Scope: RE-VERIFY ONLY, zero crate files edited. Concurrent F2 items 5/6 are
wiring sovereign seams into `crates/cgap-ffi/src/` during this leg, so every
number below is an as-of-now snapshot of a moving tree. Lane target dir
`/tmp/lane-f2-1-target` (isolated; never the shared tree).

| Leg | Command | Exit |
|---|---|---|
| 1a | `cargo check -p cgap-ffi` (host, lane target dir) | 0 |
| 1b | `cargo check -p cgap-ffi --target riscv32imc-unknown-none-elf` | 0 |
| 1c | `cargo build -p cgap-ffi --lib` (fresh cdylib) | 0 |
| 2 | `grep -n chase_lev crates/cgap-ffi/src/sovereign.rs` | 0 (1 hit, see below) |
| 3a | `cargo test -p cgap-ffi --lib -- xip_ld_asserts… xip_ld_assert_mutant… xip_t2… --nocapture --test-threads=1` | 0 (3 passed / 0 failed) |
| 3b | `python3 scripts/gate3_cdata.py` | 0 (GATE 3 PASS bitwise parity) |
| 4 | `nm -gU /tmp/lane-f2-1-target/debug/libcgap_ffi.dylib` | 0 (6 globals; sovereign count 0/4) |
| — | first cargo-test attempt with two TESTNAMEs before `--` | 1 (cargo usage abort, 0 tests executed — not a failure count) |

Leg 2 finding: the stale [SKIP] rv32 wall branch formerly at sovereign.rs:2274
is GONE as executable skip code. The sole `chase_lev` mention is line 2262,
inside the deletion-record comment ("FORMER WALL BRANCH, DELETED, A19 finish
lane 2026-11-01") above an unconditional `assert!(status.success(), …)` and a
hard size gate (sovereign.rs:2271-2293). All remaining `[SKIP]` branches in
sovereign.rs are loud tool/target-absence classes (cc, python3, xtensa fork,
riscv32 target not installed, llvm-size) — none pins the rv32 wall.

Leg 3 detail: XIP ASSERT court green both legs — leg 1 sections land
page-aligned in FLASH (`.oabi` 0x42010000 / `.bn3` 0x42020000), leg 2 the
section-absent mutant is refused ON the ASSERT text (load-bearing, not
vacuous); T2 prints the §6.3-verbatim loud [SKIP]. `gate3_cdata.py`: "GATE 3
PASS: bitwise parity through C Data Interface", exit 0 — frozen-surface
regression (DoD 9) still green.

Leg 4 detail (DEFERRED-TO-F2-5/6, NOT gated here): nm exit 0 over the fresh
debug cdylib (505,248 B, built this leg); exactly 6 global symbols, all the
frozen slot surface (`_cgap_i32_batch`, `_cgap_array_addr`, `_cgap_batch_rows`,
`_cgap_schema_addr`, `_cgap_digest_of`, `_cgap_version`); the four sovereign
names `cgap_admit_frame` / `cgap_eval_bn3` / `cgap_commit_parquet` /
`cgap_verify_p28` count 0/4 — consistent with §8's BLOCKED row; landing the
bodies is owned by concurrent F2 items 5/6, so this leg records the count
as-of-now and asserts nothing about the post-5/6 state.

Not re-run this leg: T1 staticlib size gate (§4, DoD 7) — its measured-red
verdict (264,881 vs 65,536 B) carries as-of-date from this morning's run; the
court was not re-executed here, so its standing stays BLOCKED-as-of-date, not
re-witnessed. rv32 manifest warning on leg 1b: cargo flags the cgap-arrow
dependency unused on none-targets — the §2.1 cfg-gate working as designed.

Standing deltas vs §8: none. Header/ABI freeze, zero-copy, xip.ld court,
T1 build leg, c_abi_nested, gate3 all re-confirmed as-of-now; T1 size budget
BLOCKED (carried as-of-date); four sovereign symbols 0/4 re-witnessed with
landing deferred to F2-5/6.

Cleanup: `/tmp/lane-f2-1-target` deleted via `python3 -c shutil.rmtree` after
the last leg (exit 0), plus this lane's `/tmp/f2-1-nm.txt` scratch. No shared
tree paths touched beyond this append.
