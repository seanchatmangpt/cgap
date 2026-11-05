//! Hand-rolled minimal Arrow `Message`/`Schema` flatbuffer reader/writer
//! (lane 7, v26.11.6). Zero deps, `no_std`, no `alloc`: every operation
//! runs over caller buffers and borrowed regions, panic-free on inputs —
//! every read is bounds-checked against the metadata region and typed
//! [`FbError`], never an `unwrap` on input bytes.
//!
//! Wire surface (Message.fbs / Schema.fbs, Arrow IPC streaming format):
//! - `Message`: version:short(0), header union type:ubyte(1) /
//!   value:table(2), bodyLength:long(3), custom_metadata:[KeyValue](4);
//!   file_identifier `ARROW1` at bytes 4..8 (we WRITE it; pyarrow/Arrow-C++
//!   does NOT write it in the streaming format — the reader accepts
//!   absent-or-`ARROW1`, any other value is [`FbError::BadIdentifier`]).
//! - `Schema`: endianness:short(0), fields:[Field](1), custom_metadata(2).
//! - `Field`: name:string(0), nullable:bool(1), type_type:ubyte(2),
//!   type:table(3), dictionary:DictionaryEncoding(4), children:[Field](5),
//!   custom_metadata(6).
//! - Type tables written/read: exactly the smalt closed grammar
//!   (Int/FloatingPoint/Utf8/Binary/LargeUtf8/LargeBinary/Bool/Date/Time/
//!   Timestamp/Decimal/Interval/List/FixedSizeList/Struct/Map; NO Union).
//! - DoS fences: nesting depth cap 32 ([`FbError::DepthCap`]), field-count
//!   cap 1024 ([`FbError::FieldCap`]); every vtable slot and every offset
//!   dereference is bounds-checked against the region, so a corrupt or
//!   hostile flatbuffer classifies typed instead of panicking.

/// Maximum nesting depth (smalt schema grammar cap).
pub const DEPTH_CAP: u32 = 32;
/// Maximum field count per field vector (DoS fence).
pub const FIELD_CAP: usize = 1024;

/// The Message.fbs file identifier, written at flatbuffer bytes 4..8.
pub const IDENT: [u8; 6] = *b"ARROW1";

/// Typed flatbuffer parse/emit refusal. Exactly one variant per corrupt
/// class; `Copy` so it can ride [`crate::ipc::IpcError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FbError {
    /// Region shorter than the root offset word.
    TooShort {
        /// Bytes needed.
        needed: usize,
        /// Bytes present.
        got: usize,
    },
    /// Root/vtable/field offset lands outside the region.
    BadOffset,
    /// The file identifier is present but not `ARROW1`.
    BadIdentifier,
    /// Message header union is not Schema (header_type != 1).
    NotSchemaMessage {
        /// header_type value observed.
        got: u8,
    },
    /// A type-union tag is outside the closed grammar (or Union itself).
    BadTypeTag {
        /// Tag value observed.
        got: u8,
    },
    /// A type table field carries an out-of-domain enum value.
    BadEnum {
        /// Value observed.
        got: i64,
    },
    /// A string field is not valid UTF-8.
    BadUtf8,
    /// Nesting deeper than [`DEPTH_CAP`].
    DepthCap,
    /// A field vector longer than [`FIELD_CAP`].
    FieldCap,
    /// A vtable/table shape is internally inconsistent.
    BadTable,
    /// Emission: the caller buffer is too small.
    NoRoom {
        /// Bytes needed.
        needed: usize,
        /// Bytes present.
        got: usize,
    },
}

impl core::fmt::Display for FbError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

// ===========================================================================
// Reader core: bounds-checked flatbuffer table navigation
// ===========================================================================

/// Read a `u16` at `at` inside `buf` (LE), bounds-checked.
fn rd_u16(buf: &[u8], at: usize) -> Result<u16, FbError> {
    let e = at.checked_add(2).ok_or(FbError::BadOffset)?;
    if e > buf.len() {
        return Err(FbError::BadOffset);
    }
    Ok(u16::from_le_bytes([buf[at], buf[at + 1]]))
}

/// Read an `i32` at `at` inside `buf` (LE), bounds-checked.
fn rd_i32(buf: &[u8], at: usize) -> Result<i32, FbError> {
    let e = at.checked_add(4).ok_or(FbError::BadOffset)?;
    if e > buf.len() {
        return Err(FbError::BadOffset);
    }
    Ok(i32::from_le_bytes([
        buf[at],
        buf[at + 1],
        buf[at + 2],
        buf[at + 3],
    ]))
}

/// Read a `u32` at `at` inside `buf` (LE), bounds-checked.
fn rd_u32(buf: &[u8], at: usize) -> Result<u32, FbError> {
    rd_i32(buf, at).map(|v| v as u32)
}

/// Read an `i64` at `at` inside `buf` (LE), bounds-checked.
fn rd_i64(buf: &[u8], at: usize) -> Result<i64, FbError> {
    let e = at.checked_add(8).ok_or(FbError::BadOffset)?;
    if e > buf.len() {
        return Err(FbError::BadOffset);
    }
    let mut b = [0u8; 8];
    b.copy_from_slice(&buf[at..e]);
    Ok(i64::from_le_bytes(b))
}

/// Follow a relative `i32` offset stored at `at` to the pointed position
/// (`vtable` soffset) — `pos = at - stored`, checked in-range.
fn follow_neg(buf: &[u8], at: usize) -> Result<usize, FbError> {
    let s = rd_i32(buf, at)?;
    let pos = (at as i64) - (s as i64);
    if pos < 0 || pos as usize >= buf.len() {
        return Err(FbError::BadOffset);
    }
    Ok(pos as usize)
}

/// Follow a forward `u32` offset stored at `at` to the pointed position
/// (`pos = at + stored`), checked in-range.
fn follow_pos(buf: &[u8], at: usize) -> Result<usize, FbError> {
    let s = rd_u32(buf, at)?;
    let pos = (at as u64).checked_add(s as u64).ok_or(FbError::BadOffset)?;
    if pos as usize >= buf.len() {
        return Err(FbError::BadOffset);
    }
    Ok(pos as usize)
}

/// A parsed table: its region, the table data position (past the soffset),
/// and the vtable position.
#[derive(Clone, Copy)]
pub struct Table<'a> {
    pub(crate) buf: &'a [u8],
    /// Table data start (soffset word + 4).
    pub(crate) pos: usize,
    /// Vtable start.
    pub(crate) vt: usize,
    /// Vtable byte size (including the two leading u16 words).
    pub(crate) vt_size: usize,
}

impl<'a> Table<'a> {
    /// Open the table rooted at region offset 0 (`root` flatbuffer).
    pub fn root(buf: &'a [u8]) -> Result<Table<'a>, FbError> {
        if buf.len() < 4 {
            return Err(FbError::TooShort {
                needed: 4,
                got: buf.len(),
            });
        }
        let tref = follow_pos(buf, 0)?;
        Table::open_at(buf, tref)
    }

    /// Open the table whose reference (soffset word) sits at `tref`.
    /// Voffsets are relative to the soffset word position (flatbuffers
    /// table-position law), so `pos` = `tref` itself.
    fn open_at(buf: &'a [u8], tref: usize) -> Result<Table<'a>, FbError> {
        let vt = follow_neg(buf, tref)?;
        let vt_size = rd_u16(buf, vt)? as usize;
        let tsize = rd_u16(buf, vt + 2)? as usize;
        let tpos = tref;
        if vt_size < 4 || vt + vt_size > buf.len() || tpos > buf.len() || tsize > buf.len() {
            return Err(FbError::BadTable);
        }
        Ok(Table {
            buf,
            pos: tpos,
            vt,
            vt_size,
        })
    }

    /// Open the table at a forward-offset slot `at`.
    pub fn at(buf: &'a [u8], at: usize) -> Result<Table<'a>, FbError> {
        let tref = follow_pos(buf, at)?;
        Table::open_at(buf, tref)
    }

    /// Voffset for field id `id` (0 = absent / default).
    fn voff(&self, id: usize) -> u16 {
        let slot = 4 + id * 2;
        if self.vt_size < slot + 2 {
            return 0;
        }
        rd_u16(self.buf, self.vt + slot).unwrap_or(0)
    }

    /// Position of field `id`'s data inside the region, if present.
    pub(crate) fn field_pos(&self, id: usize) -> Option<usize> {
        let v = self.voff(id) as usize;
        if v == 0 {
            return None;
        }
        Some(self.pos + v)
    }

    /// Inline scalar field `id` with default `dflt`.
    pub(crate) fn scalar_u8(&self, id: usize, dflt: u8) -> Result<u8, FbError> {
        match self.field_pos(id) {
            None => Ok(dflt),
            Some(p) if p < self.buf.len() => Ok(self.buf[p]),
            Some(_) => Err(FbError::BadOffset),
        }
    }

    /// Inline scalar field `id` with default `dflt`.
    pub(crate) fn scalar_i16(&self, id: usize, dflt: i16) -> Result<i16, FbError> {
        self.field_pos(id).map_or(Ok(dflt), |p| rd_i16(self.buf, p))
    }

    /// Inline scalar field `id` with default `dflt`.
    pub(crate) fn scalar_i32(&self, id: usize, dflt: i32) -> Result<i32, FbError> {
        self.field_pos(id).map_or(Ok(dflt), |p| rd_i32(self.buf, p))
    }

    /// Inline scalar field `id` with default `dflt`.
    pub(crate) fn scalar_i64(&self, id: usize, dflt: i64) -> Result<i64, FbError> {
        self.field_pos(id).map_or(Ok(dflt), |p| rd_i64(self.buf, p))
    }

    /// String field `id`, UTF-8 validated.
    pub(crate) fn string(&self, id: usize) -> Result<Option<&'a str>, FbError> {
        let Some(p) = self.field_pos(id) else {
            return Ok(None);
        };
        let sp = follow_pos(self.buf, p)?;
        let n = rd_u32(self.buf, sp)? as usize;
        let s = sp.checked_add(4).ok_or(FbError::BadOffset)?;
        let e = s.checked_add(n).ok_or(FbError::BadOffset)?;
        if e > self.buf.len() {
            return Err(FbError::BadOffset);
        }
        core::str::from_utf8(&self.buf[s..e]).map(Some).map_err(|_| FbError::BadUtf8)
    }

    /// Vector-of-offsets field `id`: returns (element positions base,
    /// element count). The base is the position of the first u32 slot.
    pub(crate) fn offset_vector(&self, id: usize) -> Result<Option<(usize, usize)>, FbError> {
        let Some(p) = self.field_pos(id) else {
            return Ok(None);
        };
        let vp = follow_pos(self.buf, p)?;
        let n = rd_u32(self.buf, vp)? as usize;
        if n > FIELD_CAP {
            return Err(FbError::FieldCap);
        }
        let base = vp.checked_add(4).ok_or(FbError::BadOffset)?;
        let e = base.checked_add(n * 4).ok_or(FbError::BadOffset)?;
        if e > self.buf.len() {
            return Err(FbError::BadOffset);
        }
        Ok(Some((base, n)))
    }

    /// Table field `id`.
    pub(crate) fn table(&self, id: usize) -> Result<Option<Table<'a>>, FbError> {
        let Some(p) = self.field_pos(id) else {
            return Ok(None);
        };
        Table::at(self.buf, p).map(Some)
    }

    /// `[KeyValue]` field `id` as a [`Kv`] lazy view.
    pub(crate) fn key_values(&self, id: usize) -> Result<Option<Kv<'a>>, FbError> {
        let Some((base, n)) = self.offset_vector(id)? else {
            return Ok(None);
        };
        Ok(Some(Kv { buf: self.buf, base, n }))
    }
}

/// Read an `i16` at `at` inside `buf` (LE), bounds-checked.
fn rd_i16(buf: &[u8], at: usize) -> Result<i16, FbError> {
    rd_u16(buf, at).map(|v| v as i16)
}

// ===========================================================================
// Closed type grammar (the smalt schema.ex mapping table, one lane 7 law)
// ===========================================================================

/// Arrow `MetadataVersion` — we emit V5 (=4), accept >= 1.
pub const METADATA_VERSION_V5: i16 = 4;
/// MessageHeader union tag for `Schema`.
pub const HEADER_TYPE_SCHEMA: u8 = 1;

/// Half/single/double (`FloatingPoint.Precision`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prec {
    /// `HALF` (f16).
    Half,
    /// `SINGLE` (f32).
    Single,
    /// `DOUBLE` (f64).
    Double,
}

/// `DateUnit`: `DAY` (date32) / `MILLISECOND` (date64).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateUnit {
    /// `DAY` — date32.
    Day,
    /// `MILLISECOND` — date64.
    Milli,
}

/// `TimeUnit`: `SECOND`/`MILLISECOND`/`MICROSECOND`/`NANOSECOND`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    /// Seconds.
    Second,
    /// Milliseconds.
    Milli,
    /// Microseconds.
    Micro,
    /// Nanoseconds.
    Nano,
}

/// `IntervalUnit`: `YEAR_MONTH`/`DAY_TIME`/`MONTH_DAY_NANO`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntervalUnit {
    /// Year-month.
    YearMonth,
    /// Day-time.
    DayTime,
    /// Month-day-nano.
    MonthDayNano,
}

/// Dictionary index type: the integer-width family (smalt `index_type()`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexTy {
    /// Signed 8-bit index.
    I8,
    /// Signed 16-bit index.
    I16,
    /// Signed 32-bit index.
    I32,
    /// Signed 64-bit index.
    I64,
    /// Unsigned 8-bit index.
    U8,
    /// Unsigned 16-bit index.
    U16,
    /// Unsigned 32-bit index.
    U32,
    /// Unsigned 64-bit index.
    U64,
}

impl IndexTy {
    /// Index bit width.
    #[must_use]
    pub fn bits(self) -> u16 {
        match self {
            IndexTy::I8 | IndexTy::U8 => 8,
            IndexTy::I16 | IndexTy::U16 => 16,
            IndexTy::I32 | IndexTy::U32 => 32,
            IndexTy::I64 | IndexTy::U64 => 64,
        }
    }
    /// Whether the index type is signed.
    #[must_use]
    pub fn signed(self) -> bool {
        matches!(self, IndexTy::I8 | IndexTy::I16 | IndexTy::I32 | IndexTy::I64)
    }
    fn from_field(f: &FieldView<'_>) -> Result<IndexTy, FbError> {
        match &f.ty {
            Ty::Int { bits, signed } => match (*bits, *signed) {
                (8, true) => Ok(IndexTy::I8),
                (16, true) => Ok(IndexTy::I16),
                (32, true) => Ok(IndexTy::I32),
                (64, true) => Ok(IndexTy::I64),
                (8, false) => Ok(IndexTy::U8),
                (16, false) => Ok(IndexTy::U16),
                (32, false) => Ok(IndexTy::U32),
                (64, false) => Ok(IndexTy::U64),
                _ => Err(FbError::BadEnum {
                    got: i64::from(*bits) * 2 + i64::from(*signed),
                }),
            },
            _ => Err(FbError::BadTypeTag { got: 2 }),
        }
    }
}

/// Lazy `[KeyValue]` custom_metadata view (borrowed, no allocation).
#[derive(Clone, Copy)]
pub struct Kv<'a> {
    buf: &'a [u8],
    base: usize,
    n: usize,
}

impl<'a> Kv<'a> {
    /// Pair count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.n
    }
    /// Whether there are no pairs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
    /// Pair `i` as `(key, value)`.
    pub fn get(&self, i: usize) -> Result<(&'a str, &'a str), FbError> {
        if i >= self.n {
            return Err(FbError::BadOffset);
        }
        let t = Table::at(self.buf, self.base + i * 4)?;
        let k = t.string(0)?.unwrap_or("");
        let v = t.string(1)?.unwrap_or("");
        Ok((k, v))
    }
}

impl PartialEq for Kv<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.n != other.n {
            return false;
        }
        for i in 0..self.n {
            match (self.get(i), other.get(i)) {
                (Ok(a), Ok(b)) if a == b => {}
                _ => return false,
            }
        }
        true
    }
}

/// Lazy `[Field]` view (borrowed, no allocation).
#[derive(Clone, Copy)]
pub struct Fields<'a> {
    buf: &'a [u8],
    base: usize,
    n: usize,
}

impl<'a> Fields<'a> {
    /// Field count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.n
    }
    /// Whether there are no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
    /// Field `i`.
    pub fn get(&self, i: usize) -> Result<FieldView<'a>, FbError> {
        if i >= self.n {
            return Err(FbError::BadOffset);
        }
        FieldView::open(self.buf, self.base + i * 4)
    }
}

/// Lazy child-field reference (`List`/`FixedSizeList` element). A Child
/// is a position; [`Child::field`] opens it.
#[derive(Clone, Copy)]
pub struct Child<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) at: usize,
}

impl<'a> Child<'a> {
    /// Open the child field.
    pub fn field(&self) -> Result<FieldView<'a>, FbError> {
        FieldView::open(self.buf, self.at)
    }
}

/// Dictionary encoding view (`DictionaryEncoding` table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DictView {
    /// Index (dictionary) type.
    pub index: IndexTy,
    /// Dictionary id (defaults 0).
    pub id: i64,
}

/// One parsed Arrow field (borrowed views over the metadata region).
#[derive(Clone, Copy)]
pub struct FieldView<'a> {
    /// Field name.
    pub name: &'a str,
    /// Nullability flag (flatbuffers default true; Arrow writes it).
    pub nullable: bool,
    /// Closed-grammar type.
    pub ty: Ty<'a>,
    /// Dictionary encoding, when the field is dictionary-encoded.
    pub dictionary: Option<DictView>,
    /// Field-level custom metadata.
    pub meta: Kv<'a>,
}

impl<'a> FieldView<'a> {
    fn open(buf: &'a [u8], at: usize) -> Result<FieldView<'a>, FbError> {
        let t = Table::at(buf, at)?;
        let name = t.string(0)?.unwrap_or("");
        let nullable = t.scalar_u8(1, 0)? != 0;
        let tag = t.scalar_u8(2, 0)?;
        let tyt = t.table(3)?.ok_or(FbError::BadTypeTag { got: tag })?;
        let field_children = t.offset_vector(5)?;
        let ty = Ty::from_table(buf, tyt, tag, field_children)?;
        let dictionary = match t.table(4)? {
            None => None,
            Some(d) => {
                let index_field = d.table(1)?;
                let index = match index_field {
                    None => IndexTy::I32,
                    Some(f) => match FieldView::bare(f) {
                        // pyarrow writes the dictionary indexType Field
                        // WITHOUT a type union (tag 0): the Arrow
                        // dictionary default index (i32) is the recorded
                        // law for the absent-tag case.
                        Err(FbError::BadTypeTag { got: 0 }) => IndexTy::I32,
                        other => IndexTy::from_field(&other?)?,
                    },
                };
                let id = d.scalar_i64(3, 0)?;
                Some(DictView { index, id })
            }
        };
        let meta = t.key_values(6)?.unwrap_or(Kv {
            buf,
            base: 0,
            n: 0,
        });
        Ok(FieldView {
            name,
            nullable,
            ty,
            dictionary,
            meta,
        })
    }

    /// Parse just the type of a dictionary `indexType` field (no name).
    fn bare(t: Table<'a>) -> Result<FieldView<'a>, FbError> {
        let tag = t.scalar_u8(2, 0)?;
        let tyt = t.table(3)?.ok_or(FbError::BadTypeTag { got: tag })?;
        let fc = t.offset_vector(5)?;
        let ty = Ty::from_table(t.buf, tyt, tag, fc)?;
        Ok(FieldView {
            name: "",
            nullable: false,
            ty,
            dictionary: None,
            meta: Kv {
                buf: t.buf,
                base: 0,
                n: 0,
            },
        })
    }
}

impl PartialEq for FieldView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.nullable == other.nullable
            && self.ty == other.ty
            && self.dictionary == other.dictionary
            && self.meta == other.meta
    }
}

/// Closed-grammar type view (borrowed; NO Union — outside the grammar).
#[derive(Clone, Copy)]
pub enum Ty<'a> {
    /// `Bool`.
    Bool,
    /// `Int` (bit width + signedness) — the i/u 8..64 family.
    Int {
        /// Bit width (8/16/32/64).
        bits: u16,
        /// Signedness.
        signed: bool,
    },
    /// `FloatingPoint`.
    Float(Prec),
    /// `Utf8`.
    Utf8,
    /// `LargeUtf8`.
    LargeUtf8,
    /// `Binary`.
    Binary,
    /// `LargeBinary`.
    LargeBinary,
    /// `Date`.
    Date(DateUnit),
    /// `Time` (unit + bit width 32/64).
    Time {
        /// Time unit.
        unit: TimeUnit,
        /// Bit width (32 for s/ms, 64 for us/ns).
        bits: u16,
    },
    /// `Timestamp` (unit + optional timezone).
    Timestamp {
        /// Time unit.
        unit: TimeUnit,
        /// Timezone string, `None` when absent.
        tz: Option<&'a str>,
    },
    /// `Decimal` (precision, scale, bit width 128/256).
    Decimal {
        /// Precision (1..=38 per smalt grammar).
        precision: u8,
        /// Scale.
        scale: i32,
        /// Bit width (default 128).
        bits: u16,
    },
    /// `Interval`.
    Interval(IntervalUnit),
    /// `List` (single child field).
    List(Child<'a>),
    /// `FixedSizeList` (listSize + single child field).
    Fsl {
        /// Fixed list size.
        n: i32,
        /// Child field reference.
        elem: Child<'a>,
    },
    /// `Struct` (children).
    Struct(Fields<'a>),
    /// `Map` (keysSorted + entries Struct child).
    Map {
        /// Whether keys are sorted.
        sorted: bool,
        /// Entries (key/value fields).
        entries: Fields<'a>,
    },
}

impl<'a> Ty<'a> {
    fn from_table(
        buf: &'a [u8],
        t: Table<'a>,
        tag: u8,
        field_children: Option<(usize, usize)>,
    ) -> Result<Ty<'a>, FbError> {
        match tag {
            6 => Ok(Ty::Bool),
            2 => {
                let bits = t.scalar_i32(0, 0)?;
                let signed = t.scalar_u8(1, 0)? != 0;
                Ok(Ty::Int {
                    bits: u16::try_from(bits).map_err(|_| FbError::BadEnum { got: i64::from(bits) })?,
                    signed,
                })
            }
            3 => Ok(match t.scalar_i16(0, 0)? {
                0 => Ty::Float(Prec::Half),
                1 => Ty::Float(Prec::Single),
                2 => Ty::Float(Prec::Double),
                v => return Err(FbError::BadEnum { got: i64::from(v) }),
            }),
            5 => Ok(Ty::Utf8),
            20 => Ok(Ty::LargeUtf8),
            4 => Ok(Ty::Binary),
            19 => Ok(Ty::LargeBinary),
            8 => Ok(match t.scalar_i16(0, 0)? {
                0 => Ty::Date(DateUnit::Day),
                1 => Ty::Date(DateUnit::Milli),
                v => return Err(FbError::BadEnum { got: i64::from(v) }),
            }),
            9 => {
                let unit = time_unit(t.scalar_i16(0, 0)?)?;
                let bits = t.scalar_i32(1, 32)?;
                Ok(Ty::Time {
                    unit,
                    bits: u16::try_from(bits).map_err(|_| FbError::BadEnum { got: i64::from(bits) })?,
                })
            }
            10 => {
                let unit = time_unit(t.scalar_i16(0, 0)?)?;
                let tz = t.string(1)?;
                Ok(Ty::Timestamp { unit, tz })
            }
            7 => {
                let precision = t.scalar_i32(0, 0)?;
                let scale = t.scalar_i32(1, 0)?;
                let bits = t.scalar_i32(2, 128)?;
                if !(0..=255).contains(&precision) || !(0..=256).contains(&bits) {
                    return Err(FbError::BadEnum {
                        got: i64::from(precision),
                    });
                }
                Ok(Ty::Decimal {
                    precision: precision as u8,
                    scale,
                    bits: bits as u16,
                })
            }
            11 => Ok(match t.scalar_i16(0, 0)? {
                0 => Ty::Interval(IntervalUnit::YearMonth),
                1 => Ty::Interval(IntervalUnit::DayTime),
                2 => Ty::Interval(IntervalUnit::MonthDayNano),
                v => return Err(FbError::BadEnum { got: i64::from(v) }),
            }),
            12 | 16 | 13 | 17 => {
                let (base, n) = t
                    .offset_vector(5)?
                    .or(field_children)
                    .ok_or(FbError::BadTypeTag { got: tag })?;
                if n > 1 && (tag == 12 || tag == 16) {
                    return Err(FbError::BadTypeTag { got: tag });
                }
                if n > FIELD_CAP {
                    return Err(FbError::FieldCap);
                }
                let fields = Fields { buf, base, n };
                match tag {
                    12 => Ok(Ty::List(Child { buf, at: base })),
                    16 => {
                        let sz = t.scalar_i32(0, 0)?;
                        Ok(Ty::Fsl { n: sz, elem: Child { buf, at: base } })
                    }
                    13 => Ok(Ty::Struct(fields)),
                    _ => {
                        let sorted = t.scalar_u8(0, 0)? != 0;
                        Ok(Ty::Map { sorted, entries: fields })
                    }
                }
            }
            _ => Err(FbError::BadTypeTag { got: tag }),
        }
    }
}

/// `TimeUnit` from the flatbuffer short.
#[must_use]
pub fn time_unit(v: i16) -> Result<TimeUnit, FbError> {
    Ok(match v {
        0 => TimeUnit::Second,
        1 => TimeUnit::Milli,
        2 => TimeUnit::Micro,
        3 => TimeUnit::Nano,
        _ => return Err(FbError::BadEnum { got: i64::from(v) }),
    })
}

/// Child structural equality (opens both children).
#[must_use]
fn a_elem_eq(ae: &Child<'_>, be: &Child<'_>) -> bool {
    match (ae.field(), be.field()) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

impl PartialEq for Ty<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ty::Bool, Ty::Bool)
            | (Ty::Utf8, Ty::Utf8)
            | (Ty::LargeUtf8, Ty::LargeUtf8)
            | (Ty::Binary, Ty::Binary)
            | (Ty::LargeBinary, Ty::LargeBinary) => true,
            (Ty::Int { bits: a, signed: b }, Ty::Int { bits: c, signed: d }) => a == c && b == d,
            (Ty::Float(a), Ty::Float(b)) => a == b,
            (Ty::Date(a), Ty::Date(b)) => a == b,
            (
                Ty::Time {
                    unit: a,
                    bits: ab,
                },
                Ty::Time {
                    unit: b,
                    bits: bb,
                },
            ) => a == b && ab == bb,
            (
                Ty::Timestamp { unit: a, tz: az },
                Ty::Timestamp { unit: b, tz: bz },
            ) => a == b && az == bz,
            (
                Ty::Decimal {
                    precision: ap,
                    scale: asc,
                    bits: ab,
                },
                Ty::Decimal {
                    precision: bp,
                    scale: bsc,
                    bits: bb,
                },
            ) => ap == bp && asc == bsc && ab == bb,
            (Ty::Interval(a), Ty::Interval(b)) => a == b,
            (Ty::List(a), Ty::List(b)) => a.field().ok().zip(b.field().ok()).is_some_and(|(x, y)| x == y),
            (Ty::Fsl { n: an, elem: ae }, Ty::Fsl { n: bn, elem: be }) => {
                an == bn && a_elem_eq(ae, be)
            }
            (Ty::Struct(a), Ty::Struct(b)) => fields_eq(a, b),
            (
                Ty::Map {
                    sorted: asc,
                    entries: ae,
                },
                Ty::Map {
                    sorted: bsc,
                    entries: be,
                },
            ) => asc == bsc && fields_eq(ae, be),
            _ => false,
        }
    }
}

/// Structural equality of two field lists.
#[must_use]
pub fn fields_eq(a: &Fields<'_>, b: &Fields<'_>) -> bool {
    if a.len() != b.len() {
        return false;
    }
    (0..a.len()).all(|i| match (a.get(i), b.get(i)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    })
}

// ===========================================================================
// Message + Schema views
// ===========================================================================

/// A parsed `Schema` message (borrowed over the metadata region).
#[derive(Clone, Copy)]
pub struct SchemaView<'a> {
    buf: &'a [u8],
    root: Table<'a>,
    /// Schema endianness flag (0 = little).
    pub endianness: i16,
}

impl<'a> SchemaView<'a> {
    /// Parse a `Schema` table from the flatbuffer `region` rooted at 0.
    pub fn open(region: &'a [u8]) -> Result<SchemaView<'a>, FbError> {
        let root = Table::root(region)?;
        let endianness = root.scalar_i16(0, 0)?;
        Ok(SchemaView {
            buf: region,
            root,
            endianness,
        })
    }

    /// Top-level fields.
    pub fn fields(&self) -> Result<Fields<'a>, FbError> {
        let Some((base, n)) = self.root.offset_vector(1)? else {
            return Ok(Fields {
                buf: self.buf,
                base: 0,
                n: 0,
            });
        };
        Ok(Fields {
            buf: self.buf,
            base,
            n,
        })
    }

    /// Schema-level custom metadata.
    pub fn meta(&self) -> Result<Kv<'a>, FbError> {
        Ok(self.root.key_values(2)?.unwrap_or(Kv {
            buf: self.buf,
            base: 0,
            n: 0,
        }))
    }
}

impl PartialEq for SchemaView<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self.fields(), other.fields()) {
            (Ok(a), Ok(b)) => fields_eq(&a, &b) && self.endianness == other.endianness,
            _ => false,
        }
    }
}

/// A parsed `Message` envelope (borrowed over the metadata region).
#[derive(Clone, Copy)]
pub struct MessageView<'a> {
    /// Metadata version (we emit 4 = V5; accept >= 1).
    pub version: i16,
    /// Declared body length (`bodyLength`).
    pub body_length: i64,
    /// The inner `Schema` table.
    pub schema: SchemaView<'a>,
}

/// Parse a `Message` flatbuffer rooted at region offset 0. The header
/// union MUST be `Schema` ([`FbError::NotSchemaMessage`]); the identifier
/// must be absent-or-`ARROW1` ([`FbError::BadIdentifier`]).
pub fn read_message(region: &[u8]) -> Result<MessageView<'_>, FbError> {
    if region.len() >= 8 && region[4] != 0 {
        if region[4..8] != IDENT4 {
            return Err(FbError::BadIdentifier);
        }
    }
    let _ = IDENT;
    let root = Table::root(region)?;
    let version = root.scalar_i16(0, 0)?;
    let header_type = root.scalar_u8(1, 0)?;
    if header_type != HEADER_TYPE_SCHEMA {
        return Err(FbError::NotSchemaMessage {
            got: header_type,
        });
    }
    let body_length = root.scalar_i64(3, 0)?;
    let hp = root
        .field_pos(2)
        .ok_or(FbError::NotSchemaMessage { got: header_type })?;
    let st = Table::at(region, hp)?;
    let schema = SchemaView {
        buf: region,
        root: st,
        endianness: st.scalar_i16(0, 0)?,
    };
    Ok(MessageView {
        version,
        body_length,
        schema,
    })
}

/// Structural equality of two parsed schema messages.
#[must_use]
pub fn message_eq(a: &MessageView<'_>, b: &MessageView<'_>) -> bool {
    a.version == b.version
        && a.body_length == b.body_length
        && a.schema.endianness == b.schema.endianness
        && a.schema == b.schema
}


// ===========================================================================
// Writer: backwards flatbuffer builder over a caller buffer (no alloc)
// ===========================================================================

/// File identifier written at flatbuffer bytes 4..8 (`ARROW1` truncated to
/// the 4-byte flatbuffers identifier slot).
pub const IDENT4: [u8; 4] = *b"ARRO";

/// Writer-side field/children cap per vector (the reader's DoS fence
/// [`FIELD_CAP`] stays 1024; the writer is tighter at 64 — cgap schemas
/// are small, and the asymmetry is a documented lane decision).
pub const MAX_SLOTS: usize = 64;

/// Backwards flatbuffer builder over a caller buffer. All offsets are
/// "distance from the end of the caller buffer to the object start"; a
/// final [`Builder::finish`] writes the root offset first, so the flat
/// buffer length is a multiple of 8 and every element's in-flatbuffer
/// index is 8-aligned — internal alignment holds wherever the region lands.
struct Builder<'o> {
    buf: &'o mut [u8],
    head: usize,
}

/// One table slot row.
struct Row {
    id: usize,
    slot: Slot,
    /// Offset-from-end of the slot's storage (filled by [`Builder::table`]).
    off_start: usize,
}

/// One table slot payload.
enum Slot {
    /// Forward offset slot (patched after the table is placed).
    Off(usize),
    /// `u8` / bool slot.
    U8(u8),
    /// `i16` slot.
    I16(i16),
    /// `i32` slot.
    I32(i32),
    /// `i64` slot.
    I64(i64),
}

impl<'o> Builder<'o> {
    fn new(buf: &'o mut [u8]) -> Builder<'o> {
        Builder {
            head: buf.len(),
            buf,
        }
    }

    /// Offset-from-end of the write head.
    fn used(&self) -> usize {
        self.buf.len() - self.head
    }

    /// Copy `bytes` ending at the head. Returns the offset-from-end of the
    /// written block start.
    fn push(&mut self, bytes: &[u8]) -> Result<usize, FbError> {
        let n = bytes.len();
        let new_head = self
            .head
            .checked_sub(n)
            .ok_or(FbError::NoRoom {
                needed: self.used() + n,
                got: self.buf.len(),
            })?;
        self.buf[new_head..self.head].copy_from_slice(bytes);
        self.head = new_head;
        Ok(self.used())
    }

    /// 8-align the head, then push. Every object is 8-aligned from the end,
    /// so (with an 8-aligned final length) internal flatbuffer alignment is
    /// natural wherever the region lands.
    fn push_aligned(&mut self, bytes: &[u8]) -> Result<usize, FbError> {
        let pad = (8 - (self.used() % 8)) % 8;
        if pad > 0 {
            self.push(&[0u8; 8][..pad])?;
        }
        self.push(bytes)
    }

    /// Create a string object (u32 length + UTF-8 bytes + NUL). Returns the
    /// object offset (points at the length word).
    fn string(&mut self, s: &str) -> Result<usize, FbError> {
        // Backwards build: NUL FIRST (lands above the bytes), then the
        // bytes, then the length word -> forward layout [len][bytes][NUL].
        let _ = self.push(&[0u8]);
        let _ = self.push_aligned(s.as_bytes())?;
        let mut len = [0u8; 4];
        len.copy_from_slice(&(s.len() as u32).to_le_bytes());
        let _ = self.push(&len);
        Ok(self.used())
    }

    /// Create a `[offset]` vector over `elems`. Returns the vector offset
    /// (points at the count word).
    fn offset_vector(&mut self, elems: &[usize; MAX_SLOTS], n: usize) -> Result<usize, FbError> {
        if n > FIELD_CAP {
            return Err(FbError::FieldCap);
        }
        // Placeholder rel slots (element i slot patched after the count
        // word is placed).
        let _base = self.push_aligned(&[0u8; 4 * MAX_SLOTS][..4 * n])?;
        let count = (n as u32).to_le_bytes();
        let _ = self.push(&count);
        let vec_off = self.used();
        for (i, e) in elems[..n].iter().enumerate() {
            let at = self
                .buf
                .len()
                .checked_sub(vec_off - 4 * (i + 1))
                .ok_or(FbError::BadOffset)?;
            let rel = ((vec_off - 4 * (i + 1)) - *e) as i64;
            if rel < 0 {
                return Err(FbError::BadOffset);
            }
            self.buf[at..at + 4].copy_from_slice(&(rel as u32).to_le_bytes());
        }
        Ok(vec_off)
    }

    /// Create a table from `(id, slot)` rows (unique ids, ascending order
    /// expected from callers). Returns the table offset (points at the
    /// vtable soffset word).
    fn table(&mut self, rows: &mut [Row; MAX_SLOTS]) -> Result<usize, FbError> {
        // Highest id first, so the forward layout has ascending ids.
        let n = rows.iter().take_while(|r| r.id != usize::MAX).count();
        // no_std: slice sort needs alloc; swap-based insertion sort
        // (n <= 64, [Row; MAX_SLOTS] fixed buffer).
        for i in 1..n {
            let mut j = i;
            while j > 0 && rows[j - 1].id < rows[j].id {
                rows.swap(j - 1, j);
                j -= 1;
            }
        }
        let nslots = rows[..n].iter().map(|r| r.id + 1).max().unwrap_or(0);
        if nslots > 64 {
            return Err(FbError::BadTable);
        }
        for r in rows[..n].iter_mut() {
            r.off_start = match &r.slot {
                Slot::Off(_) => self.push_aligned(&[0u8; 4])?,
                Slot::U8(v) => self.push_aligned(&[*v])?,
                Slot::I16(v) => self.push_aligned(&v.to_le_bytes())?,
                Slot::I32(v) => self.push_aligned(&v.to_le_bytes())?,
                Slot::I64(v) => self.push_aligned(&v.to_le_bytes())?,
            };
        }
        let table_data_off = self.used();
        let vt_bytes = 4 + 2 * nslots;
        let voffs = [0u8; 2 * 64];
        // vtable: [vt_size u16][table_size u16][voffsets...] (voffs slice
        // pushed first, then the head, then the soffset: block layout is
        // [so][head][voffs][slots...]).
        #[allow(unused_assignments)]
        let mut voffs_off_start = 0usize;
        for r in rows[..n].iter() {
            if let Slot::Off(elem) = &r.slot {
                if *elem == 0 {
                    // Flatbuffers slot absence == default; skip.
                    continue;
                }
                let at = self
                    .buf
                    .len()
                    .checked_sub(r.off_start)
                    .ok_or(FbError::BadOffset)?;
                let rel = (r.off_start - *elem) as i64;
                if rel < 0 {
                    return Err(FbError::BadOffset);
                }
                self.buf[at..at + 4].copy_from_slice(&(rel as u32).to_le_bytes());
            }
        }
        // Push the vtable block, then patch voffsets with exact final
        // positions: voff(id) = u_t - off_start (tref-relative).
        voffs_off_start = self.push_aligned(&voffs[..vt_bytes])?;
        let table_size = (self.used() - table_data_off) as u16 + 4;
        let mut head = [0u8; 4];
        head[..2].copy_from_slice(&(vt_bytes as u16).to_le_bytes());
        head[2..].copy_from_slice(&table_size.to_le_bytes());
        let _ = self.push(&head);
        // Soffset (i32, LE): tref_idx - vt_idx. Layout is [so][head][voffs]
        // (so pushed last), so the vtable sits at tref + 4 => so == -4.
        let so = (-4i32).to_le_bytes();
        let _ = self.push(&so);
        let u_t = self.used();
        for r in rows[..n].iter() {
            let voff = u_t - r.off_start;
            if voff > u16::MAX as usize || r.id >= 64 {
                return Err(FbError::BadTable);
            }
            let at = self
                .buf
                .len()
                .checked_sub(voffs_off_start - 2 * r.id)
                .ok_or(FbError::BadOffset)?;
            self.buf[at] = (voff & 0xff) as u8;
            self.buf[at + 1] = ((voff >> 8) & 0xff) as u8;
        }
        Ok(u_t)
    }

    /// Finish: root offset first, `ARRO` identifier at bytes 4..8. Returns
    /// the flatbuffer length (multiple of 8).
    fn finish(mut self, root: usize) -> Result<usize, FbError> {
        // Backwards builder: the LAST push lands at the lowest indices, so
        // push the identifier first, then the root offset. The root value
        // is the ABSOLUTE index of the root table reference (the head
        // after the identifier push).
        let _ = root;
        let u_t = self.used();
        let _ = self.push_aligned(&IDENT4);
        // Region-relative root value: the index of the root table
        // reference = U_f - off(tref), computed AFTER this word's own push
        // (alignment padding between ident and tref shifts the index).
        let mut r = [0u8; 4];
        let _ = self.push(&r);
        let val = (self.used() - u_t) as u32;
        r.copy_from_slice(&val.to_le_bytes());
        let at = self.head;
        self.buf[at..at + 4].copy_from_slice(&r);
        Ok(self.used())
    }
}


// ===========================================================================
// Encode driver: the caller-supplied schema tree -> Message flatbuffer
// ===========================================================================

/// Dictionary encoding for emission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DictEnc {
    /// Index type.
    pub index: IndexTy,
    /// Dictionary id.
    pub id: i64,
}

/// One field to encode (borrowed tree; zero allocation).
#[derive(Clone, Copy)]
pub struct Fe<'a> {
    /// Field name.
    pub name: &'a str,
    /// Nullability.
    pub nullable: bool,
    /// Closed-grammar type.
    pub ty: Te<'a>,
    /// Optional dictionary encoding.
    pub dict: Option<DictEnc>,
    /// Field custom metadata.
    pub meta: &'a [(&'a str, &'a str)],
}

/// Type to encode (the smalt closed grammar; NO Union).
#[derive(Clone, Copy)]
pub enum Te<'a> {
    /// `Bool`.
    Bool,
    /// `Int` (bits 8/16/32/64 + signedness).
    Int { bits: u8, signed: bool },
    /// `FloatingPoint`.
    Float(Prec),
    /// `Utf8`.
    Utf8,
    /// `LargeUtf8`.
    LargeUtf8,
    /// `Binary`.
    Binary,
    /// `LargeBinary`.
    LargeBinary,
    /// `Date`.
    Date(DateUnit),
    /// `Time` (unit + 32/64 bit width).
    Time { unit: TimeUnit, bits: u16 },
    /// `Timestamp` (unit + optional timezone).
    Timestamp { unit: TimeUnit, tz: Option<&'a str> },
    /// `Decimal` (precision 1..=38, scale, 128/256 bit width).
    Decimal { precision: u8, scale: i32, bits: u16 },
    /// `Interval`.
    Interval(IntervalUnit),
    /// `List` (element field).
    List(&'a Fe<'a>),
    /// `FixedSizeList` (size + element field).
    Fsl { n: i32, elem: &'a Fe<'a> },
    /// `Struct` (children).
    Struct(&'a [Fe<'a>]),
    /// `Map` (entries key/value field types).
    Map { sorted: bool, key: &'a Te<'a>, value: &'a Te<'a> },
}

/// Schema to encode.
#[derive(Clone, Copy)]
pub struct SchemaEnc<'a> {
    /// Little-endian flag (Arrow default true).
    pub little_endian: bool,
    /// Top-level fields.
    pub fields: &'a [Fe<'a>],
    /// Schema custom metadata.
    pub meta: &'a [(&'a str, &'a str)],
}

/// Nesting-depth fence for the encoder.
struct Depth(u32);

impl Depth {
    fn new() -> Depth {
        Depth(1)
    }
    fn enter(&mut self) -> Result<(), FbError> {
        self.0 += 1;
        if self.0 > DEPTH_CAP {
            return Err(FbError::DepthCap);
        }
        Ok(())
    }
}

/// Encode the closed-grammar type into its flatbuffer type table; returns
/// the type-table offset and the union tag. `children_rows` is scratch for
/// the children vector offsets.
fn encode_ty(
    b: &mut Builder<'_>,
    t: &Te<'_>,
    d: &mut Depth,
) -> Result<(usize, u8), FbError> {
    d.enter()?;
    let r = match t {
        Te::Bool => (empty_table(b)?, 6u8),
        Te::Int { bits, signed } => {
            let mut rows = mk_rows();
            add_i32(&mut rows, 0, i32::from(*bits));
            add_u8(&mut rows, 1, u8::from(*signed));
            (b.table(&mut rows)?, 2)
        }
        Te::Float(p) => {
            let v = match p {
                Prec::Half => 0i16,
                Prec::Single => 1,
                Prec::Double => 2,
            };
            let mut rows = mk_rows();
            add_i16(&mut rows, 0, v);
            (b.table(&mut rows)?, 3)
        }
        Te::Utf8 => (empty_table(b)?, 5),
        Te::LargeUtf8 => (empty_table(b)?, 20),
        Te::Binary => (empty_table(b)?, 4),
        Te::LargeBinary => (empty_table(b)?, 19),
        Te::Date(u) => {
            let v = match u {
                DateUnit::Day => 0i16,
                DateUnit::Milli => 1,
            };
            let mut rows = mk_rows();
            add_i16(&mut rows, 0, v);
            (b.table(&mut rows)?, 8)
        }
        Te::Time { unit, bits } => {
            let mut rows = mk_rows();
            add_i16(&mut rows, 0, unit_short(*unit));
            add_i32(&mut rows, 1, i32::from(*bits));
            (b.table(&mut rows)?, 9)
        }
        Te::Timestamp { unit, tz } => {
            let mut rows = mk_rows();
            add_i16(&mut rows, 0, unit_short(*unit));
            if let Some(tz) = tz {
                let off = b.string(tz)?;
                add_off(&mut rows, 1, off);
            }
            (b.table(&mut rows)?, 10)
        }
        Te::Decimal { precision, scale, bits } => {
            let mut rows = mk_rows();
            add_i32(&mut rows, 0, i32::from(*precision));
            add_i32(&mut rows, 1, *scale);
            if *bits != 128 {
                add_i32(&mut rows, 2, i32::from(*bits));
            }
            (b.table(&mut rows)?, 7)
        }
        Te::Interval(u) => {
            let v = match u {
                IntervalUnit::YearMonth => 0i16,
                IntervalUnit::DayTime => 1,
                IntervalUnit::MonthDayNano => 2,
            };
            let mut rows = mk_rows();
            add_i16(&mut rows, 0, v);
            (b.table(&mut rows)?, 11)
        }
        Te::List(elem) => {
            let child = encode_field(b, elem, d)?;
            let (base, n) = children_vec(b, &[child])?;
            let mut rows = mk_rows();
            add_off(&mut rows, 5, base);
            let _ = n;
            (b.table(&mut rows)?, 12)
        }
        Te::Fsl { n: sz, elem } => {
            let child = encode_field(b, elem, d)?;
            let (base, _) = children_vec(b, &[child])?;
            let mut rows = mk_rows();
            add_i32(&mut rows, 0, *sz);
            add_off(&mut rows, 5, base);
            (b.table(&mut rows)?, 16)
        }
        Te::Struct(children) => {
            let mut offs = [0usize; MAX_SLOTS];
            let mut n = 0usize;
            for c in children.iter() {
                if n >= MAX_SLOTS {
                    return Err(FbError::FieldCap);
                }
                offs[n] = encode_field(b, c, d)?;
                n += 1;
            }
            let (base, _) = children_vec(b, offs[..n].try_into().map_err(|_| FbError::FieldCap)?)?;
            let mut rows = mk_rows();
            add_off(&mut rows, 5, base);
            (b.table(&mut rows)?, 13)
        }
        Te::Map { sorted, key, value } => {
            // entries Struct child: key non-nullable, value nullable.
            let kf = Fe { name: "key", nullable: false, ty: *(*key), dict: None, meta: &[] };
            let vf = Fe { name: "value", nullable: true, ty: *(*value), dict: None, meta: &[] };
            let entries = [kf, vf];
            let entries_field = encode_field(b, &Fe {
                name: "entries",
                nullable: false,
                ty: Te::Struct(&entries),
                dict: None,
                meta: &[],
            }, d)?;
            let (base, _) = children_vec(b, &[entries_field])?;
            let mut rows = mk_rows();
            add_u8(&mut rows, 0, u8::from(*sorted));
            add_off(&mut rows, 5, base);
            (b.table(&mut rows)?, 17)
        }
    };
    d.exit();
    Ok(r)
}

/// Nesting-depth exit (encoder side).
impl Depth {
    fn exit(&mut self) {
        self.0 -= 1;
    }
}

/// Empty table (Utf8/Binary/Bool/LargeUtf8/LargeBinary families).
fn empty_table(b: &mut Builder<'_>) -> Result<usize, FbError> {
    let mut rows = mk_rows();
    b.table(&mut rows)
}

/// Row scratch helpers (fixed 64-row buffer, no alloc). Sentinel id
/// `usize::MAX` marks an unused row.
fn mk_rows() -> [Row; MAX_SLOTS] {
    core::array::from_fn(|_| Row { id: usize::MAX, slot: Slot::U8(0), off_start: 0 })
}


/// Find the first sentinel slot (caller counts rows with [`n_rows`]).
fn free_row(rows: &mut [Row; MAX_SLOTS]) -> Option<usize> {
    rows.iter().position(|r| r.id == usize::MAX)
}

/// Add an `i16` slot.
fn add_i16(rows: &mut [Row; MAX_SLOTS], id: usize, v: i16) {
    if let Some(i) = free_row(rows) {
        rows[i] = Row { id, slot: Slot::I16(v), off_start: 0 };
    }
}

/// Add an `i32` slot.
fn add_i32(rows: &mut [Row; MAX_SLOTS], id: usize, v: i32) {
    if let Some(i) = free_row(rows) {
        rows[i] = Row { id, slot: Slot::I32(v), off_start: 0 };
    }
}

/// Add an `i64` slot.
fn add_i64(rows: &mut [Row; MAX_SLOTS], id: usize, v: i64) {
    if let Some(i) = free_row(rows) {
        rows[i] = Row { id, slot: Slot::I64(v), off_start: 0 };
    }
}

/// Add a `u8`/bool slot.
fn add_u8(rows: &mut [Row; MAX_SLOTS], id: usize, v: u8) {
    if let Some(i) = free_row(rows) {
        rows[i] = Row { id, slot: Slot::U8(v), off_start: 0 };
    }
}

/// Add a forward-offset slot (patched inside [`Builder::table`]).
fn add_off(rows: &mut [Row; MAX_SLOTS], id: usize, off: usize) {
    if let Some(i) = free_row(rows) {
        rows[i] = Row { id, slot: Slot::Off(off), off_start: 0 };
    }
}

/// Children `[Field]` vector over already-encoded child field-table
/// offsets. Returns (vector offset, count).
fn children_vec(b: &mut Builder<'_>, offs: &[usize]) -> Result<(usize, usize), FbError> {
    let mut buf = [0usize; MAX_SLOTS];
    if offs.len() > MAX_SLOTS {
        return Err(FbError::FieldCap);
    }
    buf[..offs.len()].copy_from_slice(offs);
    let off = b.offset_vector(&buf, offs.len())?;
    Ok((off, offs.len()))
}

/// Encode one field (name/nullable/type-union/dictionary/children/
/// custom_metadata) -> field table offset.
fn encode_field(b: &mut Builder<'_>, f: &Fe<'_>, d: &mut Depth) -> Result<usize, FbError> {
    d.enter()?;
    let name = b.string(f.name)?;
    let (ty_off, ty_tag) = encode_ty(b, &f.ty, d)?;
    let meta_off = kv_vector(b, f.meta)?;
    let mut rows = mk_rows();
    add_off(&mut rows, 0, name);
    add_u8(&mut rows, 1, u8::from(f.nullable));
    add_u8(&mut rows, 2, ty_tag);
    add_off(&mut rows, 3, ty_off);
    if let Some(enc) = f.dict {
        // DictionaryEncoding: id:long(0), indexType:Field(1),
        // isSigned:bool(2), indexBitWidth:int(3).
        let mut irows = mk_rows();
        add_i64(&mut irows, 0, enc.id);
        add_u8(&mut irows, 2, u8::from(enc.index.signed()));
        add_i32(&mut irows, 3, i32::from(enc.index.bits()));
        let idx_field = encode_field(b, &Fe {
            name: "",
            nullable: false,
            ty: Te::Int {
                bits: match enc.index {
                    IndexTy::I8 | IndexTy::U8 => 8u8,
                    IndexTy::I16 | IndexTy::U16 => 16,
                    IndexTy::I32 | IndexTy::U32 => 32,
                    IndexTy::I64 | IndexTy::U64 => 64,
                },
                signed: enc.index.signed(),
            },
            dict: None,
            meta: &[],
        }, d)?;
        add_off(&mut irows, 1, idx_field);
        let dict = b.table(&mut irows)?;
        add_off(&mut rows, 4, dict);
    }
    if meta_off != 0 {
        add_off(&mut rows, 6, meta_off);
    }
    // DIV-7 (lane 7, recorded): the Field.children vector (slot 5) is
    // NOT emitted; the type table carries the children (single source).
    // The reader accepts either; pyarrow/arrow-rs goldens carry both and
    // read fine. smalt flatbuf.ex reads type-table children.
    let field = b.table(&mut rows)?;
    d.exit();
    Ok(field)
}

/// `[KeyValue]` custom_metadata vector -> offset (or 0 when empty).
fn kv_vector(b: &mut Builder<'_>, meta: &[(&str, &str)]) -> Result<usize, FbError> {
    if meta.is_empty() {
        return Ok(0);
    }
    let mut offs = [0usize; MAX_SLOTS];
    if meta.len() > MAX_SLOTS {
        return Err(FbError::FieldCap);
    }
    for (i, (k, v)) in meta.iter().enumerate() {
        let ko = b.string(k)?;
        let vo = b.string(v)?;
        let mut rows = mk_rows();
        add_off(&mut rows, 0, ko);
        add_off(&mut rows, 1, vo);
        offs[i] = b.table(&mut rows)?;
    }
    let (base, _) = children_vec(b, &offs[..meta.len()])?;
    Ok(base)
}

/// Encode a whole schema message: Message{version: V5, header_type: Schema,
/// header: schema, bodyLength, custom_metadata}. Returns the flatbuffer
/// length (multiple of 8). cgap:schema_blake3 / cgap:body_blake3 ancillary
/// keys ride the schema-level custom_metadata (ipc.rs law).
///
/// # Errors
/// [`FbError::NoRoom`], [`FbError::DepthCap`], [`FbError::FieldCap`].
pub fn emit_message(out: &mut [u8], s: &SchemaEnc<'_>, body_len: i64) -> Result<usize, FbError> {
    let mut b = Builder::new(out);
    let mut d = Depth::new();
    // Field tables first (they are the deepest objects).
    let mut offs = [0usize; MAX_SLOTS];
    if s.fields.len() > MAX_SLOTS {
        return Err(FbError::FieldCap);
    }
    for (i, f) in s.fields.iter().enumerate() {
        offs[i] = encode_field(&mut b, f, &mut d)?;
    }
    let fields_vec = children_vec(&mut b, &offs[..s.fields.len()])?;
    let schema_meta = kv_vector(&mut b, s.meta)?;
    let endianness: i16 = if s.little_endian { 0 } else { 1 };
    let mut srows = mk_rows();
    add_i16(&mut srows, 0, endianness);
    add_off(&mut srows, 1, fields_vec.0);
    add_off(&mut srows, 2, schema_meta);
    let schema = b.table(&mut srows)?;
    // Message{version(0), header_type(1), header(2), bodyLength(3)}.
    let mut mrows = mk_rows();
    add_i16(&mut mrows, 0, METADATA_VERSION_V5);
    add_u8(&mut mrows, 1, HEADER_TYPE_SCHEMA);
    add_off(&mut mrows, 2, schema);
    add_i64(&mut mrows, 3, body_len);
    let msg = b.table(&mut mrows)?;
    b.finish(msg)
}

/// `TimeUnit` to the flatbuffer short.
#[must_use]
pub fn unit_short(u: TimeUnit) -> i16 {
    match u {
        TimeUnit::Second => 0,
        TimeUnit::Milli => 1,
        TimeUnit::Micro => 2,
        TimeUnit::Nano => 3,
    }
}
