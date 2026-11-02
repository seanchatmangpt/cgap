//! `doctor` — REDIRECT STUB.
//!
//! Per DXD-02 (`docs/jira/v26.11.2/audit/dxd-02-doctor-design.md`) and
//! CULL-02 (`docs/jira/v26.11.2/audit/cull-02-doctor-consolidation.md`),
//! the canonical doctor is `scripts/doctor_agent_lanes.py`. This binary
//! is a 5-line redirect: it execs the Python script and mirrors its exit
//! code. The 10-check Rust implementation lives in git history at this
//! path (pre-TC-02).
//!
//! ggen-regen hazard note (TC-02): `crates/cli` is a standalone ggen
//! crate. A grep of `ggen.toml` and `gen-templates/` for `doctor`
//! returned zero matches, and repo CLAUDE.md lists `doctor` among the
//! hand-maintained side-bins — this stub is expected to survive regen,
//! but if a future pack projects a generated bin/doctor.rs it will be
//! clobbered and the redirect dies loudly (nonzero exit, non-doctor
//! output), never silently.

fn main() {
    let status = std::process::Command::new("python3")
        .arg("/Users/sac/cgap/scripts/doctor_agent_lanes.py")
        .args(std::env::args().skip(1))
        .status()
        .expect("doctor redirect: failed to spawn python3 scripts/doctor_agent_lanes.py");
    std::process::exit(status.code().unwrap_or(2));
}
