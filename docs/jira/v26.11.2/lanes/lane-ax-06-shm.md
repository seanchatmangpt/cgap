# Lane AX-6 — Linux adaptation of the SHM transport court

Standing: ALIVE (every claim backed by a command+exit run this session,
2026-11-02).

## W0 Mechanism (named, measured — not assumed)

Dispatch asked: check what the sandbox actually mounts. Witnessed this
session: docker linux/arm64 (golang:1.23-alpine, go1.23.12), container
`df /dev/shm` → `shm 65536 0 65536 0% /dev/shm` — tmpfs IS mounted, so the
shm arm is NOT blocked. The court names the mechanism at runtime via a
`Statfs` probe (`DevShmTmpfs()` checks `f_type == TMPFS_MAGIC 0x01021994`),
then backs segments in `/dev/shm` when it is a live tmpfs (true
POSIX-shared-memory pages, shm_open semantics without cgo) and falls back to
the portable IPC-4 tempfile+MAP_SHARED mechanism otherwise. Both arms are
pure stdlib, zero cgo (CGO_ENABLED=0 proven below).

## W1 Files (linux adaptations ONLY; every line disclosed)

All new, build-tagged `//go:build linux`, zero edits to IPC-4's files:

- `courts/interop-chicago/shm/shm_linux.go` — `DevShmTmpfs()` probe;
  `CreateShm(n)` returning `(seg, mechanism string, err)`; `createMapped`
  (truncate + MAP_SHARED mmap, the portable kernel primitive both OSes use).
- `courts/interop-chicago/shm/shm_linux_test.go` — `TestDevShmRoundtrip`
  (mechanism string asserted == "devshm(tmpfs)", second independent mapping,
  byte-exact vs `corpus.Encode`, per-fact equality); `TestDevShmMagicCorruptionFlip`
  (ANTI-VACUITY, both directions); `BenchmarkMmapReadLinux` (mechanism
  logged per run, never assumed).

Darwin-side fence: `go vet ./shm/` on darwin/arm64 → exit 0 — the tagged
files do not disturb the IPC-4 darwin court.

## W2 Commands + exits

| cmd | exit | result |
|---|---|---|
| `df /dev/shm` (in golang:1.23-alpine arm64) | 0 | tmpfs 64 MB mounted |
| `go vet ./shm/` (linux/arm64, in-container) | 0 | VET_OK |
| `CGO_ENABLED=0 go test -v ./shm/ -count=1` (linux/arm64) | 0 | 6/6 PASS |
| `go test ./shm/ -bench . -benchmem -count=3 -run xxx` | 0 | table below |
| `go vet ./shm/` (darwin/arm64 host, fence) | 0 | DARWIN_VET_OK |

The 6 passing courts: IPC-4's 4 (roundtrip, magic flip, zero-alloc, pread
baseline) + AX-6's 2 (devshm roundtrip, devshm magic flip).

## W3 Courts + anti-vacuity

1. `TestDevShmRoundtrip` — real /dev/shm tmpfs backing, real second
   independent PROT_READ mapping, byte-exact final state.
2. `TestDevShmMagicCorruptionFlip` — **ANTI-VACUITY**: corrupt byte 0 of the
   mapped header → typed `ErrMagic`; restore (second xor) → pass again.
   Flip proven BOTH directions; a green run without it is VACUOUS.
3. Inherited flips: `TestMagicCorruptionFlip` (tempfile arm) and
   `TestZeroAllocReadPath` (0 allocs/run witnessed on linux/arm64 too)
   PASS on linux.

No silent skips: the `[SKIP]` paths in both linux courts are loud, name the
probe and the still-running tempfile arm, and did NOT fire (tmpfs present).
Container arms LOUD-BLOCKED fallback was armed but not needed — docker
daemon was up, so the container runs happened.

## W4 Delta table (min-of-3, `go test -bench -benchmem -count=3`)

Measured with stated method this session; not generalized beyond these two
hosts.

| arm | darwin/arm64 (IPC-4, M3 Max) | linux/arm64 (AX-6, docker, 16 vCPU) | delta |
|---|---|---|---|
| mmap zero-copy read | 4.52–4.54 ns/op | 4.574–4.656 ns/op | +1%…+3% |
| pread fallback | 1377–1405 ns/op | 1185–1203 ns/op | −15%…−16% |
| allocs (both arms) | 0 | 0 | — |

Ratio mmap:pread ≈ 1:260 (darwin ~1:310) — the zero-copy arm dominates on
both OSes; the mechanism is portable with single-digit-percent cost delta.

## W5 Fences

Diff is additive, build-tagged linux-only; no shared file touched. Fences
that could regress: the IPC-4 darwin court — re-ran `go vet ./shm/` on
darwin, exit 0 (the darwin compile is the fence; the darwin courts
themselves were run by IPC-4 this session per lane-ipc-04-shm-transport.md
W1, re-read from disk before citing). Publish-frozen stdlib-only law holds:
CGO_ENABLED=0 run exit 0. `gofmt -l` flags `shm/shm.go` (PRE-EXISTING,
not this lane's file; disclosed, not fixed — not my ownership).

## W6 Divergences

- Dispatch anticipated container arms possibly LOUD-BLOCKED (image/podman
  not ready); docker was up and /dev/shm mounted, so the container arm ran
  for real — divergence from the anticipated-blocked scenario, resolved in
  the favorable direction.
- Pre-existing gofmt violation in shm.go (IPC-4's file): disclosed, not
  touched (disjoint ownership).
- No `shm_open` cgo arm was built: tmpfs-backed mmap IS shm_open semantics
  minus naming; building a cgo arm would violate the stdlib-only fence for
  zero semantic gain (IPC-4 W2 reasoning, re-derived for linux where tmpfs
  makes the equivalence exact).

## W7 Standing

ALIVE for all claims above, per-command exits in W2. Build lease
/tmp/ax6-gocache deleted at lane end. Kernel-bind arm remains IPC-0 NO-GO
(recorded, not re-litigated); the transport court now holds on BOTH OSes
and awaits the `cgap_shm_*` membrane.
