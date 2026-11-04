// ZP-18-C: divan harness cannot compose assert_release per-bench; allowlisted.
// F2-10 scaffold stub — Divan entry point only. Per-bench families are
// filled by lanes F2-11..F2-14 in their own bench targets (ingress, parquet,
// bn3, seal). Canonical corpus seed pinned repo-wide: 0xBF58_476D_1CE4_E5B9.
fn main() {
    // divan 0.1.21 entry method is `run_benches` (divan-0.1.21/src/divan.rs:64);
    // the spec's `.run()` shape does not exist on the pinned version.
    divan::Divan::default().run_benches();
}
