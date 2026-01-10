use binrw::{
    binrw,
};
use std::fmt;
use::std::sync::Arc;

use crate::tzx::{
    Config,
    blocks::{Block, BlockType},
    waveforms::{
        PauseType,
        PauseWaveform,
        Waveform,
    },
};

/// A [Pause or stop tape command](https://worldofspectrum.net/TZXformat.html#PAUSEBLOCK).
/// At present, the length 0 'stop tape' instruction is not respected (TODO: implement this as an auto-pause
/// in [Player](crate::tzx::Player)).
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct PauseOrStopTapeCommand {
    pause: u16,
}

impl fmt::Display for PauseOrStopTapeCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PauseOrStopTapeCommand: pause {}ms", self.pause)
    }
}

impl Block for PauseOrStopTapeCommand {
    fn r#type(&self) -> BlockType {
        return BlockType::PauseOrStopTapeCommand;
    }

    fn get_waveforms(&self, config: Arc<Config>, _start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let pause_source = PauseWaveform::new(config, self.pause, PauseType::Zero);
        return vec![Box::new(pause_source)];
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

/// A [Stop tape if in 48k mode](https://worldofspectrum.net/TZXformat.html#STOP48K) block.
/// Parsed, but unsupported. Potentially support for this could be added to [Player](crate::tzx::Player)
/// using a configuration option.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct StopTapeIf48K {
    pub length: u32,
    #[br(count = length)]
    pub payload: Vec<u8>
}

impl fmt::Display for StopTapeIf48K {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StopTapeIf48K")
    }
}
impl Block for StopTapeIf48K {
    fn r#type(&self) -> BlockType {
        return BlockType::StopTapeIf48K;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
