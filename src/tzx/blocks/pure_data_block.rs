use binrw::{
    binrw,
};
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Config,
    ExtendedDisplayCollector,
    blocks::{Block, BlockType},
    data::DataPayload,
    waveforms::{
        DataWaveform,
        PauseType,
        PauseWaveform,
        Waveform,
    },
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct PureDataBlock {
    length_pulse_zero: u16,
    length_pulse_one: u16,
    #[br(temp)]
    #[bw(calc = payload.used_bits)]
    used_bits: u8,
    pause: u16,
    #[br(args(used_bits))]
    payload: DataPayload,
}

impl fmt::Display for PureDataBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PureDataBlock: {:5} bytes, pause {:5}ms (0/1: {}/{}; used_bits: {})",
            self.payload.len(),
            self.pause,
            self.length_pulse_zero,
            self.length_pulse_one,
            self.payload.used_bits
        )
    }
}

impl Block for PureDataBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::TurboSpeedDataBlock;
    }

    fn get_waveforms(&self, config: Arc<Config>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let data_source = DataWaveform::new(
            config.clone(),
            self.length_pulse_zero,
            self.length_pulse_one,
            self.payload.clone(),
            start_pulse_high,
        );
        let pause_source = PauseWaveform::new(config.clone(), self.pause, PauseType::StartLow);

        return vec![Box::new(data_source), Box::new(pause_source)];
    }

    fn next_block_start_pulse_high(&self, _config: Arc<Config>, self_start_pulse_high: bool) -> bool {
        return self.pause > 0 || self_start_pulse_high;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn ExtendedDisplayCollector) {
        if let Some(payload) = self.payload.read_payload() {
            out.push(&format!("{}", payload));
        }
    }
}
