# ZD-79 — gate0 Witness Receipt (ZD-63 rung 7)

Standing: BLOCKED — exit 2 twice, identical refusal (deterministic)

Status: **BLOCKED** — gate0 refuses at the EA admission gate before reaching the byte-stable replay proof or header compile.

## Subject

- Repo: /Users/sac/cgap @ 0f067ee733cf63dcb62039a27eb5a38fc0860ab9
- Tool: ambient ggen 26.11.3 (`/Users/sac/.local/bin/ggen`, current per ZD-17)
- Recipe: `make gate0` (Makefile lines 46–56): `ggen sync run` ×2 → sha256 of GATE0_FILES → `cmp` → `cc -std=c99 -ffreestanding` header compile
- Date: 2026-11-04

## Execution

Run 1: exit 2 (make target failed). Run 2 (rerun): exit 2, identical refusal — deterministic BLOCKED, not flaky.

Exact refusal (verbatim, both runs):

```
ERROR: CLI execution failed: Command execution failed: validation error: [FM-LAW-015] EA admission gate: `/Users/sac/cgap/ea.graph.json` is absent and no admission opt-out was given. Every sync must either admit its candidate SBBs against an EA graph or explicitly skip with `--skip-ea-admission`. Remediation: add ea.graph.json, or pass --skip-ea-admission to run without architecture admission.
```

## Why not skipped

The gate0 recipe invokes bare `ggen sync run` — it does NOT pass `--skip-ea-admission` through, and no `ea.graph.json` exists at repo root (checked: absent). Per dispatch, the disclosed skip is only lawful if the recipe itself passes it; it does not, so the run is recorded BLOCKED, not forced.

## What was NOT witnessed

- Byte-stable replay (`cmp` of run1/run2 digests) — unreached
- Freestanding C header compile — unreached
- Output digest: none produced; /tmp/cgap_gate0_run{1,2}.sha never written

## Close condition

Either commit an `ea.graph.json` at repo root, or amend the gate0 recipe to pass `--skip-ea-admission` explicitly. Until one of those lands, ZD-63 rung 7 remains UNWITNESSED.
