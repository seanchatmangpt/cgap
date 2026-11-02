# REAL-bench census receipt (lane REAL-bench)

Date: 2026-11-01. Subject: `crates/bench` [[bench]] registry (17 declared targets).
Method: parse `crates/bench/Cargo.toml` `[[bench]]` entries; `cargo build -p bench
--benches` with `CARGO_TARGET_DIR=/tmp/real-bench-target`; per-file census of
`#[divan::bench` fns + `const` declarations; classification real vs shell. Build
root deleted after census (confirmed absent on disk).

## Commands / exits

- `CARGO_TARGET_DIR=/tmp/real-bench-target cargo build -p bench --benches` —
  exit 101 (rustc failure in mat_closure). 16/17 bench targets produced linked
  executables (witnessed via `--message-format=json` compiler-artifact lines,
  `kind: ['bench']`, under `/tmp/real-bench-target/debug/build/bench/<h>/out/`).
- Per-file census: `grep -c '#\[divan::bench'` and `grep -cE 'const [A-Z_0-9]+:'`.
- Cleanup: `shutil.rmtree('/tmp/real-bench-target')` — absent afterward.

## Verdicts (17 declared targets)

| bench | file | compiles | consts | divan fns | verdict |
|---|---|---|---|---|---|
| v26_10_31 | yes | yes | 3 | 4 | REAL |
| kernels | yes | yes | 0 | 0 | SHELL — declared F2-10 stub, `main` only |
| ingress | yes | yes | 6 | 2 | REAL |
| parquet | yes | yes | 2 | 3 | REAL |
| bn3 | yes | yes | 11 | 3 | REAL |
| seal | yes | yes | 9 | 3 | REAL |
| challenger_parquet | yes | yes | 2 | 4 | REAL |
| challenger_arrow | yes | yes | 1 | 2 | REAL |
| challenger_git | yes | yes | 1 | 2 | REAL |
| challenger_crypto | yes | yes | 0 | 2 | REAL (divan fns carry the corpus consts inline) |
| chains-ingress | yes | yes | 10 | 5 | REAL |
| chains-parquet | yes | yes | 9 | 6 | REAL |
| chains-bn3 | yes | yes | 18 | 7 | REAL |
| chains-seal | yes | yes | 9 | 4 | REAL (1 unused-mut warning, non-fatal) |
| chains-sovereign | yes | yes | 18 | 7 | REAL |
| mat_closure | yes | NO | 1 | 1 | BROKEN — divan 0.1.21 API misuse |
| dred_churn | yes | yes | 5 | 0 | REAL — plain `main` harness, not divan; carries final-state oracle asserts (`assert_eq!(set, oset)`, ratio fence at `dred_churn.rs:102,122,137`) |

## Registry totals

- Declared targets: 17. Files present: 17/17.
- Compile: 16/17 (exit 101 cargo, sole failure `mat_closure`).
- REAL: 15 (incl. dred_churn non-divan harness). SHELL: 1 (kernels, declared stub).
- BROKEN: 1 (mat_closure).
- Total `#[divan::bench]` fns across the registry: 55.
  (bn3 3, chains-bn3 7, chains-ingress 5, chains-parquet 6, chains-seal 4,
  chains-sovereign 7, challenger_arrow 2, challenger_crypto 2, challenger_git 2,
  challenger_parquet 4, ingress 2, kernels 0, mat_closure 1, parquet 3,
  seal 3, v26_10_31 4; dred_churn 0.)

## mat_closure exact failure (sole compile break)

`crates/bench/benches/mat_closure.rs:34`:

```rust
b.counter("facts", N as u64).bench_local(|| { ... });
```

rustc (divan 0.1.21):

- `error[E0061]`: `counter` takes 1 argument but 2 were supplied.
- `error[E0277]`: `&str: IntoCounter` not satisfied.

Fix shape (divan 0.1.21 `Bencher::counter<C: IntoCounter>(self, counter: C)`,
seen in compiler help output this session): drop the string name —
`b.counter(N as u64).bench_local(...)`. Not applied by this lane (census
lane; file owned elsewhere — no owner named in the manifest comments).

## Divergences / notes

- None of the killed-lane half-written files the census was hunting were
  found: every declared target parses and all but one compile.
- `kernels.rs` shell status is DECLARED in the manifest comment ("F2-10
  scaffold stub"), not a discovered defect.
- Bench binaries land under `debug/build/bench/<hash>/out/` (this crate
  uses a build-script output layout), not `debug/deps/` — first few
  probes looked in the wrong place; JSON artifacts settled it.
