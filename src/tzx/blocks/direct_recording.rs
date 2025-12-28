use binrw::{
    binrw,
};
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Config,
    blocks::{Block, BlockType},
    data::DataPayload,
    waveforms::{
        DirectWaveform,
        PauseType,
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
    #[br(temp)]
    #[bw(calc = payload.used_bits)]
    used_bits: u8,
    #[br(args(used_bits))]
    payload: DataPayload,
}

impl fmt::Display for DirectRecording {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DirectRecording: {:5} bytes, pause {:5}ms (length_sample: {}; used_bits: {})",
            self.payload.data.len(),
            self.pause,
            self.length_sample,
            self.payload.used_bits
        )
    }
}

impl Block for DirectRecording {
    fn r#type(&self) -> BlockType {
        return BlockType::TurboSpeedDataBlock;
    }

    fn get_waveforms(&self, config: Arc<Config>, _start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let direct_source = DirectWaveform::new(
            config.clone(),
            self.length_sample,
            self.payload.clone(),
        );
        let pause_source = PauseWaveform::new(config.clone(), self.pause, PauseType::StartLow);

        return vec![Box::new(direct_source), Box::new(pause_source)];
    }

    fn next_block_start_pulse_high(&self, config: Arc<Config>, _self_start_pulse_high: bool) -> bool {
        if self.pause > 0 {
            return true;
        }

        let direct_source = DirectWaveform::new(
            config.clone(),
            self.length_sample,
            self.payload.clone(),
        );
        return !direct_source.last_pulse_high();
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}
