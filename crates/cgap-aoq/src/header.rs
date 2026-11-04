// Hand-written module (irreducible residue): the canonical aoq-pack graph
// generates generated_aoq_header_layout.rs (field table) and the cgap-ffi
// accessors; header.rs carries the AoqHeader struct pinned to the landed
// layout by const asserts. A graph drift is a compile error here.

//! AOQ1 header struct: `#[repr(C)]`, 64 bytes, pinned field-for-field to
//! the landed ontology (`packs/aoq-pack/ontology.ttl`, R-1 ruling) via
//! compile-time offset asserts against the generated field table.

use core::cmp::Ordering;
use core::error::Error;
use core::fmt;

use crate::generated_aoq_header_layout as gen;

/// Wire magic constant, "AOQ1" (`aoq:hf-magic aoq:value "0x414F5131"`).
pub const MAGIC: u32 = gen::AOQ1_MAGIC;
/// Header version this codec speaks (`aoq:hf-version aoq:value 2`).
pub const VERSION: u16 = gen::AOQ1_VERSION;
/// Header size in bytes (`aoq:header-aq1 aoq:sizeBytes 64`).
pub const SIZE_BYTES: usize = gen::AOQ1_SIZE_BYTES;
/// FNV-1a32 coverage is half-open `[0x00, 0x3C)`; digest stored at `0x3C`.
pub const DIGEST_SPAN: (usize, usize) = (
    gen::AOQ1_DIGEST_SPAN_START,
    gen::AOQ1_DIGEST_SPAN_END,
);

/// Closed key-epoch enum (`aoq:Epoch`, u16 wire kind at header offset 6).
/// `RETIRED` (u16::MAX) is the reserved read-only sentinel.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Epoch {
    /// `aoq:Epoch Genesis` — initial key epoch (value 0).
    Genesis = 0,
    /// `aoq:Epoch Rotated1` — first key rotation (value 1).
    Rotated1 = 1,
    /// `aoq:Epoch Rotated2` — second key rotation (value 2).
    Rotated2 = 2,
    /// `aoq:Epoch Retired` — reserved read-only sentinel (value 65535);
    /// admits on decode, never lawful on a newly written frame.
    Retired = 65535,
}

/// Typed refusal for a wire epoch outside the closed `aoq:Epoch` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownEpoch(pub u16);

impl fmt::Display for UnknownEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown AOQ epoch {}", self.0)
    }
}

impl Error for UnknownEpoch {}

impl TryFrom<u16> for Epoch {
    type Error = UnknownEpoch;

    fn try_from(raw: u16) -> Result<Self, Self::Error> {
        match raw {
            0 => Ok(Epoch::Genesis),
            1 => Ok(Epoch::Rotated1),
            2 => Ok(Epoch::Rotated2),
            65535 => Ok(Epoch::Retired),
            other => Err(UnknownEpoch(other)),
        }
    }
}

impl From<Epoch> for u16 {
    fn from(e: Epoch) -> u16 {
        e as u16
    }
}

/// The AOQ1 header. `#[repr(C)]`, 64 bytes; every field offset is pinned
/// to the landed ontology field table by the compile-time court below —
/// a graph drift refuses the build, not the runtime.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AoqHeader {
    /// `aoq:hf-magic` — offset 0, u32, always `0x414F_5131` on the wire.
    pub magic: u32,
    /// `aoq:hf-version` — offset 4, u16, always 2.
    pub version: u16,
    /// `aoq:hf-epoch` — offset 6, u16, one of the closed `aoq:Epoch` values.
    pub epoch: u16,
    /// `aoq:hf-key-id` — offset 8, u64.
    pub key_id: u64,
    /// `aoq:hf-frame-ordinal` — offset 16, u32.
    pub frame_ordinal: u32,
    /// `aoq:hf-seq` — offset 20, u32.
    pub seq: u32,
    /// `aoq:hf-reserved` — offset 24, 36 bytes; zero on write,
    /// ignored-but-digest-covered on read (IPC1 zero-tail discipline).
    pub reserved: [u8; 36],
    /// `aoq:hf-header-digest` — offset 60, FNV-1a32 over `[0x00,0x3C)`.
    pub header_digest: u32,
}

/// Compile-time layout court: size/align/offset pins against the landed
/// ontology (via `generated_aoq_header_layout`). A drift is a compile
/// error — the graph is the spec.
const _: () = {
    use core::mem::{align_of, offset_of, size_of};
    assert!(size_of::<AoqHeader>() == gen::AOQ1_SIZE_BYTES);
    assert!(align_of::<AoqHeader>() == 8);
    assert!(offset_of!(AoqHeader, magic) == gen::AOQ1_MAGIC_OFFSET);
    assert!(offset_of!(AoqHeader, version) == gen::AOQ1_VERSION_OFFSET);
    assert!(offset_of!(AoqHeader, epoch) == gen::AOQ1_EPOCH_OFFSET);
    assert!(offset_of!(AoqHeader, key_id) == gen::AOQ1_KEY_ID_OFFSET);
    assert!(offset_of!(AoqHeader, frame_ordinal) == gen::AOQ1_FRAME_ORDINAL_OFFSET);
    assert!(offset_of!(AoqHeader, seq) == gen::AOQ1_SEQ_OFFSET);
    assert!(offset_of!(AoqHeader, reserved) == gen::AOQ1_RESERVED_OFFSET);
    assert!(offset_of!(AoqHeader, header_digest) == gen::AOQ1_HEADER_DIGEST_OFFSET);
};

impl fmt::Debug for AoqHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AoqHeader")
            .field("magic", &format_args!("{:#010x}", self.magic))
            .field("version", &self.version)
            .field("epoch", &self.epoch)
            .field("key_id", &self.key_id)
            .field("frame_ordinal", &self.frame_ordinal)
            .field("seq", &self.seq)
            .field("header_digest", &format_args!("{:#010x}", self.header_digest))
            .finish_non_exhaustive()
    }
}

impl AoqHeader {
    /// A canonical header: magic `0x414F_5131`, version 2, zeroed reserved
    /// region, digest zero until encode computes the FNV seal.
    pub const fn new(epoch: u16, key_id: u64, frame_ordinal: u32, seq: u32) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            epoch,
            key_id,
            frame_ordinal,
            seq,
            reserved: [0u8; 36],
            header_digest: 0,
        }
    }

    /// `true` when `epoch` is one of the closed `aoq:Epoch` wire values
    /// (0, 1, 2, 65535).
    pub const fn epoch_in_enum(epoch: u16) -> bool {
        matches!(epoch, 0 | 1 | 2 | 65535)
    }

    /// `true` when this header's epoch is the read-only `RETIRED`
    /// sentinel (u16::MAX, `aoq:epoch-retired`).
    pub const fn is_retired(&self) -> bool {
        self.epoch == 65535
    }

    /// Frame replay ordering: by frame_ordinal, then seq.
    pub fn frame_order(&self, other: &Self) -> Ordering {
        (self.frame_ordinal, self.seq).cmp(&(other.frame_ordinal, other.seq))
    }
}
