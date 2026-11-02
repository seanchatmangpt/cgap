# Error codes

CgapStatus register. Canonical ledger:
`crates/cgap-ffi/src/sovereign.rs` (constants at lines 71-84),
mirrored name-for-name and value-for-value in `include/cgap.h:58-66`.
Type: `pub type CgapStatus = u32` (sovereign.rs:68), width pinned to
4 bytes (sovereign.rs:85; cgap.h `_Static_assert`).

| Constant | Value | Meaning | Example producer path |
|---|---|---|---|
| `CGAP_OK` | `0x0000_0000` | success | every completed entrypoint |
| `CGAP_ERR_INVALID_MAGIC` | `0x4347_0001` | `'CG' + 01` | bad frame magic, admit path (`map_frame_error`, sovereign.rs:376) |
| `CGAP_ERR_OABI_MISMATCH` | `0x4347_0002` | `'CG' + 02` | payload-length/digest mismatch (sovereign.rs:379); BN3 shape/rodata faults (sovereign.rs:555, 578, 617-634) |
| `CGAP_ERR_SHACL_VIOLATION` | `0x4347_0003` | `'CG' + 03` | reserved-bits set (sovereign.rs:382); BN3 domain/shape faults (:532, 539, 600, 644); commit input domain (:681-701) |
| `CGAP_ERR_LEASE_EXHAUSTED` | `0x4347_0004` | `'CG' + 04` | exhausted `out_lanes` lease (:641); commit sink IO (:715, 719, 733); witnessed boundary: A02 commit lease accepts 65376 lanes, refuses 65377 (bench-chains-sovereign.md) |
| `CGAP_ERR_P28_SPOLIATION` | `0x5032_38FF` | `'P28' + FF` | null or non-80-byte evidence (:299, :307, :315); any A17 §5.2-§5.8 predicate failure including unbound digest; attestation mismatch (sovereign.rs:384) |

## Mapping law

- Frame errors map via `map_frame_error` (sovereign.rs:370-384):
  bad magic -> `INVALID_MAGIC`; payload/digest -> `OABI_MISMATCH`;
  reserved bits -> `SHACL_VIOLATION`; attestation -> `P28_SPOLIATION`.
- Commit carries no generic IO status: sink IO failure maps to
  `LEASE_EXHAUSTED` (sovereign.rs:669 doc block, :719).
- Reachability law: every value must stay reachable through a real code
  path on some supported target; an unreachable enum value is untested
  decoration (cgap.h:58-60 comment; status court §6.2).
