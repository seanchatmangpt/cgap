//! Capacity-bound court (lane RC-03; RED-12 closure, P3 roadmap).
//!
//! RED-12 (`docs/jira/v26.11.2/redteam/red-12-sram-overflow-exploit.md`)
//! proved the noalloc state tables at full NA_MAX_* consts charge
//! 6,520,984 B = 49.75x the 131,072 B enclave ceiling. FIX-02 landed the
//! STATIC gate refusal; this court proves the ENGINE's init-time typed
//! refusal (noalloc.rs RC-03 section: `init_enclave` / `init_within` /
//! `na_init_capacity_check`) actually fires.
//!
//! Anti-vacuity mutation: raise NA_MAX_MATCHES 2048 -> 4096 in a /tmp
//! sandbox copy of the crate; the exact-pin assertions below MUST flip
//! (charge doubles), witnessed in the lane receipt.

use cgap_bn3::noalloc::{
    na_enclave_charge_bytes, na_engine_bytes, na_init_capacity_check, na_matches_bytes,
    CapacityExceeded, NaEngine, NaMatches, NA_SRAM_CEILING_BYTES,
};

/// Red-12 W5 host size_of ground truth (gate pins BN3_PIN_*_BYTES).
const PIN_MATCHES_BYTES: usize = 2_162_696;
const PIN_ENGINE_BYTES: usize = 4_358_288;
/// Sum, red-12 W5.
const PIN_TOTAL_BYTES: usize = 6_520_984;

#[test]
fn charge_model_matches_red12_sizeof_ground_truth() {
    assert_eq!(na_matches_bytes(), PIN_MATCHES_BYTES);
    assert_eq!(na_engine_bytes(), PIN_ENGINE_BYTES);
    assert_eq!(na_enclave_charge_bytes(), PIN_TOTAL_BYTES);
    assert_eq!(NA_SRAM_CEILING_BYTES, 131_072);
    // The same arithmetic the gate uses: ~49.75x over the ceiling.
    assert!(
        na_enclave_charge_bytes() > 49 * NA_SRAM_CEILING_BYTES,
        "charge {} not > 49x ceiling {}",
        na_enclave_charge_bytes(),
        NA_SRAM_CEILING_BYTES
    );
}

/// Full-capacity init at the physical ceiling -> typed refusal, both
/// buffers, and the error carries the exact red-12 numbers. Final-state
/// assertions; nothing is constructed on Err (a construction would blow
/// the 2 MiB test stack — the refusal path is the proof).
#[test]
fn full_capacity_init_at_enclave_ceiling_refuses_typed() {
    // Even the Err path returns a Result<NaMatches, _> slot (~2.16 MB) on
    // the caller stack — run under a 64 MiB thread like the Ok path.
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(full_capacity_refusal_body)
        .expect("spawn on a 64 MiB stack")
        .join()
        .expect("refusal court ok");
}

fn full_capacity_refusal_body() {
    let m = NaMatches::init_enclave();
    assert_eq!(
        m,
        Err(CapacityExceeded {
            required_bytes: PIN_MATCHES_BYTES,
            ceiling_bytes: NA_SRAM_CEILING_BYTES
        })
    );
    let e = NaEngine::init_enclave();
    let Err(err_e) = e else {
        panic!("NaEngine::init_enclave must refuse at the 131072 B ceiling");
    };
    assert_eq!(
        err_e,
        CapacityExceeded {
            required_bytes: PIN_ENGINE_BYTES,
            ceiling_bytes: NA_SRAM_CEILING_BYTES
        }
    );
    // Combined check: same typed shape, combined charge.
    assert_eq!(
        na_init_capacity_check(NA_SRAM_CEILING_BYTES),
        Err(CapacityExceeded {
            required_bytes: PIN_TOTAL_BYTES,
            ceiling_bytes: NA_SRAM_CEILING_BYTES
        })
    );
}

/// Reduced-capacity / larger-budget path: the same constructors return
/// Ok when the embedder declares a ceiling the charge fits. NaEngine is
/// ~4.3 MB — constructed on a dedicated big-stack thread (the in-crate
/// courts Box it; an integration test cannot reach `boxed_zeroed`).
#[test]
fn declared_ceiling_that_fits_constructs_ok() {
    let budget = na_enclave_charge_bytes();
    // NaMatches itself is ~2.16 MB — its Result slot also exceeds the
    // default 2 MiB test stack, so the whole Ok path runs big-stack.
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let m = NaMatches::init_within(budget).expect("matches within own charge");
            assert!(!m.contains(0), "fresh buffer holds no matches");
            let _e = NaEngine::init_within(budget).expect("engine within own charge");
            assert_eq!(
                na_init_capacity_check(budget),
                Ok(budget),
                "combined check Ok at exact charge"
            );
        })
        .expect("spawn on a 64 MiB stack (NaEngine is ~4.3 MB)")
        .join()
        .expect("big-stack thread ok");
}
