# Lane T-X03 — Publish-Set Audit Transcription (X-03)

Status: TRANSCRIBED — archival transcription of lane X-03's publish-set
re-verification report, landed by transcription lane T-X03. Claims below are
X-03's; the transcriber re-verified only the file:line cites named in the
report (see Divergences). No registry contact, no git write commands this
session.

## Scope

X-03 re-verified the v26.11.2 publish set without registry contact:

1. PUB-01 / PUB-02 / PUB-02b documents confirmed present on disk.
2. `cargo metadata` shows 23 workspace members; `fleet-ops` and `ocel2`
   absent from the member list (both standalone, `publish = false`).
3. Spot re-packages byte-identical to PUB-01's recorded sizes:
   cgap-blake3 29439 B, cgap-arrow 120238 B, cgap-p28 65723 B, all rc=0,
   staged under /tmp.
4. The cargo nightly `-d` flag rejection reproduced (cargo rejects `-d`
   where a prior run accepted it).
5. The 4 BLOCKED-DEP-ORDER rows re-confirmed on disk:
   - root `Cargo.toml:43` (workspace pin) + `crates/cgap-gate/Cargo.toml:19`
     require `cgap-branchless` 26.11.2 — registry 404.
   - `crates/cgap-ffi/Cargo.toml:53` + `crates/integration/Cargo.toml:19`
     require `cgap-audit` 26.11.2 — registry max 26.11.1.
6. wasm-footprint registry state remains UNVERIFIED (never rounded up).
7. Exclusion guards re-confirmed (boundary gate + zero-string gate
   fail-closed on disk).

## Status note

PUB-02b's refreshed table plus FS-16's runbook cross-check are the current
carriers of the publish-set truth; this receipt is a transcription of a
re-verification against them, not a supersession.

## Divergences (transcriber)

- Dispatch cited "root Cargo.toml:28" for the branchless 26.11.2 pin; line
  28 is the `crates/cgap-gate` member entry. The version pin actually lives
  at `Cargo.toml:43` (`cgap-branchless = { path = ..., version = "26.11.2" }`).
  Transcribed with the corrected cite; the BLOCKED-DEP-ORDER substance is
  unchanged.
- All other cites verified as-quoted this session:
  `crates/cgap-gate/Cargo.toml:19`, `crates/cgap-ffi/Cargo.toml:53`,
  `crates/integration/Cargo.toml:19`.
- Registry-dependent claims (404 / max 26.11.1 / wasm-footprint) are
  X-03's observations carried forward UNVERIFIED here — no registry
  contact was permitted this session, per lane contract.
