//! Minimal protobuf wire-format reader.
//!
//! This is a deliberate dogfood of D12 rule 2: the decoder consumes the
//! field stream and simply IGNORES field numbers it does not model -
//! decode-and-skip is the iteration contract, not an error path.
//! Varint / 32-bit / 64-bit / length-delimited wire types only; groups
//! (wire types 3/4) are deprecated and rejected with a typed error.

use crate::error::CodegenError;

/// Protobuf wire types (canonical table, stable since the format shipped).
pub const WT_VARINT: u32 = 0;
pub const WT_FIXED64: u32 = 1;
pub const WT_LEN: u32 = 2;
pub const WT_FIXED32: u32 = 5;

/// A decoded wire value, already past its tag.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WireValue {
    Varint(u64),
    Fixed32(u32),
    Fixed64(u64),
    /// Borrowed sub-buffer (message / string / bytes payload).
    Len(Vec<u8>),
}

impl WireValue {
    pub fn as_varint(&self) -> Result<u64, CodegenError> {
        match self {
            WireValue::Varint(v) => Ok(*v),
            other => Err(CodegenError::wire(format!(
                "expected varint, got {other:?}"
            ))),
        }
    }

    pub fn as_len_bytes(&self) -> Result<&[u8], CodegenError> {
        match self {
            WireValue::Len(b) => Ok(b),
            other => Err(CodegenError::wire(format!(
                "expected length-delimited, got {other:?}"
            ))),
        }
    }

    pub fn as_len_str(&self) -> Result<&str, CodegenError> {
        std::str::from_utf8(self.as_len_bytes()?).map_err(|e| CodegenError::wire(e.to_string()))
    }
}

/// Cursor over one encoded buffer.
pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.buf.len()
    }

    /// Base-128 varint, max 10 bytes (u64).
    pub fn read_varint(&mut self) -> Result<u64, CodegenError> {
        let mut out: u64 = 0;
        for byte_index in 0..10u32 {
            let byte = *self
                .buf
                .get(self.pos)
                .ok_or_else(|| CodegenError::wire("truncated varint"))?;
            self.pos += 1;
            let shifted = u64::from(byte & 0x7f)
                .checked_shl(7 * byte_index)
                .ok_or_else(|| CodegenError::wire("varint overflows u64"))?;
            out |= shifted;
            if byte & 0x80 == 0 {
                return Ok(out);
            }
        }
        Err(CodegenError::wire("varint longer than 10 bytes"))
    }

    /// Little-endian fixed width (4 or 8 bytes).
    pub fn read_fixed(&mut self, width: usize) -> Result<&'a [u8], CodegenError> {
        if width != 4 && width != 8 {
            return Err(CodegenError::wire(format!("bad fixed width {width}")));
        }
        let end = self
            .pos
            .checked_add(width)
            .ok_or_else(|| CodegenError::wire("fixed width overflow"))?;
        let slice = self
            .buf
            .get(self.pos..end)
            .ok_or_else(|| CodegenError::wire("truncated fixed field"))?;
        self.pos = end;
        Ok(slice)
    }

    /// Length-delimited payload (length is a varint, then that many bytes).
    pub fn read_len_bytes(&mut self) -> Result<&'a [u8], CodegenError> {
        let len = self.read_varint()? as usize;
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| CodegenError::wire("length overflow"))?;
        let slice = self.buf.get(self.pos..end).ok_or_else(|| {
            CodegenError::wire(format!(
                "truncated length-delimited field: need {len} bytes at {}",
                self.pos
            ))
        })?;
        self.pos = end;
        Ok(slice)
    }
}

/// Field-level cursor yielding `(field number, value)` pairs in wire order.
/// Unknown numbers are consumed AND dropped by the iterator itself - a
/// caller's `match` without an arm for them is the decode-and-skip rule.
pub struct FieldCursor<'a, 'r> {
    r: &'r mut Reader<'a>,
}

impl<'a, 'r> FieldCursor<'a, 'r> {
    pub fn new(r: &'r mut Reader<'a>) -> Self {
        Self { r }
    }

    /// Next `(field_number, value)`, or `None` at buffer end.
    pub fn next_field(&mut self) -> Result<Option<(u32, WireValue)>, CodegenError> {
        if self.r.is_empty() {
            return Ok(None);
        }
        let tag = self.r.read_varint()?;
        let number = u32::try_from(tag >> 3)
            .map_err(|_| CodegenError::wire(format!("field number out of u32 range: {tag}")))?;
        if number == 0 {
            return Err(CodegenError::wire("field number 0 is illegal"));
        }
        let wire = (tag & 0x7) as u32;
        let value = match wire {
            WT_VARINT => WireValue::Varint(self.r.read_varint()?),
            WT_FIXED64 => {
                let b = self.r.read_fixed(8)?;
                WireValue::Fixed64(u64::from_le_bytes(
                    b.try_into()
                        .map_err(|_| CodegenError::wire("fixed64 len"))?,
                ))
            }
            WT_LEN => WireValue::Len(self.r.read_len_bytes()?.to_vec()),
            WT_FIXED32 => {
                let b = self.r.read_fixed(4)?;
                WireValue::Fixed32(u32::from_le_bytes(
                    b.try_into()
                        .map_err(|_| CodegenError::wire("fixed32 len"))?,
                ))
            }
            3 | 4 => return Err(CodegenError::GroupUnsupported(number)),
            other => {
                return Err(CodegenError::wire(format!(
                    "unknown wire type {other} at field {number}"
                )));
            }
        };
        Ok(Some((number, value)))
    }
}
