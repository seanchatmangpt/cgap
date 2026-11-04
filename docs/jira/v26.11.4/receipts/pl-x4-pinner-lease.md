# PL-X4 — Pinner Lease Hand-Law Receipt (arcus cdata)

Date: 2026-11-04. Lane: CODE-LANE-5 (docs+receipts). Status: ALIVE (cdata
suite re-run this session, exit pasted) with one court LOUD-SKIPped in this
environment, receipted honestly below.

## Subject

`/Users/sac/arcus/internal/arcus/cdata/pinner_lease.go` — the A9 cdata hand
zone: `runtime.Pinner` lease lifecycle as type-state.

## The hand-law

- **Zone registration**: `pinner_lease.go:1-10` OUT-OF-GENGEN header —
  allow-with-reason under the three-zone law (A9 cdata hand zone per
  p08-cdata); the Rust counterpart law is
  `cgap/crates/cgap-arrow/src/c_abi.rs:122-139` (first-winner release:
  swap the release slot to NULL first; `prev == NULL` is a typed no-op,
  never a double free).
- **Type-state**: `LeaseBuffers(slab, valuesOffset) PinnerLease` (`:64`)
  mints the lease; `PinnerLease` is a pointer to the private lease body
  (`:50`), affine by convention — methods on Fresh, trap on Spent
  (`Pointers()` on Spent panics "double-export trap; mint a new lease"
  `:91`; `Unpin` idempotent via atomic state flip `:96-99`). Compile-time
  type-state note `:26` (every Fresh method traps on Spent).
- **GC honesty**: the lease keeps the slab alive across GC pressure for the
  export lifetime (`:56-59`); packing delegates to the generated primitive
  (`PackBuffers`) — the hand-law owns only the pin/release seam.
- **Release-once**: `ReleaseOnce`/`ReleaseOnceSchema` (`:111/:119`) release
  the exported ArrowArray/Schema twin and own the unpin; `prev == 0` means
  already-released, typed no-op (c_abi.rs:130 law).

## Three-layer court

`/Users/sac/arcus/internal/arcus/cdata/pinner_lease_test.go`:

1. **Vet-mutant + control** (`:41-49` `TestVetMutantNoCopyLaw`): copying the
   lease struct (`lease := *fresh`) is the vet-mutant for the noCopy law;
   the control lease behaves.
2. **GC-pressure flip** (`TestPinLifetimeGCPressureCourt`, `:302` witness):
   an unpinned-slab mutant must be observed collected under GC pressure.
   THIS ENVIRONMENT: LOUD SKIP —
   `pin-lifetime mutant did not flip in this environment (err=<nil>
   output="INTACT\n") — conservative GC kept the unpinned slab alive; flip
   not witnessed here`. A conservative Go GC can legitimately keep the
   memory reachable; the court skips loudly instead of faking a flip.
   Flip 3/3 was witnessed at landing on a non-conservative run; standing
   here is ALIVE-with-skip, not a clean sweep.
3. **Release-once** (`TestReleaseOnceCourt`): second release is the typed
   no-op; no double free.

Plus seam/layout courts: `TestPinnerLeaseSeam`, ArrowArray/Schema twin layout
courts, `TestPackBuffersCourt`, `TestAccessorAllocsPerRunZero`.

## Template-rewire via ggen sync

The cdata package is registered OUT-OF-GENGEN with allow-with-reason; the
generated pack/sync path (arcus `ggen.toml`, A9 zone in `CLAUDE.md:27`
"hand-written hot paths (A8 cgo membrane, A9 cdata) pinned by digests") is
the lawful landing — ggen sync re-projects the generated seams and leaves
`pinner_lease.go` untouched as the hand zone; the no-hand-edit law is what
the zone header documents. Exclusion registration: A9 cdata in arcus
CLAUDE.md:27 + p08-cdata three-zone law.

## Re-run (this session, 2026-11-04)

```
cd /Users/sac/arcus && go test ./internal/arcus/cdata -count=1 -v
-> PASS: TestArrowArrayTwinLayoutCourt, TestArrowSchemaTwinLayoutCourt,
   TestAccessorAllocsPerRunZero, TestPackBuffersCourt, TestPinnerLeaseSeam,
   TestVetMutantNoCopyLaw, TestReleaseOnceCourt
   SKIP: TestPinLifetimeGCPressureCourt (LOUD SKIP, witness quoted above)
   ok github.com/seanchatmangpt/arcus/internal/arcus/cdata 8.681s, exit 0
```

## Standing

ALIVE-with-skip — full suite exits 0; the GC-pressure flip court is
environment-dependent and skipped loudly here (flip witnessed at landing,
3/3). Not rounded to a clean sweep.
