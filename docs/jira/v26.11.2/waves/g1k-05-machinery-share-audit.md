# G1K-5 machinery-share audit — receipt

Lane: G1K-5 EXPLORE (read-only audit; artifacts landed coordinator-side). Repo:
/Users/sac/cgap. No git writes. /Users/sac/stardf untouched.

## Method

`git ls-files` census; bytes via
`git ls-files | grep -E '^<class>' | xargs wc -c | tail -1`.
Total: 903 files, 9,156,129 bytes.

## Census

- GGEN-PROJECTED (73 files, 109,885 B):
  - crates/*/src/generated_*.rs: 6 files, 44,916 B
  - docs/diataxis/marketplace-generated/**: 27 files, 12,183 B
  - crates/cli/docs/command-reference.md: 2,419 B
  - cgap-pack outputs (38 files, 50,356 B)
  - generated/subject-count.txt: 11 B
- HAND-MAINTAINED: docs/jira 414 files / 3,162,488 B; crates rs 4,123,816 B total (incl.
  projected); ontologies ~230,047 B; scripts 47 files / 590,104 B; manifests+locks ~14 KB;
  include/ 12,251 B; docs/ non-jira 70 files / 369,546 B; test(s)/examples/bindings/ex
  ~350 KB; .claude/ 33,593 B; root benches/ EMPTY (0 tracked). Correction: the live bench
  surface is crates/bench/benches/ (15 tracked files), not counted in the lane's root-dir
  grep.
- Machinery share today: 109,885 / 9,156,129 = 1.20%.

## Status

ALIVE (audit complete; roadmap landed at docs/jira/v26.11.2/10-ggen-max-projection.md).
Byte counts are measured, not TARGETs; ladder steps are [TARGET-v26.11.2].

{
  "lane": "G1K-5",
  "total_tracked_files": 903,
  "total_tracked_bytes": 9156129,
  "projected_files": 73,
  "projected_bytes": 109885,
  "machinery_share_pct": 1.20,
  "hand_residue_top": {"docs_jira_bytes": 3162488,
                       "crate_rs_bytes": 4123816,
                       "scripts_bytes": 590104},
  "custom_handlers_bytes": 36920,
  "artifacts": ["docs/jira/v26.11.2/10-ggen-max-projection.md",
                "waves/g1k-05-machinery-share-audit.md"],
  "stardf_touched": false,
  "git_commands_run": 0
}
