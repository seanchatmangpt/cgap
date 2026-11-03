# Lane E-07 — claims register end-to-end audit (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
audit executed by the E-07 Explore lane; read-only). Post-audit note: the
systemic finding it raised was closed by QUAL-01's qualifier wave +
FS-13FIX's 9-violation closure (flip court 0).

## W1 control plane — verified

Claims register 11-measured-claims.md; method per 06-bench-protocol (divan
fastest/median); batch window FACT_WINDOW=1,365 / RECORD_CAP=64 /
RING_WINDOW_BYTES=32KiB at pack_demux.rs (~:41-54, coprimality rationale
1,365 = 3·5·7·13); guard verdicts consistent (row 5 in-tolerance; row 28
magnitudes not reproduced under the weaker 4bw+8cpu dose — typed
non-reproduction, not contradiction).

## W2 rows — all resolve with scoping at every quote point

Rows 1-29 + §4 30/31/31a/32 spot-verified (5/6/7/8/28/31/34/38): each
carries host-scope / divergence / refusal-leg typing; receipt cites
(w3-l1-ingress.md:63,69-70) resolve to exact figures.

## W3 systemic finding — CONFIRMED then CLOSED

Bare/unscoped 10.20 ns/frame on 10+ exec/rollup surfaces (position-paper-v3:
207, claims-appendix:30, 17-c4:147-151 + stale :26,27 cites, 14-dflss-charter:
40,53, 16-sipoc:38,58, 00-dmadv-rollup:39,209, 15-ctq-tree:47,51,
02-pareto:165-166, p4-terminus:29, tps-01:22,32, 05-cpk:66-73,
01-project-charter:37,41). Closed by QUAL-01's 14-site qualification wave +
FS-13FIX. Post-audit re-read divergences: 05-cpk:66-73 does carry host-scope
text (sub-finding not fully reproduced); 01-project-charter:37,41 already
subject-scoped (STALE sub-finding); QUAL-01's own closure record has no
on-disk lane artifact (recorded as a governance gap, since covered by
FS-13FIX's flip court).

## W4 $50M / zero-headcount — pass

All $50M hits are the A17 ceiling pin or floor-vs-ceiling disambiguation;
zero-headcount exists only inside RED-16's UNSUPPORTED finding.

## W5 standing

TRANSCRIBED, landed by the coordinator. Original audit: E-07 Explore lane.
