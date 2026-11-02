# Lane TPS-4 — ffi go-vet completion (verify-and-receipt, zero diff)

## W7 Receipt

### 1. Identity

Lane TPS-4, repo /Users/sac/cgap (main). Subject:
`courts/interop-chicago/ffi/ffi.go` (read-only this session — zero bytes
changed; sha256 below).

### 2. Authority

Dispatch prompt (owned files: ffi/ffi.go one-edit + this receipt).
DXD-03 finding cited: "7 undefined C.* constants leave go vet ./... RED".

### 3. Commands + exits

Shared tree, BEFORE (as dispatched):

    cd /Users/sac/cgap/courts/interop-chicago && go vet ./...
      → exit 0 (expected exit 1 per DXD-03 — divergence, see §7)
    zero "could not determine kind" lines in vet output (grep -c = 0)

Shared tree, full AFTER battery (same state, since no edit):

    go build ./...            → exit 0 (BUILD_OK)
    go test ./ffi/ -count=1   → exit 0 ("ok ... 2.358s")
    go test ./shm/ -count=1   → exit 0 ("ok ... 0.275s")  [darwin fence]
    go vet ./...              → exit 0

### 4. Consequence

Zero diff. No fence can regress because no byte moved.

### 5. Courts

| Court | Mutation (anti-vacuity) | Flip witness | Exit |
|---|---|---|---|
| vet-undefined-C court | Add `#include "cgap.h"` to ffi.go preamble in /tmp sandbox | typedef redefinition: `cgap_frame_view` (`ffi.go:31` vs `include/cgap.h:82`), go vet exits 1 | 1 |
| mutation-court flip (existing, ffi pkg) | XGAP corrupt magic → want 0x43470001 | go test ./ffi/ green incl. mutation_court_test.go flip witnessed | 0 |

The dispatch's prescribed one-line include is itself the anti-vacuity
mutation for the "vet is green without it" claim: in a /tmp sandbox copy
it flips vet from exit 0 → exit 1 (typedef redefinition). The include is
therefore NOT the fix — ffi.go deliberately uses a layout twin
("cited, not redefined", ffi.go:23-30) and Go-side status constants
(Status* at ffi.go:68-73) instead of C.* references.

### 6. Standing

PARTIAL (verify-and-receipt path per dispatch: "If IPC-2 or another lane
already fixed it when you read the file, VERIFY-AND-RECEIPT instead of
editing"). The dispatched BEFORE state (vet exit 1, 7 undefined C.*
errors) was not observed — the on-disk surface was already green. The
status constants resolve via Go literals pinned to include/cgap.h:58-69
values (0x43470001..0x43470006, 0x503238FF), grep-witnessed, not via a
cgap.h include.

### 7. Divergences/disclosures

- **Doc yields to reality**: DXD-03's premise (7 undefined C.* names,
  vet RED) is inverted against the current on-disk ffi.go. Either it
  audited a pre-IPC-2 state or the surface was repaired after the audit.
  No edit applied; errata against audit/dxd-03-go-ergonomics.md is the
  coordinator's call, not this lane's (read-only outside owned files).
- **Prescribed fix would break**: adding `#include "cgap.h"` produces a
  C typedef redefinition (sandbox-witnessed, exit 1). ffi.go's layout
  twin is the mechanism, not a defect.
- **Disclosure to lane-ipc-02** (per dispatch): no SLA unblock applied
  to lane-ipc-02's files this session; its surface was found complete
  and green. This receipt supersedes DXD-03's stale vet-RED claim for
  courts/interop-chicago.
