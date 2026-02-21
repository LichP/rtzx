use binrw::{
    binrw,
};
use std::any::Any;
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

/// A [Standard Speed Data Block](https://worldofspectrum.net/features/TZXformat.html#STDSPEED).
///
/// This is the default block type for Spectrum loaders, but does also occassionally feature in CDT files.
/// Waveforms are generated as per [TurboSpeedDataBlock](crate::tzx::blocks::TurboSpeedDataBlock) using
/// standard Spectrum timings.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct StandardSpeedDataBlock {
    pub pause: u16,
    #[br(temp)]
    #[bw(try_calc(u16::try_from(payload.len())))]
    length: u16,
    #[br(args(8, length.into()))]
    pub payload: DataPayload,
}

impl StandardSpeedDataBlock {
    pub fn new() -> Self {
        Self {
            pause: 2000,
            payload: DataPayload::default(),
        }
    }
}

impl Default for StandardSpeedDataBlock {
    fn default() -> Self { Self::new() }
}

impl fmt::Display for StandardSpeedDataBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StandardSpeedDataBlock: {} bytes, pause {}ms", self.payload.len(), self.pause)
    }
}

impl Block for StandardSpeedDataBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::StandardSpeedDataBlock;
    }

    fn get_waveforms(&self, config: Arc<Config>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let header = self.payload.data[0] < 128;
        let pilot_source = PilotWaveform::new(
            config.clone(),
            2168,
            if header { 8063 } else { 3223 },
            start_pulse_high,
        );
        let sync_pulses_source = SyncWaveform::new(
            config.clone(),
            667,
            735,
            !start_pulse_high,
        );
        let data_source = DataWaveform::new(
            config.clone(),
            855,
            1710,
            self.payload.clone(),
            !start_pulse_high,
        );
        let pause_source = PauseWaveform::new(config.clone(), self.pause, PauseType::StartLow);

        return vec![Box::new(pilot_source), Box::new(sync_pulses_source), Box::new(data_source), Box::new(pause_source)];
    }

    fn next_block_start_pulse_high(&self, _config: Arc<Config>, _self_start_pulse_high: bool) -> bool { !self.pause > 0 }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn ExtendedDisplayCollector) {
        if let Some(payload) = self.payload.as_payload() {
            out.push(&format!("{}", payload));
        }
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
