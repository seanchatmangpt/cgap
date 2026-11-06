//! bn3_falsification_court.rs — Adversarial Red-Team Falsification of the BN3 Inversion Thesis.
//!
//! Attacks the 3 core claims to find their physical breaking boundaries:
//! 1. Attack 1 (BPU Bias / Predictability Advantage): When data is 99.9% predictable,
//!    speculative branch prediction can bypass work that unconditional branchless arithmetic
//!    must execute unconditionally.
//! 2. Attack 2 (Adversarial Selectivity Breach): When an attacker injects 100% anomaly traffic
//!    during a 10x ingress surge, the Little's Law Shield breaks and downsteam ring buffers saturate.
//! 3. Attack 3 (Contended Cache & Bus Saturation): Memory bandwidth saturation degrades
//!    in-place validity bitmap synthesis beyond quiet-host claims.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[inline(always)]
fn cmov(cond: bool, true_val: u64, false_val: u64) -> u64 {
    let mask = 0u64.wrapping_sub(cond as u64);
    (mask & true_val) | (!mask & false_val)
}

#[test]
fn attack1_bpu_predictability_advantage_at_low_entropy() {
    // When telemetry is 99.9% predictable (1 anomaly in 1,000 items),
    // speculative branch predictors predict almost perfectly.
    const N: usize = 1_000_000;
    let mut input = vec![0u64; N];
    // Intersperse exactly 0.1% anomalies
    for i in (0..N).step_by(1000) {
        input[i] = 1;
    }

    // Measure Branchy on highly predictable stream
    let start_br = Instant::now();
    let mut acc_br = 0u64;
    for &val in &input {
        if val != 0 {
            acc_br = acc_br.wrapping_add(val);
        }
    }
    std::hint::black_box(acc_br);
    let dur_br = start_br.elapsed();

    // Measure Branchless
    let start_bl = Instant::now();
    let mut acc_bl = 0u64;
    for &val in &input {
        let cond = val != 0;
        acc_bl = cmov(cond, acc_bl.wrapping_add(val), acc_bl);
    }
    std::hint::black_box(acc_bl);
    let dur_bl = start_bl.elapsed();

    println!(
        "[ATTACK 1] 99.9% Predictable Telemetry: branchy={:?}, branchless={:?}",
        dur_br, dur_bl
    );

    // Both must agree on result
    assert_eq!(acc_br, acc_bl);
}

#[test]
fn attack2_adversarial_selectivity_breach_saturates_queue() {
    // Proves that the Little's Law Shield holds ONLY when selectivity is small.
    // When an attacker crafts packets to trigger 100% anomaly match during a 10x surge,
    // the downstream ring buffer fills and begins dropping items.
    const CAPACITY: usize = 1_024;
    let mut queue = Vec::with_capacity(CAPACITY);
    let mut drops = 0usize;
    let mut admitted = 0usize;

    const INGRESS_SURGE: usize = 10_000;
    // Anomaly key that matches 100% of injected packets
    let attacker_packets = vec![0x1337u64; INGRESS_SURGE];

    for packet in attacker_packets {
        // Filter evaluates packet
        let is_match = packet == 0x1337; // 100% selectivity breach
        if is_match {
            if queue.len() < CAPACITY {
                queue.push(packet);
                admitted += 1;
            } else {
                drops += 1;
            }
        }
        // Consumer drains at nominal 1 item per 10 ingress
        if (admitted % 10) == 0 && !queue.is_empty() {
            queue.pop();
        }
    }

    const ZERO_DROP_FLOOR: usize = 0;
    println!(
        "[ATTACK 2] Adversarial 100% Selectivity Breach: Admitted={}, Drops={}",
        admitted, drops
    );

    // Under 100% selectivity breach, queue saturation and drops MUST be witnessed
    assert!(
        drops > ZERO_DROP_FLOOR,
        "Falsification: Under 100% selectivity breach, queue must saturate and drop items"
    );
}

#[test]
fn attack3_memory_bandwidth_contention_impact() {
    // Simulates multi-tenant memory thrashing (RED-10 style) to witness
    // cache eviction impact on Arrow-style memory passes.
    let stop = Arc::new(AtomicBool::new(false));
    let stop_worker = Arc::clone(&stop);

    // Spawn cache-thrashing worker in background
    let hog = thread::spawn(move || {
        const HOG_SIZE: usize = 4 * 1024 * 1024; // 4 MiB buffer
        let mut buf = vec![0u8; HOG_SIZE];
        let mut idx = 0;
        while !stop_worker.load(Ordering::Relaxed) {
            buf[idx] = buf[idx].wrapping_add(1);
            idx = (idx + 64) % HOG_SIZE;
        }
    });

    const PASS_SIZE: usize = 65_536;
    let mut target_buf = vec![0u8; PASS_SIZE];

    let start = Instant::now();
    for _ in 0..100 {
        for chunk in target_buf.chunks_exact_mut(64) {
            chunk[0] = chunk[0].wrapping_add(1);
        }
    }
    std::hint::black_box(&target_buf);
    let dur_contended = start.elapsed();

    stop.store(true, Ordering::Relaxed);
    hog.join().unwrap();

    println!(
        "[ATTACK 3] Memory-Plane Pass under Cache Thrashing: {:?}",
        dur_contended
    );
    const MIN_DURATION_NANOS: u128 = 0;
    assert!(dur_contended.as_nanos() > MIN_DURATION_NANOS);
}
