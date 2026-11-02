# The Four Sovereign Functions

Why the entire cgap capability crosses the C-ABI boundary as exactly
four functions.

## The decision

A19 §2 ("ADR: the four sovereign entrypoints") fixes the boundary at
four functions — admit, eval, commit, verify:

1. `cgap_admit_frame` — bytes enter; magic/format/SHACL gate or typed
   refusal.
2. `cgap_eval_bn3` — admitted bytes evaluate deterministically against
   a flash-resident `.bn3` network; zero copies of the input.
3. `cgap_commit_parquet` — evaluated batches commit to Parquet.
4. `cgap_verify_p28` — P28 spoliation evidence verification,
   read-only.

## Why exactly four

A19 §2 gives the law directly: "admit/eval/commit/verify are the
irreducible verbs of the governed-process loop (admission ->
manufacture -> durability -> replay). Any consumer (Python host,
AtomVM node, bare-metal sensor) composes these four; a fifth verb is
either one of these in costume or it belongs in a lane, not the ABI."

Each half of that sentence carries weight. "In costume" means the
verb adds no new admission class — it is a parameterization of one of
the four (a specialized ingest is `admit`; a query is `verify`).
"Belongs in a lane" means it is genuinely new capability, and new
capability is admitted through the lane/court process and compiled
in, not leaked across the boundary ad hoc. The ABI is a constitution,
not an API: it enumerates the ways the outside world may act on the
substrate, and constitutions grow by amendment with receipts, not by
convenience.

## The economics of small

A19 also states the cost argument: fewer functions = smaller `cgap.h`
= smaller header-court diff surface = fewer chances the header and
Rust drift apart silently. Every additional export is a permanent
parity obligation between the C header and the Rust side (mirrored
`CGAP_ABI_VERSION` constants, asserted by the header court).

## Relationship to the frozen surface

The four sovereigns are additive: existing slot exports stay
byte-identical, Gate 3 wire-parity keeps passing unmodified, and the
sovereigns never touch `ARRAY_SLOT`/`SCHEMA_SLOT` (A19 §2).
Additivity is what makes "exactly four" stable rather than
aspirational.
