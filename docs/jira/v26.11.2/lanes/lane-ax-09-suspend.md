# Lane AX-9 Receipt — Suspend/Resume Probe (UNVERIFIED-by-design)

Date: 2026-11-02. Repo: /Users/sac/cgap (main, canonical). No git commands run.

## W1 Identity

- Lane AX-9. Owned files ONLY:
  - `/Users/sac/cgap/courts/ax-testbed/runners/suspend_probe.sh`
  - this receipt.
- Gate authority: `docs/jira/v26.11.2/lanes/lane-ax-00-gate.md:44-55` —
  AX-9 verdict NO-GO for cluster execution (ax 127, cluster exit 1,
  install NOT APPROVED). This lane lands the PROBE SCRIPT and documents
  the required sequence. No results simulated. ax NOT installed.

## W2 Commands + Exits (this session)

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `bash -n suspend_probe.sh` | 0 | SYNTAX-OK (92 lines) |
| 2 | `chmod +x suspend_probe.sh` | 0 | executable |
| 3 | `./suspend_probe.sh` | **3** | **REFUSAL WITNESS (real)**: `REFUSED[PREREQ-ABSENT]: ax CLI (ax.io/v1alpha1 surface: apply/describe/suspend/resume)` + gate-citation line, exit 3 typed |
| 4 | `command -v ax kubectl` | 1 | ax ABSENT; kubectl PRESENT (/opt/homebrew/bin/kubectl) — consistent with gate W2 rows 1-2 |

## W3 Script content summary (the executable spec for cluster GO)

`courts/ax-testbed/runners/suspend_probe.sh` encodes 5 human-resume
checkpoints: (1) control build in debug:false Task, artifact
`target/release/cgap-bn3` sha256 recorded; (2) debug:true Task + build
launched; (3) `ax suspend <task>` / `ax describe` / `ax resume <task>`;
(4) resumed-build digest compared to control via `cmp` — single
end-of-run compare, batch-amortized honesty law, ZERO sleeps anywhere
(mismatch => typed `CHECKPOINT_MISMATCH`, exit 1); (5) PASS only on
digest equality. Prereq gate (a) names EVERY absent prereq and exits 3.

## W4 Vendor claim this probe would witness

VENDOR-CLAIM-UNVERIFIED: the ax README checkpoint story — suspend
freezes the Task workspace, resume continues it losslessly. It has ZERO
in-session witnesses (gate W3: ax surface is vendor-README material).
A future green run of this script would be the FIRST witness; the
digest compare is the falsifier. Until then the claim stands
UNVERIFIED, loudly.

## W5 Verification ladder

Refusal witness is the only executable run this session (exit 3, row 3).
No cluster, no ax, no build. Falsifier for this lane: any receipt
citing a suspend/resume PASS contradicts gate row `AX-9: NO-GO` and the
absence of ax on PATH (command -v exit 1) — hearsay until ax + a
reachable ate-system cluster exist.

## W6 Fences (must-not-regress surfaces)

None broken: script is a new standalone file; no lib.rs/Cargo.toml
edits; compile-freeze SLA not engaged; shared tree untouched (courts/
sandbox law respected — script refuses before touching any tree).

## W7 Standing

- Probe script syntax: ALIVE (bash -n exit 0; refusal run exit 3).
- Suspend/resume probe END-TO-END: **UNVERIFIED-by-design** — stated
  plainly; no simulation, no install, per gate verdict.
- ax CLI: BLOCKED (absent; install NOT APPROVED).
- kubectl: present; cluster endpoint NO-GO (gate W2 row 3).

