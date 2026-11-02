# lane-05-dead-library-gate — AE5 finish receipt

Date: 2026-11-02. Lane: AE5 (production symbol consumption verifier).
Branch: main, canonical checkout /Users/sac/cgap. No git transitions run.

## What landed

- `scripts/verify_dead_library.py` — BR12 consume-then-census law as a
  fail-closed gate. Enumerates every `pub fn` in cgap-branchless from
  source, release-builds the consumer set (cgap-gate / cgap-parquet /
  cgap-p28 / cgap-ffi) into a /tmp lease target, then censuses mangled
  `<len(name)><name>` tokens inside the rcgu OBJECT members of each
  consumer rlib (rmeta excluded — it embeds source strings and would
  false-positive). Linked-binary truth per FA10: a counted token is a
  real symbol reference from production codegen.
- `scripts/dead_library_exceptions.json` — justified-exceptions file;
  every entry carries a justification + consume-then-census plan. A
  stale entry (names no enumerated symbol) exits 1; an unconsumed
  symbol absent from the file exits 1 `DEAD_LIBRARY_SHOWROOM`.
- Exit ABI: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.

## Real run (as-of-now census truth)

```
$ DEADLIB_TARGET=/tmp/ae5-deadlib-target python3 scripts/verify_dead_library.py
census subject : cgap-branchless (19 pub fns)
consumers      : cgap-gate, cgap-parquet, cgap-p28, cgap-ffi
  penstock_prefilter8              refs=2  CONSUMED
  swar_all_zero_vl                 refs=2  CONSUMED
  (remaining 17: refs=0 EXCEPTION-JUSTIFIED)
PASS: consume-then-census law satisfied (2 consumed, 17 justified
exceptions, must-consume set intact)   -> exit 0
```

Release build receipt: `CARGO_TARGET_DIR=/tmp/ae5-deadlib-target cargo
build --release -p cgap-gate -p cgap-ffi -p cgap-parquet -p cgap-p28`
exit 0 (this session). Consumer rlibs located under
`release/build/<crate>/<hash>/out/` (this workspace routes rlibs through
build-script OUT_DIRs; deps/ holds only the ffi dylib).

## Anti-vacuity witnesses

1. MUT-A stub mutant: `pub fn unconsumed_stub()` appended to a /tmp
   sandbox COPY of cgap-branchless -> gate exit 1 naming
   `unconsumed_stub`. Shared tree never written.
2. MUT-B negative control: cgap-parquet object members census ZERO
   branchless tokens (parquet has no branchless dependency) — proves the
   scanner is selective, not all-pass. Over-match => exit 1 VACUOUS.
3. Shim court (must-consume regression): erased all `penstock_prefilter8`
   token bytes in a /tmp COPY of libcgap_gate rlib -> gate exit 1,
   `DEAD_LIBRARY_SHOWROOM: penstock_prefilter8 unconsumed in
   production`. First attempt erased 1 of 2 occurrences and the gate
   correctly stayed green (refs=1 still counted) — a vacuous flip check
   caught and corrected before the flip witness.
4. Stale-exception guard fired during development: `_schema`/`_law`
   metadata keys in the exceptions file tripped exit 1 until metadata
   keys were scoped out of the symbol set — the guard is live, not
   decorative.

## Divergence from lane dispatch (doc yields to reality)

The dispatch asserted `radix_bucket_12` is consumed ("B5 precedent").
The as-of-now census and a tree-wide grep show ZERO production refs
outside cgap-branchless (grep across crates/*.rs: 0 files; gate rlib
object census: refs=0). It ships as a justified exception with the
consume-then-census plan, NOT as a must-consume — putting it in
must-consume would make the gate permanently red on today's truth.
Once consumed, its exception entry must be removed and it graduates to
the must-consume set.

## Divergence: consumer set

parquet and p28 have no dependency path to cgap-branchless; their rlibs
census 0 (that is the MUT-B control). All production consumption flows
through cgap-gate (swar_all_zero_vl + penstock_prefilter8, REP5 swap at
crates/cgap-gate/src/penstock.rs:658,683), which cgap-ffi links. The
gate scans all four consumers regardless and reports coverage honestly.
