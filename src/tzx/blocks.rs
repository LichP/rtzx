pub mod archive_info;
pub mod block_type;
pub mod call;
pub mod custom_info_block;
pub mod direct_recording;
pub mod emulation_info;
pub mod generalized_data_block;
pub mod group;
pub mod hardware_type;
pub mod jump_to_block;
pub mod r#loop;
pub mod pause_or_stop_tape_command;
pub mod pulse_sequence;
pub mod pure_data_block;
pub mod pure_tone;
pub mod select_block;
pub mod set_signal_level;
pub mod snapshot_block;
pub mod standard_speed_data_block;
pub mod turbo_speed_data_block;
pub mod text_description;

pub use archive_info::{ArchiveInfo, ArchiveInfoEntry, ArchiveInfoEntryType};
pub use block_type::BlockType;
pub use call::{CallSequence, ReturnFromSequence};
pub use custom_info_block::{CustomInfoBlock, InstructionsBlock};
pub use direct_recording::DirectRecording;
pub use emulation_info::EmulationInfo;
pub use generalized_data_block::GeneralizedDataBlock;
pub use group::{GroupStart, GroupEnd};
pub use hardware_type::{HardwareTypeBlock, HardwareTypeBlockEntry};
pub use jump_to_block::JumpToBlock;
pub use r#loop::{LoopStart, LoopEnd};
pub use pause_or_stop_tape_command::{PauseOrStopTapeCommand, StopTapeIf48K};
pub use pulse_sequence::PulseSequence;
pub use pure_data_block::PureDataBlock;
pub use pure_tone::PureTone;
pub use select_block::{SelectBlock, SelectBlockEntry};
pub use set_signal_level::SetSignalLevel;
pub use snapshot_block::SnapshotBlock;
pub use standard_speed_data_block::StandardSpeedDataBlock;
pub use turbo_speed_data_block::TurboSpeedDataBlock;
pub use text_description::{TextDescription, MessageBlock};

use crate::tzx::{
    Config,
    ExtendedDisplayCollector,
    RecoveryEnum,
    waveforms::{EmptyWaveform, Waveform}
};

use binrw::{
    binrw,
    BinRead,
    BinResult,
    Error
};
use std::fmt;
use std::io::{
    Read,
    Seek,
};
use std::sync::Arc;

pub trait Block: std::fmt::Display {
    fn r#type(&self) -> BlockType;

    fn get_waveforms<'a>(&self, config: Arc<Config>, _start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let empty_source = EmptyWaveform::new(config.clone());
        return vec![Box::new(empty_source)];
    }

    fn next_block_start_pulse_high(&self, _config: Arc<Config>, self_start_pulse_high: bool) -> bool { self_start_pulse_high }

    fn clone_box(&self) -> Box<dyn Block>;

    fn extended_display(&self, _out: &mut dyn ExtendedDisplayCollector) {}
}

impl Clone for Box<dyn Block> {
    fn clone(&self) -> Box<dyn Block> {
        self.clone_box()
    }
}

#[derive(BinRead, Clone)]
#[br(little)]
#[br(import(block_type_id: u8))]
pub struct UndefinedBlockTypeBlock {
    #[br(calc = block_type_id)]
    pub block_type: u8,
    pub length: u32,
    #[br(count = length)]
    pub payload: Vec<u8>
}

impl fmt::Display for UndefinedBlockTypeBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UndefinedBlockTypeBlock: {0} ({0:#x}), {1:5} bytes", self.block_type, self.length)
    }
}

impl Block for UndefinedBlockTypeBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::Undefined;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

#[derive(BinRead, Clone)]
#[br(little)]
#[br(import(block_type: BlockType))]
pub struct UnsupportedBlockTypeBlock {
    #[br(calc = block_type)]
    pub block_type: BlockType,
    pub length: u32,
    #[br(count = length)]
    pub payload: Vec<u8>
}

impl fmt::Display for UnsupportedBlockTypeBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UnsupportedBlockTypeBlock: {0} ({0:#x}), {1:5} bytes", self.block_type, self.length)
    }
}

impl Block for UnsupportedBlockTypeBlock {
    fn r#type(&self) -> BlockType {
        return self.block_type;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
#[binrw]
#[brw(little, magic = b"XTape!\x1A")]
pub struct GlueBlock {
    major: u8,
    minor: u8
}

impl fmt::Display for GlueBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GlueBlock: TZX version {}.{}", self.major, self.minor)
    }
}

impl Block for GlueBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::GlueBlock;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

pub fn read_block(block_type: RecoveryEnum<BlockType, u8>, mut reader: impl Read + Seek) -> Result<Box<dyn Block>, Error> {
    return match block_type {
        RecoveryEnum::Known(block_type_known) => match block_type_known {
            BlockType::StandardSpeedDataBlock => to_box_dyn(StandardSpeedDataBlock::read(&mut reader)),
            BlockType::TurboSpeedDataBlock => to_box_dyn(TurboSpeedDataBlock::read(&mut reader)),
            BlockType::PureTone => to_box_dyn(PureTone::read(&mut reader)),
            BlockType::PulseSequence => to_box_dyn(PulseSequence::read(&mut reader)),
            BlockType::PureDataBlock => to_box_dyn(PureDataBlock::read(&mut reader)),
            BlockType::DirectRecording => to_box_dyn(DirectRecording::read(&mut reader)),
            BlockType::GeneralizedDataBlock => to_box_dyn(GeneralizedDataBlock::read(&mut reader)).or_else(|_| to_box_dyn(UnsupportedBlockTypeBlock::read_args(&mut reader,(block_type_known,)))),
            BlockType::PauseOrStopTapeCommand => to_box_dyn(PauseOrStopTapeCommand::read(&mut reader)),
            BlockType::GroupStart => to_box_dyn(GroupStart::read(&mut reader)),
            BlockType::GroupEnd => to_box_dyn(GroupEnd::read(&mut reader)),
            BlockType::JumpToBlock => to_box_dyn(JumpToBlock::read(&mut reader)),
            BlockType::LoopStart => to_box_dyn(LoopStart::read(&mut reader)),
            BlockType::LoopEnd => to_box_dyn(LoopEnd::read(&mut reader)),
            BlockType::CallSequence => to_box_dyn(CallSequence::read(&mut reader)),
            BlockType::ReturnFromSequence => to_box_dyn(ReturnFromSequence::read(&mut reader)),
            BlockType::SelectBlock => to_box_dyn(SelectBlock::read(&mut reader)),
            BlockType::StopTapeIf48K => to_box_dyn(StopTapeIf48K::read(&mut reader)),
            BlockType::SetSignalLevel => to_box_dyn(SetSignalLevel::read(&mut reader)),
            BlockType::TextDescription => to_box_dyn(TextDescription::read(&mut reader)),
            BlockType::MessageBlock => to_box_dyn(MessageBlock::read(&mut reader)),
            BlockType::ArchiveInfo => to_box_dyn(ArchiveInfo::read(&mut reader)),
            BlockType::HardwareType => to_box_dyn(HardwareTypeBlock::read(&mut reader)),
            BlockType::EmulationInfo => to_box_dyn(EmulationInfo::read(&mut reader)),
            BlockType::CustomInfoBlock => to_box_dyn(CustomInfoBlock::read(&mut reader)),
            BlockType::SnapshotBlock => to_box_dyn(SnapshotBlock::read(&mut reader)),
            BlockType::InstructionsBlock => to_box_dyn(InstructionsBlock::read(&mut reader)),
            BlockType::GlueBlock => to_box_dyn(GlueBlock::read(&mut reader)),
            BlockType::Undefined => to_box_dyn(UndefinedBlockTypeBlock::read_args(&mut reader,(0xff,))),
            _ => to_box_dyn(UnsupportedBlockTypeBlock::read_args(&mut reader,(block_type_known,))),
        },
        RecoveryEnum::Unknown(block_type_unknown) => to_box_dyn(UndefinedBlockTypeBlock::read_args(&mut reader,(block_type_unknown,))),
    }
}

fn to_box_dyn<T>(block_result: BinResult<T>) -> Result<Box<dyn Block>, Error>
where T: Block + 'static
{
    block_result.map(|block| -> Box<dyn Block> { Box::new(block) })
}
