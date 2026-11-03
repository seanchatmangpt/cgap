# ADV-08 — Charter scope creep & retroactive target inflation (corrosion wave)

Date: 2026-11-02. Repo: /Users/sac/cgap (main). Contract: READ-ONLY;
write target = this file only. No git commands. Fail-closed logging.

Attack: the "CLOSED" verdict on the red-12 SRAM arm is goalpost-moving —
the 128 KiB ceiling was a hardware-envelope claim; the "fix" is a
software gate that REFUSES the bn3 enclave surface entirely (workload
rejected, not shrunk to fit).

## A1 Probe 1 — is the CLOSED cell scoped honestly?

VERDICT: HONEST. The cell refuses to claim bn3 fits. Quote
(`dflss/01-project-charter-ax-testbed.md:221-229`):

> "[CLOSED 2026-11-02] The red-12 SRAM arm of this cell is closed: the
> bn3 static-state overburden (6,520,984 B = 49.75x the 131,072 B
> ceiling) is now REFUSED at admission, not standing"

and it closes the cell as "REFUSED at admission, not standing" — the
cell says refused, never "fits". The only residual overclaim risk is
the word "closed" itself on a Muri row whose other arm (10 ns USL vs
85 ns floor) explicitly "remain[s] open"; the SRAM arm closure is a
typed-refusal closure, which the cell states in full.

## A2 Probe 2 — does ANY doc claim the full engine fits the envelope?

SPOILER-REFUTED (no spoiler found). Repo-wide sweep for
"fits the envelope"/"fits the 128"/"fits within" (run this session,
redteam excluded):

- `dflss/02-voc-ctq-matrix.md:219-227`: "M-CTQ-03 SRAM fit — VERIFIED
  (SCOPED to gated subjects); bn3 gated-and-REFUSED ... Envelope
  verdict: baseline subject fits (VERIFIED); bn3 over ceiling and
  REFUSED by gate (loud, not silent)." — scoped, not inflated.
- `research/nanoarrow-oracle-synthesis.md:52` hits the substring
  "fits the 128 KiB gate" but its subject is cgap-form const tables,
  not bn3 state tables — not a bn3-fits claim.
- Zero hits claiming the full engine fits. SPOILER-CONFIRMED = NO.

## A3 Probe 3 — was bn3-fits ever claimed pre-wave? (goalpost check)

VERDICT: NEVER CLAIMED, so no goalpost moved. The original CTQ-05
(`dflss/02-ax-charter-ctq.md:131-148`) measured 36,048 B over the
gated subject list ("blake3+branchless", per red-12 W3: the gate's
`DEPENDENCY_CRATES` excluded cgap-bn3) and its honest bound was the
"minimum entry surface" pass line, which survives verbatim in today's
gate output ("csct/branchless kernel surface fits the physical
envelope (minimum entry surface; headroom 95024 B)"). red-12 W3
characterized the pre-wave posture exactly: the envelope claim "is
honest about what it measures and silent about bn3". Silence is not a
claim; a gate widened to make prior silence loud is scope-honesting,
not retroactive target inflation. The refusal-gate posture is the
honest closure of an honestly-scoped (silently-scoped) original claim.

## A4 Probe 4 — is RC-03's bound operable at init, or documentation-only?

VERDICT: OPERABLE. `crates/cgap-bn3/src/noalloc.rs:137-147`
(`init_enclave` → `init_within`): the Err path returns
`CapacityExceeded { required_bytes, ceiling_bytes }` BEFORE any static
is materialized (`Err` arm precedes `Self::zeroed()`); the Ok path is
the real constructor. Re-run this session (ADV-08, exit 0,
`3 passed; 0 failed`):

- `CARGO_TARGET_DIR=/tmp/adv08-target cargo test -p cgap-bn3
  --test capacity_bound_court` → exit 0, 3 passed
- `python3 scripts/verify_cscb_sram.py` → exit 0 (36,048 B baseline)
- `python3 scripts/verify_cscb_sram.py --subject bn3` → exit 1,
  `BN3_ENCLAVE_REFUSED: static noalloc state-table charge 6520984 B
  ... > 131072 B hard bound (49.75x ceiling; red-12 topology)`
  (string re-read on disk this session).

## A5 Standing

ALIVE. Commands + exits run this session against the exact subject.
