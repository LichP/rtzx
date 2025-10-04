use binrw::{
    binrw,
};
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Machine,
    blocks::{Block, BlockType},
    waveforms::{
        DirectWaveform,
        PauseWaveform,
        Waveform,
    },
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct DirectRecording {
    length_sample: u16,
    pause: u16,
    used_bits: u8,
    #[br(parse_with = binrw::helpers::read_u24)]
    #[bw(write_with = binrw::helpers::write_u24)]
    length: u32,
    #[br(count = length)]
    data: Vec<u8>,
}

impl fmt::Display for DirectRecording {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DirectRecording: {:5} bytes, pause {:5}ms (length_sample: {}; used_bits: {})",
            self.data.len(),
            self.pause,
            self.length_sample,
            self.used_bits
        )
    }
}

impl Block for DirectRecording {
    fn r#type(&self) -> BlockType {
        return BlockType::TurboSpeedDataBlock;
    }

    fn get_waveforms(&self, machine: Arc<Machine>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let direct_source = DirectWaveform::new(
            machine.clone(),
            self.length_sample,
            &self.data,
            self.used_bits,
            start_pulse_high,
        );
        let pause_source = PauseWaveform::new(self.pause);

        return vec![Box::new(direct_source), Box::new(pause_source)];
    }

    fn next_block_start_pulse_high(&self, self_start_pulse_high: bool) -> bool {
        if self.pause > 0 {
            return true;
        }

        let direct_source = DirectWaveform::new(
            Arc::new(Machine::ZXSpectrum),
            self.length_sample,
            &self.data,
            self.used_bits,
            self_start_pulse_high,
        );
        return !direct_source.last_pulse_high();
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
