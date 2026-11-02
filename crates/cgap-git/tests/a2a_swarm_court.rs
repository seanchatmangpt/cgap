//! Lane GIT-15 mutation court: agent lease table + coordinate zoning
//! + per-agent CAS refs. Chicago discipline: real collaborators, final
//! state asserts, no mocks. Anti-vacuity flip twins: the cross-zone
//! refusal court inverts (returns Ok) if the bitmask-AND zone gate is
//! removed; the foreign-ref-path court inverts if the namespace gate
//! is removed; the revoke court inverts if the revoked gate is
//! removed. Each twin names its mutation in the test body.

// Module wired in by #[path] (lib.rs off limits this lane). If the
// coordinator later adds `pub mod a2a_swarm;` to lib.rs, this include
// becomes a loud duplicate-definition error - never a silent skip.
#[path = "../src/a2a_swarm.rs"]
mod a2a_swarm;

use a2a_swarm::*;
use std::time::Instant;

const A: AgentId = AgentId(1);
const B: AgentId = AgentId(2);
const C: AgentId = AgentId(3);

fn oid(seed: u8) -> [u8; REF_LEN] {
    let mut o = [0u8; REF_LEN];
    o[0] = seed;
    o
}

/// 20 interleaved agent sessions (round-robin, single thread), 3 CAS
/// rounds. Asserts no cross-lease contamination of head refs and
/// reports sequential throughput. Concurrency model, stated honestly:
/// sequential interleaved ingress over a single-writer-per-agent
/// structure; no SMP / lock-free claim is made or tested. Zero locks
/// exist, so "zero lock contention" is by construction, not a
/// measured SMP property.
#[test]
fn court_clean_20_agent_interleaved_ingress() {
    let mut t = LeaseTable::new();
    let mut expected: [[u8; REF_LEN]; 20] = [[0u8; REF_LEN]; 20];
    for i in 0..20u64 {
        let zone = ZoneMask::coord((i / 64) as usize, (i % 64) as u32);
        t.register(AgentId(100 + i), zone)
            .unwrap_or_else(|e| panic!("register agent {i} refused: {e:?}"));
    }
    assert_eq!(t.active_count(), 20);

    let mut ops = 0u64;
    let start = Instant::now();
    for _round in 0..3 {
        for i in 0..20u64 {
            let a = AgentId(100 + i);
            let cur = t.head(a).unwrap();
            assert_eq!(cur, expected[i as usize], "contamination on agent {i}");
            let next = oid((ops % 251) as u8 + 1);
            let zone = ZoneMask::coord((i / 64) as usize, (i % 64) as u32);
            let got = t
                .propose_commit(a, a, &zone, &cur, &next)
                .unwrap_or_else(|e| panic!("clean push refused for agent {i}: {e:?}"));
            assert_eq!(got, next);
            expected[i as usize] = next;
            ops += 1;
        }
        // Namespace isolation sweep after each round.
        for i in 0..20u64 {
            assert_eq!(
                t.head(AgentId(100 + i)).unwrap(),
                expected[i as usize],
                "cross-lease contamination after round"
            );
        }
    }
    let elapsed = start.elapsed();
    eprintln!(
        "GIT-15 bench: 20 agents x 3 rounds = {ops} CAS ops in {elapsed:?} \
         ({:.0} ops/s); sequential interleaved, single-writer-per-agent",
        ops as f64 / elapsed.as_secs_f64()
    );
    assert_eq!(ops, 60);
}

/// ANTI-VACUITY MUTATION: delete the `s.agent != agent` clause in
/// check_zone (or the intersects AND itself) and this court inverts:
/// the cross-zone push returns Ok instead of ForeignZone.
#[test]
fn court_cross_zone_push_refused_andon() {
    let mut t = LeaseTable::new();
    t.register(A, ZoneMask::coord(0, 0)).unwrap();
    t.register(B, ZoneMask::coord(0, 1)).unwrap();
    // A pushes into B's coordinate zone: AndonRefusal class.
    let b_before = t.head(B).unwrap();
    let err = t
        .propose_commit(A, A, &ZoneMask::coord(0, 1), &oid(1), &oid(2))
        .unwrap_err();
    assert_eq!(err, SwarmError::ForeignZone { holder: B });
    assert_eq!(
        t.head(B).unwrap(),
        b_before,
        "B ref mutated by refused push"
    );
    // A's own zone still admits (gate skips the proposer).
    let a_before = t.head(A).unwrap();
    t.propose_commit(A, A, &ZoneMask::coord(0, 0), &a_before, &oid(2))
        .unwrap_or_else(|e| panic!("own-zone push must admit: {e:?}"));
    assert_eq!(t.head(A).unwrap(), oid(2));
}

/// ANTI-VACUITY MUTATION: delete the `ref_owner != agent` namespace
/// gate and this court inverts: pushing into B's ref path returns Ok.
#[test]
fn court_foreign_ref_path_refused() {
    let mut t = LeaseTable::new();
    t.register(A, ZoneMask::coord(0, 0)).unwrap();
    let err = t
        .propose_commit(A, B, &ZoneMask::coord(0, 0), &oid(1), &oid(2))
        .unwrap_err();
    assert_eq!(err, SwarmError::ForeignRefPath { owner: B });
}

/// ANTI-VACUITY MUTATION: drop the revoked gate in propose_commit and
/// this court inverts: the post-revoke push returns Ok.
#[test]
fn court_revoke_blocks_further_pushes() {
    let mut t = LeaseTable::new();
    t.register(A, ZoneMask::coord(0, 5)).unwrap();
    let cur = t.head(A).unwrap();
    t.revoke(A).unwrap();
    let err = t
        .propose_commit(A, A, &ZoneMask::coord(0, 5), &cur, &oid(9))
        .unwrap_err();
    assert_eq!(err, SwarmError::LeaseRevoked(A));
    // Released zone is re-grantable to a fresh agent.
    t.register(C, ZoneMask::coord(0, 5)).unwrap();
    // Re-registration of the revoked id stays refused.
    let err = t.register(A, ZoneMask::coord(0, 9)).unwrap_err();
    assert_eq!(err, SwarmError::LeaseRevoked(A));
}
