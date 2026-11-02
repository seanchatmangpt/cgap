# Lane IPC-8 — interop charter + stitch index

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. No git commands run.

## Files written (owned)

- `docs/jira/v26.11.2/dflss/01-project-charter-interop.md`
- `docs/jira/v26.11.2/lanes/lane-ipc-08-index.md` (this receipt/index)

## Peer census (this session)

`ls docs/jira/v26.11.2/lanes/lane-ipc-0*.md` → zero matches (exit 2
from ls as no match). IPC-0..IPC-7 all PENDING at receipt time; no
waiting, no creation on their behalf.

## Stitch index

| Lane | Owned surface | Standing as receipted | Load-bearing fact |
|------|---------------|----------------------|-------------------|
| IPC-0 | ground-truth export table | PENDING (no receipt on disk) | the authoritative C-ABI export table — absent, so all table claims withheld |
| IPC-1 | PENDING (no receipt on disk) | PENDING | PENDING |
| IPC-2 | PENDING (no receipt on disk) | PENDING | PENDING |
| IPC-3 | latency measurement | PENDING (no receipt on disk) | the measured ns/frame that supersedes the 6.04 ns provisional arithmetic |
| IPC-4 | shared-memory mechanism | PENDING (no receipt on disk) | the actual macOS mechanism replacing the `/dev/shm` dispatch claim |
| IPC-5 | PENDING (no receipt on disk) | PENDING | PENDING |
| IPC-6 | PENDING (no receipt on disk) | PENDING | PENDING |
| IPC-7 | PENDING (no receipt on disk) | PENDING | PENDING |
| IPC-8 | this index + interop charter | ALIVE (this receipt) | corrections grounded: B06 median replaces the 4.0M seals/s bullet; claim ceiling = kernel-call path owed in cgap-ffi |

Peers are lanes named by dispatch; owned surfaces for IPC-1/2/5/6/7 were
not stated in this lane's dispatch, so those cells are PENDING by law —
never inferred.

## Honest post-drain claim ceiling (coordinator note)

Corroborated this session by read-only grep over `crates/cgap-ffi/src/`:
`radix` returns ZERO matches; batch surface is `cgap_i32_batch` only
(`crates/cgap-ffi/src/lib.rs:43-45`). Ceiling:
**"membrane harness landed; kernel-call path owed in cgap-ffi."**
End-to-end interop throughput is UNVERIFIED at this ceiling until
kernel-call entry points land.
## Commands + exits (this session)

- `ls /Users/sac/cgap/docs/jira/v26.11.2/lanes/ | grep -i ipc` → exit 1,
  zero IPC receipts (census, PENDING basis) — witness above.
- `grep -rn "radix" /Users/sac/cgap/crates/cgap-ffi/src/` → zero matches,
  exit 1 (ceiling corroboration).
- `grep -c "no_mangle|export_name" ...` per file → lib.rs 6,
  xip_tables.rs 3, sovereign.rs 10 (charter export-surface census).
- `grep -n ... /Users/sac/cgap/docs/jira/v26.11.2/lanes/lane-b06-cscb-c7.md`
  → exit 0, lines 23-32, 51-54 read from disk this session.

## Standing

ALIVE for the two landed files and the grounded corrections (B06 median,
FFI grep census, ceiling claim). PENDING for all IPC-0..7 dependent
bullets. No court flips in this lane: it is documentation-only; the
charter's falsifier (verbatim in the charter) is discharged by the
IPC-wave Verify tollgate, not by this lane — a [SKIP] would be loud and
is named here: mutation matrix 4/4 requires IPC-0..7 receipts, all
absent at receipt time.

## Divergences

- Dispatch assumed peers might have landed; none had. All PENDING rows
  are honest absences, never fabricated standing.
- The 6.04 ns figure and `/dev/shm` mechanism were not found in any
  on-disk dflss doc; they exist only in dispatch prose — corrected
  in the charter as provisional/PENDING respectively.
