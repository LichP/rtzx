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
pub struct StandardSpeedDataBlock {
    pause: u16,
    #[bw(try_calc(u16::try_from(data.len())))]
    length: u16,
    #[br(count = length, map = |v: Vec<u8>| Arc::new(v))]
    #[bw(map = |arc: &Arc<Vec<u8>>| &**arc)]
    data: Arc<Vec<u8>>,
}

impl fmt::Display for StandardSpeedDataBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StandardSpeedDataBlock: {} bytes, pause {}ms", self.data.len(), self.pause)
    }
}

impl Block for StandardSpeedDataBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::StandardSpeedDataBlock;
    }

    fn get_waveforms(&self, config: Arc<Config>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let payload = DataPayload::new(8, self.data.len() as u32, self.data.clone());

        let header = payload.data[0] < 128;
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
            start_pulse_high,
        );
        let data_source = DataWaveform::new(
            config.clone(),
            855,
            1710,
            payload.clone(),
            start_pulse_high,
        );
        let pause_source = PauseWaveform::new(config.clone(), self.pause, PauseType::StartLow);

        return vec![Box::new(pilot_source), Box::new(sync_pulses_source), Box::new(data_source), Box::new(pause_source)];
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn ExtendedDisplayCollector) {
        let payload = DataPayload::new(8, self.data.len() as u32, self.data.clone());
        if let Some(payload) = payload.read_payload() {
            out.push(&format!("{}", payload));
        }
    }
}
