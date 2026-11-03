# Lane E-12 — GAP-04 terminal receipt verification (archival transcription)

Date: 2026-11-02 · Repo: /Users/sac/cgap · Standing: TRANSCRIBED (original
verification executed by the E-12 Explore lane; read-only). Both open items
it recorded were subsequently closed: the ffi failure superseded-green by
R-ADV10FIX (83/0), and the cfg-warning site corrected by WARN-01's errata +
the Cargo lints fix.

## W1 build 101→0 transient — CONFIRMED

R-ADV11FIX mid-write (mismatched delimiters at ref_store.rs:842/847/853,
mtime 19s before check); bounded re-poll exit 0; /tmp/gap04-build2.log
corroborates 17 warning lines (= 15 concrete + 2 summary).

## W2 battery — CONFIRMED

4/4 + 8/8 rollups matching R-05 (audit doc :30-38).

## W3 fences — CONFIRMED

bn3 139/0, storage 51/0, p28 50/0, cgap-git exit 0 (flip court by design).

## W4 ffi 101 — real-then, superseded

fa9_rename_race_denied panic at fa9_commit_gating.rs:464, reproduced on
bounded re-run; SUPERSEDED by lane-r-adv10fix W3 (exit 0, 83 tests 0
failed, :48-50/:87).

## W5 lib.rs corruption — non-reproducing, holds

364 lines read clean (W1, W7 div 2).

## W6 warning census — substance confirmed, site misattributed

14 pre-existing + 1 wave-introduced confirmed; the audit's own ERRATA
(:103-110) renames the site to ref_store.rs:230:13 (second use :224);
pkt_line.rs carries ZERO cfg attributes (E-12 independent grep). WARN-01
closure confirmed: [lints.rust] unexpected_cfgs with check-cfg added to
crates/cgap-git/Cargo.toml (:111-120 of the ERRATA), warning gone, lib
delta 12→11, width-arms court 8/8 fence.

## W7 standing

TRANSCRIBED, landed by the coordinator. Original audit: E-12 Explore lane.
