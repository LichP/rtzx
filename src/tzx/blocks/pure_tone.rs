use binrw::{
    binrw,
};
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Machine,
    blocks::{Block, BlockType},
    waveforms::{
        PilotWaveform,
        Waveform,
    },
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct PureTone {
    length_pulse: u16,
    length_tone: u16,
}

impl fmt::Display for PureTone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PureTone: {}*{}",
            self.length_pulse,
            self.length_tone,
        )
    }
}

impl Block for PureTone {
    fn r#type(&self) -> BlockType {
        return BlockType::PureTone;
    }

    fn get_waveforms(&self, machine: Arc<Machine>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let pilot_source = PilotWaveform::new(
            machine.clone(),
            self.length_pulse,
            self.length_tone,
            start_pulse_high,
        );

        return vec![Box::new(pilot_source)];
    }

    fn next_block_start_pulse_high(&self, self_start_pulse_high: bool) -> bool {
        return if self.length_tone % 2 == 0 { self_start_pulse_high } else { !self_start_pulse_high };
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
