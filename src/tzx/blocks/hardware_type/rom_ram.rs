use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum RomRamType {
    SamRam = 0x00,
    MultifaceONE = 0x01,
    Multiface128k = 0x02,
    MultifacePlus3 = 0x03,
    MultiPrint = 0x04,
    MB02ROMRAMExpansion = 0x05,
    SoftROM = 0x06,
    Ram1k = 0x07,
    Ram16k = 0x08,
    Ram48k = 0x09,
    MemoryIn8To16kUsed = 0x10,
}
