# Decision D3: NIF/BEAM-leg doubly refused
Standing: PARTIAL_ALIVE

## Decision

D3 DOUBLY-REFUSED: a NIF implementation of the wire-shm data path is refused
on two independent grounds, either of which alone blocks the work.

## Refusal 1 — smalt charter

The smalt charter pins the native surface to pure-Elixir with wasmex-only
native access. A compiled NIF is outside the charter's authorized surface; a
charter revision is a formal prerequisite, not a lane-level choice.

## Refusal 2 — cgap-beam scheduler CTQ

The cgap-beam scheduler CTQ FAIL: measured 7985µs against a ≤50µs budget
(instrument-class UNSUPPORTED — the measurement exists but does not support
the claim). NIF-class latency cannot be asserted from the current evidence;
only a probe can move it.

## Authorized next step

M0 probe is authorized as the ONLY lawful BEAM-leg next step: go threshold
≥2× p50 improvement or no-go. A probe-go does not lift either refusal — it
only unblocks a formal smalt charter revision, which is itself the second
gate.

## Falsifier

M0 probe-go (≥2× p50) AND a formal smalt charter revision landing together
refute this refusal. Either condition alone does not.

## Standing

REFUSED (doubly); M0 probe authorized at PARTIAL_ALIVE standing (authorized,
not yet executed).

Date: 2026-11-04
