//! Data payloads corresponding to known platform encodings, such as defined in TAP files.

pub mod cpc;
pub mod crc_reader;
pub mod spectrum;
pub mod xor_reader_writer;

pub use cpc::{CPCData, CPCHeader, CPCFlag};
pub use crc_reader::CrcPagedRW;
pub use spectrum::{SpectrumData, SpectrumHeader, SpectrumFlag};
pub use xor_reader_writer::{XorReader, XorWriter};

use binrw::{
    BinRead,
    BinWrite,
    BinResult,
    Error,
};
use std::any::Any;
use std::fmt;
use std::io::{
    BufReader,
    ErrorKind,
    Read,
    Seek,
    Write,
};

use crate::{
    TzxData,
    tzx::blocks::Block,
};

/// A payload corresponding to a known platform encoding.
pub trait Payload: std::fmt::Display + Any {
    /// Returns the bytes representing this payload, excluding any padding and checksums.
    fn bytes(&self) -> Vec<u8>;

    fn clone_box(&self) -> Box<dyn Payload>;

    fn flag_byte(&self) -> Option<u8> { None }

    fn into_block_box(self: Box<Self>) -> Box<dyn Block>;

    // Required for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl Clone for Box<dyn Payload> {
    fn clone(&self) -> Box<dyn Payload> {
        self.clone_box()
    }
}

impl fmt::Debug for Box<dyn Payload> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct PayloadError(String);

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<binrw::Error> for PayloadError {
    fn from(e: binrw::Error) -> Self {
        PayloadError(format!("binrw::Error: {:?}", e))
    }
}

impl From<std::io::Error> for PayloadError {
    fn from(e: std::io::Error) -> Self {
        PayloadError(format!("io::Error: {:?}", e))
    }
}

macro_rules! payload_ref_enum {
    (
        $(
            $ty:ty => $variant:ident ( args: $args:tt )
        ),* $(,)?
    ) => {
        pub enum PayloadRef<'a> {
            $( $variant(&'a $ty), )*
        }

        impl dyn Payload {
            pub fn as_payload_ref(&self) -> Option<PayloadRef<'_>> {
                $(
                    if let Some(b) = self.as_any().downcast_ref::<$ty>() {
                        return Some(PayloadRef::$variant(b));
                    }
                )*
                None
            }
        }
    };

    // --- dispatch rules ---

    // no args
    (@write $b:ident $writer:ident $to_tap:ident ()) => {
        $b.write_le($writer)
    };

    // uses to_tap
    (@write $b:ident $writer:ident $to_tap:ident (to_tap)) => {
        $b.write_le_args($writer, ($to_tap,))
    };
}

payload_ref_enum! {
    CPCData => CPCData (args: (to_tap)),
    CPCHeader => CPCHeader (args: (to_tap)),
    SpectrumData => SpectrumData (args: ()),
    SpectrumHeader => SpectrumHeader (args: ()),
}

pub fn read_payload(length: usize, from_tap: bool, mut reader: impl Read + Seek) -> Result<Box<dyn Payload>, PayloadError> {
    let flag_byte = u8::read_le(&mut reader)?;

    // Attempt to parse as a CPC payload:
    if let Ok(cpc_flag) = CPCFlag::try_from(flag_byte) {
        if from_tap {
            return match cpc_flag {
                CPCFlag::CPCHeader => { to_box_dyn(CPCHeader::read_args(&mut reader, (true,))) }
                CPCFlag::CPCData => { to_box_dyn(CPCData::read_args(&mut reader, (length - 1,))) }
            }
        }

        // Only attempt to parse if data length matches:
        // Flag (1) + (256 + CheckSum (2) = 258) * x + Trailer (4) = 258x + 5
        if length < 5 || (length - 5) % 258 != 0 {
            return Err(PayloadError("Bad length".to_string()));
        }
        let payload_len = (length - 5) / 258 * 256;

        let stream_position = reader.stream_position()?;
        let mut crc_reader = CrcPagedRW::new(reader, stream_position, 256);

        return match cpc_flag {
            CPCFlag::CPCHeader => { to_box_dyn(CPCHeader::read_args(&mut crc_reader, (false,))) }
            CPCFlag::CPCData => { to_box_dyn(CPCData::read_args(&mut crc_reader, (payload_len,))) }
        }

        // Technically we should parse the trailer for CPC blocks - do we care?
        // // read trailer (always 4×0xFF)
        // let mut trailer = [0xFFu8; 4];
        // reader.read_exact(&mut trailer);
        // if trailer != [0xFFu8; 4] { return None }
    }

    if let Ok(spectrum_flag) = SpectrumFlag::try_from(flag_byte) {
        // Only attempt to parse if data length greater than two bytes (flag + checksum)
        if length < 3 {
            return Err(PayloadError("Bad length".to_string()))
        }

        return match spectrum_flag {
            SpectrumFlag::SpectrumHeader => { to_box_dyn(SpectrumHeader::read(&mut reader)) }
            SpectrumFlag::SpectrumData => { to_box_dyn(SpectrumData::read_args(&mut reader, (length - 2 as usize,))) }
        }
    }

    // No flag matched
    return Err(PayloadError(format!("Unrecognized flag byte: {:02X}", flag_byte)));
}

/// Attempts to write a payload as TAP data to the writer.
pub fn write_payload<W: Write + Seek>(payload: &Box<dyn Payload>, to_tap: bool, writer: &mut W) -> BinResult<()> {
    match payload.as_payload_ref() {
        Some(PayloadRef::CPCData(p)) => {
            let stream_position = writer.stream_position()?;
            let mut crc_writer = CrcPagedRW::new(writer, stream_position, 256);
            return p.write_le(&mut crc_writer);
        },
        Some(PayloadRef::CPCHeader(p)) => { p.write_le_args(writer, (to_tap,)) },
        Some(PayloadRef::SpectrumData(p)) => { p.write_le(writer) },
        Some(PayloadRef::SpectrumHeader(p)) => { p.write_le(writer) },
        None => Ok(()),
    }
}


/// Represents a parsed TAP data source.
#[derive(Clone, Debug, Default)]
pub struct TapData {
    /// The TAP block [Payload]s.
    pub blocks: Vec<Box<dyn Payload + 'static>>
}

impl TapData {
    pub fn new() -> Self {
        TapData { blocks: Vec::new() }
    }

    /// Attempts to parse [TapData] from the supplied reader.
    ///
    /// We process the data in a loop, reading a block length and flag byte and attempting to pass the next chunk of
    /// data as a corresponding block payload type.
    ///
    /// block type identification byte and then attempting to parse the corresponding block data using
    /// [read_block].
    ///
    /// Once a block has successfully been parsed, the reader will be aligned to the end of the block ready to read
    /// the next length and flag byte. However, should a file be incorrectly formatted it is possible that the reader
    /// will become incorrectly aligned, resulting in further parse errors throughout the remainder of the file.
    ///
    /// Any such parse errors will not cause a panic unless they cause the parser to try and do something bad like
    /// reading beyond the end of the file.
    pub fn read<R: Read + Seek>(reader: & mut R) -> Result<Self, Error> {
        TapData::read_le(reader)
    }

    // /// Writes [TapData] to the supplied writer.
    // pub fn write<W: Write + Seek>(&self, writer: & mut W) -> Result<(), Error> {
    //     self.write_le(writer)
    // }
}

impl BinRead for TapData {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        // Use a BufReader to handle underlying reads from the input.
        let mut reader = BufReader::new(reader);

        let mut blocks: Vec<Box<dyn Payload + 'static>> = Vec::new();

        'parse_blocks: loop {
            let block_length_result = u16::read_le(&mut reader);

            if block_length_result.is_err() {
                match block_length_result.unwrap_err() {
                    Error::Io(why) => match why.kind() {
                        ErrorKind::UnexpectedEof => break 'parse_blocks,
                        _ => panic!("IO error: {}", why),
                    }
                    other => eprintln!("Unhandled error: {}", other),
                }
                if reader.seek_relative(1).is_ok() {
                    continue 'parse_blocks;
                } else {
                    break 'parse_blocks;
                };
            }
            let block_length = block_length_result.unwrap();

            let payload_result: Result<Box<dyn Payload>, PayloadError> = read_payload(block_length as usize, true, &mut reader);
            let block = match payload_result {
                Err(why) => {
                    reader.seek(std::io::SeekFrom::Current(block_length as i64 - 1)).unwrap();
                    eprintln!("Failed to parse after block {}: {:?}", blocks.len(), why);
                    None
                },
                Ok(payload) => Some(payload),
            };

            if block.is_none() { continue }

            blocks.push(block.unwrap());
        }

        return Ok(TapData {
            blocks
        });
    }
}

impl BinWrite for TapData {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {

        for block in self.blocks.iter() {
            // Write length
            let length = block.bytes().len();
            (length as u16).write_le(writer)?;
            if let Some(flag_byte) = block.flag_byte() {
                flag_byte.write_le(writer)?;
            }

            write_payload(block, true, writer)?;
        }

        Ok(())
    }
}

impl From<TapData> for TzxData {
    fn from(value: TapData) -> Self {
        let mut tzx_data = Self::default();
        for tap_block in value.blocks {
            tzx_data.blocks.push(tap_block.into_block_box())
        }
        return tzx_data;
    }
}

fn to_box_dyn<T>(payload_result: BinResult<T>) -> Result<Box<dyn Payload>, PayloadError>
where T: Payload + 'static
{
    match payload_result {
        Ok(p) => Ok(Box::new(p)),
        Err(e) => Err(PayloadError::from(e)),
    }
}
