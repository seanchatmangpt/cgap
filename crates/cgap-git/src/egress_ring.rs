//! Lane GIT-07: 32,768-byte egress ring serving derived fixpoint state as
//! synthetic Arrow IPC / Parquet page blobs.
//!
//! Framing-constant reuse (stated per dispatch; cgap-git is a zero-dep
//! kernel, so shared constants are RESTATED here with source pins, not
//! imported — same law as ingress_ring.rs):
//! - Arrow IPC magic "IPC1" = 0x4950_4331: REUSED verbatim from
//!   `crates/cgap-arrow/src/ipc.rs:40` (`ipc::MAGIC`), along with the
//!   8-byte frame prefix (`ipc.rs:46 PREFIX_SIZE`), the 64-byte metadata
//!   header block (`ipc.rs:44 META_HEADER_SIZE`), and layout version 1
//!   (`ipc.rs:42 LAYOUT_VERSION`). Page shape: 8B prefix (magic + payload
//!   length), 64B meta header, `ncols` x 8B column-descriptor entries,
//!   then the raw row payload.
//! - Parquet page magic `b"PAR2"`: REUSED from
//!   `crates/cgap-parquet/src/footer.rs:248` (`footer::MAGIC`) as the page
//!   tag on the Parquet-flavored serve path. The page body is SYNTHETIC
//!   (magic + length + already-wire-serialized payload) — this is NOT a
//!   conformant Parquet file and makes no such claim; no new Parquet
//!   format is invented, only the container magic constant is reused.
//! - Index discipline and the Acquire/Release publish law are reused from
//!   `crates/cgap-git/src/ingress_ring.rs` (same free-running counters,
//!   same fail-closed overflow refusal, same andon exit class 101).
//!
//! The courted invariant (dispatch): serving a read window performs ZERO
//! heap allocations and ZERO re-serialization. Zero-copy is structural:
//! `EgressRing::window` returns `&[u8]` slices directly into the fixed
//! inline buffer (two segments at the wrap boundary); `parse_page` returns
//! a `Page<'a>` borrowing the input bytes. Zero re-serialization is
//! structural: records are pushed in final wire layout and the serve path
//! emits a fixed 72-byte header (memcpy into the caller buffer) followed
//! by one contiguous memcpy of stored payload bytes — no per-field
//! encoding at serve time. Zero-alloc is PROVEN, not asserted: the court
//! runs a tracking `GlobalAlloc` instrument and asserts a delta of zero
//! across the serve path (and proves the instrument itself fires via an
//! allocating control). `#![forbid(unsafe_code)]`.

#![forbid(unsafe_code)]

use core::sync::atomic::{AtomicUsize, Ordering};
/// Standard egress ring capacity (dispatch): 32,768 bytes.
pub const EGRESS_RING_BYTES: usize = 32_768;

/// Arrow IPC magic "IPC1" — reuse pin: `crates/cgap-arrow/src/ipc.rs:40`.
pub const IPC_MAGIC: u32 = 0x4950_4331;
/// 8-byte frame prefix — reuse pin: `ipc.rs:46 PREFIX_SIZE`.
pub const IPC_PREFIX_SIZE: usize = 8;
/// 64-byte metadata header block — reuse pin: `ipc.rs:44 META_HEADER_SIZE`.
pub const IPC_META_SIZE: usize = 64;
/// Layout version — reuse pin: `ipc.rs:42 LAYOUT_VERSION`.
pub const IPC_LAYOUT_VERSION: u16 = 1;
/// Parquet page magic — reuse pin: `crates/cgap-parquet/src/footer.rs:248`.
pub const PARQUET_MAGIC: [u8; 4] = *b"PAR2";

/// Maximum columns in a page schema descriptor (fixed, no heap).
pub const MAX_COLS: usize = 8;
/// Absolute page offset where descriptor entries begin (prefix + meta).
pub const DESC_TABLE_OFFSET: usize = PAGE_FIXED_HEADER;
/// Serialized size of one 8-byte column-descriptor entry.
pub const DESC_ENTRY_BYTES: usize = 8;
/// Fixed page-header size: prefix + meta (72 bytes) before descriptors.
pub const PAGE_FIXED_HEADER: usize = IPC_PREFIX_SIZE + IPC_META_SIZE;

/// One column of the synthetic schema descriptor: fixed 8-byte wire entry
/// `{ u32 name_hash | u16 offset | u16 width }` (little-endian).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnDesc {
    /// Stable hash of the column name (producer-assigned).
    pub name_hash: u32,
    /// Byte offset of the column inside a record (must be `width`-aligned).
    pub offset: u16,
    /// Column width in bytes (must be nonzero).
    pub width: u16,
}

/// Typed egress refusals. No variant is ever a silent no-op (RingSink law).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EgressError {
    /// Push refused: advancing head would overwrite unserved bytes.
    /// Ring state is untouched on refusal (fail-closed).
    Overflow {
        head: usize,
        tail: usize,
        record_len: usize,
    },
    /// Record longer than the record stride.
    RecordTooLarge { record_len: usize },
    /// Record shorter than the record stride.
    RecordTooSmall { record_len: usize },
    /// Requested read window extends past pending rows.
    RangeExhausted { requested: usize, available: usize },
    /// Page bytes do not start with the IPC magic.
    BadMagic { got: u32 },
    /// Layout version mismatch.
    BadVersion { got: u16 },
    /// Column count outside `1..=MAX_COLS`.
    TooManyColumns { got: usize },
    /// Column width is zero (degenerate descriptor).
    ZeroWidth { index: usize },
    /// MISALIGNED column offset in the schema descriptor: `offset` is not
    /// a multiple of `width`. Read validation fails CLOSED here (typed).
    /// This is the dispatch mutation-court target.
    MisalignedColumn {
        index: usize,
        offset: u16,
        width: u16,
    },
    /// Column offset + width runs past the record stride.
    ColumnOverrunsStride {
        index: usize,
        offset: u16,
        width: u16,
        stride: u32,
    },
    /// Page too short to hold the fixed header / declared descriptors.
    PageTooSmall { need: usize, got: usize },
    /// Declared payload length exceeds the bytes actually present.
    PayloadTruncated { declared: usize, available: usize },
    /// Caller output buffer too small for the page being served.
    OutBufTooSmall { need: usize, got: usize },
}

impl EgressError {
    /// Andon refusal class: all egress refusals are class 101 (typed,
    /// caught by the caller as `Err` — never a silent drop).
    pub fn andon_exit_class(&self) -> u32 {
        101
    }
}

/// Capacity guard as a pure named function so the mutation court can pin
/// exactly the guard whose deletion the flip twin targets.
#[inline(always)]
const fn would_overflow(head: usize, tail: usize, record_len: usize, cap: usize) -> bool {
    head.wrapping_sub(tail) + record_len > cap
}

/// Schema validation: the fail-closed gate shared by encode, serve, and
/// parse. A descriptor whose offset is not `width`-aligned is rejected
/// with [`EgressError::MisalignedColumn`] (the dispatch mutation target).
pub fn validate_schema(cols: &[ColumnDesc], stride: u32) -> Result<(), EgressError> {
    if cols.is_empty() || cols.len() > MAX_COLS {
        return Err(EgressError::TooManyColumns { got: cols.len() });
    }
    for (i, c) in cols.iter().enumerate() {
        if c.width == 0 {
            return Err(EgressError::ZeroWidth { index: i });
        }
        if (c.offset % c.width) != 0 {
            return Err(EgressError::MisalignedColumn {
                index: i,
                offset: c.offset,
                width: c.width,
            });
        }
        if (c.offset as u32) + (c.width as u32) > stride {
            return Err(EgressError::ColumnOverrunsStride {
                index: i,
                offset: c.offset,
                width: c.width,
                stride,
            });
        }
    }
    Ok(())
}

/// Meta header wire layout (inside the 64-byte IPC-shaped block):
/// `+0x00 u32 magic | +0x04 u16 version | +0x06 u16 ncols |
///  +0x08 u32 stride | +0x0C u32 nrows | +0x10..0x40 zero pad`.
const fn meta_len(ncols: usize) -> usize {
    PAGE_FIXED_HEADER + ncols * DESC_ENTRY_BYTES
}

/// Total page length for a schema of `ncols` columns and `nrows` rows of
/// `stride`-byte records.
pub const fn page_len(ncols: usize, nrows: usize, stride: usize) -> usize {
    PAGE_FIXED_HEADER + ncols * DESC_ENTRY_BYTES + nrows * stride
}

/// Encode the fixed page header (prefix + meta + descriptors) into `out`.
/// Returns the header length. Validates the schema first (fail-closed at
/// encode, not just at parse).
pub fn encode_page_header(
    cols: &[ColumnDesc],
    stride: u32,
    nrows: u32,
    out: &mut [u8],
) -> Result<usize, EgressError> {
    validate_schema(cols, stride)?;
    let need = meta_len(cols.len());
    if out.len() < need {
        return Err(EgressError::OutBufTooSmall {
            need,
            got: out.len(),
        });
    }
    let payload_len = (nrows as usize) * stride_len(cols, stride);
    out[0x00..0x04].copy_from_slice(&IPC_MAGIC.to_le_bytes());
    out[0x04..0x08].copy_from_slice(&(payload_len as u32).to_le_bytes());
    out[IPC_PREFIX_SIZE..IPC_PREFIX_SIZE + 0x04].copy_from_slice(&IPC_MAGIC.to_le_bytes());
    out[IPC_PREFIX_SIZE + 0x04..IPC_PREFIX_SIZE + 0x06]
        .copy_from_slice(&IPC_LAYOUT_VERSION.to_le_bytes());
    out[IPC_PREFIX_SIZE + 0x06..IPC_PREFIX_SIZE + 0x08]
        .copy_from_slice(&(cols.len() as u16).to_le_bytes());
    out[IPC_PREFIX_SIZE + 0x08..IPC_PREFIX_SIZE + 0x0C].copy_from_slice(&stride.to_le_bytes());
    out[IPC_PREFIX_SIZE + 0x0C..IPC_PREFIX_SIZE + 0x10].copy_from_slice(&nrows.to_le_bytes());
    out[IPC_PREFIX_SIZE + 0x10..PAGE_FIXED_HEADER].fill(0);
    for (i, c) in cols.iter().enumerate() {
        let b = PAGE_FIXED_HEADER + i * DESC_ENTRY_BYTES;
        out[b..b + 4].copy_from_slice(&c.name_hash.to_le_bytes());
        out[b + 4..b + 6].copy_from_slice(&c.offset.to_le_bytes());
        out[b + 6..b + 8].copy_from_slice(&c.width.to_le_bytes());
    }
    Ok(need)
}

/// Stride resolution: a stride of 0 means "derive from the schema" (the
/// sum of column widths; descriptors are layout-complete). Nonzero stride
/// is honored as given.
const fn stride_len(cols: &[ColumnDesc], stride: u32) -> usize {
    if stride != 0 {
        return stride as usize;
    }
    let mut s = 0usize;
    let mut i = 0usize;
    while i < cols.len() {
        s += cols[i].width as usize;
        i += 1;
    }
    s
}

/// A parsed page: descriptors copied into a fixed array (no heap) and a
/// ZERO-COPY payload slice borrowing the input bytes. Payload bytes are
/// never copied or re-encoded by the parser.
#[derive(Debug)]
pub struct Page<'a> {
    /// Validated schema descriptor (copied, `ncols` entries used).
    pub schema: [ColumnDesc; MAX_COLS],
    /// Number of columns in `schema`.
    pub ncols: usize,
    /// Record stride declared in the meta header.
    pub stride: u32,
    /// Row count declared in the meta header.
    pub nrows: u32,
    /// Zero-copy borrow of the page payload.
    pub payload: &'a [u8],
}

/// Parse and validate a synthetic IPC page. Zero-copy: the returned
/// [`Page`] borrows `bytes`; only the fixed descriptor array is copied.
/// Validation fails CLOSED (typed `Err`) on any framing or schema fault —
/// including a misaligned column offset in the schema descriptor.
pub fn parse_page(bytes: &[u8]) -> Result<Page<'_>, EgressError> {
    if bytes.len() < PAGE_FIXED_HEADER {
        return Err(EgressError::PageTooSmall {
            need: PAGE_FIXED_HEADER,
            got: bytes.len(),
        });
    }
    let rd32 = |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let rd16 = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
    if rd32(0x00) != IPC_MAGIC {
        return Err(EgressError::BadMagic { got: rd32(0x00) });
    }
    if rd32(IPC_PREFIX_SIZE) != IPC_MAGIC {
        return Err(EgressError::BadMagic {
            got: rd32(IPC_PREFIX_SIZE),
        });
    }
    let version = rd16(IPC_PREFIX_SIZE + 0x04);
    if version != IPC_LAYOUT_VERSION {
        return Err(EgressError::BadVersion { got: version });
    }
    let ncols = rd16(IPC_PREFIX_SIZE + 0x06) as usize;
    if ncols == 0 || ncols > MAX_COLS {
        return Err(EgressError::TooManyColumns { got: ncols });
    }
    let stride = rd32(IPC_PREFIX_SIZE + 0x08);
    let nrows = rd32(IPC_PREFIX_SIZE + 0x0C);
    let need = meta_len(ncols);
    if bytes.len() < need {
        return Err(EgressError::PageTooSmall {
            need,
            got: bytes.len(),
        });
    }
    let mut schema = [ColumnDesc {
        name_hash: 0,
        offset: 0,
        width: 0,
    }; MAX_COLS];
    for i in 0..ncols {
        let b = PAGE_FIXED_HEADER + i * DESC_ENTRY_BYTES;
        schema[i] = ColumnDesc {
            name_hash: rd32(b),
            offset: rd16(b + 4),
            width: rd16(b + 6),
        };
    }
    validate_schema(&schema[..ncols], stride)?;
    let declared = rd32(0x04) as usize;
    let available = bytes.len() - need;
    if declared > available {
        return Err(EgressError::PayloadTruncated {
            declared,
            available,
        });
    }
    Ok(Page {
        schema,
        ncols,
        stride,
        nrows,
        payload: &bytes[need..need + declared],
    })
}

/// SPSC circular byte ring over an inline fixed buffer serving
/// fixed-stride records as synthetic page blobs. One handle; `&mut self`
/// methods enforce single-accessor. Free-running `head` (producer) and
/// `tail` (serve/consume) counters (ingress_ring reuse); live bytes are
/// `head.wrapping_sub(tail)` in `[0, CAP]`, and CAP must be a whole
/// number of records (const assert below).
pub struct EgressRing<const CAP: usize, const STRIDE: usize> {
    buf: [u8; CAP],
    head: AtomicUsize,
    tail: AtomicUsize,
}

const _: () = assert!(EGRESS_RING_BYTES % 32 == 0);

impl<const CAP: usize, const STRIDE: usize> EgressRing<CAP, STRIDE> {
    /// Fresh empty ring. `const` so it can live in statics. The CAP %
    /// STRIDE whole-record assert runs on every construction (fail-fast,
    /// compile-time-evaluated in const/static contexts).
    pub const fn new() -> Self {
        assert!(CAP % STRIDE == 0, "ring capacity must be whole records");
        EgressRing {
            buf: [0u8; CAP],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Free-running producer counter.
    pub fn head(&self) -> usize {
        self.head.load(Ordering::Acquire)
    }

    /// Free-running serve/consume counter.
    pub fn tail(&self) -> usize {
        self.tail.load(Ordering::Acquire)
    }

    /// Pending bytes (head - tail distance), in `[0, CAP]`.
    pub fn pending_bytes(&self) -> usize {
        self.head
            .load(Ordering::Acquire)
            .wrapping_sub(self.tail.load(Ordering::Acquire))
    }

    /// Pending whole records (`STRIDE`-strided) available to serve.
    pub fn pending_rows(&self) -> usize {
        self.pending_bytes() / STRIDE
    }

    /// Ring capacity in bytes.
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Ring capacity in whole records.
    pub fn capacity_rows(&self) -> usize {
        CAP / STRIDE
    }

    /// Push one `STRIDE`-byte record (already in final wire layout).
    /// Overflow refusal leaves the ring untouched (fail-closed).
    pub fn push_record(&mut self, record: &[u8]) -> Result<(), EgressError> {
        if record.len() != STRIDE {
            if record.len() > STRIDE {
                return Err(EgressError::RecordTooLarge {
                    record_len: record.len(),
                });
            }
            return Err(EgressError::RecordTooSmall {
                record_len: record.len(),
            });
        }
        let head = self.head();
        let tail = self.tail();
        if would_overflow(head, tail, STRIDE, CAP) {
            return Err(EgressError::Overflow {
                head,
                tail,
                record_len: STRIDE,
            });
        }
        let pos = head % CAP;
        let first = core::cmp::min(STRIDE, CAP - pos);
        self.buf[pos..pos + first].copy_from_slice(&record[..first]);
        if STRIDE > first {
            self.buf[..STRIDE - first].copy_from_slice(&record[first..]);
        }
        // Publish law (ingress_ring / dma_ring reuse): bytes land before
        // the Release store; observers load with Acquire.
        self.head
            .store(head.wrapping_add(STRIDE), Ordering::Release);
        Ok(())
    }

    /// Advance the serve tail by `rows` records without reading (typed
    /// refusal if more rows are requested than are pending; the tail is
    /// NOT advanced on refusal — fail-closed).
    pub fn advance_tail(&mut self, rows: usize) -> Result<(), EgressError> {
        let bytes = rows
            .checked_mul(STRIDE)
            .ok_or(EgressError::RangeExhausted {
                requested: usize::MAX,
                available: self.pending_rows(),
            })?;
        if bytes > self.pending_bytes() {
            return Err(EgressError::RangeExhausted {
                requested: rows,
                available: self.pending_rows(),
            });
        }
        let tail = self.tail();
        self.tail.store(tail.wrapping_add(bytes), Ordering::Release);
        Ok(())
    }
}

/// A zero-copy read window: up to two `&[u8]` segments of the ring's
/// inline buffer (one when the window is contiguous, two across the wrap
/// boundary). No intermediate buffer, no allocation, no copy.
#[derive(Debug)]
pub struct Window<'a> {
    /// First (or only) contiguous segment.
    pub a: &'a [u8],
    /// Second segment when the window wraps; empty slice otherwise.
    pub b: &'a [u8],
}

impl<'a> Window<'a> {
    /// Total bytes across both segments.
    pub fn total_len(&self) -> usize {
        self.a.len() + self.b.len()
    }

    /// Gather the segments into a caller-owned buffer (single memcpy per
    /// segment; caller buffer, no heap). Used by the serve paths.
    pub fn copy_into(&self, out: &mut [u8]) {
        out[..self.a.len()].copy_from_slice(self.a);
        out[self.a.len()..self.total_len()].copy_from_slice(self.b);
    }
}

impl<const CAP: usize, const STRIDE: usize> EgressRing<CAP, STRIDE> {
    /// Zero-copy read window of `nrows` whole records starting at record
    /// `row` (0 = oldest pending). Returns slices DIRECTLY into the ring's
    /// inline buffer — the courted invariant (zero alloc, zero copy,
    /// zero re-serialization) is structural here.
    pub fn window(&self, row: usize, nrows: usize) -> Result<Window<'_>, EgressError> {
        let available = self.pending_rows();
        if nrows == 0 || row + nrows > available {
            return Err(EgressError::RangeExhausted {
                requested: row + nrows,
                available,
            });
        }
        let pos = (self.tail() + row * STRIDE) % CAP;
        let nbytes = nrows * STRIDE;
        let first = core::cmp::min(nbytes, CAP - pos);
        if nbytes <= first {
            return Ok(Window {
                a: &self.buf[pos..pos + nbytes],
                b: &[],
            });
        }
        Ok(Window {
            a: &self.buf[pos..pos + first],
            b: &self.buf[..nbytes - first],
        })
    }

    /// Serve ALL pending rows as one synthetic Arrow IPC page into `out`.
    /// The header is a fixed memcpy encode of the ring's schema; the
    /// payload is one gather of stored (already wire-serialized) bytes.
    /// NO re-serialization of any field and NO heap allocation on this
    /// path. Fails closed on a misaligned schema descriptor.
    pub fn serve_ipc_page(
        &self,
        cols: &[ColumnDesc],
        out: &mut [u8],
    ) -> Result<usize, EgressError> {
        let nrows = self.pending_rows();
        validate_schema(cols, STRIDE as u32)?;
        let need = page_len(cols.len(), nrows, STRIDE);
        if out.len() < need {
            return Err(EgressError::OutBufTooSmall {
                need,
                got: out.len(),
            });
        }
        let hlen = encode_page_header(cols, STRIDE as u32, nrows as u32, out)?;
        let win = self.window(0, nrows)?;
        win.copy_into(&mut out[hlen..hlen + win.total_len()]);
        Ok(hlen + win.total_len())
    }

    /// Serve ALL pending rows as one synthetic Parquet-flavored page:
    /// reused `PAR2` magic + u32 payload length + the stored payload
    /// bytes (single gather, no re-serialization, no heap). SYNTHETIC
    /// page body — reuses the footer.rs container magic only; this is
    /// not a conformant Parquet file and claims to be nothing more.
    pub fn serve_parquet_page(&self, out: &mut [u8]) -> Result<usize, EgressError> {
        let nrows = self.pending_rows();
        let need = PAGE_FIXED_HEADER + nrows * STRIDE;
        if out.len() < need {
            return Err(EgressError::OutBufTooSmall {
                need,
                got: out.len(),
            });
        }
        out[0..4].copy_from_slice(&PARQUET_MAGIC);
        out[4..8].copy_from_slice(&((nrows * (STRIDE as usize)) as u32).to_le_bytes());
        out[8..PAGE_FIXED_HEADER].fill(0);
        let win = self.window(0, nrows)?;
        win.copy_into(&mut out[PAGE_FIXED_HEADER..PAGE_FIXED_HEADER + win.total_len()]);
        Ok(PAGE_FIXED_HEADER + win.total_len())
    }
}

impl<const CAP: usize, const STRIDE: usize> Default for EgressRing<CAP, STRIDE> {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard instantiation: 32,768-byte ring of 32-byte records (1024 rows
/// capacity). 32-byte stride: four 8-byte columns, wire layout complete.
pub type StdEgressRing = EgressRing<EGRESS_RING_BYTES, 32>;

impl StdEgressRing {
    /// Fixed schema of the standard instantiation: four 8-byte columns at
    /// offsets 0/8/16/24 (width-aligned by construction; validated at
    /// serve and parse, never trusted).
    pub const SCHEMA: [ColumnDesc; 4] = [
        ColumnDesc {
            name_hash: 0x1A2B_3C4D,
            offset: 0,
            width: 8,
        },
        ColumnDesc {
            name_hash: 0x5E6F_7081,
            offset: 8,
            width: 8,
        },
        ColumnDesc {
            name_hash: 0x92A3_B4C5,
            offset: 16,
            width: 8,
        },
        ColumnDesc {
            name_hash: 0xD6E7_F809,
            offset: 24,
            width: 8,
        },
    ];

    /// Allocating snapshot control (instrument non-vacuity twin): the
    /// ONLY heap-using path in this module. `alloc` resolves against the
    /// crate-root `extern crate alloc;` (lib.rs) or the test binary's
    /// own `extern crate alloc;` declaration. Gathers both window
    /// segments into a fresh heap buffer.
    pub fn snapshot_owned(&self) -> ::alloc::vec::Vec<u8> {
        let n = self.pending_bytes();
        let mut v = ::alloc::vec::Vec::with_capacity(n);
        if let Ok(win) = self.window(0, self.pending_rows()) {
            let start = v.len();
            v.resize(start + win.total_len(), 0);
            win.copy_into(&mut v[start..]);
        }
        v
    }
}
