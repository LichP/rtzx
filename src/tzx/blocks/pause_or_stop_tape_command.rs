use binrw::{
    binrw,
};
use std::fmt;
use::std::sync::Arc;

use crate::tzx::{
    Config,
    blocks::{Block, BlockType},
    waveforms::{
        PauseWaveform,
        Waveform,
    },
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
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
        let pause_source = PauseWaveform::new(config, self.pause);
        return vec![Box::new(pause_source)];
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct StopTapeIf48K {
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
