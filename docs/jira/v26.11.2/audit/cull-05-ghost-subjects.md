# cull-05-ghost-subjects

CULL-05 ghost-subject gate inventory (2026-11-02, read-only).
Every verdict evidence-backed from the current tree.

## 1. zero_datafusion_gate.py vs the removed family

Gate enumerates NOTHING crate-specific: NEEDLE = "datafusion"
(zero_datafusion_gate.py:40); probes P1 cargo metadata, P2 cargo
tree, P3 literal scan of root/member Cargo.toml + Cargo.lock,
folding root [workspace.dependencies] as latent opt-in.
Subject family gone (0 hits in Cargo.lock / any Cargo.toml; no
crates/cgap-datafusion* dir; removal receipt
v26.11.1/waves/cgap-datafusion-removal.md on disk).
Verdict: KEEP-FROZEN — the subject is the ABSENCE of datafusion;
the gate is a reintroduction vaccine, not a ghost. Its --self-check
exit 1 is the vendor symlink loop (lane-c06 ERRATA) — laned fix.

## 2. Gates vs registry-absent / publish=false crates

publish=false: cgap-beam:8, cgap-court:7, cgap-verification:8,
cgap-branchless:7, cli:9, csct:5. cgap-verification + csct
standalone non-members; cli standalone by law
(release_check.py:137-142 handles all three).
- boundary_gate.py:863-881 COVERED_CRATES (15): every entry exists.
  No ghost. KEEP. (cgap-court IS a member despite publish=false,
  deliberately delisted in registry_sweep.py:48.)
- registry_sweep.py LEDGER :34-65: deliberately encodes
  publish=false as DELISTED/UNPUBLISHED with court pointers —
  subjects are the registry STATE, not the crates. KEEP.
- zero_string_gate.py DOC_EXECUTION_FAMILY (:447-456): two genuine
  ghosts — "cli" and "integration" are DIRECTORY names not package
  names (real: cgap-cli, cgap-integration); "cli" also resolves to
  a standalone non-member. Verdict: RE-TARGET (drop the two stale
  keys; the other 9 exist as packages). KEEP the gate.
- cgap-verification: referenced by no gate subject list. Nothing
  to cull.
- csct: subject of verify_cscb_sram.py (:4,:344) — crates/csct
  alive with real bins consuming branchless scan (:82). KEEP.
- lease_janitor/doctor "csct-*" hits = lease debris patterns,
  not subject gates. KEEP.

## 3. receipt_lint.py scope vs the docs tree

DEFAULT_DIRS waves/ (166 files); landing traversal (:135-141);
DOC-RECEIPT scope audit/+waves (:220-222) — both populated
(audit/ 118, waves/ 166); lanes/ 118 files for RECEIPT-STRUCT.
Verdict: KEEP. No scanned doc class has vanished.

## 4. dead_library_exceptions.json (26 entries) vs consumption

Exactly one REAL consumption found outside branchless:
prefix_max_u8x8 — consumed by cscb_c1_c2_ingress.rs:82. Its entry
says "consume or delete; then REMOVE this entry" — STALE, masking
a must-consume symbol. Verdict: REMOVE-ENTRY (re-target the gate
to enforce it consumed).
swar_lt/splat/range_mask: "consumers" in cgap-git ingress_ring.rs
(:41,:51,:58) are RE-DECLARATIONS, not imports — entries stay.
penstock_prefilter8_mix: only outside ref is a COMMENT
(penstock.rs:676) — matches its own justification. KEEP-FROZEN.
Remaining 22 entries: ZERO production consumption; symbols still
defined (alive-but-unconsumed, exactly as the entries justify).
KEEP-FROZEN (known-red x9 disclosed per _c05_note).
FIX-07 cross-check: the restored bench bindings were bench locals,
NOT exception-JSON symbols — no JSON entry affected.

## 5. workstream-receipt-map.md UNVERIFIED rows

- F10 (:24): putative subject never landed. RE-TARGET (documented
  absence -> point at open R7; or remove if R7 retired).
- F5 (:23): PARTIAL, not UNVERIFIED — receipt alive in lsp-max
  (docs/lane-f5-workspace-integration.md:11-17, 21 tests green)
  but predates the 90+ test expansion. RE-TARGET (refresh).
- WG2 (:25) / WG3 (:26): ALIVE — triple receipts incl. live
  re-verify 2026-11-02 (dod-lsp-max-wire.md:44,124; lane-g05;
  lane-g04). KEEP. The prompt's premise (UNVERIFIED) is wrong
  for WG2/WG3 — only F10 is UNVERIFIED.

## Summary

| item | verdict |
|---|---|
| zero_datafusion_gate | KEEP-FROZEN |
| boundary_gate COVERED_CRATES | KEEP |
| registry_sweep LEDGER | KEEP |
| zero_string "cli"/"integration" keys | RE-TARGET |
| receipt_lint scope | KEEP |
| dead_library prefix_max_u8x8 | REMOVE-ENTRY |
| dead_library other 25 | KEEP-FROZEN |
| workstream F10 | RE-TARGET |
| workstream F5 | RE-TARGET |
| workstream WG2/WG3 | KEEP |

## See Also

audit/cull-01-gate-overlap-matrix.md ·
lanes/lane-c06-scripts-gate-coverage.md ·
audit/workstream-receipt-map.md
