# The Representation Law: AM -> FM

Why cgap refuses to carry domain information in strings.

## The law

Strings encode information in a dimension coupled to the noise. Every
access to a string pays in the amplitude domain: the allocator, the
cache line, the parser. A fixed-width slot encodes the same information
orthogonally — constant width, constant time, no panic path — so the
signal no longer rides the noise dimension at all.

The phase-change charter states it as transition 1 (AM->FM,
`docs/jira/v26.11.1/phase-change-charter.md` §1): "Information moves
from a dimension coupled to the noise (allocator, cache, parse —
amplitude domain) into one orthogonal to it (fixed-width slots,
constant time, no panic path)."

## Why a diode, not a convention

A coding convention decays; a diode does not. The transition becomes
real only when strings cannot re-enter the runtime. The charter's own
instrument is `scripts/zero_string_gate.py` ("Diode ... Already
armed"): it scans compiled artifacts (never sources), extracts
per-section bytes, and refuses (exit 1) when any ontology-vocabulary
token appears in `.rodata`/`.text` on ELF/PE or `__TEXT` on Mach-O
(`scripts/zero_string_gate.py` module docstring, A07 ADR-002).

## Why the falsifier counts strings per receipt

Every wave receipt ends with `strings_admitted: int` — runtime string
paths added by that receipt (charter §1). Nonzero is not a warning;
it is the falsifier firing: "transition not happening." This is why
the gate is paired with a per-receipt counter instead of trusting the
gate alone: the gate proves the artifact, the counter proves the
direction of travel.

## The honest boundary

The charter does not claim the phase change in advance. "A phase
change is claimed only retrospectively from an armed cost curve going
vertical" (charter law, line 7-8). The witness curve is domain strings
in `.rodata` per execution crate, monotonic -> 0. If it stays flat,
the honest output is witnessed flatness (charter §Structural honesty).
