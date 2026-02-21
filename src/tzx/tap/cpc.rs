use binrw::{
    binrw,
    BinWrite,
};
use num_enum::TryFromPrimitive;
use strum_macros::Display;
use std::any::Any;
use std::borrow::Cow;
use std::fmt;
use std::io::Cursor;
use std::sync::Arc;

use crate::tzx::{
    blocks::{Block, TurboSpeedDataBlock},
    data::DataPayload,
    tap::{CrcPagedRW, Payload},
};

/// A standard Amstrad CPC tape block header.
///
/// CPC header blocks are 28 bytes long, and then padded with zeroes to 256 bytes and appended with a two byte checksum
/// when encoded to tape. When loaded from a tap file, the checksum bytes and padding are omitted. When loading from a
/// CDT block [DataPayload], use [CrcPagedRW](crate::tzx::tap::CrcPagedRW) to handle the checksum validation / calculation
/// and padding.
#[binrw]
#[brw(little)]
#[brw(import(to_from_tap: bool))]
#[derive(Debug, Clone, Hash)]
pub struct CPCHeader {
    #[br(count = 16)]
    filename: Vec<u8>,
    block_number: u8,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |x: &bool| if *x { 1 } else { 0 })]
    last_block: bool,
    file_type: u8,
    data_length: u16,
    data_location: u16,
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |x: &bool| if *x { 1 } else { 0 })]
    first_block: bool,
    logical_length: u16,
    entry_address: u16,
    #[br(if(!to_from_tap, [0; 228]))]
    padding: [u8; 228], // pad the rest of the 256-byte block
}

impl CPCHeader {
    fn filename(&self) -> Cow<'_, str> {
        let end = self.filename.iter().position(|&b| b == 0).unwrap_or(self.filename.len());
        return String::from_utf8_lossy(&self.filename[..end]);
    }
}

impl CPCHeader {
    pub fn new(
        filename: &str,
        block_number: u8,
        last_block: bool,
        file_type: u8,
        data_length: u16,
        data_location: u16,
        first_block: bool,
        logical_length: u16,
        entry_address: u16
    ) -> Self {
        CPCHeader {
            filename: filename.as_bytes().to_vec(),
            block_number,
            last_block,
            file_type,
            data_length,
            data_location,
            first_block,
            logical_length,
            entry_address,
            padding: [0; 228],
        }
    }

    pub fn into_turbo_speed_data_block(&self) -> TurboSpeedDataBlock {
        let mut tsdb = TurboSpeedDataBlock::new();
        tsdb.pause = 15;
        tsdb.payload = self.into();
        tsdb
    }
}

impl Default for CPCHeader {
    fn default() -> Self {
        CPCHeader::new("", 1, true, 1, 0, 0, true, 0, 0)
    }
}

impl From<&CPCHeader> for DataPayload {
    fn from(value: &CPCHeader) -> Self
    {
        let mut writer = Cursor::new(Vec::new());
        value.flag_byte().write(&mut writer).unwrap();
        let mut crc_writer = CrcPagedRW::new(writer, 1, 256);
        value.write(&mut crc_writer).unwrap();
        writer = crc_writer.into_inner();
        (0xffffffff as u32).write_le(&mut writer).unwrap();
        let encoded = writer.into_inner();
        return Self::new(8, Arc::new(encoded));
    }
}

impl Payload for CPCHeader {
    fn bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        self.write(&mut writer).unwrap();
        let mut encoded = writer.into_inner();
        encoded.truncate(28);
        return encoded;
    }

    fn clone_box(&self) -> Box<dyn Payload> {
        Box::new(self.clone())
    }

    fn flag_byte(&self) -> Option<u8> { Some(CPCFlag::CPCHeader as u8) }

    fn into_block_box(self: Box<Self>) -> Box<dyn Block> {
        Box::new((*self).into_turbo_speed_data_block())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl fmt::Display for CPCHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPCHeader: {:16} block {} (type: {}; len: {}/{}; loc: {:04x}; ent: {:04x})",
            self.filename(),
            self.block_number,
            self.file_type,
            self.data_length,
            self.logical_length,
            self.data_location,
            self.entry_address,
        )
    }
}

/// A block of CPC data.
///
/// On tape this is padded with zeros to be a multiple of 256 bytes long, and two bytes of checksum are included every 256 bytes.
/// Checksum values are not included in the data here, but zero padding is included. Checksums should be handled using
/// [CrcPagedRW](crate::tzx::tap::CrcPagedRW).
#[binrw]
#[brw(little)]
#[br(import(payload_len: usize))]
#[derive(Debug, Clone, Hash)]
pub struct CPCData {
    #[br(count = payload_len)]
    pub data: Vec<u8>,
}

impl CPCData {
    pub fn new(data: Vec<u8>) -> Self { CPCData { data } }

    pub fn into_turbo_speed_data_block(&self) -> TurboSpeedDataBlock {
        let mut tsdb = TurboSpeedDataBlock::new();
        tsdb.payload = self.into();
        tsdb
    }
}

impl Default for CPCData {
    fn default() -> Self { CPCData::new(Vec::new()) }
}

impl From<&CPCData> for DataPayload {
    fn from(value: &CPCData) -> Self
    {
        let mut writer = Cursor::new(Vec::new());
        value.flag_byte().write(&mut writer).unwrap();
        let mut crc_writer = CrcPagedRW::new(writer, 1, 256);
        value.write(&mut crc_writer).unwrap();
        writer = crc_writer.into_inner();
        (0xffffffff as u32).write_le(&mut writer).unwrap();
        let encoded = writer.into_inner();
        return Self::new(8, Arc::new(encoded));
    }
}

impl Payload for CPCData {
    fn bytes(&self) -> Vec<u8> { self.data.clone() }

    fn clone_box(&self) -> Box<dyn Payload> {
        Box::new(self.clone())
    }

    fn flag_byte(&self) -> Option<u8> { Some(CPCFlag::CPCData as u8) }

    fn into_block_box(self: Box<Self>) -> Box<dyn Block> {
        Box::new((*self).into_turbo_speed_data_block())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl fmt::Display for CPCData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPCData: {} bytes",
            self.data.len(),
        )
    }
}

/// Flag byte indicating whether a payload contains a CPC header or data .
#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug, Default, Eq, PartialEq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum CPCFlag {
    #[default]
    CPCHeader = 0x2c,
    CPCData = 0x16,
}
