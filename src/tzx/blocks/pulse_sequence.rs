use binrw::{
    binrw,
};
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Machine,
    blocks::{Block, BlockType},
    waveforms::{
        PulseSequenceWaveform,
        Waveform,
    },
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct PulseSequence {
    length: u8,
    #[br(count = length)]
    pulse_lengths: Vec<u16>,
}

impl fmt::Display for PulseSequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PulseSequence: {} pulses",
            self.length,
        )
    }
}

impl Block for PulseSequence {
    fn r#type(&self) -> BlockType {
        return BlockType::PulseSequence;
    }

    fn get_waveforms(&self, machine: Arc<Machine>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let pulse_sequence_source = PulseSequenceWaveform::new(
            machine.clone(),
            &self.pulse_lengths,
            start_pulse_high,
        );

        return vec![Box::new(pulse_sequence_source)];
    }

    fn next_block_start_pulse_high(&self, self_start_pulse_high: bool) -> bool {
        return if self.length % 2 == 0 { self_start_pulse_high } else { !self_start_pulse_high };
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
