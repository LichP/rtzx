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
        PilotWaveform,
        SyncWaveform,
        Waveform,
    },
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct TurboSpeedDataBlock {
    length_pulse_pilot: u16,
    length_pulse_sync_first: u16,
    length_pulse_sync_second: u16,
    length_pulse_zero: u16,
    length_pulse_one: u16,
    length_tone_pilot: u16,
    #[br(temp)]
    #[bw(calc = payload.used_bits)]
    used_bits: u8,
    pause: u16,
    #[br(args(used_bits))]
    payload: DataPayload,
}

impl fmt::Display for TurboSpeedDataBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TurboSpeedDataBlock: {:5} bytes, pause {:5}ms (pilot: {}*{}; sync {}+{}; 0/1: {}/{}; used_bits: {})",
            self.payload.len(),
            self.pause,
            self.length_pulse_pilot,
            self.length_tone_pilot,
            self.length_pulse_sync_first,
            self.length_pulse_sync_second,
            self.length_pulse_zero,
            self.length_pulse_one,
            self.payload.used_bits
        )
    }
}

impl Block for TurboSpeedDataBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::TurboSpeedDataBlock;
    }

    fn get_waveforms(&self, config: Arc<Config>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let pilot_source = PilotWaveform::new(
            config.clone(),
            self.length_pulse_pilot,
            self.length_tone_pilot,
            start_pulse_high,
        );
        let sync_pulses_source = SyncWaveform::new(
            config.clone(),
            self.length_pulse_sync_first,
            self.length_pulse_sync_second,
            if self.length_tone_pilot % 2 == 0 { start_pulse_high } else { !start_pulse_high },
        );
        let data_source = DataWaveform::new(
            config.clone(),
            self.length_pulse_zero,
            self.length_pulse_one,
            self.payload.clone(),
            if self.length_tone_pilot % 2 == 0 { start_pulse_high } else { !start_pulse_high },
        );
        let pause_source = PauseWaveform::new(config.clone(), self.pause, PauseType::Zero);

        return vec![Box::new(pilot_source), Box::new(sync_pulses_source), Box::new(data_source), Box::new(pause_source)];
    }

    fn next_block_start_pulse_high(&self, _config: Arc<Config>, self_start_pulse_high: bool) -> bool {
        if self.pause > 0 {
            return true;
        }

        return if self.length_tone_pilot % 2 == 0 { self_start_pulse_high } else { !self_start_pulse_high };
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
