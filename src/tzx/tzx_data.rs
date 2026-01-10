//! TZX data.

use binrw::{
    BinRead,
    Error,
};
use std::io::{
    BufReader,
    ErrorKind,
    Read,
    Seek,
};

use crate::tzx::{
    Header,
    RecoveryEnum,
};
use crate::tzx::blocks::{
    read_block,
    Block,
    BlockType,
};

/// Represents a parsed TZX/CDT data source.
#[derive(Clone, Debug)]
pub struct TzxData {
    /// The TZX [Header].
    pub header: Header,
    /// The TZX [Block]s.
    pub blocks: Vec<Box<dyn Block + 'static>>
}

impl TzxData {
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
    ///
    /// # Panics
    ///
    /// At present attempting to parse non-TZX data (i.e. failing to parse the [Header])
    /// or any IO error other than an unexpected EOF will cause a panic. This will change in a future version and we
    /// will return a Result instead.
    pub fn parse_from<R>(reader: R) -> TzxData where R: Read + Seek {
        // Use a BufReader to handle underlying reads from the file.
        let mut reader = BufReader::new(reader);

        let header = Header::read(&mut reader).expect("File not in TZX format");

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

        return TzxData {
            header,
            blocks
        }
    }
}
