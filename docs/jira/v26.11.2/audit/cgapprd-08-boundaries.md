# cgapprd-08-boundaries

CGAP-PRD-8 boundary audit: §4 integration claims vs real referents
(2026-11-02, read-only lane).

## W0 Key finding up front

The PRD clause names "AshA2A". Whole-tree grep (rs/toml/md/go/
Cargo.lock/vendor/): the ONLY hit is a stale wave note
(v26.11.1 waves/errc-eval-ad.md:20, "ash_a2a-pinned" packs v0.1.0).
NO ash_a2a crate, NO Cargo dep, NO Go import, NO FFI symbol, NO code
path connecting ash_a2a to cgap anywhere. The sentence is vendor-claim
language with zero in-tree wiring.

## W1 Clause-by-clause

| # | PRD §4 clause | real referent | verdict |
|---|---|---|---|
| 1 | "projects work orders into content-addressed coordinate identities" | content addressing REAL but in cgap-git (dag.rs:1 Merkle DAG, object_store.rs:1 32B BLAKE3); work-order projection: NO referent | ASPIRATIONAL-UNVERIFIED (substrate real, projection absent) |
| 2 | "authority verified at the ingress ring" | two real ingress paths, neither a2a: (a) sovereign.rs:413 cgap_admit_frame two-stage admission (penstock limiter :434, header decode + BLAKE3 verify :455-463, zero-copy out_view, typed refusals); (b) cgap-git ingress_ring (GIT-03) + admission_gate (GIT-09, typed AndonRefusal). "Authority" at either ring = format/magic/digest/ontology checks; no a2a authority principal | VERIFIED as mechanism; ASPIRATIONAL as stated |
| 3 | "AshA2A pack consumption" | ZERO in-tree wiring | VENDOR-CLAIM-UNVERIFIED |
| 4 | "zero-copy vector I/O" | REAL, measured, different names: 11-symbol C-ABI + cgap_admit_frame zero-copy view; ingress_ring; IPC-4/AX-6 mmap transport 4.52-4.54 darwin / 4.574-4.656 linux ns/op, 0 allocs, mmap:pread ~1:260 | VERIFIED (renamed, measured, landed) |
| 5 | "socket trapped before connection init" | DESIGN-ONLY, ABSENT: no net.Dialer/.Control in the Go module | ASPIRATIONAL-UNVERIFIED |

## W2 POSH reconciliation (11-symbol table)

All landed (lane-ipc-00 W3, nm -gU on the 615,552 B cdylib).
PRD zero-copy I/O maps onto cgap_admit_frame's projection (aligned
callers get true zero-copy; unaligned get decode — same gates,
sovereign.rs ~452-466) + IPC-4's mmap court holding on BOTH OSes
(lane-ax-06 W4). The transport court "awaits the cgap_shm_* membrane"
(lane-ax-06 W7) — the receiving symbol does not exist yet.

## W3 FFI roadmap entries needed

| missing symbol | why the PRD clause needs it | status |
|---|---|---|
| cgap_ingress_bytes | batch-ingress over the A14 penstock (penstock.rs:513/:674 real Rust paths) — clause 2's ring admission has NO C-ABI membrane | MISSING at C-ABI, NO-GO CTQ-02 |
| cgap_radix_join | radix join exists at NO layer — clause 1's projection has no join primitive | MISSING everywhere, NO-GO CTQ-04 |
| cgap_shm_* set | shm court built + measured on both OSes with nothing to bind | NO-GO shm suite |
| penstock filter C-ABI | prefilter8 Rust-only, no #[no_mangle] wrapper | MISSING at C-ABI |

Verdict: until the symbol batch lands, every cross-boundary Go/POSH
PRD claim is bounded by the frozen 11-symbol admit/eval/commit/verify
surface.

## See Also

lanes/lane-ipc-00-ground-truth.md · audit/cgapprd-02-equation-grounding.md
