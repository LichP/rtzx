use binrw::{
    binrw,
    BinWrite,
};
use strum_macros::Display;
use std::borrow::Cow;
use std::fmt;
use std::io::Cursor;

use crate::tzx::tap::Payload;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
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
    padding: [u8; 228], // pad the rest of the 256-byte block
}

impl CPCHeader {
    fn filename(&self) -> Cow<'_, str> {
        let end = self.filename.iter().position(|&b| b == 0).unwrap_or(self.filename.len());
        return String::from_utf8_lossy(&self.filename[..end]);
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

#[binrw]
#[brw(little)]
#[br(import(payload_len: usize))]
#[derive(Debug, Clone)]
pub struct CPCData {
    #[br(count = payload_len)]
    pub data: Vec<u8>,
}

impl Payload for CPCData {
    fn bytes(&self) -> Vec<u8> { self.data.clone() }

    fn clone_box(&self) -> Box<dyn Payload> {
        Box::new(self.clone())
    }
}

impl fmt::Display for CPCData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPCData: {} bytes",
            self.data.len(),
        )
    }
}

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum CPCSync {
    CPCHeader = 0x2c,
    CPCData = 0x16,
}
