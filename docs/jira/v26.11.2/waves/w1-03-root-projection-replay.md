# W1-03 Root Projection Replay — Receipt

- **Lane**: W1-03 (root `ggen sync run` replay determinism)
- **Date**: 2026-11-02
- **Repo**: /Users/sac/cgap @ 6801affa (main)
- **Actuator**: ggen@26.10.11, root `/Users/sac/cgap/ggen.toml`
  (`ontology/cgap.ttl` + `gen-templates/` + `cgap-pack`)
- **Falsifier**: two back-to-back `ggen sync run` passes produce different
  bytes on any generated output.

## Commands and exits

| Command | Exit |
|---|---|
| `ggen sync run` (run 1) | 0 |
| `ggen sync run` (run 2) | 0 |
| `ggen sync run` (run 3) + sha256 A/B + `cmp` | 0 |
| `make gate0` (sync x2, sha, cmp, freestanding C header) | 0 |

`make gate0` performs its own two-run sha256 + `cmp` over `GATE0_FILES`
and compiles `include/cgap_c_data.h` under `-std=c99 -ffreestanding
-Wall -Wextra -pedantic`; exit 0, "byte-stable replay proof PASS" +
"header compile PASS".

## sha256 of generated outputs (stable across all runs)

```
01d828b391ca739b5d482dcf76df8b841b4b356412ac8b36d49c697dfbef4f88  include/cgap_c_data.h
bf7f5b54215c81d30f11ec9cbdb1f60bb72a162b978eed506a10a6da49bf2bc1  include/cgap_format_codes.h
96c67c577582d4193513aab686b7a3f7f7a638abe7e12db69004e68f7f5c09c3  crates/cgap-arrow/src/generated_layout.rs
ae0f02f73ce5f4d8fa83c7a6392f9fb3e98754364a53a8ed4d72a1119326939f  crates/cgap-arrow/src/generated_version.rs
2523112746347b90db4834a39ab9d8abcc68dfefc524648d76302444e8e0e8d6  crates/cgap-parquet/src/generated_compiled_manifest.rs
fdf9e27cd21f06c61966c2d9328e85d1bb7ce1035c145f5381dc801d2bb6b1ce  generated/subject-count.txt
```

The first four are the Makefile `GATE0_FILES`; the last two are the
remaining sync outputs named in ggen's `decisions` (all
"skipped: unchanged: content identical" every run). Graph hash
`941065f9ee502b4c...79dba3` identical across runs; `written: []` every
run — the projector is a no-op against the committed state.

## Standing

**ALIVE.** Replay determinism proven on the exact subject (6 outputs,
4 sync runs, `cmp` byte-identical, gate0 exit 0). No drift found; no
CRITICAL determinism finding.

## Divergences

- First manual sha256 pass captured only post-run-2 digests; the strict
  two-sided proof was re-run as hash(A) → sync → hash(B) → `cmp`
  (runs 3). All runs 1–4 in `make gate0` additionally confirm it.
- Lane scope is replay determinism only; vacuity court (mutating a
  /tmp sandbox copy of an input to force `written` non-empty) not run —
  the flip channel would be ggen's input hashing, outside this lane's
  named file set.
