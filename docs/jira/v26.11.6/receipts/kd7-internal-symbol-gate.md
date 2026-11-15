# KD-7 closure receipt — `_cgap_p28_invariant_check` declared INTERNAL with a gate

Standing: ALIVE (Linux x86_64 container, 2026-11-15, omniplatform thread)

## Close condition (register text)

"GAP-1 resolves the question; if the symbol is intended to be public, mint the
header prototype; if not, declare it internal with a gate."

## Resolution

INTERNAL. The public surface already carries the deliberate un-prefixed
`cgap_p28_invariant_check` (prototyped in `include/cgap.h`, exported by the
cdylib, covered by the 1:1 rule). The leading-underscore variant stays
internal: `crates/cgap-ffi/src/sovereign.rs` doc comments already say so.

Gate: `scripts/ffi_surface_gate.py` gains `INTERNAL_SYMBOLS =
{"_cgap_p28_invariant_check"}`. `run_gate` fails (exit 1) when any internal
symbol has a header prototype (comments stripped) or is exported as a text
symbol by the cdylib. Self-check mutation 4 appends
`int _cgap_p28_invariant_check(void);` to a /tmp sandbox header copy and must
flip the gate to FAIL; the copy is restored and sha256-verified.

Side fixes in the same file (needed to run it off the original host): `REPO`
derives from the script location (`CGAP_REPO` overrides) instead of
`/Users/sac/cgap`; the hostile-shim mutation builds with `cc -shared -fPIC`
on non-Darwin hosts instead of `clang -dynamiclib`.

## Command and output

`python3 scripts/ffi_surface_gate.py --self-check` -> exit 0

```
[GATE] PASS: 20 exported cgap_* symbols == 20 header prototypes (1:1)
[SELF-CHECK] baseline (unmutated subject) -> exit 0 (want 0) ok
[SELF-CHECK] mutation: fake prototype injected -> exit 1 (want 1) ok
[SELF-CHECK] restore verify (fake_proto.h byte-exact) -> exit 0 (want 0) ok
[SELF-CHECK] mutation: cgap_version prototype dropped -> exit 1 (want 1) ok
[SELF-CHECK] restore verify (drop_proto.h byte-exact) -> exit 0 (want 0) ok
GATE-FAIL: internal symbol on the public surface (KD-7): _cgap_p28_invariant_check
[SELF-CHECK] mutation: internal _cgap_p28_invariant_check prototyped -> exit 1 (want 1) ok
[SELF-CHECK] restore verify (internal_proto.h byte-exact) -> exit 0 (want 0) ok
[SELF-CHECK] mutation: hostile 1-symbol shim dylib -> exit 1 (want 1) ok
[SELF-CHECK] corrupted restore is detected (fail-closed) -> exit 0 (want 0) ok
[SELF-CHECK] final re-restore byte-exact -> exit 0 (want 0) ok
rc=0
```
