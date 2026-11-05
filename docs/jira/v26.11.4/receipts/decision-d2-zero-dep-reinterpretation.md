# Decision D2: zero-dep reinterpretation (ADOPTED)
Standing: ALIVE

## Decision

D2 ADOPTED: "zero-dependency" in the wire-shm law means **zero crates, not
zero syscalls**. Direct libc calls via bare `extern "C"` blocks satisfy the
zero-dep law; the previous reading ("mmap requires libc and the zero-dep law
forbids it") is reinterpreted, not the law repealed.

## Evidence

- `/Users/sac/cgap/crates/cgap-beam/src/shm.rs:5` — declares the discipline
  in-source: "zero crates, matching the crate's hand-rolled-ABI discipline";
  bare `extern "C"` `shm_open` + `mmap` at `shm.rs:127` (and `shm.rs:822`).
- `/Users/sac/cgap/crates/cgap-parquet/src/scan.rs:65` — same pattern in the
  published kernel (`unsafe extern "C"` mmap block).
- Coordinator court receipt: cgap-beam shm.rs courted 747-720.

## Law amendment (wire-shm lib.rs)

`crates/wire-shm/src/lib.rs:7` law sentence is amended in the same commit as
this receipt to cite the reinterpretation: the zero-dep law constrains the
dependency graph (zero crates), not the syscall surface; a bare `extern "C"`
libc call is zero-dep lawful.

## Falsifier

Any `Cargo.toml` diff that adds a dependency crate while claiming the zero-dep
law under this ruling refutes the reinterpretation. Conversely, a bare
`extern "C"` mmap landing in any wire-shm-family crate without a new crate
dependency is confirmatory, not a violation.

## Standing

ALIVE (decision adopted; two independent in-tree precedents cited; lib.rs
amendment lands with this receipt).

Date: 2026-11-04
