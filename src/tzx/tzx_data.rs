//! TZX data.

use binrw::{
    BinRead,
    BinWrite,
    BinResult,
    Error,
};
use std::io::{
    BufReader,
    ErrorKind,
    Read,
    Seek,
    Write,
};

use crate::tzx::{
    Header,
    RecoveryEnum,
};
use crate::tzx::blocks::{
    read_block,
    write_block,
    Block,
    BlockRef,
    BlockType,
};

/// Represents a parsed TZX/CDT data source.
#[derive(Clone, Debug, Default)]
pub struct TzxData {
    /// The TZX [Header].
    pub header: Header,
    /// The TZX [Block]s.
    pub blocks: Vec<Box<dyn Block + 'static>>
}

impl TzxData {
    pub fn new() -> Self {
        TzxData { header: Header::default(), blocks: Vec::new() }
    }

    /// Attempts to parse [TzxData] from the supplied reader.
    ///
    /// The data is expected to start with a [Header]. After the header, we process the remainder of the data in a
    /// loop, reading a block type identification byte and then attempting to parse the corresponding block data using
    /// [read_block].
    ///
    /// Once a block has successfully been parsed, the reader will be aligned to the end of the block ready to read
    /// the next block type identification byte. However, should a file be incorrectly formatted it is possible that
    /// the reader will become incorrectly aligned, resulting in further parse errors throughout the remainder of the
    /// file.
    ///
    /// Any such parse errors will not cause a panic unless they cause the parser to try and do something bad like
    /// reading beyond the end of the file. This will not happen with any validly formatted TZX/CDT files.
    pub fn read<R: Read + Seek>(reader: & mut R) -> Result<Self, Error> {
        TzxData::read_le(reader)
    }

    /// Writes [TzxData] to the supplied writer.
    pub fn write<W: Write + Seek>(&self, writer: & mut W) -> Result<(), Error> {
        self.write_le(writer)
    }
}

impl BinRead for TzxData {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        // Use a BufReader to handle underlying reads from the input.
        let mut reader = BufReader::new(reader);

        let header = Header::read(&mut reader)?;

        let mut blocks: Vec<Box<dyn Block + 'static>> = Vec::new();

        'parse_blocks: loop {
            let block_type_result = RecoveryEnum::<BlockType, u8>::read_le(&mut reader);
            if block_type_result.is_err() {
                match block_type_result.unwrap_err() {
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
            let block_type_recoverable = block_type_result.unwrap();

            let block_result: Result<Box<dyn Block>, Error> = read_block(block_type_recoverable, &mut reader);
            let block = match block_result {
                Err(why) => {
                    match block_type_recoverable {
                        RecoveryEnum::Known(block_type) => {
                            eprintln!("Failed to parse {} after block {}: {}", block_type, blocks.len(), why);
                        }
                        RecoveryEnum::Unknown(block_type_id) => {
                            eprintln!("Failed to parse undefined block type {} after block {}: {}", block_type_id, blocks.len(), why);
                        }
                    }
                    None
                },
                Ok(block) => Some(block),
            };

            if block.is_none() { continue }

            blocks.push(block.unwrap());
        }

        return Ok(TzxData {
            header,
            blocks
        });
    }
}

impl BinWrite for TzxData {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {

        self.header.write_le(writer)?;

        for block in self.blocks.iter() {
            if let Some(BlockRef::UndefinedBlockTypeBlock(b)) = block.as_block_ref() {
                b.block_type.write_le(writer)?;
            } else if let Some(BlockRef::UnsupportedBlockTypeBlock(b)) = block.as_block_ref() {
                b.block_type.write_le(writer)?;
            } else {
                block.r#type().write_le(writer)?;
            }

            write_block(block, writer)?;
        }

        Ok(())
    }
}
