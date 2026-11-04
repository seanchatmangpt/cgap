//! Packfile V2 streaming demuxer (lane GIT-01, v26.11.2).
//!
//! Zero-alloc, no_std-compatible header-walk demuxer facet over the
//! existing pack registers. Composition statement (drift law):
//!
//! REUSED (no redefinition): the pack header law ("PACK" + u32 BE
//! version 2 + u32 BE object count) asserts the same byte law as
//! `pack_write::parse_pack_v2` (pack_write.rs:448-451); the object
//! varint header law (bit-7 continuation, bits 6..4 type, 4 low size
//! bits, then 7-bit LE continuation, first shift 4) mirrors the
//! private `pack_write::read_varint_header` (pack_write.rs:416-438);
//! the pack trailer is verified by the court through the existing
//! public `pack_write::sha1` — the demuxer only CAPTURES the 20
//! trailer bytes and never re-implements SHA-1.
//!
//! ADDED (no prior primitive exists): the OFS_DELTA negative-offset
//! varint (git's biased `((acc+1)<<7)|low7` decode; no encoder exists
//! in this crate because write_pack_v2 emits no deltas) and the
//! REF_DELTA 20-byte base capture; the streaming byte-at-a-time
//! state machine.
//!
//! Divergence, stated: `pack_write::read_varint_header` REFUSES type
//! 6/7 (`UnsupportedDeltaEncoding`, pack_write.rs:424-426) because the
//! writer produces no deltas; the demuxer DECODES 6/7 (its reason to
//! exist), so the private parser could not be reused for delta
//! records. The type 1..4 law is mirrored byte-for-byte. Body
//! framing: bodies are treated as stored frames of `declared_size`
//! bytes (header-walk facet; zlib extent resolution stays in
//! `parse_pack_v2`, which inflates).
//!
//! Reachability: lib.rs is contract-off-limits this wave, so the
//! module is courted via `#[path = "../src/pack_demux.rs"] mod
//! pack_demux;` in tests/pack_demux_court.rs (the census pattern
//! documented at pack.rs:11-16). Promotion is a one-line coordinator
//! `pub mod pack_demux;`.

// court-consumed surface (CEN2 ALLOW-WITH-REASON): pack_demux_court / pack_demux_poison_court / posh_wire_court / posh_alloc_sentry_court / posh_refusal_matrix_court / breach_regression_court / soc_full_chip_court; wire consumers pending — module-level allow (uniform surface, not scattered attrs).
#![allow(dead_code)]

use core::fmt;

/// Ring-window size the caller feeds per `feed` call.
pub const RING_WINDOW_BYTES: usize = 32 * 1024;

/// Deterministic feed stride (the "1,365-fact window"), a court-driving
/// parameter, not a wire constant. Provenance: HONEST NONE — no prior
/// constant exists in this crate (pack.rs / pack_write.rs carry none;
/// checked this session). Named rather than invented as lore. Rationale
/// for the value: 1,365 = 3 x 5 x 7 x 13, coprime with every structural
/// stride in the format (12-byte header, 20-byte trailer / ref base,
/// 64-byte SHA-1 block, 4-byte BE fields), so a fixed stride never
/// aliases one structural field across successive windows; any coprime
/// value would serve, and callers may drive other strides.
pub const FACT_WINDOW: usize = 1_365;

/// Records buffered between drains.
pub const RECORD_CAP: usize = 64;

/// Errors from the streaming demuxer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemuxError {
    /// Leading bytes are not "PACK": the stream has no packfile subject
    /// at its head (gate exit-code class 2: subject missing).
    SubjectMissing,
    /// Pack version is not 2.
    BadVersion(u32),
    /// Stream ended mid-structure (header, varint, delta base, declared
    /// body, or trailer).
    Truncated,
    /// Malformed object-header varint (reserved type 0/5, or size
    /// continuation exceeding the u64 field).
    BadVarint,
    /// Malformed or overflowing OFS_DELTA offset varint.
    BadDeltaOffset,
    /// More than RECORD_CAP records accumulated without a drain.
    RecordCapExceeded,
    /// Bytes arrived after the 20-byte trailer was captured (FIX-05 F1:
    /// post-trailer poison is refused, never silently absorbed). The
    /// payload is the cumulative trailing-byte count.
    TrailingBytes(u64),
    /// A record tried to close past the header-declared object count
    /// (FIX-05 F2: surplus-object header lie is a typed refusal).
    /// Invariant guard: the streaming state machine enters ObjHeader
    /// only while `objects_seen < declared_objects`, so this arm is
    /// unreachable through well-formed transitions — it exists so the
    /// surplus channel is refused AT THE RECORD CLOSURE SITE if the
    /// phase law ever drifts (defense against RED-09 F2).
    ObjectSurplus { seen: u32, declared: u32 },
}

impl fmt::Display for DemuxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DemuxError::SubjectMissing => {
                write!(f, "not a packfile: leading bytes are not \"PACK\"")
            }
            DemuxError::BadVersion(v) => write!(f, "pack version {v} is not 2"),
            DemuxError::Truncated => write!(f, "pack stream ended mid-structure"),
            DemuxError::BadVarint => write!(f, "malformed object-header varint"),
            DemuxError::BadDeltaOffset => write!(f, "malformed OFS_DELTA offset varint"),
            DemuxError::RecordCapExceeded => write!(
                f,
                "more than {RECORD_CAP} records without a drain_records call"
            ),
            DemuxError::TrailingBytes(n) => {
                write!(f, "{n} poison byte(s) after the pack trailer")
            }
            DemuxError::ObjectSurplus { seen, declared } => {
                write!(f, "surplus object: {seen} seen exceeds {declared} declared")
            }
        }
    }
}

/// Delta base reference carried by a delta record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaBase {
    None,
    /// OFS_DELTA: byte offset of the base object in the same pack.
    Ofs(u64),
    /// REF_DELTA: 20-byte OID of the base object.
    Ref([u8; 20]),
}

/// One demuxed object record: header-walk facts, no body bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemuxRecord {
    /// Byte offset of the object header start in the stream.
    pub offset: u64,
    /// Pack type code (1 commit, 2 tree, 3 blob, 4 tag, 6 OFS_DELTA,
    /// 7 REF_DELTA).
    pub obj_type: u8,
    /// Declared uncompressed size from the varint header.
    pub declared_size: u64,
    /// Delta base; DeltaBase::None for full objects.
    pub base: DeltaBase,
    /// Header length (varint + delta-base bytes) in bytes.
    pub header_len: u32,
}

const DEMUX_EMPTY_RECORD: DemuxRecord = DemuxRecord {
    offset: 0,
    obj_type: 0,
    declared_size: 0,
    base: DeltaBase::None,
    header_len: 0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Header,
    ObjHeader,
    OfsBase,
    RefBase,
    Body,
    Trailer,
    Done,
}

/// Streaming pack demuxer. Feed successive windows; drain records.
pub struct Demux {
    phase: Phase,
    hdr: [u8; 12],
    hdr_filled: usize,
    declared_objects: u32,
    objects_seen: u32,
    // current-record accumulator
    cur_type: u8,
    cur_size: u64,
    cur_base: DeltaBase,
    cur_hdr_start: u64,
    cur_hdr_end: u64,
    // varint accumulator
    varint_shift: u32,
    varint_size: u64,
    varint_started: bool,
    // OFS_DELTA biased varint accumulator
    ofs_acc: u64,
    ofs_bytes: u32,
    // REF_DELTA base capture
    ref_buf: [u8; 20],
    ref_filled: usize,
    // body skip
    size_remaining: u64,
    // trailer capture
    trailer: [u8; 20],
    trailer_filled: usize,
    // record table
    records: [DemuxRecord; RECORD_CAP],
    n_records: usize,
    bytes_fed: u64,
    /// FIX-05 F1: cumulative post-trailer poison bytes refused.
    trailing_bytes: u64,
}

impl Default for Demux {
    fn default() -> Self {
        Self::new()
    }
}

impl Demux {
    pub fn new() -> Self {
        Demux {
            phase: Phase::Header,
            hdr: [0; 12],
            hdr_filled: 0,
            declared_objects: 0,
            objects_seen: 0,
            cur_type: 0,
            cur_size: 0,
            cur_base: DeltaBase::None,
            cur_hdr_start: 0,
            cur_hdr_end: 0,
            varint_shift: 0,
            varint_size: 0,
            varint_started: false,
            ofs_acc: 0,
            ofs_bytes: 0,
            ref_buf: [0; 20],
            ref_filled: 0,
            size_remaining: 0,
            trailer: [0; 20],
            trailer_filled: 0,
            records: [DEMUX_EMPTY_RECORD; RECORD_CAP],
            n_records: 0,
            bytes_fed: 0,
            trailing_bytes: 0,
        }
    }

    /// True once the 20-byte trailer has been captured.
    pub fn is_done(&self) -> bool {
        self.phase == Phase::Done
    }

    /// Typed completion check: Err(Truncated) unless the trailer was
    /// captured. Call after the final feed.
    pub fn finish(&self) -> Result<(), DemuxError> {
        if self.phase == Phase::Done {
            Ok(())
        } else {
            Err(DemuxError::Truncated)
        }
    }

    /// Objects completed so far.
    pub fn objects_seen(&self) -> u32 {
        self.objects_seen
    }

    /// Object count declared by the pack header, once the header landed.
    pub fn declared_objects(&self) -> Option<u32> {
        if self.hdr_filled == 12 {
            Some(self.declared_objects)
        } else {
            None
        }
    }

    /// Records awaiting drain.
    pub fn records_pending(&self) -> usize {
        self.n_records
    }

    /// Bytes consumed across all feeds.
    pub fn bytes_fed(&self) -> u64 {
        self.bytes_fed
    }

    /// FIX-05 F1: cumulative post-trailer poison bytes refused so far
    /// (nonzero only after a TrailingBytes error fired).
    pub fn trailing_poison(&self) -> u64 {
        self.trailing_bytes
    }

    /// Trailer captured at Done (pack checksum bytes, unverified —
    /// verification reuses `pack_write::sha1` at the court layer).
    pub fn captured_trailer(&self) -> Option<[u8; 20]> {
        if self.phase == Phase::Done {
            Some(self.trailer)
        } else {
            None
        }
    }

    /// Drain pending records into `out`; returns the count drained.
    /// Records beyond `out.len()` stay buffered.
    pub fn drain_records(&mut self, out: &mut [DemuxRecord]) -> usize {
        let k = core::cmp::min(out.len(), self.n_records);
        out[..k].copy_from_slice(&self.records[..k]);
        let rem = self.n_records - k;
        if rem > 0 {
            self.records.copy_within(k..self.n_records, 0);
        }
        self.n_records -= k;
        k
    }

    /// Feed one caller-provided window (typical size
    /// RING_WINDOW_BYTES; any slice accepted). Zero heap allocation:
    /// all state lives in the struct; parsing is byte-at-a-time over
    /// `window`.
    pub fn feed(&mut self, mut window: &[u8]) -> Result<(), DemuxError> {
        while let Some((&b, rest)) = window.split_first() {
            window = rest;
            self.bytes_fed += 1;
            match self.phase {
                Phase::Header => {
                    self.hdr[self.hdr_filled] = b;
                    self.hdr_filled += 1;
                    if self.hdr_filled == 12 {
                        if &self.hdr[0..4] != b"PACK" {
                            return Err(DemuxError::SubjectMissing);
                        }
                        let ver = u32::from_be_bytes(self.hdr[4..8].try_into().unwrap());
                        if ver != 2 {
                            return Err(DemuxError::BadVersion(ver));
                        }
                        self.declared_objects =
                            u32::from_be_bytes(self.hdr[8..12].try_into().unwrap());
                        self.cur_hdr_start = self.bytes_fed;
                        self.phase = if self.declared_objects == 0 {
                            Phase::Trailer
                        } else {
                            Phase::ObjHeader
                        };
                    }
                }
                Phase::ObjHeader => self.step_obj_header(b)?,
                Phase::OfsBase => {
                    if self.ofs_bytes == 0 {
                        // first byte sets the accumulator without bias
                        self.ofs_acc = (b & 0x7F) as u64;
                    } else {
                        self.ofs_acc = self
                            .ofs_acc
                            .checked_add(1)
                            .and_then(|a| a.checked_shl(7))
                            .ok_or(DemuxError::BadDeltaOffset)?;
                        self.ofs_acc |= (b & 0x7F) as u64;
                    }
                    self.ofs_bytes += 1;
                    if self.ofs_bytes > 10 {
                        return Err(DemuxError::BadDeltaOffset);
                    }
                    if b & 0x80 == 0 {
                        self.cur_base = DeltaBase::Ofs(self.ofs_acc);
                        self.enter_body()?;
                    }
                }
                Phase::RefBase => {
                    self.ref_buf[self.ref_filled] = b;
                    self.ref_filled += 1;
                    if self.ref_filled == 20 {
                        self.cur_base = DeltaBase::Ref(self.ref_buf);
                        self.enter_body()?;
                    }
                }
                Phase::Body => {
                    self.size_remaining -= 1;
                    if self.size_remaining == 0 {
                        self.close_record()?;
                    }
                }
                Phase::Trailer => {
                    self.trailer[self.trailer_filled] = b;
                    self.trailer_filled += 1;
                    if self.trailer_filled == 20 {
                        self.phase = Phase::Done;
                    }
                }
                Phase::Done => {
                    // FIX-05 F1: post-trailer poison is a typed refusal,
                    // never silent absorption (RED-09 F1 witness: feed=Ok,
                    // done=true, bytes_fed 34->38 on a 4-byte poison tail).
                    self.trailing_bytes += 1;
                }
            }
        }
        if self.trailing_bytes > 0 {
            return Err(DemuxError::TrailingBytes(self.trailing_bytes));
        }
        Ok(())
    }

    /// Parse one byte of the object-header varint (mirror of the
    /// pack_write::read_varint_header byte law, with delta types
    /// decoded instead of refused).
    fn step_obj_header(&mut self, b: u8) -> Result<(), DemuxError> {
        if !self.varint_started {
            let t = (b >> 4) & 0x7;
            if t == 0 || t == 5 {
                return Err(DemuxError::BadVarint);
            }
            self.cur_type = t;
            self.varint_size = (b & 0x0F) as u64;
            self.varint_shift = 4;
            self.varint_started = true;
            if b & 0x80 == 0 {
                self.varint_started = false;
                self.finish_obj_header()?;
            }
            return Ok(());
        }
        self.varint_size |= ((b & 0x7F) as u64)
            .checked_shl(self.varint_shift)
            .ok_or(DemuxError::BadVarint)?;
        if self.varint_shift > 60 {
            return Err(DemuxError::BadVarint);
        }
        self.varint_shift += 7;
        if b & 0x80 == 0 {
            self.varint_started = false;
            self.finish_obj_header()?;
        }
        Ok(())
    }

    fn finish_obj_header(&mut self) -> Result<(), DemuxError> {
        self.cur_size = self.varint_size;
        self.cur_base = DeltaBase::None;
        self.ofs_acc = 0;
        self.ofs_bytes = 0;
        self.ref_filled = 0;
        if self.cur_type == 6 {
            self.phase = Phase::OfsBase;
        } else if self.cur_type == 7 {
            self.phase = Phase::RefBase;
        } else {
            self.enter_body()?;
        }
        Ok(())
    }

    fn enter_body(&mut self) -> Result<(), DemuxError> {
        self.cur_hdr_end = self.bytes_fed;
        self.size_remaining = self.cur_size;
        if self.cur_size == 0 {
            self.close_record()?;
        } else {
            self.phase = Phase::Body;
        }
        Ok(())
    }

    /// Close the current record into the fixed table.
    fn close_record(&mut self) -> Result<(), DemuxError> {
        if self.n_records == RECORD_CAP {
            return Err(DemuxError::RecordCapExceeded);
        }
        // FIX-05 F2 (RED-09): surplus-object header lie refused at the
        // record-closure site. Guard is placed BEFORE the increment so
        // the legitimate final record (objects_seen == declared - 1)
        // closes normally.
        if self.objects_seen >= self.declared_objects {
            return Err(DemuxError::ObjectSurplus {
                seen: self.objects_seen,
                declared: self.declared_objects,
            });
        }
        self.records[self.n_records] = DemuxRecord {
            offset: self.cur_hdr_start,
            obj_type: self.cur_type,
            declared_size: self.cur_size,
            base: self.cur_base,
            header_len: (self.cur_hdr_end - self.cur_hdr_start) as u32,
        };
        self.n_records += 1;
        self.objects_seen += 1;
        self.cur_hdr_start = self.bytes_fed;
        self.phase = if self.objects_seen == self.declared_objects {
            Phase::Trailer
        } else {
            Phase::ObjHeader
        };
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// FIX-05 in-module courts (cfg(test)): the ObjectSurplus guard in
// close_record is unreachable through the public streaming API (the
// phase law only enters ObjHeader while objects_seen < declared), so
// the guard's runtime flip is proven here by forcing the surplus state
// through the private fields — real type, no mocks.
// ---------------------------------------------------------------------------
#[cfg(all(test, feature = "std"))]
mod fix05_courts {
    use super::*;

    /// Q1 twin baseline: a record close under the normal law (seen <
    /// declared) succeeds and pins the phase transition.
    #[test]
    fn q1_normal_close_succeeds_below_declared() {
        let mut d = Demux::new();
        d.declared_objects = 1;
        d.phase = Phase::ObjHeader;
        d.cur_hdr_start = 1;
        d.bytes_fed = 1;
        // 0x34: type 3, size 4; then 4 body bytes -> close_record
        let ok = d.feed(&[0x34, b'a', b'a', b'a', b'a']);
        assert_eq!(ok, Ok(()), "VACUOUS twin: normal close must succeed");
        assert_eq!(d.objects_seen(), 1);
        assert_eq!(
            d.phase,
            Phase::Trailer,
            "seen==declared must transition to Trailer"
        );
    }

    /// Q2 F2 MUTATION COURT: with declared_objects already consumed, a
    /// record close MUST refuse typed ObjectSurplus (the surplus guard).
    /// ANTI-VACUITY MUTATION (m3): delete the close_record surplus guard
    /// (run_fix05_mutation.sh); the close then succeeds and this court
    /// flips to failure.
    #[test]
    fn q2_surplus_close_refuses_typed() {
        let mut d = Demux::new();
        d.declared_objects = 0; // everything is surplus
        d.phase = Phase::ObjHeader;
        d.cur_hdr_start = 1;
        d.bytes_fed = 1;
        let err = d
            .feed(&[0x34, b'a', b'a', b'a', b'a'])
            .expect_err("VACUOUS: surplus close admitted");
        assert_eq!(
            err,
            DemuxError::ObjectSurplus {
                seen: 0,
                declared: 0
            }
        );
    }
}
