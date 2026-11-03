//! A6 — SLO snapshot + host-side bounded-poll alerting (R-A6.1/A6.2).
//!
//! The kernel exposes state read-only via a typed snapshot; the host
//! polls with a bounded deadline and evaluates typed alert predicates.
//! No sleeps: the poll is bounded by iteration budget, and every poll
//! step observes real snapshot state.

use crate::fleet::NodeId;

/// CGAP status codes carried in the SLO histogram. Never conflated:
/// values pinned per repo CLAUDE.md (`sovereign.rs:80/87`, `trap.rs:36`).
pub mod status_codes {
    /// 0x4347_0004 CGAP_ERR_LEASE_EXHAUSTED
    pub const LEASE_EXHAUSTED: u32 = 0x4347_0004;
    /// 0x4347_0006 CGAP_ERR_CONCURRENT_WRITER_DENIED
    pub const CONCURRENT_WRITER_DENIED: u32 = 0x4347_0006;
    /// 0x5032_38FF CGAP_ERR_P28_SPOLIATION
    pub const P28_SPOLIATION: u32 = 0x5032_38FF;
}

/// Typed SLO snapshot (R-A6.1): kernel-side stats feed consumed
/// read-only by the host poller. Round-trips through fixed-width bytes
/// with a checksum so a corrupt transfer is a typed rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SloSnapshot {
    /// Node the snapshot belongs to.
    pub node: NodeId,
    /// Max commit-latency class observed (kernel-classified bucket).
    pub commit_latency_class_max: u8,
    /// Cumulative lease-denies.
    pub lease_deny_count: u64,
    /// Status-code histogram over the three pinned codes.
    pub lease_exhausted: u64,
    /// Cumulative 0x4347_0006 concurrent-writer-denied count.
    pub concurrent_writer_denied: u64,
    /// Cumulative 0x5032_38FF P28 spoliation count.
    pub p28_spoliation: u64,
}

/// 40 bytes: 16 id + 1 lat + 8*4 counters + 4 checksum.
pub const SNAPSHOT_LEN: usize = 16 + 1 + 8 * 4 + 4;

fn fnv1a(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c9dc5;
    for &b in bytes {
        h ^= b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    h
}

impl SloSnapshot {
    /// Fixed-width little-endian wire encoding + fnv1a checksum tail.
    pub fn to_bytes(&self) -> [u8; SNAPSHOT_LEN] {
        let mut out = [0u8; SNAPSHOT_LEN];
        out[..16].copy_from_slice(&self.node.0);
        out[16] = self.commit_latency_class_max;
        let mut w = 17usize;
        for v in [
            self.lease_deny_count,
            self.lease_exhausted,
            self.concurrent_writer_denied,
            self.p28_spoliation,
        ] {
            out[w..w + 8].copy_from_slice(&v.to_le_bytes());
            w += 8;
        }
        let c = fnv1a(&out[..SNAPSHOT_LEN - 4]);
        out[SNAPSHOT_LEN - 4..].copy_from_slice(&c.to_le_bytes());
        out
    }

    /// Decode; checksum mismatch is a typed None, never a silent zero.
    pub fn from_bytes(b: &[u8; SNAPSHOT_LEN]) -> Option<SloSnapshot> {
        let stored = u32::from_le_bytes(b[SNAPSHOT_LEN - 4..].try_into().ok()?);
        if stored != fnv1a(&b[..SNAPSHOT_LEN - 4]) {
            return None;
        }
        let mut node = [0u8; 16];
        node.copy_from_slice(&b[..16]);
        if b.len() < SNAPSHOT_LEN {
            return None;
        }
        let rd = |i: usize| -> Option<u64> {
            Some(u64::from_le_bytes(b[i..i + 8].try_into().ok()?))
        };
        Some(SloSnapshot {
            node: NodeId(node),
            commit_latency_class_max: b[16],
            lease_deny_count: rd(17)?,
            lease_exhausted: rd(25)?,
            concurrent_writer_denied: rd(33)?,
            p28_spoliation: rd(41)?,
        })
    }
}

/// Kernel-side stats feed trait (read-only). A real kernel exports the
/// A1 stats block through this seam; courts back it with a real
/// in-process source, not a mock framework.
pub trait SloSource {
    /// Current snapshot from the node's stats block.
    fn snapshot(&self) -> SloSnapshot;
    /// `Some` when the stats source itself is gone/stalled.
    fn is_stalled(&self) -> bool { false }
}

/// Typed alert emitted by predicate evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alert {
    /// Lease-deny rate crossed the ceiling.
    LeaseDenyStorm(NodeId),
    /// Status histogram crossed a typed ceiling.
    StatusStorm(NodeId, u32),
    /// Stats source stalled => typed UNHEALTHY (R-A1.4 analog).
    SourceUnhealthy(NodeId),
}

/// Typed alert predicate (R-A6.2). Thresholds are explicit values.
#[derive(Clone, Copy, Debug)]
pub struct AlertPredicate {
    /// Alert when lease_deny_count > this ceiling.
    pub lease_deny_ceiling: u64,
    /// Alert when any pinned status code exceeds this ceiling.
    pub status_code_ceiling: u64,
}

impl AlertPredicate {
    /// Evaluate one snapshot. Off-by-one is load-bearing: crossing is
    /// strictly-greater, a counter exactly at ceiling stays quiet.
    pub fn evaluate(&self, s: &SloSnapshot) -> Option<Alert> {
        if s.lease_deny_count > self.lease_deny_ceiling {
            return Some(Alert::LeaseDenyStorm(s.node));
        }
        for (count, code) in [
            (s.lease_exhausted, status_codes::LEASE_EXHAUSTED),
            (s.concurrent_writer_denied, status_codes::CONCURRENT_WRITER_DENIED),
            (s.p28_spoliation, status_codes::P28_SPOLIATION),
        ] {
            if count > self.status_code_ceiling {
                return Some(Alert::StatusStorm(s.node, code));
            }
        }
        None
    }
}

/// Bounded poll (R-A6.2 / no-sleep doctrine): at most `max_polls`
/// observations of a real source; returns the last snapshot seen. No
/// sleeps — each iteration observes state, and a stalled source is a
/// typed verdict, not a wait.
pub fn poll_until(
    src: &dyn SloSource,
    pred: &AlertPredicate,
    max_polls: u32,
) -> (SloSnapshot, Option<Alert>) {
    let mut last = src.snapshot();
    for _ in 1..max_polls {
        if src.is_stalled() {
            break;
        }
        last = src.snapshot();
    }
    let alert = if src.is_stalled() {
        Some(Alert::SourceUnhealthy(last.node))
    } else {
        pred.evaluate(&last)
    };
    (last, alert)
}
