# NOTICE — cgap-blake3 upstream attribution

This file closes red-07 provenance gap P3
(`docs/jira/v26.11.2/redteam/red-07-retaliatory-discovery-plan.md:21-23`)
and remediation item 1 (same file, §5 item 1-2). It applies to the single
crate `crates/cgap-blake3`; the repo-wide consolidated vendor register is
FIX-10 territory (`docs/jira/v26.11.2/redteam/fix-10-license-aggregation.md`)
plus the vendored nanoarrow tree (`vendor/nanoarrow/LICENSE.txt`), both
out of scope here.

## 1. What this crate actually is (on-disk facts)

- `crates/cgap-blake3` is an INDEPENDENT REIMPLEMENTATION of the BLAKE3
  cryptographic hash function from the published specification. It is NOT
  a vendored, copied, or translated tree of the BLAKE3 reference
  implementation.
- Evidence: the crate carries zero external dependencies
  (`crates/cgap-blake3/Cargo.toml` — empty `[dependencies]`); no
  `vendor/` BLAKE3 subtree exists (`vendor/` contains only `nanoarrow/`);
  `Cargo.lock` contains no upstream `blake3` package entry (only
  `cgap-blake3` workspace members, `Cargo.lock:102-109`).
- `crates/cgap-blake3/src/lib.rs:1-8` names the specification as the sole
  derivation source: "Implements the complete BLAKE3 cryptographic hash
  specification", covering the chunk/parent compression, Merkle reduction,
  and SIMD dispatch surfaces.
- Conformance is proven against the official known-answer corpus, not
  against reference-implementation source:
  `crates/cgap-blake3/tests/official_vectors.rs:3-14` states the corpus
  follows the canonical BLAKE3-team `test_vectors.json` recipe, with
  expected digests derived via `b3sum` and cross-checked byte-identical to
  published values for 14 lengths (0..8192).

## 2. Upstream attribution

The BLAKE3 specification and reference implementations originate from the
BLAKE3 project (the BLAKE3 team; the specification paper is
"BLAKE3: one function for all hashers on multiple platforms",
J.-P. Aumasson, S. Neves, Z. Wilcox-O'Hearn, C. Winstanley).

The upstream BLAKE3 repository dedicates its reference code to the public
domain (CC0-equivalent dedication) and additionally offers an Apache-2.0
license option. As of this writing no upstream tree is vendored in this
repository, so the exact upstream license file bytes are UNVERIFIED from
disk; this NOTICE states the upstream license posture as documented by the
upstream project. Should BLAKE3 upstream code ever be vendored here, the
verbatim upstream license text and NOTICE must be added to this file and
the pin recorded (sha256) before merge. This attribution is made
regardless: the specification itself is the derivation source, and the
project gratefully acknowledges the BLAKE3 authors.


## 3. Clean-room protocol record

Protocol class: SPEC-ONLY DERIVATION. The implementers of
`crates/cgap-blake3` derived the implementation from the published
BLAKE3 specification (the paper plus the parameter tables), not from the
reference C/Rust/Go sources. On-disk corroboration:

- Spec constants only, in the crate's own form:
  `crates/cgap-blake3/src/lib.rs:15-31` (IV, MSG_PERMUTATION, domain
  separation flags) — constants that are identical for every independent
  implementation by definition of the function.
- Architecture is deliberately non-reference: a zero-allocation stack
  Merkle reduction and horizontal SIMD multi-parent kernels
  (`lib.rs:4-7`) that do not exist in the reference C implementation.
- KAT conformance via the published test-vector recipe, not via
  reference-source diffing (`tests/official_vectors.rs:3-14`).
- No BLAKE3 reference source exists anywhere in this repository to have
  been transcribed (checked: `vendor/`, `Cargo.lock`, crate tree).

TO BE COMPLETED BY THE ORGANIZATION (placeholders, never invented by
this document):

- Designated implementer(s): [ORG RECORD REQUIRED]
- Spec-version / paper citation pinned: [ORG RECORD REQUIRED]
- Spec-access boundary and access log locus: [ORG RECORD REQUIRED]
- Isolation agreement between spec readers and implementers: [ORG RECORD
  REQUIRED]
- Date range of derivation work: [ORG RECORD REQUIRED]

Until those five fields are filled from real organizational records, the
clean-room posture is PARTIAL, not DEFEATED, against red-07 subpoena
items 16-21. This file does not fabricate them.

## 4. License of this crate

`crates/cgap-blake3` is distributed under the repository instrument
P28-SIL v2.1.0 (`LICENSE.md:3-9`,
`urn:avec:license:p28-sil:v2.1.0-sovereign`), declared via
`license-file = "../../LICENSE.md"` in `crates/cgap-blake3/Cargo.toml:11`.
The BLAKE3 specification's authors carry no claim over independent
implementations of the published function; this NOTICE attributes
lineage, not permission scope.
