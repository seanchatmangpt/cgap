//! CSCT runner binary (RFC-0042 §7), lane CSCT-L12.
//!
//! Dogfoods the `csct` public API: `run_suite()`,
//! `mutation::run_mutation_controls()`, `receipt::render_report`,
//! `types::ExitCode`. No Python, no JSON, no serde — stdout is the
//! markdown receipt. Arg parse by hand (std::env::args, no clap).
//!
//! Exit mapping: Pass=0, GateFail=1, SubjectMissing=2, LoudSkip=3,
//! AndonRefusal=101; unknown subject -> 2 with a printed loud-skip
//! line; mutation-control Err -> 1 with a printed VACUOUS line.

use csct::mutation;
use csct::receipt;
use csct::run_suite;
use csct::types::ExitCode;
use csct::types::Report;

/// Accepted suite subject (RFC-0042 §7 registry, frozen).
const ACCEPTED_SUITE: &str = "cloud-cartel-1360shard-vs-cgap";
/// Accepted mutation-control subject (frozen).
const ACCEPTED_MUTATION: &str = "zero-tax";

/// Parsed CLI intent. `Unknown` carries the offending token for the
/// loud-skip line.
#[derive(Clone, Debug, PartialEq)]
enum Action {
    Run,
    Mutate,
    Receipt,
    Unknown,
}

/// Pure arg classification. `args` is the flag list (no argv[0]).
///
/// Shapes:
/// - []                                -> Run (default suite)
/// - ["--suite=<accepted>"]            -> Run
/// - ["--mutate=zero-tax"]             -> Mutate (controls run first)
/// - ["--receipt"]                     -> Receipt (markdown to stdout)
/// - anything else (unknown suite id, unknown flag, unknown mutation,
///   extra tokens)                     -> Unknown
fn classify(args: &[String]) -> Action {
    let mut action = Action::Run;
    for arg in args {
        match arg.as_str() {
            "--receipt" => action = Action::Receipt,
            _ => {
                if let Some(id) = arg.strip_prefix("--suite=") {
                    if id != ACCEPTED_SUITE {
                        return Action::Unknown;
                    }
                } else if let Some(id) = arg.strip_prefix("--mutate=") {
                    if id != ACCEPTED_MUTATION {
                        return Action::Unknown;
                    }
                    action = Action::Mutate;
                } else {
                    return Action::Unknown;
                }
            }
        }
    }
    action
}

/// Process exit code for a finished report: first non-Pass stage's
/// exit code wins; otherwise verdict (true=0, false=1).
fn exit_for(report: &Report) -> u32 {
    for stage in &report.stages {
        if stage.exit != ExitCode::Pass {
            return stage.exit.as_u32();
        }
    }
    if report.final_commercial_verdict {
        ExitCode::Pass.as_u32()
    } else {
        ExitCode::GateFail.as_u32()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match classify(&args) {
        Action::Unknown => {
            let tok = args.first().map(String::as_str).unwrap_or("");
            println!(
                "[SKIP] csct_runner: unknown subject/flag '{tok}' \
                     (accepted: --suite={ACCEPTED_SUITE}, --mutate={ACCEPTED_MUTATION}, --receipt)"
            );
            std::process::exit(ExitCode::SubjectMissing.as_u32() as i32);
        }
        Action::Mutate => {
            if let Err(exit) = mutation::run_mutation_controls() {
                println!(
                    "[VACUOUS] csct_runner: mutation controls failed \
                         (exit {}); refusing to admit an unmutated court",
                    exit.as_u32()
                );
                std::process::exit(exit.as_u32() as i32);
            }
            let report = run_suite();
            println!("{}", receipt::render_report(&report));
            std::process::exit(exit_for(&report) as i32);
        }
        action => {
            let report = run_suite();
            if let Action::Receipt = action {
                println!("{}", receipt::render_report(&report));
            }
            std::process::exit(exit_for(&report) as i32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn default_empty_args_is_run() {
        assert_eq!(classify(&[]), Action::Run);
    }

    #[test]
    fn accepted_suite_is_run() {
        assert_eq!(
            classify(&s(&["--suite=cloud-cartel-1360shard-vs-cgap"])),
            Action::Run
        );
    }

    #[test]
    fn mutate_shape_classifies() {
        assert_eq!(classify(&s(&["--mutate=zero-tax"])), Action::Mutate);
    }

    #[test]
    fn receipt_shape_classifies() {
        assert_eq!(classify(&s(&["--receipt"])), Action::Receipt);
    }

    #[test]
    fn unknown_suite_id_is_unknown() {
        assert_eq!(classify(&s(&["--suite=other-corp"])), Action::Unknown);
    }

    #[test]
    fn unknown_flag_is_unknown() {
        assert_eq!(classify(&s(&["--bogus"])), Action::Unknown);
    }

    #[test]
    fn unknown_mutation_id_is_unknown() {
        assert_eq!(classify(&s(&["--mutate=half-tax"])), Action::Unknown);
    }
}
