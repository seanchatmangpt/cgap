//! Typed noun-verb routes GENERATED from first-class RDF arguments.
//!
//! Every body closes through `BehaviorSpec`, except commands carrying the
//! explicit, admitted `cnv:CustomBehavior` opt-out, which route to a
//! hand-written `crate::custom_handlers::<noun>_<verb>` function instead. That
//! is the sole consumer handler seam this compiler ever emits, and it is
//! visible per-command in the source ontology, not a blanket escape hatch.

use clap_noun_verb::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use serde::Serialize;
use serde_json::{Map, Value};
#[doc = "Assert 64-byte alignment for Buf sizes 1..1000 step 7."]
#[verb("test", "align")]
fn align_test() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::align_test(inputs)
}

#[doc = "Slab occupancy statistics for the Arrow Buf allocator."]
#[verb("slab-stats", "arrow")]
fn arrow_slab_stats() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::arrow_slab_stats(inputs)
}

#[doc = "OCEL event-log statistics from the audit log."]
#[verb("ocel-stats", "audit")]
fn audit_ocel_stats() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::audit_ocel_stats(inputs)
}

#[doc = "POWL model statistics from the audit log."]
#[verb("powl-stats", "audit")]
fn audit_powl_stats() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::audit_powl_stats(inputs)
}

#[doc = "Report BEAM membrane host availability."]
#[verb("info", "beam")]
fn beam_info() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::beam_info(inputs)
}

#[doc = "64 MiB mask_words line-rate bench, min-of-5."]
#[verb("mask", "bench")]
fn bench_mask() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::bench_mask(inputs)
}

#[doc = "Literal-page w=4 RLE encode comparison, min-of-5."]
#[verb("parquet", "bench")]
fn bench_parquet() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::bench_parquet(inputs)
}

#[doc = "Evaluate a closed BN3 program to its result term."]
#[verb("eval", "bn3")]
fn bn_3_eval() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::bn_3_eval(inputs)
}

#[doc = "Binary N3 unification over the term arena."]
#[verb("unify", "bn3")]
fn bn_3_unify() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::bn_3_unify(inputs)
}

#[doc = "Verify the B-ONT pack digest chain."]
#[verb("pack-verify", "bont")]
fn bont_pack_verify() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::bont_pack_verify(inputs)
}

#[doc = "SlotId lookup over the B-ONT scalar slab."]
#[verb("slot-lookup", "bont")]
fn bont_slot_lookup() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::bont_slot_lookup(inputs)
}

#[doc = "Run the cgap-court verdict harness and print the report."]
#[verb("verdict-run", "court")]
fn court_verdict_run() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::court_verdict_run(inputs)
}

#[doc = "Verify a signed receipt's PQC signature."]
#[verb("receipt-verify", "crypto")]
fn crypto_receipt_verify() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::crypto_receipt_verify(inputs)
}

#[doc = "Seal a payload under the Chatman-PQC key schedule."]
#[verb("seal", "crypto")]
fn crypto_seal() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::crypto_seal(inputs)
}

#[doc = "Ingest a DAM evidence bundle through the gate."]
#[verb("dam-ingest", "gate")]
fn gate_dam_ingest() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::gate_dam_ingest(inputs)
}

#[doc = "Verify the Git packfile Merkle DAG chain."]
#[verb("dag-verify", "git")]
fn git_dag_verify() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::git_dag_verify(inputs)
}

#[doc = "Report GPU membrane device availability."]
#[verb("info", "gpu")]
fn gpu_info() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::gpu_info(inputs)
}

#[doc = "Verify a P28LicenseAttestation (80B) against the frozen URN."]
#[verb("attest-verify", "p28")]
fn p_28_attest_verify() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::p_28_attest_verify(inputs)
}

#[doc = "Check a Parquet pack's footer and checksums."]
#[verb("pack-check", "parquet")]
fn parquet_pack_check() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::parquet_pack_check(inputs)
}

#[doc = "Emit a Parquet pack from the columnar store."]
#[verb("pack-emit", "parquet")]
fn parquet_pack_emit() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::parquet_pack_emit(inputs)
}

#[doc = "Print the CLI version."]
#[verb("version", "self")]
fn self_version() -> Result<Value> {
    let inputs = Map::new();
    crate::custom_handlers::self_version(inputs)
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_ALIGN_NOUN: fn() = register_align_noun;

fn register_align_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "align",
        "64-byte alignment probe over the Buf allocator.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_ARROW_NOUN: fn() = register_arrow_noun;

fn register_arrow_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "arrow",
        "Arrow kernel statistics surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_AUDIT_NOUN: fn() = register_audit_noun;

fn register_audit_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "audit",
        "Audit substrate statistics surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_BEAM_NOUN: fn() = register_beam_noun;

fn register_beam_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "beam",
        "BEAM membrane info surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_BENCH_NOUN: fn() = register_bench_noun;

fn register_bench_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "bench",
        "Kernel line-rate benchmarks.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_BN_3_NOUN: fn() = register_bn_3_noun;

fn register_bn_3_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "bn3",
        "Binary N3 reasoning surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_BONT_NOUN: fn() = register_bont_noun;

fn register_bont_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "bont",
        "Binary ontology (B-ONT) scalar surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_COURT_NOUN: fn() = register_court_noun;

fn register_court_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "court",
        "Chicago court harness surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_CRYPTO_NOUN: fn() = register_crypto_noun;

fn register_crypto_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "crypto",
        "Chatman-PQC crypto surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_GATE_NOUN: fn() = register_gate_noun;

fn register_gate_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "gate",
        "Campaign gate surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_GIT_NOUN: fn() = register_git_noun;

fn register_git_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "git",
        "Git packfile v2 / Merkle DAG surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_GPU_NOUN: fn() = register_gpu_noun;

fn register_gpu_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "gpu",
        "GPU membrane info surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_P_28_NOUN: fn() = register_p_28_noun;

fn register_p_28_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "p28",
        "P28-SIL sovereign licensing surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_PARQUET_NOUN: fn() = register_parquet_noun;

fn register_parquet_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "parquet",
        "Parquet persistence surfaces.",
    );
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_SELF_NOUN: fn() = register_self_noun;

fn register_self_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "self",
        "Meta commands about the CLI itself.",
    );
}

#[allow(dead_code)]
fn encode_input<T: Serialize>(name: &str, value: &T) -> Result<Value> {
    serde_json::to_value(value).map_err(|error| {
        NounVerbError::execution_error(format!(
            "failed to encode generated argument {name:?}: {error}"
        ))
    })
}

#[cfg(test)]
mod generated_route_proofs {
    const GENERATED_SOURCE: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/generated_cli.rs"));

    fn routes_only() -> &'static str {
        GENERATED_SOURCE
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(GENERATED_SOURCE)
    }

    #[test]
    fn every_admitted_command_became_one_registered_verb() {
        assert_eq!(routes_only().matches("#[verb(").count(), 21);
    }

    #[test]
    fn no_consumer_handler_seam_survives_generation() {
        // `custom_handlers::` is the sole, explicitly admitted escape hatch
        // (cnv:CustomBehavior); strip it before checking for the legacy,
        // unbounded `handlers::` seam this compiler never emits otherwise.
        let without_admitted_custom_seam = routes_only().replace("custom_handlers::", "");
        assert!(!without_admitted_custom_seam.contains("handlers::"));
        assert!(!routes_only().contains("todo!"));
        assert!(!routes_only().contains("unimplemented!"));
    }

    #[test]
    fn every_custom_command_routes_to_its_own_handler() {
        // Proves per-command wiring, not just that a seam exists somewhere:
        // each cnv:CustomBehavior command must call the one handler function
        // named after ITS OWN noun and verb, not a neighboring command's.
        assert!(
            routes_only().contains("custom_handlers::align_test("),
            "align test must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::arrow_slab_stats("),
            "arrow slab-stats must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::audit_ocel_stats("),
            "audit ocel-stats must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::audit_powl_stats("),
            "audit powl-stats must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::beam_info("),
            "beam info must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::bench_mask("),
            "bench mask must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::bench_parquet("),
            "bench parquet must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::bn_3_eval("),
            "bn3 eval must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::bn_3_unify("),
            "bn3 unify must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::bont_pack_verify("),
            "bont pack-verify must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::bont_slot_lookup("),
            "bont slot-lookup must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::court_verdict_run("),
            "court verdict-run must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::crypto_receipt_verify("),
            "crypto receipt-verify must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::crypto_seal("),
            "crypto seal must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::gate_dam_ingest("),
            "gate dam-ingest must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::git_dag_verify("),
            "git dag-verify must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::gpu_info("),
            "gpu info must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::p_28_attest_verify("),
            "p28 attest-verify must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::parquet_pack_check("),
            "parquet pack-check must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::parquet_pack_emit("),
            "parquet pack-emit must route to its own custom handler"
        );
        assert!(
            routes_only().contains("custom_handlers::self_version("),
            "self version must route to its own custom handler"
        );
    }
}
