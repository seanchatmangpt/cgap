# Lane FIX-07 receipt — FFI surface gate + RED-03 header gap closure

Lane: FIX-07 (hardening wave over /Users/sac/cgap, branch main). Owned files:
`include/cgap.h`, `scripts/ffi_surface_gate.py` (new). No git commands run.

## W1 Subject

- `/Users/sac/cgap/include/cgap.h` — 7 prototypes added for the RED-03 §2
  exported-symbol census (lines 172-206 on disk now).
- `/Users/sac/cgap/scripts/ffi_surface_gate.py` — new fail-closed court,
  exit 0/1/2/3 law, --self-check non-vacuity battery.

## W2 Authority

Dispatch contract (FIX-07 lane prompt) + RED-03 receipt
`docs/jira/v26.11.2/redteam/red-03-ffi-shim-decoupling.md` §1/§2.

## W3 Diff

1. Header: forward decls `struct ArrowArray; struct ArrowSchema;` + 7
   prototypes — cgap_i32_batch(int64_t,int64_t)->struct ArrowArray*,
   cgap_schema_addr/cgap_array_addr->size_t, cgap_batch_rows->int64_t,
   cgap_version->const char*, cgap_digest_of(size_t)->uint64_t,
   cgap_p28_invariant_check(const uint8_t*)->uint32_t (NULL halts, noted).
   Signatures derived from crates/cgap-ffi/src/lib.rs:43-138 and
   crates/cgap-p28/src/trap.rs:100. Self-contained (no include of
   cgap_c_data.h); header compiles C11 and C99 clean.
2. Gate: parses cgap_* prototypes (comments stripped first — see W5), nm
   -gU the cdylib, strips macOS underscore, 1:1 set compare.
   --self-check: 4-mutation sandbox battery.

## W4 Commands + exits (this session)

- clang -std=c11 -Wall -Wextra -I include -c header probe — exit 0.
- clang -std=c99 -fsyntax-only — exit 0.
- `python3 scripts/ffi_surface_gate.py` — exit 0: "PASS: 11 exported
  cgap_* symbols == 11 header prototypes (1:1)" against
  /Users/sac/cgap/target/release/libcgap_ffi.dylib.
- `--self-check` — exit 0. Battery (all flips witnessed):
  baseline PASS -> fake-prototype inject exit 1 -> restore sha256-exact ->
  cgap_version-drop exit 1 -> restore sha256-exact -> hostile 1-symbol shim
  dylib (clang, /tmp) exit 1 -> corrupted restore DETECTED -> final
  re-restore byte-exact. /tmp/fix07-sandbox deleted at lane end.
- exit ladder: missing dylib exit 2, missing header exit 2.
- `cargo test -p cgap-ffi` — exit 101: 49 passed, 1 failed
  (see W5 pre-existing). Prior to unblock the crate did not compile.
- `scripts/c99_smoke.sh` — exit 0, "c99 smoke: ALL CHECKS PASSED".

## W5 Courts, fences, defects found

- First self-check run FAILED as designed-against-me: (a) restores were
  compared without being performed; (b) the header comment at cgap.h:42
  ("cgap_version() remains") matched the prototype regex and masked the
  drop mutation — 11==11 vacuously. Fix: strip /* */ and // comments before
  extraction; restore-then-verify. The court catching its own author is the
  non-vacuity witness.
- Compile-freeze SLA unblock (disclose-in-both-directions): sovereign.rs
  (NOT an owned file) was mid-compile-frozen by another lane's in-flight
  edit (OsStr::from_encoded_bytes / PathBuf::into_os_str do not exist on
  this toolchain; absent at HEAD). Minimal unblock at
  crates/cgap-ffi/src/sovereign.rs:898-912: stable Unix OsStrExt
  (OsStr::from_bytes) + compiler-suggested into_os_string();
  into_encoded_bytes() retained. The OWNING lane supersedes this diff on
  landing and must cite this receipt; I cite their landing as owner.
- Fence run: cargo test -p cgap-ffi — 49/50 pass. The one failure
  (wave9_a07_zero_string_gate_over_all_built_crates, GATE-REFUSED
  "sections-unreadable .../a07_ar.*/lib.rmeta_") is DETERMINISTIC in the
  current shared target state (reproduced twice with fresh temp dirs) and
  unrelated to this diff — the header and the python gate are not rustc
  inputs; the court reads /Users/sac/cgap/target/debug rlib debris from
  concurrent lane builds. Pre-existing/foreign; owning A07 lane to chase.

## W6 Standing

- Header prototypes: ALIVE (clang C11/C99 exit 0 this session; gate 1:1
  vs live cdylib exit 0).
- ffi_surface_gate.py: ALIVE (exit 0 baseline, exit 1 x3 mutations, exit 2
  x2 missing, --self-check exit 0 this session).
- cgap-ffi test suite: PARTIAL 49/50 (one deterministic foreign failure,
  W5); compile restored from frozen state by disclosed SLA unblock.
- W7 Replay: python3 /Users/sac/cgap/scripts/ffi_surface_gate.py
  (--self-check for the battery); clang -std=c99 -I include -fsyntax-only
  a TU including cgap.h; CARGO_TARGET_DIR=/tmp/... cargo test -p cgap-ffi.

Divergences: dispatch said "missing 6"; RED-03 §2 enumerates 7 unprototyped
symbols (6 Arrow-slot + cgap_p28_invariant_check) — all 7 were added.
