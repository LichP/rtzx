//! TZX block payload data.

use binrw::{
    binrw,
    BinRead,
    BinResult,
    Error
};

use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::ops::Range;
use std::sync::{Arc, OnceLock};

use crate::tzx::tap::{
    CPCSync,
    CPCData,
    CPCHeader,
    CrcPagedRW,
    Payload,
};

/// A struct representing the numbers of bits in a [DataPayload].
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct BitCounts {
    /// The total number of bits in the payload.
    pub total: usize,
    /// The number of bits with value 1.
    pub ones: usize,
    /// The number of bits with value 0.
    pub zeros: usize,
}

impl fmt::Display for BitCounts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0 + 1: {} + {} = {}", self.zeros, self.ones, self.total)
    }
}

/// A data payload encoded by a TZX block.
///
/// This data structure wraps an underlying byte vector to provide methods to help with with bit counting.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
#[br(import(used_bits: u8))]
pub struct DataPayload {
    /// The number of used bits in the last byte of [data](DataPayload::data).
    /// When a waveform is generated for this payload, pulses are generated for the most significant bits
    /// of the ast byte up to this number, and subsequent bits in the byte are ignored.
    #[br(calc = used_bits)]
    pub used_bits: u8,

    /// The length of the data in bytes.
    #[br(parse_with = binrw::helpers::read_u24)]
    #[bw(write_with = binrw::helpers::write_u24)]
    pub length: u32,

    /// The data as a vector of bytes. We wrap the data in an [Arc] to allow for efficient cloning.
    #[br(count = length, map = |v: Vec<u8>| Arc::new(v))]
    #[bw(map = |arc: &Arc<Vec<u8>>| &**arc)]
    pub data: Arc<Vec<u8>>,
    #[brw(ignore)]
    cached_bit_counts: OnceLock<BitCounts>,
}

impl Hash for DataPayload {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
        self.used_bits.hash(state);
    }
}

impl DataPayload {
    /// Creates a new [DataPayload].
    pub fn new(used_bits: u8, length: u32, data: Arc<Vec<u8>>) -> Self {
        Self {
            used_bits,
            length,
            data,
            cached_bit_counts: OnceLock::new(),
        }
    }

    /// Returns the length of the data in bytes, including unused bits.
    pub fn len(&self) -> usize { self.data.len() }

    /// Returns bit counts for the data.
    pub fn bit_counts(&self) -> &BitCounts {
        self.cached_bit_counts.get_or_init(|| self.compute_bit_counts())
    }

    fn compute_bit_counts(&self) -> BitCounts {
        let total = self.total_bits();
        let ones = popcnt::count_ones(self.data.as_slice()) as usize;
        return BitCounts {
            total,
            ones,
            zeros: total - ones,
        };
    }

    /// Returns bit counts for the data over the supplied range.
    pub fn bit_counts_for_range(&self, range: Range<usize>) -> Result<BitCounts, &'static str> {
        if range.end >= self.data.len() {
            return Err("range out of bounds");
        }

        let total = if range.end == self.data.len() {
            (range.len() - 1) * 8 + self.used_bits as usize
        } else {
            range.len() * 8
        };
        let ones = popcnt::count_ones(&self.data[range]) as usize;
        return Ok(BitCounts {
            total,
            ones,
            zeros: total - ones,
        })
    }

    /// Attempts to parse the data as a known payload type, such as a [CPCHeader] or [CPCData] payload.
    /// Returns `Some(Box<dyn Payload>)` if the data can be so parsed, and `None` if not.
    pub fn read_payload(&self) -> Option<Box<dyn Payload>> {
        // Only attempt to parse if data length matches:
        // Sync (1) + (256 + CheckSum (2) = 258) * x + Trailer (4) = 258x + 5
        if self.len() < 5 || (self.len() - 5) % 258 != 0 { return None }
        let payload_len = (self.len() - 5) / 258 * 256;

        let mut reader = Cursor::new(&self.data[..]);
        let sync = CPCSync::read(&mut reader);
        if !sync.is_ok() { return None }

        let mut crc_reader = CrcPagedRW::new(reader, 256);

        return match sync.unwrap() {
            CPCSync::CPCHeader => { to_box_dyn(CPCHeader::read(&mut crc_reader)).ok() }
            CPCSync::CPCData => { to_box_dyn(CPCData::read_args(&mut crc_reader, (payload_len,))).ok() }
        }

        // Technically we should parse the trailer for CPC blocks - do we care?
        // // read trailer (always 4×0xFF)
        // let mut trailer = [0xFFu8; 4];
        // reader.read_exact(&mut trailer);
        // if trailer != [0xFFu8; 4] { return None }
    }

    /// Returns the total number of bits in the data, excluding unused bits in the last byte.
    #[inline]
    pub fn total_bits(&self) -> usize { (self.data.len() - 1) * 8 + self.used_bits as usize }
}

impl Default for DataPayload {
    fn default() -> Self { DataPayload::new(0, 0, Arc::new(Vec::new())) }
}

impl fmt::Display for DataPayload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DataPayload: {} bytes (used_bits: {}; total_bits: {})", self.data.len(), self.used_bits, self.total_bits())
    }
}

/// Wraps [DataPayload] with current byte and bit index positions
/// Provide helper methods for viewing data relating to the current position in the payload.
#[derive(Clone, Debug, Hash)]
pub struct DataPayloadWithPosition {
    pub payload: DataPayload,
    pub current_byte_index: usize,
    pub current_bit_index: u8,
}

impl DataPayloadWithPosition {
    /// Wraps [DataPayload::len].
    pub fn len(&self) -> usize { self.payload.len() }

    /// Returns the index of the byte at the start of the row of 16 bytes the current byte is in.
    pub fn current_row_address(&self) -> usize { self.current_byte_index - (self.current_byte_index % 16) }

    /// Returns the index of the byte at the start of the subsequent row, or the length of the data if the
    /// current position is in the last row.
    pub fn current_row_end(&self) -> usize { if self.current_row_address() + 16 < self.len() { self.current_row_address() + 16 } else { self.len() }}

    /// Returns a slice of the data for the row corresponding to the current position.
    pub fn current_row_bytes(&self) -> &[u8] { &self.payload.data[self.current_row_address()..self.current_row_end()] }
}

impl fmt::Display for DataPayloadWithPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DataPayloadWithPosition: {} bytes (used_bits: {}; total_bits: {}; current: {}.{})", self.payload.len(), self.payload.used_bits, self.payload.total_bits(), self.current_byte_index, self.current_bit_index)
    }
}

fn to_box_dyn<T>(block_result: BinResult<T>) -> Result<Box<dyn Payload>, Error>
where T: Payload + 'static
{
    block_result.map(|u| -> Box<dyn Payload> { Box::new(u) })
}
