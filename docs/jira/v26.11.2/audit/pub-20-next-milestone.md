# PUB-20 — Next-Milestone Inventory (out-of-26.11.2-publish-scope artifacts)

Date: 2026-11-02. Method: fresh on-disk reads of crates/, courts/,
scripts/, docs/jira/v26.11.2 (commits c0ebd750, 194010b6, 86ff5f93,
e5f3de88, b3386826). Scope: artifacts this session produced that are
NOT in the 26.11.2 publish scope but feed 26.11.3 / 26.12.

Publish law baseline (audit/pub-08-version-unification.md): the 26.11.2
bump moves workspace version + every intra-repo hard pin + Cargo.lock
atomically. csct carries 0.1.0 and is explicitly OUT of the calendar
bump. cgap-git is publish=true at 26.11.2 (registry max 26.11.1,
REGISTERED-DIVERGENCE).

## 1. Per-artifact inventory

### csct crate (crates/csct)
- registry target: crates.io FIRST-PUBLISH CANDIDATE; today
  publish=false, standalone [workspace] (like crates/cli).
- version: 0.1.0 local; carries no 26.11.x identity.
- prerequisite: decide version scheme (0.1.0 vs calendar); flip
  publish=true; path deps cgap-blake3/core/bn3/storage must resolve
  on the registry (currently disclosed normal dep, not dev-dep).
- court status: ALIVE. cargo test 65/0 (audit/dod-csct-cscb.md);
  G11-G15 green. DEFECT-OPEN: C3 closure 796 ms vs 25 ms SLA;
  C6 242M facts/s vs 800M fps gate.

### cgap-git 15-module no_std substrate (crates/cgap-git/src)
- registry target: crates.io, publish=true already set; inherits
  workspace 26.11.2.
- version: 26.11.2 local / registry max 26.11.1 (divergence).
- modules (commit c0ebd750): pack_demux, pkt_line, ingress_ring,
  delta_resolver, object_store, tree_synth, egress_ring, commit_synth,
  admission_gate, patent_court, retract, octopus_merge, ref_store,
  mcp_driver, a2a_swarm. All mounted in lib.rs.
- disposition: ADDITIVE in 26.11.2 — ships with the calendar bump;
  no separate deferral.
- court status: ALIVE. soc_full_chip_court (1M-fact in-image,
  0-alloc, 3 mutation courts); e2e_wire_substrate_court S1-S5 with
  F1-F4 seam flips.

### checkpoint prototype (crates/cgap-core/src/checkpoint.rs)
- registry target: internal only. UNMOUNTED from lib.rs; reachable
  solely via #[path] test mount (tests/checkpoint_court.rs,
  E0152 divergence note; tracking_alloc.rs same convention).
- version: workspace 26.11.2; no registry presence.
- prerequisite for 26.11.3 core: PRD-4 reconciliation (audit/
  cgapprd-04-checkpoint-design.md was ABSENT at write time —
  disclosed PENDING-design note in the file header); bn3 fixpoint
  derived-set capture is OUT of v1 scope (allocates by construction).
- court status: ALIVE via path-mount court only. 88-B CheckpointBuf,
  corruption -> CheckpointMismatch flip witnessed.

### AX testbed (courts/ax-testbed)
- registry target: none. Internal container harness (Dockerfile,
  build_image.sh, runners, manifests).
- version: n/a.
- prerequisite: docker LOCAL GO (AX-0); spliced AX-3 stage fragment;
  arm64 go substitution disclosed in lane-tc-14 (AX-1 file never
  edited).
- court status: ALIVE. In-image alloc arm 6/6 PASS (steady-state
  0.0 allocs/op, anti-vacuity flip); mutation arms 1/2/4 PASS
  in-image; arm 3 SKIP-MISSING-EXPORT cgap_ingress_bytes (loud).

### interop-chicago (courts/interop-chicago)
- registry target: NOT a crates.io artifact. Standalone Go module
  (own go.mod, stdlib-only). Module path
  avec.chatman.dev/cgap/courts/interop-chicago. REGISTRY QUESTION
  UNDECIDED: Go module proxy push vs vendored-in-repo only.
- version: go 1.23; no semver tag.
- prerequisite: decide distribution law; IPC-6 owns the real
  seal_receipt emitter (current cmd is LOUD-STUB).
- court status: ALIVE. go build ./... + go test ./ffi/ exit 0
  (ok, 2.490s); darwin fence, no-regression law holds.

### layer-qualifier lint (scripts/receipt_lint.py)
- registry target: none. Internal DX tooling, repo-only.
- version: n/a.
- prerequisite: none (extends TC-09 DOC-RECEIPT + DX-4
  RECEIPT-STRUCT classes; LAYER-QUALIFIER-001, claim rows only,
  audit/ + waves/ scope, +/-3-line proximity).
- court status: ALIVE. FLIP A fires / FLIP B silent / FLIP C
  self-compliance zero findings; --self-check rc 0; --strict rc 1
  typed exit citing 5 finding classes.

### frame64 path (PRD-3 path b)
- registry target: none. REJECTED artifact; no code moved.
- version: n/a.
- prerequisite: none — terminal state.
- court status: REJECTED by PRD-3 §5-§6. Docs errata landed
  (lane-cgap-prd-05-frame-errata): 64 B = cgap-arrow Quad64 only;
  CSCB fact frame is 24 B (3x u64 LE). No next-milestone work.

## 2. 26.11.3 / 26.12 disposition summary

- csct: DECISION REQUIRED (first publish; version scheme; C3/C6
  defects close or re-fence before public publish).
- cgap-git 15 modules: additive, ships in 26.11.2 publish.
- checkpoint: 26.11.3 core work (lib.rs mount + PRD-4 errata).
- AX testbed / interop-chicago: internal court infrastructure;
  Go-module registry law UNDECIDED.
- layer-qualifier lint: internal DX, ships as repo tooling only.
- frame64: closed, rejected, no carry-over.
