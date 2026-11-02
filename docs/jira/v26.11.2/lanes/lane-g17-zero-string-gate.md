# Lane G17 — Zero-String Gate Receipt

Date: 2026-11-02. Lane: G17. Subject: `scripts/zero_string_gate.py` @ gate=1.4.2,
canonical tree `/Users/sac/cgap`, branch main. No gate edits (not red).

## Commands + exits (this session, this subject)

| Command | Exit | Verdict |
|---|---|---|
| `python3 scripts/zero_string_gate.py` | 0 | PASS — `SUMMARY scanned=88 crates=22 hits=0 emitter_hits=2 adv_hits=0 unknown=633563 allowed=15707 skipped_artifacts=0 vocab=9 source=graph-derivation:ontologies/*/src/lib.rs gate=1.4.2` |
| `python3 scripts/zero_string_gate.py --self-check` | 0 | PASS — `SELF-CHECK RESULT PASS`; poison arm `hits=2 fired=True`; clean arm `scanned=1 hits=0`; `scan-mutation-witness artifacts=2 mutated=none` |
| `python3 scripts/zero_string_gate.py --root /tmp/g17poison --target-dir /tmp/g17poison/target` (poisoned fixture) | 1 | REFUSE — HIT rows name `fibo:FndUtlCur:USD`; exit 1 |

## Task 1 — scan mode

Exit 0, 22 crates, 88 artifacts scanned, 0 vocabulary hits in fail-set
sections. 2 EMITTER-EXEMPT-HIT rows on `target/debug/emit-fibo-pack`
(`cgap-fibo:emit-fibo-pack`, the Wave-7 producer exemption — loud, counted
in `emitter_hits=2`, non-fatal). 0 [SKIP]ed whole artifacts
(`skipped_artifacts=0`; rlib `lib.rmeta` members loud-[SKIP]ed on stderr as
designed).

## Task 2 — self-check (anti-vacuity)

Exit 0. Poison arm REFUSED-by-HIT (fired=True, 2 hits); clean twin zero
hits; scan read-only witnessed (sha256, mutated=none). The REFUSE flip is
witnessed outside the fixture too, at the exit-code level, by the /tmp
poison court above — quote of the poison-arm HIT line from that court:

```
HIT /private/tmp/g17poison/target/debug/libg17poison.rlib::g17poison-478fbe020eec1f96...rcgu.o __TEXT,__const 1392 fibo:FndUtlCur:USD
```

exit=1 on the same binary that exits 0 clean. A gate that cannot flip
would have exited 0 there; it did not.

## Task 3 — crates/csct exclusion mechanism

`crates/csct` is a STANDALONE crate: its own `Cargo.toml` carries
`[workspace]` (line 11) and the root `Cargo.toml` members array names no
`csct` path. The gate enumerates members mechanically from the root
manifest (`workspace_members()`, zero_string_gate.py:370-392), so csct is
never enumerated — exclusion is by ENUMERATION (not a workspace member),
not by `CRATE_EXCLUSIONS` (which remains empty, line 438-440) and not by a
producer-exemption grant. Witnessed: no `PER-CRATE csct` row in the scan
output; `SUMMARY crates=22`; `grep csct Cargo.toml` empty. Its
format!/vec! use is therefore out of the gate's subject surface entirely;
no exemption needed, no gate edit made.

## Fences this lane could have broken

- Gate scan + self-check are the fences themselves; both re-run green this
  session (exits above). No edits made, so no regression surface
  introduced. The /tmp poison fixture used a symlink to the canonical
  `ontologies/` dir read-only (vocab derivation); canonical tree untouched
  (fixtures live entirely in /tmp, since deleted via shutil.rmtree).

## Divergences / observations

1. Side observation (not a defect on the canonical tree, recorded for the
   coordinator): `EMITTER_BINARIES` keys are global across `--root`, so a
   non-cgap fixture root lacking a built `cgap-fibo:emit-fibo-pack`
   artifact trips `GATE-REFUSED: emitter-exempt-unresolved
   cgap-fibo:emit-fibo-pack` (witnessed live in the /tmp court, printed on
   stderr alongside the HIT rows; exit still 1 via the HITs). On the
   canonical tree the producer is built and the key resolves — gate=1.4.2
   behavior unchanged.
2. Standing: ALIVE for gate=1.4.2 scan + self-check + poison-flip, per the
   command+exit table above. No perf claims. Gate file unmodified this
   lane (0-line diff).
