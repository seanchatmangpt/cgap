# f3-06-pins

Status: ALIVE — zero undeclared duplicate authority pins in the five-crate
publish set (coordinator-written from court-auditor F2-lane F3-6's verdict;
that lane is read-only by contract and did not write this file).

- Positive controls fired on all five canonicals (CgapFrameHeader
  cgap-core/frame.rs:179/:383/:384; ATTESTATION_LEN cgap-p28/attestation.rs:60;
  CgapStatus cgap-ffi/sovereign.rs:68; ceiling frame.rs:68 + penstock.rs:268 +
  asc450.rs:74; SlotId alive repo-wide, absent from the five).
- Sole by-value pin in the publish set:
  cgap-parquet/src/footer.rs:84 ATTESTATION_LEN=80 — self-declared
  framing-only, cites A17/cgap-p28 (footer.rs:621, mmap_writer.rs:727-729).
- CgapStatus/FRAME_* occurrences in cgap-arrow are test-only dev-dep imports
  (slab.rs:1270+ cfg(test)); cgap-git/dag.rs:92 cites the canonical by path in
  a doc comment. No headers ship in the five.
- Corruption sweep over the five: zero tokens; sweep fire-test 3/3 on
  synthetic stdin (non-vacuous).
- Registered divergence (non-published crates): cgap-core/frame.rs:44
  ATTESTATION_LEN twin — outside the publish set.
- Audit-time caveat: dag.rs/pack.rs in-flight at audit time; re-run the
  cgap-git sweep if they change again before publish.
