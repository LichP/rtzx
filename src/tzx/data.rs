use binrw::{
    binrw,
    BinRead,
    BinResult,
    Error
};
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

#[binrw]
#[brw(little)]
#[derive(Clone)]
#[br(import(used_bits: u8))]
pub struct DataPayload {
    #[br(calc = used_bits)]
    pub used_bits: u8,

    #[br(parse_with = binrw::helpers::read_u24)]
    #[bw(write_with = binrw::helpers::write_u24)]
    pub length: u32,

    #[br(count = length, map = |v: Vec<u8>| Arc::new(v))]
    #[bw(map = |arc: &Arc<Vec<u8>>| &**arc)]
    pub data: Arc<Vec<u8>>,
    #[brw(ignore)]
    cached_bit_counts: OnceLock<BitCounts>,
}

#[derive(Copy, Clone)]
pub struct BitCounts {
    pub total: usize,
    pub ones: usize,
    pub zeros: usize,
}

impl DataPayload {
    pub fn new(used_bits: u8, length: u32, data: Arc<Vec<u8>>) -> Self {
        Self {
            used_bits,
            length,
            data,
            cached_bit_counts: OnceLock::new(),
        }
    }

    pub fn len(&self) -> usize { self.data.len() }

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


    #[inline]
    pub fn total_bits(&self) -> usize { (self.data.len() - 1) * 8 + self.used_bits as usize }
}

pub struct DataPayloadWithPosition {
    pub payload: DataPayload,
    pub current_byte_index: usize,
    pub current_bit_index: u8,
}

impl DataPayloadWithPosition {
    pub fn len(&self) -> usize { self.payload.len() }

    pub fn current_row_address(&self) -> usize { self.current_byte_index - (self.current_byte_index % 16) }

    pub fn current_row_end(&self) -> usize { if self.current_row_address() + 16 < self.len() { self.current_row_address() + 16 } else { self.len() }}

    pub fn current_row_bytes(&self) -> &[u8] { &self.payload.data[self.current_row_address()..self.current_row_end()] }
}

fn to_box_dyn<T>(block_result: BinResult<T>) -> Result<Box<dyn Payload>, Error>
where T: Payload + 'static
{
    block_result.map(|u| -> Box<dyn Payload> { Box::new(u) })
}
