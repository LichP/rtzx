use binrw::{
    binrw,
};
use std::fmt;
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum BlockType {
    StandardSpeedDataBlock = 0x10,
    TurboSpeedDataBlock = 0x11,
    PureTone = 0x12,
    PulseSequence = 0x13,
    PureDataBlock = 0x14,
    DirectRecording = 0x15,
    C64ROMTypeDataBlock = 0x16,
    C64TurboTapeDataBlock = 0x17,
    CSWRecording = 0x18,
    GeneralizedDataBlock = 0x19,
    PauseOrStopTapeCommand = 0x20,
    GroupStart = 0x21,
    GroupEnd = 0x22,
    JumpToBlock = 0x23,
    LoopStart = 0x24,
    LoopEnd = 0x25,
    CallSequence = 0x26,
    ReturnFromSequence = 0x27,
    SelectBlock = 0x28,
    StopTapeIf48K = 0x2a,
    SetSignalLevel = 0x2b,
    TextDescription = 0x30,
    MessageBlock = 0x31,
    ArchiveInfo = 0x32,
    HardwareType = 0x33,
    EmulationInfo = 0x34,
    CustomInfoBlock = 0x35,
    SnapshotBlock = 0x40,
    GlueBlock = 0x5a,
}

impl fmt::LowerHex for BlockType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", (*self) as u8)
    }
}
