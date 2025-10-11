pub mod block_type;
pub mod call;
pub mod direct_recording;
pub mod group;
pub mod jump_to_block;
pub mod r#loop;
pub mod pause_or_stop_tape_command;
pub mod pulse_sequence;
pub mod pure_data_block;
pub mod pure_tone;
pub mod set_signal_level;
pub mod standard_speed_data_block;
pub mod turbo_speed_data_block;
pub mod text_description;

pub use block_type::BlockType;
pub use call::{CallSequence, ReturnFromSequence};
pub use direct_recording::DirectRecording;
pub use group::{GroupStart, GroupEnd};
pub use jump_to_block::JumpToBlock;
pub use r#loop::{LoopStart, LoopEnd};
pub use pause_or_stop_tape_command::{PauseOrStopTapeCommand, StopTapeIf48K};
pub use pulse_sequence::PulseSequence;
pub use pure_data_block::PureDataBlock;
pub use pure_tone::PureTone;
pub use set_signal_level::SetSignalLevel;
pub use standard_speed_data_block::StandardSpeedDataBlock;
pub use turbo_speed_data_block::TurboSpeedDataBlock;
pub use text_description::{TextDescription, MessageBlock};

use crate::tzx::{
    Config,
    waveforms::{EmptyWaveform, Waveform}
};

use binrw::{
    binrw,
    BinRead,
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
}

impl Clone for Box<dyn Block> {
    fn clone(&self) -> Box<dyn Block> {
        self.clone_box()
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
        write!(f, "UnsupportedBlockTypeBlock: {0} ({0:#x})", self.block_type)
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

pub fn read_block(block_type: BlockType, mut reader: impl Read + Seek) -> Result<Box<dyn Block>, Error> {
    return match block_type {
        BlockType::StandardSpeedDataBlock => Ok(Box::new(StandardSpeedDataBlock::read(&mut reader)?)),
        BlockType::TurboSpeedDataBlock => Ok(Box::new(TurboSpeedDataBlock::read(&mut reader)?)),
        BlockType::PureTone => Ok(Box::new(PureTone::read(&mut reader)?)),
        BlockType::PulseSequence => Ok(Box::new(PulseSequence::read(&mut reader)?)),
        BlockType::PureDataBlock => Ok(Box::new(PureDataBlock::read(&mut reader)?)),
        BlockType::DirectRecording => Ok(Box::new(DirectRecording::read(&mut reader)?)),
        BlockType::PauseOrStopTapeCommand => Ok(Box::new(PauseOrStopTapeCommand::read(&mut reader)?)),
        BlockType::GroupStart => Ok(Box::new(GroupStart::read(&mut reader)?)),
        BlockType::GroupEnd => Ok(Box::new(GroupEnd::read(&mut reader)?)),
        BlockType::JumpToBlock => Ok(Box::new(JumpToBlock::read(&mut reader)?)),
        BlockType::TextDescription => Ok(Box::new(TextDescription::read(&mut reader)?)),
        _ => Ok(Box::new(UnsupportedBlockTypeBlock::read_args(&mut reader,(block_type,))?)),
    };
}
